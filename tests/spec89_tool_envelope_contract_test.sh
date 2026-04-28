#!/usr/bin/env bash
# Spec89 Phase 0 skeleton: validates FocusaToolResult v1 required fields and current focusa_* tool enumeration.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_TS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
SCHEMA="$ROOT_DIR/docs/contracts/focusa-tool-result-schema-v1.json"
SAMPLE="$ROOT_DIR/tests/fixtures/spec89_tool_result_valid_sample.json"
required=(ok status canonical degraded summary retry side_effects evidence_refs next_tools)
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
[[ -f "$SCHEMA" ]] || fail "schema missing: $SCHEMA"
jq empty "$SCHEMA" >/dev/null || fail "schema is not valid JSON"
tool_count=$(rg -n 'name: "focusa_' "$TOOLS_TS" | wc -l | tr -d ' ')
[[ "$tool_count" == "39" ]] || fail "expected 39 focusa_* tools, got $tool_count"
rg 'interface FocusaToolResultV1' "$TOOLS_TS" >/dev/null || fail "FocusaToolResultV1 helper missing"
rg 'function withToolResultEnvelope' "$TOOLS_TS" >/dev/null || fail "withToolResultEnvelope wrapper missing"
rg 'tool_result_v1' "$TOOLS_TS" >/dev/null || fail "tool_result_v1 details extension missing"
validate_result(){
  local file="$1"
  jq empty "$file" >/dev/null || fail "$file invalid JSON"
  for field in "${required[@]}"; do
    jq -e "has(\"$field\")" "$file" >/dev/null || fail "$file missing required field $field"
  done
  jq -e '.retry | has("safe") and has("posture")' "$file" >/dev/null || fail "$file retry missing safe/posture"
  jq -e '(.side_effects|type)=="array" and (.evidence_refs|type)=="array" and (.next_tools|type)=="array"' "$file" >/dev/null || fail "$file array fields invalid"
}
if [[ "$#" -gt 0 ]]; then
  for file in "$@"; do validate_result "$file"; done
else
  validate_result "$SAMPLE"
fi
pass "Spec89 tool envelope skeleton validated required fields and $tool_count tools"
