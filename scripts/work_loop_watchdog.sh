#!/usr/bin/env bash
set -euo pipefail

BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
SLEEP_SECS="${WATCHDOG_SLEEP_SECS:-2}"

pick_parent_epic() {
  (cd /home/wirebot/focusa && bd list --type epic --status open --status in_progress 2>/dev/null | awk '{print $2}' | head -n1) || true
}

enable_continuous_unbounded() {
  local writer="$1"
  local parent="$2"
  curl -sS -X POST "$BASE/v1/work-loop/enable" \
    -H 'Content-Type: application/json' \
    -H "x-focusa-writer-id: $writer" \
    -H 'x-focusa-approval: approved' \
    -d "{\"preset\":\"push\",\"root_work_item_id\":\"$parent\",\"policy_overrides\":{\"require_verification_before_persist\":false,\"require_operator_for_governance\":false,\"require_operator_for_scope_change\":false,\"max_turns\":1000000,\"max_wall_clock_ms\":315360000000,\"max_retries\":1000000,\"max_same_subproblem_retries\":1000000,\"max_consecutive_low_productivity_turns\":1000000,\"max_consecutive_failures\":1000000,\"cooldown_ms\":50}}" >/dev/null || true
}

while true; do
  status_json=$(curl -sS "$BASE/v1/work-loop/status" || true)
  if [[ -z "${status_json}" ]]; then
    sleep "$SLEEP_SECS"
    continue
  fi

  closure_bundle_json=$(curl -sS "$BASE/v1/work-loop/replay/closure-bundle" || true)
  closure_bundle_status=$(jq -r '.status // "error"' <<<"$closure_bundle_json")

  if [[ "$closure_bundle_status" == "ok" ]]; then
    closure_replay_status=$(jq -r '.secondary_loop_replay_consumer.status // "error"' <<<"$closure_bundle_json")
    closure_pair_observed=$(jq -r '.secondary_loop_replay_consumer.secondary_loop_closure_replay_evidence.evidence.current_task_pair_observed // false' <<<"$closure_bundle_json")
    continuity_gate_state=$(jq -r '.secondary_loop_continuity_gate.state // "fail-closed"' <<<"$closure_bundle_json")
  else
    closure_replay_json=$(curl -sS "$BASE/v1/work-loop/replay/closure-evidence" || true)
    closure_replay_status=$(jq -r '.status // "error"' <<<"$closure_replay_json")
    closure_pair_observed=$(jq -r '.secondary_loop_closure_replay_evidence.evidence.current_task_pair_observed // false' <<<"$closure_replay_json")
    continuity_gate_state=$(jq -r '.secondary_loop_continuity_gate.state // (if .status == "ok" then "open" else "fail-closed" end)' <<<"$closure_replay_json")
  fi

  closure_replay_ready=false
  if [[ "$closure_replay_status" == "ok" && "$continuity_gate_state" == "open" ]]; then
    closure_replay_ready=true
  fi

  enabled=$(jq -r '.enabled // false' <<<"$status_json")
  writer=$(jq -r '.active_writer // empty' <<<"$status_json")
  status=$(jq -r '.status // empty' <<<"$status_json")
  current_task=$(jq -r '.current_task.work_item_id // empty' <<<"$status_json")
  tranche_id=$(jq -r '.current_task.tranche_id // empty' <<<"$status_json")
  last_blocker_reason=$(jq -r '.last_blocker_reason // empty' <<<"$status_json")
  commitment_release_state=$(jq -r '.commitment_lifecycle.release_semantics.state // empty' <<<"$status_json")
  commitment_decay_posture=$(jq -r '.commitment_lifecycle.decay_semantics.decay_posture // empty' <<<"$status_json")
  session_id=$(jq -r '.transport.daemon_supervised_session.session_id // empty' <<<"$status_json")
  operator_steering_detected=$(jq -r '.decision_context.operator_steering_detected // false' <<<"$status_json")
  governance_decision_pending=$(jq -r '.pause_flags.governance_decision_pending // false' <<<"$status_json")

  continuation_boundary_active=false
  continuation_boundary_reason=""
  if [[ "$operator_steering_detected" == "true" ]]; then
    continuation_boundary_active=true
    continuation_boundary_reason="operator steering detected"
  elif [[ "$governance_decision_pending" == "true" ]]; then
    continuation_boundary_active=true
    continuation_boundary_reason="governance decision pending"
  fi

  if [[ "$enabled" != "true" || -z "$writer" ]]; then
    sleep "$SLEEP_SECS"
    continue
  fi

  if [[ -z "$session_id" ]]; then
    curl -sS -X POST "$BASE/v1/work-loop/driver/start" \
      -H 'Content-Type: application/json' \
      -H "x-focusa-writer-id: $writer" \
      -d '{"cwd":"/home/wirebot/focusa"}' >/dev/null || true
  fi

  if [[ "$status" == "paused" && "$last_blocker_reason" == *"max_turns budget exhausted"* ]]; then
    parent=$(pick_parent_epic)
    curl -sS -X POST "$BASE/v1/work-loop/stop" \
      -H 'Content-Type: application/json' \
      -H "x-focusa-writer-id: $writer" \
      -d '{"reason":"watchdog budget rollover"}' >/dev/null || true
    if [[ -n "$parent" ]]; then
      enable_continuous_unbounded "$writer" "$parent"
    fi
  fi

  # Doc-78 first consumer path: fail-closed continuity handoff requires replay consumer health + open continuity gate from closure-bundle (fallback: closure-evidence).
  # Secondary continuation boundary is stronger than replay readiness: no auto-handoff while operator steering or governance pause is active.
  if [[ "$status" == "blocked" && "$commitment_release_state" == "released_on_blocker" && "$closure_replay_ready" == "true" && "$continuation_boundary_active" == "false" ]]; then
    parent="$tranche_id"
    if [[ -z "$parent" ]]; then
      parent=$(pick_parent_epic)
    fi
    if [[ -n "$parent" ]]; then
      curl -sS -X POST "$BASE/v1/work-loop/select-next" \
        -H 'Content-Type: application/json' \
        -H "x-focusa-writer-id: $writer" \
        -d "{\"parent_work_item_id\":\"$parent\"}" >/dev/null || true
    fi
  fi

  # Doc-73 first consumer path: continuity handoff is driven by commitment release state.
  # Doc-78 extension: replay consumer must be healthy and continuity gate open before auto handoff proceeds.
  # Continuation-boundary guard keeps watchdog from issuing select-next while operator/governance authority is unresolved.
  if [[ -z "$current_task" && ( "$status" == "awaiting_harness_turn" || "$status" == "idle" || "$status" == "selecting_ready_work" || "$status" == "advancing_task" || "$status" == "evaluating_outcome" || "$status" == "blocked" || "$status" == "paused" ) && ( "$commitment_release_state" == "released_on_completion" || "$commitment_release_state" == "released_on_blocker" || "$commitment_release_state" == "released_or_unbound" ) && "$closure_replay_ready" == "true" && "$continuation_boundary_active" == "false" ]]; then
    parent="$tranche_id"
    if [[ -z "$parent" ]]; then
      parent=$(pick_parent_epic)
    fi
    if [[ -n "$parent" ]]; then
      curl -sS -X POST "$BASE/v1/work-loop/select-next" \
        -H 'Content-Type: application/json' \
        -H "x-focusa-writer-id: $writer" \
        -d "{\"parent_work_item_id\":\"$parent\"}" >/dev/null || true
    fi
  fi

  if [[ "$closure_replay_status" == "ok" && "$closure_pair_observed" == "false" && "$status" == "blocked" && "$commitment_release_state" == "released_on_blocker" ]]; then
    : # fail closed: blocked release cannot auto-handoff until per-task replay pair evidence is observed.
  fi

  if [[ "$continuation_boundary_active" == "true" ]]; then
    : # fail closed: operator/governance continuation boundary supersedes watchdog continuity handoff.
  fi

  if [[ "$commitment_decay_posture" == "decaying" && -n "$current_task" ]]; then
    : # keep current commitment bound; daemon will enforce retry/decay policy thresholds.
  fi

  sleep "$SLEEP_SECS"
done
