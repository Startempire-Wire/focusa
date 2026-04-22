#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

npx tsx "$ROOT_DIR/tests/spec81_pi_extension_runtime_contract.ts"
echo "✓ PASS: SPEC81 extension runtime contract"

FOCUSA_API_URL="${FOCUSA_API_URL:-http://127.0.0.1:8787/v1}" \
  npx tsx "$ROOT_DIR/tests/spec81_live_chain_extension_runtime_test.ts"
echo "✓ PASS: SPEC81 extension live chain"
