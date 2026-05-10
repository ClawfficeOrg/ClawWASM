# `clawasm-engine`

Embedded WasmEdge wrapper for the ClawWASM Godot plugin. Exposes a small,
stable surface — `Engine`, `Instance`, `Output` — that the rest of the
project can rely on while the underlying WasmEdge integration evolves.

## Implementation status (v0.2.0)

The current implementation invokes the **`wasmedge` command-line binary**
as a subprocess and captures its stdout / stderr / exit code. This was
chosen over `wasmedge-sys` because the published `wasmedge-sys` lines
(0.4.x and 0.17.x) are both ABI-incompatible with the WasmEdge 0.14.1
release we pin in CI (see `docs/LEARNINGS.md` 2026-04-30).

Subprocess invocation:

- works against any WasmEdge release that ships a CLI,
- has no native build dependency on the consuming crate (no bindgen, no
  cmake), so `cargo check -p clawasm` works on a clean machine,
- keeps the public API stable for an in-process swap-in later.

When/if we move to in-process embedding, only the body of `Instance::run`
needs to change. Tracked in `ralph/PLAN.md`.

## Public API

The crate exposes:

- **`Engine`** — lightweight handle for locating the WasmEdge binary.
- **`Instance`** — a loaded `.wasm` module ready to run.
- **`Output`** — captured stdout/stderr + exit code from a completed run.
- **`Runner`** / **`Event`** — streaming subprocess runner used by the
  Godot `ClawEngine` node and kept for future WASM ↔ host bridge work.

> **Note (v0.6.0):** `LlmConfig` and `DEFAULT_LLAMA_CLI_BIN` were removed
> from this crate. Native LLM inference now lives in `clawasm/src/llm_node.rs`
> and is compiled via the `llama-cpp-2` crate (enabled with
> `--features with-llama`). See `ralph/PLAN.md` HANDOFF section.


```rust,no_run
use engine::Engine;

let engine = Engine::new()?;
// Optional: fail fast if WasmEdge isn't installed.
let _version = engine.probe()?;

let instance = engine.load("path/to/module.wasm")?;
let out = instance.run(&["arg1".into(), "arg2".into()])?;
println!("exit={} stdout={}", out.exit_code, out.stdout);
# Ok::<_, anyhow::Error>(())
```

## Configuration

| Variable        | Effect                                                                 |
| --------------- | ---------------------------------------------------------------------- |
| `WASMEDGE_BIN`  | Path to the `wasmedge` binary. Defaults to `wasmedge` on `$PATH`.      |

## Testing

| Command                                                                      | Requires WasmEdge? | What it tests                              |
| ---------------------------------------------------------------------------- | ------------------ | ------------------------------------------ |
| `cargo test -p clawasm-engine`                                               | no                 | Unit tests (env, load validation, probe).  |
| `cargo test -p clawasm-engine --features with-wasmedge`                      | yes                | Above + end-to-end `hello-wasm` smoke.     |

The `with-wasmedge` feature only gates the integration test; library
behaviour is identical with and without it.

## Installing WasmEdge

The official installer drops everything into `$HOME/.wasmedge`:

```sh
curl -sSfL https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh \
  | bash -s -- --version=0.14.1 --path=$HOME/.wasmedge
export PATH="$HOME/.wasmedge/bin:$PATH"
```

Or set `WASMEDGE_BIN=$HOME/.wasmedge/bin/wasmedge` and skip the PATH edit.
