# Changelog

All notable changes to ClawWASM are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project follows [Semantic Versioning](https://semver.org/).

## Unreleased

### Added
- **`ClawEngine` Godot 4 node** (`clawasm/src/engine_node.rs`). Drop into
  any scene; exposes `register_module(path)`, `set_wasmedge_binary(path)`,
  `start(args)`, `stop()`, `is_running()`, and `module_path()` to GDScript.
  Emits `stdout_line(line)`, `stderr_line(line)`, `finished(code)`, and
  `failed(message)` signals. Accepts `res://` and `user://` paths
  (resolved via `ProjectSettings.globalize_path`).
- **`clawasm_engine::stream` module** — `Runner` / `Event` types that spawn
  a subprocess with piped stdout/stderr and ferry output line-by-line through
  an mpsc channel. `Instance::stream(args)` is the streaming counterpart of
  `Instance::run`. `Finished`/`Failed` events are guaranteed after every line.
  Five unit tests cover streaming, ordering, kill-on-`stop()`, and
  missing-binary handling.
- **Godot smoke-test scaffold** (`tests/godot-smoke/`): `project.godot`,
  `main.tscn`, `main.gd` reference script, and a full headless runbook
  in `README.md`. Verified GREEN on macOS arm64 (Godot 4.6.2, WasmEdge
  0.14.1, godot-rust 0.5.2). Linux run pending.

### Changed
- **`tests/godot-smoke/README.md`** — headless steps now documented;
  macOS+Homebrew `rustc` caveat and `.godot/extension_list.cfg` requirement
  recorded. See `docs/LEARNINGS.md` 2026-05-06.

## [v0.2.0] - 2026-04-30

First real engine release: `clawasm-engine` can load a `.wasm` file from
disk and run it under WasmEdge, capturing stdout / stderr / exit code.
The `clawasm` host plugin builds in stub mode on a clean machine (no
native WasmEdge install required for `cargo check`/`clippy`). Adds the
autonomous-agent substrate (AGENTS.md, Superpowers skills, Ralph loop)
and the full CI/CD pipeline.

### Added
- **`ClawEngine` Godot 4 node** (`clawasm/src/engine_node.rs`). Drop into
  any scene; exposes `register_module(path)`, `set_wasmedge_binary(path)`,
  `start(args)`, `stop()`, `is_running()`, and `module_path()` to
  GDScript. Emits `stdout_line(line)`, `stderr_line(line)`, `finished(code)`,
  and `failed(message)` signals. Accepts `res://` and `user://` paths in
  addition to filesystem paths (resolved via `ProjectSettings.globalize_path`).
- **`clawasm_engine::stream` module** with `Runner` / `Event` types that
  spawn a subprocess with piped stdout/stderr and ferry output line-by-line
  through an mpsc channel. `Instance::stream(args)` is the new streaming
  counterpart of `Instance::run` and is what `ClawEngine` uses on each
  `_process` tick. `Finished`/`Failed` events are guaranteed to arrive
  after every line of output. Five new unit tests cover line streaming,
  ordering, kill-on-`stop()`, and missing-binary handling.

- **`clawasm-engine` v0.2.0 MVP.** Real `Instance::run` implementation
  (subprocess to the `wasmedge` CLI) with stdout / stderr / exit-code
  capture. New public surface: `Engine::with_binary`, `Engine::probe`,
  `Engine::binary`, `Instance::module_path`, `Output::success`,
  `Output::stderr`. `Engine::load` now also rejects empty files and
  takes `impl AsRef<Path>` instead of `&str`.
- **`WASMEDGE_BIN` environment variable** — lets callers point at a
  non-PATH WasmEdge install (e.g. `$HOME/.wasmedge/bin/wasmedge`,
  which is where the official installer lands).
- **`clawasm-engine` CLI** — `cargo run -p clawasm-engine -- <module.wasm> [args...]`
  is now a thin wrapper that exercises the same code path Godot uses.
- **6 unit tests** for env override, path validation, probe error
  reporting, and the `Output::success` helper. No external deps.
- **Feature-gated integration test** (`clawasm/engine/tests/smoke.rs`,
  behind `with-wasmedge`) loads `examples/hello-wasm` under WasmEdge
  and asserts `success() && stdout.contains("hello")`.
- **Engine README** rewritten with usage, configuration, testing, and
  install instructions.
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
- **`clawasm` host crate** no longer depends on `wasmedge-sys` directly.
  All WasmEdge usage is routed through the `clawasm-engine` path dependency,
  which feature-gates `wasmedge-sys` behind `with-wasmedge`. The plugin now
  builds and lints in stub mode (`cargo check -p clawasm`,
  `cargo clippy --workspace`) on a clean machine without libwasmedge
  installed. The `clawasm/with-wasmedge` feature forwards to
  `clawasm-engine/with-wasmedge` for the native path.
- **CI** — restored `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo check -p clawasm` in the host jobs (previously scoped narrower
  while the dep mismatch was being resolved).

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

[v0.2.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.2.0
[v0.1.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.1.0
