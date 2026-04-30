---
name: Writing Plans
description: How to draft and maintain ralph/PLAN.md so the autonomous loop makes steady progress.
when_to_use: At the start of a Ralph iteration, after merging a PR, or when scope changes.
---

# Writing Plans

A plan is a *living* document, not a contract. The Ralph loop reads it every
iteration, so it must be:

1. **Skimmable in 30 seconds.** One-page rule: if it grows, archive old sections.
2. **Concrete.** Each task names the file(s) to touch, the test(s) to write, and
   the acceptance check.
3. **Ordered.** The first unchecked item is what the loop works on next.

## Structure

```markdown
# Ralph Plan — <ISO date>

## North star
One paragraph: what are we building this iteration block?

## Active task
A single H3 with the *one* thing being worked on right now.
Includes: files, tests, acceptance criteria, blockers.

## Up next (ordered)
- [ ] Short task A — file(s), acceptance.
- [ ] Short task B — file(s), acceptance.

## Done this iteration block
- [x] feat(scope): … (PR #N)

## Open questions
- Q1 — assumption being made until answered.

## Archive
Link to or summarize older plan blocks moved out of this file.
```

## Sizing

- **Active task** ≤ ~200 LOC of change. If bigger, split.
- **Up next** ≤ 7 items. More than that means re-plan.
- A task that can't be merged in a single PR is too big.

## Updating during the loop

After each Ralph iteration the agent must:

1. Tick the box for the completed task.
2. Move it under "Done this iteration block" with the merged PR number.
3. Promote the next "Up next" item to "Active task" with concrete file paths.
4. If a surprise was discovered, add it to `docs/LEARNINGS.md` (NOT the plan).

## When to reset

After a release tag or when the North star changes. Move the current plan into
the archive (link a commit hash) and start fresh.
