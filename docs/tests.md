# Tests and local wasm run instructions

To build and run the hello-wasm example locally (notes to address cross-platform differences):

1) Check which targets your toolchain supports:

   rustc --print=target-list | grep wasm

2) macOS (Apple Silicon / aarch64):

   - The system Rust toolchain `stable-aarch64-apple-darwin` may not support `wasm32-wasi`.
   - If `wasm32-wasi` is not listed, consider `wasm32-wasip1` (the newer WASI preview target) or use an x86_64 toolchain via Rosetta.

   Example (add wasm32-wasip1):

   rustup target add wasm32-wasip1

3) Linux / x86_64:

   - `wasm32-wasi` is usually available. Install with:

   rustup target add wasm32-wasi

4) Build only the example crate (recommended to avoid workspace resolver / dependency issues):

   cargo build --manifest-path examples/hello-wasm/Cargo.toml --target wasm32-wasi --release

   Or, if you need wasm32-wasip1 for your platform:

   cargo build --manifest-path examples/hello-wasm/Cargo.toml --target wasm32-wasip1 --release

Notes on workspace resolver and dependency conflicts
- If you see a warning about `resolver = "1"` vs `resolver = "2"`, you can either:
  A) Build the example in isolation using `--manifest-path` (recommended), or
  B) Set `workspace.resolver = "2"` in the root Cargo.toml to opt into the newer resolver (may affect dependency resolution across the workspace).

- If building the whole workspace fails (for example, a `godot = "^0.15"` dependency cannot be found), that usually means the examples
