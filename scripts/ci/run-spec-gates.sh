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

./tests/focusa_toggle_persistence_test.sh
./tests/tool_contract_test.sh
./tests/command_write_contract_test.sh
./tests/trace_dimensions_test.sh
./tests/pi_extension_contract_test.sh
./tests/behavioral_alignment_test.sh
./tests/channel_separation_test.sh
./tests/proxy_mode_b_parity_test.sh
./tests/checkpoint_trigger_test.sh
./tests/restart_recovery_test.sh
./tests/fork_compact_recovery_test.sh
./tests/continuous_pruning_test.sh
./tests/thread_runtime_test.sh
./tests/proposal_submit_contract_test.sh
./tests/proposal_resolution_enforcement_test.sh
./tests/proposal_kind_enforcement_test.sh
./tests/proposal_governance_enforcement_test.sh
./tests/canonical_writer_guardrail_test.sh
./tests/focus_frame_write_contract_test.sh
./tests/work_loop_continuation_inputs_test.sh
./tests/work_loop_policy_consumption_test.sh
./tests/work_loop_policy_enforcement_test.sh
./tests/work_loop_preset_semantics_test.sh
./tests/pi_rpc_driver_contract_test.sh
./tests/focus_work_command_surface_test.sh
./tests/worktree_discipline_guardrail_test.sh
./tests/work_loop_turn_outcome_wiring_test.sh
./tests/work_loop_autocontinue_wiring_test.sh
./tests/work_loop_route_contract_test.sh
./tests/ontology_event_contract_test.sh
./tests/ontology_world_contract_test.sh
./tests/golden_tasks_eval.sh
./tests/scope_routing_regression_eval.sh
./tests/golden_tasks_comparative_eval.sh
