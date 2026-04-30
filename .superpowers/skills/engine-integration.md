---
name: Engine Integration
description: Conventions for the clawasm-engine crate (WasmEdge wrapper, feature-gated).
when_to_use: Touching clawasm/engine/*.
---

# Engine Integration

`clawasm-engine` is the thin Rust wrapper around WasmEdge that the host plugin
(`clawasm`) and stand-alone tools call into. It lives at `clawasm/engine/`
and is a workspace member.

## Public surface (current)

```rust
pub struct Engine { /* … */ }
pub struct Instance { /* … */ }
pub struct Output { pub stdout: String, pub exit_code: i32 }

impl Engine {
    pub fn new() -> anyhow::Result<Self>;
    pub fn load(&self, path: &str) -> anyhow::Result<Instance>;
}
impl Instance {
    pub fn run(&self, args: &[String]) -> anyhow::Result<Output>;
}
```

Treat this as the **stable** surface. Any breaking change requires a
`feat(engine)!:` commit and a `BREAKING CHANGE:` footer.

## Feature flags

| Feature          | Effect                                                      |
| ---------------- | ----------------------------------------------------------- |
| (default = none) | Stub mode — `Instance::run` returns canned output.          |
| `with-wasmedge`  | Links `wasmedge-sys`; real WasmEdge VM. Requires native lib.|

Stub mode is what CI runs by default so contributors without WasmEdge installed
can still build, test, and review. The `with-wasmedge` job runs in a separate
CI matrix axis after WasmEdge is provisioned.

## Adding a new method

1. Add the method to `Engine` or `Instance` with a stub-mode default.
2. Cfg-guard the real implementation: `#[cfg(feature = "with-wasmedge")]`.
3. Provide a stub-mode counterpart that returns either a canned value or a
   clear `anyhow::Error` like
   `bail!("with-wasmedge feature required for {}", op)`.
4. Add a unit test for stub mode (always runs).
5. Add a `#[cfg(feature = "with-wasmedge")] #[test]` for the live path.
6. Document the method in `clawasm/engine/README.md`.

## Resource hygiene

- VM construction is expensive — cache `Engine` for the lifetime of the host.
- WASI preopens are configured at `Instance::load` time. Do not mutate after
  load.
- All WasmEdge handles are `Drop`-safe via `wasmedge-sys` wrappers; do not
  reach for raw pointers.

## Error mapping

Convert WasmEdge errors to `anyhow::Error` at the boundary:

```rust
let vm = Vm::new(None).map_err(|e| anyhow::anyhow!("WasmEdge VM init: {e:?}"))?;
```

Keep the error message structured; downstream Godot logging will surface it.
