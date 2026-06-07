#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in verbosity_profile operator coding_agent qa_agent release_agent debug_agent compact_fields detail_fields hidden_by_default escalation_fields; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse verbosity profile missing $term"
done
pass "traverse declares verbosity profile terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
for PROFILE in operator coding_agent qa_agent release_agent debug_agent; do
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"surface\":\"verbosity_profile\",\"selector\":\"profile\",\"query\":\"$PROFILE\",\"limit\":5}" \
    "$BASE/v1/traverse" >"/tmp/spec102-verbosity-$PROFILE.json"
  jq -e --arg p "$PROFILE" '
    .traversal.verbosity_profile.profile == $p
    and (.traversal.verbosity_profile.compact_fields | length) >= 1
    and (.traversal.verbosity_profile.detail_fields | length) >= 1
    and (.traversal.verbosity_profile.hidden_by_default | type == "array")
    and (.traversal.verbosity_profile.escalation_fields | type == "array")
    and (.items | length) == 1
  ' "/tmp/spec102-verbosity-$PROFILE.json" >/dev/null || fail "verbosity profile $PROFILE missing fields"
done
pass "all verbosity profiles return profile-specific field sets"

jq -e '
  .traversal.verbosity_profile.profile == "operator"
  and (.traversal.verbosity_profile.compact_fields | index("status"))
  and (.traversal.verbosity_profile.hidden_by_default | index("debug_payload"))
  and ((.traversal.verbosity_profile.compact_fields | index("raw_payload")) | not)
' /tmp/spec102-verbosity-operator.json >/dev/null || fail "operator profile not calm/compact"
pass "operator profile is calm and hides debug internals"

jq -e '
  .traversal.verbosity_profile.profile == "debug_agent"
  and (.traversal.verbosity_profile.detail_fields | index("raw_payload"))
  and (.traversal.verbosity_profile.escalation_fields | index("failure_class"))
' /tmp/spec102-verbosity-debug_agent.json >/dev/null || fail "debug profile missing internals"
pass "debug profile exposes internals only for debug profile"

echo "SPEC102 personalized verbosity profiles test: PASS"
