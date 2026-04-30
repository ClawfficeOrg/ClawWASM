# Ralph Plan — 2026-05-01

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = engine MVP (subprocess, shipped in PR #11).
v0.3.0 = `ClawEngine` Godot node streams stdout/stderr to GDScript via
signals (in flight, this iteration). v0.4.0+ = revisit in-process
WasmEdge embedding once bindings stabilise.

## Active task

### Land the `ClawEngine` node + Godot smoke runbook (PR open)

- **Files added/edited:**
  - `clawasm/engine/src/stream.rs` — new streaming `Runner` / `Event`.
  - `clawasm/engine/src/lib.rs` — `Instance::stream(args)` helper.
  - `clawasm/src/engine_node.rs` — `ClawEngine` Godot node.
  - `clawasm/src/lib.rs` — re-export `ClawEngine`.
  - `clawasm.gdextension` — gdextension manifest.
  - `tests/godot-smoke/{README.md,main.gd}` — manual runbook.
  - `CHANGELOG.md`, `docs/LEARNINGS.md` — entries for the above.
- **Tests:** `cargo test -p clawasm-engine --no-default-features` covers
  the streaming runner end-to-end against `sh`. `cargo clippy --workspace`
  green. Godot-side smoke is documented but manual.
- **Acceptance:** ✅ Workspace builds in stub mode, ✅ engine streams
  stdout lines + exit code, ✅ `ClawEngine` compiles against
  `godot-rust 0.5`, ⏳ manual Godot smoke (operator).

## Up next (ordered)

- [ ] **Cut v0.2.0** — bump `clawasm-engine` to 0.2.0 (already), bump
      `clawasm` to 0.2.0, finalize `CHANGELOG.md` Unreleased → v0.2.0,
      tag, run `release.yml`. See `.superpowers/skills/release-engineering.md`.
- [ ] **Run the manual Godot smoke** on macOS + Linux and capture
      results in `docs/LEARNINGS.md`. If green on both, document the
      v0.3.0 release.
- [ ] **Move from subprocess to in-process WasmEdge embedding** — once a
      `wasmedge-sys` (or alternative binding) version compatible with
      our pinned WasmEdge release exists, swap the body of
      `Instance::run` and `Instance::stream` without changing the
      public API.

## Done this iteration block

- [x] feat(repo): add superpowers skills, Ralph loop, agents contract, CI/CD scaffolding (PR #9)
- [x] fix(clawasm): drop direct `wasmedge-sys` dep; route through `clawasm-engine` (PR #10)
- [x] feat(engine): v0.2.0 MVP — subprocess `Instance::run` (PR #11)
- [x] feat(godot): `ClawEngine` node + streaming `Runner` + smoke runbook (this PR)

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error`?~~
  Resolved in PR #11: now required.
- **Q2:** Headless Godot smoke in CI — still deferred. Manual runbook
  lives at `tests/godot-smoke/README.md` for v0.2.0/v0.3.0.
- **Q3:** ~~Subprocess vs in-process embedding for streaming.~~ Resolved
  in this PR: streaming `Runner` uses piped subprocess + reader threads;
  public API stays stable for an in-process swap-in later.
- **Q4 (new):** When/how do we publish a pre-built `addons/clawasm/`
  bundle with the cdylib + manifest so users don't need a Rust toolchain?
  Probably part of `release.yml` once v0.2.0 ships.

## Archive

(empty — first iteration block)
