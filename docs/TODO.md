# TODO — ClaWASM (ClawWASM) Roadmap

This TODO organizes work into semver-mapped phases with tasks, estimates, dependencies, acceptance criteria, and recommended agent assignments.

## v0.1.0 — Foundation: build, run, and host integration (MVP)

Tasks:
- Tooling & CI
  - Add cargo target and build script for wasm32-unknown-unknown. (Est: 2d)
  - Add Github Actions: build for debug/release, wasm-opt step, run basic checks. (Est: 2d)
  - Recommended agent: gpt-5-mini (docs & CI), sonnet (review)
  - Dependencies: none
  - Acceptance: CI builds release wasm artifact and passes linting.

- Minimal runtime
  - Implement a "hello world" Wasm target that prints to stdout via WASI. (Est: 1d)
  - Recommended agent: minimax (implementation)
  - Acceptance: wasmedge runs the wasm and prints message.

- Godot plugin integration
  - Ensure Godot node (clawasm) compiles as cdylib and exposes plugin ready/logging. (Est: 1d)
  - Recommended agent: minimax
  - Acceptance: Godot prints readiness message when plugin is loaded.

- Documentation
  - Document build/run steps in README and docs/BUILD.md. (Est: 1d)
  - Recommended agent: gpt-5-mini
  - Acceptance: Clear steps reproduce local build and run.

## v0.2.0 — Networking & Hub connectivity

Tasks:
- WebSocket client
  - Implement outbound WebSocket client to connect to hub (wss://). Start with a test echo server. (Est: 3d)
  - Recommended agent: nemotron/minimax
  - Dependencies: v0.1.0 artifacts
  - Acceptance: wasm binary connects to test ws and exchanges JSON messages.

- Protocol & registration
  - Define simple HELLO/SESSION/MSG JSON protocol and implement registration. (Est: 2d)
  - Recommended agent: sonnet (design), minimax (impl)
  - Acceptance: hub shows node registered; node lists capabilities.

- Tool subset & stubs
  - Implement a Wasm-safe tool subset (memory_get, memory_search proxy, web_fetch proxy). Stub remaining tools to return NotSupported. (Est: 2d)
  - Recommended agent: gpt-5-mini (docs), minimax (impl)
  - Acceptance: tool requests handled or return graceful error.

## v0.3.0 — Persistence & Session Management

Tasks:
- WASI persistence
  - Add filesystem-backed persistence (JSON or SQLite) via WASI pre-opened dir. (Est: 3d)
  - Recommended agent: minimax
  - Dependencies: v0.1.0, v0.2.0
  - Acceptance: state survives restart; can read/write memory files.

- Session & agent lifecycle
  - Implement in-memory session manager with save/load to disk. (Est: 3d)
  - Acceptance: sessions persist and restore correctly.

- Tests & benchmarks
  - Add integration tests for persistence and message flows. (Est: 2d)
  - Acceptance: tests pass in CI.

## v0.4.0 — Host integrations & advanced features

Tasks:
- Godot ↔ Wasm control API
  - Expose start/stop/status APIs via Godot signals or WASI FDs. (Est: 3d)
  - Acceptance: Godot can start/stop the gateway and receive logs.

- Security & TLS
  - Document options for TLS termination (host vs in-wasm rustls). (Est: 2d)
  - Acceptance: clear recommended deployment patterns.

- Tooling improvements
  - Optional: enable proxying heavy tools via hub (offload web_fetch/web_search). (Est: 4d)
  - Acceptance: proxy works end-to-end with hub.

## v1.0.0 — Production readiness

Tasks:
- Performance tuning & size optimization (wasm-opt flags, strip symbols). (Est: 3d)
- Full documentation (deployment, Godot plugin usage, hub setup). (Est: 3d)
- End-to-end tests (multi-node message routing). (Est: 5d)
- Recommended agent assignments: sonnet for architecture review, minimax/nemotron for implementation, gpt-5-mini for docs and tests.
- Acceptance: stable release artifacts, CI passing, documented upgrade path.

## Cross-phase notes

- Branching & PRs: use feature branches per milestone (feature/wasm-build, feature/ws-client, feature/persistence, feature/godot-api). Open PRs for review; Sonnet should review architecture-affecting PRs.
- Naming: do NOT rename repo without explicit confirmation from Michael. Suggestion: include a PR proposing rename to "ClaWASM" with steps (create new repo or rename via GitHub repo settings, update README, redirect links). If approved, list exact rename steps in PR description.
- Agent assignment guidance:
  - sonnet: architecture, critical design, reviews
  - nemotron/minimax: implementation and orchestration
  - gpt-5-mini: docs, CI, tests, small refactors

## Acceptance criteria template (use per-task)
- Task description
- Estimated time
- Dependencies
- How to test
- Who reviews

## Next actions (immediate)
1. Create feature/docs-todo branch.
2. Add docs/TODO.md (this file) and update README with a short roadmap summary.
3. Open PR: "docs: add granular TODO.md with semver roadmap"

