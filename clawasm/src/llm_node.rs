//! Godot 4 binding for LLM inference via llama.cpp.
//!
//! `CLLawM` is a `Node` you drop into a Godot scene. From GDScript:
//!
//! ```text
//! var llm := CLLawM.new()
//! add_child(llm)
//! llm.set_model("res://models/gemma-4-E2B-it-Q4_K_M.gguf")
//! llm.token_generated.connect(func(tok): print(tok, ""))  # no newline
//! llm.inference_done.connect(func(full): print("\n--- done ---"))
//! llm.generate("Why is the sky blue?")
//! ```
//!
//! ## Backend
//!
//! Inference is delegated to the `llama-cli` binary from llama.cpp. The
//! binary must be on `$PATH` or pointed at via `$LLAMA_CLI_BIN`. Download
//! the GGUF model with `scripts/download-model.sh`.
//!
//! ## Streaming
//!
//! Tokens are read byte-by-byte from `llama-cli`'s stdout (via
//! [`clawasm_engine::Runner::spawn_chunked`]) and forwarded as
//! `token_generated` signals on every Godot `_process` tick. The full
//! accumulated response is sent via `inference_done` once the process exits.
//!
//! ## Chat template
//!
//! The Gemma 4 IT chat template is applied automatically in
//! [`engine::LlmConfig::format_prompt`]. Override `system_prompt` to
//! change the model's persona.

use std::path::PathBuf;

use engine::{Event, LlmConfig, Runner};
use godot::classes::ProjectSettings;
use godot::prelude::*;

/// Resolve a possibly-Godot-virtual path (`res://`, `user://`) to an
/// absolute filesystem path. Same logic as [`crate::engine_node`].
fn resolve_path(input: &str) -> PathBuf {
    if input.starts_with("res://") || input.starts_with("user://") {
        let globalised = ProjectSettings::singleton().globalize_path(input);
        PathBuf::from(globalised.to_string())
    } else {
        PathBuf::from(input)
    }
}

/// A Godot `Node` that runs LLM inference via `llama-cli` and streams
/// tokens as Godot signals.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct CLLawM {
    base: Base<Node>,
    /// Path to the GGUF model file.
    model_path: Option<PathBuf>,
    /// Optional override for the `llama-cli` binary path.
    llama_cli_bin: Option<PathBuf>,
    /// System prompt prepended to every generation.
    system_prompt: String,
    /// Max tokens to generate per call.
    n_predict: u32,
    /// KV-cache context window size in tokens.
    ctx_size: u32,
    /// CPU thread count. 0 = let llama-cli decide.
    n_threads: u32,
    /// Active inference subprocess.
    runner: Option<Runner>,
    /// Accumulates all tokens from the current generation.
    accumulated: String,
}

#[godot_api]
impl INode for CLLawM {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            model_path: None,
            llama_cli_bin: None,
            system_prompt: "You are a helpful assistant.".into(),
            n_predict: 512,
            ctx_size: 4096,
            n_threads: 0,
            runner: None,
            accumulated: String::new(),
        }
    }

    /// Drain pending events from the active runner and forward them as
    /// Godot signals. Called every frame by the Godot engine.
    fn process(&mut self, _delta: f64) {
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
                Event::StdoutChunk(chunk) => {
                    self.accumulated.push_str(&chunk);
                    self.signals()
                        .token_generated()
                        .emit(&GString::from(&chunk));
                }
                Event::Stderr(line) => {
                    self.signals()
                        .inference_stderr()
                        .emit(&GString::from(&line));
                }
                Event::Finished(code) => {
                    let full = GString::from(&self.accumulated);
                    self.signals().inference_done().emit(&full, code as i64);
                    self.accumulated.clear();
                    clear_runner = true;
                }
                Event::Failed(msg) => {
                    self.signals().inference_failed().emit(&GString::from(&msg));
                    self.accumulated.clear();
                    clear_runner = true;
                }
                // Line-based stdout is not produced by spawn_chunked;
                // ignore defensively in case Runner is ever swapped.
                Event::Stdout(_) => {}
            }
        }

        if clear_runner {
            self.runner = None;
        }
    }
}

#[godot_api]
impl CLLawM {
    // ── Signals ────────────────────────────────────────────────────────────

    /// Emitted for each chunk of text produced by the model (typically
    /// one or a few tokens at a time). Concatenating all chunks yields
    /// the full response.
    #[signal]
    fn token_generated(token: GString);

    /// Emitted exactly once when inference finishes. `full_text` is the
    /// entire response; `exit_code` is llama-cli's exit code (0 = OK).
    #[signal]
    fn inference_done(full_text: GString, exit_code: i64);

    /// Emitted if the inference subprocess itself failed (e.g. binary not
    /// found). Mutually exclusive with `inference_done`.
    #[signal]
    fn inference_failed(message: GString);

    /// Emitted for each line written to llama-cli's stderr (timing stats,
    /// model load progress, etc.).
    #[signal]
    fn inference_stderr(line: GString);

    // ── Configuration ──────────────────────────────────────────────────────

    /// Set the path to the GGUF model file. Accepts `res://` and
    /// `user://` Godot paths as well as ordinary filesystem paths.
    #[func]
    pub fn set_model(&mut self, path: GString) {
        let resolved = resolve_path(&path.to_string());
        godot_print!("CLLawM: model set to {}", resolved.display());
        self.model_path = Some(resolved);
    }

    /// Currently registered model path. Empty if none.
    #[func]
    pub fn model_path(&self) -> GString {
        self.model_path
            .as_ref()
            .map(|p| GString::from(&p.display().to_string()))
            .unwrap_or_default()
    }

    /// Override the `llama-cli` binary path. Pass an empty string to
    /// clear and fall back to `$LLAMA_CLI_BIN` / `$PATH`.
    #[func]
    pub fn set_llama_cli(&mut self, path: GString) {
        let s = path.to_string();
        self.llama_cli_bin = if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        };
    }

    /// Set the system prompt used for every `generate` call.
    #[func]
    pub fn set_system_prompt(&mut self, prompt: GString) {
        self.system_prompt = prompt.to_string();
    }

    /// Current system prompt.
    #[func]
    pub fn system_prompt(&self) -> GString {
        GString::from(&self.system_prompt)
    }

    /// Maximum tokens to generate per call (default 512).
    #[func]
    pub fn set_n_predict(&mut self, n: i64) {
        self.n_predict = n.max(1) as u32;
    }

    /// KV-cache context size in tokens (default 4096).
    #[func]
    pub fn set_ctx_size(&mut self, n: i64) {
        self.ctx_size = n.max(128) as u32;
    }

    /// CPU thread count. Pass 0 to let llama-cli decide (default).
    #[func]
    pub fn set_n_threads(&mut self, n: i64) {
        self.n_threads = n.max(0) as u32;
    }

    // ── Inference control ──────────────────────────────────────────────────

    /// Begin inference for `prompt`. Returns `true` on successful spawn.
    ///
    /// Emits `token_generated` for each token chunk, then
    /// `inference_done` (or `inference_failed`) when complete. Calling
    /// `generate` while already running is a no-op (returns `false`).
    #[func]
    pub fn generate(&mut self, prompt: GString) -> bool {
        if self.runner.is_some() {
            godot_warn!("CLLawM::generate called while already running; ignoring");
            return false;
        }

        let model = match self.model_path.as_ref() {
            Some(p) => p.clone(),
            None => {
                godot_error!("CLLawM::generate: no model set; call set_model() first");
                return false;
            }
        };

        let mut cfg = LlmConfig::new(&model);
        cfg.system_prompt = self.system_prompt.clone();
        cfg.n_predict = self.n_predict;
        cfg.ctx_size = self.ctx_size;
        if self.n_threads > 0 {
            cfg.n_threads = Some(self.n_threads);
        }
        if let Some(bin) = self.llama_cli_bin.as_ref() {
            cfg.llama_cli_bin = bin.clone();
        }

        match cfg.stream_generate(&prompt.to_string()) {
            Ok(runner) => {
                self.runner = Some(runner);
                true
            }
            Err(e) => {
                godot_error!("CLLawM::generate: spawn failed: {e:#}");
                false
            }
        }
    }

    /// Interrupt the current inference run. Idempotent. A final
    /// `inference_done` signal fires on the next `process` tick once
    /// the subprocess is reaped.
    #[func]
    pub fn stop(&mut self) {
        if let Some(r) = self.runner.as_mut() {
            r.stop();
        }
    }

    /// Returns `true` while inference is running.
    #[func]
    pub fn is_running(&self) -> bool {
        self.runner
            .as_ref()
            .map(Runner::is_running)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_plain_path_unchanged() {
        let p = resolve_path("/tmp/model.gguf");
        assert_eq!(p, PathBuf::from("/tmp/model.gguf"));
    }

    #[test]
    fn resolve_relative_path_unchanged() {
        let p = resolve_path("models/gemma.gguf");
        assert_eq!(p, PathBuf::from("models/gemma.gguf"));
    }
}
