#!/usr/bin/env bash
# ralph/loop.sh — autonomous development loop for ClawWASM.
#
# Usage:
#   bash ralph/loop.sh                     # default adapter (claude-code), infinite
#   RALPH_ADAPTER=codex bash ralph/loop.sh # use ralph/adapters/codex.sh
#   RALPH_MAX_ITERATIONS=5 bash ralph/loop.sh
#
# Stop:
#   touch ralph/STOP   # graceful: exits at next iteration boundary
#
# See `.superpowers/skills/ralph-loop.md` for the full protocol.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ADAPTER="${RALPH_ADAPTER:-claude-code}"
ADAPTER_SCRIPT="ralph/adapters/${ADAPTER}.sh"
MAX_ITER="${RALPH_MAX_ITERATIONS:-0}"   # 0 = unlimited
SLEEP_BETWEEN="${RALPH_SLEEP:-30}"
ITER_TIMEOUT="${RALPH_ITER_TIMEOUT_SEC:-1800}"  # 30 min default

if [ ! -x "$ADAPTER_SCRIPT" ]; then
  echo "ralph: adapter '$ADAPTER_SCRIPT' not found or not executable." >&2
  echo "ralph: available adapters:" >&2
  ls -1 ralph/adapters/ 2>/dev/null | sed 's/^/  - /' >&2
  exit 2
fi

log() { printf '[ralph %s] %s\n' "$(date -u +%FT%TZ)" "$*"; }

iter=0
while :; do
  if [ -f ralph/STOP ]; then
    log "STOP file present, exiting."
    exit 0
  fi

  iter=$((iter + 1))
  if [ "$MAX_ITER" -gt 0 ] && [ "$iter" -gt "$MAX_ITER" ]; then
    log "reached RALPH_MAX_ITERATIONS=$MAX_ITER, exiting."
    exit 0
  fi

  log "iteration $iter — adapter=$ADAPTER timeout=${ITER_TIMEOUT}s"

  # Bound the iteration. Adapters are responsible for their own internal
  # bookkeeping; the loop just enforces a wall-clock cap.
  if command -v timeout >/dev/null 2>&1; then
    timeout --foreground --kill-after=30s "${ITER_TIMEOUT}s" \
      bash "$ADAPTER_SCRIPT" "$(cat ralph/PROMPT.md)"
    rc=$?
  else
    # macOS without coreutils `timeout` — run un-bounded but warn.
    log "warning: no 'timeout' binary; running adapter without wall-clock cap."
    bash "$ADAPTER_SCRIPT" "$(cat ralph/PROMPT.md)"
    rc=$?
  fi

  log "iteration $iter exited rc=$rc"

  # rc=124 from `timeout` means we hit the wall clock; that's not fatal,
  # the loop continues. The adapter should leave the working tree clean.
  if [ "$rc" -ne 0 ] && [ "$rc" -ne 124 ]; then
    log "adapter returned non-zero rc=$rc; sleeping before retry."
  fi

  sleep "$SLEEP_BETWEEN"
done
