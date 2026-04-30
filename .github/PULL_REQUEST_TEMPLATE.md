## Summary

<!-- One paragraph: what this PR changes and why. -->

## Why

<!-- Link to ralph/PLAN.md task, docs/TODO.md milestone, or issue. -->

## Test plan

<!-- Concrete steps a reviewer can run. List passing commands. -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace --all-targets`
- [ ] (if wasm) `bash scripts/test-wasm.sh`
- [ ] New/updated tests cover the change.

## Risks

<!-- What could break? Migration notes? Rollback plan? -->

## Checklist

- [ ] Branch follows `feature/`, `fix/`, `docs/`, `chore/`, `refactor/` naming.
- [ ] Conventional Commits in every commit and the PR title.
- [ ] `CHANGELOG.md` updated under `## Unreleased` (user-facing prose).
- [ ] `docs/LEARNINGS.md` appended if anything surprising was discovered.
- [ ] Reviewers requested: `@CompewterTutor` and the GPT-5.5 reviewer agent.
