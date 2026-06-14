#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUMMARY="$ROOT_DIR/docs/current/generated/tool-surface-summary.md"
CONTRACTS="$ROOT_DIR/docs/current/focusa-tool-contracts.json"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

cd "$ROOT_DIR"

scripts/generate-tool-surface-summary --check >/dev/null || fail "generated tool-surface summary is stale"
pass "generated tool-surface summary matches contract registry"

expected_tools="$(jq 'if type=="array" then length else ((.tools // .contracts // []) | length) end' "$CONTRACTS")"
summary_tools="$(awk -F'|' '/Tool contracts/ {gsub(/ /,"",$3); print $3}' "$SUMMARY")"
[[ "$summary_tools" == "$expected_tools" ]] || fail "summary tool count $summary_tools != registry $expected_tools"
pass "summary tool count equals registry count ($expected_tools)"

for file in README.md BENEFITS.md docs/current/CURRENT_RUNTIME_STATUS.md docs/focusa-tools/README.md; do
  rg -n '\b(59|63|64|65|79)\s+(current\s+)?(focusa_\*\s+)?(Pi\s+)?tools?\b|all\s+(59|63|64|65|79)\s+tools?|\b(59|63|64|65|79)\s+tool contracts?\b' "$file" \
    && fail "$file contains a hardcoded tool count; link to docs/current/generated/tool-surface-summary.md instead"
  rg -n 'docs/current/generated/tool-surface-summary.md|generated/tool-surface-summary.md' "$file" >/dev/null \
    || fail "$file must reference generated tool-surface summary"
done
pass "primary docs avoid hardcoded tool counts and reference generated summary"

echo "tool surface summary static test: PASS"
