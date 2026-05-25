#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found" \| "permission_denied" \| "writer_conflict" \| "approval_required"\)|do_not_retry_unchanged' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop failure helper derives non-retry posture for writer/approval/validation states"
else
  echo "✗ FAIL: work-loop failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$WORK_LOOP" >/dev/null; then
  echo "✗ FAIL: work-loop failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Work-loop failure retry posture static test: PASS"
