#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMMANDS="${ROOT_DIR}/crates/focusa-api/src/routes/commands.rs"

if rg -n 'matches!\(failure_class, "validation_rejected" \| "not_found" \| "frame_unavailable"\)|do_not_retry_unchanged' "$COMMANDS" >/dev/null; then
  echo "✓ PASS: command failure helper derives non-retry posture for validation/not_found/frame_unavailable"
else
  echo "✗ FAIL: command failure helper lacks failure_class-derived retry posture" >&2
  exit 1
fi

if rg -n '"retry": \{"safe": true, "posture": "safe_retry", "reason": failure_class\}' "$COMMANDS" >/dev/null; then
  echo "✗ FAIL: command failure helper still advertises all failures as safe_retry" >&2
  exit 1
fi

echo "SPEC96 Command failure retry posture static test: PASS"
