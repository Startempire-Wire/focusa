#!/bin/bash
# Contract: Spec80 Gate D must be executable with >=4/6 rule, sample/form floors, and deterministic final decision.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUNNER="${ROOT_DIR}/scripts/spec80/gate_d_report.py"
TMP_DIR="$(mktemp -d /tmp/spec80-gated.XXXXXX)"
trap 'rm -rf "$TMP_DIR"' EXIT
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if [ -x "$RUNNER" ]; then
  log_pass "gate D runner exists and is executable"
else
  log_fail "gate D runner missing or not executable"
fi

cat > "$TMP_DIR/input.json" <<'JSON'
{
  "report_id": "gate-d-runtime-1",
  "forms": {"valid": 55, "novel_context": 25},
  "contracts": [
    {"contract_id":"failed_turn_ratio","status":"pass","relative_delta":-0.10,"sample_size_ok":true},
    {"contract_id":"execution_success_rate","status":"pass","relative_delta":0.08,"sample_size_ok":true},
    {"contract_id":"rework_loop_rate","status":"pass","relative_delta":0.12,"sample_size_ok":true},
    {"contract_id":"tool_call_precision","status":"pass","relative_delta":0.05,"sample_size_ok":true},
    {"contract_id":"state_alignment_score","status":"fail","relative_delta":-0.01,"sample_size_ok":true},
    {"contract_id":"operator_override_rate","status":"fail","relative_delta":-0.02,"sample_size_ok":true}
  ]
}
JSON

python3 "$RUNNER" --input "$TMP_DIR/input.json" --output "$TMP_DIR/out.json" >/dev/null
if jq -e '.gate_id=="Gate D" and .pass_count >= 4 and .final_decision=="pass" and .forms.meets_floor==true' "$TMP_DIR/out.json" >/dev/null 2>&1; then
  log_pass "gate D runner enforces >=4/6 and emits pass decision"
else
  log_fail "gate D runner output invalid"
fi

echo "=== SPEC80 IMPL GATE D RUNNER RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
