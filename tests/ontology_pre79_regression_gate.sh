#!/bin/bash
# Pre-79 ontology regression gate (docs 45-77)
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
  tests/ontology_world_contract_test.sh
  tests/ontology_event_contract_test.sh
  tests/ontology_visual_object_frontier_contract_test.sh
  tests/ontology_visual_reverse_extraction_pipeline_contract_test.sh
  tests/ontology_visual_implementation_handoff_contract_test.sh
  tests/ontology_route_metadata_contract_test.sh
  tests/ontology_affordance_schema_contract_test.sh
  tests/work_loop_query_scope_boundary_contract_test.sh
  tests/scope_failure_taxonomy_events_contract_test.sh
  tests/work_loop_governing_priors_consumer_path_test.sh
  tests/doc73_first_consumer_path_test.sh
  tests/doc74_reference_resolution_consumer_path_test.sh
  tests/doc76_retention_policy_consumer_path_test.sh
  tests/work_loop_migration_conformance_checks_test.sh
  tests/doc70_shared_interfaces_lifecycle_contract_test.sh
  tests/proxy_mode_b_parity_test.sh
)

echo "=== PRE-79 ONTOLOGY REGRESSION GATE ==="
for t in "${TESTS[@]}"; do
  if bash "${ROOT_DIR}/${t}" >/dev/null 2>&1; then
    log_pass "$t"
  else
    log_fail "$t"
  fi
done

echo "=== RESULTS ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
