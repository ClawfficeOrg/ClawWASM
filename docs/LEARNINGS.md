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
