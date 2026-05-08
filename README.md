# ClawWASM

Rust + WasmEdge plugin for Godot 4 that embeds a minimal OpenClaw-like
gateway inside a WASM sandbox. Lets Clawffice-Space nodes run on
WASM-capable targets (Godot host, SteamDeck, consoles) without Docker or
Podman.

> **Status: v0.5.0.** `ClawEngine` node is live — drop it into any Godot
> 4.6+ scene and stream stdout/stderr from a WasmEdge subprocess into
> GDScript signals. Pre-built addon bundles ship with every release.
> See [`CHANGELOG.md`](CHANGELOG.md) for what's in each version and
> [`ralph/PLAN.md`](ralph/PLAN.md) for the active workplan.

---

## Installation (no Rust required)

1. Download **`clawasm-addon-vX.Y.Z.zip`** from the
   [latest GitHub Release](https://github.com/ClawfficeOrg/ClawWASM/releases/latest).
2. Unzip and copy the `addons/clawasm/` folder into your Godot project root.
3. Godot 4.6+ auto-discovers `clawasm.gdextension` — the `ClawEngine` node
   is available immediately.
4. Install [WasmEdge 0.14.x](https://wasmedge.org/docs/start/install) so
   `ClawEngine` can spawn the runtime. The official installer drops it in
   `$HOME/.wasmedge/bin/wasmedge`; point `ClawEngine` at it via
   `set_wasmedge_binary(path)` or the `WASMEDGE_BIN` environment variable.

---

## Using `ClawEngine` in GDScript

```gdscript
@onready var engine := ClawEngine.new()

func _ready() -> void:
    add_child(engine)
    engine.register_module("res://my-module.wasm")
    engine.stdout_line.connect(func(line): print("[wasm] ", line))
    engine.finished.connect(func(code): print("[wasm] exit ", code))
    engine.start(PackedStringArray())
```

### API surface

| Method | Description |
| --- | --- |
| `register_module(path)` | Path to the `.wasm` file. Accepts `res://`, `user://`, or absolute paths. |
| `set_wasmedge_binary(path)` | Override the WasmEdge binary (default: `$WASMEDGE_BIN` or `wasmedge` on `$PATH`). |
| `start(args: PackedStringArray) -> bool` | Spawn the WasmEdge subprocess. Returns `false` if spawn fails. |
| `stop()` | Kill the running subprocess. |
| `is_running() -> bool` | |
| `module_path() -> String` | |

### Signals

| Signal | Args | When |
| --- | --- | --- |
| `stdout_line(line: String)` | one line at a time | While running |
| `stderr_line(line: String)` | one line at a time | While running |
| `finished(code: int)` | exit code | After all output, on clean exit |
| `failed(message: String)` | error description | If the subprocess couldn't start |

---

## Building from source

### Toolchain

```bash
rustup default stable
rustup target add wasm32-wasip1   # for the wasm example
```

> **macOS + Homebrew:** if you have Homebrew's Rust installed, set
> `RUSTC=~/.cargo/bin/rustc` when cross-compiling so Homebrew's `rustc`
> (which has no wasm targets) doesn't shadow the rustup shim.

### Build the Godot plugin (cdylib)

```bash
cargo build -p clawasm --release
# → target/release/libclawasm.{dylib,so,dll}
```

Copy the artifact into `addons/clawasm/` in your project alongside
`clawasm.gdextension` from the repo root.

### Build and run the wasm example

```bash
cargo build --manifest-path examples/hello-wasm/Cargo.toml \
            --target wasm32-wasip1 --release
wasmedge examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm
# → hello-wasm
```

---

## Quality gates (run before pushing)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace --all-targets
```

CI mirrors these plus a headless Godot 4.6.2 smoke in
`.github/workflows/ci.yml`.

---

## Repository layout

| Path | What lives here |
| --- | --- |
| `clawasm/` | Native Godot 4 plugin (cdylib), godot-rust ≥ 0.5. |
| `clawasm/engine/` | WasmEdge engine wrapper (feature-gated `with-wasmedge`). |
| `clawasm/src/engine_node.rs` | `ClawEngine` GodotClass implementation. |
| `examples/hello-wasm/` | Minimal `wasm32-wasip1` smoke binary. |
| `tests/godot-smoke/` | Headless Godot smoke project + runbook. |
| `docs/` | Plan, TODO, guidelines, architecture, memory. |
| `scripts/` | Build & smoke-test helpers. |
| `ralph/` | Autonomous Ralph development loop runner. |
| `.superpowers/skills/` | Skill files loaded by AI coding agents. |
| `.github/` | CI workflows, PR/issue templates, Copilot instructions. |

---

## Working on ClawWASM

This repo is designed for AI coding agents and humans interchangeably.
The binding contract is in [`AGENTS.md`](AGENTS.md) — **read it first.**

Key docs:

| File | Purpose |
| --- | --- |
| [`AGENTS.md`](AGENTS.md) | Universal agent rules (branch policy, commit style, test gates, memory protocol). |
| [`ralph/PLAN.md`](ralph/PLAN.md) | Active workplan — current task, up-next queue, open questions. |
| [`docs/PLAN.md`](docs/PLAN.md) | Long-term embedding plan. |
| [`docs/MEMORY.md`](docs/MEMORY.md) | Long-lived decisions and invariants. |
| [`docs/LEARNINGS.md`](docs/LEARNINGS.md) | Append-only lab notebook. |
| [`docs/TODO.md`](docs/TODO.md) | Semver-mapped roadmap. |
| [`docs/guidelines.md`](docs/guidelines.md) | Coding standards. |
| [`RELEASING.md`](RELEASING.md) | How releases are cut. |
| [`CHANGELOG.md`](CHANGELOG.md) | What shipped when. |

---

## License

TODO — choose and add `LICENSE` before v1.0.0.
