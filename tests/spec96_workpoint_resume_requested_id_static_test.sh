#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
API="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n 'requested_workpoint_id\s*=\s*req\.workpoint_id|"requested_workpoint_id"\s*:\s*requested_workpoint_id' "$API" >/dev/null; then
  echo "✓ PASS: Workpoint resume not_found envelope preserves requested_workpoint_id"
else
  echo "✗ FAIL: Workpoint resume not_found envelope drops requested_workpoint_id" >&2
  exit 1
fi

if rg -n 'body\?\.workpoint_id \|\| body\?\.active_workpoint_id \|\| body\?\.requested_workpoint_id \|\| "none"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi Workpoint resume summary uses requested_workpoint_id fallback"
else
  echo "✗ FAIL: Pi Workpoint resume summary still reports id=none for explicit not_found requests" >&2
  exit 1
fi

echo "SPEC96 Workpoint resume requested id recovery static test: PASS"
