#!/usr/bin/env bash
set -euo pipefail

BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-/home/wirebot/focusa}"
CONTINUITY_ID="spec102-requested-id-fallback-$(date +%s)-$$"
TMP_DIR="${TMPDIR:-/tmp}/spec102-workpoint-requested-id-fallback"
mkdir -p "${TMP_DIR}"

fail() {
  echo "✗ FAIL: $*" >&2
  exit 1
}

pass() {
  echo "✓ PASS: $*"
}

checkpoint_body="${TMP_DIR}/checkpoint-body.json"
checkpoint_out="${TMP_DIR}/checkpoint.json"
valid_out="${TMP_DIR}/resume-valid.json"
bad_body="${TMP_DIR}/resume-bad-body.json"
bad_out="${TMP_DIR}/resume-bad.json"

jq -n \
  --arg project_root "${PROJECT_ROOT}" \
  --arg continuity_id "${CONTINUITY_ID}" \
  --arg mission "Spec102 requested id fallback regression" \
  --arg next_slice "Verify requested Workpoint id fallback disclosure" \
  '{project_root:$project_root, continuity_id:$continuity_id, mission:$mission, current_action:"spec102_requested_id_fallback_test", next_slice:$next_slice, canonical:true}' \
  >"${checkpoint_body}"

curl -fsS --max-time 15 -X POST "${BASE}/v1/workpoint/checkpoint" \
  -H 'content-type: application/json' \
  --data-binary @"${checkpoint_body}" \
  >"${checkpoint_out}"

valid_id="$(jq -r '.workpoint_id // empty' "${checkpoint_out}")"
[[ -n "${valid_id}" && "${valid_id}" != "null" ]] || fail "checkpoint did not return workpoint_id"

jq -n \
  --arg project_root "${PROJECT_ROOT}" \
  --arg continuity_id "${CONTINUITY_ID}" \
  '{mode:"compact_prompt", project_root:$project_root, continuity_id:$continuity_id}' \
  | curl -fsS --max-time 15 -X POST "${BASE}/v1/workpoint/resume" \
      -H 'content-type: application/json' \
      --data-binary @- \
      >"${valid_out}"

jq -e --arg valid_id "${valid_id}" '
  .status == "completed"
  and .canonical == true
  and .workpoint_id == $valid_id
  and ((.fallback_used // false) == false)
  and (.requested_workpoint_id // null) == null
' "${valid_out}" >/dev/null || fail "valid happy path should stay canonical and free of fallback disclosure"
pass "valid Workpoint resume happy path has no requested-id/fallback scar"

bad_id="00000000-0000-4000-8000-000000000102"
jq -n \
  --arg project_root "${PROJECT_ROOT}" \
  --arg continuity_id "${CONTINUITY_ID}" \
  --arg bad_id "${bad_id}" \
  '{mode:"compact_prompt", project_root:$project_root, continuity_id:$continuity_id, workpoint_id:$bad_id}' \
  >"${bad_body}"

curl -fsS --max-time 15 -X POST "${BASE}/v1/workpoint/resume" \
  -H 'content-type: application/json' \
  --data-binary @"${bad_body}" \
  >"${bad_out}"

jq -e --arg bad_id "${bad_id}" --arg valid_id "${valid_id}" '
  .requested_workpoint_id == $bad_id
  and .requested_found == false
  and .fallback_used == true
  and .fallback_source == "active_workstream"
  and .fallback_object_id == $valid_id
  and .canonical_for_requested_scope == false
  and .canonical_for_fallback_scope == true
  and .workpoint_id == $valid_id
' "${bad_out}" >/dev/null || fail "missing requested Workpoint id must disclose active-workstream fallback and remain non-canonical for requested scope"
pass "missing requested Workpoint id discloses fallback without silent canonical-for-requested result"

echo "SPEC102 workpoint requested-id fallback runtime test: PASS"
