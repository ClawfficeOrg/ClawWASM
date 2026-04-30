//! End-to-end smoke test: load `examples/hello-wasm` under WasmEdge and
//! assert it exits 0 with non-empty stdout containing `"hello"`.
//!
//! Gated behind the `with-wasmedge` feature so it only runs when CI (or
//! a developer) has explicitly opted in to having a real `wasmedge`
//! binary available.

#![cfg(feature = "with-wasmedge")]

use engine::Engine;
use std::path::PathBuf;

/// Resolve `examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm`
/// relative to the workspace root, so the test runs from anywhere.
fn hello_wasm_path() -> PathBuf {
    // CARGO_MANIFEST_DIR points at clawasm/engine; jump up two levels.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("expected clawasm/engine/.. to resolve");
    workspace_root.join("examples/hello-wasm/target/wasm32-wasip1/release/hello-wasm.wasm")
}

#[test]
fn hello_wasm_runs_under_wasmedge() {
    let module = hello_wasm_path();
    assert!(
        module.is_file(),
        "expected built hello-wasm at {}; run `bash scripts/test-wasm.sh` first",
        module.display()
    );

    let engine = Engine::new().expect("Engine::new");
    // Probe so we get a clearer failure than a missing-binary error from run().
    let version = engine
        .probe()
        .expect("wasmedge --version (is WasmEdge installed and on PATH?)");
    eprintln!("wasmedge version: {version}");

    let instance = engine.load(&module).expect("Engine::load");
    let out = instance.run(&[]).expect("Instance::run spawn");

    assert!(
        out.success(),
        "wasm exited with {}: stdout={:?} stderr={:?}",
        out.exit_code,
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.to_lowercase().contains("hello"),
        "expected stdout to contain 'hello', got: {:?}",
        out.stdout
    );
}
