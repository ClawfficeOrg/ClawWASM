---
name: Godot Binding
description: How the clawasm Godot 4 plugin is wired (godot-rust 0.5+, GDExtension).
when_to_use: Touching clawasm/src/*, clawasm/Cargo.toml, or anything that interacts with Godot.
---

# Godot Binding

## Stack

- **Godot 4.x** (no Godot 3 / `gdnative` support — that path is dead).
- **godot-rust** crate `godot` ≥ 0.5 (note: docs sometimes refer to
  "v0.15" which is the *book* version, not the crate version on crates.io).
- **GDExtension**: built as `cdylib`, loaded via a `.gdextension` manifest in
  the consuming Godot project.

## Macro cheat sheet

| Old (gdnative)                | New (godot-rust 0.5+)                          |
| ----------------------------- | ---------------------------------------------- |
| `use gdnative::prelude::*`    | `use godot::prelude::*`                        |
| `#[derive(NativeClass)]`      | `#[derive(GodotClass)]`                        |
| `#[inherit(Node)]`            | `#[class(base = Node)]`                        |
| `#[methods]`                  | `#[godot_api]` + `impl INode for ClawWasm`     |
| `fn new(_owner: &Node)`       | `fn init(base: Base<Node>) -> Self`            |
| `_ready(&self, owner)`        | `fn ready(&mut self)` inside `impl INode`      |
| `godot_init!(init)`           | `#[gdextension] unsafe impl ExtensionLibrary`  |

## Building the plugin

```bash
cargo build -p clawasm --release
# artifact: target/release/libclawasm.{dylib,so,dll}
```

Copy the artifact + a `.gdextension` manifest into your Godot project's
`addons/clawasm/` folder. Example manifest in `clawasm/clawasm.gdextension`
(when it exists).

## Host-only constraints

`clawasm` depends on `wasmedge-sys` (native) and **must not be built for any
wasm target.** It lives in the workspace, but wasm CI explicitly avoids it
via `--manifest-path examples/hello-wasm/Cargo.toml`.

## Testing

- **Unit tests** that don't need a Godot runtime go in `clawasm/src/**`
  inside `#[cfg(test)] mod tests`.
- **Integration with Godot itself** is manual today: open a tiny Godot 4
  project under `tests/godot-smoke/` (TODO) and verify the `ClawWasm` node
  prints "ready". This is gated behind `cargo test --features godot-smoke`
  and skipped in CI until we have a headless Godot runner.

## Common pitfalls

- **Mixing the two macro families.** If you see `#[derive(NativeClass)]`
  with `use godot::prelude::*` it will not compile. Pick one (always 0.5+).
- **`Base<Node>` lifetime.** Don't store `&self.base` across awaits. Re-borrow
  it each call.
- **Threading.** Godot calls are main-thread only. WasmEdge calls from a
  background thread must marshal results back via channels.
