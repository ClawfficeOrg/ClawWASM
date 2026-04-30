---
name: Test-Driven Development
description: Red → green → refactor for ClawWASM. Always write the failing test first.
when_to_use: Whenever you are about to add or change behaviour in any crate.
---

# Test-Driven Development

## Core rule

**No production change without a test that would fail without it.** This applies
to library code, scripts, and even doc-fixture changes that affect builds.

## The loop

1. **Red** — write the smallest failing test that captures the new behaviour.
   Run it. Confirm it fails *for the right reason* (not a compile error in an
   unrelated path).
2. **Green** — write the minimum code to make the test pass. Resist scope creep.
3. **Refactor** — clean up; tests stay green.

## Where tests live

| Crate                  | Test type            | Location                                  |
| ---------------------- | -------------------- | ----------------------------------------- |
| `clawasm`              | Unit                 | `clawasm/src/**/*` `#[cfg(test)] mod tests`|
| `clawasm`              | Integration (native) | `clawasm/tests/*.rs`                      |
| `clawasm-engine`       | Unit                 | inline `#[cfg(test)]`                     |
| `clawasm-engine`       | Integration          | `clawasm/engine/tests/*.rs`               |
| `examples/hello-wasm`  | Smoke (WASI runtime) | `scripts/test-wasm.sh` (CI)               |

## Patterns

- **Stub mode tests must not require WasmEdge.** Gate any test that needs the
  native WasmEdge library behind `#[cfg(feature = "with-wasmedge")]`.
- **Use `anyhow::Result` in tests** for ergonomic `?` propagation.
- **Snapshot tests** (`insta`) for protocol/JSON shapes that must stay stable.
- **WASI smoke tests** assert exit code + stdout substring at minimum.

## What "done" looks like

- [ ] Failing test exists and was confirmed failing.
- [ ] Implementation makes it pass.
- [ ] `cargo test --workspace --all-targets` is green.
- [ ] If touching WASM: `bash scripts/test-wasm.sh` is green.
- [ ] CHANGELOG entry added under `## Unreleased`.

## Anti-patterns

- Writing tests *after* the implementation "to satisfy CI". You'll miss bugs the
  red phase would have surfaced.
- Asserting on log output without a stable formatter.
- Tests that depend on network without `#[ignore]` and a `_online` suffix.
