#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found" \| "scope_mismatch" \| "permission_denied"\)|do_not_retry_unchanged' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint failure helper derives non-retry posture for validation/not_found/scope/permission states"
else
  echo "✗ FAIL: Workpoint failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$WORKPOINT" >/dev/null; then
  echo "✗ FAIL: Workpoint failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Workpoint failure retry posture static test: PASS"
