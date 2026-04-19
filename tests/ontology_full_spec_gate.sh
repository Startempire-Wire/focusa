#!/bin/bash
# Full ontology-spec gate (pre-78 + post-78 contract surfaces)
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

TESTS=(
  tests/ontology_pre79_regression_gate.sh
  tests/doc72_identity_role_self_model_contract_test.sh
  tests/doc75_projection_view_semantics_contract_test.sh
  tests/work_loop_affordance_environment_surface_test.sh
  tests/doc78_first_consumer_path_test.sh
  tests/doc78_frontier_scorecard_contract_test.sh
  tests/doc78_remaining_frontier_contract_test.sh
  tests/doc78_secondary_cognition_runtime_test.sh
  tests/doc78_tui_replay_dashboard_surface_test.sh
)

echo "=== ONTOLOGY FULL SPEC GATE ==="
for t in "${TESTS[@]}"; do
  if bash "${ROOT_DIR}/${t}" >/dev/null 2>&1; then
    log_pass "$t"
  else
    log_fail "$t"
  fi
done

echo "=== RESULTS ==="
echo "Passed: ${PASSED}"
echo "Failed: ${FAILED}"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
