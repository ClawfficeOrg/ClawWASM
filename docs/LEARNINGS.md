# Learnings

Append-only lab notebook of non-obvious things discovered during
development. See `.superpowers/skills/memory-keeper.md` for the protocol.

> Format: dated H3 entries, newest at the bottom (chronological). Each entry
> is 1–4 sentences and links to the PR that produced it where possible.

---

### 2026-04-12 — `wasm32-wasi` was removed in Rust 1.84+

Tried `rustup target add wasm32-wasi` on stable Rust ≥ 1.84 and got "error:
toolchain ... does not support target 'wasm32-wasi'". Switched to
`wasm32-wasip1` everywhere (CI, scripts, docs). PR #5.

### 2026-04-12 — Workspace resolver mismatch breaks isolated wasm builds

Building `examples/hello-wasm` from the workspace root pulled in
`wasmedge-sys` (a host-only dep) and failed for `wasm32-wasip1`. Fix:
`examples/hello-wasm/Cargo.toml` declares its own empty `[workspace]`
table, and CI uses `--manifest-path` to build it in isolation. The root
workspace also pins `resolver = "2"`. PRs #5, #6.

### 2026-04-29 — godot-rust crate version vs book version

The godot-rust *book* refers to "v0.15" but the crate published to
crates.io is in the 0.5.x range. `Cargo.toml` should pin `godot = "0.5"`
(or higher), not `0.15`. PR #7.

### 2026-04-29 — Stale branches after the OpenClaw memory corruption

After the upgrade, several remote branches still existed for already-merged
PRs (#1–#5, #7) and a duplicate `feature/v0.1.0-wasm-hello`. They were
pruned along with `feature/docs-quickstart` (a stale fork that would have
reverted later work). Always check `git log origin/main..origin/<branch>`
before deleting; deleting a branch that's only an ancestor of `main` is
safe.

### 2026-04-30 — `wasmedge-sys 0.17.5` is ABI-incompatible with WasmEdge 0.14.1

Wiring CI to actually compile `clawasm` (the Godot host crate) against
libwasmedge surfaced ~38 type errors in `wasmedge-sys 0.17.5/src/types.rs`
(`WasmEdge_ValType` is a struct in 0.14.1 headers but the bindings expect
an integer-like enum). The 0.17.x line targets newer WasmEdge releases.
Resolution options for the engine-MVP PR: (a) downgrade `wasmedge-sys`
to a 0.1x version compatible with WasmEdge 0.14.1, (b) bump the WasmEdge
pin to a release the 0.17.x bindings support, or (c) route all WasmEdge
use through `clawasm-engine` (feature-gated) and drop the direct dep
from `clawasm`. Until that PR lands, scaffolding-CI scopes
`cargo clippy` to `-p clawasm-engine --no-default-features` and skips
host-side `cargo check -p clawasm`. PR #9.
