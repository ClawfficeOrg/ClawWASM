# ClawWASM — Quick start

Quick start (build & run hello-wasm)

1) Install Rust toolchain and add the WASI preview target:

   rustup default stable
   rustup target add wasm32-wasip1

2) Build the example (isolated):

   cargo build --manifest-path examples/hello-wasm/Cargo.toml --target wasm32-wasip1 --release

3) Run the wasm with WasmEdge (assumes WasmEdge 0.16.1 installed):

   wasmedge target/wasm32-wasip1/release/hello-wasm.wasm

Notes: if your platform/toolchain prefers , substitute the target and path accordingly.
