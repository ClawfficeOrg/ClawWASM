// Migrated from gdnative (Godot 3) to godot-rust 0.5 (Godot 4).
// Key API changes:
//   - `use gdnative::prelude::*`       → `use godot::prelude::*`
//   - `#[derive(NativeClass)]`         → `#[derive(GodotClass)]`
//   - `#[inherit(Node)]`               → `#[class(base=Node)]`
//   - `#[gdnative::methods]`           → `#[godot_api]` + `impl INode for ClawWasm`
//   - `fn new(_owner: &Node) -> Self`  → `fn init(base: Base<Node>) -> Self`
//   - `fn _ready(&self, owner: &Node)` → `fn ready(&mut self)` inside `impl INode`
//   - `godot_init!(init)`              → auto-registered via `#[gdextension]` on
//                                       `ClawWasmExtension` + `#[derive(GodotClass)]`.
//
// See: https://godot-rust.github.io/book/
use godot::prelude::*;

mod engine_node;
pub use engine_node::ClawEngine;

/// Entry point — godot-rust v0.15 registers this automatically via the derive macro.
struct ClawWasmExtension;

#[gdextension]
unsafe impl ExtensionLibrary for ClawWasmExtension {}

/// A simple Godot 4 Node that initialises WASMEdge.
///
/// FIXME: Deep WASMEdge integration (wasmedge-sys calls) lives outside this Node for now.
///        Wire it in once the Godot 4 scaffold builds cleanly.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct ClawWasm {
    base: Base<Node>,
}

#[godot_api]
impl INode for ClawWasm {
    /// Called by godot-rust to construct the struct.
    fn init(base: Base<Node>) -> Self {
        ClawWasm { base }
    }

    /// Called when the node enters the scene tree for the first time.
    fn ready(&mut self) {
        godot_print!("ClawWASM: ClawWasm node is ready (Godot 4).");
        // The actual wasm execution lives on the `ClawEngine` node
        // (see `engine_node.rs`), which can be added to a scene
        // independently of this top-level entry point.
    }
}
