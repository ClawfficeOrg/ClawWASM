# ClawWASM

Rust + WasmEdge plugin for Godot 4 that embeds a minimal OpenClaw-like
gateway inside a WASM sandbox. Lets Clawffice-Space nodes run on
WASM-capable targets (Godot host, browsers, SteamDeck, consoles) without
Docker or Podman.

> **Status:** v0.1.0 dev release. Engine integration is in progress on
> `main`; see [`ralph/PLAN.md`](ralph/PLAN.md) for the active workplan and
> [`docs/TODO.md`](docs/TODO.md) for the semver roadmap.

## Repository layout

| Path                  | What lives here                                              |
| --------------------- | ------------------------------------------------------------ |
| `clawasm/`            | Native Godot 4 plugin (cdylib), godot-rust ≥ 0.5.            |
| `clawasm/engine/`     | Embedded WasmEdge engine wrapper (feature-gated).            |
| `examples/hello-wasm/`| Minimal `wasm32-wasip1` smoke binary (CI runs this).         |
| `docs/`               | Plan, TODO, guidelines, tests, architecture, memory.         |
| `scripts/`            | Build & smoke-test helpers.                                  |
| `ralph/`              | Autonomous development loop runner.                          |
| `.superpowers/skills/`| Skill files loaded by AI coding agents.                      |
| `.github/`            | CI workflows, PR/issue templates, Copilot instructions.      |

## Quickstart

### 1. Toolchain

```bash
rustup default stable
rustup target add wasm32-wasip1
```

### 2. Build & smoke-test the wasm example

```bash
cargo build --manifest-path examples/hello-wasm/Cargo.toml \
            --target wasm32-wasip1 --release
wasmedge examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm
```

Or the all-in-one helper:

```bash
bash scripts/test-wasm.sh
```

### 3. Build the Godot 4 plugin (native)

```bash
cargo build -p clawasm --release
# artifact: target/release/libclawasm.{so,dylib,dll}
```

Copy the artifact and a `.gdextension` manifest into your Godot project's
`addons/clawasm/` folder. (A reference manifest will land with the v0.2.0
plugin work — see `ralph/PLAN.md`.)

## Quality gates (run before pushing)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace --all-targets
bash scripts/test-wasm.sh
```

CI mirrors these in `.github/workflows/ci.yml`.

## Working on ClawWASM

This repo is set up to be developed by AI coding agents (Claude Code,
Codex, Copilot, …) and humans interchangeably. The contract is in:

- [`AGENTS.md`](AGENTS.md) — universal agent rules (read this first).
- [`.superpowers/skills/`](.superpowers/skills) — pluggable skill files.
- [`ralph/`](ralph) — the autonomous Ralph development loop.
- [`docs/MEMORY.md`](docs/MEMORY.md) and [`docs/LEARNINGS.md`](docs/LEARNINGS.md) — persistent project memory.
- [`docs/PLAN.md`](docs/PLAN.md) — long-term embedding plan.
- [`docs/TODO.md`](docs/TODO.md) — semver-mapped roadmap.
- [`docs/guidelines.md`](docs/guidelines.md) — coding standards.
- [`RELEASING.md`](RELEASING.md) — how releases are cut.
- [`CHANGELOG.md`](CHANGELOG.md) — what shipped when.

PRs follow Conventional Commits and request reviews from
`@CompewterTutor` (Claude) and the GPT-5.5 reviewer agent. See
`.superpowers/skills/pr-review.md`.

## License

TODO — choose and add `LICENSE` before v1.0.0.
