#!/usr/bin/env bash
set -euo pipefail
BASE_URL="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
TMP="${TMPDIR:-/tmp}/spec97-reflex-runtime-dogfood.json"
REQ="${TMPDIR:-/tmp}/spec97-reflex-runtime-dogfood-request.json"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

curl -fsS --max-time 5 "$BASE_URL/v1/reflex/primitives?family=recovery&limit=2" > "$TMP"
jq -e '.status=="completed" and .read_only==true and .advisory_only==true and (.items|length)==2 and .bounds.truncated==true' "$TMP" >/dev/null || fail "direct reflex primitive route dogfood failed"
pass "direct reflex primitive route dogfood passed"

printf '%s' '{"surface":"reflex_primitives","selector":"family","anchor":"recovery","limit":2}' > "$REQ"
curl -fsS --max-time 5 "$BASE_URL/v1/traverse" \
  -H 'content-type: application/json' \
  --data-binary "@$REQ" > "$TMP"
jq -e '.status=="completed" and (.items|length)>=1 and .items[0].data.primitive_id=="route_noncanonical_result" and .items[0].data.family=="recovery"' "$TMP" >/dev/null || fail "traverse reflex primitive dogfood failed"
pass "traverse reflex primitive dogfood passed"

printf '%s' '{"surface":"reflex_primitives","selector":"family","anchor":"recovery","limit":1,"include_full_payload":true,"budget_tokens":1}' > "$REQ"
curl -fsS --max-time 5 "$BASE_URL/v1/traverse" \
  -H 'content-type: application/json' \
  --data-binary "@$REQ" > "$TMP"
jq -e '.degraded==true and (.reflex_suggestions | index("resource_mode_fallback")) and (.details.tool_result_v1.reflex_suggestions | index("resource_mode_fallback"))' "$TMP" >/dev/null || fail "degraded reflex payload did not expose API-native reflex suggestions"
pass "degraded reflex payload exposes API-native reflex suggestions"

echo "SPEC97 reflex runtime dogfood: PASS"
