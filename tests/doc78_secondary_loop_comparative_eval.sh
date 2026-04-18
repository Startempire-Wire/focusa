#!/bin/bash
# Controlled-run comparative proof harness for Doc-78 §15.1 secondary-loop quality hooks.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

run_case() {
  local label="$1"
  local logfile="$2"
  shift 2
  if (cd "$ROOT_DIR" && "$@") >"$logfile" 2>&1; then
    log_pass "$label"
  else
    log_fail "$label"
    tail -n 40 "$logfile" || true
  fi
}

run_case \
  "daemon controlled run proves promoted-vs-baseline quality divergence" \
  "/tmp/doc78_secondary_loop_comparative_core.log" \
  cargo test -q -p focusa-core observe_continuous_turn_outcome_comparative_baseline_proves_improvement_for_same_task

run_case \
  "status acceptance hooks expose comparative-improvement pair evidence" \
  "/tmp/doc78_secondary_loop_comparative_api.log" \
  cargo test -q -p focusa-api secondary_loop_acceptance_hooks_surface_controlled_run_proofs

run_case \
  "status acceptance hooks remain fail-closed when comparative evidence absent" \
  "/tmp/doc78_secondary_loop_comparative_api_default.log" \
  cargo test -q -p focusa-api secondary_loop_acceptance_hooks_default_to_false_without_evidence

echo "=== DOC78 SECONDARY LOOP COMPARATIVE EVAL RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then
  exit 1
fi
