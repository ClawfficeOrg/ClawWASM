# ClaWASM: Wasm Embedding of OpenClaw-like Gateway for Clawffice-Space

## Overview
This document outlines a plan to embed a minimal OpenClaw-like gateway/software/memory system inside a Wasm32 container using the WASMEdge SDK, so that Clawffice-Space can run nodes on Wasm-capable targets (consoles, SteamDeck, browsers via WASI+JS, etc.) without requiring Docker, Podman, or a full OS. The design takes inspiration from the ironclaw project (if accessible) and the WASMEdge Rust SDK to create a secure, portable node that can participate in a federated Clawffice network.

## Goals
- Compile a minimal subset of OpenClaw (HTTP server, WebSocket handler, in-memory session/agent/tools) to `wasm32-unknown-unknown` using WASMEdge.
- Expose the gateway to the host Godot engine via WASI file descriptors or JS glue so the plugin can start/stop the gateway and exchange messages.
- Persist memory (sessions, agent state, project memory) inside the Wasm sandbox using WASI-friendly storage (host filesystem via WASI fd, or embedded SQLite via wasmfs, or IndexedDB via JS glue).
- Scale to multiple Wasm nodes (each running in its own container) and connect them via a lightweight bootstrap relay or rendezvous server (WebSocket or TCP).
- Identify which parts of OpenClaw need to be stripped or adapted to be Wasm-friendly (e.g., avoid direct FS access where not allowed, limit spawning of OS processes, use async where possible).
- Provide clear build and test instructions: `cargo build --target wasm32-unknown-unknown`, then `wasm-opt`, then run with `wasmedge` and verify the gateway endpoint.

## Assumptions & Scope
- We do **not** attempt to run the full OpenGPT or heavy LLM inference inside Wasm (too heavy); instead we focus on the gateway, session handling, tool execution (if tools are Wasm-compatible or proxied), and message routing.
- The Wasm node acts as a **leaf or relay node** in the Clawffice network: it accepts incoming WebSocket/HTTP connections, manages sessions, executes allowed tools (if they are Wasm-safe), and forwards messages to other nodes or a central hub.
- Authentication and encryption: we assume TLS termination happens at a reverse proxy or the Wasm node can use `rustls` if the target supports networking and sufficient entropy; for simplicity we may start with plaintext WS/HTTP behind a trusted TLS terminator.
- The host Godot plugin (ClawWASM) is responsible for:
  * Loading the Wasm binary.
  * Providing WASI overrides (e.g., stderr/stdout to Godot log, fs access to a sandboxed directory).
  * Starting/stopping the Wasm instance.
  * Sending/receiving messages via a channel (e.g., a Godot Signal or custom interface) to/from the Wasm gateway.

## Detailed Plan

### 1. Toolchain & Target
- **Target**: `wasm32-unknown-unknown`
- **Rust toolchain**: `rustup target add wasm32-unknown-unknown`
- **WASMEdge**: Use the `wasmedge-sys` crate (low-level bindings) or `wasmedge` high-level if available; we will primarily use the `wasmedge-sys` crate to instantiate a VM, load the Wasm bytecode, and invoke the exported `_start` function.
- **Optimization**: After `cargo build --target wasm32-unknown-unknown`, run `wasm-opt -Oz -o output.wasm input.wasm` to shrink the binary.
- **Testing**: Run with `wasmedge --dir .:. /path/to/output.wasm` to grant access to the current directory (adjust as needed).

### 2. Minimal OpenClaw Subset to Port
We identify the following components as essential for a basic gateway node:
- **HTTP Server**: `actix-web` or `warp` (does it support Wasm? `actix-web` does not currently support Wasm due to OS dependencies; we may need to use `warp` or `hyper` + `tokio` with Wasm-compatible features, or fall back to a tiny custom server using `tokio` + `tungstenite` for WS only).
  * Investigation needed: check if `actix-web` has a `wasm` feature; if not, we will use `warp` + `tokio` (which does support Wasm when targeting `wasm32-unknown-unknown` with the `wasm-bindgen` or direct syscall approach).
  * Alternative: use `hyper` + `tokio` (both have Wasm support when built for `wasm32-unknown-unknown` with appropriate flags).
- **WebSocket**: `tokio-tungstenite` works with Wasm when using the `ws` feature and providing a compatible I/O stream (we can adapt a WASI socket or use a JS WebSocket bridge if running in a browser-like Wasm environment).
- **Session & Agent Management**: In-memory structs (no OS spawning); we avoid `exec`, `process`, and any tool that spawns OS threads. We can still run subagents if they are pure-Rust and Wasm-safe (e.g., simple text processing, math, or JSON transformation), but we will disable heavy tools like `browser`, `canvas`, `exec` unless we provide Wasm-safe stubs.
- **Tools**: We will define a Wasm-safe tool subset (e.g., `memory_search`, `memory_get`, `web_search` if we can proxy via HTTP to an external service, `web_fetch` similarly). Tools requiring OS access (`exec`, `process`, `browser`, `canvas`) will be stubbed to return an error or a Wasm-safe simulation if possible.
- **Memory Persistence**: 
  * Primary: Use WASI file descriptors to read/write files from a host-mounted directory (the Godot plugin can map a sandboxed folder into the Wasm instance via WASI `__wasi_fd_*`).
  * Alternative: Embed `sqlite3` via the `rusqlite` feature `bundled` (which bundles SQLite) and store memory in a `.sqlite` file inside the Wasm linear memory or a WASI-mapped file.
  * Experimental: Use IndexedDB via JS glue if the Wasm is running in a browser context (less likely for consoles/SteamDeck but useful for testing).
- **Configuration**: Use `config` crate or `serde_json` to read a `config.json` from the WASI-mapped directory at startup.
- **Logging**: Use `tracing` + `tracing-subscriber` with a WASI-compatible logger (e.g., writing to stderr/stdout which the host can capture via WASI fd 2/1).

### 3. WASI Integration (Host ↔ Wasm Boundary)
The Godot plugin (ClawWASM) will instantiate the Wasm module with the following WASI overrides:
- **stdin/stdout/stderr**: Map to Godot’s logging system or a custom pipe so the plugin can capture logs and display them in the editor.
- **fd 3+**: Open a sandboxed directory (e.g., `/var/lib/clawwasm/data`) for persistence; the plugin will create and manage this directory on the host and map it into the Wasm instance.
- **Networking**: WASI sockets are not yet fully standardized; for now we will use a JS-like approach if targeting browsers, or for native Wasm (console/SteamDeck) we will rely on the host to perform TCP/UDP and pass data via WASI file descriptors (e.g., the host accepts a TCP connection, then dupes the socket into the Wasm instance as a pre-opened fd, and the Wasm instance reads/writes using that fd). This requires cooperation from the host plugin.
  * Simpler alternative: Run the gateway as a **client-only** node that connects out to a central WebSocket relay (the host opens the TCP connection, then passes the socket into the Wasm instance). The Wasm instance never opens listeners; it only dials out. This avoids the need for WASI listening sockets and works well in restrictive environments.
  * For a true server node (listening for incoming connections), we would need the host to accept the connection, then duplicate the socket into the Wasm instance as a pre-opened fd (WASI fd >=3). The Wasm instance would then use `poll` or `my_socket` APIs to read/write. This is doable but requires careful coordination.

Given the complexity of WASI networking, the **client-outbound-only model** (Wasm node dials to a central relay) is the safest first step for consoles/SteamDeck where inbound ports may be blocked or unavailable. The Clawffice-Space hub can act as the relay: each Wasm node connects to `wss://hub.clawffice.space/node` and registers itself; the hub then routes messages between nodes.

### 4. Communication Design (Node ↔ Hub)
- **Protocol**: Use JSON over WebSocket (same as existing OpenClaw agent-to-agent or session messaging).
- **Each Wasm node**: on startup, connects to the hub’s WebSocket endpoint (`wss://hub/claw/node`) and sends a `HELLO` message containing:
  * `node_id`: a UUID or hash of the Wasm instance + timestamp.
  * `capabilities`: list of tool IDs this node supports (e.g., `["memory_search", "memory_get", "web_search(proxy)", "web_fetch(proxy)]"`).
  * `version`: "clawwasm-0.1.0"
- The hub acknowledges and adds the node to its registry.
- **Incoming messages**: the hub forwards `SESSION_MSG`, `AGENT_MSG`, `TOOL_REQUEST`, etc. to the appropriate node based on `node_id` in the message header.
- **Outgoing messages**: the Wasm node sends messages to the hub with the `to` field set to the target node ID or `"hub"` for broadcast/general messages.
- **Heartbeat**: each Wasm node sends a periodic `PING`; the hub replies with `PONG`. If no PONG within timeout, the hub considers the node dead.
- **Security**: All WebSocket connections should be WSS (TLS) via a trusted terminator; the Wasm instance does not need to handle TLS itself if the host provides a cleartext WebSocket over a trusted TLS tunnel (or we use `rustls` if the target allows and entropy is sufficient).

### 5. Build & Test Instructions
```bash
# 1. Build for Wasm
cargo build --target wasm32-unknown-unknown --release

# 2. Optimize (optional but recommended)
wasm-opt -Oz -o target/wasm32-unknown-unknown/release/clawwasm.wasm target/wasm32-unknown-unknown/release/clawwasm.wasm

# 3. Run with WASMEdge (assuming we have mapped a data directory)
# Example: map the current directory as readable/writable via WASI
wasmedge --dir .:. --net --nn_preload bundler:wasmedge_nn_interface_bundler_none.wasm target/wasm32-unknown-unknown/release/clawwasm.wasm

#   (The --net flag enables networking; adjust the dir mapping as needed.)
# 4. Verify: the Wasm instance should print a startup message and attempt to connect to the WebSocket hub.
#    Check the host Godot plugin logs for connection attempts.
```

### 6. Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| WASMEdge SDK version mismatch | Pin to a known-good version in Cargo.toml; test with the version used in ironclaw if available. |
| Memory limits in Wasm (typically <2GB) | Keep in-memory datasets small; use disk/WASI-fd for persistence; avoid loading large models. |
| Lack of OS process spawning | Restrict tools to pure-Rust, Wasm-safe functions; disable heavy tools unless we provide Wasm-safe stubs. |
| Networking complexity | Start with client-outbound-only model (dial to hub); defer server/listener support until WASI sockets are stable or we use a host-mediated approach. |
| Entropy for TLS | If we need TLS inside Wasm, use a pre-shared key or rely on the host to terminate TLS and provide cleartext WS. |
| Debugging | Enable `tracing` and log to WASI stderr; the host plugin can capture and display logs. |

### 7. Reference Projects
- **ironclaw**: (if accessible) inspect how it structures the Wasm module, what WASI imports it uses, and how it handles messaging.
- **wasmedge-examples**: The official WASMEdge repo contains examples of HTTP servers and WebSocket clients in Rust that target Wasm.
- **Cloudflare Workers Architecture**: For inspiration on isolating untrusted code and providing a limited set of APIs via bindings.

### 8. Milestones
1. [ ] Create a minimal Wasm module that prints “hello” when run with wasmedge.
2. [ ] Add WASI fd mapping for stdout/stderr to capture logs in Godot.
3. [ ] Implement a minimal TCP client (using WASI socket or host-mediated) that connects to a hardcoded echo server and sends/receives a line.
4. [ ] Replace the echo server with a WebSocket dial to a local test ws server and exchange a JSON message.
5. [ ] Add session-in-memory structs and a simple HTTP/WS handler that echoes JSON.
6. [ ] Add persistence: write a JSON file to the WASI-mapped directory on each state change and read it on startup.
7. [ ] Define the Wasm-safe tool subset and stub the rest.
8. [ ] Test the full flow: Godot plugin loads the Wasm, starts it, sends a control message, receives a response, and shuts it down.
9. [ ] Document performance and memory usage; optimize as needed.
10. [ ] Publish to the Clawffice-Space asset gateway or Godot asset library as an official plugin.

---
*Generated as initial plan for ClaWASM Wasm embedding of OpenClaw-like gateway.*