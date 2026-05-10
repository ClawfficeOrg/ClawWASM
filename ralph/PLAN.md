# Ralph Plan — 2026-05-10 (updated)

## HANDOFF — read this first in the next session

### Branch: `feature/llm-inference`

Everything below is the current state of this branch. The PR is NOT yet open.
Open it when the ai-character example is working.

---

## What is fully done on this branch

- `CLLawM` Godot node — native in-process Gemma-4 inference via `llama-cpp-2` ✅
- `apply_chat_template` fallback — Gemma-4 IT hand-written formatter (Jinja2 too old) ✅
- `generate_raw(prompt)` API — bypasses template, takes pre-formatted prompt ✅
- Stop strings — `<end_of_turn>`, `</start_of_turn>`, `<start_of_turn>`, `<eos>` etc. ✅
- Multi-turn conversation history in `chat.gd` with `_build_gemma_prompt()` ✅
- Auto-apply model on first Send if path is set ✅
- Step-by-step inference diagnostics (`tlog!` macro, `LlmEvent::Log`) ✅
- `scripts/build-plugin.sh` — builds, ad-hoc signs dylib, guards against Godot running ✅
- HiDPI window fix — `canvas_items` stretch + `mode=2` maximized + `allow_hidpi` ✅
- `examples/llm-chat/` — working streaming chat demo ✅
- CHANGELOG.md, docs/LEARNINGS.md updated ✅

## What is IN PROGRESS (start here next session)

### AI Character Tool-Calling Demo

Create `examples/ai-character/` as a NEW standalone Godot project.
A character (blue square with eyes — user will add sprites later) moves
around a 2D game world. The AI controls it via JSON tool calls.

#### Files to create

1. `examples/ai-character/project.godot` — same display settings as llm-chat
   (canvas_items, maximized, HiDPI)
2. `examples/ai-character/ai_character.tscn` — scene layout (see below)
3. `examples/ai-character/ai_character.gd` — main script (see below)
4. `examples/ai-character/addons/clawasm/clawasm.gdextension` — copy from llm-chat
5. Update `scripts/build-plugin.sh` to also install dylib into
   `examples/ai-character/addons/clawasm/` (add a second install block)

#### Scene layout (`ai_character.tscn`)

```
AICharacter (Control, full rect)
└── HSplitContainer (full rect)
    ├── LeftPanel (VBoxContainer, min_width=420)
    │   ├── Label "🎮  Game World" (font_size=14)
    │   ├── GameArea (Control, size_flags_vertical=3, clip_contents=true)
    │   │   ├── GameBackground (ColorRect, full rect, Color(0.12,0.15,0.12))
    │   │   └── Character (Control, size=48x48, unique_name)
    │   │       ├── CharacterBody (ColorRect, full rect, Color(0.25,0.45,0.85))
    │   │       ├── EyeLeft (ColorRect, pos=(8,12), size=10x10, white)
    │   │       ├── EyeRight (ColorRect, pos=(30,12), size=10x10, white)
    │   │       └── SpeechPanel (PanelContainer, visible=false, pos=(-64,-52), size=176x44, unique_name)
    │   │           └── SpeechLabel (Label, autowrap=1, unique_name)
    │   └── CoordLabel (Label, font_size=11, grey, unique_name)
    └── RightPanel (VBoxContainer, size_flags_horizontal=3)
        ├── Label "🤖  AI Controller" (font_size=14)
        ├── ModelRow (HBoxContainer)
        │   ├── ModelPathEdit (LineEdit, size_flags_h=3, unique_name)
        │   └── BrowseBtn (Button, text="…", unique_name)
        ├── ApplyBtn (Button, text="Load Model", unique_name)
        ├── HSeparator
        ├── ChatLog (RichTextLabel, bbcode=true, scroll_following=true, size_flags_v=3, unique_name)
        ├── StatusLabel (Label, font_size=11, grey, unique_name)
        └── InputRow (HBoxContainer)
            ├── PromptEdit (LineEdit, size_flags_h=3, placeholder="Tell the AI what to do...", unique_name)
            ├── SendBtn (Button, text="Send", unique_name)
            └── StopBtn (Button, text="Stop", unique_name)
+ ModelFileDialog (FileDialog, file_mode=0, access=2, filters=["*.gguf"], unique_name)
```

#### Script (`ai_character.gd`) key architecture

```gdscript
extends Control

# @onready refs for all unique_name nodes (GameArea, Character, SpeechPanel,
# SpeechLabel, CoordLabel, ChatLog, StatusLabel, ModelPathEdit, BrowseBtn,
# ApplyBtn, PromptEdit, SendBtn, StopBtn, ModelFileDialog)

const STEP      := 60.0        # pixels per move
const CHAR_SIZE := Vector2(48, 48)
const SPEECH_SECS := 4.0
const MAX_TOOL_LOOPS := 8      # safety cap

var llm: Node
var _history: Array = []
var _streaming: String = ""
var _running: bool = false
var _frozen: String = ""
var _speech_timer: float = 0.0

# _ready(): await one frame, _center_character(), instantiate CLLawM,
#   connect signals, wire UI buttons.

# _process(delta): decrement _speech_timer, hide SpeechPanel at 0,
#   update CoordLabel with character.position.

# _on_token(token): _streaming += token; _render()

# _on_done(full, code):
#   1. _clean() the streaming text (strip stop strings)
#   2. _parse_tool_calls(text) -> Array of {name, arguments} dicts
#      Parser: split by "\n", try JSON.parse_string() on each line,
#      keep if result is Dictionary and has key "name"
#   3. If no tool calls: save to history, show in chat, _set_running(false)
#   4. If tool calls:
#      - save model turn to history
#      - execute each call via _execute_tool(name, args) -> result dict
#      - _append_tool_event() shows "🔧 tool: name → result" in yellow in chat
#      - inject tool results as {role:"user", content:"<tool_response>\n{results}\n</tool_response>"}
#      - call llm.generate_raw(_build_prompt()) to continue
#      - guard with MAX_TOOL_LOOPS

# _execute_tool(name, args) -> Dictionary:
#   match name:
#     "move_up"    -> character.position.y -= STEP (clamped), return {moved, position}
#     "move_down"  -> character.position.y += STEP (clamped), return {moved, position}
#     "move_left"  -> character.position.x -= STEP (clamped), return {moved, position}
#     "move_right" -> character.position.x += STEP (clamped), return {moved, position}
#     "get_position" -> return {position: {x,y}, bounds: {width,height}}
#     "speak"      -> speech_label.text = args["text"]; speech_panel.visible=true;
#                     _speech_timer=SPEECH_SECS; return {spoken: text}
#     _            -> return {error: "unknown tool"}

# _build_prompt() -> String:
#   Builds full Gemma-4 IT multi-turn prompt from _history.
#   System prompt is _make_system_prompt() (included as first turn).
#   Same format as chat.gd _build_gemma_prompt() but includes system inline.

# _make_system_prompt() -> String:
#   Describes the 2D world size (from game_area.get_rect()),
#   the JSON tool call format, and all 6 tools.
#   IMPORTANT: tell model to output JSON on its OWN LINE, nothing else on that line.
#   Format: {"name": "tool_name"} or {"name": "speak", "arguments": {"text": "..."}}

# _parse_tool_calls(text) -> Array:
#   for line in text.split("\n"):
#     trimmed = line.strip_edges()
#     if trimmed.begins_with("{") and trimmed.ends_with("}"):
#       parsed = JSON.parse_string(trimmed)
#       if parsed is Dictionary and parsed.has("name"): append

# _strip_json_lines(text) -> String:
#   Remove lines that parse as tool calls (leaving only narration text)

# _render(), _bubble(), _esc(), _clean(), _append_tool_event() — same pattern
#   as chat.gd. _append_tool_event() uses yellow color for tool calls.
```

#### System prompt format (critical — this determines if tool calling works)

```
You are an AI controlling a character in a 2D game world in Godot Engine.
The world is {W}x{H} pixels. The character is a blue square (48x48).

To call a tool, output a JSON object on its own line (nothing else on that line):
{"name": "tool_name"}
or with arguments:
{"name": "speak", "arguments": {"text": "Hello!"}}

Available tools:
- move_up: Move character up by 60 pixels
- move_down: Move character down by 60 pixels
- move_left: Move character left by 60 pixels
- move_right: Move character right by 60 pixels
- get_position: Get current x,y position and world bounds
- speak: Show speech bubble (arguments: {"text": "..."})

Rules:
- Call one tool per line. Call multiple tools in sequence if needed.
- After finishing your moves, narrate in plain text what you did.
- Be playful and expressive!
```

#### gdextension file to copy

Copy `examples/llm-chat/addons/clawasm/clawasm.gdextension` verbatim.
Do NOT copy the dylib — build-plugin.sh will install it.

#### build-plugin.sh update

Add a second install block after the llm-chat install:
```bash
AI_ADDON="$REPO_ROOT/examples/ai-character/addons/clawasm"
mkdir -p "$AI_ADDON"
cp "$BUILT_DYLIB" "$AI_ADDON/$DYLIB_NAME"
codesign --sign - --force --timestamp=none "$AI_ADDON/$DYLIB_NAME"
codesign --verify --strict "$AI_ADDON/$DYLIB_NAME"
echo "    Also installed to $AI_ADDON"
```

---

## After the ai-character demo works

- [ ] **Open PR** for `feature/llm-inference` → main
  - PR description needs: test plan, CHANGELOG reference, reviewer @CompewterTutor
- [ ] **Cut v0.6.0** — bump clawasm to 0.6.0, tag
- [ ] **Vision support** — `MtmdContext` + mmproj GGUF for Gemma-4 image input
  - llama-cpp-2 v0.1.146 HAS multimodal API in `mtmd.rs` (947 lines!)
  - bartowski repo HAS `mmproj-google_gemma-4-E2B-it-f16.gguf`
  - Need: `set_mmproj(path)` on CLLawM, `generate_with_image(prompt, img_path)`,
    image picker in UI, update download-model.sh
- [ ] Multi-turn conversation memory (currently cleared on Apply)
- [ ] More tools: scene tree inspection, node creation, etc.

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = engine MVP. v0.3.0 = `ClawEngine` Godot node.
v0.4.0 = headless CI. v0.5.0 = pre-built addon bundle (in flight).
v0.6.0+ = in-process embedding, ironclaw/LLM wiring.
eventually run llm inference and tools like ironclaw in godot wasmedge
## HANDOFF — read this first in the next session

We are mid-PR on `feature/llm-inference`. The subprocess (`llama-cli`) approach
was scrapped. The correct architecture is:

```
Godot scene
 ├─ CLLawM  (GodotClass — native cdylib compiled per platform)
 │    └─ llama.cpp baked in via `llama-cpp-2` Rust crate
 │         Metal auto-enabled on macOS, CUDA on Linux/Windows
 │         Arc<LlamaModel> cached across calls
 │         Inference runs in background thread via mpsc channel
 └─ ClawEngine  (existing — WasmEdge subprocess)
      └─ clawasm-llm.wasm  (application logic, future milestone)
```

### What is already done on this branch

- `clawasm/engine/src/stream.rs` — `Event::StdoutChunk` + `Runner::spawn_chunked` ✅
- `clawasm/engine/src/engine_node.rs` — `StdoutChunk` wildcard arm ✅
- `scripts/download-model.sh` — pulls bartowski Q4_K_M (3.46 GB) ✅
- `docs/LEARNINGS.md` — WasmEdge WASI-NN incompatibility + chunk-reading entries ✅
- `CHANGELOG.md` — Unreleased section started ✅

### What still needs to be written (next session starts here)

#### 1. `clawasm/Cargo.toml` — add deps
```toml
[features]
default = []
with-wasmedge = ["clawasm-engine/with-wasmedge"]
with-llama    = ["dep:llama-cpp-2", "dep:anyhow", "dep:encoding_rs"]

[dependencies]
godot          = "0.5"
clawasm-engine = { path = "engine", default-features = false }
llama-cpp-2    = { version = "0.1", optional = true }
anyhow         = { version = "1.0", optional = true }
encoding_rs    = { version = "0.8", optional = true }
```
Metal is AUTO-ENABLED on macOS by llama-cpp-sys-2's cmake — no extra feature needed.

#### 2. `clawasm/engine/src/lib.rs` — REMOVE `LlmConfig`
Delete the entire `LlmConfig` struct, `impl LlmConfig`, `DEFAULT_LLAMA_CLI_BIN`
constant, and the 4 LlmConfig tests. Keep everything else.

#### 3. `clawasm/src/llm_node.rs` — FULL REWRITE

Key API facts confirmed from docs.rs (llama-cpp-2 v0.1.146):

```rust
// Backend — create wherever needed, init is idempotent, drop is safe
LlamaBackend::init() -> Result<LlamaBackend>

// Model — Send + Sync, cache as Arc<LlamaModel>
LlamaModel::load_from_file(&backend, path, &LlamaModelParams::default())
model.chat_template(None) -> Result<LlamaChatTemplate>       // embedded in GGUF
model.apply_chat_template(&tmpl, &msgs, true) -> Result<String>  // add_ass=true
model.str_to_token(&str, AddBos::Never) -> Result<Vec<LlamaToken>>  // template adds BOS
model.is_eog_token(token) -> bool
model.token_to_piece(token, &mut decoder, false, None) -> Result<String>
model.n_params() -> u64

// Chat messages
LlamaChatMessage::new("system", &sys) -> Result<LlamaChatMessage>
LlamaChatMessage::new("user", &prompt) -> Result<LlamaChatMessage>

// Context — !Send, must stay in inference thread
LlamaContextParams::default()
    .with_n_ctx(NonZeroU32::new(ctx_size))  // Option<NonZeroU32>
    .with_n_threads(n: i32)
    .with_n_threads_batch(n: i32)
model.new_context(&backend, ctx_params) -> Result<LlamaContext<'_>>
ctx.decode(&mut batch) -> Result<()>

// Batch — !Send, create in thread
// add_sequence sets logits=true on last token automatically
LlamaBatch::new(n_tokens: usize, n_seq_max: i32) -> LlamaBatch<'_>
batch.add_sequence(&tokens, seq_id: i32, logits_all: bool) -> Result<()>
batch.add(token, pos: i32, seq_ids: &[i32], logits: bool) -> Result<()>
batch.clear()
batch.n_tokens() -> i32

// Sampler — !Send, create in thread
LlamaSampler::chain_simple([
    LlamaSampler::top_k(k: i32),
    LlamaSampler::top_p(p: f32, min_keep: usize),  // min_keep=1
    LlamaSampler::temp(t: f32),
    LlamaSampler::dist(seed: u32),  // random sampling
])
sampler.sample(&ctx, idx: i32) -> LlamaToken  // idx = batch.n_tokens()-1
sampler.accept(token)

// Decoder for token_to_piece
let mut decoder = encoding_rs::UTF_8.new_decoder();
```

Struct layout:
```rust
#[derive(GodotClass)]
#[class(base = Node)]
pub struct CLLawM {
    base: Base<Node>,
    model_path: Option<PathBuf>,
    system_prompt: String,
    n_predict: u32,      // default 512
    ctx_size: u32,       // default 4096
    n_threads: i32,      // default 4
    temperature: f32,    // default 1.0
    top_p: f32,          // default 0.95
    top_k: i32,          // default 64
    rx: Option<Receiver<LlmEvent>>,
    stop_flag: Arc<AtomicBool>,
    accumulated: String,
    #[cfg(feature = "with-llama")]
    model: Option<Arc<llama_cpp_2::model::LlamaModel>>,
}
```

Thread model:
- `ensure_model_loaded()` → loads model on main thread, caches as `Arc<LlamaModel>`
- `generate_impl()` → clones Arc, spawns thread, sends `Receiver` back via `self.rx`
- Inference thread: creates its own `LlamaBackend` (idempotent), creates context,
  runs token loop, sends `LlmEvent::Token/Done/Error` via mpsc
- `_process()` → drains `self.rx`, emits Godot signals
- `stop()` → sets `stop_flag: Arc<AtomicBool>`, thread checks it each token

Signals: `token_generated(token: GString)`,
         `inference_done(full_text: GString, exit_code: i64)`,
         `inference_failed(message: GString)`

Methods: `set_model`, `model_path`, `set_system_prompt`, `system_prompt`,
         `set_n_predict`, `set_ctx_size`, `set_n_threads`, `set_temperature`,
         `set_top_p`, `set_top_k`, `generate`, `stop`, `is_running`

Stub path (no `with-llama`):
- All methods compile, `generate` logs error and returns false
- Use `#[cfg(feature = "with-llama")]` impl block for `ensure_model_loaded`
  and `generate_impl`; separate `#[cfg(not(...))]` stub for `generate_impl`

#### 4. `clawasm/engine/README.md` — note that LlmConfig was removed

#### 5. CI (`.github/workflows/ci.yml`)
Add a `with-llama` job on macOS-latest that does:
```yaml
- run: cargo build -p clawasm --features with-llama
```
Note: first build downloads and compiles llama.cpp (~2 min). Cache `target/`.
Do NOT gate it as required — `continue-on-error: true` until CI is validated.

#### 6. WASM bridge — plan only, no code yet
Document in `docs/TODO.md` under v0.7.0:
- JSON-over-stdout protocol: WASM module writes
  `{"__cllaw__":"generate","prompt":"...","id":"uuid"}` to stdout
- GDScript `ClawBridge` autoload detects prefix, calls `CLLawM.generate()`
- CLLawM tokens feed back to WASM module via stdin pipe
- This is the bridge between `ClawEngine` + `clawasm-llm.wasm` and `CLLawM`

#### 7. After code is written
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
# with-llama build (requires cmake + clang on PATH):
cargo build -p clawasm --features with-llama
```
Then commit, push, open PR.

### Model to use for smoke testing
```bash
bash scripts/download-model.sh Q4_K_M   # 3.46 GB
# set LLAMA_BIN_PATH if needed (not needed — no subprocess any more)
```

### Key decisions already locked
- Model: `bartowski/google_gemma-4-E2B-it-GGUF` Q4_K_M
- Crate: `llama-cpp-2 = "0.1"` (latest 0.1.146)
- Metal: automatic on macOS via cmake, no feature flag
- WasmEdge WASI-NN: NOT used (too old for Gemma 4 architecture)
- Subprocess wrapping: SCRAPPED (defeats portability)
- `Runner::spawn_chunked` / `Event::StdoutChunk`: KEPT for future WASM module stdout

---

## Active task

### CLLawM: Gemma 4 E2B-IT inference in Godot (this PR)

- **Branch:** `feature/llm-inference`
- **New node:** `CLLawM` — wraps `llama-cli` subprocess, streams tokens via
  `token_generated` signal, accumulates full response in `inference_done`.
- **Engine additions:** `LlmConfig` struct, `Runner::spawn_chunked`,
  `Event::StdoutChunk`.
- **Download helper:** `scripts/download-model.sh` (bartowski Q4_K_M, 3.46 GB).
- **Why not WasmEdge WASI-NN:** WasmEdge 0.14.1's bundled llama.cpp predates
  Gemma 4's PLE / hybrid-attention architecture. Documented in LEARNINGS.md.
- **Acceptance:** `cargo test --workspace` green; `CLLawM` node visible in
  Godot editor; `generate("hello")` produces streaming tokens via signals.

- **File edited:** `.github/workflows/release.yml` — new "Build addon bundle
  zip" step in the `release` job. After all platform builds complete and
  artifacts are flattened, assembles `addon-bundle/addons/clawasm/` with
  `clawasm.gdextension` + all three cdylibs + a drop-in `README.md`, then
  zips to `clawasm-addon-vX.Y.Z.zip` and attaches it to the release.
- **Dry-run verified** locally: correct zip structure, macOS dylib present,
  `find | xargs` pipeline works for all three extensions.
- **Acceptance:** `release.yml` CI green; GitHub Release for v0.5.0 includes
  `clawasm-addon-v0.5.0.zip` alongside the per-platform files.

## Up next (ordered)

- [ ] **Cut v0.6.0** — merge this PR, bump `clawasm` to 0.6.0, tag.
- [ ] **Multi-turn conversation** — accumulate chat history in `CLLawM`;
      `generate_turn(role, text)` API.
- [ ] **ironclaw / LLM tool wiring** — first GDScript API sketch.
- [ ] **In-process WasmEdge embedding** — revisit once a
      `wasmedge-sys` version compatible with WasmEdge 0.14.1 appears.

## Done this iteration block

- [x] feat(repo): add superpowers skills, Ralph loop, agents contract, CI/CD scaffolding (PR #9)
- [x] fix(clawasm): drop direct `wasmedge-sys` dep; route through `clawasm-engine` (PR #10)
- [x] feat(engine): v0.2.0 MVP — subprocess `Instance::run` (PR #11)
- [x] feat(godot): `ClawEngine` node + streaming `Runner` + smoke runbook (PR #12)
- [x] chore(release): v0.2.0 — bumped clawasm to 0.2.0, tagged (PR #13)
- [x] docs(smoke): headless macOS smoke GREEN — Godot 4.6.2, godot-rust 0.5.2, WasmEdge 0.14.1
- [x] ci(godot): headless Godot 4.6.2 CI smoke job, both platforms green (PR #15)
- [x] chore(release): v0.4.0 (PR #16)
- [x] feat(release): pre-built addon bundle zip (PR #17, v0.5.0)
- [x] feat(llm): `CLLawM` node — Gemma 4 E2B-IT inference via llama-cli,
      `LlmConfig`, `Runner::spawn_chunked`, `Event::StdoutChunk` (this PR)

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error`?~~
  Resolved in PR #11: now required.
- **Q2:** ~~Headless Godot smoke in CI~~ — wired PR #15;
  `godot-smoke` job, Godot 4.6.2, green on macOS + Linux.
- **Q4:** ~~Pre-built addon bundle~~ — wired in this PR.

## Archive

(empty — first iteration block)
