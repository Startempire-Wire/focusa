#!/bin/bash
# SPEC-50 ontology event contract
# Verifies named ontology reducer/audit events are emitted into the canonical event log.

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

recent_events() {
  curl -sS "${BASE_URL}/v1/events/recent?limit=300"
}

proposal_id_for_source() {
  local source="$1"
  curl -sS "${BASE_URL}/v1/proposals" | jq -r --arg source "$source" '.proposals | map(select(.source == $source)) | last | .id // empty'
}

assert_event_for_proposal() {
  local proposal_id="$1"
  local event_type="$2"
  local desc="$3"
  if recent_events | jq -e --arg id "$proposal_id" --arg typ "$event_type" '.events | any(.type == $typ and .proposal_id == $id)' >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc"
  fi
}

assert_event_type() {
  local event_type="$1"
  local desc="$2"
  if recent_events | jq -e --arg typ "$event_type" '.events | any(.type == $typ)' >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc"
  fi
}

thread_name="ontology-events-thread-$(date +%s%N)"
thread_resp=$(curl -sS -X POST "${BASE_URL}/v1/threads" -H "Content-Type: application/json" -d "{\"name\":\"${thread_name}\",\"primary_intent\":\"ontology event seed\"}")
thread_id=$(echo "$thread_resp" | jq -r '.thread.id // .thread_id // empty')

log_info "working set membership proposed + promoted + refreshed"
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" -H "Content-Type: application/json" -d '{"kind":"focus_change"}' >/dev/null || true
focus_source="spec50-focus-$(date +%s%N)"
focus_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" -H "Content-Type: application/json" -d "{\"kind\":\"focus_change\",\"source\":\"${focus_source}\",\"score\":0.999,\"deadline_ms\":60000,\"payload\":{\"title\":\"spec50 focus\",\"goal\":\"spec50 focus\",\"beads_issue_id\":\"spec50-focus\"}}")
if echo "$focus_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then log_pass "focus proposal submitted"; else log_fail "focus proposal submit failed"; fi
focus_id=$(proposal_id_for_source "$focus_source")
if [ -n "$focus_id" ]; then
  assert_event_for_proposal "$focus_id" "ontology_working_set_membership_proposed" "ontology_working_set_membership_proposed emitted"
else
  log_fail "focus proposal id not found"
fi
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" -H "Content-Type: application/json" -d '{"kind":"focus_change"}' >/dev/null
assert_event_type "ontology_proposal_promoted" "ontology_proposal_promoted emitted"
if recent_events | jq -e '.events | any(.type == "ontology_verification_applied" and .outcome == "accepted")' >/dev/null 2>&1; then
  log_pass "ontology_verification_applied emitted"
else
  log_fail "ontology_verification_applied missing"
fi
if recent_events | jq -e '.events | any(.type == "ontology_working_set_refreshed")' >/dev/null 2>&1; then
  log_pass "ontology_working_set_refreshed emitted"
else
  log_fail "ontology_working_set_refreshed missing"
fi

log_info "object upsert proposed"
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" -H "Content-Type: application/json" -d '{"kind":"memory_write"}' >/dev/null || true
object_source="spec50-object-$(date +%s%N)"
object_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" -H "Content-Type: application/json" -d "{\"kind\":\"memory_write\",\"source\":\"${object_source}\",\"score\":0.97,\"deadline_ms\":60000,\"payload\":{\"key\":\"spec50-object-key\",\"value\":\"spec50-object-val\"}}")
if echo "$object_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then log_pass "object proposal submitted"; else log_fail "object proposal submit failed"; fi
object_id=$(proposal_id_for_source "$object_source")
if [ -n "$object_id" ]; then
  assert_event_for_proposal "$object_id" "ontology_object_upsert_proposed" "ontology_object_upsert_proposed emitted"
else
  log_fail "object proposal id not found"
fi
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" -H "Content-Type: application/json" -d '{"kind":"memory_write"}' >/dev/null

log_info "status change proposed"
auto_source="spec50-status-$(date +%s%N)"
auto_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" -H "Content-Type: application/json" -d "{\"kind\":\"autonomy_adjustment\",\"source\":\"${auto_source}\",\"score\":0.91,\"deadline_ms\":60000,\"payload\":{\"level\":\"AL1\"}}")
if echo "$auto_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then log_pass "status proposal submitted"; else log_fail "status proposal submit failed"; fi
auto_id=$(proposal_id_for_source "$auto_source")
if [ -n "$auto_id" ]; then
  assert_event_for_proposal "$auto_id" "ontology_status_change_proposed" "ontology_status_change_proposed emitted"
else
  log_fail "status proposal id not found"
fi

log_info "link upsert proposed"
link_source="spec50-link-$(date +%s%N)"
link_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" -H "Content-Type: application/json" -d "{\"kind\":\"thesis_update\",\"source\":\"${link_source}\",\"score\":0.92,\"deadline_ms\":60000,\"payload\":{\"thread_id\":\"${thread_id}\",\"primary_intent\":\"spec50-link-thesis\",\"link_type\":\"supports\",\"source_id\":\"source-a\",\"target_id\":\"target-b\"}}")
if echo "$link_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then log_pass "link proposal submitted"; else log_fail "link proposal submit failed"; fi
link_id=$(proposal_id_for_source "$link_source")
if [ -n "$link_id" ]; then
  assert_event_for_proposal "$link_id" "ontology_link_upsert_proposed" "ontology_link_upsert_proposed emitted"
else
  log_fail "link proposal id not found"
fi

log_info "proposal rejected"
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" -H "Content-Type: application/json" -d '{"kind":"memory_write"}' >/dev/null || true
reject_source="spec50-reject-$(date +%s%N)"
reject_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" -H "Content-Type: application/json" -d "{\"kind\":\"memory_write\",\"source\":\"${reject_source}\",\"score\":0.05,\"deadline_ms\":60000,\"payload\":{\"key\":\"spec50-reject-key\",\"value\":\"spec50-reject-val\"}}")
if echo "$reject_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then log_pass "reject proposal submitted"; else log_fail "reject proposal submit failed"; fi
reject_id=$(proposal_id_for_source "$reject_source")
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" -H "Content-Type: application/json" -d '{"kind":"memory_write"}' >/dev/null
if [ -n "$reject_id" ]; then
  assert_event_type "ontology_proposal_rejected" "ontology_proposal_rejected emitted"
else
  log_fail "reject proposal id not found"
fi

echo ""
echo "=== ONTOLOGY EVENT CONTRACT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Ontology event contract verified${NC}"
  exit 0
else
  echo -e "${RED}Ontology event contract failed${NC}"
  exit 1
fi
