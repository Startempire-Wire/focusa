#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${ROOT}/target/debug/focusa-daemon}"
PORT="${FOCUSA_SPEC130_HANDOFF_PORT:-$(python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
)}"
BASE_URL="http://127.0.0.1:${PORT}"
DATA_DIR="$(mktemp -d /tmp/focusa-spec130-handoff.XXXXXX)"
PROJECT_ROOT="${DATA_DIR}/project"
WORK_ITEM_ID="focusa-w26jj.3.11.3"
DAEMON_PID=""

cleanup() {
  if [[ -n "${DAEMON_PID}" ]]; then
    kill "${DAEMON_PID}" 2>/dev/null || true
    wait "${DAEMON_PID}" 2>/dev/null || true
  fi
  rm -rf "${DATA_DIR}"
}
trap cleanup EXIT

[[ -x "${DAEMON_BIN}" ]] || { echo "missing daemon binary: ${DAEMON_BIN}" >&2; exit 1; }
mkdir -p "${DATA_DIR}/daemon" "${PROJECT_ROOT}/.beads"
cat >"${PROJECT_ROOT}/.focusa-project.json" <<JSON
{"schema":"focusa.project.v1","project_id":"focusa-spec130-handoff","canonical_name":"Spec130 Handoff Fixture","project_root":"${PROJECT_ROOT}","beads_prefix":"focusa","workspace_kind":"fixture","aliases":[]}
JSON
cat >"${PROJECT_ROOT}/.beads/issues.jsonl" <<JSON
{"id":"${WORK_ITEM_ID}","title":"Cross-agent handoff fixture","description":"Bounded target attachment proof","status":"closed","priority":0,"issue_type":"task","created_at":"2026-07-22T00:00:00Z","updated_at":"2026-07-22T00:00:00Z","closed_at":"2026-07-22T00:00:00Z"}
JSON

FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_DATA_DIR="${DATA_DIR}/daemon" \
  "${DAEMON_BIN}" >"${DATA_DIR}/daemon.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 160); do
  curl -fsS "${BASE_URL}/v1/health" >/dev/null 2>&1 && break
  sleep 0.1
done
curl -fsS "${BASE_URL}/v1/health" >/dev/null

capabilities="$(curl -fsS "${BASE_URL}/v1/agent/adapter-capabilities")"
jq -e '
  .schema == "focusa.adapter_capability_registry.v1" and
  ([.adapters[].adapter] | sort) == (["claude","codex","opencode","pi"] | sort) and
  (.adapters[] | select(.adapter == "pi") | .supports_rpc_rollover == true) and
  ([.adapters[] | select(.adapter != "pi") | .supports_rpc_rollover] | all(. == false))
' <<<"${capabilities}" >/dev/null

source_continuity="handoff-pi-source"
source_session="session-pi-source"
checkpoint="$(jq -n \
  --arg root "${PROJECT_ROOT}" \
  --arg continuity "${source_continuity}" \
  --arg session "${source_session}" \
  --arg work_item "${WORK_ITEM_ID}" \
  '{project_root:$root,continuity_id:$continuity,session_id:$session,work_item_id:$work_item,mission:"Preserve mission, blockers and evidence across adapter rotation",current_action:"cross_agent_handoff",next_slice:"materialize the next target attachment",active_object_refs:["evidence:handoff-source","blocker:capability-truth"],canonical:true,promote:true,idempotency_key:"spec130-cross-agent-source"}')"
source_checkpoint="$(curl -fsS -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  -H "x-scope-project-root: ${PROJECT_ROOT}" \
  -H "x-scope-continuity-id: ${source_continuity}" \
  -H 'content-type: application/json' -d "${checkpoint}")"
source_workpoint="$(jq -er '.workpoint_id' <<<"${source_checkpoint}")"

receipts=()
lineage=("${source_workpoint}")
adapters=(claude codex opencode pi)
for index in "${!adapters[@]}"; do
  adapter="${adapters[$index]}"
  target_continuity="handoff-${adapter}-$((index + 1))"
  target_session="session-${adapter}-$((index + 1))"
  checkpoint_ref="checkpoint:${source_workpoint}"
  workpoint_packet_ref="workpoint:${source_workpoint}"
  compaction_packet_ref="compaction:hop-$((index + 1))"

  rollover_body="$(jq -n \
    --arg root "${PROJECT_ROOT}" \
    --arg source_continuity "${source_continuity}" \
    --arg target_continuity "${target_continuity}" \
    --arg source_session "${source_session}" \
    --arg target_session "${target_session}" \
    --arg source_checkpoint "${checkpoint_ref}" \
    --arg adapter "${adapter}" \
    '{action:"rollover",project_root:$root,continuity_id:$source_continuity,target_continuity_id:$target_continuity,source_session_id:$source_session,target_session_id:$target_session,source_checkpoint_id:$source_checkpoint,compaction_packet_id:("compaction:"+$target_continuity),adapter:$adapter,evidence_refs:["evidence:mission-fidelity","evidence:blocker-fidelity"],mission:"Preserve mission, blockers and evidence across adapter rotation",next_action:"materialize and verify target Workpoint",receipt_preview:true,receipt_commit:false}')"
  rollover="$(curl -fsS -X POST "${BASE_URL}/v1/project/session-transfer" \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${source_continuity}" \
    -H 'content-type: application/json' -d "${rollover_body}")"
  jq -e --arg source "${source_continuity}" --arg target "${target_continuity}" --arg adapter "${adapter}" '
    .status == "completed" and .saved == true and
    .transfer.source_scope.continuity_id == $source and
    .transfer.target_scope.continuity_id == $target and
    .transfer.transition.status == "target_attachment_pending" and
    .transfer.transition.requires_target_resume_verification == true and
    .transfer.transition.adapter == $adapter
  ' <<<"${rollover}" >/dev/null

  materialize_body="$(jq -n \
    --arg root "${PROJECT_ROOT}" \
    --arg source_continuity "${source_continuity}" \
    --arg target_continuity "${target_continuity}" \
    --arg source_session "${source_session}" \
    --arg target_session "${target_session}" \
    --arg checkpoint_ref "${checkpoint_ref}" \
    --arg workpoint_packet_ref "${workpoint_packet_ref}" \
    --arg compaction_packet_ref "${compaction_packet_ref}" \
    '{project_root:$root,source_continuity_id:$source_continuity,target_continuity_id:$target_continuity,source_session_id:$source_session,target_session_id:$target_session,checkpoint_ref:$checkpoint_ref,workpoint_packet_ref:$workpoint_packet_ref,compaction_packet_ref:$compaction_packet_ref}')"
  materialized="$(curl -fsS -X POST "${BASE_URL}/v1/workpoint/rollover/target-materialize" \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${source_continuity}" \
    -H 'content-type: application/json' -d "${materialize_body}")"
  target_workpoint="$(jq -er --arg target "${target_continuity}" --arg source_wp "${source_workpoint}" '
    select(.status == "completed" and .canonical == true and .target_continuity_id == $target and (.source_workpoint_id|tostring) == $source_wp) | .target_workpoint_id
  ' <<<"${materialized}")"
  materialized_replay="$(curl -fsS -X POST "${BASE_URL}/v1/workpoint/rollover/target-materialize" \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${source_continuity}" \
    -H 'content-type: application/json' -d "${materialize_body}")"
  jq -e --arg target_wp "${target_workpoint}" '
    .status == "completed" and .canonical == true and .status_hint == "idempotent_replay" and
    (.target_workpoint_id|tostring) == $target_wp
  ' <<<"${materialized_replay}" >/dev/null

  resume_body="$(jq -n --arg root "${PROJECT_ROOT}" --arg continuity "${target_continuity}" --arg session "${target_session}" --arg workpoint "${target_workpoint}" '{project_root:$root,continuity_id:$continuity,session_id:$session,workpoint_id:$workpoint,mode:"operator_summary",current_ask:"Continue cross-agent handoff proof"}')"
  resumed="$(curl -fsS -X POST "${BASE_URL}/v1/workpoint/resume" \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${target_continuity}" \
    -H 'content-type: application/json' -d "${resume_body}")"
  jq -e --arg target "${target_continuity}" --arg workpoint "${target_workpoint}" '
    .status == "completed" and .canonical == true and
    .details.tool_result_v1.scope.continuity_id == $target and
    (.details.tool_result_v1.scope.workpoint_id|tostring) == $workpoint
  ' <<<"${resumed}" >/dev/null

  verify_body="$(jq -n \
    --arg root "${PROJECT_ROOT}" \
    --arg source_continuity "${source_continuity}" \
    --arg target_session "${target_session}" \
    --arg target_workpoint "${target_workpoint}" \
    '{action:"verify_target",project_root:$root,continuity_id:$source_continuity,target_session_id:$target_session,target_workpoint_id:$target_workpoint,target_resume_canonical:true,evidence_refs:["evidence:target-resume","evidence:lineage-fidelity"]}')"
  verified="$(curl -fsS -X POST "${BASE_URL}/v1/project/session-transfer" \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${source_continuity}" \
    -H 'content-type: application/json' -d "${verify_body}")"
  receipt="$(jq -er --arg target "${target_continuity}" --arg target_wp "${target_workpoint}" '
    select(.transfer.transition.status == "target_resume_verified" and .transfer.target_scope.continuity_id == $target and (.transfer.transition_receipt.target_workpoint_id|tostring) == $target_wp and .transfer.transition_receipt.target_resume_canonical == true) | .transfer.transition_receipt.receipt_id
  ' <<<"${verified}")"
  verified_replay="$(curl -fsS -X POST "${BASE_URL}/v1/project/session-transfer" \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${source_continuity}" \
    -H 'content-type: application/json' -d "${verify_body}")"
  jq -e --arg receipt "${receipt}" '
    .transfer.transition.status == "target_resume_verified" and
    .transfer.transition_receipt.receipt_id == $receipt and
    .transfer.transition_receipt.idempotent_replay == true
  ' <<<"${verified_replay}" >/dev/null

  receipts+=("${receipt}")
  lineage+=("${target_workpoint}")
  source_continuity="${target_continuity}"
  source_session="${target_session}"
  source_workpoint="${target_workpoint}"
done

[[ "$(printf '%s\n' "${receipts[@]}" | sort -u | wc -l | tr -d ' ')" == "4" ]]
[[ "${#lineage[@]}" == "5" ]]

jq -n \
  --argjson hops "${#adapters[@]}" \
  --arg final_continuity "${source_continuity}" \
  --arg final_workpoint "${source_workpoint}" \
  --argjson unique_receipts "$(printf '%s\n' "${receipts[@]}" | sort -u | wc -l | tr -d ' ')" \
  '{status:"passed",hops:$hops,adapters:["pi","claude","codex","opencode","pi"],final_continuity:$final_continuity,final_workpoint:$final_workpoint,unique_transition_receipts:$unique_receipts,capability_truth_preserved:true}'
