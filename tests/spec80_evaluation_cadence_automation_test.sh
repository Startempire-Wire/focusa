#!/bin/bash
# Contract: Spec80 E2.2 cadence automation must encode Appendix C daily/weekly/14-day scheduling and deterministic reporting semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_EVALUATION_CADENCE_AUTOMATION_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'daily internal snapshot|weekly operator report|Gate D decision every 14 days' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design captures required Appendix C reporting cadence"
else
  log_fail "design missing required Appendix C reporting cadence"
fi

if rg -n 'daily_snapshot_job|weekly_report_job|gate_decision_job' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines explicit scheduler jobs"
else
  log_fail "design missing explicit scheduler jobs"
fi

if rg -n 'run_id|report_id|decision_id|pass_count|critical_regression|final_decision' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines report output contracts for each cadence layer"
else
  log_fail "design missing report output contract fields"
fi

if rg -n 'upstream_data_missing|insufficient_sample|backfill_required|must not emit partial pass/fail decisions' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines typed failure and recovery semantics"
else
  log_fail "design missing typed failure and recovery semantics"
fi

if rg -n 'daily -> weekly -> 14-day gate decision|UTC window boundaries|deterministic ids' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines determinism and sequencing constraints"
else
  log_fail "design missing determinism and sequencing constraints"
fi

echo "=== SPEC80 EVALUATION CADENCE AUTOMATION RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
