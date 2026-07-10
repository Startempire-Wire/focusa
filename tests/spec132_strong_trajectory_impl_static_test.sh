#!/usr/bin/env bash
# Phase 4 strong Trajectory implementation guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$ROUTE" ] || fail "trajectory route missing"

for symbol in \
  'trajectory_text_is_vague' \
  'trajectory_goal_quality_errors' \
  'trajectory_goal_recovery_hint' \
  'vague_or_unverifiable_trajectory_goal' \
  'long_term_goal_vague_or_unverifiable' \
  'desired_end_state_vague_or_unverifiable'; do
  grep -q "$symbol" "$ROUTE" || fail "missing define-goal strength symbol: $symbol"
done

for field in \
  '"strong_trajectory"' \
  '"gap_description"' \
  '"next_tool"' \
  '"next_command"' \
  '"proof_needed"' \
  '"workpoint_relation"'; do
  grep -q "$field" "$ROUTE" || fail "missing strong trajectory field: $field"
done

for behavior in \
  'workpoint_candidate_missing_target_ref' \
  'workpoint_candidate_missing_action_type' \
  'proposal_blocked_until_concrete_target_and_action' \
  'repair_trajectory_definition' \
  'current_state_or_workpoint_evidence'; do
  grep -q "$behavior" "$ROUTE" || fail "missing strong behavior: $behavior"
done

if grep -A8 'let missing_facts = \[' "$ROUTE" | grep -q 'next_workpoint'; then
  fail "next_workpoint must not be a trajectory-definition missing fact"
fi

pass "Strong Trajectory implementation static guard passed"
