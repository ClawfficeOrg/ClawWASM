#!/usr/bin/env bash
# ralph/adapters/codex.sh — invoke OpenAI Codex CLI for one Ralph iteration.
#
# Requirements:
#   - `codex` CLI installed and authenticated (`npm i -g @openai/codex`).
#   - `gh` CLI installed and authenticated.

set -uo pipefail

PROMPT="${1:-}"
if [ -z "$PROMPT" ]; then
  echo "codex adapter: empty prompt" >&2
  exit 2
fi

if ! command -v codex >/dev/null 2>&1; then
  echo "codex adapter: 'codex' CLI not found in PATH." >&2
  exit 127
fi

# --quiet: print only the assistant's final message + tool output.
exec codex --quiet "$PROMPT"
