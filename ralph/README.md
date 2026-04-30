# ralph/

Autonomous development substrate for ClawWASM. See `.superpowers/skills/ralph-loop.md`
for the protocol; this README is a quickstart.

## Run the loop

```bash
# default adapter (Claude Code), infinite iterations
bash ralph/loop.sh

# 5 iterations only, with the Codex adapter
RALPH_ADAPTER=codex RALPH_MAX_ITERATIONS=5 bash ralph/loop.sh
```

## Stop the loop

```bash
touch ralph/STOP   # graceful — exits at next iteration boundary
rm   ralph/STOP   # to resume later
```

## Files

| File              | Role                                                            |
| ----------------- | --------------------------------------------------------------- |
| `PROMPT.md`       | The stable prompt fed to the agent every iteration. Edit rarely.|
| `PLAN.md`         | The active plan. The agent edits this each iteration.           |
| `loop.sh`         | The runner. Bounds iterations, calls the chosen adapter.        |
| `adapters/*.sh`   | Per-LLM-CLI adapters (claude-code, codex, …).                   |
| `STOP`            | Sentinel — when present, the loop exits cleanly.                |

## Environment variables

| Var                       | Default      | Effect                                    |
| ------------------------- | ------------ | ----------------------------------------- |
| `RALPH_ADAPTER`           | `claude-code`| Which adapter under `ralph/adapters/`.    |
| `RALPH_MAX_ITERATIONS`    | `0` (∞)      | Stop after N iterations.                  |
| `RALPH_SLEEP`             | `30`         | Seconds to sleep between iterations.      |
| `RALPH_ITER_TIMEOUT_SEC`  | `1800` (30m) | Hard wall-clock cap per iteration.        |
