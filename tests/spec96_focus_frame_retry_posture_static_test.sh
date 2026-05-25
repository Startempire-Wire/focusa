#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FOCUS="${ROOT_DIR}/crates/focusa-api/src/routes/focus.rs"

if rg -n 'retry_posture": "refresh_scoped_frame"' "$FOCUS" >/dev/null; then
  echo "✗ FAIL: Focus frame unavailable response emits non-contract retry_posture refresh_scoped_frame" >&2
  exit 1
fi

if rg -n '"retry_posture": "safe_retry"|"retry": \{"safe": true, "posture": "safe_retry", "reason": "frame_unavailable"\}|"next_tools": \["focusa_workpoint_resume", "focusa_workpoint_checkpoint", "focusa_tool_doctor"\]' "$FOCUS" >/dev/null; then
  echo "✓ PASS: Focus frame unavailable response has contract retry posture and recovery tools"
else
  echo "✗ FAIL: Focus frame unavailable response lacks contract retry recovery envelope" >&2
  exit 1
fi

echo "SPEC96 Focus frame retry posture static test: PASS"
