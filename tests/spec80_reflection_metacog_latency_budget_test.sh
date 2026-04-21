#!/bin/bash
# Contract: Spec80 D3.1 latency budget spec must define p95 <=12% gate and deterministic benchmark semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_REFLECTION_METACOG_LATENCY_BUDGET_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'p95 added latency <= 12%|<=12%|threshold_ratio.*0\.12' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec encodes the <=12% p95 added-latency gate"
else
  log_fail "spec missing <=12% p95 added-latency gate"
fi

if rg -n 'added latency ratio|\(p95_measured - p95_baseline\) / p95_baseline' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines added-latency ratio formula"
else
  log_fail "spec missing added-latency ratio formula"
fi

if rg -n 'minimum 200 requests|sample_count < 200|insufficient_sample' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines sample-size validity semantics"
else
  log_fail "spec missing sample-size validity semantics"
fi

if rg -n 'nearest-rank p95|Percentile method must be fixed' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec fixes percentile methodology for determinism"
else
  log_fail "spec missing fixed percentile methodology"
fi

if rg -n 'gate_id|p95_baseline_ms|p95_with_metacog_ms|added_latency_ratio|decision' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines benchmark result output contract"
else
  log_fail "spec missing benchmark result output contract"
fi

echo "=== SPEC80 REFLECTION/METACOG LATENCY BUDGET RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
