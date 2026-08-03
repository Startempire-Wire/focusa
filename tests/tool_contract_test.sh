#!/bin/bash
# SPEC-55: Tool Action Contracts — strict CI gate

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BODY_FILE="/tmp/focusa-tool-contract-body.json"
SCOPE_HEADERS=()
FAILED=0
PASSED=0
LIVE_MUTATION=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() {
  echo -e "${RED}✗ FAIL${NC}: $1"
  FAILED=$((FAILED+1))
  if [ "$LIVE_MUTATION" = "1" ]; then
    exit 1
  fi
}
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

http_code() {
  curl -sS -o "$BODY_FILE" -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" "$BODY_FILE" >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat "$BODY_FILE")"
  fi
}

echo "=== SPEC-55: Tool Action Contracts (strict) ==="
echo "Base URL: ${BASE_URL}"
echo ""

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n '(^|[[:space:]])(export[[:space:]]+)?type[[:space:]]+PushDeltaFailureReason[[:space:]]*=' "$TOOLS_TS" >/dev/null 2>&1 \
  && rg -n '"offline"' "$TOOLS_TS" >/dev/null 2>&1 \
  && rg -n '"no_active_frame"' "$TOOLS_TS" >/dev/null 2>&1 \
  && rg -n '"validation_rejected"' "$TOOLS_TS" >/dev/null 2>&1 \
  && rg -n '"write_failed"' "$TOOLS_TS" >/dev/null 2>&1; then
  log_pass "PushDelta exposes required write failure reasons"
else
  log_fail "PushDelta failure reasons missing required contract taxonomy"
fi

if rg -n 'response\.status === "no_active_frame"|response\.status === "rejected"|response\.status !== "accepted"' "$TOOLS_TS" >/dev/null 2>&1; then
  log_pass "PushDelta inspects write status envelope before reporting success"
else
  log_fail "PushDelta does not inspect write status envelope faithfully"
fi

if rg -n 'mirrorFailedFocusWrite\("decision"|mirrorFailedFocusWrite\("constraint"|mirrorFailedFocusWrite\("failure"' "$TOOLS_TS" >/dev/null 2>&1; then
  log_pass "Operator-critical write tools mirror unrecoverable failures to scratchpad"
else
  log_fail "Operator-critical write fallback mirroring missing"
fi

if [ "$#" -eq 0 ]; then
  echo ""
  echo "Static/read-only contract gate complete: ${PASSED} passed, ${FAILED} failed."
  [ "$FAILED" -eq 0 ]
  exit $?
fi
if [ "$#" -ne 2 ] || [ "$1" != "--live-isolated" ] || [ "$2" != "--confirm" ]; then
  echo "Usage: $0 [--live-isolated --confirm]" >&2
  echo "Default mode is static/read-only; live mutation always starts a disposable daemon and scope." >&2
  exit 2
fi

LIVE_MUTATION=1
TEST_RUN_ID="tool-contract-$(date -u +%Y%m%dT%H%M%SZ)-$$"
TEST_ROOT="$(mktemp -d "/tmp/${TEST_RUN_ID}.XXXXXX")"
TEST_PROJECT_ROOT="${TEST_ROOT}/project"
TEST_CONTINUITY_ID="${TEST_RUN_ID}"
TEST_SESSION_ID="session-${TEST_RUN_ID}"
DAEMON_BIN="${FOCUSA_TEST_DAEMON_BIN:-${ROOT_DIR}/target/debug/focusa-daemon}"
RECEIPT_DIR="${FOCUSA_TEST_RECEIPT_DIR:-/tmp}"
RECEIPT_PATH="${RECEIPT_DIR}/${TEST_RUN_ID}-cleanup-receipt.json"
PORT="$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(('127.0.0.1', 0))
    print(sock.getsockname()[1])
PY
)"
BASE_URL="http://127.0.0.1:${PORT}"
BODY_FILE="${TEST_ROOT}/response.json"
mkdir -p "$TEST_PROJECT_ROOT/.git" "$TEST_ROOT/data" "$RECEIPT_DIR"
printf '{"schema":"focusa.project.v1","project_id":"%s","canonical_name":"Focusa isolated tool contract","project_root":"%s"}\n' \
  "$TEST_RUN_ID" "$TEST_PROJECT_ROOT" > "$TEST_PROJECT_ROOT/.focusa-project.json"
if [ ! -x "$DAEMON_BIN" ]; then
  echo "isolated daemon binary unavailable: $DAEMON_BIN" >&2
  exit 2
fi
if ! command -v bd >/dev/null 2>&1; then
  echo "bd is required to create the isolated contract fixture" >&2
  exit 2
fi
(
  cd "$TEST_PROJECT_ROOT"
  env -u BEADS_DIR bd init --prefix tc --quiet
)
TEST_BEAD_ID="$(cd "$TEST_PROJECT_ROOT" && env -u BEADS_DIR bd create "Isolated tool contract fixture" --silent)"

DAEMON_PID=""
cleanup_isolated_test() {
  local exit_status=$?
  trap - EXIT INT TERM
  if [ -n "$DAEMON_PID" ] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  python3 - "$RECEIPT_PATH" "$TEST_RUN_ID" "$TEST_PROJECT_ROOT" "$TEST_CONTINUITY_ID" "$TEST_SESSION_ID" "$exit_status" <<'PY'
import json, pathlib, sys
path, run_id, root, continuity, session, status = sys.argv[1:]
pathlib.Path(path).write_text(json.dumps({
  "schema": "focusa.tool_contract_cleanup_receipt.v1",
  "run_id": run_id,
  "project_root": root,
  "continuity_id": continuity,
  "session_id": session,
  "exit_status": int(status),
  "disposition": "disposable daemon stopped; isolated state retained for diagnosis",
}, indent=2) + "\n")
PY
  echo "Cleanup receipt: $RECEIPT_PATH"
  exit "$exit_status"
}
trap cleanup_isolated_test EXIT INT TERM

env -u BEADS_DIR \
FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="${TEST_ROOT}/data" \
FOCUSA_DISABLE_MDNS=1 \
"$DAEMON_BIN" >"${TEST_ROOT}/daemon.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 80); do
  if curl -fsS "${BASE_URL}/v1/health" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$DAEMON_PID" 2>/dev/null; then
    echo "isolated daemon exited before health; see ${TEST_ROOT}/daemon.log" >&2
    exit 1
  fi
  sleep 0.1
done
curl -fsS "${BASE_URL}/v1/health" >/dev/null || { echo "isolated daemon health timeout" >&2; exit 1; }
SCOPE_HEADERS=(
  -H "x-scope-project-root: ${TEST_PROJECT_ROOT}"
  -H "x-scope-continuity-id: ${TEST_CONTINUITY_ID}"
  -H "x-scope-session-id: ${TEST_SESSION_ID}"
)
log_info "Preflight isolated daemon pid=${DAEMON_PID} scope=${TEST_RUN_ID}"

log_info "Health"
code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  json_assert '.ok == true and (.version | type == "string")' "Daemon health/version schema"
else
  log_fail "Daemon health failed with HTTP ${code}"
  exit 1
fi

log_info "Input schema validation"
code=$(http_code -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" \
  "${SCOPE_HEADERS[@]}" \
  -d "{\"adapter_id\":\"pi\",\"workspace_id\":\"${TEST_PROJECT_ROOT}\",\"project_root\":\"${TEST_PROJECT_ROOT}\",\"continuity_id\":\"${TEST_CONTINUITY_ID}\"}")
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Session start accepted before focus push"
else
  log_fail "Session start returned HTTP ${code}"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
  "${SCOPE_HEADERS[@]}" \
  -d "{\"title\":\"tool-contract-test\",\"goal\":\"verify contract\",\"beads_issue_id\":\"${TEST_BEAD_ID}\",\"project_root\":\"${TEST_PROJECT_ROOT}\",\"continuity_id\":\"${TEST_CONTINUITY_ID}\"}")
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Valid focus push accepted"
else
  log_fail "Valid focus push returned HTTP ${code}"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus/set-active" -H "Content-Type: application/json" \
  -d '{"frame_id":"not-a-uuid"}')
if [ "$code" = "422" ]; then
  log_pass "Invalid UUID rejected with HTTP 422"
else
  log_fail "Invalid UUID expected HTTP 422, got ${code} :: $(cat "$BODY_FILE")"
fi

log_info "Failure modes"
code=$(http_code -X POST "${BASE_URL}/v1/nonexistent" -H "Content-Type: application/json" -d '{}')
if [ "$code" = "404" ]; then
  log_pass "Unknown route rejected with HTTP 404"
else
  log_fail "Unknown route expected HTTP 404, got ${code}"
fi

code=$(http_code -X POST "${BASE_URL}/v1/prompt/assemble" -H "Content-Type: application/json" -d '{"turn_id":"bad"}')
if [ "$code" = "422" ]; then
  log_pass "Bad prompt payload rejected with HTTP 422"
else
  log_fail "Bad prompt payload expected HTTP 422, got ${code} :: $(cat "$BODY_FILE")"
fi

log_info "Idempotency — strict"
TURN_ID="idem-test-$(date +%s%N)"
code=$(http_code -X POST "${BASE_URL}/v1/turn/start" -H "Content-Type: application/json" \
  "${SCOPE_HEADERS[@]}" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"test\",\"adapter_id\":\"test\",\"timestamp\":\"2026-04-11T00:00:00Z\"}")
if [ "$code" = "200" ]; then
  log_pass "Turn start accepted for idempotency test"
else
  log_fail "Turn start failed for idempotency test"
fi

code=$(http_code -X POST "${BASE_URL}/v1/turn/complete" -H "Content-Type: application/json" \
  "${SCOPE_HEADERS[@]}" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
if [ "$code" = "200" ]; then
  log_pass "First turn complete accepted"
else
  log_fail "First turn complete failed"
fi

duplicate_seen=0
for _ in 1 2 3 4 5; do
  sleep 0.3
  code=$(http_code -X POST "${BASE_URL}/v1/turn/complete" -H "Content-Type: application/json" \
    "${SCOPE_HEADERS[@]}" \
    -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
  if [ "$code" = "200" ] && jq -e '.duplicate == true' "$BODY_FILE" >/dev/null 2>&1; then
    duplicate_seen=1
    break
  fi
done
if [ "$duplicate_seen" = "1" ]; then
  log_pass "Turn complete duplicate flagged explicitly"
else
  log_fail "Idempotency duplicate flag missing :: $(cat "$BODY_FILE")"
fi

log_info "Observable side effects"
code=$(http_code -X POST "${BASE_URL}/v1/memory/semantic/upsert" -H "Content-Type: application/json" \
  "${SCOPE_HEADERS[@]}" \
  -d "{\"key\":\"${TEST_RUN_ID}\",\"value\":\"testing\"}")
if [ "$code" = "200" ]; then
  log_pass "Semantic upsert accepted"
else
  log_fail "Semantic upsert failed"
fi
sleep 1
code=$(http_code "${SCOPE_HEADERS[@]}" "${BASE_URL}/v1/memory/semantic")
if [ "$code" = "200" ]; then
  json_assert '.semantic != null' "Semantic memory observable after upsert"
else
  log_fail "Semantic memory fetch failed"
fi

log_info "Timeout policy / degraded fallback"
code=$(http_code "${BASE_URL}/v1/status")
if [ "$code" = "200" ]; then
  json_assert '.worker_status.queue_size_config != null and .worker_status.job_timeout_ms != null' "Worker timeout policy visible"
else
  log_fail "Status fetch failed"
fi

code=$(http_code "${BASE_URL}/v1/reflect/status")
if [ "$code" = "200" ]; then
  json_assert '.enabled != null' "Reflection degraded fallback status visible"
else
  log_fail "Reflect status fetch failed"
fi

log_info "Action contract matrix"
code=$(http_code "${BASE_URL}/v1/ontology/contracts")
if [ "$code" = "200" ]; then
  json_assert '.contracts | length >= 10' "Ontology action contract catalog exposed"
  json_assert '.contracts | any(.name == "refactor_module" and .input_schema.required != null and .failure_modes != null and .tool_mappings != null)' "Refactor contract exposes schema/failure/tool mappings"
  json_assert '.contracts | any(.name == "modify_schema" and .rollback_availability.available == true and .timeout_policy.job_timeout_ms_field == "worker_status.job_timeout_ms")' "Modify-schema contract exposes rollback + timeout policy"
  json_assert '.contracts | any(.name == "mark_blocked" and (.failure_modes | index("dependency_failure")) and (.verification_hooks | length) >= 1)' "Blocker contract exposes failure/verification semantics"
else
  log_fail "Ontology contracts fetch failed"
fi


PROXY_RS="${ROOT_DIR}/crates/focusa-api/src/routes/proxy.rs"
if rg -n 'proxy_failure|proxy_auth_missing|proxy_upstream_failed|proxy_upstream_http|proxy_validation_rejected|recovery_hint|misuse_hint|tool_result_v1' "$PROXY_RS" >/dev/null; then
  log_pass "Proxy failures expose no-guess recovery contract"
else
  log_fail "Proxy failures lack no-guess recovery contract"
fi

TRUST_RS="${ROOT_DIR}/crates/focusa-api/src/routes/trust.rs"
if rg -n 'trust_failure|trust_forbidden|trust_dispatch_failed|recovery_hint|misuse_hint|tool_result_v1' "$TRUST_RS" >/dev/null; then
  log_pass "Trust failures expose no-guess recovery contract"
else
  log_fail "Trust failures lack no-guess recovery contract"
fi

echo ""
echo "=== SPEC-55 TOOL ACTION CONTRACTS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All strict tool contract checks passed${NC}"
  exit 0
else
  echo -e "${RED}Strict tool contract checks failed${NC}"
  exit 1
fi
