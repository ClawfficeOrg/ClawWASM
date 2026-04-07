# Development Guidelines for ClawWASM

## Purpose
This document defines coding standards, testing practices, CI checklist, release and PR policies for ClawWASM — a Rust + WasmEdge project targeting wasm32-wasi and native development. It exists to keep contributions consistent, reduce CI noise, and ensure reliable Wasm builds and runtime behavior.

## Rust coding standards
- Target stable Rust (MSRV: latest stable at time of release; update in CI). Prefer edition=2021 or 2024 as project-wide Cargo.toml specifies.
- Formatting: run `rustfmt` on all code. CI will enforce `cargo fmt -- --check`.
- Linting: run `cargo clippy --all-targets --all-features -- -D warnings`. Fix warnings before PR.
- Safety: prefer safe Rust. Any `unsafe` must include a comment explaining invariants and a link to tests that validate them.
- Error handling: use `thiserror`/`anyhow` consistently; return Result where appropriate. Avoid unwrap() in library code — use expect only when necessary with clear messages.
- Documentation: public APIs must have rustdoc comments (///). Run `cargo doc --no-deps` locally to preview.
- Module organization: keep lib.rs small; group modules in src/ and expose a clear public API surface.
- Dependencies: prefer small, maintained crates. Add new deps only with a short justification in PR description.

## WasmEdge and wasm32-wasi best practices
- Build target: wasm32-wasi for WasmEdge compatibility. Example build command: `cargo build --target wasm32-wasi --release`.
- Note on WASI variants: there are multiple WASI target triples (wasm32-wasi, wasm32-wasip1, wasm32-unknown-unknown). For WasmEdge prefer `wasm32-wasi`. `wasm32-wasip1` targets the newer preview1 ABI and may require different host support; only use it if your runtime and toolchain explicitly support wasip1. Avoid `wasm32-unknown-unknown` for WASI-based programs — it's for non-WASI raw Wasm.
- Optimize for size: enable LTO and strip symbols for release Wasm builds. Example: set in Cargo.toml profiles or build with:
  - `RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-wasi --release`
  - Consider using `-C opt-level=z` for size-sensitive modules
- WasmEdge runtime: test Wasm with WasmEdge CLI: `wasmedge target/wasm32-wasi/release/<crate>.wasm [args]`.
- Host bindings: prefer wasi APIs when possible. For custom host functions, document ABI and versioning.
- Wasm features: avoid relying on unsupported syscalls. Use wasi APIs and preview1 where available.

## Testing plan (local and CI)
Local commands to run before pushing:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all --lib --tests`
- Build native release: `cargo build --release`
- Build wasm release: `RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-wasi --release`
- Run wasm with WasmEdge (local dev): `wasmedge target/wasm32-wasi/release/<crate>.wasm` or `wasmedge run target/wasm32-wasi/release/<crate>.wasm`

CI commands (example .github/workflows/ci.yml):
- Setup Rust (latest stable)
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all --release`
- Build wasm: `rustup target add wasm32-wasi && RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-wasi --release`
- Optionally run WasmEdge CLI inside CI (setup wasmedge/wasmedge-rust) and run `wasmedge run ...` to exercise runtime.

Integration / E2E tests
- Provide an `integration/` directory or tests that exercise the compiled wasm under WasmEdge.
- Example CI step: `wasmedge target/wasm32-wasi/release/<crate>.wasm -- <integration-args>` and assert output/exit codes.

## CI checklist
- [ ] All tests pass (unit + integration)
- [ ] `cargo fmt` check passes
- [ ] `cargo clippy` passes with no warnings
- [ ] Wasm build succeeds for wasm32-wasi
- [ ] Wasm runtime smoke test with WasmEdge (if available in CI)
- [ ] CHANGELOG updated for user-facing changes
- [ ] README updated if usage or API changed

## Release & changelog guidelines
- Follow semantic versioning (semver.org).
- Patch: bug fixes, documentation, tests → x.y.z+1
- Minor: backwards-compatible features → x.y+1.0
- Major: breaking API changes → x+1.0.0
- Release process:
  1. Prepare CHANGELOG entry in `CHANGELOG.md` under Unreleased
  2. Bump version in Cargo.toml
  3. Tag release `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
  4. Push tag and merge to main
- Keep a human-written changelog entry for each release.

## PR review policy (map to phase completions)
- Phase patch (.0.2.1, .0.2.2): small fixes — 1 reviewer, automated CI green
- Phase minor (.0.3.0): new features — 2 reviewers, tests + changelog entry, update docs
- Major (1.0.0, 2.0.0): breaking changes — discussion issue + design doc, 2+ reviewers including a maintainer, migration guide
- Every PR must include: description, testing steps, linked issue (if any), changelog note (if user-visible).

## Branching and versioning policy
- Main branch `main` always reflects production-ready code.
- Feature branches: `feature/<short-desc>`, bugfix: `fix/<short-desc>`, docs: `docs/<short-desc>`.
- Rebase small branches onto latest main before merging to reduce merge commits.
- Protect `main` with branch protection rules (require PR, CI passes, review).

## Commit conventions
- Use conventional commits style: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
- Keep commit messages short and descriptive

## Documentation & learnings requirement
- Any PR that changes code must update README and CHANGELOG if user-visible behavior changes.
- Authors must add a short "Learnings" section to docs/guidelines.md describing decisions or surprises for future contributors. This should be appended on the next commit that touches related code.

## Recommended tooling
- rustup, cargo, rustfmt, clippy
- wasm32-wasi target via rustup
- WasmEdge CLI for runtime testing
- Optionally wasmprinter/wasm-tools for inspection

## Godot 4 Migration (godot-rust v0.5 / April 2026)

The `clawasm` crate has been migrated from `gdnative` (Godot 3) to the `godot` crate (godot-rust v0.5, Godot 4).

### What changed
- `clawasm/Cargo.toml`: dependency changed from `gdnative` to `godot = "0.5"` (the v4-only crate).
- `clawasm/src/lib.rs`: rewrote bindings using godot-rust 0.5 API:
  - `#[derive(NativeClass)]` → `#[derive(GodotClass)]`
  - `#[inherit(Node)]` → `#[class(base=Node)]`
  - `#[gdnative::methods]` → `#[godot_api] impl INode for ClawWasm`
  - `fn new(_owner)` → `fn init(base: Base<Node>) -> Self`
  - `fn _ready(&self, owner)` → `fn ready(&mut self)`
  - `godot_init!()` → removed; auto-registered via `#[derive(GodotClass)]`
  - Added `ExtensionLibrary` entry point required by Godot 4 GDExtension API

### Build commands (Godot 4)

```bash
# Native cdylib (GDExtension for Godot 4)
cargo build --manifest-path clawasm/Cargo.toml --release
```

> **Note:** `wasmedge-sys` requires the WasmEdge C library to be installed on the host.
> See https://wasmedge.org/book/en/embed/rust.html for installation.
> Without it, the build will fail at the `wasmedge-sys` crate — the godot bindings
> themselves compile cleanly.

### Remaining TODOs
- Wire WASMEdge runtime calls inside the `ready()` method (see FIXMEs in lib.rs).
- Add a `.gdextension` file to the Godot project pointing to the compiled `.so`/`.dll`.
- Remove the old GDNativeLibrary / `.gdns` resources from any Godot 3 project files.
- Install WasmEdge in CI to unblock the full build (see CI appendix below).

---

```yaml
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          override: true
      - name: Install wasm target
        run: rustup target add wasm32-wasi
      - name: Format check
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: Test
        run: cargo test --all --release
      - name: Build wasm
        run: RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-wasi --release
      - name: Wasm smoke test (optional)
        run: |
          sudo apt-get update && sudo apt-get install -y curl
          curl -sSfL https://github.com/WasmEdge/WasmEdge/releases/download/1.0.3/wasmedge-ubuntu.tar.gz | tar xz
          ./wasmedge run target/wasm32-wasi/release/<crate>.wasm || true
```

---

Please follow these guidelines and add practical notes/learnings back into this file as the project evolves.
