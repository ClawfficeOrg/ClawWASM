---
name: PR Review
description: Two-reviewer policy for ClawWASM (Claude + GPT-5.5) and the structured review checklist.
when_to_use: Opening, reviewing, or merging any PR.
---

# PR Review

## Reviewers

Every PR requests **two** reviewers:

1. **`@CompewterTutor`** — Claude (architecture, Rust safety, design coherence).
2. **GPT-5.5 reviewer agent** — second-pair-of-eyes on logic, tests, and prose.

If a reviewer is unavailable for >24h, leave a comment naming the blocker. Do
not merge with one missing review unless the PR is `docs/*` or `chore/ci-*`
*and* the author leaves a note acknowledging the deviation.

## Author checklist (before requesting review)

- [ ] Branch follows naming convention.
- [ ] All commits are Conventional Commits.
- [ ] `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace` pass locally.
- [ ] WASM smoke test run if touching wasm bits.
- [ ] `CHANGELOG.md` updated under `## Unreleased`.
- [ ] `docs/LEARNINGS.md` appended if anything surprising was discovered.
- [ ] PR description includes: **Summary**, **Why**, **Test plan**, **Risks**.

## Reviewer checklist

### Correctness
- [ ] The code does what the PR title and description claim.
- [ ] Edge cases (empty input, large input, wasm-only paths, missing native lib).
- [ ] Error handling: no silent `unwrap`, no swallowed errors.

### Tests
- [ ] At least one test covers each new branch.
- [ ] Tests fail without the new code (mentally or by spot-check).
- [ ] WASM tests run in CI without requiring WasmEdge if feature is off.

### Architecture
- [ ] Stays within the boundary documented in `AGENTS.md` (host vs wasm crates).
- [ ] No host-only deps creeping into wasm-buildable crates.
- [ ] Public API additions are documented.

### Hygiene
- [ ] No commented-out code, no `dbg!`, no TODO without an issue link.
- [ ] CHANGELOG entry is user-facing prose, not commit log dump.
- [ ] Docs/README updated if user-facing behaviour changed.

## Review verdicts

- **Approve** — ship it.
- **Comment** — questions, no blockers.
- **Request changes** — must address before merge; reviewer re-reviews.

## Merging

- **Squash-merge** by default; PR title becomes the commit subject (must be
  Conventional Commits).
- **Merge commit** for release branches only (after a `chore(release): vX.Y.Z` PR).
- Delete the source branch on merge.
