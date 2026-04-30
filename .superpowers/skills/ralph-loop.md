---
name: Ralph Loop
description: How the autonomous Ralph loop drives ClawWASM development between human check-ins.
when_to_use: Before invoking ralph/loop.sh, or when authoring/updating ralph/PROMPT.md.
---

# Ralph Loop

The Ralph loop is a thin shell around an LLM agent. It reads a stable
prompt, lets the agent execute one bounded slice of work, commits the result,
and repeats. The point: turn "a big roadmap" into "many small merged PRs"
while a human sleeps.

## The contract

Each iteration the agent must:

1. **Read context, in order:**
   - `AGENTS.md`
   - `docs/MEMORY.md`
   - `docs/LEARNINGS.md`
   - `ralph/PLAN.md`
   - `docs/PLAN.md` and `docs/TODO.md` (skim, don't memorize)
2. **Pick the next task.** Always the *Active task* in `ralph/PLAN.md`. If
   none exists, promote the first unchecked item under "Up next".
3. **Branch.** `git checkout -B feature/<short-kebab>` off latest `main`.
4. **TDD.** Write the failing test, then the code (see test-driven-development skill).
5. **Run quality gates** locally: `cargo fmt --check`, `cargo clippy -D warnings`,
   `cargo test --workspace`, plus `bash scripts/test-wasm.sh` if wasm bits changed.
6. **Update artifacts:**
   - `CHANGELOG.md` `## Unreleased` block.
   - `docs/LEARNINGS.md` if surprising.
   - `ralph/PLAN.md` (tick boxes, promote next task).
7. **Commit** with Conventional Commits + `-s`.
8. **Push** the branch and **open a PR** via `gh pr create`. Request reviewers
   `@CompewterTutor` and the GPT-5.5 agent.
9. **Stop.** One PR per iteration. Exit cleanly.

## The runner (`ralph/loop.sh`)

The runner is intentionally dumb:

```bash
while [ ! -f ralph/STOP ]; do
  agent --prompt "$(cat ralph/PROMPT.md)"   # one bounded iteration
  sleep "${RALPH_SLEEP:-30}"
done
```

Adapter scripts in `ralph/adapters/` map `agent` to whatever harness the user
prefers (Claude Code, Codex CLI, Aider, OpenHands, …).

## Stopping

To halt the loop without `kill -9`:

```
touch ralph/STOP
```

The next iteration exits cleanly. Delete `ralph/STOP` to resume.

## Bounding the iteration

Long iterations cause merge conflicts and review pain. Hard caps:

- ≤ ~200 LOC of code change per PR.
- ≤ 30 minutes of agent wall clock per iteration (enforced by adapter timeout).
- ≤ 1 PR per iteration.

If the active task can't fit, the agent must split it (edit `ralph/PLAN.md`)
*before* writing code.

## When CI fails

If the freshly opened PR's CI fails:

1. The next iteration sees the open PR, checks out its branch, fixes the
   failure, pushes, and stops. **Do not open a second PR for the same task.**
2. If three iterations in a row fail the same way, the agent writes a
   blocker entry to `ralph/PLAN.md` "Open questions" and waits for human input.

## Memory hygiene

Skill files, `AGENTS.md`, and the memory docs are *the* knowledge base. The
LLM context resets every iteration; everything that must persist lives on
disk.
