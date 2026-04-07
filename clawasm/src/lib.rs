// Migrated from gdnative (Godot 3) to godot-rust v0.15 (Godot 4).
// Key API changes:
//   - `use gdnative::prelude::*`       → `use godot::prelude::*`
//   - `#[derive(NativeClass)]`         → `#[derive(GodotClass)]`
//   - `#[inherit(Node)]`               → `#[class(base=Node)]`
//   - `#[gdnative::methods]`           → `#[godot_api]` + `impl INode for ClawWasm`
//   - `fn new(_owner: &Node) -> Self`  → `fn init(base: Base<Node>) -> Self`
//   - `fn _ready(&self, owner: &Node)` → `fn ready(&mut self)` inside `impl INode`
//   - `godot_init!(init)`              → auto-registered via `#[derive(GodotClass)]`
//
// See: https://godot-rust.github.io/book/godot-api/builtins.html
use godot::prelude::*;

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
        // FIXME: Initialize WASMEdge runtime here.
        // Example (stubbed — requires wasmedge-sys feature-gating for non-wasm targets):
        //   let vm = wasmedge_sys::Vm::new(None).expect("WasmEdge VM init failed");
        //   godot_print!("ClawWASM: WasmEdge VM initialised.");
    }
}
