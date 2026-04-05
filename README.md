# ClaWASM

Minimal Rust Godot plugin scaffold embedding WASMEdge SDK (placeholder).

Build instructions:

- Install Rust and cargo.
- Add Godot Rust bindings and WASMEdge as needed.
- From repository root:

  cargo build -p clawasm --release

- Copy the generated library from target/(release|debug) to your Godot project's addons folder and register the plugin.
