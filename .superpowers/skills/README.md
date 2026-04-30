# Skills index

Markdown skill files loaded by superpowers-aware agents (Claude Code,
Cursor, Copilot via `.github/copilot-instructions.md`, etc.).

## Always-on (load every session)

| File                          | Purpose                                         |
| ----------------------------- | ----------------------------------------------- |
| `test-driven-development.md`  | Red → green → refactor; tests are mandatory.    |
| `writing-plans.md`            | How to maintain `ralph/PLAN.md`.                |
| `pr-review.md`                | Two-reviewer policy + checklist.                |
| `memory-keeper.md`            | `MEMORY.md` vs `LEARNINGS.md` protocol.         |
| `commit-discipline.md`        | Conventional Commits + atomic commits.          |
| `ralph-loop.md`               | The autonomous Ralph development loop contract. |

## On-demand (load when the area is touched)

| File                       | Trigger                                             |
| -------------------------- | --------------------------------------------------- |
| `wasm-build.md`            | Anything wasm/WasmEdge/`examples/`.                 |
| `godot-binding.md`         | `clawasm/src/*` or godot-rust deps.                 |
| `engine-integration.md`    | `clawasm/engine/*`.                                 |
| `release-engineering.md`   | Cutting a versioned release.                        |

## Adding a new skill

1. Create `<kebab-name>.md` with a YAML front-matter `name`/`description`/
   `when_to_use` block.
2. Add it to the table above.
3. Reference it from `AGENTS.md` if it's always-on.
4. PR with `docs(skills): add <name>` and request review as usual.
