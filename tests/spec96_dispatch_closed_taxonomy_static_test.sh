#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMMANDS="${ROOT_DIR}/crates/focusa-api/src/routes/commands.rs"
WORK_LOOP="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"

if rg -n 'command_dispatch_failed[\s\S]*"daemon_unavailable"|command dispatch unavailable|idempotency key/command_id' "$COMMANDS" >/dev/null; then
  echo "✓ PASS: command dispatch closed-channel failures use daemon_unavailable with idempotency guidance"
else
  echo "✗ FAIL: command dispatch closed-channel failure remains ambiguous" >&2
  exit 1
fi

if rg -n 'work_loop_dispatch_failed[\s\S]*"daemon_unavailable"|dispatch channel unavailable for' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop dispatch closed-channel failures use daemon_unavailable"
else
  echo "✗ FAIL: work-loop dispatch closed-channel failure remains ambiguous" >&2
  exit 1
fi

if rg -n 'work_loop_dispatch_failed[\s\S]*unknown_ambiguous_completion|command_dispatch_failed[\s\S]*unknown_ambiguous_completion' "$COMMANDS" "$WORK_LOOP" >/dev/null; then
  echo "✗ FAIL: dispatch closed-channel helpers still contain unknown_ambiguous_completion" >&2
  exit 1
fi

echo "SPEC96 Dispatch closed-channel taxonomy static test: PASS"
