#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"

if rg -n 'no contention at daemon construction|\.try_write\(\)\s*\.expect\(' "$DAEMON" >/dev/null; then
  echo "✗ FAIL: daemon construction still panics on shared-state write lock contention" >&2
  exit 1
fi

if rg -n 'shared Focusa state write lock unavailable during daemon construction|try_write\(\)\.map_err' "$DAEMON" >/dev/null; then
  echo "✓ PASS: daemon construction returns typed lock contention error"
else
  echo "✗ FAIL: daemon construction typed lock error missing" >&2
  exit 1
fi

echo "SPEC96 Daemon construction no-expect static test: PASS"
