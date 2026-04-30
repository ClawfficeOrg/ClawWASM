---
name: WASM Build
description: How to build, optimize, and smoke-test WASM targets in ClawWASM.
when_to_use: Touching anything under examples/, scripts/test-wasm.sh, the wasm CI workflow, or wasm-target deps.
---

# WASM Build

## Target

We use **`wasm32-wasip1`** (WASI preview1). This is the WasmEdge-compatible
preview target on current stable Rust (1.84+). The older `wasm32-wasi` triple
was removed; do not reintroduce it.

```bash
rustup target add wasm32-wasip1
```

## Build wasm crates in isolation

The repo's root workspace contains host-only crates (`clawasm`,
`clawasm-engine`) that **cannot** cross-compile to wasm. Always pass
`--manifest-path` and exclude wasm crates from the workspace via their own
`[workspace]` table (see `examples/hello-wasm/Cargo.toml`).

```bash
cargo build --manifest-path examples/hello-wasm/Cargo.toml \
            --target wasm32-wasip1 --release
```

## Optimize

After `--release`, optionally:

```bash
wasm-opt -Oz -o out.wasm \
  examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm
```

Track binary size in PRs that touch wasm output (note bytes before/after).

## Smoke-test with WasmEdge

```bash
wasmedge examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm
```

Or:

```bash
bash scripts/test-wasm.sh
```

## CI quirks

- Use `dtolnay/rust-toolchain@stable` with `targets: wasm32-wasip1`.
- Install WasmEdge from the GitHub release tarball (the `setup-wasmedge`
  action 404'd in the past). Pin the version (currently 0.14.1) until a
  conscious bump.
- Cache `target` and `~/.cargo/registry` keyed on `**/Cargo.lock`.

## Common pitfalls

- **Pulling host-only deps into a wasm crate.** Symptom: "could not find
  `std::os::unix` for target wasm32-wasip1". Fix: move the dep behind a
  `#[cfg(not(target_arch = "wasm32"))]` gate or to a different crate.
- **Resolver mismatch.** Root workspace pins `resolver = "2"`. Isolated wasm
  crates with their own `[workspace]` should also use it.
- **Networking in wasm.** WASI preview1 has no sockets. Use a host-mediated
  fd pre-open or a JS bridge. See `docs/PLAN.md` §3.

## Required exports

Smoke wasm modules export `_start` (WASI command). Library wasm modules will
export named functions; document them in their crate README and add a test
that asserts the export list with `wasm-tools dump`.
