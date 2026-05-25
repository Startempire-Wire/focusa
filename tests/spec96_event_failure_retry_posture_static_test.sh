#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
EVENTS="${ROOT_DIR}/crates/focusa-api/src/routes/events.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found"\)|do_not_retry_unchanged' "$EVENTS" >/dev/null; then
  echo "✓ PASS: legacy event failure helper derives non-retry posture for validation/not_found"
else
  echo "✗ FAIL: legacy event failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$EVENTS" >/dev/null; then
  echo "✗ FAIL: legacy event failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Event failure retry posture static test: PASS"
