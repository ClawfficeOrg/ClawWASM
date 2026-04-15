Minimal embedded engine wrapper for ClawWASM.

This crate is feature-gated: enable the "with-wasmedge" feature to compile against the native WasmEdge library using wasmedge-sys.

To install WasmEdge 0.16.1 on Debian/Ubuntu:
  curl -sSfL https://github.com/WasmEdge/WasmEdge/releases/download/0.16.1/WasmEdge-0.16.1-Ubuntu.deb -o WasmEdge.deb
  sudo apt install -y ./WasmEdge.deb

If WasmEdge native lib is not present, the crate builds in stub mode and returns a friendly error or stubbed output.
