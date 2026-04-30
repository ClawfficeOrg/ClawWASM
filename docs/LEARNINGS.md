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

**Resolved (PR #10):** Option (c) — `clawasm` no longer depends on
`wasmedge-sys` directly; the engine wrapper crate owns the native dep,
and the host plugin gets a stub-mode `cargo check` for free. `clawasm`
exposes a forwarding `with-wasmedge` feature that pulls
`clawasm-engine/with-wasmedge` for callers that want the native path.
CI restored `cargo clippy --workspace` and `cargo check -p clawasm`.

### 2026-04-30 — Engine v0.2.0 ships subprocess to `wasmedge`, not in-process embedding

`wasmedge-sys 0.4.x` (which `clawasm-engine` originally pinned) and
`0.17.x` are both ABI-incompatible with the WasmEdge 0.14.1 release
we install in CI — 0.4.x references removed `WasmEdge_ImportObject*`
and `WasmEdge_HostRegistration_WasmEdge_Process` symbols; 0.17.x
expects the newer `WasmEdge_ValType` ABI. Rather than blocking the
engine MVP on finding a binding version that lines up (or bumping
the WasmEdge install pin and re-validating downstream), v0.2.0
implements `Instance::run` by `Command`-ing the `wasmedge` CLI
binary that we already install for the wasm-smoke job. Pros: zero
native build deps for consumers, works against any WasmEdge release
that ships a CLI, public API stays stable for an in-process swap-in.
Cons: per-invocation process-launch overhead, no fine-grained host
function injection. Tracked as a v0.3.0+ follow-up in `ralph/PLAN.md`
(Q3). PR #11.

### 2026-05-01 — `godot-rust` v0.5 signal emit takes `&GString`, not `GString`

Wiring `ClawEngine` exposed two godot-rust 0.5 quirks:
(1) `signals().<name>().emit(...)` takes its arguments by reference
(`&GString`, `i64`), not by value, so signal payloads must be borrowed.
(2) `GString: From<String>` is *not* implemented — only
`From<&String>` and `From<&str>`. Construct via `GString::from(&s)` for
owned strings. Catching this at compile time saved a confusing runtime
signal panic later. Surface in any future Rust→GDScript glue.
