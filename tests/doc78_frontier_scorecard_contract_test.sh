#!/bin/bash
# Contract: Doc78 frontier scorecard must map F1-F5 to concrete evidence routes/tests and focusa-o8vn completion gates.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCORECARD_FILE="${ROOT_DIR}/docs/DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17.md"
FRONTIER_FILE="${ROOT_DIR}/docs/DOC78_REMAINING_IMPLEMENTATION_FRONTIER_2026-04-16.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if [ -f "$SCORECARD_FILE" ]; then
  log_pass "doc78 closure scorecard exists"
else
  log_fail "doc78 closure scorecard missing"
fi

if rg -n 'F1|F2|F3|F4|F5' "$SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "scorecard enumerates all frontier slices"
else
  log_fail "scorecard missing one or more frontier slices"
fi

if rg -n '/v1/work-loop/replay/closure-evidence|/v1/work-loop/replay/closure-bundle' "$SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "scorecard anchors replay consumer + closure bundle routes"
else
  log_fail "scorecard missing replay consumer route anchors"
fi

if rg -n 'tests/doc78_secondary_cognition_runtime_test.sh|tests/doc78_tui_replay_dashboard_surface_test.sh|tests/work_loop_route_contract_test.sh|tests/doc78_live_runtime_closure_bundle_smoke.sh|tests/doc78_live_continuation_boundary_pressure_smoke.sh|tests/doc78_live_non_closure_objective_profile_smoke.sh|tests/doc78_production_runtime_governance_replay_smoke.sh|tests/doc78_production_runtime_series_smoke.sh|tests/doc73_first_consumer_path_test.sh|tests/work_loop_commitment_lifecycle_contract_test.sh|tests/doc74_reference_resolution_consumer_path_test.sh|tests/doc76_retention_policy_consumer_path_test.sh|tests/work_loop_query_scope_boundary_contract_test.sh' "$SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "scorecard references executable proof harnesses"
else
  log_fail "scorecard missing executable proof harness references"
fi

if rg -n 'FOCUSA_DOC78_PROD_ARTIFACT_DIR|docs/evidence/doc78-production-runtime-latest|FOCUSA_DOC78_PROD_SERIES_DIR|docs/evidence/doc78-production-runtime-series-latest' "$SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "scorecard production verification commands include single-run and sustained-run artifact aliases"
else
  log_fail "scorecard missing artifact capture aliases in production verification commands"
fi

if rg -n 'docs/evidence/DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17.md|docs/evidence/DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md' "$SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "scorecard anchors baseline and sustained production evidence documents"
else
  log_fail "scorecard missing baseline/sustained production evidence document references"
fi

if rg -n 'focusa-o8vn|completion criteria' "$SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "scorecard defines focusa-o8vn completion criteria"
else
  log_fail "scorecard missing focusa-o8vn completion criteria"
fi

if rg -n 'DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17.md|DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17.md|DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md' "$FRONTIER_FILE" >/dev/null 2>&1; then
  log_pass "frontier doc links to scorecard mapping plus baseline/sustained production evidence"
else
  log_fail "frontier doc missing scorecard/evidence linkage"
fi

echo "=== DOC78 FRONTIER SCORECARD CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
