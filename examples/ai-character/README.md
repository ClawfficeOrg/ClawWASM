# `examples/ai-character` — CLLawM AI Character tool-calling demo

A self-contained Godot 4.6+ project that shows the `CLLawM` node from
ClawWASM driving a 2D game character using **JSON tool calls**.

```
┌─────────────────────────────────────────────────────┬──── │
│                  🎮  Game World                      │ 🤖  AI │
│                                                     │ Character ⚙ │
│                                                     │────────── │
│                                                     │System  Welcome! │
│            ┌─────────────────┐                      │Set model…    │
│            │ I'm moving up!  │                      │            │
│            └────────┐        │                      │You  Go to   │
│                 ┌───┴──┐     │                      │  top-left! │
│                 │  👀  │ ← AI character             │            │
│                 └──────┘     │                      │AI  Moving…  │
│                              │                      │🔧 move_up → │
│                              │                      │   {moved:t} │
│                              │                      │🔧 speak → … │
│                              │                      │            │
│  Pos: (480, 200)             │                      │ Done.      │
│                              │                      │[Tell AI…][▶][■]│
└──────────────────────────────┴──────────────────────┴────────────┘
```

---

## Prerequisites

| Requirement | Notes |
|---|---|
| **Godot 4.6+** | GUI or headless. |
| **clawasm cdylib** built with `--features with-llama` | See step 1. |
| **GGUF model file** | Gemma 4 E2B-IT Q4_K_M recommended (~3.5 GB). See step 2. |
| **macOS:** Xcode Command Line Tools | `xcode-select --install` |
| **Linux:** `cmake`, `clang` | `apt install cmake clang` |

---

## Setup

### 1. Build and install the plugin

From the **repository root**:

```bash
# Quick build + sign + install (macOS). Quits if Godot is running — close it first.
bash scripts/build-plugin.sh --example examples/ai-character

# Or build for both examples at once (default target is llm-chat):
bash scripts/build-plugin.sh
bash scripts/build-plugin.sh --example examples/ai-character
```

For Linux/Windows, build manually:

```bash
cargo build -p clawasm --features with-llama --release
# Copy the cdylib:
cp target/release/libclawasm.so  examples/ai-character/addons/clawasm/  # Linux
# cp target/release/clawasm.dll  examples/ai-character/addons/clawasm/  # Windows
```

### 2. Download a GGUF model

```bash
# Pulls bartowski's Gemma 4 E2B-IT Q4_K_M (~3.5 GB)
bash scripts/download-model.sh Q4_K_M
# Model lands at: models/gemma-4-E2B-it-Q4_K_M.gguf
```

### 3. Open the project

```bash
godot --path examples/ai-character
```

Or open via *File → Open Project* in the Godot editor.

---

## In-editor quick start

1. Press **Play** (F5).
2. Click **⚙** in the top-right of the chat panel to open Settings.
3. Paste or browse to your `.gguf` model path.
4. Click **Apply & Reload Model**.
5. Type a command — e.g. *"Go to the top-right corner and say hello!"*
6. Watch the character move! Tool calls appear in yellow (`🔧`) in the chat.

---

## How it works

### Tool-calling protocol

The system prompt tells the model to emit **one JSON object per line** to
call a tool:

```
{"name": "move_up"}
{"name": "speak", "arguments": {"text": "Hello!"}}
```

`ai_character.gd` parses these lines from the model's response. For each
tool call it:

1. Executes the tool (moves character, shows speech bubble, etc.)
2. Collects the result dict
3. Injects the results back into the conversation as a `user` turn:
   ```
   <tool_response>
   [{"tool": "move_up", "result": {"moved": true, "position": {...}}}]
   </tool_response>
   ```
4. Calls `llm.generate_raw(prompt)` again so the model can continue.

This loop runs until the model produces a plain-text reply (no JSON tool
calls), or until `MAX_TOOL_LOOPS` (8) is reached.

### Available tools

| Tool | Description | Arguments |
|------|-------------|-----------|
| `move_up` | Move character up 60 px | — |
| `move_down` | Move character down 60 px | — |
| `move_left` | Move character left 60 px | — |
| `move_right` | Move character right 60 px | — |
| `get_position` | Query current position + world bounds | — |
| `speak` | Show speech bubble above character | `{"text": "..."}` |

### File layout

| File | Purpose |
|---|---|
| `project.godot` | Godot 4.6 config; main scene = `ai_character.tscn`. |
| `ai_character.tscn` | `HBoxContainer` — game world (left, expands) + chat panel (right, fixed). Settings in a `Window` sub-node. |
| `ai_character.gd` | Full GDScript: CLLawM wiring, tool parser, tool executor, prompt builder. |
| `clawasm.gdextension` | Extension manifest; dylib goes in `addons/clawasm/`. |
| `.godot/extension_list.cfg` | Pre-committed so the extension loads without editor startup. |

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| `CLLawM class not found` | Build with `--features with-llama` and install the cdylib. Run `bash scripts/build-plugin.sh --example examples/ai-character`. |
| Character doesn't move | Open ⚙ Settings and confirm model path is set + **Apply** was clicked. |
| No tool calls / AI ignores JSON format | Ensure you're using a model compatible with instruction tuning (e.g. Gemma-4-IT). Raw pre-train models don't follow tool format. |
| AI outputs raw `<end_of_turn>` tokens | Already stripped by `_clean()`. If still visible, check your model quantisation. |
| ⚠ Tool call limit reached | Increase `MAX_TOOL_LOOPS` in `ai_character.gd` or give the AI a simpler task. |
| App crash on macOS when rebuilding | Close Godot before running `build-plugin.sh`. The dylib can't be replaced while mmap'd. |
