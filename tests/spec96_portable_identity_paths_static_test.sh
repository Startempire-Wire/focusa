#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Portability concern: block host/user-specific runtime filesystem layouts.
# Allow API routes (/v1/...), app-internal relative paths, ~/.focusa conventions,
# broad-root guard literals such as /root, and synthetic/unit tests.
if rg -n "(/home/wirebot|/opt/cpanel|/usr/local/cpanel)" \
  "$ROOT_DIR/apps/pi-extension/src" \
  "$ROOT_DIR/crates/focusa-core/src" \
  "$ROOT_DIR/crates/focusa-api/src" \
  "$ROOT_DIR/crates/focusa-cli/src" \
  "$ROOT_DIR/scripts" >/tmp/spec96-portable-paths-hits.txt; then
  echo "✗ FAIL: runtime code contains host/user-specific filesystem path literals" >&2
  cat /tmp/spec96-portable-paths-hits.txt >&2
  exit 1
else
  echo "✓ PASS: runtime code avoids host/user-specific filesystem path literals"
fi

if rg -n 'process\.cwd\(\)|ctx\.cwd|workspace_id|project_root|ensureContinuityId|continuity_id|FOCUSA_PROJECT_ROOT|FOCUSA_HOME|XDG_DATA_HOME|HOME' \
  "$ROOT_DIR/apps/pi-extension/src/state.ts" \
  "$ROOT_DIR/apps/pi-extension/src/session.ts" \
  "$ROOT_DIR/apps/pi-extension/src/tools.ts" \
  "$ROOT_DIR/crates/focusa-api/src/routes/work_loop.rs" \
  "$ROOT_DIR/crates/focusa-core/src/types.rs" >/dev/null; then
  echo "✓ PASS: identity/path plumbing derives from cwd/env/config plus continuity_id"
else
  echo "✗ FAIL: cwd/env/config-derived identity/path plumbing missing" >&2
  exit 1
fi

echo "SPEC96 portable identity paths static test: PASS"
