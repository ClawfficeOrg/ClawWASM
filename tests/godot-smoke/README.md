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

## Steps

### 1. Build the host cdylib

```sh
cargo build -p clawasm --release
```

Produces, depending on your platform:

- `target/release/libclawasm.dylib` (macOS)
- `target/release/libclawasm.so` (Linux)
- `target/release/clawasm.dll` (Windows)

### 2. Build the wasm module

```sh
cargo build --manifest-path examples/hello-wasm/Cargo.toml \
  --target wasm32-wasip1 --release
```

Produces `examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm`.

### 3. Lay out the smoke project

Create a fresh Godot project directory and copy in:

```
my-clawasm-smoke/
├── project.godot                   # any 4.2+ project, main scene = main.tscn
├── clawasm.gdextension             # copy from repo root
├── addons/clawasm/
│   └── libclawasm.{dylib,so,dll}   # from step 1
├── modules/
│   └── hello-wasm.wasm             # from step 2
├── main.tscn                       # a Node scene with main.gd attached
└── main.gd                         # copy from this directory
```

### 4. Run

Open the project in Godot, hit Play. Expected output:

```
[wasm] hello, ClawWASM!
[wasm] exit 0
```

If you see `ClawEngine::start: spawn failed: ...` instead, WasmEdge is
not on `$PATH` — either call `engine.set_wasmedge_binary("/abs/path/to/wasmedge")`
in `_ready()` or export `WASMEDGE_BIN` before launching Godot.

## What this proves

- The `clawasm` cdylib loads in Godot 4 (extension entry point).
- `ClawEngine` is registered as a class and constructable from GDScript.
- `register_module` resolves `res://` paths via `ProjectSettings.globalize_path`.
- `start` spawns WasmEdge and streams `stdout_line` signals on the main thread.
- `finished` fires exactly once, after the last line of output.

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| `Can't open dynamic library` | Wrong arch (`x86_64` vs `arm64`) or missing rpath. Rebuild release on the same host. |
| `ClawEngine` not found | `clawasm.gdextension` not loaded; check *Project > Project Settings > GDExtensions*. |
| No output, no error | WasmEdge wrote to stderr and your guest doesn't `println!`. Connect `stderr_line` too. |
| Hung after `start` | The wasm module loops forever. Call `engine.stop()` from a button or signal. |
