---
name: Memory Keeper
description: How to maintain docs/MEMORY.md and docs/LEARNINGS.md so future agents don't repeat past mistakes.
when_to_use: Whenever you make a decision, hit a surprise, or land a non-obvious fix.
---

# Memory Keeper

The repository has two persistent memory surfaces. They are checked into git so
they survive any agent context wipe, IDE crash, or runtime upgrade.

## `docs/MEMORY.md` — the long-lived ledger

Use for things that should *always* be true going forward:

- Architectural decisions ("we target wasm32-wasip1, not wasm32-wasi").
- Tooling versions pinned for a reason ("WasmEdge 0.14.1 because 0.16 breaks X").
- Owner / contact for a subsystem.
- Invariants ("`clawasm-engine` must build with `--no-default-features` on wasm targets").

**Format:** dated H2 sections, newest at the top.

```markdown
## 2026-04-29 — Decision: Godot 4 only

We dropped Godot 3 (gdnative) support. clawasm uses godot-rust 0.5+. Reason:
upstream gdnative is unmaintained for 4.x. Migration done in PR #7.
```

## `docs/LEARNINGS.md` — the lab notebook

Use for things you discovered the hard way:

- A subtle build flag that fixes CI on macOS arm64.
- A WasmEdge runtime quirk.
- A godot-rust API that does not behave like the docs claim.

**Format:** append-only, dated H3 entries.

```markdown
### 2026-04-12 — wasm32-wasi removed in Rust 1.84+

Tried `rustup target add wasm32-wasi` on Rust 1.85; gone. Switched to
`wasm32-wasip1` everywhere (CI, scripts, docs). PR #5.
```

## Update rules

1. **In the same PR.** Memory updates ride with the change that produced the
   discovery. Don't batch them.
2. **Append, never delete.** If a learning becomes obsolete, write a new
   superseding entry that links to the old one.
3. **Be terse.** 1–4 sentences. Link to the PR for context.
4. **No secrets.** Never put tokens, internal URLs, or PII here.

## Pre-flight read

When an agent starts a session it should `cat docs/MEMORY.md docs/LEARNINGS.md`
(or the equivalent) before reading code. This is cheap and catches "I was
about to redo a thing we already learned doesn't work."
