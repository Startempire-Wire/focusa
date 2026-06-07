#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in profile_selector routine_commands BLOATGAURD_PROFILE CONTEXT_POSTURE FULL_PAYLOAD Daily Driver Beast Mode Speedy Neat Freak Tightwad scout librarian squeezer deep_dive gatekeeper; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "profile/routine registry missing $term"
done
pass "traverse declares profile selector and routine command terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
curl -fsS --max-time 15 -H 'Content-Type: application/json' -d '{"surface":"profile_selector","selector":"registry","limit":20}' "$BASE/v1/traverse" >/tmp/spec102-profiles.json
jq -e '
  .traversal.profile_selector.schema == "focusa.profile_selector.v1"
  and ([.items[].data.profile_id] | index("daily_driver"))
  and ([.items[].data.profile_id] | index("beast_mode"))
  and ([.items[].data.profile_id] | index("speedy"))
  and ([.items[].data.profile_id] | index("neat_freak"))
  and ([.items[].data.profile_id] | index("tightwad"))
  and all(.items[].data; .availability == "implemented" or .availability == "partial" or .availability == "spec_only")
  and all(.items[].data; (.mutates == false) and (.authority == "render_policy_only"))
' /tmp/spec102-profiles.json >/dev/null || fail "profiles missing/unsafe"
pass "profiles discoverable with availability and non-authority semantics"

curl -fsS --max-time 15 -H 'Content-Type: application/json' -d '{"surface":"routine_commands","selector":"registry","limit":20}' "$BASE/v1/traverse" >/tmp/spec102-routines.json
jq -e '
  .traversal.routine_commands.schema == "focusa.routine_commands.v1"
  and ([.items[].data.routine_id] | index("scout"))
  and ([.items[].data.routine_id] | index("librarian"))
  and ([.items[].data.routine_id] | index("squeezer"))
  and ([.items[].data.routine_id] | index("deep_dive"))
  and ([.items[].data.routine_id] | index("gatekeeper"))
  and all(.items[].data; (.mutates == false) and (.requires_verified_scope == true))
  and all(.items[].data; .availability == "implemented" or .availability == "partial" or .availability == "spec_only")
' /tmp/spec102-routines.json >/dev/null || fail "routine commands missing/unsafe"
pass "routine commands discoverable and labeled by availability"

echo "SPEC102 profile selector routine commands test: PASS"
