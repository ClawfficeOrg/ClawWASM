---
name: Commit Discipline
description: Conventional Commits + small, atomic commits + signed-off-by for ClawWASM.
when_to_use: Every commit.
---

# Commit Discipline

## Conventional Commits

```
<type>(<scope>): <subject>

<body — optional, wrap at 72>

<footer — optional: BREAKING CHANGE:, Refs:, Co-authored-by:>
```

| Type      | Use                                                    |
| --------- | ------------------------------------------------------ |
| feat      | New user-facing functionality.                         |
| fix       | Bug fix.                                               |
| docs      | Docs only (README, *.md).                              |
| chore     | Build, deps, repo housekeeping.                        |
| ci        | CI workflows / scripts only.                           |
| build     | Build system / packaging.                              |
| refactor  | Internal change, no behaviour change.                  |
| test      | Tests only (adding/fixing tests with no new code).     |
| perf      | Performance improvement.                               |
| style     | Formatting, whitespace, semicolons.                    |
| revert    | Reverts a previous commit (include the SHA in body).   |

**Scope** is the crate or top-level area: `engine`, `godot`, `ci`, `examples`,
`docs`, `ralph`, `release`.

**Subject** is imperative, lowercase first letter, no trailing period, ≤ 72 chars.

### Good

```
feat(engine): add WASI preopen mapping for /data
fix(ci): pin WasmEdge to 0.14.1 to unblock ubuntu-latest
docs(readme): clarify wasm32-wasip1 vs wasm32-wasi
chore(release): v0.2.0
```

### Bad

```
update stuff                  # no type, no scope, no info
feat: things                  # no real subject
fix(ci): Fixed the CI.        # past tense, capitalized, period
WIP                           # squash before pushing
```

## Atomic commits

One logical change per commit. Use `git add -p` to split mixed changes.

- Refactors that enable a feature go in their own commit *before* the feature.
- Renames go in their own commit (use `git mv` so blame survives).
- Generated files / lockfile bumps go in their own commit.

## Breaking changes

- Append `!` after the type/scope: `feat(engine)!: redesign Output struct`.
- Add a `BREAKING CHANGE:` footer describing the migration.
- Bump the major version in the next release.

## Sign-off

Use `git commit -s` to add a `Signed-off-by:` line. Required on all commits.

## Squash policy

PRs are squash-merged by default — the PR title becomes the squashed commit
subject. Make the PR title a valid Conventional Commit subject *before*
merging.
