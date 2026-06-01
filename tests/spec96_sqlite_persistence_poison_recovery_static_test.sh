#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SQLITE="${ROOT_DIR}/crates/focusa-core/src/runtime/persistence_sqlite.rs"

if rg -n 'sqlite conn mutex poisoned' "$SQLITE" >/dev/null; then
  echo "✗ FAIL: SQLite persistence still panics on poisoned connection mutex" >&2
  exit 1
fi

count=$(rg -n 'unwrap_or_else\(\|poisoned\| poisoned\.into_inner\(\)\)' "$SQLITE" | wc -l | tr -d ' ')
if [ "$count" -ge 10 ]; then
  echo "✓ PASS: SQLite persistence locks recover from poison instead of panicking"
else
  echo "✗ FAIL: SQLite persistence poison recovery not applied broadly" >&2
  exit 1
fi

echo "SPEC96 SQLite persistence poison recovery static test: PASS"
