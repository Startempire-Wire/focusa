#!/usr/bin/env bash
# Root-cause guard for GH #4 / focusa-gh-4-perpetua-scope.
# This is NOT a Perpetua-only guard. It verifies core project directory detection
# for parent projects, child folders, and subdomain/alias hints before authority.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJ_ROUTES="$ROOT_DIR/crates/focusa-api/src/routes/project.rs"
PI_STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"
DOC="$ROOT_DIR/docs/current/PROJECT_DIRECTORY_DETECTION.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

for needle in \
  'normalize_project_hint' \
  'marker_hint_values' \
  'marker_matches_project_hint' \
  'project_hint_candidates' \
  'project_directory_search_roots' \
  'find_project_marker_for_hint' \
  'project_directory_detector' \
  'current_ask_or_alias_domain' \
  'select_canonical_project_root' \
  'directory_detection_priority' \
  'core directory detection resolves parent/child/subdomain project roots before Workpoint/Trajectory authority'; do
  rg -n -F "$needle" "$PROJ_ROUTES" >/dev/null || fail "project.rs missing core detector marker: $needle"
done
pass "API ProjectIdentity has reusable directory detector and priority selector"

for needle in \
  'candidate_project_root' \
  'expected_project_root' \
  'alias_scope_matches'; do
  rg -n -F "$needle" "$PROJ_ROUTES" >/dev/null || fail "identity_name_matches missing root-aware alias marker: $needle"
done
pass "identity alias matching remains project-root scoped"

for needle in \
  'projectAliasesForText' \
  'firstLabel' \
  'normalizeProjectHint' \
  'markerHintValues' \
  'markerMatchesProjectHint' \
  'Core directory detection: recursive bounded marker search' \
  'FOCUSA_PROJECT_SEARCH_DIRS' \
  'directory_detector'; do
  rg -n -F "$needle" "$PI_STATE" >/dev/null || fail "Pi state missing directory detector marker: $needle"
done
pass "Pi extension has recursive marker/domain detector, not one-project special-case logic"

if rg -n 'perpetua.*hardcode|/home/focusadev/perpetua|/home/wirebot/focusa' "$PROJ_ROUTES" "$PI_STATE" >/dev/null; then
  fail "core detector contains project-specific hardcoded root"
fi
pass "core detector avoids Perpetua/Focusa hardcoded root workaround"

for needle in \
  'Project Directory Detection' \
  'parent projects, child repositories, subdomain apps, and folder-based projects' \
  'focusa_workpoint_resume' \
  'focusa_workpoint_checkpoint' \
  'focusa_trajectory_view' \
  'Pi Focus Slice' \
  'No project-specific hardcoded root workaround'; do
  rg -n -F "$needle" "$DOC" >/dev/null || fail "directory detection doc missing marker: $needle"
done
pass "directory detection authority contract documents all Focusa surfaces"

echo "GH4/core directory detection static test: PASS"
