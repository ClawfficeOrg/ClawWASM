//! Tiny CLI front-end for `clawasm-engine`. Mainly useful for ad-hoc
//! local testing without spinning up Godot.
//!
//! ```text
//! cargo run -p clawasm-engine -- path/to/module.wasm [args...]
//! ```

use anyhow::Result;
use engine::Engine;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let module = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: clawasm-engine <module.wasm> [args...]");
            std::process::exit(2);
        }
    };
    let rest: Vec<String> = args.collect();

    let engine = Engine::new()?;
    let version = engine.probe()?;
    eprintln!("[clawasm-engine] using {version}");

    let instance = engine.load(&module)?;
    let out = instance.run(&rest)?;

    // Mirror child stdout/stderr to ours so this behaves like a thin
    // wrapper around the runtime.
    print!("{}", out.stdout);
    eprint!("{}", out.stderr);
    std::process::exit(out.exit_code);
}
