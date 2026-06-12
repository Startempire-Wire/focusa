#!/bin/bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:18787}"
export FOCUSA_BASE_URL="$BASE_URL"
export FOCUSA_BIND="${FOCUSA_BIND:-127.0.0.1:18787}"
export FOCUSA_DATA_DIR="${FOCUSA_DATA_DIR:-$(mktemp -d /tmp/focusa-spec-gates.XXXXXX)}"

DAEMON_BIN="${DAEMON_BIN:-./target/release/focusa-daemon}"
if [ ! -x "$DAEMON_BIN" ]; then
  CARGO_BIN="${CARGO_BIN:-cargo}"
  "$CARGO_BIN" build -p focusa-api --release --bin focusa-daemon
fi
"$DAEMON_BIN" >/tmp/focusa-daemon.log 2>&1 &
DAEMON_PID=$!
cleanup() {
  kill "$DAEMON_PID" >/dev/null 2>&1 || true
  rm -rf "$FOCUSA_DATA_DIR" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for i in $(seq 1 60); do
  if curl -fsS "${BASE_URL}/v1/health" >/dev/null; then
    break
  fi
  sleep 1
  if [ "$i" -eq 60 ]; then
    echo "daemon failed to become healthy"
    exit 1
  fi
done

run_gate() {
  curl -sS -X POST "${BASE_URL}/v1/session/close" \
    -H "Content-Type: application/json" \
    -d '{"reason":"ci-spec-gate-isolation"}' >/dev/null || true
  "$@"
}

run_gate ./tests/focusa_toggle_persistence_test.sh
run_gate ./tests/tool_contract_test.sh
run_gate ./tests/command_write_contract_test.sh
run_gate ./tests/trace_dimensions_test.sh
run_gate ./tests/pi_extension_contract_test.sh
run_gate ./tests/behavioral_alignment_test.sh
run_gate ./tests/channel_separation_test.sh
run_gate ./tests/proxy_mode_b_parity_test.sh
run_gate ./tests/checkpoint_trigger_test.sh
run_gate ./tests/restart_recovery_test.sh
run_gate ./tests/fork_compact_recovery_test.sh
run_gate ./tests/continuous_pruning_test.sh
run_gate ./tests/thread_runtime_test.sh
run_gate ./tests/proposal_submit_contract_test.sh
run_gate ./tests/proposal_resolution_enforcement_test.sh
run_gate ./tests/proposal_kind_enforcement_test.sh
run_gate ./tests/proposal_governance_enforcement_test.sh
run_gate ./tests/canonical_writer_guardrail_test.sh
run_gate ./tests/focus_frame_write_contract_test.sh
run_gate ./tests/work_loop_continuation_inputs_test.sh
run_gate ./tests/work_loop_policy_consumption_test.sh
run_gate ./tests/work_loop_policy_enforcement_test.sh
run_gate ./tests/work_loop_preset_semantics_test.sh
run_gate ./tests/pi_rpc_driver_contract_test.sh
run_gate ./tests/focus_work_command_surface_test.sh
run_gate ./tests/worktree_discipline_guardrail_test.sh
run_gate ./tests/work_loop_turn_outcome_wiring_test.sh
run_gate ./tests/work_loop_autocontinue_wiring_test.sh
run_gate ./tests/work_loop_route_contract_test.sh
run_gate ./tests/ontology_event_contract_test.sh
run_gate ./tests/ontology_world_contract_test.sh
run_gate ./tests/golden_tasks_eval.sh
run_gate ./tests/scope_routing_regression_eval.sh
run_gate ./tests/golden_tasks_comparative_eval.sh
run_gate bash ./tests/security_dynamic_api_smoke_static_test.sh
run_gate env DAEMON_BIN="$DAEMON_BIN" bash ./tests/security_dynamic_api_smoke_test.sh
run_gate bash ./tests/security_non_loopback_auth_guard_static_test.sh
run_gate env DAEMON_BIN="$DAEMON_BIN" bash ./tests/security_non_loopback_auth_guard_dynamic_test.sh
run_gate bash ./tests/spec96_menubar_cockpit_foundation_static_test.sh
run_gate bash ./tests/phone_bridge_public_url_static_test.sh
run_gate bash ./tests/phone_bridge_automatic_callback_static_test.sh
run_gate bash ./tests/release_notes_workflow_static_test.sh
run_gate bash ./tests/release_proof_status_route_static_test.sh
run_gate bash ./tests/spec80_impl_parquet_export_support_test.sh
run_gate bash ./tests/spec96_trajectory_context_tool_docs_static_test.sh
run_gate bash ./tests/spec96_static_false_positive_guard_test.sh
