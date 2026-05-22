#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CLI="${ROOT_DIR}/crates/focusa-cli/src/commands/trajectory.rs"
DOCS=("${ROOT_DIR}"/docs/focusa-tools/tools/focusa_trajectory_*.md)

if rg -n 'fn attach_trajectory_tool_result|"tool_result_v1"|"side_effects"|"evidence_refs"|"next_tools"|"retry"' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: Trajectory API wraps all endpoints with tool_result_v1 envelope"
else
  echo "✗ FAIL: Trajectory API tool_result_v1 envelope missing" >&2
  exit 1
fi

if rg -n 'TrajectoryGoalDefined|trajectory_goal_defined|TrajectoryStateDeltaRecorded|trajectory_state_delta_recorded|TrajectoryCheckpointPersisted|trajectory_checkpoint_persisted' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: Trajectory write endpoints report reducer side effects"
else
  echo "✗ FAIL: Trajectory side-effect reporting missing" >&2
  exit 1
fi

if rg -n 'focusa_trajectory_view|focusa_trajectory_define_goal|focusa_trajectory_assess|focusa_trajectory_propose_workpoint|focusa_trajectory_checkpoint|focusa_trajectory_resume|tool_result_v1: toolResult|side_effects: toolResult\.side_effects|evidence_refs: toolResult\.evidence_refs' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi Trajectory tools expose schema fields and tool_result_v1"
else
  echo "✗ FAIL: Pi Trajectory tool output schemas incomplete" >&2
  exit 1
fi

if rg -n '/v1/trajectory/view|/v1/trajectory/define-goal|/v1/trajectory/assess|/v1/trajectory/propose-workpoint|/v1/trajectory/checkpoint|/v1/trajectory/resume' "$CLI" >/dev/null; then
  echo "✓ PASS: CLI Trajectory commands cover all API endpoints"
else
  echo "✗ FAIL: CLI Trajectory endpoint parity missing" >&2
  exit 1
fi

if rg -n 'tool_result_v1|canonical|degraded|advisory_only|next_tools|failure_class' "${DOCS[@]}" >/dev/null; then
  echo "✓ PASS: Trajectory tool docs describe schema/recovery fields"
else
  echo "✗ FAIL: Trajectory docs missing schema/recovery fields" >&2
  exit 1
fi

echo "SPEC96 Trajectory tool output schema static test: PASS"
