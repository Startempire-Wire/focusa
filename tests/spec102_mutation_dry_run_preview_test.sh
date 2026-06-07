#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in mutation_preview would_create would_update would_link authority_scope safe_to_apply irreversible preview dry_run; do
  rg -F "$term" crates/focusa-api/src/routes/workpoint.rs >/dev/null || fail "workpoint preview missing $term"
done
pass "workpoint routes declare mutation preview envelope terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-preview-$$"
WP="019ea0aa-0000-7000-8000-$(printf '%012x' $$)"
# Preview checkpoint must not create/resume the Workpoint.
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"preview\":true,\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"preview-agent\",\"workpoint_id\":\"$WP\",\"work_item_id\":\"focusa-pm2b.30\",\"mission\":\"Spec102 dry-run preview\",\"active_object_refs\":[\"crates/focusa-api/src/routes/workpoint.rs\"],\"next_slice\":\"preview mutation\",\"canonical\":true}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-preview-checkpoint.json
jq -e --arg wp "$WP" '
  .status == "preview"
  and .preview == true
  and (.side_effects | length) == 0
  and .workpoint_id == $wp
  and .mutation_preview.route == "POST /v1/workpoint/checkpoint"
  and (.mutation_preview.would_create | length) >= 1
  and (.mutation_preview.would_update | length) >= 1
  and .mutation_preview.authority_scope.project_root == "'$ROOT_DIR'"
  and .mutation_preview.risk == "low"
  and .mutation_preview.irreversible == false
  and .mutation_preview.safe_to_apply == true
' /tmp/spec102-preview-checkpoint.json >/dev/null || fail "checkpoint preview missing mutation envelope"
set +e
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$WP\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-preview-resume.json
RES=$?
set -e
jq -e '.status == "not_found" or .canonical == false' /tmp/spec102-preview-resume.json >/dev/null || fail "checkpoint preview unexpectedly created Workpoint"
pass "checkpoint preview shows would_create/update without mutation"

# Apply one real checkpoint, then preview evidence link without linking.
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"preview-agent\",\"work_item_id\":\"focusa-pm2b.30\",\"mission\":\"Spec102 dry-run apply fixture\",\"next_slice\":\"preview evidence\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-preview-real-wp.json
RWP=$(jq -r '.workpoint_id // empty' /tmp/spec102-preview-real-wp.json)
EVID="evidence:$KEY:preview-only"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"preview\":true,\"workpoint_id\":\"$RWP\",\"target_ref\":\"tests/spec102_mutation_dry_run_preview_test.sh\",\"result\":\"PASS preview only $KEY\",\"evidence_ref\":\"$EVID\"}" \
  "$BASE/v1/workpoint/evidence/link" >/tmp/spec102-preview-evidence.json
jq -e --arg wp "$RWP" --arg evid "$EVID" '
  .status == "preview"
  and .preview == true
  and (.side_effects | length) == 0
  and .mutation_preview.route == "POST /v1/workpoint/evidence/link"
  and (.mutation_preview.would_link[] | select(.workpoint_id == $wp and .evidence_ref == $evid))
  and .mutation_preview.safe_to_apply == true
' /tmp/spec102-preview-evidence.json >/dev/null || fail "evidence preview missing would_link envelope"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$RWP\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-preview-real-resume.json
jq -e --arg evid "$EVID" '[.. | strings | select(. == $evid)] | length == 0' /tmp/spec102-preview-real-resume.json >/dev/null || fail "evidence preview unexpectedly linked evidence"
pass "evidence preview shows would_link without mutation"

echo "SPEC102 mutation dry-run preview test: PASS"
