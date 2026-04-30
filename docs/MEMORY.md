# Project memory

Long-lived decisions, invariants, and ownership notes for ClawWASM. Read
this before reading code. See `.superpowers/skills/memory-keeper.md` for
how to maintain it.

> Newest entries on top. Append-only in spirit (do not rewrite history; if
> a decision is reversed, add a new dated entry that supersedes the old).

---

## 2026-04-29 — Autonomous-development substrate

ClawWASM uses a Ralph-loop + superpowers-skills setup for AI-driven
development:

- The binding contract for **all** AI agents is `AGENTS.md` at the repo root.
- Skills live in `.superpowers/skills/` and are referenced by `AGENTS.md` and
  `.github/copilot-instructions.md`.
- Autonomous iterations run via `bash ralph/loop.sh` which feeds
  `ralph/PROMPT.md` to a chosen adapter (`ralph/adapters/<name>.sh`) once per
  iteration. Active workplan lives in `ralph/PLAN.md`.

Non-negotiables: trunk-based, PR-only, Conventional Commits, two-reviewer
policy (`@CompewterTutor` + GPT-5.5).

## 2026-04-29 — Repo cleanup after the OpenClaw memory corruption

After the OpenClaw upgrade fragmented project state, all dangling feature
branches except `main` were pruned and the two open PRs (#6 sonnet review,
#8 engine scaffold) were merged. `main` at this point contains:

- Godot 4 plugin (`clawasm/`, godot-rust 0.5+).
- Stub `clawasm-engine` crate (feature-gated `with-wasmedge`).
- `examples/hello-wasm/` smoke binary (`wasm32-wasip1`).
- Wasm CI (`.github/workflows/wasm-ci.yml`).
- v0.1.0 tag at the pre-Godot-4 commit `8fc3069` (kept for archeology; v0.2.0
  will be the first "real" release).

## 2026-04-29 — Decision: Godot 4 only

We dropped Godot 3 (`gdnative`) support. `clawasm` uses `godot` ≥ 0.5
(crates.io version; the book version "v0.15" is unrelated). Reason:
upstream `gdnative` is unmaintained for 4.x. Migration completed in PR #7.

## 2026-04-12 — Decision: target `wasm32-wasip1`, not `wasm32-wasi`

Rust 1.84+ removed the `wasm32-wasi` triple. We use `wasm32-wasip1`
everywhere (CI, scripts, docs). Wasm crates that need to be cross-compiled
(`examples/hello-wasm`) carry their own `[workspace]` table to avoid
pulling host-only deps. PR #5.

## Standing invariants

1. `clawasm` (Godot plugin) and `clawasm-engine` (WasmEdge wrapper) are
   **host-only** crates. They must not be built for any wasm target.
2. `clawasm-engine` builds in stub mode by default; the real WasmEdge VM is
   gated behind `with-wasmedge`. Stub-mode tests must always pass.
3. CI must run on every PR and on every push to `feature/**`. Releases run
   on tag pushes (`v*`).
4. WasmEdge is pinned to **0.14.1** in CI tarball downloads. Bumps require
   a PR with regression notes.
5. Public API surface of `clawasm-engine` (Engine/Instance/Output) is stable;
   breaking changes require `feat(engine)!:` and a major bump.

## Owners

| Area                     | Owner                          |
| ------------------------ | ------------------------------ |
| Architecture / reviews   | `@CompewterTutor` (Claude)     |
| Second review            | GPT-5.5 reviewer agent         |
| Repo admin / releases    | Captain-Clawffice (Michael)    |
| Wasm CI                  | Ralph loop / Captain-Clawffice |
