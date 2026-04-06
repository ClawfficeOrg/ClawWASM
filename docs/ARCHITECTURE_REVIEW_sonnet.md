Architecture review — ClaWASM (sonnet voice)

Summary

ClawWASM is a small Rust workspace with two visible goals: a native Godot plugin (clawasm) intended to host a Wasm-based gateway, and a minimal wasm example (examples/hello-wasm) used by CI. The repository already has useful docs and CI scaffolding; however, several architectural inconsistencies and CI fragilities make the repo fragile for contributors and for reliable wasm builds. The recommendations below focus on conservative fixes you can adopt quickly and a road map for stabilization.

Repository layout & component roles

- Root workspace: contains workspace metadata and common CI/docs. Add resolver=2 (done) to avoid workspace resolver issues.
- clawasm/: native Godot plugin (cdylib). Intended to embed or interact with a Wasm runtime (WasmEdge). This is a native crate and may use host-only dependencies.
- examples/hello-wasm/: single-file wasm example. It is intentionally isolated with its own [workspace] to avoid pulling host-only deps into the workspace build.
- .github/workflows/wasm-ci.yml: runs Rust toolchain, installs WasmEdge, builds example for wasm, and runs WasmEdge to smoke-test.
- docs/: design notes, tests, TODOs and guidelines — well-written and actionable.

Build/test matrix and target strategy

- Current CI mixes wasm32-wasip1 and references to wasm32-wasi in docs/scripts. Rust stable (1.84+) removed wasm32-wasi in favor of wasip1 for preview features; pick one target across CI, scripts and docs. Recommendation: use wasm32-wasip1 for CI and examples when testing with WasmEdge that supports wasip1. For local development, document both and how to switch.
- MSRV: explicitly pin MSRV (e.g., CI should export RUSTUP_TOOLCHAIN=stable and test on the current stable; set an MSRV in docs and test matrix). Recommendation: use latest stable at release time and record MSRV in README (update during releases).
- Resolver: workspace.resolver = "2" added to root (avoids surprises with isolated example builds). Keep examples built with --manifest-path in CI to avoid pulling native-only deps.

Dependency & workspace risks

- Godot bindings mismatch: clawasm/src/lib.rs uses gdnative (Godot 3 API) while Cargo.toml depends on godot = "0.15" (Godot 4 crate). These are incompatible; either migrate code to godot-rust v0.15 (Godot 4) or change dependency back to the gdnative crate. This is a blocking compilation issue.
- wasmedge-sys is a host/native dependency. Keep it confined to the native plugin crate to avoid cross-compilation failures when building wasm examples.
- Cross-compilation pitfalls: avoid building workspace tests that pull native-only crates for wasm targets — build examples in isolation via --manifest-path or use cargo profiles to separate targets.

CI/CD weaknesses and fixes

- WasmEdge install in CI is brittle: tarball paths and versions are hardcoded. Use a maintained installer action where possible, validate paths, and fail early with clear messages.
- Cache placement: cache cargo registry/target to reduce CI time. Move cache step before heavy installs where possible and key on Cargo.lock + workflow file hash.
- Matrix builds: add a small matrix (linux/macos, rust-stable) to run basic cross-platform checks; and a separate matrix axis for wasm target (wasip1) vs native tests.
- Use explicit --manifest-path when building examples (fixed in scripts/test-wasm.sh).

Packaging & runtime contract for Wasm modules

- ABI expectations: standardize on wasm32-wasip1 (or wasm32-wasi if you decide) and document required exports (e.g., _start) and imports (WASI fd conventions). Publish a short Runtime Contract document describing: expected WASI preopens (data dir), logging (stdout/stderr), capability surface (net access, file access), and how Godot should instantiate and map FDs.
- Host imports & capability policy: minimize host-supplied imports. Where host functions are required, provide versioned shim modules and document stability guarantees.

Release/versioning guidance

- Follow semver. Use changelog automation (keep Unreleased section in CHANGELOG.md, require changelog entry in PRs). Release artifacts should include wasm blobs (optimized, stripped), native cdylib builds for common platforms, and checksums.
- Tag releases and attach artifacts in CI (built wasm and native libs). Consider GitHub Releases automation with a release pipeline step.

Security, testing & observability

- Testing: add integration tests that run the wasm artifacts under WasmEdge in CI (smoke tests) and verify expected outputs/exit codes.
- Fuzzing & sanitizers: for native plugin code, use sanitizers in dedicated CI runs; for wasm modules, consider wasm-snip/wasm-opt and size checks, and property tests on deterministic inputs. Consider AFL/LibFuzzer against exposed parsing functions where applicable.
- Runtime tracing & logs: use tracing crate and capture logs via WASI stdout/stderr; Godot host should surface logs. Provide structured JSON logs for observability.

Prioritized action plan

Immediate (0-2 days)
1. Merge low-risk fixes (branch feature/sonnet-arch-review): add resolver=2, update scripts to use wasm32-wasip1 and --manifest-path, add FIXME note in clawasm/src/lib.rs about Godot API mismatch. Owner: sonnet. Effort: 1-2 hours. (DONE)
2. CI hardening: change WasmEdge install to use a stable action or validate tarball paths, and move cache step earlier. Owner: gpt-5-mini. Effort: 0.5-1 day.

Short (1-2 sprints)
3. Fix Godot API mismatch: choose target Godot version (3 vs 4) and migrate clawasm code or dependencies accordingly. Owner: minimax. Effort: 3-5 days.
4. Add CI matrix for wasm target and native tests; add wasm smoke test step that asserts wasm runs and prints expected output. Owner: gpt-5-mini. Effort: 2-3 days.
5. Document the Wasm runtime contract (ABI, preopens, logs, capabilities). Owner: sonnet. Effort: 1-2 days.

Medium (quarter)
6. Add integration tests that run multi-node flows via the hub (mocked) and persistence testing under WasmEdge. Owner: minimax + sonnet. Effort: 2-4 weeks.
7. Release automation: produce optimized wasm blobs, artifacts, and automated Release notes via changelog. Owner: gpt-5-mini. Effort: 1-2 weeks.

Notes on CI failures observed

- Previously, CI references both wasm32-wasi and wasm32-wasip1; the mismatch caused build/test failures when the toolchain changed. Building examples without --manifest-path could pull native-only deps and fail when targeting wasm. The godot/gdnative mismatch will fail compilation for the plugin crate and can break workspace-wide builds.

Deliverables created

- Branch created: feature/sonnet-arch-review
- Low-risk fixes applied: workspace resolver, test script fix, added FIXME in clawasm/src/lib.rs
- Report saved: docs/ARCHITECTURE_REVIEW_sonnet.md
- Progress written: memory/sessions/subagents/56bb7144-8756-4129-86da-b3207bd7669e.md

Recommended next steps

1. Prioritize resolving the Godot API mismatch (decide Godot 3 vs 4). This is blocking for builds.
2. Harden CI: use manifest-path builds, reliable WasmEdge installation, and explicit matrix entries.
3. Author the runtime contract (small doc) and use it to align Godot plugin instantiation and CI smoke tests.

Appendix: small PR proposals exist in NOTES and are conservative; do not merge large refactors without review.


