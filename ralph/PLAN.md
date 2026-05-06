# Ralph Plan — 2026-05-01

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = engine MVP (subprocess, shipped in PR #11).
v0.3.0 = `ClawEngine` Godot node streams stdout/stderr to GDScript via
signals (in flight, this iteration). v0.4.0+ = revisit in-process
WasmEdge embedding once bindings stabilise.
eventually run llm inference and tools like ironclaw in godot wasmedge
## Active task

### Wire headless Godot CI smoke (this PR)

- **Files added/edited:**
  - `.github/workflows/ci.yml` — new `godot-smoke` job + `GODOT_VERSION` env.
  - `tests/godot-smoke/main_headless.gd` — headless GDScript variant.
  - `tests/godot-smoke/main_headless.tscn` + `project_headless.godot`.
  - `CHANGELOG.md`, `ralph/PLAN.md` — this update.
- **Acceptance:** CI green on this branch (fmt/clippy/test/godot-smoke).

## Up next (ordered)

- [ ] **Linux Godot smoke result** — capture the CI run result in
      `tests/godot-smoke/README.md` smoke-results table once the
      headless CI job goes green.
- [ ] **Pre-built addon bundle** — ship `addons/clawasm/` zip in
      `release.yml` so users don’t need Rust (Q4 from v0.3.0).
- [ ] **In-process WasmEdge embedding** — revisit once a
      `wasmedge-sys` version compatible with WasmEdge 0.14.1 appears.
- [ ] **ironclaw / LLM tool wiring** — first GDScript API sketch.

## Done this iteration block

- [x] feat(repo): add superpowers skills, Ralph loop, agents contract, CI/CD scaffolding (PR #9)
- [x] fix(clawasm): drop direct `wasmedge-sys` dep; route through `clawasm-engine` (PR #10)
- [x] feat(engine): v0.2.0 MVP — subprocess `Instance::run` (PR #11)
- [x] feat(godot): `ClawEngine` node + streaming `Runner` + smoke runbook (PR #12)
- [x] chore(release): v0.2.0 — bumped clawasm to 0.2.0, tagged (PR #13)
- [x] docs(smoke): headless macOS smoke GREEN — Godot 4.6.2, godot-rust 0.5.2, WasmEdge 0.14.1
- [x] ci(godot): headless Godot 4.2.2 CI smoke job (this PR)

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error`?~~
  Resolved in PR #11: now required.
- **Q2:** ~~Headless Godot smoke in CI~~ — wired in this PR;
  `godot-smoke` job uses `.godot/extension_list.cfg` + Godot 4.2.2.
- **Q3:** ~~Subprocess vs in-process embedding for streaming.~~ Resolved
  in this PR: streaming `Runner` uses piped subprocess + reader threads;
  public API stays stable for an in-process swap-in later.
- **Q4 (new):** When/how do we publish a pre-built `addons/clawasm/`
  bundle with the cdylib + manifest so users don't need a Rust toolchain?
  Probably part of `release.yml` once v0.2.0 ships.

## Archive

(empty — first iteration block)
