# Godot smoke-test runbook (manual)

This is a manual end-to-end check that the `ClawEngine` node loads in
Godot 4 and successfully runs `examples/hello-wasm` under WasmEdge.
There is **no CI runner yet** (Q2 in `ralph/PLAN.md`).

## Prerequisites

- Godot 4.2 or newer.
- WasmEdge 0.14.x installed (the official installer drops it in
  `$HOME/.wasmedge`).
- A `release` build of the `clawasm` cdylib (step 1).
- A `wasm32-wasip1` build of `examples/hello-wasm` (step 2).

> **macOS + Homebrew note:** if Homebrew's Rust is installed,
> `/opt/homebrew/bin/rustc` appears before the rustup shim in `$PATH`
> and the wasm build will fail with "can't find crate for `std`". Fix:
> prefix step 2 with `RUSTC=~/.cargo/bin/rustc` or ensure
> `~/.cargo/bin` is first in `$PATH`. See `docs/LEARNINGS.md` 2026-05-06.

## Steps

### 1. Build the host cdylib

    cargo build -p clawasm --release

Produces, depending on your platform:

- `target/release/libclawasm.dylib` (macOS)
- `target/release/libclawasm.so` (Linux)
- `target/release/clawasm.dll` (Windows)

### 2. Build the wasm module

    RUSTC=~/.cargo/bin/rustc \
      cargo build --manifest-path examples/hello-wasm/Cargo.toml \
        --target wasm32-wasip1 --release

Produces `examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm`.

### 3. Lay out the smoke project

Use the checked-in scaffold (copy into a temporary directory):

    mkdir -p /tmp/clawasm-smoke/addons/clawasm /tmp/clawasm-smoke/modules
    cp target/release/libclawasm.dylib  /tmp/clawasm-smoke/addons/clawasm/
    cp examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm \
       /tmp/clawasm-smoke/modules/
    cp clawasm.gdextension              /tmp/clawasm-smoke/
    cp tests/godot-smoke/project.godot  /tmp/clawasm-smoke/
    cp tests/godot-smoke/main.tscn      /tmp/clawasm-smoke/

For the main script use the headless variant (calls `get_tree().quit()`):

    cat tests/godot-smoke/main.gd | \
      sed 's/^func _on_finished.*$/&\n\tget_tree().quit(code)/' \
      > /tmp/clawasm-smoke/main.gd

Or write it manually — add `get_tree().quit(code)` at the end of
`_on_finished` and `get_tree().quit(1)` at the end of `_on_failed`.

The final layout:

    /tmp/clawasm-smoke/
    ├── .godot/extension_list.cfg       # step 4 — headless only
    ├── project.godot
    ├── clawasm.gdextension
    ├── addons/clawasm/
    │   └── libclawasm.dylib
    ├── modules/
    │   └── hello-wasm.wasm
    ├── main.tscn
    └── main.gd

### 4. Pre-create the extension list (headless only)

When running headlessly Godot skips the editor startup that normally
writes `.godot/extension_list.cfg`. Without that file no GDExtension
loads, so `ClawEngine` is undefined and GDScript parse fails.

    mkdir -p /tmp/clawasm-smoke/.godot
    echo 'res://clawasm.gdextension' > /tmp/clawasm-smoke/.godot/extension_list.cfg

Skip this step if you open the project in the Godot GUI editor — the
editor writes the file automatically on first open.

### 5. Run

**GUI (interactive):** Open `/tmp/clawasm-smoke` in Godot, hit Play.

**Headless (automated):**

    WASMEDGE_BIN="$HOME/.wasmedge/bin/wasmedge" \
      /Applications/Godot.app/Contents/MacOS/Godot \
      --headless --path /tmp/clawasm-smoke

Expected output (headless):

    Initialize godot-rust (API v4.6.stable.official, ...)
    ...
    ClawEngine: registered module /tmp/clawasm-smoke/modules/hello-wasm.wasm
    [wasm] hello-wasm
    [wasm] exit 0

If you see `ClawEngine::start: spawn failed: ...`, WasmEdge is not on
`$PATH` — set `WASMEDGE_BIN` as above or export it before launching Godot.

## What this proves

- The `clawasm` cdylib loads in Godot 4 (extension entry point).
- `ClawEngine` is registered as a class and constructable from GDScript.
- `register_module` resolves `res://` paths via `ProjectSettings.globalize_path`.
- `start` spawns WasmEdge and streams `stdout_line` signals on the main thread.
- `finished` fires exactly once, after the last line of output.

## Smoke results

| Date | Platform | Godot | godot-rust | WasmEdge | Result |
| --- | --- | --- | --- | --- | --- |
| 2026-05-06 | macOS arm64 | 4.6.2 | 0.5.2 | 0.14.1 | ✅ PASS |
| — | Linux x86_64 | — | — | — | ⏳ pending |

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| `Can't open dynamic library` | Wrong arch (`x86_64` vs `arm64`) or missing rpath. Rebuild release on the same host. |
| `ClawEngine` not found (GDScript parse error) | `.godot/extension_list.cfg` missing (headless), or extension not loaded in editor. |
| No output, no error | WasmEdge wrote to stderr; connect `stderr_line` too. |
| `can't find crate for std` during wasm build | Homebrew `rustc` shadowing rustup; set `RUSTC=~/.cargo/bin/rustc`. |
| Hung after `start` | The wasm module loops forever. Call `engine.stop()` from a button or signal. |
