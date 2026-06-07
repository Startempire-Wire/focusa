#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TMP="${TMPDIR:-/tmp}/spec102-doctor-readiness"
mkdir -p "$TMP"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

curl -fsS --max-time 15 "$BASE/v1/doctor" > "$TMP/api-doctor.json"
focusa --json doctor > "$TMP/cli-doctor.json" || true

for file in api-doctor.json cli-doctor.json; do
  jq -e '
    .readiness_categories.runtime_readiness.status != null
    and .readiness_categories.project_scope_readiness.status != null
    and .readiness_categories.workpoint_readiness.status != null
    and .readiness_categories.trajectory_readiness.status != null
    and .readiness_categories.focus_state_readiness.status != null
    and .readiness_categories.source_build_readiness.status != null
    and .readiness_categories.release_readiness.status != null
    and .readiness_categories.telemetry_readiness.status != null
    and .readiness_categories.ui_browser_readiness.status != null
  ' "$TMP/$file" >/dev/null || fail "$file missing doctor readiness categories"
done
pass "API and CLI expose readiness categories"

jq -e '
  .readiness_categories.runtime_readiness.status == "ready"
  and (.status == "ok" or .status == "completed" or .status == "warn")
' "$TMP/api-doctor.json" >/dev/null || fail "API runtime readiness should be ready without source-build headline block"
pass "API runtime readiness not masked by source-build plane"

jq -e '
  .readiness_categories.runtime_readiness.status != "blocked"
  and .readiness_categories.source_build_readiness.status != null
  and .readiness_categories.release_readiness.status != null
' "$TMP/cli-doctor.json" >/dev/null || fail "CLI doctor must split source/release readiness from runtime readiness"
pass "CLI doctor splits runtime/source/release planes"

echo "SPEC102 doctor readiness categories test: PASS"
