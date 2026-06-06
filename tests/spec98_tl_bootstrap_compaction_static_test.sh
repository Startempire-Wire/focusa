#!/bin/bash
# Spec98 / focusa-877z.8: Trajectory Ladder north-star persistence must survive Pi bootstrap and compaction.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
SESSION_FILE="${ROOT_DIR}/apps/pi-extension/src/session.ts"
COMPACTION_FILE="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"

FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'params\.set\("allow_prior_project_trajectory", "true"\)' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "Focus Slice trajectory view allows prior same-project TL fallback"
else
  log_fail "Focus Slice trajectory view missing allow_prior_project_trajectory fallback"
fi

if rg -n 'pi_trajectory_prompt_suppressed_prior_project_fallback' "$SESSION_FILE" >/dev/null 2>&1; then
  log_pass "session bootstrap suppresses HLT prompt when prior TL fallback loaded"
else
  log_fail "session bootstrap lacks prior-TL prompt suppression"
fi

if rg -n 'focusaFetch\("/trajectory/checkpoint"' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction creates a Trajectory checkpoint"
else
  log_fail "compaction missing Trajectory checkpoint"
fi

if rg -n 'focusaFetch\("/trajectory/resume"' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction refreshes a Trajectory resume packet"
else
  log_fail "compaction missing Trajectory resume packet"
fi

if rg -n '## TrajectoryResumePacket' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "post-compaction steer injects TrajectoryResumePacket section"
else
  log_fail "post-compaction steer missing TrajectoryResumePacket section"
fi

if rg -n 'TL is north-star route context; Workpoint remains immediate action authority' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction prompt preserves TL/Workpoint authority split"
else
  log_fail "compaction prompt missing TL/Workpoint authority split"
fi

echo "=== SPEC98 TL BOOTSTRAP/COMPACTION STATIC RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
