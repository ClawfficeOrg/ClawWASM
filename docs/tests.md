# Tests and local wasm run instructions

To build and run the hello-wasm example locally:

1. Install Rust and add target:

   rustup default stable
   rustup target add wasm32-wasi

2. Build the example:

   cargo build --target wasm32-wasi --release -p hello-wasm

3. Run with WasmEdge CLI (install WasmEdge from https://github.com/WasmEdge/WasmEdge/releases):

   wasmedge target/wasm32-wasi/release/hello-wasm.wasm

Or use the provided script:

   ./scripts/test-wasm.sh

