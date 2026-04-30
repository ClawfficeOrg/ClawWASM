---
name: Release Engineering
description: Cutting a versioned release of ClawWASM (semver, tags, CHANGELOG, artifacts).
when_to_use: When the team agrees a release is ready, or the Ralph loop sees the release checklist satisfied.
---

# Release Engineering

## Versioning

Strict [SemVer 2.0](https://semver.org):

- **MAJOR** — breaking change to a public Rust API or to user-visible behaviour
  of the Godot plugin. Any commit with `!` or `BREAKING CHANGE:` triggers this.
- **MINOR** — new functionality, backwards-compatible.
- **PATCH** — bug fixes, docs, internal refactors.

Pre-1.0 we still bump MINOR for breaking API changes (per SemVer §4) but call
it out loudly in `CHANGELOG.md`.

## Pre-flight (manual or via Ralph)

1. Verify `main` is green on CI.
2. `git pull --ff-only origin main`.
3. Decide the version: read commits since the last tag and pick the bump.
4. Update versions in:
   - `clawasm/Cargo.toml`
   - `clawasm/engine/Cargo.toml`
   - any other `Cargo.toml` whose `version` is hand-managed.
5. Move everything in `CHANGELOG.md` `## Unreleased` into a new
   `## [vX.Y.Z] — YYYY-MM-DD` section. Add a fresh empty `## Unreleased`.
6. Run full quality gates:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test  --workspace --all-targets
   bash scripts/test-wasm.sh
   ```
7. Open a `chore(release): vX.Y.Z` PR. Title is the squashed commit subject.

## Tag & publish

After the release PR is merged:

```bash
git checkout main && git pull --ff-only
git tag -as vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

The `release.yml` workflow fires on tag push and:

1. Re-runs CI.
2. Builds release artifacts (native `cdylib` for linux/macos/windows; optimized
   `hello-wasm.wasm`).
3. Creates a GitHub Release with the CHANGELOG section as the body.
4. Attaches artifacts + checksums.

## Hotfix

For a `vX.Y.Z+1` patch:

1. Branch `fix/<short>` off the tag.
2. Cherry-pick or fix forward.
3. Open PR against `main`. After merge, tag as above.

## Yanking

If a release is broken in a way users hit:

1. `gh release edit vX.Y.Z --draft` to hide it.
2. Cut the next patch immediately.
3. Add a `## Yanked` note to the CHANGELOG entry referencing the replacement.

We do **not** publish to crates.io yet (workspace is private-ish and
plugin-shaped). When that changes, document the `cargo publish` order here.
