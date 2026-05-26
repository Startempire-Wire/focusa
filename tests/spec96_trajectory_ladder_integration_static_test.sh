#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'mid_level_goal|waypoints|trajectory_ladder|HLT -> MLG -> STG -> Waypoints' "$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs" >/dev/null || fail "trajectory API does not expose full ladder"
pass "trajectory API exposes HLT/MLG/STG/Waypoints ladder"

rg -n 'mid_level_goal|waypoint|ladder: HLT' "$ROOT_DIR/crates/focusa-cli/src/commands/trajectory.rs" >/dev/null || fail "trajectory CLI lacks MLG/Waypoint flags or ladder output"
pass "trajectory CLI accepts and renders ladder fields"

rg -n 'mid_level_goal|waypoints|HLT=.*MLG=.*STG=|TRAJECTORY_LADDER' "$ROOT_DIR/apps/pi-extension/src" >/dev/null || fail "Pi extension lacks ladder fields/focus slice"
pass "Pi extension carries ladder through tools and focus slice"

rg -n 'trajectory_hlt|trajectory_mlg|trajectory_stg|trajectory_waypoint|derive_mlg_from_hlt|derive_stg_from_mlg|offer_waypoints|derived_from_hlt|marks_waypoint_for' "$ROOT_DIR/crates/focusa-api/src/routes/ontology.rs" >/dev/null || fail "ontology lacks trajectory ladder classes/actions/links"
pass "ontology includes trajectory ladder vocabulary"

rg -n 'HLT.*MLG.*STG.*Waypoints|operator.*actively offering' "$ROOT_DIR/docs/00-glossary.md" "$ROOT_DIR/README.md" "$ROOT_DIR/docs/current/TRAJECTORY_GTM_AND_GAPS.md" >/dev/null || fail "public docs lack trajectory ladder doctrine"
pass "public docs carry trajectory ladder doctrine"

echo "SPEC96 trajectory ladder integration static test: PASS"
