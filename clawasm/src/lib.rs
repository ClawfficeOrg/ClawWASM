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
