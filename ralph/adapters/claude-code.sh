#!/usr/bin/env bash
# ralph/adapters/claude-code.sh — invoke Claude Code (`claude` CLI) for one Ralph iteration.
#
# Receives the prompt as argv[1]. Honors RALPH_ITER_TIMEOUT_SEC via the parent loop.
#
# Requirements:
#   - `claude` CLI installed and authenticated.
#   - `gh` CLI installed and authenticated (the agent calls it to open PRs).
#
# This adapter is intentionally thin: the entire iteration contract lives in
# ralph/PROMPT.md. The adapter just shuttles the prompt to the model and
# lets the model use its filesystem tools to do the work.

set -uo pipefail

PROMPT="${1:-}"
if [ -z "$PROMPT" ]; then
  echo "claude-code adapter: empty prompt" >&2
  exit 2
fi

if ! command -v claude >/dev/null 2>&1; then
  echo "claude-code adapter: 'claude' CLI not found in PATH." >&2
  echo "  install: https://docs.anthropic.com/claude/docs/claude-code" >&2
  exit 127
fi

# --print: non-interactive, single completion.
# --dangerously-skip-permissions is intentionally NOT passed; the loop should
# run in a container or VM where the agent has appropriate access.
exec claude --print "$PROMPT"
