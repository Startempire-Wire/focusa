#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if npx tsx "$ROOT_DIR/tests/spec80_pi_extension_runtime_contract.ts"; then
  echo "✓ PASS: SPEC80 pi-extension runtime contract"
else
  echo "✗ FAIL: SPEC80 pi-extension runtime contract" >&2
  exit 1
fi
