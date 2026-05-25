#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found" \| "scope_mismatch"\)|do_not_retry_unchanged' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory failure helper derives non-retry posture for validation/not_found/scope_mismatch"
else
  echo "✗ FAIL: trajectory failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$TRAJECTORY" >/dev/null; then
  echo "✗ FAIL: trajectory failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Trajectory failure retry posture static test: PASS"
