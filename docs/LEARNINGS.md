# Learnings

Append-only lab notebook of non-obvious things discovered during
development. See `.superpowers/skills/memory-keeper.md` for the protocol.

> Format: dated H3 entries, newest at the bottom (chronological). Each entry
> is 1–4 sentences and links to the PR that produced it where possible.

---

### 2026-04-12 — `wasm32-wasi` was removed in Rust 1.84+

Tried `rustup target add wasm32-wasi` on stable Rust ≥ 1.84 and got "error:
toolchain ... does not support target 'wasm32-wasi'". Switched to
`wasm32-wasip1` everywhere (CI, scripts, docs). PR #5.

### 2026-04-12 — Workspace resolver mismatch breaks isolated wasm builds

Building `examples/hello-wasm` from the workspace root pulled in
`wasmedge-sys` (a host-only dep) and failed for `wasm32-wasip1`. Fix:
`examples/hello-wasm/Cargo.toml` declares its own empty `[workspace]`
table, and CI uses `--manifest-path` to build it in isolation. The root
workspace also pins `resolver = "2"`. PRs #5, #6.

### 2026-04-29 — godot-rust crate version vs book version

The godot-rust *book* refers to "v0.15" but the crate published to
crates.io is in the 0.5.x range. `Cargo.toml` should pin `godot = "0.5"`
(or higher), not `0.15`. PR #7.

### 2026-04-29 — Stale branches after the OpenClaw memory corruption

After the upgrade, several remote branches still existed for already-merged
PRs (#1–#5, #7) and a duplicate `feature/v0.1.0-wasm-hello`. They were
pruned along with `feature/docs-quickstart` (a stale fork that would have
reverted later work). Always check `git log origin/main..origin/<branch>`
before deleting; deleting a branch that's only an ancestor of `main` is
safe.

### 2026-04-30 — `wasmedge-sys 0.17.5` is ABI-incompatible with WasmEdge 0.14.1

Wiring CI to actually compile `clawasm` (the Godot host crate) against
libwasmedge surfaced ~38 type errors in `wasmedge-sys 0.17.5/src/types.rs`
(`WasmEdge_ValType` is a struct in 0.14.1 headers but the bindings expect
an integer-like enum). The 0.17.x line targets newer WasmEdge releases.
Resolution options for the engine-MVP PR: (a) downgrade `wasmedge-sys`
to a 0.1x version compatible with WasmEdge 0.14.1, (b) bump the WasmEdge
pin to a release the 0.17.x bindings support, or (c) route all WasmEdge
use through `clawasm-engine` (feature-gated) and drop the direct dep
from `clawasm`. Until that PR lands, scaffolding-CI scopes
`cargo clippy` to `-p clawasm-engine --no-default-features` and skips
host-side `cargo check -p clawasm`. PR #9.

**Resolved (PR #10):** Option (c) — `clawasm` no longer depends on
`wasmedge-sys` directly; the engine wrapper crate owns the native dep,
and the host plugin gets a stub-mode `cargo check` for free. `clawasm`
exposes a forwarding `with-wasmedge` feature that pulls
`clawasm-engine/with-wasmedge` for callers that want the native path.
CI restored `cargo clippy --workspace` and `cargo check -p clawasm`.

### 2026-04-30 — Engine v0.2.0 ships subprocess to `wasmedge`, not in-process embedding

`wasmedge-sys 0.4.x` (which `clawasm-engine` originally pinned) and
`0.17.x` are both ABI-incompatible with the WasmEdge 0.14.1 release
we install in CI — 0.4.x references removed `WasmEdge_ImportObject*`
and `WasmEdge_HostRegistration_WasmEdge_Process` symbols; 0.17.x
expects the newer `WasmEdge_ValType` ABI. Rather than blocking the
engine MVP on finding a binding version that lines up (or bumping
the WasmEdge install pin and re-validating downstream), v0.2.0
implements `Instance::run` by `Command`-ing the `wasmedge` CLI
binary that we already install for the wasm-smoke job. Pros: zero
native build deps for consumers, works against any WasmEdge release
that ships a CLI, public API stays stable for an in-process swap-in.
Cons: per-invocation process-launch overhead, no fine-grained host
function injection. Tracked as a v0.3.0+ follow-up in `ralph/PLAN.md`
(Q3). PR #11.

### 2026-05-01 — `godot-rust` v0.5 signal emit takes `&GString`, not `GString`

Wiring `ClawEngine` exposed two godot-rust 0.5 quirks:
(1) `signals().<name>().emit(...)` takes its arguments by reference
(`&GString`, `i64`), not by value, so signal payloads must be borrowed.
(2) `GString: From<String>` is *not* implemented — only
`From<&String>` and `From<&str>`. Construct via `GString::from(&s)` for
owned strings. Catching this at compile time saved a confusing runtime
signal panic later. Surface in any future Rust→GDScript glue.

### 2026-05-06 — Homebrew `rustc` shadows rustup shims when cross-compiling to `wasm32-wasip1`

On macOS with Homebrew Rust installed, `/opt/homebrew/bin/rustc` appears earlier
in `$PATH` than `~/.cargo/bin/rustc` (the rustup shim). Running `cargo build
--target wasm32-wasip1` then fails with "can't find crate for `std`" even though
`rustup target list --installed` shows `wasm32-wasip1` present. Fix: set
`RUSTC=~/.cargo/bin/rustc` explicitly, or ensure `~/.cargo/bin` comes before
Homebrew entries in `$PATH`. CI is unaffected (dtolnay action, no Homebrew Rust).
Document in any contributor setup guide. PR #12.

### 2026-05-06 — Godot headless requires `.godot/extension_list.cfg` to load GDExtensions

`Godot --headless --path <project>` skips the editor startup that normally writes
`.godot/extension_list.cfg`; without it no GDExtension loads at runtime, so native
classes like `ClawEngine` are undefined and GDScript parse fails. Fix: create
`.godot/extension_list.cfg` manually (one `res://` path per line) before the
headless run, or open the project once in the GUI so Godot writes the file.
Relevant for any future CI headless Godot job that loads native extensions. PR #12.

### 2026-04-30 — `sh -c "single-cmd"` forks on Linux, execs on macOS

The `Runner::stop` test was green on macOS and red on Linux. Cause:
`sh -c "sleep 30"` — macOS's `/bin/sh` recognises the single-command
pattern and `exec`s into `sleep`, so killing the shell PID kills the
only process. Linux's `bash` always forks `sleep` as a child; killing
the shell PID leaves `sleep` orphaned holding the stdout pipe's write
end open, which deadlocks the reader threads and the test times out at
Fix: invoke `sleep` directly (`Command::new("sleep")`). General
lesson: when a test relies on `Child::kill()` semantics, never wrap
the command in a shell unless you explicitly `exec` inside it. PR #12.

### 2026-05-06 — `ClawEngine` Godot smoke GREEN on macOS (Godot 4.6.2, godot-rust 0.5.2)

Manual headless smoke on macOS (arm64, Godot 4.6.2.stable, WasmEdge 0.14.1):
godot-rust 0.5.2 initialises correctly ("Initialize godot-rust (API v4.6.stable)"),
`ClawEngine` registers as a class, `register_module` resolves the `res://` path,
`start` spawns WasmEdge, `stdout_line` fires with `"hello-wasm"`, and `finished`
fires with exit code 0. Required two environment steps not in the original runbook:
(1) set `WASMEDGE_BIN` to `$HOME/.wasmedge/bin/wasmedge` (not on Godot's PATH),
(2) pre-create `.godot/extension_list.cfg` (see entry above). Both caveats are
now documented in `tests/godot-smoke/README.md`. Linux smoke still pending.
PR #12.

### 2026-05-08 — WasmEdge 0.14.1 WASI-NN llama.cpp backend does not support Gemma 4 E2B

Gemma 4 E2B uses Per-Layer Embeddings (PLE) and a hybrid sliding-window /
global attention architecture added to llama.cpp well after WasmEdge 0.14.1
shipped. bartowski's GGUF quants were built with llama.cpp b8746; WasmEdge
0.14.1's bundled llama.cpp backend is far older and will reject or misparse
the GGUF. As a result, the WasmEdge WASI-NN path (`--nn-preload
default:GGUF:AUTO:...`) cannot be used for this model at our pinned version.
Decision: `CLLawM` shells out to the `llama-cli` binary directly (same
subprocess pattern as `ClawEngine` with `wasmedge`). The WASI-NN path can
be revisited if/when WasmEdge ships a newer llama.cpp plugin. PR feature/llm-inference.

### 2026-05-10 — `download-model.sh` had two bugs: wrong CLI binary and wrong filename prefix

`huggingface-cli` has been superseded by `hf` (same `huggingface_hub` package,
new binary name); the old binary now exits immediately with a deprecation
warning. Additionally, bartowski's GGUF filenames use the prefix
`google_gemma-4-E2B-it-` (matching the original model repo slug), but the
script was constructing `gemma-4-E2B-it-` (missing `google_`), causing a
"File not found in repository" error even after the CLI was fixed. Fix:
detect `hf` first (`command -v hf`) with a graceful fallback to
`huggingface-cli`; correct the `FILENAME` prefix; strip any trailing `.gguf`
from the `QUANT` argument defensively. PR feature/llm-inference.

### 2026-05-10 — `ffi error -1` from llama-cpp-2 needs step-by-step diagnostics to pinpoint failure

When `CLLawM` reported `ffi error -1` during inference the error came from one of
several `ctx.decode()` / sampling calls in `run_inference` — the bare anyhow `?`
propagation gave no context about *which* step. Fix: added a `tlog!` macro that
sends a `LlmEvent::Log` through the existing mpsc channel (forwarded to
`godot_print!` on the main thread) AND prints to `eprintln!` for terminal
visibility. Every major step (1/8 through 8/8) is logged so the last printed step
before an error identifies the culprit. Every `?` is also wrapped in `.context()`
so the anyhow chain includes a description of the failing call. The full chain
appears via `{:#}` in both `godot_error!` and the `eprintln!` inside the thread.
This pattern (LlmEvent::Log channel + tlog macro) should be reused for any future
background-thread GDExtension work. PR feature/llm-inference.

### 2026-05-10 — llama-cpp-2 v0.1.146 bundled llama.cpp likely too old for Gemma-4 E2B

`llama-cpp-2` 0.1.146 is the newest release on crates.io. Its bundled llama.cpp
build predates Gemma-4's Per-Layer Embeddings (PLE) and hybrid attention; the
in-process `ctx.decode()` returns non-zero (FFI -1) when processing the prompt
batch. The system `llama-cli` binary (Homebrew / manual build) works because it
ships a much newer llama.cpp. Two paths forward: (A) if a newer llama-cpp-sys-2
becomes available, upgrade; (B) write a thin C++ GDExtension using godot-cpp
that calls llama.cpp directly via CMake submodule/FetchContent, giving full
control of the llama.cpp version. Path B is preferred long-term as it removes
the Rust-crate indirection and makes updating llama.cpp trivial. PR feature/llm-inference.

