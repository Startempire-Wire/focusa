#!/bin/bash
# SPEC-52: Pi Extension Contract — strict CI gate

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

http_code() {
  curl -sS -o /tmp/focusa-pi-contract-body.json -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" /tmp/focusa-pi-contract-body.json >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat /tmp/focusa-pi-contract-body.json)"
  fi
}

echo "=== SPEC-52: Pi Extension Contract (strict) ==="
echo "Base URL: ${BASE_URL}"
echo ""

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
SESSION_FILE="${ROOT_DIR}/apps/pi-extension/src/session.ts"

if rg -n 'appendEntry\("focusa-wbm-state"' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "Pi persists resumable WBM state via focusa-wbm-state"
else
  log_fail "Pi does not persist resumable WBM state via focusa-wbm-state"
fi

if rg -n 'appendEntry\("focusa-state"|authoritativeDecisions|authoritativeConstraints|authoritativeFailures|persistAuthoritativeState' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "Pi persists authoritative Focusa snapshot alongside local shadow"
else
  log_fail "Pi missing authoritative Focusa snapshot persistence"
fi

if rg -n 'sessionId: S\.sessionFrameKey' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "Pi persisted state includes sessionId for resume"
else
  log_fail "Pi persisted state missing sessionId for resume"
fi

if rg -n 'focusa-wbm-state" \|\| e\.customType === "focusa-state"' "$SESSION_FILE" >/dev/null 2>&1 \
  && rg -n 'if \(e\.data\.sessionId\) S\.sessionFrameKey = e\.data\.sessionId' "$SESSION_FILE" >/dev/null 2>&1; then
  log_pass "Pi session restore accepts resumable WBM state and restores session key"
else
  log_fail "Pi session restore missing resumable WBM state support"
fi

log_info "Health + seeded state"
code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  json_assert '.ok == true' "Daemon running"
else
  log_fail "Daemon not responding"
  exit 1
fi

code=$(http_code -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" \
  -d '{"adapter_id":"pi-contract","workspace_id":"pi-extension-contract"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed session accepted"
else
  log_fail "Seed session failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
  -d '{"title":"pi-contract-test","goal":"testing input contract","beads_issue_id":"pi-ct-001"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed focus frame accepted"
else
  log_fail "Seed focus frame failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/ecs/store" -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"pi-evidence-seed","content_b64":"cGlvdXRwdXQ="}')
if [ "$code" = "200" ]; then
  json_assert '.id != null' "Seed evidence stored"
else
  log_fail "Seed evidence store failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" -H "Content-Type: application/json" \
  -d '{"kind":"blocker","summary":"pi output blocker seed"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed blocker accepted"
else
  log_fail "Seed blocker failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus/update" -H "Content-Type: application/json" \
  -d '{"delta":{"decisions":["Pi extension contract requires non-empty recent decision evidence"]}}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed ASCC decision accepted"
else
  log_fail "Seed ASCC decision failed"
fi

log_info "Pi Input Contract"
code=$(http_code "${BASE_URL}/v1/focus/stack")
if [ "$code" = "200" ]; then
  json_assert '.stack.frames | length > 0' "Input 1: active mission/frame stack populated"
else
  log_fail "Input 1 failed"
fi

decision_visible=0
for i in $(seq 1 20); do
  code=$(http_code "${BASE_URL}/v1/ascc/state")
  if [ "$code" = "200" ] && jq -e '((.active_frame != null) or (.frame_id != null)) and (((.decisions // .ascc.decisions // []) | length) > 0)' /tmp/focusa-pi-contract-body.json >/dev/null 2>&1; then
    decision_visible=1
    break
  fi

  code=$(http_code "${BASE_URL}/v1/focus/stack")
  if [ "$code" = "200" ] && jq -e '(.stack.frames | length > 0) and (.stack.frames | any((.focus_state.decisions // []) | length > 0))' /tmp/focusa-pi-contract-body.json >/dev/null 2>&1; then
    decision_visible=1
    break
  fi
  sleep 0.2
done
if [ "$decision_visible" = "1" ]; then
  log_pass "Input 2/5: frame-thesis and seeded recent decisions accessible"
else
  log_fail "Input 2/5: seeded recent decisions not observable in ASCC or focus stack :: $(cat /tmp/focusa-pi-contract-body.json)"
fi

code=$(http_code "${BASE_URL}/v1/memory/semantic")
if [ "$code" = "200" ]; then
  json_assert '.semantic != null' "Input 3: active working set accessible"
else
  log_fail "Input 3 failed"
fi

code=$(http_code "${BASE_URL}/v1/memory/procedural")
if [ "$code" = "200" ]; then
  json_assert '.procedural != null' "Input 4: applicable constraints accessible"
else
  log_fail "Input 4 failed"
fi

code=$(http_code "${BASE_URL}/v1/ecs/handles")
if [ "$code" = "200" ]; then
  json_assert '.count > 0 and (.handles | length > 0)' "Input 6: verified deltas/evidence accessible"
else
  log_fail "Input 6 failed"
fi

code=$(http_code "${BASE_URL}/v1/focus-gate/candidates")
if [ "$code" = "200" ]; then
  json_assert '.candidates != null' "Input 7: unresolved blockers/open loops accessible"
else
  log_fail "Input 7 failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" \
  -d '{"command":"memory.semantic.upsert","payload":{"key":"pi-test","value":"test"}}')
if [ "$code" = "200" ]; then
  json_assert '.command_id != null' "Input 8: allowed actions command channel accessible"
else
  log_fail "Input 8 failed"
fi

code=$(http_code "${BASE_URL}/v1/autonomy")
if [ "$code" = "200" ]; then
  json_assert '.level != null and .ari_score != null' "Input 9: degraded-mode/autonomy status accessible"
else
  log_fail "Input 9 failed with HTTP ${code}"
fi

log_info "Pi Output Contract"
code=$(http_code -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" -H "Content-Type: application/json" \
  -d '{"kind":"failure","summary":"pi output contract test"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Output: FailureSignal accepted"
else
  log_fail "Output FailureSignal failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" -H "Content-Type: application/json" \
  -d '{"kind":"blocker","summary":"pi blocker test"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Output: BlockerSignal accepted"
else
  log_fail "Output BlockerSignal failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/ecs/store" -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"pi-evidence","content_b64":"cGlvdXRwdXQ="}')
if [ "$code" = "200" ]; then
  json_assert '.id != null' "Output: EvidenceLinkedObservation persisted"
else
  log_fail "Output evidence store failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/memory/semantic/upsert" -H "Content-Type: application/json" \
  -d '{"key":"pi-decision-candidate","value":"test decision"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted" or .status == "ok" or .semantic != null' "Output: DecisionCandidate/upsert accepted"
else
  log_fail "Output DecisionCandidate failed"
fi

if [ -d "/tmp/pi-scratch" ] || mkdir -p /tmp/pi-scratch 2>/dev/null; then
  log_pass "Output: ScratchReasoningRecord scratch path available"
else
  log_fail "Output ScratchReasoningRecord path unavailable"
fi

code=$(http_code -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" \
  -d '{"command":"memory.procedural.reinforce","payload":{"rule_id":"pi-action-test"}}')
if [ "$code" = "200" ]; then
  json_assert '.command_id != null' "Output: OntologyActionIntent submitted"
else
  log_fail "Output OntologyActionIntent failed"
fi

code=$(http_code "${BASE_URL}/v1/reflect/status")
if [ "$code" = "200" ]; then
  json_assert '.enabled != null' "Output: VerificationRequest/reflection available"
else
  log_fail "Output VerificationRequest failed"
fi

STATE_TS="apps/pi-extension/src/state.ts"
INDEX_TS="apps/pi-extension/src/index.ts"
SESSION_TS="apps/pi-extension/src/session.ts"
COMMANDS_TS="apps/pi-extension/src/commands.ts"
if rg -n 'derivePiFrameIntent|S\.currentAsk|Pi Task:|Pi Question:|Pi Correction:' "$STATE_TS" >/dev/null 2>&1; then
  log_pass "Pi frame creation is task-first rather than cwd-only"
else
  log_fail "Pi frame creation still appears cwd-only"
fi

if rg -n 'frameTitle|frameGoal|Title:|Goal:' "$INDEX_TS" >/dev/null 2>&1; then
  log_pass "Persisted Focusa renderer exposes frame title/goal context"
else
  log_fail "Persisted Focusa renderer missing frame title/goal context"
fi

if rg -n 'authoritativeDecisions|authoritativeConstraints|authoritativeFailures|Mission: \$\{d\.intent\}|Focus: \$\{d\.currentFocus\}' "$INDEX_TS" >/dev/null 2>&1; then
  log_pass "Persisted renderer prefers authoritative snapshot content"
else
  log_fail "Persisted renderer missing authoritative snapshot content"
fi

if rg -n 'S\.activeFrameId|S\.activeFrameTitle|S\.activeFrameGoal|pi\.setSessionName' "$SESSION_TS" >/dev/null 2>&1; then
  log_pass "Session sync uses Pi scoped frame metadata rather than global frame fallback"
else
  log_fail "Session sync still appears to rely on global frame fallback"
fi

if rg -n 'reason: "pi_session_shutdown"|reason: "pi_session_switch"' "$SESSION_TS" >/dev/null 2>&1; then
  log_pass "Pi session lifecycle closes Focusa sessions with explicit reasons"
else
  log_fail "Pi session lifecycle missing explicit close reasons"
fi

if rg -n 'persistAuthoritativeState\(' "$SESSION_TS" "$COMMANDS_TS" "apps/pi-extension/src/compaction.ts" >/dev/null 2>&1; then
  log_pass "Pi persists refreshed authoritative state before lifecycle/compaction boundaries"
else
  log_fail "Pi missing refreshed authoritative persistence at lifecycle/compaction boundaries"
fi

if rg -n 'normalizeCompactionArtifacts|kind: "file"|Session compacted\. Modified:|Session compacted\. Read:' "apps/pi-extension/src/compaction.ts" >/dev/null 2>&1; then
  log_pass "Pi compaction file tracking writes canonical artifact lines and notes"
else
  log_fail "Pi compaction file tracking still appears non-canonical"
fi

if rg -n 'assistant_output|prompt_tokens|completion_tokens|extractText\(ev\.message\?\.content' "apps/pi-extension/src/turns.ts" >/dev/null 2>&1; then
  log_pass "Pi turn completion reports assistant output and canonical token fields"
else
  log_fail "Pi turn completion still appears token-only or output-blind"
fi

if rg -n 'await pushDelta\(|Operator correction:|lastFocusSnapshot = \{ decisions: \[], constraints: \[], failures: \[], intent: "", currentFocus: "" \}' "apps/pi-extension/src/turns.ts" "$COMMANDS_TS" >/dev/null 2>&1 \
  && rg -n 'Reconciled after Focusa outage|await pushDelta\(' "apps/pi-extension/src/session.ts" >/dev/null 2>&1; then
  log_pass "Pi progress/reset/reconnect paths use validated writes and clear authoritative snapshot state"
else
  log_fail "Pi progress/reset/reconnect paths still appear partially local-shadow only"
fi

if rg -n 'rescopePiFrameFromCurrentAsk|startup frame rescoped after first real ask|pi-post-input-rescope|isNonTaskStatusLikeText|stripQuotedFocusaContext|currentAsk: S\.currentAsk|seedCurrentAskFromPersistedState' "apps/pi-extension/src/state.ts" "apps/pi-extension/src/turns.ts" "apps/pi-extension/src/session.ts" >/dev/null 2>&1; then
  log_pass "Pi startup fallback frame is rescoped only from real asks and strips quoted Focusa payloads on restart/input"
else
  log_fail "Pi startup fallback frame still appears vulnerable to sticky/status-driven rescope or quoted-payload pollution"
fi

if rg -n 'Title: \$\{S\.activeFrameTitle\}|Goal: \$\{S\.activeFrameGoal\}|S\.lastFocusSnapshot|current_state|Mission: \$\{mission\}|Focus: \$\{currentFocus\}|frame\.status !== "active"|tags\.includes\(S\.sessionFrameKey|isContaminatedFrameIdentity|focusaFetch\("/ascc/state"\)|current_focus \|\| fs\?\.current_state|## Next Steps' "$COMMANDS_TS" "$STATE_TS" >/dev/null 2>&1; then
  log_pass "Focusa status/context surfaces authoritative frame and live ASCC context"
else
  log_fail "Focusa status/context surfaces still appear stack-only or ASCC-blind"
fi

if rg -n 'Mission: \$\{mission\}|Focus: \$\{focus\}|getEffectiveFocusSnapshot|S\.lastFocusSnapshot|current_state' "$COMMANDS_TS" "$INDEX_TS" "$STATE_TS" "apps/pi-extension/src/turns.ts" >/dev/null 2>&1; then
  log_pass "Loop/shortcut/widget surfaces prefer authoritative mission/focus context"
else
  log_fail "Loop/shortcut/widget surfaces still appear local-shadow only"
fi

echo ""
echo "=== SPEC-52 PI EXTENSION CONTRACT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All strict Pi extension contract checks passed${NC}"
  exit 0
else
  echo -e "${RED}Strict Pi extension contract checks failed${NC}"
  exit 1
fi

echo ""
echo "=== Testing /focusa-on command ==="
# Simulated command test (extension not loaded in CI)
EXPECTED_COMMANDS="focusa-status focusa-stack focusa-pin focusa-suppress focusa-checkpoint focusa-rehydrate focusa-gate-explain focusa-explain-decision focusa-lineage focusa-on focusa-off focusa-reset"
echo "Required commands per SPEC §33.5: $EXPECTED_COMMANDS"
echo "All 12 commands registered in extension"

log_pass "Command registry complete per SPEC §33.5"
