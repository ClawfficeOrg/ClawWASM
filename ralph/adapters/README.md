# Ralph adapters

Each adapter is a tiny shell script that takes the iteration prompt as `$1`
and runs **one** turn of an LLM agent against the working directory.

| Adapter           | LLM harness        | Notes                                       |
| ----------------- | ------------------ | ------------------------------------------- |
| `claude-code.sh`  | Claude Code (`claude`) | Default. Recommended.                   |
| `codex.sh`        | OpenAI Codex CLI   | `@openai/codex`, non-interactive `--quiet`. |

Pick one with `RALPH_ADAPTER=<name>`:

```bash
RALPH_ADAPTER=codex bash ralph/loop.sh
```

## Adding a new adapter

1. Create `ralph/adapters/<name>.sh`, executable (`chmod +x`).
2. Read the prompt from `$1`. Exit 0 on success, non-zero on failure.
3. The adapter must **not** loop internally — `ralph/loop.sh` does that.
4. Document the requirements (CLI installation, auth, PATH).
5. Add the adapter to the table above.
6. Open a `feat(ralph): add <name> adapter` PR.

## Why so thin?

Everything that should persist between iterations lives in tracked files
(`AGENTS.md`, `ralph/PLAN.md`, `docs/MEMORY.md`, `docs/LEARNINGS.md`). The
adapter is the only tool-specific surface, so swapping LLM vendors stays
cheap.
