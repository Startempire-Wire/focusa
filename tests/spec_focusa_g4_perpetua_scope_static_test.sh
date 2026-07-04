#!/usr/bin/env bash
# spec_focusa_g4_perpetua_scope_static_test.sh
#
# Static guard for focusa-gh-4-perpetua-scope + GH #4:
# Focusa scope resolver conflates Perpetua sub-project with Focusa root.
#
# Acceptance: when project_root is /home/focusadev/perpetua, the daemon's
# project identity returns "Perpetua" not "Focusa"; when project_root is
# /home/wirebot/focusa, identity is "Focusa". Sub-projects (focusa.dev
# subdomains) do not inherit parent alias 'focusa'.
#
# This test verifies the static structural pieces; the live behavioral
# test is the same-shape `curl /v1/project/identity?project_root=...` check
# which currently returns the right identity but renders a scope conflict
# in Workpoint resume when persisted vs current mismatch.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJ_ROUTES="$ROOT_DIR/crates/focusa-api/src/routes/project.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# project.rs must have identity_name_matches and persist/restore
grep -q 'fn identity_name_matches' "$PROJ_ROUTES" \
  || fail "project.rs missing fn identity_name_matches"
pass "project.rs has fn identity_name_matches (alias resolution)"

# Per the issue: aliases must NOT match across sub-projects.
# The fix surface: identity_name_matches should reject a match when the
# canonical_name and the candidate's project_id disagree at the project_root
# level. (Static verification: the function must consult the candidate's
# project_root, not just the alias list.)
grep -n "fn identity_name_matches" "$PROJ_ROUTES" | head -1
# Note: the current implementation only matches on canonical_name / project_id /
# aliases, NOT on project_root. A fix may need to extend the function signature.

# Verify marker parsing exposes aliases field (so we can use it for
# sub-project disambiguation)
grep -q "marker_string_array" "$PROJ_ROUTES" \
  || fail "project.rs missing marker_string_array helper"
pass "project.rs exposes marker_string_array helper (reads aliases from .focusa-project.json)"

# Verify root-aware alias disambiguation is present.
grep -q "candidate_project_root" "$PROJ_ROUTES"   || fail "identity_name_matches must accept candidate_project_root"
grep -q "expected_project_root" "$PROJ_ROUTES"   || fail "identity_name_matches must accept expected_project_root"
grep -q "alias_scope_matches" "$PROJ_ROUTES"   || fail "identity_name_matches must scope alias matches by project_root"
pass "project.rs scopes alias matches by project_root"

echo "✓ All focusa-gh-4-perpetua-scope static structural checks passed"
