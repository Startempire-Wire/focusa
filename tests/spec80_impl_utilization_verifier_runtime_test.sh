#!/bin/bash
# Contract: Spec80 §20.3 full utilization verifier must be executable and output blocking criteria when needed.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
VERIFIER="${ROOT_DIR}/scripts/spec80/utilization_verifier.py"
TMP_DIR="$(mktemp -d /tmp/spec80-util.XXXXXX)"
trap 'rm -rf "$TMP_DIR"' EXIT
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if [ -x "$VERIFIER" ]; then
  log_pass "utilization verifier script exists and is executable"
else
  log_fail "utilization verifier missing or not executable"
fi

touch "$TMP_DIR/tree.json" "$TMP_DIR/meta.json" "$TMP_DIR/parity.json" "$TMP_DIR/gov.json"
cat > "$TMP_DIR/gate_d.json" <<'JSON'
{"final_decision":"pass"}
JSON

python3 "$VERIFIER" \
  --tree-dossier "$TMP_DIR/tree.json" \
  --metacog-dossier "$TMP_DIR/meta.json" \
  --parity-dossier "$TMP_DIR/parity.json" \
  --outcome-report "$TMP_DIR/gate_d.json" \
  --governance-dossier "$TMP_DIR/gov.json" \
  --output "$TMP_DIR/out.json" >/dev/null

if jq -e '.verifier_id=="spec80_full_utilization_v1" and .all_pass==true and (.blocking_criteria|length)==0 and (.criteria|length)==5' "$TMP_DIR/out.json" >/dev/null 2>&1; then
  log_pass "utilization verifier returns all-pass when all dossiers + Gate D pass are present"
else
  log_fail "utilization verifier output invalid"
fi

echo "=== SPEC80 IMPL UTILIZATION VERIFIER RUNTIME RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
