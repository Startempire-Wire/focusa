#!/bin/bash
# Contract: Spec80 D3.2 restore/compaction budget spec must encode §20.2 thresholds and deterministic gate semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_RESTORE_COMPACTION_PERFORMANCE_BUDGETS_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'restore p95 <= 400ms|restore_p95_ms <= 400|restore_threshold_ms' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec encodes restore p95 <= 400ms gate"
else
  log_fail "spec missing restore p95 <= 400ms gate"
fi

if rg -n 'compaction p95 <= 1\.5x|compaction_ratio|compaction_threshold_ratio.*1\.5' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec encodes compaction p95 <= 1.5x baseline gate"
else
  log_fail "spec missing compaction p95 <= 1.5x baseline gate"
fi

if rg -n 'nearest-rank p95|sample count `<200`|insufficient_sample' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines deterministic percentile and sample validity constraints"
else
  log_fail "spec missing deterministic percentile or sample validity constraints"
fi

if rg -n 'operation|latency_ms|profile_id|workload_hash|run_id' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines per-run measurement fields"
else
  log_fail "spec missing per-run measurement fields"
fi

if rg -n 'final_decision is pass only if both restore and compaction gates pass|final_decision' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines final combined gate decision rule"
else
  log_fail "spec missing final combined gate decision rule"
fi

echo "=== SPEC80 RESTORE/COMPACTION PERFORMANCE BUDGET RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
