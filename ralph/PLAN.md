# Ralph Plan — 2026-05-01

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = engine MVP. v0.3.0 = `ClawEngine` Godot node.
v0.4.0 = full headless CI coverage (shipped). v0.5.0+ = pre-built addon
bundle, in-process embedding, ironclaw/LLM wiring.
eventually run llm inference and tools like ironclaw in godot wasmedge
## Active task

### Cut v0.4.0 (release prep)

CI headless smoke green on both macOS and Linux. Bumping version and
finalising changelog.

- **Files:** `clawasm/Cargo.toml`, `CHANGELOG.md`,
  `tests/godot-smoke/README.md`, `ralph/PLAN.md`.

## Up next (ordered)

- [ ] **Pre-built addon bundle** — ship `addons/clawasm/` zip in
      `release.yml` so users don’t need Rust (resolves Q4).
- [ ] **In-process WasmEdge embedding** — revisit once a
      `wasmedge-sys` version compatible with WasmEdge 0.14.1 appears.
- [ ] **ironclaw / LLM tool wiring** — first GDScript API sketch for
      running inference tasks through a wasm module.

## Done this iteration block

- [x] feat(repo): add superpowers skills, Ralph loop, agents contract, CI/CD scaffolding (PR #9)
- [x] fix(clawasm): drop direct `wasmedge-sys` dep; route through `clawasm-engine` (PR #10)
- [x] feat(engine): v0.2.0 MVP — subprocess `Instance::run` (PR #11)
- [x] feat(godot): `ClawEngine` node + streaming `Runner` + smoke runbook (PR #12)
- [x] chore(release): v0.2.0 — bumped clawasm to 0.2.0, tagged (PR #13)
- [x] docs(smoke): headless macOS smoke GREEN — Godot 4.6.2, godot-rust 0.5.2, WasmEdge 0.14.1
- [x] ci(godot): headless Godot 4.6.2 CI smoke job, both platforms green (PR #15)
- [x] chore(release): v0.4.0 (this commit)

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error`?~~
  Resolved in PR #11: now required.
- **Q2:** ~~Headless Godot smoke in CI~~ — wired PR #15;
  `godot-smoke` job, Godot 4.6.2, green on macOS + Linux.
- **Q4:** Pre-built addon bundle — next slice.

## Archive

(empty — first iteration block)
