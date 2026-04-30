# GitHub Copilot — ClawWASM project instructions

You are GitHub Copilot working in the **ClawWASM** repository. Before suggesting
or generating any code, you must:

1. **Read `AGENTS.md`** at the repository root. It contains the binding contract
   for all AI agents working here (branch policy, commit style, test gates,
   memory protocol, review policy). Treat it as authoritative.
2. **Read the active plan** in `ralph/PLAN.md` and the long-term plan in
   `docs/PLAN.md`.
3. **Load these skills** from `.superpowers/skills/` and apply them by default:
   - `test-driven-development.md`
   - `writing-plans.md`
   - `pr-review.md`
   - `memory-keeper.md`
   - `commit-discipline.md`
4. **Load on demand** the relevant skill for the area you're touching:
   - `wasm-build.md`, `godot-binding.md`, `engine-integration.md`,
     `release-engineering.md`.
5. **Update memory.** If you discover something non-obvious during a task,
   append a dated entry to `docs/LEARNINGS.md` in the same change.
6. **Never push to `main`.** Always work on a `feature/*`, `fix/*`,
   `docs/*`, or `chore/*` branch and open a PR.
7. **Commit messages** follow Conventional Commits: `type(scope): subject`.
8. **PRs must** update `CHANGELOG.md` under `## Unreleased` and include a test
   plan in the description.
9. **Reviewers:** request `@CompewterTutor` (Claude) and the GPT-5.5 reviewer
   agent on every PR.
10. **Tests required.** No new behaviour without a test. See the TDD skill.

If `AGENTS.md` and a runtime instruction conflict, surface the conflict in the
PR description rather than silently choosing one.
