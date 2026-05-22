#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"
TURNS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
DOC_VIEW="${ROOT_DIR}/docs/focusa-tools/tools/focusa_trajectory_view.md"
DOC_INDEX="${ROOT_DIR}/docs/focusa-tools/trajectory.md"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'mid_level_goal|low_level_goal|similarity_group|high_level_group_key|mid_level_group_key|low_level_group_key' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory API exposes high/mid/low advisory grouping"
else
  echo "✗ FAIL: trajectory hierarchy fields missing" >&2
  exit 1
fi

if rg -n 'authority_boundary.*project_root_plus_continuity_id|must_not_merge_sessions|continuity_id.*mismatch|session_id_is_temporal_metadata' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory API preserves project_root+continuity_id authority"
else
  echo "✗ FAIL: trajectory authority boundary missing" >&2
  exit 1
fi

if rg -n 'TRAJECTORY_SIMILARITY_GROUP|authority=project_root\+continuity_id|must_not_merge_sessions=true' "$TURNS" >/dev/null; then
  echo "✓ PASS: Focus Slice surfaces advisory grouping without authority merge"
else
  echo "✗ FAIL: Focus Slice hierarchy grouping missing" >&2
  exit 1
fi

if rg -n 'continuity_id|S\.continuityId' "$TOOLS" >/dev/null; then
  echo "✓ PASS: trajectory tool calls carry logical continuity id"
else
  echo "✗ FAIL: trajectory tool continuity id missing" >&2
  exit 1
fi

if rg -n 'high/mid/low|must_not_merge_sessions|Same high-level|project_root \+ continuity_id|session_id.*temporal metadata' "$DOC_VIEW" "$DOC_INDEX" "$SPEC" >/dev/null; then
  echo "✓ PASS: docs state hierarchy grouping and no-confusion rule"
else
  echo "✗ FAIL: hierarchy/no-confusion docs missing" >&2
  exit 1
fi

if rg -n 'trajectory_similarity_grouping_is_advisory_not_authority|trajectory_view_degrades_on_continuity_mismatch_not_session_metadata_change' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: regression tests cover same-high-level distinct sessions"
else
  echo "✗ FAIL: trajectory hierarchy regression tests missing" >&2
  exit 1
fi

echo "SPEC96 trajectory hierarchy grouping static test: PASS"
