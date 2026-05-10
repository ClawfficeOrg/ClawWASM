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
    /// Diagnostic log line from the inference thread; forwarded to
    /// godot_print! on the main thread so it appears in the Godot console.
    Log(String),
}

// ── InferenceParams (with-llama only) ─────────────────────────────────────────

/// Stop strings that terminate generation even if the model doesn't mark them
/// as EOG tokens (older llama.cpp builds may not do so for all models).
#[cfg(feature = "with-llama")]
const STOP_STRINGS: &[&str] = &[
    "<end_of_turn>", // Gemma
    "<eos>",         // generic
    "<|endoftext|>", // GPT-style
    "[/INST]",       // Llama-2 / Mistral instruct
];

/// All parameters needed to run one inference call, sent into the thread.
#[cfg(feature = "with-llama")]
struct InferenceParams {
    /// User message. When `raw = false` this is wrapped in the chat template;
    /// when `raw = true` this is used verbatim (caller already formatted it).
    prompt: String,
    system_prompt: String,
    /// If true, skip template application and tokenise `prompt` as-is with
    /// `AddBos::Always`. Used for multi-turn history where GDScript builds
    /// the full Gemma-4 IT prompt and passes it pre-formatted.
    raw: bool,
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
    use anyhow::Context as _;
    use llama_cpp_2::{
        context::params::LlamaContextParams,
        llama_backend::LlamaBackend,
        llama_batch::LlamaBatch,
        model::{AddBos, LlamaChatMessage},
        sampling::LlamaSampler,
    };
    use std::num::NonZeroU32;

    // Helper: send a diagnostic log via the channel AND print to stderr so
    // it is visible even if the Godot main thread isn't draining yet.
    macro_rules! tlog {
        ($($arg:tt)*) => {{
            let _msg = format!($($arg)*);
            eprintln!("[CLLawM] {}", _msg);
            let _ = tx.send(LlmEvent::Log(_msg));
        }};
    }

    tlog!(
        "run_inference start | raw={} n_predict={} ctx={} threads={} temp={} top_p={} top_k={}",
        params.raw,
        params.n_predict,
        params.ctx_size,
        params.n_threads,
        params.temperature,
        params.top_p,
        params.top_k
    );

    // `LlamaBackend::init()` is idempotent — safe to call from any thread.
    tlog!("step 1/8: LlamaBackend::init");
    let backend = LlamaBackend::init().context("LlamaBackend::init failed")?;
    tlog!("step 1/8: backend OK");

    // Destructure params so we can move the Strings directly into
    // LlamaChatMessage::new without cloning (it takes owned Strings).
    let InferenceParams {
        prompt,
        system_prompt,
        raw,
        n_predict,
        ctx_size,
        n_threads,
        temperature,
        top_p,
        top_k,
    } = params;

    // ── Step 2/8: format prompt ────────────────────────────────────────────────
    //
    // Two paths:
    //  raw=true  — prompt is already fully formatted by the caller (multi-turn
    //              history case from GDScript). Use it verbatim with AddBos::Always.
    //  raw=false — single-turn: try apply_chat_template; fall back to hand-written
    //              Gemma-4 IT formatter if the bundled Jinja2 engine is too old.
    let (formatted, add_bos) = if raw {
        tlog!(
            "step 2/8: raw mode — using pre-formatted prompt ({} bytes)",
            prompt.len()
        );
        (prompt, AddBos::Always)
    } else {
        // Clone before moving into LlamaChatMessage (which takes ownership).
        let sys_raw = system_prompt.clone();
        let usr_raw = prompt.clone();
        tlog!("step 2/8: building chat messages for template path");
        let sys_msg = LlamaChatMessage::new("system".to_string(), system_prompt)
            .context("LlamaChatMessage::new for 'system' role failed")?;
        let usr_msg = LlamaChatMessage::new("user".to_string(), prompt)
            .context("LlamaChatMessage::new for 'user' role failed")?;
        // Older bundled Jinja2 can't parse Gemma-4's template — fall back to
        // a hand-written Gemma-4 IT format if apply_chat_template returns -1.
        tlog!("step 2/8: calling apply_chat_template");
        let tmpl_result: anyhow::Result<_> = model
            .chat_template(None)
            .context("model.chat_template() failed")
            .and_then(|tmpl| {
                model
                    .apply_chat_template(&tmpl, &[sys_msg, usr_msg], true)
                    .context("model.apply_chat_template() failed")
            });
        match tmpl_result {
            Ok(s) => {
                tlog!("step 2/8: template OK ({} bytes)", s.len());
                (s, AddBos::Never)
            }
            Err(e) => {
                tlog!(
                    "step 2/8: apply_chat_template failed ({}) -- using Gemma-4 IT fallback",
                    e
                );
                let fallback = format!(
                    "<start_of_turn>system\n{sys}\n<end_of_turn>\n\
                     <start_of_turn>user\n{usr}\n<end_of_turn>\n\
                     <start_of_turn>model\n",
                    sys = sys_raw,
                    usr = usr_raw,
                );
                tlog!("step 2/8: fallback prompt ({} bytes)", fallback.len());
                (fallback, AddBos::Always)
            }
        }
    };

    // Tokenise.
    tlog!("step 3/8: tokenising prompt");
    let prompt_tokens = model
        .str_to_token(&formatted, add_bos)
        .context("model.str_to_token() failed")?;
    let n_prompt = prompt_tokens.len();
    tlog!("step 3/8: {} tokens", n_prompt);

    // Create context.
    tlog!(
        "step 4/8: creating LlamaContext (ctx_size={} n_threads={})",
        ctx_size,
        n_threads
    );
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(ctx_size))
        .with_n_threads(n_threads)
        .with_n_threads_batch(n_threads);
    // `ctx` borrows from both `backend` and `model` (via deref of Arc).
    // Both outlive `ctx` within this function's scope.
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .context("model.new_context() failed — ctx too large or llama.cpp ABI mismatch")?;
    tlog!("step 4/8: context OK");

    // Decode the prompt in one batch.
    // `add_sequence` with `logits_all = false` enables logits only on the
    // last token, which is what the sampler needs.
    tlog!("step 5/8: filling prompt batch");
    let mut batch = LlamaBatch::new(ctx_size as usize, 1);
    batch
        .add_sequence(&prompt_tokens, 0, false)
        .context("LlamaBatch::add_sequence failed")?;
    tlog!(
        "step 6/8: decoding prompt batch ({} tokens) — most common failure point for new model arches",
        n_prompt
    );
    ctx.decode(&mut batch).context(
        "ctx.decode(prompt batch) failed — llama_decode returned non-zero; \
         bundled llama.cpp is likely too old for this model architecture",
    )?;
    tlog!("step 6/8: prompt decode OK");

    // Seed the sampler from wall-clock time for non-deterministic output.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_k(top_k),
        LlamaSampler::top_p(top_p, 1),
        LlamaSampler::temp(temperature),
        LlamaSampler::dist(seed),
    ]);
    tlog!("step 7/8: sampler chain created (seed={})", seed);

    let mut n_cur = n_prompt as i32;
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut n_generated: u32 = 0;

    tlog!("step 8/8: generation loop (max {} tokens)", n_predict);
    for _ in 0..n_predict {
        if stop.load(Ordering::Relaxed) {
            tlog!(
                "generation: stop flag set — exiting early at token {}",
                n_generated
            );
            break;
        }

        // Sample the next token from the last batch position.
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            tlog!("generation: EOG at position {} — done", n_cur);
            break;
        }

        let piece = model
            .token_to_piece(token, &mut decoder, false, None)
            .with_context(|| {
                format!("token_to_piece failed at token {} (id={})", n_cur, token.0)
            })?;

        // Check for stop strings.  Older llama.cpp may not register special
        // tokens like `<end_of_turn>` as EOG tokens, so they surface as text.
        // Strip the stop string and any content after it, then halt cleanly.
        let stop_hit = STOP_STRINGS.iter().find(|&&s| piece.contains(s));
        if let Some(&stop_str) = stop_hit {
            // Emit content that came before the stop string (if any).
            let before = piece.split(stop_str).next().unwrap_or("").trim_end();
            if !before.is_empty() {
                if tx.send(LlmEvent::Token(before.to_string())).is_err() {
                    break;
                }
                n_generated += 1;
            }
            tlog!(
                "generation: stop string {:?} found at token {} — done",
                stop_str,
                n_cur
            );
            break;
        }

        // If the receiver is gone (e.g. the node was freed), stop silently.
        if tx.send(LlmEvent::Token(piece)).is_err() {
            eprintln!("[CLLawM] receiver gone — stopping generation");
            break;
        }
        n_generated += 1;

        // Advance: one-token batch at the next position with logits enabled.
        batch.clear();
        batch
            .add(token, n_cur, &[0i32], true)
            .with_context(|| format!("LlamaBatch::add failed at position {}", n_cur))?;
        n_cur += 1;
        ctx.decode(&mut batch).with_context(|| {
            format!(
                "ctx.decode(token batch) failed at token #{} position {} — llama_decode non-zero",
                n_generated, n_cur
            )
        })?;
    }

    tlog!("generation complete — {} tokens emitted", n_generated);
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
        use anyhow::Context as _;
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

        godot_print!("CLLawM: loading model from {}", path.display());
        godot_print!(
            "CLLawM: llama-cpp-2 v{} (bundled llama.cpp may not support newest model arches)",
            env!("CARGO_PKG_VERSION")
        );

        let backend =
            LlamaBackend::init().context("LlamaBackend::init failed during model load")?;
        let model = LlamaModel::load_from_file(&backend, path, &LlamaModelParams::default())
            .with_context(|| format!("LlamaModel::load_from_file failed for {:?}", path))?;

        godot_print!(
            "CLLawM: model loaded — {} params | {} vocab tokens | {} layers",
            model.n_params(),
            model.n_vocab(),
            model.n_layer(),
        );
        self.model = Some(Arc::new(model));
        Ok(())
    }

    /// Spawn the inference thread and wire up the event channel.
    fn start_inference(&mut self, prompt: String) -> anyhow::Result<()> {
        use anyhow::Context as _;

        let model = self
            .model
            .as_ref()
            // PANIC: `ensure_model_loaded` must be called before this.
            .expect("PANIC: start_inference called without a loaded model")
            .clone();

        // Fresh stop flag for this generation run.
        let stop_flag = Arc::new(AtomicBool::new(false));
        self.stop_flag = Arc::clone(&stop_flag);

        godot_print!(
            "CLLawM: starting inference — prompt={:?}... n_predict={} ctx={} threads={} temp={} top_p={} top_k={}",
            prompt.chars().take(60).collect::<String>(),
            self.n_predict,
            self.ctx_size,
            self.n_threads,
            self.temperature,
            self.top_p,
            self.top_k,
        );

        let params = InferenceParams {
            prompt,
            system_prompt: self.system_prompt.clone(),
            raw: false,
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
                        // Log the full anyhow chain to stderr immediately so it
                        // is visible in the terminal even before the main thread drains.
                        eprintln!("[CLLawM] INFERENCE ERROR: {:#}", e);
                        let _ = tx.send(LlmEvent::Error(format!("{:#}", e)));
                    }
                },
            )
            .context("failed to spawn inference thread")?;

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
                    // Surface the full anyhow chain in the Godot error console.
                    godot_error!("CLLawM: inference failed — {}", msg);
                    self.signals().inference_failed().emit(&GString::from(&msg));
                    self.accumulated.clear();
                    finished = true;
                }
                LlmEvent::Log(line) => {
                    // Diagnostic progress logs from the inference thread.
                    godot_print!("[CLLawM thread] {}", line);
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

    /// Like `do_generate` but skips template application entirely.
    /// `formatted_prompt` must already be a complete, tokeniser-ready string
    /// (e.g. a full Gemma-4 IT multi-turn prompt built by GDScript).
    /// `AddBos::Always` is used so the tokeniser prepends the BOS token.
    fn do_generate_raw(&mut self, formatted_prompt: String) -> bool {
        use anyhow::Context as _;

        if self.rx.is_some() {
            godot_warn!("CLLawM::generate_raw called while already running; ignoring");
            return false;
        }
        if let Err(e) = self.ensure_model_loaded() {
            godot_error!("CLLawM::generate_raw: failed to load model: {e:#}");
            return false;
        }

        let model = self
            .model
            .as_ref()
            .expect("PANIC: model must be loaded before do_generate_raw")
            .clone();

        let stop_flag = Arc::new(AtomicBool::new(false));
        self.stop_flag = Arc::clone(&stop_flag);

        godot_print!(
            "CLLawM: starting raw inference ({} bytes prompt)",
            formatted_prompt.len()
        );

        let params = InferenceParams {
            prompt: formatted_prompt,
            system_prompt: String::new(), // unused in raw mode
            raw: true,
            n_predict: self.n_predict,
            ctx_size: self.ctx_size,
            n_threads: self.n_threads,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
        };

        let (tx, rx) = mpsc::channel::<LlmEvent>();
        self.rx = Some(rx);

        match std::thread::Builder::new()
            .name("clawasm-llm-inference".into())
            .spawn(
                move || match run_inference(model, stop_flag, tx.clone(), params) {
                    Ok(()) => {
                        let _ = tx.send(LlmEvent::Done);
                    }
                    Err(e) => {
                        eprintln!("[CLLawM] INFERENCE ERROR: {:#}", e);
                        let _ = tx.send(LlmEvent::Error(format!("{:#}", e)));
                    }
                },
            )
            .context("failed to spawn raw inference thread")
        {
            Ok(_) => true,
            Err(e) => {
                godot_error!("CLLawM::generate_raw: {e:#}");
                self.rx = None;
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

    /// Begin inference using a pre-formatted prompt string.
    ///
    /// Unlike `generate`, no chat template is applied — `prompt` is tokenised
    /// verbatim with a BOS prefix.  Use this when GDScript has already built
    /// the full multi-turn Gemma-4 IT conversation string.
    ///
    /// Returns `true` when inference is successfully started.
    #[func]
    pub fn generate_raw(&mut self, prompt: GString) -> bool {
        #[cfg(feature = "with-llama")]
        return self.do_generate_raw(prompt.to_string());
        #[cfg(not(feature = "with-llama"))]
        {
            let _ = prompt;
            godot_error!(
                "CLLawM::generate_raw: node compiled without `with-llama`; \
                 recompile clawasm with --features with-llama"
            );
            false
        }
    }

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
