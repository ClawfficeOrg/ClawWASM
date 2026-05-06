# Ralph Plan — 2026-05-01

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = engine MVP (subprocess, shipped in PR #11).
v0.3.0 = `ClawEngine` Godot node streams stdout/stderr to GDScript via
signals (in flight, this iteration). v0.4.0+ = revisit in-process
WasmEdge embedding once bindings stabilise.
eventually run llm inference and tools like ironclaw in godot wasmedge
## Active task

### Cut v0.3.0 and open next-slice PR

`ClawEngine` PR (#12) is fully green. Acceptance criteria all met.
Merge PR #12 → bump `clawasm` to 0.3.0 → move CHANGELOG Unreleased → v0.3.0
→ tag → start next feature slice.

**Files to touch for release commit:**
- `clawasm/Cargo.toml` — bump `version` to `"0.3.0"`.
- `CHANGELOG.md` — move Unreleased block → `[v0.3.0] - 2026-05-06`.

**Next feature candidates (pick one for v0.4.0 slice):**
1. Headless Godot CI smoke (wire `.godot/extension_list.cfg` trick into CI).
2. In-process WasmEdge embedding (swap `Command` for a `wasmedge-sys`
   binding version compatible with WasmEdge 0.14.1, if one exists).
3. Pre-built addon bundle in `release.yml` (`addons/clawasm/` zip).
4. ironclaw / LLM tool wiring — first GDScript API sketch.

## Up next (ordered)

- [ ] **Cut v0.3.0** — merge PR #12, bump `clawasm` to 0.3.0,
      finalize CHANGELOG Unreleased → v0.3.0, tag. Linux smoke still
      pending but can follow in a docs patch.
- [ ] **Headless Godot CI** — add a CI job that creates
      `.godot/extension_list.cfg`, builds the cdylib, and runs the
      headless smoke (resolves Q2).
- [ ] **Pre-built addon bundle** — ship `addons/clawasm/` zip in
      `release.yml` so users don't need Rust toolchain (Q4).
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

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error`?~~
  Resolved in PR #11: now required.
- **Q2:** Headless Godot smoke in CI — `.godot/extension_list.cfg`
  trick now documented; CI wiring deferred to the next slice.
- **Q3:** ~~Subprocess vs in-process embedding for streaming.~~ Resolved
  in this PR: streaming `Runner` uses piped subprocess + reader threads;
  public API stays stable for an in-process swap-in later.
- **Q4 (new):** When/how do we publish a pre-built `addons/clawasm/`
  bundle with the cdylib + manifest so users don't need a Rust toolchain?
  Probably part of `release.yml` once v0.2.0 ships.

## Archive

(empty — first iteration block)
