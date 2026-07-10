#!/usr/bin/env bash
# BAD-007: empty trajectory guidance must include actionable define + verify steps in API and Pi wrapper.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRAJECTORY="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$TRAJECTORY" ] || fail "trajectory route missing"
[ -f "$TOOLS" ] || fail "Pi tools missing"

for token in \
  'how_to_verify_empty_trajectory_repair' \
  'define_goal_command' \
  'verify_command' \
  'success_condition' \
  'required_check' \
  'strong_trajectory.next_tool'; do
  grep -q "$token" "$TRAJECTORY" || fail "API empty-trajectory guidance missing: $token"
done

for token in \
  'emptyTrajectoryHowTo' \
  'how_to_verify:' \
  'define_goal=' \
  'verify=' \
  'success=' \
  'how_to_verify_empty_trajectory_repair'; do
  grep -q "$token" "$TOOLS" || fail "Pi wrapper empty-trajectory guidance missing: $token"
done

pass "BAD-007 empty trajectory guidance includes define + verify steps"
