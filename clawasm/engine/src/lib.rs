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

pub mod stream;
pub use stream::{Event, Runner};

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

    /// Spawn the module under WasmEdge and return a streaming
    /// [`Runner`] whose [`Event`]s can be polled line-by-line. Used by
    /// the Godot `ClawEngine` node so stdout can be surfaced as
    /// signals during execution rather than only at completion.
    ///
    /// Errors only on failure to *spawn* the runtime. Non-zero exit of
    /// the wasm module surfaces as `Event::Finished(code)`.
    pub fn stream(&self, args: &[String]) -> Result<Runner> {
        let mut cmd = Command::new(self.engine.binary());
        cmd.arg(&self.module_path);
        cmd.args(args);
        Runner::spawn(cmd)
    }
}

// ── LLM inference via llama-cli ────────────────────────────────────────────

/// Name / path of the `llama-cli` binary. Overridable via `LLAMA_CLI_BIN`.
const DEFAULT_LLAMA_CLI_BIN: &str = "llama-cli";

/// Configuration for a single LLM inference run against a GGUF model.
///
/// Wraps a `llama-cli` subprocess (from llama.cpp). The CLI must be on
/// `$PATH` or pointed at via the `LLAMA_CLI_BIN` environment variable.
/// Tokens are streamed via [`Runner::spawn_chunked`] so callers receive
/// output as it is generated rather than waiting for completion.
///
/// ## Gemma 4 E2B-IT defaults
///
/// [`LlmConfig::new`] pre-populates sampling parameters from the
/// Gemma 4 model card (temp 1.0, top-p 0.95, top-k 64).
///
/// ## Example
///
/// ```no_run
/// use engine::LlmConfig;
/// let cfg = LlmConfig::new("/models/gemma-4-E2B-it-Q4_K_M.gguf");
/// let runner = cfg.stream_generate("Tell me a joke.").unwrap();
/// while let Some(ev) = runner.recv_blocking() {
///     println!("{ev:?}");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Path to the `.gguf` model file.
    pub model_path: PathBuf,
    /// Path to the `llama-cli` binary. Falls back to `$LLAMA_CLI_BIN`
    /// then `"llama-cli"` on `$PATH`.
    pub llama_cli_bin: PathBuf,
    /// System prompt injected at the start of every conversation.
    pub system_prompt: String,
    /// Maximum number of tokens to predict (`-n`). Default: 512.
    pub n_predict: u32,
    /// KV-cache context size in tokens (`-c`). Default: 4096.
    pub ctx_size: u32,
    /// Number of CPU threads (`-t`). `None` lets llama-cli decide.
    pub n_threads: Option<u32>,
    /// Sampling temperature. Gemma 4 recommended: 1.0.
    pub temperature: f32,
    /// Top-p nucleus sampling. Gemma 4 recommended: 0.95.
    pub top_p: f32,
    /// Top-k sampling. Gemma 4 recommended: 64.
    pub top_k: u32,
}

impl LlmConfig {
    /// Construct a config with Gemma 4 recommended defaults.
    /// Override fields directly before calling [`Self::stream_generate`].
    pub fn new(model_path: impl Into<PathBuf>) -> Self {
        let llama_cli_bin = std::env::var_os("LLAMA_CLI_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LLAMA_CLI_BIN));
        Self {
            model_path: model_path.into(),
            llama_cli_bin,
            system_prompt: "You are a helpful assistant.".into(),
            n_predict: 512,
            ctx_size: 4096,
            n_threads: None,
            temperature: 1.0,
            top_p: 0.95,
            top_k: 64,
        }
    }

    /// Format `prompt` into the Gemma 4 instruction-tuned chat template.
    ///
    /// Template (from bartowski/google_gemma-4-E2B-it-GGUF):
    /// ```text
    /// <bos><|turn>system\n{system}\<turn|>\n<|turn>user\n{user}<turn|>\n<|turn>model\n
    /// ```
    pub fn format_prompt(&self, prompt: &str) -> String {
        format!(
            "<bos><|turn>system\n{}<turn|>\n<|turn>user\n{}<turn|>\n<|turn>model\n",
            self.system_prompt, prompt
        )
    }

    /// Spawn `llama-cli` with this config and `prompt`, returning a
    /// [`Runner`] whose [`Event::StdoutChunk`]s carry individual tokens
    /// as they are generated.
    ///
    /// Errors only on failure to *spawn* the binary. A non-zero exit
    /// from llama-cli is surfaced as `Event::Finished(code)`.
    pub fn stream_generate(&self, prompt: &str) -> Result<Runner> {
        if !self.model_path.is_file() {
            anyhow::bail!(
                "GGUF model not found: {} (download with scripts/download-model.sh)",
                self.model_path.display()
            );
        }

        let formatted = self.format_prompt(prompt);

        let mut cmd = Command::new(&self.llama_cli_bin);
        cmd.arg("--model").arg(&self.model_path);
        cmd.arg("-p").arg(&formatted);
        cmd.arg("-n").arg(self.n_predict.to_string());
        cmd.arg("-c").arg(self.ctx_size.to_string());
        cmd.arg("--temp").arg(self.temperature.to_string());
        cmd.arg("--top-p").arg(self.top_p.to_string());
        cmd.arg("--top-k").arg(self.top_k.to_string());
        // Suppress echoing the prompt back to stdout.
        cmd.arg("--no-display-prompt");
        if let Some(t) = self.n_threads {
            cmd.arg("-t").arg(t.to_string());
        }

        Runner::spawn_chunked(cmd)
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
    fn llmconfig_new_picks_up_env_override() {
        std::env::set_var("LLAMA_CLI_BIN", "/custom/llama-cli");
        let cfg = LlmConfig::new("/models/test.gguf");
        assert_eq!(cfg.llama_cli_bin, PathBuf::from("/custom/llama-cli"));
        std::env::remove_var("LLAMA_CLI_BIN");
    }

    #[test]
    fn llmconfig_format_prompt_gemma4_template() {
        let cfg = LlmConfig {
            system_prompt: "You are a pirate.".into(),
            ..LlmConfig::new("/models/test.gguf")
        };
        let out = cfg.format_prompt("What is your name?");
        assert!(
            out.starts_with("<bos><|turn>system\n"),
            "missing bos/system: {out}"
        );
        assert!(
            out.contains("You are a pirate."),
            "missing system prompt: {out}"
        );
        assert!(
            out.contains("<|turn>user\nWhat is your name?<turn|>"),
            "missing user turn: {out}"
        );
        assert!(out.ends_with("<|turn>model\n"), "missing model turn: {out}");
    }

    #[test]
    fn llmconfig_missing_model_errors() {
        let cfg = LlmConfig::new("/definitely/does/not/exist.gguf");
        match cfg.stream_generate("hello") {
            Err(e) => assert!(e.to_string().contains("not found"), "got: {e}"),
            Ok(_) => panic!("expected an error for missing model"),
        }
    }

    #[test]
    fn llmconfig_defaults_match_gemma4_recommendations() {
        let cfg = LlmConfig::new("/models/test.gguf");
        assert_eq!(cfg.temperature, 1.0);
        assert_eq!(cfg.top_p, 0.95);
        assert_eq!(cfg.top_k, 64);
        assert_eq!(cfg.n_predict, 512);
        assert_eq!(cfg.ctx_size, 4096);
        assert!(cfg.n_threads.is_none());
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
