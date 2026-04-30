# Changelog

All notable changes to ClawWASM are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project follows [Semantic Versioning](https://semver.org/).

## Unreleased

### Added
- **Autonomous-development substrate.** Added `AGENTS.md` (binding contract
  for all AI coding agents), six "always-on" superpowers skills under
  `.superpowers/skills/` (TDD, writing-plans, PR-review, memory-keeper,
  commit-discipline, ralph-loop), four on-demand skills (wasm-build,
  godot-binding, engine-integration, release-engineering), and a Ralph loop
  runner under `ralph/` with Claude Code and Codex CLI adapters.
- **Project memory surfaces.** `docs/MEMORY.md` (long-lived decisions and
  invariants) and `docs/LEARNINGS.md` (append-only lab notebook).
- **Operator docs.** `RELEASING.md` with the semver/tag/release workflow.
- **GitHub plumbing.** `.github/copilot-instructions.md` pointing Copilot at
  `AGENTS.md`; PR template; bug-report and feature-request issue templates;
  issue config redirecting design questions to Discussions.
- **CI/CD.** New `ci.yml` workflow with fmt, clippy, multi-OS test, wasm
  smoke (wasm32-wasip1 + WasmEdge 0.14.1), feature-gated
  `clawasm-engine --features with-wasmedge` job, and a Conventional Commits
  PR-title check. New `release.yml` workflow that fires on `v*.*.*` tags
  and ships native cdylibs and an optimized `hello-wasm.wasm` to a GitHub
  Release with CHANGELOG-derived notes.

### Changed
- (none — scaffolding only.)

### Repo housekeeping
- Pruned all merged feature branches; `main` is the single source of truth.
- Merged outstanding PRs #6 (architecture review) and #8 (engine scaffold).

## [v0.1.0] - 2026-04-06

Initial developer release. Snapshot before the Godot 4 migration.

- Repo scaffold with `clawasm` cdylib crate (originally `gdnative` /
  Godot 3) (#1, #66dc776).
- Granular semver-mapped TODO roadmap in `docs/TODO.md` (#1).
- Development guidelines and testing plan in `docs/guidelines.md` (#2).
- v0.1.0 changelog scaffolding and project-memory entries (#3).
- Wasm CI workflow `.github/workflows/wasm-ci.yml` and `examples/hello-wasm`
  (#4).
- CI hardening: WasmEdge tarball install pinned, `wasm32-wasip1` target,
  isolated `examples/hello-wasm` build via `--manifest-path` (#5).

[v0.1.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.1.0
