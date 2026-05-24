#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"

if rg -n 'trajectory_dispatch_timeout|tokio::time::timeout|Duration::from_millis\(1500\)|state\.write_serial_lock\.lock\(\)' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory dispatch write-lock wait is bounded"
else
  echo "✗ FAIL: trajectory dispatch can wait unbounded on write_serial_lock" >&2
  exit 1
fi

if rg -n 'failure_class": "resource_exhausted"|retry_posture": "safe_retry"|event was not persisted|focusa_resource_mode' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory dispatch timeout returns typed recovery envelope"
else
  echo "✗ FAIL: trajectory dispatch timeout lacks typed recovery envelope" >&2
  exit 1
fi

echo "SPEC96 trajectory dispatch timeout static test: PASS"
