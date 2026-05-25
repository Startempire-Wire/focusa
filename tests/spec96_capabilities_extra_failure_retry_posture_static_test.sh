#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CAP_EXTRA="${ROOT_DIR}/crates/focusa-api/src/routes/capabilities_extra.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found" \| "permission_denied"\)|do_not_retry_unchanged' "$CAP_EXTRA" >/dev/null; then
  echo "✓ PASS: extra capabilities failure helper derives non-retry posture for validation/not_found/permission_denied"
else
  echo "✗ FAIL: extra capabilities failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$CAP_EXTRA" >/dev/null; then
  echo "✗ FAIL: extra capabilities failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Extra capabilities failure retry posture static test: PASS"
