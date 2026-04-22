#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

npx tsx "$ROOT_DIR/tests/spec87_extension_desirability_contract.ts"
echo "✓ PASS: SPEC87 extension desirability contract"

bash "$ROOT_DIR/tests/spec87_tool_pickup_and_effectiveness_smoke_test.sh"
echo "✓ PASS: SPEC87 pickup and effectiveness smoke"
