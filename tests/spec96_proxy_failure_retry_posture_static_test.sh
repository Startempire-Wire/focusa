#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROXY="${ROOT_DIR}/crates/focusa-api/src/routes/proxy.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found" \| "permission_denied"\)|do_not_retry_unchanged' "$PROXY" >/dev/null; then
  echo "✓ PASS: proxy failure helper derives non-retry posture for validation/not_found/permission_denied"
else
  echo "✗ FAIL: proxy failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$PROXY" >/dev/null; then
  echo "✗ FAIL: proxy failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Proxy failure retry posture static test: PASS"
