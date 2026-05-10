#!/usr/bin/env bash
# download-model.sh — Download the Gemma 4 E2B-IT Q4_K_M GGUF from
# bartowski's HuggingFace repo into ./models/.
#
# Usage:
#   bash scripts/download-model.sh [QUANT]
#
# QUANT defaults to Q4_K_M (3.46 GB, good quality / size trade-off).
# Other options from bartowski/google_gemma-4-E2B-it-GGUF:
#   Q4_K_S  (3.38 GB) — slightly smaller, still good
#   Q5_K_M  (3.66 GB) — higher quality
#   Q6_K    (3.90 GB) — near-perfect quality
#   Q8_0    (4.97 GB) — maximum quality
#   IQ4_XS  (3.32 GB) — smallest reasonable quality
#
# Requirements: hf  (https://huggingface.co/docs/huggingface_hub/package_reference/cli)
#   Install: pip install -U "huggingface_hub[cli]"
#   The modern CLI binary is `hf` (replaces the deprecated `huggingface-cli`).

set -euo pipefail

# Strip a trailing .gguf suffix so both "Q4_K_M" and "Q4_K_M.gguf" work.
QUANT="${1:-Q4_K_M}"
QUANT="${QUANT%.gguf}"
REPO="bartowski/google_gemma-4-E2B-it-GGUF"
FILENAME="google_gemma-4-E2B-it-${QUANT}.gguf"
DEST_DIR="$(dirname "$0")/../models"

mkdir -p "$DEST_DIR"

echo "==> Downloading ${FILENAME} from ${REPO}"
echo "    Destination: ${DEST_DIR}/${FILENAME}"
echo ""

if command -v hf &>/dev/null; then
  HF_CMD="hf"
elif command -v huggingface-cli &>/dev/null; then
  HF_CMD="huggingface-cli"
else
  echo "ERROR: 'hf' not found."
  echo "       Install it with: pip install -U \"huggingface_hub[cli]\""
  echo "       Then run: hf auth login"
  exit 1
fi

"$HF_CMD" download \
  "$REPO" \
  "$FILENAME" \
  --local-dir "$DEST_DIR"

echo ""
echo "==> Done. Model saved to: ${DEST_DIR}/${FILENAME}"
echo ""
echo "In Godot (GDScript):"
echo "  llm.set_model(\"res://models/${FILENAME}\")"
echo ""
echo "Or set the model path via GDScript after loading the extension."
