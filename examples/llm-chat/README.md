# `examples/llm-chat` — CLLawM streaming chat demo

A self-contained Godot 4.6+ project that demonstrates the `CLLawM` node
from the ClawWASM extension. It shows a side-by-side settings panel and
streaming chat log, with every sampling parameter exposed in the UI.

```
┌─────────────────────┬───────────────────────────────────────────┐
│ ⚙  Settings         │ 💬  Chat                                   │
│                     │                                           │
│ Model path (.gguf)  │  System  Welcome to CLLawM Chat! ...     │
│ [/path/to/…] […]    │                                           │
│                     │  You  Why is the sky blue?                │
│ System prompt       │                                           │
│ ┌─────────────────┐ │  Assistant  Rayleigh scattering causes …  │
│ │You are a help…  │ │            shorter blue wavelengths to … ▌│
│ └─────────────────┘ │                                           │
│                     │ Generating…                               │
│ Temperature  ──●──  │ ┌────────────────────────────┐ [Send][Stop]
│ Top-p        ──●──  │ │ Type a message and press…  │           │
│ Top-k        [64 ]  │ └────────────────────────────┘           │
│ Max tokens  [512 ]  └───────────────────────────────────────────┘
│ CPU threads   [4 ]
│ Context     [4096]
│ [Apply & Reload Model]
└─────────────────────┘
```

---

## Prerequisites

| Requirement | Notes |
|---|---|
| **Godot 4.6+** | GUI or headless. |
| **clawasm cdylib** built with `--features with-llama` | See step 1 below. |
| **GGUF model file** | Gemma 4 E2B-IT Q4_K_M recommended (~3.5 GB). See step 2. |
| **macOS:** Xcode toolchain | Required by llama-cpp-sys-2's cmake build. |
| **Linux:** `cmake`, `clang` | `apt install cmake clang` or equivalent. |

---

## Setup

### 1. Build the plugin with llama.cpp support

From the **repository root**:

```bash
# macOS — Metal GPU acceleration enabled automatically by cmake
cargo build -p clawasm --features with-llama --release

# Copy the cdylib into this example's addon directory
cp target/release/libclawasm.dylib  examples/llm-chat/addons/clawasm/  # macOS
# cp target/release/libclawasm.so   examples/llm-chat/addons/clawasm/  # Linux
# cp target/release/clawasm.dll     examples/llm-chat/addons/clawasm/  # Windows
```

> **First build:** llama-cpp-sys-2 compiles llama.cpp from source using
> cmake. This takes ~2 minutes but is cached by Cargo on subsequent builds.

### 2. Download a GGUF model

```bash
# From the repository root — pulls bartowski's Gemma 4 E2B-IT Q4_K_M (3.46 GB)
bash scripts/download-model.sh Q4_K_M
# Model lands at: models/gemma-4-E2B-it-Q4_K_M.gguf
```

Or use any GGUF model compatible with llama.cpp. Point the Settings
panel at the file or use the `…` button to browse.

### 3. Open the project in Godot

**GUI:**
```bash
godot --path examples/llm-chat
```
Or open `examples/llm-chat` via *File → Open Project* in the Godot editor.

**Headless (quick smoke test):**
```bash
godot --headless --path examples/llm-chat
# Note: headless mode skips rendering but the inference thread still runs.
# Signals are still emitted; you'll see godot_print output in the terminal.
```

> The `.godot/extension_list.cfg` is pre-committed so the extension loads
> without the editor needing to write it first.

---

## In-editor quick start (once open)

1. Press **Play** (F5).
2. In the **Settings** panel, paste or browse to your `.gguf` model path.
3. Optionally edit the system prompt or adjust sampling sliders.
4. Click **Apply & Reload Model**.
5. Type a message in the bottom bar and press **Enter** or **Send**.
6. Watch tokens stream into the chat log in real time.
7. Click **Stop** to interrupt generation early.

---

## Scene / script layout

| File | Purpose |
|---|---|
| `project.godot` | Godot 4.6 project config; main scene = `chat.tscn`. |
| `chat.tscn` | Full UI scene: `HSplitContainer` with settings left, chat right. |
| `chat.gd` | GDScript: wires `CLLawM` signals, manages chat state, renders BBCode. |
| `clawasm.gdextension` | Extension manifest pointing at `addons/clawasm/libclawasm.*`. |
| `.godot/extension_list.cfg` | Pre-committed so the extension loads without editor startup. |
| `addons/clawasm/` | Drop your compiled cdylib here (not committed). |

---

## How chat.gd works

```
_ready()
  ├─ ClassDB.instantiate("CLLawM")   → add_child(llm)
  ├─ llm.token_generated → _on_token(token)
  │     _streaming += token
  │     _render()   ← clear + append(_frozen + streaming_bubble + "▌")
  ├─ llm.inference_done → _on_done(full, code)
  │     _frozen += _bubble("assistant", _streaming)
  │     _render()
  └─ llm.inference_failed → _on_failed(msg)

_on_send()
  ├─ _frozen += _bubble("user", prompt)
  ├─ llm.generate(prompt)   → spawns background thread in Rust
  └─ _render()

_on_apply()
  └─ llm.set_model / set_system_prompt / set_temperature / … → apply_btn
```

**Why `ClassDB.instantiate` instead of `CLLawM.new()`?**  
Using `ClassDB.instantiate("CLLawM")` lets the script parse cleanly even
when the extension isn't loaded (e.g. during development without the
cdylib present). `CLLawM.new()` would cause a parse error.

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| `CLLawM class not found` | Build with `--features with-llama` and copy the cdylib into `addons/clawasm/`. |
| `generate() returned false` | Click **Apply & Reload Model** first. Check the path points to a valid `.gguf`. |
| No tokens appear | The model loads lazily on first `generate()` — this can take 10–30 s for a 3.5 GB model. Watch the status bar. |
| App freezes on first generate | The model is being loaded on the main thread (by design). Large models may stall the UI briefly during initial load. |
| `cmake: command not found` | Install cmake. macOS: `xcode-select --install`. Linux: `apt install cmake clang`. |
| Metal not used on macOS | Ensure Xcode Command Line Tools are installed (`xcode-select --install`). Metal is auto-enabled by the cmake build. |
