#!/usr/bin/env bash
set -euo pipefail

# Build hello-wasm for wasm32-wasi and run with wasmedge
rustup target add wasm32-wasi || true
cargo build -p hello-wasm --target wasm32-wasi --release
WASM_PATH="target/wasm32-wasi/release/hello-wasm.wasm"
if [ ! -f "$WASM_PATH" ]; then
  echo "wasm file not found: $WASM_PATH" >&2
  exit 2
fi
wasmedge "$WASM_PATH"
