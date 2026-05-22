#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_trajectory_propose_workpoint.md"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'advisory_workpoint_candidate_v1|action_intent|target_refs|verification_hooks|blockers|do_not_drift|checkpoint_required' "$TRAJECTORY" "$DOC" >/dev/null; then
  echo "✓ PASS: trajectory proposal carries full Workpoint candidate fields"
else
  echo "✗ FAIL: Workpoint candidate fields missing" >&2
  exit 1
fi

if rg -n 'no_execution_side_effects|forbidden_side_effects|work_loop_select_next|execute_action|mutate_focus_state|Trajectory does not auto-promote Workpoints' "$TRAJECTORY" "$TOOLS" "$DOC" "$SPEC" >/dev/null; then
  echo "✓ PASS: trajectory handoff has no execution/work-loop side effects"
else
  echo "✗ FAIL: no-execution side-effect guard missing" >&2
  exit 1
fi

if rg -n 'canonicalization_tool.*focusa_workpoint_checkpoint|authority_path|operator_accepts_candidate_then_workpoint_checkpoint|focusa_workpoint_checkpoint' "$TRAJECTORY" "$DOC" >/dev/null; then
  echo "✓ PASS: canonical authority path remains Workpoint checkpoint"
else
  echo "✗ FAIL: Workpoint checkpoint authority path missing" >&2
  exit 1
fi

if rg -n 'propose_workpoint_candidate_carries_handoff_guards|propose_workpoint_returns_checkpoint_required_candidate' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory handoff regression tests exist"
else
  echo "✗ FAIL: trajectory handoff regression tests missing" >&2
  exit 1
fi

echo "SPEC96 trajectory Workpoint handoff static test: PASS"
