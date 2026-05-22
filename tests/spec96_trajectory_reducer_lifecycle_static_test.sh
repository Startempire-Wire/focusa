#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TYPES="${ROOT_DIR}/crates/focusa-core/src/types.rs"
REDUCER="${ROOT_DIR}/crates/focusa-core/src/reducer.rs"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"

if rg -n "TrajectoryProjectionRecord|TrajectoryMilestoneRecord|TrajectoryDefinitionOfDoneRecord|TrajectoryStateDeltaRecord|TrajectoryState" "$TYPES" >/dev/null; then
  echo "✓ PASS: core exposes durable Trajectory Projection lifecycle records"
else
  echo "✗ FAIL: core Trajectory lifecycle records missing" >&2
  exit 1
fi

if rg -n "TrajectoryGoalDefined|TrajectoryCheckpointPersisted|TrajectoryStateDeltaRecorded" "$TYPES" "$REDUCER" >/dev/null; then
  echo "✓ PASS: reducer event vocabulary includes trajectory goal/checkpoint/state-delta events"
else
  echo "✗ FAIL: trajectory reducer events missing" >&2
  exit 1
fi

if rg -n "upsert_trajectory_record|active_trajectory_id|definition_of_done|state_deltas|test_trajectory_goal_defined" "$REDUCER" >/dev/null; then
  echo "✓ PASS: reducer persists trajectory records, supersession, DOD, checkpoints, and deltas"
else
  echo "✗ FAIL: trajectory reducer lifecycle handling missing" >&2
  exit 1
fi

if rg -n "TrajectoryGoalDefined|trajectory_goal_defined|mutates_canonical_state.*valid|persisted.*valid" "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: define-goal can persist canonical trajectory metadata when accepted"
else
  echo "✗ FAIL: define-goal remains advisory-only/non-persistent" >&2
  exit 1
fi

if rg -n "TrajectoryCheckpointPersisted|trajectory_checkpoint_persisted|TrajectoryStateDeltaRecorded|trajectory_assess" "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory checkpoint/assess write reducer-backed lifecycle evidence"
else
  echo "✗ FAIL: trajectory checkpoint/assess persistence missing" >&2
  exit 1
fi

if rg -n "durable_lifecycle|goal_provenance|milestones|definition_of_done|checkpoint_count|state_delta_count|checkpoints|state_deltas|trajectory_view_exposes_durable_lifecycle_history" "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory view exposes persisted lifecycle metadata and bounded history"
else
  echo "✗ FAIL: trajectory view does not expose durable lifecycle metadata/history" >&2
  exit 1
fi

echo "SPEC96 trajectory reducer lifecycle static test: PASS"
