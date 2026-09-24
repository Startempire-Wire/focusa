#!/usr/bin/env bash
# Shared runtime-test fixture: real scoped admission, no production gate bypass.
# Caller owns cleanup via `trap focusa_test_scope_cleanup EXIT`.
focusa_test_scope_cleanup() {
  python3 - "$FOCUSA_FIXTURE_ROOT" <<'PY'
import shutil
import sys
shutil.rmtree(sys.argv[1])
PY
}

focusa_test_scope_create() {
  local base="$1" continuity="$2" response
  [[ "${FOCUSA_TEST_MODE:-0}" == 1 ]] || {
    echo 'Admitted fixture requires an isolated FOCUSA_TEST_MODE daemon' >&2
    return 1
  }
  FOCUSA_FIXTURE_ROOT="$(mktemp -d /tmp/focusa-admitted-fixture.XXXXXX)"
  mkdir "$FOCUSA_FIXTURE_ROOT/.beads"
  printf '%s\n' '{"id":"focusa-032h","title":"Isolated runtime fixture","status":"open","priority":1,"issue_type":"task"}' > "$FOCUSA_FIXTURE_ROOT/.beads/issues.jsonl"
  jq -nc --arg root "$FOCUSA_FIXTURE_ROOT" '{schema:"focusa.project.v1",project_id:"runtime-contract",canonical_name:"Runtime contract fixture",project_root:$root,workspace_kind:"isolated-test"}' > "$FOCUSA_FIXTURE_ROOT/.focusa-project.json"
  response=$(curl -sS --fail-with-body -X POST "$base/v1/trajectory/define-goal" \
    -H "x-scope-project-root: $FOCUSA_FIXTURE_ROOT" -H "x-scope-continuity-id: $continuity" \
    -H 'Content-Type: application/json' -d "$(jq -nc --arg root "$FOCUSA_FIXTURE_ROOT" --arg continuity "$continuity" '{project_root:$root,continuity_id:$continuity,long_term_goal:"Verify isolated runtime compatibility",desired_end_state:"Admitted runtime contract checks pass",current_state:"Fresh isolated test project",current_ask:"Verify isolated runtime compatibility",mid_level_goal:"Verify runtime writes",short_term_goal:"Run runtime contract",waypoints:["Verify runtime contract"],goal_source:"operator",operator_confirmed:true}')") || return 1
  jq -e '.status == "completed" and .canonical == true' <<<"$response" >/dev/null || {
    printf 'Trajectory fixture rejected: %s\n' "$response" >&2
    return 1
  }
  response=$(curl -sS --fail-with-body -X POST "$base/v1/workpoint/checkpoint" \
    -H "x-scope-project-root: $FOCUSA_FIXTURE_ROOT" -H "x-scope-continuity-id: $continuity" \
    -H 'Content-Type: application/json' -d "$(jq -nc --arg root "$FOCUSA_FIXTURE_ROOT" --arg continuity "$continuity" '{project_root:$root,continuity_id:$continuity,mission:"Verify isolated runtime compatibility",current_ask:"Verify isolated runtime compatibility",action_intent:{action_type:"verify_runtime_contract",lifecycle_stage:"verify_outcome",status:"ready"},next_slice:"Verify runtime contract state",canonical:true}')") || return 1
  jq -e '.status == "accepted" and .canonical == true' <<<"$response" >/dev/null || {
    printf 'Workpoint fixture rejected: %s\n' "$response" >&2
    return 1
  }
}
