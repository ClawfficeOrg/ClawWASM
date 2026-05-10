# Changelog

All notable changes to ClawWASM are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project follows [Semantic Versioning](https://semver.org/).

## Unreleased

### Added
- **`CLLawM` Godot 4 node** (`clawasm/src/llm_node.rs`) — native in-process
  LLM inference via the `llama-cpp-2` crate (llama.cpp baked into the cdylib).
  Metal GPU acceleration auto-enabled on macOS via llama-cpp-sys-2's cmake;
  no extra feature flag needed. Enabled with `--features with-llama`.
  GDScript API: `set_model(path)`, `set_system_prompt(text)`,
  `set_temperature(v)`, `set_top_p(v)`, `set_top_k(k)`,
  `set_n_predict(n)`, `set_n_threads(n)`, `set_ctx_size(n)`,
  `generate(prompt) -> bool`, `stop()`, `is_running() -> bool`.
  Signals: `token_generated(token)`, `inference_done(full_text, exit_code)`,
  `inference_failed(message)`. Chat template read from the GGUF's embedded
  metadata via `model.apply_chat_template()`. Inference runs on a background
  thread (`Arc<LlamaModel>` cached across calls, `LlamaContext`/`LlamaBatch`/
  `LlamaSampler` created per-call, all `!Send` and thread-local). Without
  `with-llama`, the node compiles as a safe no-op stub.
- **`with-llama` Cargo feature** in `clawasm/Cargo.toml` — pulls in
  `llama-cpp-2 = "0.1"`, `anyhow = "1.0"`, `encoding_rs = "0.8"` as
  optional deps.
- **`with-llama-build` CI job** (`.github/workflows/ci.yml`) — builds
  `clawasm --features with-llama` on `macos-latest` with cargo/target
  cache; `continue-on-error: true` until validated.
- **`examples/llm-chat/`** — self-contained Godot 4.6+ project with a
  streaming chat UI and a full settings panel (temperature, top-p, top-k,
  max tokens, CPU threads, context window). Open in Godot, point at a
  `.gguf` model, click Apply, and chat. See `examples/llm-chat/README.md`.
- **v0.7.0 WASM bridge plan** documented in `docs/TODO.md` — JSON-over-stdout
  protocol for routing `CLLawM` generation requests from WASM modules via
  a `ClawBridge` GDScript autoload.

### Fixed
- **`scripts/download-model.sh`** — two bugs squashed:
  1. Switched from the deprecated `huggingface-cli` to the current `hf` CLI
     (same `huggingface_hub` package; falls back to `huggingface-cli` if
     `hf` is not on `PATH`).
  2. Corrected the GGUF filename prefix from `gemma-4-E2B-it-` to
     `google_gemma-4-E2B-it-` (matching bartowski's actual repo filenames).
  3. Defensive `.gguf` stripping from the `QUANT` argument so both
     `Q4_K_M` and `Q4_K_M.gguf` work.

### Removed
- **`LlmConfig`** and `DEFAULT_LLAMA_CLI_BIN` from `clawasm-engine`
  (`clawasm/engine/src/lib.rs`). The subprocess approach is superseded by
  the native `llama-cpp-2` integration. `Runner::spawn_chunked` and
  `Event::StdoutChunk` are retained for the future WASM bridge.

## [v0.5.0] - 2026-05-08

Pre-built addon bundle. Every GitHub Release now ships
`clawasm-addon-vX.Y.Z.zip` — unzip, drop `addons/clawasm/` into your
Godot 4.6+ project, and `ClawEngine` is ready with no Rust toolchain.

### Added
- `release.yml` — "Build addon bundle zip" step assembles
  `addons/clawasm/{clawasm.gdextension, libclawasm.so, libclawasm.dylib,
  clawasm.dll, README.md}` and attaches `clawasm-addon-v0.5.0.zip` +
  sha256 to the GitHub Release.

## [v0.4.0] - 2026-05-06

Headless Godot CI smoke. Every PR now runs `ClawEngine` end-to-end
against Godot 4.6.2 on Linux x86_64 in CI — no manual Godot session
needed. Both macOS arm64 and Linux x86_64 confirmed green.

### Added
- **`ci.yml` `godot-smoke` job** — installs WasmEdge 0.14.1 + Godot
  4.6.2, builds the cdylib + `hello-wasm.wasm`, lays out the smoke
  project with `.godot/extension_list.cfg`, and asserts
  `[wasm] hello-wasm` + `[wasm] exit 0` in headless output.
- `tests/godot-smoke/main_headless.gd` / `main_headless.tscn` /
  `project_headless.godot` — headless-CI variants with
  `get_tree().quit()` for clean exit.
- Linux x86_64 smoke result recorded in `tests/godot-smoke/README.md`.

## [v0.3.0] - 2026-05-06

`ClawEngine` Godot 4 node: drop it into any scene and stream stdout/stderr
from a WasmEdge subprocess directly into GDScript signals. Zero native
WasmEdge build dependency — the cdylib still builds with `cargo check` on
a clean machine. Smoke-tested headlessly on macOS arm64.

### Added
- **`ClawEngine` Godot 4 node** (`clawasm/src/engine_node.rs`). Drop into
  any scene; exposes `register_module(path)`, `set_wasmedge_binary(path)`,
  `start(args)`, `stop()`, `is_running()`, and `module_path()` to GDScript.
  Emits `stdout_line(line)`, `stderr_line(line)`, `finished(code)`, and
  `failed(message)` signals. Accepts `res://` and `user://` paths
  (resolved via `ProjectSettings.globalize_path`).
- **`clawasm_engine::stream` module** — `Runner` / `Event` types spawn a
  subprocess with piped stdout/stderr and ferry output line-by-line through
  an mpsc channel. `Instance::stream(args)` is the new streaming API;
  `Finished`/`Failed` events are guaranteed to arrive after all output lines.
  Five unit tests cover streaming, ordering, kill-on-`stop()`, and
  missing-binary handling.
- **Godot smoke-test scaffold** (`tests/godot-smoke/project.godot`,
  `main.tscn`, `main.gd`) with a full headless runbook. Verified GREEN on
  macOS arm64 (Godot 4.6.2, WasmEdge 0.14.1, godot-rust 0.5.2).

### Fixed
- `Runner::stop` test: replaced `sh -c "sleep 30"` with `Command::new("sleep")`
  to avoid Linux bash forking a child that outlives the killed shell PID,
  which previously caused a 30-second deadlock on CI.

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

[v0.5.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.5.0
[v0.4.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.4.0
[v0.3.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.3.0
[v0.2.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.2.0
[v0.1.0]: https://github.com/ClawfficeOrg/ClawWASM/releases/tag/v0.1.0
