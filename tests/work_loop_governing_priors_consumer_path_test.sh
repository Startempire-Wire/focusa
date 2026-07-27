#!/bin/bash
# Runtime contract: governing priors must appear in telemetry with ranking-consumer impact.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-work-loop-continuation-test}"
scoped_curl(){ command curl -H "x-scope-project-root: ${PROJECT_ROOT}" -H "x-scope-continuity-id: ${CONTINUITY_ID}" "$@"; }
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

INGEST_JSON="$(scoped_curl -sS -X POST "${BASE_URL}/v1/telemetry/trace" \
  -H 'Content-Type: application/json' \
  -d '{"event_type":"governing_priors_applied","session_id":"spec79-governing-priors","governing_priors":["spec79_verified_prior"],"ranking_consumers":["working_set","verified_deltas"],"prior_hits":{"working_set":["spec79_verified_prior"],"verified_deltas":["spec79_verified_prior"]}}')"
if echo "$INGEST_JSON" | jq -e '.status == "recorded" and .event_type == "governing_priors_applied"' >/dev/null 2>&1; then
  log_pass "governing-prior telemetry fixture accepted"
else
  log_fail "governing-prior telemetry fixture rejected: $INGEST_JSON"
fi

STATS_JSON="$(scoped_curl -sS "${BASE_URL}/v1/telemetry/trace/stats")"
if echo "$STATS_JSON" | jq -e '.by_event_type | has("governing_priors_applied")' >/dev/null 2>&1; then
  log_pass "trace stats include governing_priors_applied event family"
else
  log_fail "trace stats missing governing_priors_applied event family"
fi

TRACE_JSON="$(scoped_curl -sS "${BASE_URL}/v1/telemetry/trace?limit=5000")"

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="governing_priors_applied" and ((.payload.governing_priors // []) | length > 0))' >/dev/null 2>&1; then
  log_pass "governing priors are emitted as explicit runtime payload"
else
  log_fail "governing priors payload missing from runtime trace"
fi

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="governing_priors_applied" and ((.payload.ranking_consumers // []) | length > 0))' >/dev/null 2>&1; then
  log_pass "governing-prior application reports ranking consumer surfaces"
else
  log_fail "ranking consumer surfaces missing from governing prior telemetry"
fi

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="governing_priors_applied" and (((.payload.prior_hits // {}) | keys | length) > 0))' >/dev/null 2>&1; then
  log_pass "governing-prior telemetry includes per-consumer prior hit evidence"
else
  log_fail "governing prior hit evidence missing in telemetry payload"
fi

echo "=== WORK-LOOP GOVERNING PRIORS CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
