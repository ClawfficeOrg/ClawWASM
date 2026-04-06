#!/usr/bin/env bash
set -euo pipefail

# Build hello-wasm for wasm32-wasip1 and run with wasmedge
# Note: wasm32-wasi was removed from Rust stable in 1.84+; use wasm32-wasip1
rustup target add wasm32-wasip1 || true
cargo build --manifest-path examples/hello-wasm/Cargo.toml --target wasm32-wasip1 --release
WASM_PATH="examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm"
if [ ! -f "$WASM_PATH" ]; then
  echo "wasm file not found: $WASM_PATH" >&2
  exit 2
fi
wasmedge "$WASM_PATH"
