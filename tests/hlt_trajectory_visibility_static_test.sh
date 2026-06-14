#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOL_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_trajectory_view.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
TURNS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for label in 'HLT:' 'MLG:' 'STG:' 'Waypoints:' 'Workpoint:' 'Scope:' 'Status:' 'Canonical/advisory/degraded:' 'Evidence refs:'; do
  rg -n "$label" "$TOOL_DOC" "$SPEC" >/dev/null || fail "trajectory render requirement missing $label"
done
pass "trajectory docs expose required agent-facing render labels"

for phrase in 'degraded placeholder' 'Prior-project/foreign trajectory fallback is advisory only' 'Workpoint and Trajectory disagreement requires `verify_first`'; do
  rg -n "$phrase" "$TOOL_DOC" >/dev/null || fail "trajectory doc missing warning phrase: $phrase"
done
pass "trajectory docs warn on generic/foreign/disagreement states"

rg -n 'HLT=|MLG=|STG=|TRAJECTORY_CONTEXT' "$TURNS" >/dev/null || fail "Pi turns do not render HLT/MLG/STG trajectory context"
rg -n 'fallback_prior_project_trajectory|fallback_source_continuity_id|long_term_goal|mid_level_goal|short_term_goal|waypoints' "$TOOLS" "$STATE" >/dev/null || fail "Pi trajectory state lacks ladder/fallback fields"
pass "Pi surfaces preserve trajectory ladder and fallback warning fields"

echo "hlt trajectory visibility static test: PASS"
