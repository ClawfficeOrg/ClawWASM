#!/usr/bin/env bash
# build-plugin.sh — build clawasm with `with-llama`, ad-hoc sign the dylib,
# and install it into the llm-chat example addon directory.
#
# Usage:
#   bash scripts/build-plugin.sh [--release] [--example <path>]
#
# Options:
#   --release          Build in release mode (default: debug)
#   --example <path>   Path to the Godot project to install into.
#                      Defaults to examples/llm-chat
#
# macOS WARNING:
#   dyld keeps a cdylib mmap'd for the lifetime of any process that loaded it.
#   Overwriting the file while Godot (or any consumer) is running invalidates
#   the on-disk code signature and causes:
#     - the running Godot to crash with SIGKILL / Code Signature Invalid
#     - any `git add` on the file to be SIGKILL'd by the kernel
#   This script checks for running Godot instances and aborts if found.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROFILE="debug"
EXAMPLE_DIR="$REPO_ROOT/examples/llm-chat"

# ── Parse args ────────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release) PROFILE="release"; shift ;;
    --example) EXAMPLE_DIR="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

ADDON_DIR="$EXAMPLE_DIR/addons/clawasm"
DYLIB_NAME="libclawasm.dylib"

# ── Guard: Godot must not be running ─────────────────────────────────────────
if pgrep -x "Godot" &>/dev/null || pgrep -x "godot" &>/dev/null; then
  echo ""
  echo "ERROR: Godot is currently running."
  echo ""
  echo "  dyld keeps $DYLIB_NAME mmap'd for the lifetime of the Godot process."
  echo "  Overwriting the file while Godot is running will:"
  echo "    1. Crash the running Godot with SIGKILL (Code Signature Invalid)"
  echo "    2. Make the file unusable by git until re-signed"
  echo ""
  echo "  Please quit Godot first, then re-run this script."
  echo ""
  exit 1
fi

echo "==> Building clawasm (features: with-llama, profile: $PROFILE)"
cd "$REPO_ROOT"

if [[ "$PROFILE" == "release" ]]; then
  cargo build -p clawasm --features with-llama --release
  BUILT_DYLIB="$REPO_ROOT/target/release/$DYLIB_NAME"
else
  cargo build -p clawasm --features with-llama
  BUILT_DYLIB="$REPO_ROOT/target/debug/$DYLIB_NAME"
fi

# ── Ad-hoc code sign ─────────────────────────────────────────────────────────
# On macOS 12+, dyld enforces code signatures for all loaded dylibs.
# Cargo/rustc produces a valid ad-hoc signature on arm64, but it can become
# stale if the linker re-ran after the signing step, or if the file was
# modified post-build.  Re-signing here guarantees a fresh valid signature
# before the file is copied anywhere.
echo "==> Ad-hoc signing $BUILT_DYLIB"
codesign --sign - --force --timestamp=none "$BUILT_DYLIB"
codesign --verify --strict "$BUILT_DYLIB"
echo "    Signature OK"

# ── Install ───────────────────────────────────────────────────────────────────
echo "==> Installing to $ADDON_DIR/$DYLIB_NAME"
mkdir -p "$ADDON_DIR"
cp "$BUILT_DYLIB" "$ADDON_DIR/$DYLIB_NAME"

# The cp destination also needs its own valid signature (macOS validates the
# file at the destination path when Godot loads it, not the source).
codesign --sign - --force --timestamp=none "$ADDON_DIR/$DYLIB_NAME"
codesign --verify --strict "$ADDON_DIR/$DYLIB_NAME"
echo "    Destination signature OK"

echo ""
echo "==> Done. You can now open Godot and load the project at:"
echo "    $EXAMPLE_DIR"
echo ""
echo "    The CLLawM node uses the freshly built libclawasm.dylib."
