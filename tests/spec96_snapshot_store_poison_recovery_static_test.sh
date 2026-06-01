#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SNAPSHOTS="${ROOT_DIR}/crates/focusa-api/src/routes/snapshots.rs"

if rg -n 'snapshot store poisoned' "$SNAPSHOTS" >/dev/null; then
  echo "✗ FAIL: snapshot routes still panic on poisoned store lock" >&2
  exit 1
fi

count=$(rg -n 'unwrap_or_else\(\|poisoned\| poisoned\.into_inner\(\)\)' "$SNAPSHOTS" | wc -l | tr -d ' ')
if [ "$count" -ge 4 ]; then
  echo "✓ PASS: snapshot store locks recover from poison instead of panicking"
else
  echo "✗ FAIL: snapshot store poison recovery not applied broadly" >&2
  exit 1
fi

echo "SPEC96 Snapshot store poison recovery static test: PASS"
