#!/bin/bash
# Contract: Spec80 D2.2 snapshot lifecycle plan must define idempotency + consistency requirements and typed errors.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_SNAPSHOT_IDEMPOTENCY_CONSISTENCY_PLAN_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '/v1/focus/snapshots|/v1/focus/snapshots/restore|/v1/focus/snapshots/diff' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan covers full snapshot lifecycle endpoint surface"
else
  log_fail "plan missing one or more snapshot lifecycle endpoints"
fi

if rg -n 'Create idempotency|Restore idempotency|Diff idempotency' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines core idempotency requirements"
else
  log_fail "plan missing idempotency requirements"
fi

if rg -n 'Checksum consistency|Reference consistency|Temporal consistency|Conflict consistency' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines consistency requirements"
else
  log_fail "plan missing consistency requirements"
fi

for code in SNAPSHOT_NOT_FOUND SNAPSHOT_CONFLICT RESTORE_CONFLICT DIFF_INPUT_INVALID; do
  if rg -n "$code" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "plan includes typed error: $code"
  else
    log_fail "plan missing typed error: $code"
  fi
done

if rg -n '\| I1 \||\| I2 \||\| I3 \||\| C1 \||\| C2 \|' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines explicit test lanes (I1-I3/C1-C2)"
else
  log_fail "plan missing explicit test lanes"
fi

echo "=== SPEC80 SNAPSHOT IDEMPOTENCY/CONSISTENCY PLAN RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
