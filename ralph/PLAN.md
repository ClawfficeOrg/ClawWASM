# Ralph Plan — 2026-04-30

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = "engine MVP can load + run a wasip1 module
from disk under WasmEdge". v0.2.0 engine is shipping in PR #11
(subprocess-to-`wasmedge` implementation); v0.3.0 wires it into Godot
and revisits in-process embedding.

## Active task

### Expose `Engine` to Godot via a `ClawEngine` node

With the engine MVP shipped (PR #11), the next bite is wiring it into
Godot so a `.gd` script can `register_module()` / `start()` / `stop()`
a wasm module and receive `stdout_line` signals.

- **Files:** new `clawasm/src/engine_node.rs`, edit `clawasm/src/lib.rs`.
- **Tests:** unit tests for the pure-Rust glue (path resolution, signal
  payloads). Manual Godot smoke deferred to `tests/godot-smoke/`.
- **Acceptance:**
  - `cargo check -p clawasm` and `cargo clippy --workspace` green.
  - A scripted Godot scene can attach a `ClawEngine` node, point it at
    `hello-wasm.wasm`, and receive at least one `stdout_line` signal.

## Up next (ordered)

- [ ] **Add `clawasm.gdextension` manifest + Godot smoke project skeleton**
      — under `tests/godot-smoke/`. Document load steps in
      `.superpowers/skills/godot-binding.md`. No CI runner yet.
- [ ] **Move from subprocess to in-process WasmEdge embedding** — once a
      `wasmedge-sys` (or alternative binding) version compatible with
      our pinned WasmEdge release exists, swap the body of
      `Instance::run` without changing the public API. Track stdout via
      a pipe-redirect trick or a custom WASI host.
- [ ] **Release v0.2.0** — bump versions, CHANGELOG, tag. See
      `.superpowers/skills/release-engineering.md`.

## Done this iteration block

- [x] feat(repo): add superpowers skills, Ralph loop, agents contract, CI/CD scaffolding (PR #9)
- [x] fix(clawasm): drop direct `wasmedge-sys` dep; route through `clawasm-engine`
      path dep; restore workspace clippy + `cargo check -p clawasm` in CI (PR #10)
- [x] feat(engine): v0.2.0 MVP — `Instance::run` shells out to the `wasmedge`
      CLI; full unit-test suite + feature-gated `hello-wasm` integration test;
      `engine-with-wasmedge` CI job promoted from `continue-on-error` to required (PR #11)

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error: true`
  for the first month?~~ Resolved in PR #11: now required.
- **Q2:** Headless Godot smoke: GH Actions has a `godotengine/godot` container,
  but extension loading on Linux without a display server has been flaky.
  *Default: defer to v0.3.0; ship manual smoke instructions for v0.2.0.*
- **Q3 (new):** Subprocess vs in-process embedding — for v0.2.0 we ship
  subprocess. The Godot integration will need to surface partial stdout
  (line-by-line signals) which is awkward over `Command::output()`. Plan
  is to use `Stdio::piped()` and a reader thread inside `ClawEngine` for
  v0.3.0 streaming, then revisit in-process when bindings stabilise.

## Archive

(empty — first iteration block)
