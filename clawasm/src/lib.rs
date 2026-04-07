// FIXME(arch-review): This file uses the `gdnative` crate API (Godot 3 bindings),
// but Cargo.toml declares `godot = "0.15"` which is the godot-rust crate for Godot 4.
// These APIs are incompatible — this crate will NOT compile until migrated to the
// godot-rust 0.x (Godot 4) API. See: https://godot-rust.github.io/book/
// Tracked: replace gdnative::* usages with godot::prelude::* and GodotClass derive.
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct ClawWasm;

#[gdnative::methods]
impl ClawWasm {
    fn new(_owner: &Node) -> Self {
        ClawWasm
    }

    #[export]
    fn _ready(&self, owner: &Node) {
        godot_print!("ClawWASM: ClawWasm node is ready.");
        // Placeholder: initialize WASMEdge here when integrating further
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ClawWasm>();
}

godot_init!(init);
