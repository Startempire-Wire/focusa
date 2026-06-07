#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in rollback_card latest_safe_snapshot reversible_actions irreversible_actions restore_tool restore_scope expected_after_restore focusa_tree_restore_state; do
  rg -F "$term" crates/focusa-api/src/routes/workpoint.rs >/dev/null || fail "workpoint rollback card missing $term"
done
pass "workpoint routes declare rollback_card contract terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-rollback-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"rollback-agent\",\"work_item_id\":\"focusa-pm2b.31\",\"mission\":\"Spec102 rollback fixture\",\"next_slice\":\"rollback affordance\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-rollback-wp.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-rollback-wp.json)
[[ -n "$WP" && "$WP" != null ]] || fail "checkpoint missing workpoint_id"
jq -e --arg wp "$WP" '
  .rollback_card.latest_safe_snapshot.snapshot_id != null
  and (.rollback_card.reversible_actions | index("workpoint_checkpoint"))
  and (.rollback_card.irreversible_actions | length) == 0
  and .rollback_card.restore_tool == "focusa_tree_restore_state"
  and .rollback_card.restore_scope.workpoint_id == $wp
  and .rollback_card.restore_scope.project_root == "'$ROOT_DIR'"
  and (.rollback_card.expected_after_restore | test("safe snapshot"; "i"))
  and (.next_step_hint | test("resume"; "i"))
' /tmp/spec102-rollback-wp.json >/dev/null || fail "checkpoint response missing scoped rollback_card"
pass "checkpoint response includes scoped rollback_card in details"

EVID="evidence:$KEY"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"workpoint_id\":\"$WP\",\"target_ref\":\"tests/spec102_undo_rollback_affordance_test.sh\",\"result\":\"PASS rollback proof $KEY\",\"evidence_ref\":\"$EVID\"}" \
  "$BASE/v1/workpoint/evidence/link" >/tmp/spec102-rollback-evidence.json
jq -e --arg wp "$WP" '
  .rollback_card.latest_safe_snapshot.snapshot_id != null
  and (.rollback_card.reversible_actions | index("workpoint_evidence_link"))
  and (.rollback_card.irreversible_actions | length) == 0
  and .rollback_card.restore_tool == "focusa_tree_restore_state"
  and .rollback_card.restore_scope.workpoint_id == $wp
  and .rollback_card.restore_scope.project_root == "'$ROOT_DIR'"
  and (.rollback_card.expected_after_restore | test("verification_records|safe snapshot"; "i"))
' /tmp/spec102-rollback-evidence.json >/dev/null || fail "evidence response missing scoped rollback_card"
pass "evidence link response includes scoped rollback_card"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$WP\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-rollback-resume.json
jq -e '(.rendered_summary | contains("rollback_card") | not) and (.rendered_summary | contains("restore_tool") | not)' /tmp/spec102-rollback-resume.json >/dev/null || fail "normal happy path rendered alarming rollback banner"
pass "normal resume happy path does not show rollback banner"

echo "SPEC102 undo rollback affordance test: PASS"
