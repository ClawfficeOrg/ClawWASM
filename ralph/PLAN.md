# Ralph Plan — 2026-04-29

## North star

Stand up the autonomous-development substrate (this PR), then make `clawasm-engine`
real enough that the Godot plugin can run a wasm module end-to-end. Targeting
v0.2.0 = "engine MVP can load + run a wasip1 module from disk under WasmEdge".

## Active task

### Wire the v0.2.0 engine: `Engine::new` → real WasmEdge VM (feature-gated)

- **Files:** `clawasm/engine/src/lib.rs`, `clawasm/engine/Cargo.toml`,
  `clawasm/engine/tests/smoke.rs` (new).
- **Tests:**
  - Stub-mode unit test (always runs): `Engine::new()` returns `Ok`,
    `Instance::run` returns the canned stub output.
  - `#[cfg(feature = "with-wasmedge")]` integration test: load
    `examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm`,
    assert `exit_code == 0` and `stdout.contains("hello")`.
- **Acceptance:**
  - `cargo test -p clawasm-engine` green without WasmEdge installed.
  - `cargo test -p clawasm-engine --features with-wasmedge` green when
    WasmEdge 0.14.1 is present.
  - `clawasm/engine/README.md` documents both modes.
- **Blockers:** none — depends only on the present scaffolding PR landing first.

## Up next (ordered)

- [ ] **Fix `clawasm` host-crate `wasmedge-sys` pin** — the current
      `wasmedge-sys = "0.17"` does not compile against WasmEdge 0.14.1
      headers (see `docs/LEARNINGS.md` 2026-04-30). Pick one: downgrade
      to a 0.1x line compatible with 0.14.1, bump the WasmEdge install
      pin, or remove the direct dep and route WasmEdge use only through
      `clawasm-engine`. Re-enable `cargo clippy --workspace` and
      `cargo check -p clawasm` in CI once green.
- [ ] **`clawasm/engine`: implement `Instance::run` with WasmEdge VmBuilder**
      — replace the `bail!` stub, hook stdout capture, return real `Output`.
      Files: `clawasm/engine/src/lib.rs`. Tests: extend the feature-gated
      integration test from above.
- [ ] **Add `clawasm.gdextension` manifest + Godot smoke project skeleton**
      — under `tests/godot-smoke/`. Document load steps in
      `.superpowers/skills/godot-binding.md`. No CI runner yet.
- [ ] **CI matrix axis: `with-wasmedge` job** — install WasmEdge, run
      `cargo test -p clawasm-engine --features with-wasmedge`. Update
      `.github/workflows/ci.yml`.
- [ ] **`clawasm` plugin: expose `Engine` to Godot via a `ClawEngine` node**
      — `register/start/stop` methods, signals for stdout lines.
      Files: `clawasm/src/lib.rs`, new `clawasm/src/engine_node.rs`.
- [ ] **Release v0.2.0** — bump versions, CHANGELOG, tag. See
      `.superpowers/skills/release-engineering.md`.

## Done this iteration block

- [x] feat(repo): add superpowers skills, Ralph loop, agents contract, CI/CD scaffolding (PR pending — this one)

## Open questions

- **Q1:** Do we want the `with-wasmedge` CI job to be `continue-on-error: true`
  for the first month, or block PRs from day one? *Default assumption: not
  blocking until it has been green for two consecutive weeks.*
- **Q2:** Headless Godot smoke: GH Actions has a `godotengine/godot` container,
  but extension loading on Linux without a display server has been flaky.
  *Default: defer to v0.3.0; ship manual smoke instructions for v0.2.0.*

## Archive

(empty — first iteration block)
