#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ATTACHMENTS="${ROOT_DIR}/crates/focusa-api/src/routes/attachments.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found"\)|do_not_retry_unchanged' "$ATTACHMENTS" >/dev/null; then
  echo "✓ PASS: attachment failure helper derives non-retry posture for validation/not_found"
else
  echo "✗ FAIL: attachment failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$ATTACHMENTS" >/dev/null; then
  echo "✗ FAIL: attachment failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Attachment failure retry posture static test: PASS"
