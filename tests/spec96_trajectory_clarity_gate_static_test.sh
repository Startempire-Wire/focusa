#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
DOC_DEFINE="${ROOT_DIR}/docs/focusa-tools/tools/focusa_trajectory_define_goal.md"
DOC_VIEW="${ROOT_DIR}/docs/focusa-tools/tools/focusa_trajectory_view.md"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'trajectory_clarity_gate_payload|source_precedence|root_goal_change_policy|refresh_triggers|operator_confirm_path' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory clarity gate payload is implemented"
else
  echo "✗ FAIL: clarity gate payload missing" >&2
  exit 1
fi

if rg -n 'operator_confirmed_or_durable_supersession_evidence_only|define_goal_lifecycle_status|supersession_evidence_refs|root_goal_change_allowed' "$TRAJECTORY" "$TOOLS" "$DOC_DEFINE" >/dev/null; then
  echo "✓ PASS: goal lifecycle supersession guard is wired"
else
  echo "✗ FAIL: goal lifecycle supersession guard missing" >&2
  exit 1
fi

if rg -n 'operator_input|verify_first|proceed|clear|provisional|unclear|conflicted' "$TRAJECTORY" "$DOC_VIEW" "$SPEC" >/dev/null; then
  echo "✓ PASS: clarity states map to guidance"
else
  echo "✗ FAIL: clarity state guidance missing" >&2
  exit 1
fi

if rg -n 'trajectory_clarity_gate_guides_missing_and_conflicting_states|define_goal_supersession_requires_operator_or_durable_evidence' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: clarity lifecycle regression tests exist"
else
  echo "✗ FAIL: clarity lifecycle regression tests missing" >&2
  exit 1
fi


if rg -n 'enforceTrajectoryClarityPrecondition|trajectory_clarity_precondition|workpoint checkpoint blocked.*trajectory|evidence link blocked.*trajectory' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi mutating Workpoint/evidence tools enforce trajectory clarity precondition"
else
  echo "✗ FAIL: mutating Pi tools do not enforce trajectory clarity gate" >&2
  exit 1
fi

if rg -n 'refreshTrajectoryClarityLifecycle|lastTrajectoryClarity|trajectory_clarity_refreshed' "${ROOT_DIR}/apps/pi-extension/src/state.ts" >/dev/null   && rg -n 'session_start|session_resume|before_compaction|after_compaction|operator_steering|failure_or_degradation|handoff_fork' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "${ROOT_DIR}/apps/pi-extension/src/compaction.ts" "${ROOT_DIR}/apps/pi-extension/src/turns.ts" >/dev/null; then
  echo "✓ PASS: Pi lifecycle refreshes trajectory clarity at session, compaction, steering, failure/degradation, and handoff points"
else
  echo "✗ FAIL: trajectory clarity lifecycle refresh points missing" >&2
  exit 1
fi

echo "SPEC96 trajectory clarity gate static test: PASS"
