#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'focus_trajectory_sync|focus_state_current_focus|trajectory_short_term_goal|projection_only' "$TRAJECTORY" >/dev/null || fail "Trajectory view lacks bidirectional current_focus/short_term_goal projection"
pass "Trajectory view exposes focus_trajectory_sync projection"

rg -n 'trajectory_view_syncs_focus_current_focus_and_short_term_goal_projection' "$TRAJECTORY" >/dev/null || fail "Rust regression test for focus/trajectory short-term sync missing"
pass "Rust regression test covers bidirectional projection"

rg -n 'focus_trajectory_sync|S\.lastTrajectoryClarity|current_focus_source: "trajectory_short_term_goal"' "$TOOLS" >/dev/null || fail "Pi trajectory view tool does not persist short_term_goal as current-focus fallback metadata"
pass "Pi trajectory tool preserves focus_trajectory_sync metadata"

rg -n 'lastTrajectoryClarity\?\.short_term_goal' "$STATE" >/dev/null || fail "Pi effective Focus snapshot does not fall back to trajectory short_term_goal"
pass "Pi effective Focus snapshot falls back to trajectory short_term_goal"

echo "SPEC96 Focus/Trajectory short-term sync static test: PASS"
