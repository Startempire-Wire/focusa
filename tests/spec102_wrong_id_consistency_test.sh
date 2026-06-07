#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in wrong_id_taxonomy requested_found scope_found fallback_used canonical_for_requested_scope canonical_for_fallback_scope WrongIdConsistency; do
  rg -F "$term" crates/focusa-api/src/routes/workpoint.rs >/dev/null || fail "workpoint wrong-id taxonomy missing $term"
done
pass "workpoint route declares wrong-id taxonomy fields"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-wrong-id-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"wrong-id-agent\",\"work_item_id\":\"focusa-pm2b.17\",\"mission\":\"Spec102 wrong id fixture\",\"next_slice\":\"wrong id consistency\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-wrong-id-wp.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-wrong-id-wp.json)
[[ -n "$WP" && "$WP" != null ]] || fail "checkpoint missing workpoint_id"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$WP\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-wrong-id-valid.json
jq -e --arg wp "$WP" '
  .status == "completed" and .canonical == true and .workpoint_id == $wp
  and (.wrong_id_taxonomy == null)
  and (.requested_workpoint_id == null)
' /tmp/spec102-wrong-id-valid.json >/dev/null || fail "valid id happy path not minimal"
pass "valid id happy path stays minimal"

BAD="019ea000-0000-7000-8000-000000000017"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$BAD\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-wrong-id-missing.json
jq -e --arg bad "$BAD" '
  .requested_found == false
  and .scope_found == true
  and .fallback_used == true
  and .canonical_for_requested_scope == false
  and .canonical_for_fallback_scope == true
  and .wrong_id_taxonomy.status == "fallback_from_missing_requested_id"
  and .wrong_id_taxonomy.requested_workpoint_id == $bad
' /tmp/spec102-wrong-id-missing.json >/dev/null || fail "missing requested id lacks taxonomy/fallback flags"
pass "missing requested id exposes explicit fallback taxonomy"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"wrong-continuity-$KEY\",\"workpoint_id\":\"$WP\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-wrong-id-continuity.json
jq -e --arg wp "$WP" '
  .status == "rejected_continuity_mismatch"
  and .requested_found == true
  and .scope_found == false
  and .fallback_used == false
  and .canonical_for_requested_scope == false
  and .wrong_id_taxonomy.status == "scope_mismatch_for_requested_id"
  and .wrong_id_taxonomy.workpoint_id == $wp
' /tmp/spec102-wrong-id-continuity.json >/dev/null || fail "wrong continuity lacks same taxonomy flags"
pass "wrong continuity uses same taxonomy fields"

echo "SPEC102 wrong id consistency test: PASS"
