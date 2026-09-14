#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
  export CARGO_TARGET_DIR="/tmp/focusa-ci-local-$$-1"
fi
export FOCUSA_CARGO_TARGET_DIR="${FOCUSA_CARGO_TARGET_DIR:-$CARGO_TARGET_DIR}"
cleanup_ephemeral_builds() {
  "$ROOT_DIR/scripts/ci/cleanup-ephemeral-build-target.sh" "$CARGO_TARGET_DIR"
}
trap cleanup_ephemeral_builds EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

EXPECTED_OWNER="$(stat -c %U "$ROOT_DIR")"
find_owner_drift() {
  find "$ROOT_DIR" -xdev     \( -path "$ROOT_DIR/.git" -o -path "$ROOT_DIR/target" -o -path '*/node_modules' -o -path "$ROOT_DIR/data" -o -path "$ROOT_DIR/ecs" \) -prune -o     -user root -print -quit
}
if [[ "$EXPECTED_OWNER" != root ]]; then
  OWNER_DRIFT="$(find_owner_drift)"
  if [[ -n "$OWNER_DRIFT" && ${EUID:-$(id -u)} -eq 0 && -x /usr/local/bin/fix-user-perms ]]; then
    /usr/local/bin/fix-user-perms "$EXPECTED_OWNER"
    OWNER_DRIFT="$(find_owner_drift)"
  fi
  if [[ -n "$OWNER_DRIFT" ]]; then
    echo "workspace ownership drift: $OWNER_DRIFT is root-owned; run fix-user-perms $EXPECTED_OWNER" >&2
    exit 1
  fi
fi

if [[ -z "${FOCUSA_BIND:-}" ]]; then
  GATE_PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
  export FOCUSA_BIND="127.0.0.1:${GATE_PORT}"
else
  GATE_PORT="${FOCUSA_BIND##*:}"
fi
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:${GATE_PORT}}"
export FOCUSA_BASE_URL="$BASE_URL"
export FOCUSA_DATA_DIR="${FOCUSA_DATA_DIR:-$(mktemp -d /tmp/focusa-spec-gates.XXXXXX)}"
# Isolated CI daemon must exercise real entitlement path, not 403.
# FOCUSA_TEST_MODE=1 grants a bounded test lease (active, sha256, 1h) so write
# gating still executes. See crates/focusa-api/src/main.rs:322 and
# crates/focusa-api/src/middleware/entitlement.rs:369.
export FOCUSA_TEST_MODE="${FOCUSA_TEST_MODE:-1}"
export FOCUSA_HISTORYLESS_GATE="${FOCUSA_HISTORYLESS_GATE:-0}"
TEST_BEADS_FIXTURE=""
TEST_GIT_DIR=""
if [[ "$FOCUSA_TEST_MODE" == "1" ]] && ! git -C "$ROOT_DIR" rev-parse --git-dir >/dev/null 2>&1; then
  # OVH source sync intentionally excludes the worktree's .git metadata.
  # Provide a disposable two-commit graph for read-only evidence gates;
  # never copy or mutate repository history on the build host.
  TEST_GIT_DIR="$(mktemp -d "$ROOT_DIR/../gate-git-meta.XXXXXX")"
  git init -q "$TEST_GIT_DIR"
  git -C "$TEST_GIT_DIR" -c user.name=focusa-test -c user.email=focusa-test@invalid commit --allow-empty -qm 'synthetic gate base'
  git -C "$TEST_GIT_DIR" -c user.name=focusa-test -c user.email=focusa-test@invalid commit --allow-empty -qm 'synthetic gate head'
  export GIT_DIR="$TEST_GIT_DIR"
  export GIT_WORK_TREE="$ROOT_DIR"
fi
if [[ "$FOCUSA_TEST_MODE" == "1" && ! -s "$ROOT_DIR/.beads/issues.jsonl" ]]; then
  # OVH source sync intentionally excludes repository Beads history. Supply
  # only the synthetic issue required by command-write contract tests, and
  # remove it on every exit; never copy or mutate operator task history.
  mkdir -p "$ROOT_DIR/.beads"
  printf '%s\n' '{"id":"focusa-032h","status":"open"}' > "$ROOT_DIR/.beads/issues.jsonl"
  TEST_BEADS_FIXTURE="$ROOT_DIR/.beads/issues.jsonl"
fi

export DAEMON_BIN="${DAEMON_BIN:-$CARGO_TARGET_DIR/release/focusa-daemon}"
if [ ! -x "$DAEMON_BIN" ]; then
  CARGO_BIN="${CARGO_BIN:-cargo}"
  export CARGO_PROFILE_RELEASE_LTO="${CARGO_PROFILE_RELEASE_LTO:-off}"
  # The release daemon build with thin-LTO can exhaust CI/small-host
  # resources.  CI workflows supply CARGO_PROFILE_RELEASE_LTO=off so the
  # spec-gates daemon builds without cross-crate optimization; the
  # release pipeline uses musl + cross for the shipped artifact.
  "$CARGO_BIN" build -p focusa-api --release --bin focusa-daemon
fi
if [ ! -x "$DAEMON_BIN" ]; then
  echo "spec-gates daemon missing after successful build: $DAEMON_BIN" >&2
  exit 1
fi
# Per-user daemon log: a root/wirebot-owned /tmp/focusa-daemon.log once
# blocked the github-runner user from starting the daemon (EACCES).
DAEMON_LOG="${DAEMON_LOG:-/tmp/focusa-daemon.$(id -u).log}"
"$DAEMON_BIN" >"$DAEMON_LOG" 2>&1 &
DAEMON_PID=$!
cleanup() {
  kill "$DAEMON_PID" >/dev/null 2>&1 || true
  rm -rf "$FOCUSA_DATA_DIR" >/dev/null 2>&1 || true
  if [[ -n "$TEST_BEADS_FIXTURE" ]]; then
    rm -f "$TEST_BEADS_FIXTURE" >/dev/null 2>&1 || true
  fi
  if [[ -n "$TEST_GIT_DIR" ]]; then
    rm -rf "$TEST_GIT_DIR" >/dev/null 2>&1 || true
  fi
  cleanup_ephemeral_builds
}
trap cleanup EXIT

for i in $(seq 1 60); do
  if ! kill -0 "$DAEMON_PID" 2>/dev/null; then
    echo "spec-gates daemon exited before health on ${FOCUSA_BIND}" >&2
    tail -60 "$DAEMON_LOG" >&2
    exit 1
  fi
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
  curl -sS --max-time 2 -X POST "${BASE_URL}/v1/session/close" \
    -H "Content-Type: application/json" \
    -d '{"reason":"ci-spec-gate-isolation"}' >/dev/null 2>&1 || true
  "$@"
}

run_gate ./tests/focusa_toggle_persistence_test.sh
run_gate ./tests/tool_contract_test.sh
run_gate ./tests/command_write_contract_test.sh
run_gate ./tests/trace_dimensions_test.sh
run_gate ./tests/pi_extension_contract_test.sh
run_gate bash ./tests/spec142_workflow_dependency_onboarding_static_test.sh
run_gate env FOCUSA_DAEMON_BIN="$DAEMON_BIN" python3 ./tests/spec135_task_materialization_e2e_test.py
run_gate env FOCUSA_DAEMON_BIN="$DAEMON_BIN" python3 ./tests/spec135_work_rail_e2e_test.py
run_gate bash ./tests/spec135_mission_canvas_naming_and_multiplexing_static_test.sh
run_gate python3 ./tests/spec135_m1_workspace_shell_test.py
run_gate python3 ./tests/spec135_m2_pi_work_rail_test.py
run_gate env FOCUSA_DAEMON_BIN="$DAEMON_BIN" python3 ./tests/spec135_mission_canvas_surfaces_e2e_test.py
run_gate python3 ./tests/spec135_m4_surface_bindings_static_test.py
run_gate env FOCUSA_DAEMON_BIN="$DAEMON_BIN" python3 ./tests/spec135_m4_surface_bindings_e2e_test.py
run_gate python3 ./tests/spec135_m5_browser_context_isolation_test.py
run_gate env FOCUSA_DAEMON_BIN="$DAEMON_BIN" python3 ./tests/spec135_m5_browser_context_isolation_e2e_test.py
run_gate python3 ./tests/spec135_mission_canvas_portability_test.py
run_gate python3 ./tests/spec135_m6_canvas_resume_test.py
run_gate env FOCUSA_DAEMON_BIN="$DAEMON_BIN" python3 ./tests/spec135_m6_canvas_resume_e2e_test.py
run_gate python3 ./tests/spec135_m7_accessibility_recovery_test.py
run_gate python3 ./tests/spec135_u3_browser_eval_suite_test.py
run_gate python3 ./tests/spec135_alpha5_alpha6_closure_test.py
run_gate python3 ./tests/spec135_u4_u5_usability_friction_test.py
run_gate python3 ./tests/spec135_u6_adaptive_ui_test.py
run_gate python3 ./tests/spec135_v1_v6_domain_projection_test.py
run_gate ./tests/spec130_bounded_persistence_test.sh
run_gate ./tests/spec130_native_session_pressure_test.sh
run_gate ./tests/spec130_auto_compaction_test.sh
run_gate ./tests/spec104_scoped_state_foundation_test.sh
run_gate ./tests/focusa_decide_scope_recovery_test.sh
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
run_gate bash ./tests/spec96_menubar_mission_canvas_foundation_static_test.sh
run_gate bash ./tests/spec135_mission_canvas_naming_and_multiplexing_static_test.sh
run_gate bash ./tests/spec135h_implementation_acceleration_static_test.sh
run_gate bash ./tests/spec135i_real_time_generated_ui_static_test.sh
run_gate bash ./tests/spec135j_core_api_runtime_reuse_static_test.sh
run_gate bash ./tests/spec135k_uxp_ufi_generated_ui_static_test.sh
run_gate bash ./tests/spec135_generated_ui_core_integration_audit_static_test.sh
run_gate bash ./tests/spec135_delivery_contract_regression_static_test.sh
run_gate bash ./tests/spec128_menubar_updater_static_test.sh
run_gate bash ./tests/phone_bridge_public_url_static_test.sh
run_gate bash ./tests/phone_bridge_automatic_callback_static_test.sh
run_gate bash ./tests/release_notes_workflow_static_test.sh
run_gate python3 ./tests/release_tag_template_static_test.py
run_gate bash ./tests/release_proof_status_route_static_test.sh
run_gate bash ./tests/build_cruft_cleanup_test.sh
run_gate bash ./tests/spec80_impl_parquet_export_support_test.sh
run_gate bash ./tests/spec96_trajectory_context_tool_docs_static_test.sh
run_gate bash ./tests/spec96_static_false_positive_guard_test.sh

for gate in ./tests/spec133_*static_test.py; do
  run_gate python3 "$gate"
done
for fixture_mode in harness subprocess child-leak prompt-wait output-flood model-mismatch retry-failure isolated-git entitlement runner-disconnect; do
  run_gate python3 ./tests/spec133_fault_fixture.py "$fixture_mode" --lines 32
done
python3 ./tests/run_spec137_138_full_conformance_gates.py
run_gate python3 ./tests/spec137a_138a_144_documentation_closure_gate.py
run_gate python3 ./tests/bead_closure_evidence_gate.py
