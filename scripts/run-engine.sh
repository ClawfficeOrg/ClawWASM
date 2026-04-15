#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Build with feature if requested
FEATURE_FLAG=""
if [ "${1-}" = "with-wasmedge" ]; then
  FEATURE_FLAG="--features with-wasmedge"
fi

cargo check -p clawasm-engine $FEATURE_FLAG || {
  echo "cargo check failed. If failures mention wasmedge native lib, install WasmEdge 0.16.1 (see README)."
  exit 1
}

# Run example using the engine crate (stubbed if feature not enabled)
# Note: we didn't add an example binary; this script demonstrates the intended flow.
echo "cargo check passed for clawasm-engine (feature:${FEATURE_FLAG})"
