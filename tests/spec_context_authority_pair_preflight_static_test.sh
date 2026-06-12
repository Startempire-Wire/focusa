#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
FILE="crates/focusa-cli/src/commands/pair.rs"

rg -q 'environment_contract_summary' "$FILE"
rg -q 'runtime::collect_inventory' "$FILE"
rg -q 'action::evaluate_preflight' "$FILE"
rg -q '"environment_contract"' "$FILE"
rg -q '"runtime_inventory"' "$FILE"
rg -q '"action_preflight"' "$FILE"
rg -q '"initiate Phone Bridge pairing"' "$FILE"
rg -q '"pairing_start"' "$FILE"

echo "pair preflight static test passed"
