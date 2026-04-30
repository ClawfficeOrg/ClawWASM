//! ClawWASM embedded engine.
//!
//! This crate is the boundary between the host plugin (`clawasm`) and the
//! WasmEdge runtime. It exposes a small, stable surface — `Engine`,
//! `Instance`, `Output` — that the rest of the project can rely on while
//! the underlying WasmEdge integration evolves.
//!
//! ## Implementation: v0.2.0 (CLI subprocess)
//!
//! The current implementation invokes the `wasmedge` command-line binary
//! as a subprocess and captures its stdout / stderr / exit code. This was
//! chosen over `wasmedge-sys` (the in-process bindings) because the
//! `wasmedge-sys` 0.4.x and 0.17.x lines are both ABI-incompatible with
//! the WasmEdge 0.14.1 release we pin in CI (see
//! `docs/LEARNINGS.md` 2026-04-30). Subprocess invocation:
//!
//! - works against any WasmEdge release that ships a CLI,
//! - has no native build dependency on the consuming crate,
//! - keeps the public API stable for an in-process swap-in later.
//!
//! When/if we move to in-process embedding, only the body of
//! [`Instance::run`] needs to change.
//!
//! ## Probing
//!
//! [`Engine::new`] succeeds even if `wasmedge` is not installed — it does
//! not eagerly probe. [`Instance::run`] is the call that actually
//! executes the module and will return a descriptive error if the
//! `wasmedge` binary is missing or non-executable. Callers that want to
//! fail fast can use [`Engine::probe`].

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default name of the WasmEdge CLI we shell out to. Overridable via
/// the `WASMEDGE_BIN` environment variable so callers can point at a
/// non-PATH install (e.g. `$HOME/.wasmedge/bin/wasmedge`, which is
/// where the official installer lands).
const DEFAULT_WASMEDGE_BIN: &str = "wasmedge";

/// Engine handle. Cheap to construct; holds no native resources.
#[derive(Debug, Clone)]
pub struct Engine {
    wasmedge_bin: PathBuf,
}

/// A loaded module instance. Currently a thin wrapper around the wasm
/// file path; future in-process implementations will hold a real VM
/// handle here without changing the public API.
#[derive(Debug, Clone)]
pub struct Instance {
    engine: Engine,
    module_path: PathBuf,
}

/// Result of running an instance to completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// Captured stdout from the wasm module.
    pub stdout: String,
    /// Captured stderr from the wasm module / runtime.
    pub stderr: String,
    /// Process exit code. `0` is success.
    pub exit_code: i32,
}

impl Output {
    /// Returns `true` if the process exited with code 0.
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

impl Engine {
    /// Construct a new engine using either `$WASMEDGE_BIN` or the
    /// default `wasmedge` binary on `$PATH`. Does not probe; see
    /// [`Engine::probe`].
    pub fn new() -> Result<Self> {
        let bin = std::env::var_os("WASMEDGE_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_WASMEDGE_BIN));
        Ok(Self { wasmedge_bin: bin })
    }

    /// Construct an engine pointing at a specific `wasmedge` binary.
    pub fn with_binary(path: impl Into<PathBuf>) -> Self {
        Self {
            wasmedge_bin: path.into(),
        }
    }

    /// Path to the `wasmedge` binary this engine will invoke.
    pub fn binary(&self) -> &Path {
        &self.wasmedge_bin
    }

    /// Try to invoke `wasmedge --version` and return its stdout.
    /// Returns an error if the binary is missing or not executable.
    pub fn probe(&self) -> Result<String> {
        let out = Command::new(&self.wasmedge_bin)
            .arg("--version")
            .output()
            .with_context(|| {
                format!(
                    "failed to execute `{}` (set WASMEDGE_BIN or install WasmEdge)",
                    self.wasmedge_bin.display()
                )
            })?;
        if !out.status.success() {
            anyhow::bail!(
                "`{} --version` exited with status {}",
                self.wasmedge_bin.display(),
                out.status
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Load a `.wasm` module from disk. Validates that the file exists
    /// and is non-empty; defers actual instantiation to [`Instance::run`].
    pub fn load(&self, path: impl AsRef<Path>) -> Result<Instance> {
        let module_path = path.as_ref().to_path_buf();
        if !module_path.is_file() {
            anyhow::bail!("WASM module not found: {}", module_path.display());
        }
        let meta = std::fs::metadata(&module_path)
            .with_context(|| format!("stat {}", module_path.display()))?;
        if meta.len() == 0 {
            anyhow::bail!("WASM module is empty: {}", module_path.display());
        }
        Ok(Instance {
            engine: self.clone(),
            module_path,
        })
    }
}

impl Instance {
    /// Run the module under WasmEdge, passing `args` as program
    /// arguments to the wasm module. Captures stdout, stderr, and exit
    /// code.
    ///
    /// Errors only on failures to *spawn* the runtime (e.g. missing
    /// binary). A non-zero exit from the wasm module itself is reported
    /// in [`Output::exit_code`] as `Ok`.
    pub fn run(&self, args: &[String]) -> Result<Output> {
        let mut cmd = Command::new(self.engine.binary());
        // `wasmedge <module> [args...]` — the CLI forwards everything
        // after the module path to the guest as `_start` argv.
        cmd.arg(&self.module_path);
        cmd.args(args);

        let out = cmd.output().with_context(|| {
            format!(
                "failed to execute `{} {} {}` (is WasmEdge installed?)",
                self.engine.binary().display(),
                self.module_path.display(),
                args.join(" ")
            )
        })?;

        Ok(Output {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            // `code()` is `None` only when killed by a signal; surface that
            // as -1 (POSIX-ish convention) rather than panicking.
            exit_code: out.status.code().unwrap_or(-1),
        })
    }

    /// Path to the loaded `.wasm` file.
    pub fn module_path(&self) -> &Path {
        &self.module_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_new_picks_up_env_override() {
        // SAFETY: tests run single-threaded per default; this env mutation
        // is local to the test process.
        std::env::set_var("WASMEDGE_BIN", "/nonexistent/wasmedge-test-stub");
        let e = Engine::new().expect("Engine::new");
        assert_eq!(e.binary(), Path::new("/nonexistent/wasmedge-test-stub"));
        std::env::remove_var("WASMEDGE_BIN");
    }

    #[test]
    fn engine_with_binary_holds_path() {
        let e = Engine::with_binary("/opt/wasmedge/bin/wasmedge");
        assert_eq!(e.binary(), Path::new("/opt/wasmedge/bin/wasmedge"));
    }

    #[test]
    fn load_missing_file_fails() {
        let e = Engine::with_binary("/bin/true");
        let err = e.load("/definitely/does/not/exist.wasm").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn load_empty_file_fails() {
        let e = Engine::with_binary("/bin/true");
        let dir = std::env::temp_dir();
        let p = dir.join("clawasm-engine-empty.wasm");
        std::fs::write(&p, b"").unwrap();
        let err = e.load(&p).unwrap_err();
        assert!(err.to_string().contains("empty"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn probe_reports_missing_binary() {
        let e = Engine::with_binary("/nonexistent/wasmedge-xyz");
        let err = e.probe().unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("failed to execute"), "got: {msg}");
    }

    #[test]
    fn output_success_helper() {
        let ok = Output {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        };
        let bad = Output {
            exit_code: 1,
            ..ok.clone()
        };
        assert!(ok.success());
        assert!(!bad.success());
    }
}
