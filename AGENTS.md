# AGENTS.md — ClawWASM agent contract

This file is the single source of truth for **any** AI coding agent working in
this repository (Claude Code, Codex CLI, Cursor, GitHub Copilot, Aider, OpenHands,
Devin, etc.). Tool-specific files (`.github/copilot-instructions.md`,
`.cursorrules`, `CLAUDE.md`, …) should be thin pointers to this document.

> If you are an agent reading this: **read this whole file before touching code.**
> Then read `docs/MEMORY.md`, `docs/LEARNINGS.md`, and the active plan in
> `ralph/PLAN.md`. Skills you should load live in `.superpowers/skills/`.

---

## 1. Mission

ClawWASM is a Rust + WasmEdge project that embeds a minimal OpenClaw-like
gateway inside a WASM sandbox so Clawffice-Space nodes can run on
WASM-capable targets (Godot 4 host, browsers, SteamDeck, consoles) without
Docker/Podman.

The repository hosts:

| Path                  | Role                                                           |
| --------------------- | -------------------------------------------------------------- |
| `clawasm/`            | Native Godot 4 plugin (`cdylib`), godot-rust 0.5+ bindings.    |
| `clawasm/engine/`     | Embedded WasmEdge engine wrapper (feature-gated).              |
| `examples/hello-wasm/`| Minimal `wasm32-wasip1` smoke test, isolated from workspace.   |
| `docs/`               | Plan, TODO, guidelines, tests, architecture review, memory.    |
| `scripts/`            | Build/run helpers.                                             |
| `ralph/`              | Ralph-loop runner (autonomous dev driver).                     |
| `.superpowers/`       | Skill files loaded by superpowers-aware agents.                |

## 2. Non-negotiable rules

1. **Trunk-based, PR-only.** Never push directly to `main`. Open a PR from a
   feature branch named `feature/<short-kebab>` (or `fix/`, `docs/`, `chore/`).
2. **Every PR must:** pass CI, update `CHANGELOG.md` under `## Unreleased`,
   include a brief test plan, and reference the plan/issue it addresses.
3. **Conventional Commits + semver.** Commit messages follow
   `type(scope): subject`. Releases are tagged `vMAJOR.MINOR.PATCH`. See
   `RELEASING.md`.
4. **Never rename the repo or `clawasm` crate** without explicit approval from
   Michael (Captain-Clawffice).
5. **Tests before merge.** New behaviour requires at least one test (unit or
   integration). See `.superpowers/skills/test-driven-development.md`.
6. **Update memory.** When you discover something non-obvious about the build,
   the runtime, the host plugin, or a tool, append a dated entry to
   `docs/LEARNINGS.md` *in the same PR* that caused the discovery.
7. **No `unwrap()` / `expect()` in library code** without a `// SAFETY:` or
   `// PANIC:` comment justifying it.
8. **Reviews:** every PR must request review from `@CompewterTutor` (Claude /
   me) **and** the GPT-5.5 reviewer agent. See `.superpowers/skills/pr-review.md`.

## 3. Branch & PR conventions

```
feature/<short-desc>   # new functionality
fix/<short-desc>       # bug fixes
docs/<short-desc>      # docs-only
chore/<short-desc>     # build, CI, deps
refactor/<short-desc>  # internal-only changes
```

Commit subjects: `feat(engine): add WASI preopen mapping`,
`fix(ci): pin WasmEdge to 0.14.1`, `docs(readme): clarify wasip1 usage`, etc.

Allowed types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `ci`, `build`,
`perf`, `style`, `revert`. A `!` after the type or a `BREAKING CHANGE:` footer
signals a major bump.

## 4. The development loop (Ralph)

The autonomous development loop lives in `ralph/`:

- `ralph/PROMPT.md` — the **stable** prompt the loop feeds the agent each iteration.
- `ralph/PLAN.md` — the **active** working plan; the agent edits this as work progresses.
- `ralph/loop.sh` — the runner. One iteration = read prompt + plan, execute one slice, commit, push, open/update PR.
- `ralph/STOP` — if this file exists, the loop exits cleanly at the start of the next iteration.

Read `.superpowers/skills/ralph-loop.md` for the full protocol before running.

## 5. Skills

Skills are loadable markdown contracts in `.superpowers/skills/`. Load these by
default in every session:

- `test-driven-development.md`
- `writing-plans.md`
- `pr-review.md`
- `memory-keeper.md`
- `commit-discipline.md`

Load on demand:

- `wasm-build.md` — when touching wasm targets, WasmEdge, or `examples/`.
- `godot-binding.md` — when touching `clawasm/src/*` or godot-rust deps.
- `engine-integration.md` — when touching `clawasm/engine/`.
- `release-engineering.md` — when cutting a release.

## 6. Memory protocol

There are three persistent memory files. Treat them like a logbook:

| File                    | Contains                                                  | Update frequency        |
| ----------------------- | --------------------------------------------------------- | ----------------------- |
| `docs/MEMORY.md`        | Long-lived project facts: decisions, invariants, owners.  | When a decision is made.|
| `docs/LEARNINGS.md`     | Dated, append-only "I discovered X" entries.              | Every PR with a surprise.|
| `ralph/PLAN.md`         | Current iteration plan (mutable, may be reset on release).| Every Ralph iteration.  |

Never delete entries from `LEARNINGS.md`; only append. If something becomes
obsolete, add a new entry that supersedes it and link to the old one.

## 7. Quality gates (must pass locally before pushing)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace --all-targets
# wasm smoke test (when touching wasm bits):
bash scripts/test-wasm.sh
```

CI runs these on every PR. See `.github/workflows/ci.yml`.

## 8. When in doubt

- Read `docs/PLAN.md` for the long-term embedding plan.
- Read `docs/TODO.md` for the semver roadmap.
- Read `docs/guidelines.md` for coding standards.
- Read `docs/ARCHITECTURE_REVIEW_sonnet.md` for known weaknesses.
- If still unclear: write your assumption into `ralph/PLAN.md` under
  "Open questions" and proceed conservatively.
