# Releasing ClawWASM

This is the operator's guide. The full protocol lives in
`.superpowers/skills/release-engineering.md`.

## TL;DR

```bash
# 1. main is green and you're up to date
git checkout main && git pull --ff-only

# 2. open a release PR
git checkout -b chore/release-vX.Y.Z
# bump versions in clawasm/Cargo.toml and clawasm/engine/Cargo.toml
# move CHANGELOG ## Unreleased into ## [vX.Y.Z] - YYYY-MM-DD
# add a new empty ## Unreleased
git commit -sam "chore(release): vX.Y.Z"
git push -u origin chore/release-vX.Y.Z
gh pr create --fill --reviewer CompewterTutor

# 3. after merge, tag
git checkout main && git pull --ff-only
git tag -as vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

The `release` workflow runs on the tag push and produces a GitHub Release
with native `cdylib` artifacts and an optimized `hello-wasm.wasm`.

## Picking the version

Walk commits since the last tag (`git log $(git describe --tags --abbrev=0)..HEAD --oneline`)
and pick:

- **MAJOR** — any commit with `!` after the type, or `BREAKING CHANGE:` in
  the footer.
- **MINOR** — at least one `feat:` since the last tag.
- **PATCH** — only `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`,
  `build:`, `perf:`, `style:`.

Pre-1.0 we still bump MINOR for breaking changes (per SemVer §4) but call
it out loudly in the CHANGELOG.

## Hotfix

```bash
git checkout -b fix/<short> vX.Y.Z   # branch off the tag
# fix
gh pr create --base main --fill
# after merge:
git tag -as vX.Y.(Z+1) -m "Release vX.Y.(Z+1)"
git push origin vX.Y.(Z+1)
```

## Yanking

```bash
gh release edit vX.Y.Z --draft          # hide it
# add a "## Yanked" note to the CHANGELOG entry pointing to the replacement
# cut the next patch immediately
```

We do not currently publish to crates.io. When that changes, document the
`cargo publish` order here.
