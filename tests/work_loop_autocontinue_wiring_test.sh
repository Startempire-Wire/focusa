#!/bin/bash
# SPEC-79 auto-continuation wiring: daemon-owned loop must self-dispatch without operator nudges.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
TURN_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/turn.rs"
SERVER_FILE="${ROOT_DIR}/crates/focusa-api/src/server.rs"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
COMPACTION_FILE="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"

FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'pub async fn maybe_dispatch_continuous_turn_prompt' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "continuous-turn prompt dispatch helper exists"
else
  log_fail "continuous-turn prompt dispatch helper missing"
fi

if rg -n 'WorkLoopStatus::AwaitingHarnessTurn|reprompt_stale_ms' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "stale awaiting-harness turns can be auto-reprompted"
else
  log_fail "awaiting-harness stale re-prompt wiring missing"
fi

if rg -n 'blocked loop state without bound task|auto-advanced from blocked state without bound task|auto-advanced from blocked task' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "blocked loop state can auto-advance to alternate ready work"
else
  log_fail "blocked loop auto-advance wiring missing"
fi

if rg -n 're-selected work after unassigned turn state|maybe_select_global_ready_work_item' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "unassigned loop state can re-select from global ready queue"
else
  log_fail "unassigned loop reselection wiring missing"
fi

if rg -n 'continuous work enabled with ready work selected' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "enable route can seed initial ready-work selection and dispatch"
else
  log_fail "enable route missing initial select+dispatch wiring"
fi

if rg -n 'ready work selected for continuous execution' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "select_next route triggers next-turn dispatch helper"
else
  log_fail "select_next route missing dispatch helper call"
fi

if rg -n 'continuous turn outcome evaluated and ready work remains' "$TURN_FILE" >/dev/null 2>&1; then
  log_pass "turn_complete triggers next-turn dispatch helper after outcome evaluation"
else
  log_fail "turn_complete missing follow-on dispatch helper"
fi

if rg -n 'pi rpc turn_end/agent_end observed and ready work remains|pi rpc compaction observed with ready work pending' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "Pi RPC stream events feed continuation outcome observation and follow-on dispatch"
else
  log_fail "Pi RPC stream -> continuation dispatch wiring missing"
fi

if rg -n 'pi rpc turn ended without assistant output \(compaction/housekeeping\); auto-retrying' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_pass "empty-output turn_end events retry automatically"
else
  log_fail "empty-output retry wiring missing"
fi

if rg -n 'three consecutive pi rpc turn_end/agent_end events without assistant output|consecutive_empty_turns' "$WORK_LOOP_FILE" >/dev/null 2>&1; then
  log_fail "Pi RPC empty-output retry path still appears hard-capped"
else
  log_pass "Pi RPC empty-output retry path no longer has a hard cap marker"
fi

if rg -n 'scheduleCompactionResumeRetry\(ctx, steerMessage, (nextAttempt|retryAttempt \+ 1)\)' "$COMPACTION_FILE" >/dev/null 2>&1 \
  && rg -n 'if \(!S\.compactResumePending\) return;' "$COMPACTION_FILE" >/dev/null 2>&1 \
  && ! rg -n 'maxAttempts' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction auto-resume retries are pending-gated and no longer hard-capped"
else
  log_fail "compaction auto-resume still appears hard-capped or ungated"
fi

if rg -n 'Compaction resume exhausted retries' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_fail "artificial compaction-resume exhaustion warning still present"
else
  log_pass "compaction-resume no longer emits artificial exhaustion warning"
fi

if rg -n 'S\.compactResumePending = false' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" "${ROOT_DIR}/apps/pi-extension/src/session.ts" "${ROOT_DIR}/apps/pi-extension/src/commands.ts" >/dev/null 2>&1; then
  log_pass "compaction retries remain bounded by lifecycle/governance reset gates"
else
  log_fail "compaction retries missing lifecycle reset guards"
fi

if rg -n 'daemon heartbeat supervisor tick' "$SERVER_FILE" >/dev/null 2>&1; then
  log_pass "daemon heartbeat supervisor actively self-dispatches prompts"
else
  log_fail "server supervisor heartbeat dispatch wiring missing"
fi

if rg -n 'no current task bound; select-next required before requesting turn' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "daemon rejects null-task turn requests to prevent fake awaiting-harness stalls"
else
  log_fail "daemon missing null-task guard before turn-start emission"
fi

echo "=== WORK-LOOP AUTO-CONTINUE WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
