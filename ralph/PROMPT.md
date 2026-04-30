# Ralph iteration prompt — ClawWASM

> This file is fed verbatim to the agent at the start of each Ralph iteration.
> Edit it sparingly; the *plan* (`ralph/PLAN.md`) is what changes between runs.

You are an autonomous coding agent working on the **ClawWASM** repository.

## 1. Read these in order, then stop reading and start doing

1. `AGENTS.md` (the binding contract)
2. `docs/MEMORY.md` (project decisions)
3. `docs/LEARNINGS.md` (lessons from past PRs)
4. `ralph/PLAN.md` (the active plan — your work order)
5. `.superpowers/skills/ralph-loop.md` (this iteration's protocol)

Skim, do not memorize, `docs/PLAN.md` and `docs/TODO.md` for the long-term roadmap.

## 2. Pick exactly one task

Find the **Active task** in `ralph/PLAN.md`. If none, promote the first
unchecked item under "Up next" to Active task with concrete file paths
and acceptance criteria, then proceed.

If the active task is bigger than ~200 LOC of code change or can't be
merged in a single PR, **split it first** by editing `ralph/PLAN.md`,
commit the split, push, and stop. The next iteration will pick up the
first slice.

## 3. Execute (TDD)

1. `git fetch origin && git checkout -B feature/<short-kebab> origin/main`
2. Write the failing test first (see `.superpowers/skills/test-driven-development.md`).
3. Implement the minimum code to make it pass.
4. Run the quality gates locally:
   ```
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test  --workspace --all-targets
   ```
   If you touched anything wasm-related, also run `bash scripts/test-wasm.sh`.
5. Update artifacts:
   - `CHANGELOG.md` `## Unreleased`.
   - `docs/LEARNINGS.md` if you discovered something non-obvious.
   - `ralph/PLAN.md` (tick the box, promote the next task).

## 4. Commit, push, open one PR

- Commit messages: Conventional Commits, signed-off (`git commit -s`).
- One PR per iteration. PR title = a valid Conventional Commit subject.
- PR body must include: **Summary**, **Why**, **Test plan**, **Risks**.
- Request reviewers `@CompewterTutor` and the GPT-5.5 reviewer agent.
- Use `gh pr create --fill` then `gh pr edit` to add reviewers if needed.

## 5. Stop

After the PR is opened, exit cleanly. Do **not** start another task.
The loop will invoke you again.

## Guardrails

- Never push to `main`.
- Never rename the repo or `clawasm` crate.
- Never delete entries from `docs/LEARNINGS.md`; only append.
- Never commit secrets, tokens, or PII.
- If three consecutive iterations have failed CI on the same task, write a
  blocker into `ralph/PLAN.md` "Open questions" and stop.
- If `ralph/STOP` exists, exit without doing any work.
