#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"

if rg -n 'active_workpoint_for_scope\(|expected_project_root|expected_continuity_id' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: evidence link selects scoped active Workpoint when session_identity provides scope"
else
  echo "✗ FAIL: evidence link does not use scoped active Workpoint selection" >&2
  exit 1
fi

if rg -n '\.or_else\(\|\| active_workpoint\(&focusa\)\)' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: evidence link keeps global-active fallback after scoped lookup"
else
  echo "✗ FAIL: evidence link scoped lookup fallback missing" >&2
  exit 1
fi

echo "SPEC96 Evidence link scoped active Workpoint static test: PASS"
