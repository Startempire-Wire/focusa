#!/usr/bin/env bash
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

curl -fsS --max-time 5 "$BASE_URL/v1/health" >/dev/null || fail "daemon health unreachable"
pass "daemon health reachable"

card=$(curl -fsS --get "$BASE_URL/v1/awareness/card" \
  --data-urlencode 'adapter_id=public-stream-test' \
  --data-urlencode 'workspace_id=focusa' \
  --data-urlencode 'agent_id=test-agent' \
  --data-urlencode 'operator_id=verious' \
  --data-urlencode 'project_root=/home/wirebot/focusa' \
  --data-urlencode 'continuity_id=focusa-cont-root-20b6704c-5a49-4d9d-a4b6-a30bf45bfc61')

for field in schema project_identity_display_name redacted_scope_id canonical_status tool_family evidence_refs_public_safe redaction_status secret_scan_status publish_allowed; do
  jq -e --arg field "$field" '.public_stream_policy | has($field)' <<<"$card" >/dev/null || fail "public_stream_policy missing $field"
done
pass "awareness/card JSON includes required public policy fields"

jq -e '.public_stream_policy.schema == "focusa.public_card.v1" and .public_stream_policy.publish_allowed == false and .public_stream_policy.redaction_status == "redacted_scope_only" and .public_stream_policy.secret_scan_status == "not_required_no_raw_payload" and (.public_stream_policy.redacted_scope_id | startswith("scope:"))' <<<"$card" >/dev/null || fail "public_stream_policy values mismatch"
pass "awareness/card public policy is deny-by-default and scope-redacted"

rendered=$(jq -r '.rendered_card' <<<"$card")
for needle in 'PUBLIC_CARD:' 'schema=focusa.public_card.v1' 'redacted_scope_id=scope:' 'publish_allowed=false' 'secret_scan_status=not_required_no_raw_payload'; do
  grep -F "$needle" <<<"$rendered" >/dev/null || fail "rendered card missing $needle"
done
pass "rendered card includes PUBLIC_CARD redaction block"

if jq -e '.public_stream_policy.redacted_scope_id | contains("/home/wirebot/focusa")' <<<"$card" >/dev/null; then
  fail "redacted_scope_id leaked raw project path"
fi
pass "redacted scope id does not leak raw project path"

echo "public stream redaction policy live-safe test: PASS"
