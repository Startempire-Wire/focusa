#!/bin/bash
# Consumer-path contract: doc 78 replay comparative evidence must drive a live continuity consumer.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/FIRST_CONSUMER_CANDIDATES_2026-04-13.md"
WATCHDOG_FILE="${ROOT_DIR}/scripts/work_loop_watchdog.sh"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '### Doc 78 — bounded secondary cognition / persistent autonomy' "$DOC_FILE" >/dev/null 2>&1 && rg -n 'Selected first real consumer' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 78 section names a selected first consumer"
else
  log_fail "doc 78 section missing selected first consumer"
fi

if rg -n 'scripts/work_loop_watchdog\.sh|/v1/work-loop/replay/closure-bundle|/v1/work-loop/replay/closure-evidence' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 78 section anchors consumer to watchdog + replay bundle/evidence routes"
else
  log_fail "doc 78 section missing watchdog/replay-route anchors"
fi

if rg -n '/v1/work-loop/replay/closure-bundle|/v1/work-loop/replay/closure-evidence|closure_bundle_json|closure_replay_status|closure_pair_observed|continuity_gate_state|closure_replay_ready|operator_steering_detected|governance_decision_pending|continuation_boundary_active' "$WATCHDOG_FILE" >/dev/null 2>&1; then
  log_pass "watchdog consumes replay bundle/evidence and continuation-boundary status fields"
else
  log_fail "watchdog does not consume replay/boundary payload surfaces"
fi

if rg -n 'released_on_blocker" && "\$closure_replay_ready" == "true" && "\$continuation_boundary_active" == "false"' "$WATCHDOG_FILE" >/dev/null 2>&1 && rg -n 'released_on_completion|released_or_unbound' "$WATCHDOG_FILE" >/dev/null 2>&1 && rg -n '"\$continuation_boundary_active" == "false"' "$WATCHDOG_FILE" >/dev/null 2>&1; then
  log_pass "watchdog continuity handoff is fail-closed on replay evidence and continuation-boundary readiness"
else
  log_fail "watchdog continuity handoff missing replay/boundary fail-closed gate"
fi

if rg -n 'fn secondary_loop_replay_consumer_payload_for_status\(' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'async fn closure_replay_evidence|async fn closure_replay_bundle' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "replay consumer payload source exists in work-loop route"
else
  log_fail "replay consumer payload source missing in work-loop route"
fi

echo "=== DOC 78 FIRST CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
