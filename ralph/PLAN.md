# Ralph Plan — 2026-05-12 (updated)

## HANDOFF — read this first in the next session

### Branch: `feature/llm-inference`

Everything below is the current state of this branch. The PR is NOT yet open.
Open it when satisfied with the ai-character demo in Godot.

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
- `examples/ai-character/` — tool-calling AI character demo ✅
  - Scene: HBoxContainer — game world (left, expands) + chat panel (right, ~310px fixed)
  - Chat panel has ⚙ button opening a Window sub-node (settings popup)
  - Tools: move_up/down/left/right, get_position, speak (speech bubble)
  - Tool loop guard: MAX_TOOL_LOOPS = 8
  - System prompt rebuilt every turn with live world dimensions
- CHANGELOG.md, docs/LEARNINGS.md updated ✅

## What is IN PROGRESS (start here next session)

### 1. Open PR + cut v0.6.0  (immediate)

- [ ] **Open PR** for `feature/llm-inference` → main
  - PR description: test plan (run ai-character demo, show character moves), CHANGELOG reference, reviewer @CompewterTutor
  - Squash-merge is fine given the iterative commits on this branch
- [ ] **Cut v0.6.0** — bump `clawasm` version, tag `v0.6.0`, publish release

---

### 2. Maze Navigator with Vision  (next feature branch: `feature/maze-vision`)

A 2D maze where `CLLawM` navigates autonomously by *seeing* a partial fog-of-war
image of the maze after each step. This is the primary driver for implementing
vision/multimodal support in CLLawM.

#### Big picture

```
User: "navigate this maze"
  └→ AI sees partial maze image (fog of war) → emits {"name": "move_right"}
    └→ character moves → new partial image rendered → sent back to AI
      └→ repeat until exit reached (or MAX_STEPS exceeded)
```

#### Step A — Vision API in Rust  (`clawasm/src/llm_node.rs`)

`llama-cpp-2` v0.1.146 already contains `mtmd.rs` (947 lines) with a full
multimodal/MTMD context API.  bartowski's Gemma-4 repo has
`mmproj-google_gemma-4-E2B-it-f16.gguf` (the vision projector GGUF).

New GDScript-callable methods on `CLLawM`:
```
set_mmproj(path: String)                       # load mmproj GGUF
generate_with_image(image_path: String, prompt: String) -> bool
```

Internally:
- `MtmdContext::new(&model, &mtmd_params)` after model load if mmproj is set
- `MtmdBitmap::from_file(image_path)` to load the frame
- `mtmd_ctx.tokenize_and_embed(bitmap, ...)` to embed the image tokens
- Continue with the standard sampling loop via `ctx.decode()`
- Image sent once per `generate_with_image` call; text turns use `generate_raw`

Dependency: update `scripts/download-model.sh` to optionally fetch the mmproj:
```bash
bash scripts/download-model.sh Q4_K_M mmproj   # adds --mmproj flag
```

#### Step B — Maze generation in GDScript

Use the **Recursive Backtracker (DFS)** algorithm — simplest correct maze
generator, creates perfect mazes (exactly one path between any two cells).

Grid convention: `width` × `height` cells where odd-indexed cells are passages
and even-indexed cells are walls (so the actual grid array is
`(2*cols+1) × (2*rows+1)` to include border walls).

```gdscript
func _generate_maze(cols: int, rows: int) -> Array:
    # 0 = wall, 1 = passage
    var grid: Array = []
    for _y in (2 * rows + 1):
        var row: Array = []
        for _x in (2 * cols + 1): row.append(0)
        grid.append(row)
    var visited: Dictionary = {}
    var stack: Array = [Vector2i(1, 1)]
    grid[1][1] = 1
    visited[Vector2i(1, 1)] = true
    while not stack.is_empty():
        var cur: Vector2i = stack[-1]
        var nbrs := _maze_neighbors(cur, grid, visited, cols, rows)
        if nbrs.is_empty():
            stack.pop_back()
        else:
            var nxt: Vector2i = nbrs[randi() % nbrs.size()]
            var mid := Vector2i((cur.x + nxt.x) / 2, (cur.y + nxt.y) / 2)
            grid[mid.y][mid.x] = 1
            grid[nxt.y][nxt.x] = 1
            visited[nxt] = true
            stack.append(nxt)
    return grid

func _maze_neighbors(c: Vector2i, grid: Array, visited: Dictionary,
                     cols: int, rows: int) -> Array:
    var result: Array = []
    for d: Vector2i in [Vector2i(2,0), Vector2i(-2,0), Vector2i(0,2), Vector2i(0,-2)]:
        var n := c + d
        if n.x > 0 and n.x < 2*cols and n.y > 0 and n.y < 2*rows \
                and not visited.has(n):
            result.append(n)
    return result
```

Exit cell: bottom-right passage cell `Vector2i(2*cols-1, 2*rows-1)`.

Rendering: Use `_draw()` overrides on a `Control` node (or a grid of `ColorRect`
nodes).  Fog-of-war: track `visited_cells` set; at render time only show cells
within Manhattan distance 1 of any visited cell or the current cell.  Everything
else draws as opaque black.

#### Step C — Fog-of-war image capture

After each move:
1. Render the fog-of-war view to a `SubViewport` (sized to the visible area).
2. `subviewport.get_texture().get_image()` → `Image`.
3. Save to a temp file: `image.save_png(OS.get_temp_dir() + "/maze_view.png")`.
4. Pass to `llm.generate_with_image(temp_path, maze_prompt)`.

The maze prompt (injected as a user turn):
```
You are navigating a maze.  White = passage, black = wall, grey = fog (unseen),
blue = you, green = exit.  Which direction moves you closer to the exit?
Output a single tool call on its own line, e.g.:
{"name": "move_right"}
```

#### Step D — Scene layout for `examples/maze-navigator/`

Reuse the same HBoxContainer pattern as ai-character:
```
MazeNavigator (Control, full-rect)
└── HBoxContainer
    ├── MazePane (Control, expand) — SubViewport renders fog-of-war here
    └── ChatPanel (VBoxContainer, 310px) — same as ai-character
        ├── TitleRow: "Maze Navigator" + ⚙ button
        ├── ChatLog (streaming tool calls + narration)
        └── InputRow: [Navigate!] [Stop]
└── SettingsWindow (Window, exclusive) — model path + mmproj path + sampling
```

When the user types "navigate this maze" (or clicks Navigate!):
1. `_history.clear()`, `_visited.clear()`, `_center_character()`
2. Start the vision loop: capture image → `generate_with_image` → `_on_done`
3. Parse tool call, execute move, add to `_visited`, repeat
4. Stop when character reaches the exit cell, or after `MAX_STEPS = 50`

#### Stretch goals (later)
- Animate character movement between cells (tweening)
- Larger maze sizes (configurable in Settings)
- Step counter + path trace overlay
- "Solve" mode: optimal path shown as ghost after AI finishes

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
