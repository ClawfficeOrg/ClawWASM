# Ralph Plan — 2026-05-01

## North star

Make `clawasm-engine` real enough that the Godot plugin can run a wasm
module end-to-end. v0.2.0 = engine MVP. v0.3.0 = `ClawEngine` Godot node.
v0.4.0 = headless CI. v0.5.0 = pre-built addon bundle (in flight).
v0.6.0+ = in-process embedding, ironclaw/LLM wiring.
eventually run llm inference and tools like ironclaw in godot wasmedge
## Active task

### Pre-built addon bundle in `release.yml` (this PR)

- **File edited:** `.github/workflows/release.yml` — new "Build addon bundle
  zip" step in the `release` job. After all platform builds complete and
  artifacts are flattened, assembles `addon-bundle/addons/clawasm/` with
  `clawasm.gdextension` + all three cdylibs + a drop-in `README.md`, then
  zips to `clawasm-addon-vX.Y.Z.zip` and attaches it to the release.
- **Dry-run verified** locally: correct zip structure, macOS dylib present,
  `find | xargs` pipeline works for all three extensions.
- **Acceptance:** `release.yml` CI green; GitHub Release for v0.5.0 includes
  `clawasm-addon-v0.5.0.zip` alongside the per-platform files.

## Up next (ordered)

- [ ] **Cut v0.5.0** — merge this PR, bump `clawasm` to 0.5.0, tag.
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
- [x] ci(godot): headless Godot 4.6.2 CI smoke job, both platforms green (PR #15)
- [x] chore(release): v0.4.0 (PR #16)
- [x] feat(release): pre-built addon bundle zip (this PR)

## Open questions

- **Q1:** ~~Do we want the `with-wasmedge` CI job to be `continue-on-error`?~~
  Resolved in PR #11: now required.
- **Q2:** ~~Headless Godot smoke in CI~~ — wired PR #15;
  `godot-smoke` job, Godot 4.6.2, green on macOS + Linux.
- **Q4:** ~~Pre-built addon bundle~~ — wired in this PR.

## Archive

(empty — first iteration block)
