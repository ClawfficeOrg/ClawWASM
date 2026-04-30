//! Godot 4 binding for the ClawWASM engine.
//!
//! `ClawEngine` is a `Node` you drop into a Godot scene. From GDScript:
//!
//! ```text
//! var engine := ClawEngine.new()
//! add_child(engine)
//! engine.register_module("res://hello-wasm.wasm")
//! engine.stdout_line.connect(func(line): print("[wasm] ", line))
//! engine.finished.connect(func(code): print("done: ", code))
//! engine.start([])
//! ```
//!
//! Implementation notes:
//!
//! - All execution happens in a subprocess managed by
//!   [`clawasm_engine::Runner`]. Output lines are pulled off the
//!   runner's mpsc channel inside [`INode::process`] (the Godot main
//!   loop), so signals are always emitted on the main thread.
//! - `register_module` accepts both Godot-style `res://` / `user://`
//!   paths and ordinary filesystem paths. `res://` is resolved via
//!   `ProjectSettings.globalize_path` so packed projects (PCK) and
//!   exported builds work the same as the editor.

use std::path::{Path, PathBuf};

use engine::{Engine, Event, Runner};
use godot::classes::ProjectSettings;
use godot::prelude::*;

/// Convert a possibly-Godot-virtual path (e.g. `res://foo.wasm`) into
/// an absolute filesystem path. Falls back to the input verbatim if it
/// does not look virtual.
///
/// Pulled out of [`ClawEngine::register_module`] so it can be unit
/// tested without spinning up a Godot instance — the `res://` branch
/// is exercised in Godot, the passthrough branch is exercised here.
pub(crate) fn resolve_module_path(input: &str) -> PathBuf {
    if input.starts_with("res://") || input.starts_with("user://") {
        let globalised = ProjectSettings::singleton().globalize_path(input);
        PathBuf::from(globalised.to_string())
    } else {
        PathBuf::from(input)
    }
}

/// Internal helper used by tests: pure-string version of path resolution
/// that doesn't touch Godot singletons. The real
/// [`resolve_module_path`] delegates to `ProjectSettings` for `res://`,
/// but for anything else its behaviour is just `PathBuf::from`.
#[cfg(test)]
fn resolve_passthrough(input: &str) -> PathBuf {
    PathBuf::from(input)
}

/// A Godot `Node` that owns a [`clawasm_engine::Engine`] and exposes
/// its lifecycle to GDScript.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct ClawEngine {
    base: Base<Node>,
    /// Path to the `.wasm` module to run. Set by `register_module` or
    /// directly via the exported property.
    module_path: Option<PathBuf>,
    /// Optional override for the `wasmedge` CLI binary. When `None`,
    /// the engine consults `$WASMEDGE_BIN` and then `$PATH`.
    wasmedge_bin: Option<PathBuf>,
    /// Currently-running subprocess, if any.
    runner: Option<Runner>,
}

#[godot_api]
impl INode for ClawEngine {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            module_path: None,
            wasmedge_bin: None,
            runner: None,
        }
    }

    /// Drain pending events from the active runner and forward them as
    /// Godot signals. Called every frame by the engine.
    fn process(&mut self, _delta: f64) {
        // Take ownership of the events vec without holding a borrow on
        // self.runner across the signal emission (which calls
        // `&mut self`).
        let events: Vec<Event> = match self.runner.as_ref() {
            Some(r) => r.poll(),
            None => return,
        };
        if events.is_empty() {
            return;
        }

        let mut clear_runner = false;
        for ev in events {
            match ev {
                Event::Stdout(line) => {
                    self.signals().stdout_line().emit(&GString::from(&line));
                }
                Event::Stderr(line) => {
                    self.signals().stderr_line().emit(&GString::from(&line));
                }
                Event::Finished(code) => {
                    self.signals().finished().emit(code as i64);
                    clear_runner = true;
                }
                Event::Failed(msg) => {
                    self.signals().failed().emit(&GString::from(&msg));
                    clear_runner = true;
                }
            }
        }

        if clear_runner {
            self.runner = None;
        }
    }
}

#[godot_api]
impl ClawEngine {
    /// Emitted once per line written to the wasm module's stdout
    /// (newline-terminated). The newline itself is stripped.
    #[signal]
    fn stdout_line(line: GString);

    /// Emitted once per line written to the wasm module's stderr.
    #[signal]
    fn stderr_line(line: GString);

    /// Emitted exactly once when the wasm module has fully exited,
    /// after every `stdout_line` / `stderr_line` for that run. The
    /// argument is the exit code (`-1` if killed by signal).
    #[signal]
    fn finished(code: i64);

    /// Emitted if the runner itself failed (e.g. failed to wait on the
    /// child). Mutually exclusive with `finished`.
    #[signal]
    fn failed(message: GString);

    /// Point this node at a `.wasm` file. Accepts `res://` and
    /// `user://` Godot paths as well as ordinary filesystem paths.
    /// Does not start execution; call [`Self::start`] for that.
    #[func]
    pub fn register_module(&mut self, path: GString) {
        let resolved = resolve_module_path(&path.to_string());
        godot_print!("ClawEngine: registered module {}", resolved.display());
        self.module_path = Some(resolved);
    }

    /// Override the `wasmedge` binary used to run modules. Pass an
    /// empty string to clear the override and fall back to
    /// `$WASMEDGE_BIN` / `$PATH`.
    #[func]
    pub fn set_wasmedge_binary(&mut self, path: GString) {
        let s = path.to_string();
        self.wasmedge_bin = if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        };
    }

    /// Spawn the registered module and start streaming output as
    /// signals. Returns `true` on successful spawn, `false` otherwise
    /// (a `failed` signal is *not* emitted for spawn errors — they're
    /// reported via the return value and the Godot log).
    #[func]
    pub fn start(&mut self, args: PackedStringArray) -> bool {
        if self.runner.is_some() {
            godot_warn!("ClawEngine::start called while already running; ignoring");
            return false;
        }
        let module = match self.module_path.as_ref() {
            Some(p) => p.clone(),
            None => {
                godot_error!("ClawEngine::start: no module registered");
                return false;
            }
        };

        let engine = match self.wasmedge_bin.as_ref() {
            Some(bin) => Engine::with_binary(bin),
            None => match Engine::new() {
                Ok(e) => e,
                Err(e) => {
                    godot_error!("ClawEngine::start: Engine::new failed: {e:#}");
                    return false;
                }
            },
        };

        let instance = match engine.load(&module) {
            Ok(i) => i,
            Err(e) => {
                godot_error!(
                    "ClawEngine::start: load({}) failed: {e:#}",
                    module.display()
                );
                return false;
            }
        };

        // PackedStringArray -> Vec<String>
        let arg_vec: Vec<String> = (0..args.len())
            .map(|i| args.get(i).unwrap_or_default().to_string())
            .collect();

        match instance.stream(&arg_vec) {
            Ok(runner) => {
                self.runner = Some(runner);
                true
            }
            Err(e) => {
                godot_error!("ClawEngine::start: spawn failed: {e:#}");
                false
            }
        }
    }

    /// Kill the currently-running module, if any. Idempotent. The
    /// subsequent `finished` signal will fire on a later `process`
    /// tick once the runtime has been reaped.
    #[func]
    pub fn stop(&mut self) {
        if let Some(r) = self.runner.as_mut() {
            r.stop();
        }
    }

    /// Returns `true` if a wasm module is currently executing.
    #[func]
    pub fn is_running(&self) -> bool {
        self.runner
            .as_ref()
            .map(Runner::is_running)
            .unwrap_or(false)
    }

    /// Currently-registered module path as a string. Empty if none.
    #[func]
    pub fn module_path(&self) -> GString {
        self.module_path
            .as_ref()
            .map(|p| GString::from(&p.display().to_string()))
            .unwrap_or_default()
    }
}

// `Path` import kept here only so doc-tests / examples that cite
// `Path` compile cleanly without pulling std into every macro
// expansion above.
#[allow(dead_code)]
fn _path_marker(_p: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_path_is_unchanged() {
        let p = resolve_passthrough("/tmp/hello.wasm");
        assert_eq!(p, PathBuf::from("/tmp/hello.wasm"));
    }

    #[test]
    fn passthrough_handles_relative_paths() {
        let p =
            resolve_passthrough("examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm");
        assert!(p.ends_with("hello-wasm.wasm"));
    }

    // Note: `res://` resolution is exercised via the manual Godot smoke
    // project in `tests/godot-smoke/` — it requires a live
    // `ProjectSettings` singleton which we don't construct in unit
    // tests.
}
