#!/usr/bin/env bash
# Phase 5 strong Trajectory test coverage guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$ROUTE" ] || fail "trajectory route missing"

for test_name in \
  'define_goal_rejects_vague_unverifiable_goal' \
  'define_goal_returns_advisory_candidate_without_canonical_mutation' \
  'trajectory_view_strong_guidance_and_candidate_follow_gap_state' \
  'assess_returns_exact_next_call_guidance' \
  'propose_workpoint_blocks_candidate_without_concrete_target_and_action' \
  'propose_workpoint_candidate_carries_handoff_guards'; do
  grep -q "fn $test_name" "$ROUTE" || fail "missing strong trajectory unit test: $test_name"
done

for assertion in \
  'vague_or_unverifiable_trajectory_goal' \
  'focusa_trajectory_propose_workpoint' \
  'gap_description' \
  'Next: call' \
  'workpoint_candidate_missing_target_ref' \
  'next_workpoint_candidate"].is_null' \
  'Some("proposed")'; do
  grep -q "$assertion" "$ROUTE" || fail "missing phase-5 assertion token: $assertion"
done

pass "Strong Trajectory phase-5 test coverage static guard passed"
