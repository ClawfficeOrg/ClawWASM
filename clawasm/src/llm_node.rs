//! Godot 4 binding for LLM inference via llama.cpp.
//!
//! `CLLawM` is a `Node` you drop into a Godot scene. From GDScript:
//!
//! ```text
//! var llm := CLLawM.new()
//! add_child(llm)
//! llm.set_model("res://models/gemma-4-E2B-it-Q4_K_M.gguf")
//! llm.token_generated.connect(func(tok): print(tok, ""))
//! llm.inference_done.connect(func(full, _code): print("\n--- done ---"))
//! llm.generate("Why is the sky blue?")
//! ```
//!
//! ## Backend
//!
//! When compiled with the `with-llama` Cargo feature, inference is run
//! in-process via the [`llama-cpp-2`] crate, which bakes llama.cpp into
//! the cdylib. Metal GPU acceleration is enabled automatically on macOS
//! (via the cmake build script in `llama-cpp-sys-2`); no extra flag is
//! needed.
//!
//! Without `with-llama`, all methods compile and return safe stubs.
//! `generate` logs an error and returns `false`.
//!
//! ## Threading
//!
//! `Arc<LlamaModel>` is `Send + Sync` and is cached on the main struct.
//! `LlamaContext`, `LlamaBatch`, and `LlamaSampler` are `!Send` and are
//! created fresh inside the inference thread each call. Tokens are sent
//! back to the main thread via an `mpsc::channel` and drained on every
//! `_process` tick.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use godot::classes::ProjectSettings;
use godot::prelude::*;

#[cfg(feature = "with-llama")]
use std::sync::mpsc;

// ── LlmEvent (with-llama only) ────────────────────────────────────────────────

/// Messages forwarded from the inference thread to the Godot main thread.
#[cfg(feature = "with-llama")]
enum LlmEvent {
    /// A decoded token piece (may be multiple bytes).
    Token(String),
    /// Inference completed successfully.
    Done,
    /// Inference aborted with an error message.
    Error(String),
}

// ── InferenceParams (with-llama only) ─────────────────────────────────────────

/// All parameters needed to run one inference call, sent into the thread.
#[cfg(feature = "with-llama")]
struct InferenceParams {
    prompt: String,
    system_prompt: String,
    n_predict: u32,
    ctx_size: u32,
    n_threads: i32,
    temperature: f32,
    top_p: f32,
    top_k: i32,
}

// ── Inference thread (with-llama only) ───────────────────────────────────────

/// Run a complete inference call inside a background thread.
///
/// `model` is an `Arc<LlamaModel>` cloned from the cached field on the
/// Godot node — it is `Send + Sync` so moving it here is safe. The
/// `LlamaBackend`, `LlamaContext`, `LlamaBatch`, and `LlamaSampler` are
/// all `!Send` and are created locally inside this function.
///
/// Returns `Ok(())` after the generation loop finishes normally
/// (including early-stop via `stop_flag`). The caller wraps the return
/// in an `LlmEvent::Done` / `LlmEvent::Error`.
#[cfg(feature = "with-llama")]
fn run_inference(
    model: Arc<llama_cpp_2::model::LlamaModel>,
    stop: Arc<AtomicBool>,
    tx: mpsc::Sender<LlmEvent>,
    params: InferenceParams,
) -> anyhow::Result<()> {
    use llama_cpp_2::{
        context::params::LlamaContextParams, llama_backend::LlamaBackend, llama_batch::LlamaBatch,
        model::AddBos, sampling::LlamaSampler,
    };
    use std::num::NonZeroU32;

    // `LlamaBackend::init()` is idempotent — safe to call from any thread.
    let backend = LlamaBackend::init()?;

    // Build chat messages and apply the model's embedded chat template.
    let sys_msg = llama_cpp_2::LlamaChatMessage::new("system", &params.system_prompt)?;
    let usr_msg = llama_cpp_2::LlamaChatMessage::new("user", &params.prompt)?;
    let tmpl = model.chat_template(None)?;
    // `add_ass = true` appends the assistant turn prefix so the model
    // continues rather than re-generating a role header.
    let formatted = model.apply_chat_template(&tmpl, &[sys_msg, usr_msg], true)?;

    // Tokenise (the template already inserted BOS).
    let prompt_tokens = model.str_to_token(&formatted, AddBos::Never)?;
    let n_prompt = prompt_tokens.len();

    // Create context.
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(params.ctx_size))
        .with_n_threads(params.n_threads)
        .with_n_threads_batch(params.n_threads);
    // `ctx` borrows from both `backend` and `model` (via deref of Arc).
    // Both outlive `ctx` within this function's scope.
    let mut ctx = model.new_context(&backend, ctx_params)?;

    // Decode the prompt in one batch.
    // `add_sequence` with `logits_all = false` enables logits only on the
    // last token, which is what the sampler needs.
    let mut batch = LlamaBatch::new(params.ctx_size as usize, 1);
    batch.add_sequence(&prompt_tokens, 0, false)?;
    ctx.decode(&mut batch)?;

    // Seed the sampler from wall-clock time for non-deterministic output.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_k(params.top_k),
        LlamaSampler::top_p(params.top_p, 1),
        LlamaSampler::temp(params.temperature),
        LlamaSampler::dist(seed),
    ]);

    let mut n_cur = n_prompt as i32;
    let mut decoder = encoding_rs::UTF_8.new_decoder();

    for _ in 0..params.n_predict {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        // Sample the next token from the last batch position.
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }

        let piece = model.token_to_piece(token, &mut decoder, false, None)?;
        // If the receiver is gone (e.g. the node was freed), stop silently.
        if tx.send(LlmEvent::Token(piece)).is_err() {
            break;
        }

        // Advance: one-token batch at the next position with logits enabled.
        batch.clear();
        batch.add(token, n_cur, &[0i32], true)?;
        n_cur += 1;
        ctx.decode(&mut batch)?;
    }

    Ok(())
}

// ── Resolve Godot-virtual paths ───────────────────────────────────────────────

/// Resolve a possibly-Godot-virtual path (`res://`, `user://`) to an
/// absolute filesystem path suitable for native Rust I/O.
fn resolve_path(input: &str) -> PathBuf {
    if input.starts_with("res://") || input.starts_with("user://") {
        let globalised = ProjectSettings::singleton().globalize_path(input);
        PathBuf::from(globalised.to_string())
    } else {
        PathBuf::from(input)
    }
}

// ── CLLawM GodotClass ─────────────────────────────────────────────────────────

/// A Godot `Node` that runs LLM inference via llama.cpp and streams tokens
/// as Godot signals.
///
/// Compile the host plugin with `--features with-llama` to enable actual
/// inference. Without the feature the node is a safe no-op stub.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct CLLawM {
    base: Base<Node>,

    /// Path to the GGUF model file.
    model_path: Option<PathBuf>,
    /// System prompt prepended to every generation.
    system_prompt: String,
    /// Max tokens to generate per call (default 512).
    n_predict: u32,
    /// KV-cache context window size in tokens (default 4096).
    ctx_size: u32,
    /// CPU thread count (default 4).
    n_threads: i32,
    /// Sampling temperature, Gemma 4 recommended 1.0 (default 1.0).
    temperature: f32,
    /// Top-p nucleus sampling, Gemma 4 recommended 0.95 (default 0.95).
    top_p: f32,
    /// Top-k sampling, Gemma 4 recommended 64 (default 64).
    top_k: i32,

    /// Set to `true` to ask the active inference thread to stop early.
    stop_flag: Arc<AtomicBool>,

    // ── with-llama feature-gated fields ──────────────────────────────────────
    /// Full response accumulated token-by-token for `inference_done`.
    #[cfg(feature = "with-llama")]
    accumulated: String,
    /// Receiver for inference events from the background thread.
    #[cfg(feature = "with-llama")]
    rx: Option<mpsc::Receiver<LlmEvent>>,
    /// Loaded model shared with inference threads (Send + Sync).
    #[cfg(feature = "with-llama")]
    model: Option<Arc<llama_cpp_2::model::LlamaModel>>,
}

#[godot_api]
impl INode for CLLawM {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            model_path: None,
            system_prompt: "You are a helpful assistant.".into(),
            n_predict: 512,
            ctx_size: 4096,
            n_threads: 4,
            temperature: 1.0,
            top_p: 0.95,
            top_k: 64,
            stop_flag: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "with-llama")]
            accumulated: String::new(),
            #[cfg(feature = "with-llama")]
            rx: None,
            #[cfg(feature = "with-llama")]
            model: None,
        }
    }

    /// Drain pending inference events and emit Godot signals.
    /// Called every frame by the Godot engine.
    fn process(&mut self, _delta: f64) {
        #[cfg(feature = "with-llama")]
        self.drain_events();
    }
}

// ── Private helpers (with-llama) ──────────────────────────────────────────────

#[cfg(feature = "with-llama")]
impl CLLawM {
    /// Load the model from disk if not already cached.
    ///
    /// Creates a temporary `LlamaBackend` only for the duration of the
    /// load; the model is self-contained once loaded.
    fn ensure_model_loaded(&mut self) -> anyhow::Result<()> {
        use llama_cpp_2::{
            llama_backend::LlamaBackend,
            model::{params::LlamaModelParams, LlamaModel},
        };

        if self.model.is_some() {
            return Ok(());
        }

        let path = self
            .model_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no model path set; call set_model() first"))?;

        let backend = LlamaBackend::init()?;
        let model = LlamaModel::load_from_file(&backend, path, &LlamaModelParams::default())?;
        godot_print!(
            "CLLawM: loaded model ({} params) from {}",
            model.n_params(),
            path.display()
        );
        self.model = Some(Arc::new(model));
        Ok(())
    }

    /// Spawn the inference thread and wire up the event channel.
    fn start_inference(&mut self, prompt: String) -> anyhow::Result<()> {
        let model = self
            .model
            .as_ref()
            // PANIC: `ensure_model_loaded` must be called before this.
            .expect("PANIC: start_inference called without a loaded model")
            .clone();

        // Fresh stop flag for this generation run.
        let stop_flag = Arc::new(AtomicBool::new(false));
        self.stop_flag = Arc::clone(&stop_flag);

        let params = InferenceParams {
            prompt,
            system_prompt: self.system_prompt.clone(),
            n_predict: self.n_predict,
            ctx_size: self.ctx_size,
            n_threads: self.n_threads,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
        };

        let (tx, rx) = mpsc::channel::<LlmEvent>();
        self.rx = Some(rx);

        std::thread::Builder::new()
            .name("clawasm-llm-inference".into())
            .spawn(
                move || match run_inference(model, stop_flag, tx.clone(), params) {
                    Ok(()) => {
                        let _ = tx.send(LlmEvent::Done);
                    }
                    Err(e) => {
                        let _ = tx.send(LlmEvent::Error(e.to_string()));
                    }
                },
            )?;

        Ok(())
    }

    /// Drain all pending events from the inference channel without blocking,
    /// emit the corresponding Godot signals, and clear the receiver when done.
    fn drain_events(&mut self) {
        let events: Vec<LlmEvent> = match self.rx.as_ref() {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };

        let mut finished = false;
        for ev in events {
            match ev {
                LlmEvent::Token(piece) => {
                    self.accumulated.push_str(&piece);
                    self.signals()
                        .token_generated()
                        .emit(&GString::from(&piece));
                }
                LlmEvent::Done => {
                    let full = GString::from(&self.accumulated);
                    self.signals().inference_done().emit(&full, 0i64);
                    self.accumulated.clear();
                    finished = true;
                }
                LlmEvent::Error(msg) => {
                    self.signals().inference_failed().emit(&GString::from(&msg));
                    self.accumulated.clear();
                    finished = true;
                }
            }
        }

        if finished {
            self.rx = None;
        }
    }

    /// Inner implementation of `generate` when compiled with `with-llama`.
    fn do_generate(&mut self, prompt: String) -> bool {
        if self.rx.is_some() {
            godot_warn!("CLLawM::generate called while already running; ignoring");
            return false;
        }
        if let Err(e) = self.ensure_model_loaded() {
            godot_error!("CLLawM::generate: failed to load model: {e:#}");
            return false;
        }
        match self.start_inference(prompt) {
            Ok(()) => true,
            Err(e) => {
                godot_error!("CLLawM::generate: failed to start inference: {e:#}");
                false
            }
        }
    }
}

// ── Stub helpers (no with-llama) ──────────────────────────────────────────────

#[cfg(not(feature = "with-llama"))]
impl CLLawM {
    /// Stub: logs a clear error and returns false.
    fn do_generate(&mut self, _prompt: String) -> bool {
        godot_error!(
            "CLLawM::generate: node was compiled without the `with-llama` feature; \
             recompile clawasm with `--features with-llama` to enable inference"
        );
        false
    }
}

// ── Godot API ─────────────────────────────────────────────────────────────────

#[godot_api]
impl CLLawM {
    // ── Signals ──────────────────────────────────────────────────────────────

    /// Emitted for each token piece produced by the model. Concatenating
    /// all pieces yields the full response.
    #[signal]
    fn token_generated(token: GString);

    /// Emitted once when inference finishes. `full_text` is the entire
    /// response; `exit_code` is 0 on success.
    #[signal]
    fn inference_done(full_text: GString, exit_code: i64);

    /// Emitted if inference fails (e.g. model load error, OOM).
    /// Mutually exclusive with `inference_done` for a given call.
    #[signal]
    fn inference_failed(message: GString);

    // ── Configuration ─────────────────────────────────────────────────────────

    /// Set the path to the GGUF model file. Accepts `res://` and
    /// `user://` Godot paths as well as regular filesystem paths.
    /// Clears the cached model so it is reloaded on the next `generate`.
    #[func]
    pub fn set_model(&mut self, path: GString) {
        let resolved = resolve_path(&path.to_string());
        godot_print!("CLLawM: model path set to {}", resolved.display());
        self.model_path = Some(resolved);
        // Invalidate cached model so it is re-loaded from the new path.
        #[cfg(feature = "with-llama")]
        {
            self.model = None;
        }
    }

    /// Currently registered model path. Empty string if none.
    #[func]
    pub fn model_path(&self) -> GString {
        self.model_path
            .as_ref()
            .map(|p| GString::from(&p.display().to_string()))
            .unwrap_or_default()
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

    /// Maximum tokens to generate per call (default 512; minimum 1).
    #[func]
    pub fn set_n_predict(&mut self, n: i64) {
        self.n_predict = n.max(1) as u32;
    }

    /// KV-cache context size in tokens (default 4096; minimum 128).
    #[func]
    pub fn set_ctx_size(&mut self, n: i64) {
        self.ctx_size = n.max(128) as u32;
    }

    /// CPU thread count (default 4; minimum 1).
    #[func]
    pub fn set_n_threads(&mut self, n: i64) {
        self.n_threads = n.max(1) as i32;
    }

    /// Sampling temperature (default 1.0; Gemma 4 recommended 1.0).
    #[func]
    pub fn set_temperature(&mut self, v: f64) {
        self.temperature = v as f32;
    }

    /// Top-p nucleus sampling (default 0.95; Gemma 4 recommended 0.95).
    #[func]
    pub fn set_top_p(&mut self, v: f64) {
        self.top_p = v as f32;
    }

    /// Top-k sampling (default 64; Gemma 4 recommended 64).
    #[func]
    pub fn set_top_k(&mut self, k: i64) {
        self.top_k = k as i32;
    }

    // ── Inference control ─────────────────────────────────────────────────────

    /// Begin inference for `prompt`. Returns `true` when inference is
    /// successfully started.
    ///
    /// Emits `token_generated` for each token, then `inference_done` (or
    /// `inference_failed`) when complete. Calling `generate` while already
    /// running is a no-op that returns `false`.
    #[func]
    pub fn generate(&mut self, prompt: GString) -> bool {
        self.do_generate(prompt.to_string())
    }

    /// Request early termination of the active inference run. Idempotent.
    /// The inference thread checks the flag between tokens and exits
    /// cleanly; a final `inference_done` signal fires on the next tick.
    #[func]
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /// Returns `true` while inference is running.
    #[func]
    pub fn is_running(&self) -> bool {
        #[cfg(feature = "with-llama")]
        return self.rx.is_some();
        #[cfg(not(feature = "with-llama"))]
        return false;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

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
