#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_silent_sessions.md"

if rg -n 'process_control_failed|List/health/tail the SilentSession|tmux session missing' "$TOOLS" >/dev/null; then
  echo "✓ PASS: SilentSession process-control failure taxonomy exists"
else
  echo "✗ FAIL: SilentSession process-control failure taxonomy missing" >&2
  exit 1
fi

if ! rg -n 'tmux (restart kill phase|interrupt|send-keys|kill-session)' "$TOOLS" >/dev/null \
  && rg -n 'case "process_control_failed"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: daemon-native process-control failures are typed without legacy tmux ambiguity"
elif rg -n 'tmux (restart kill phase|interrupt|send-keys|kill-session) failed' "$TOOLS" >/dev/null \
  && ! rg -n 'tmux (restart kill phase|interrupt|send-keys|kill-session) failed[\s\S]{0,160}unknown_ambiguous_completion' "$TOOLS" >/dev/null; then
  echo "✓ PASS: legacy tmux process-control failures are not classified as ambiguous completion"
else
  echo "✗ FAIL: process-control failures still classify ambiguously" >&2
  exit 1
fi

if rg -n 'failure_class=process_control_failed' "$DOC" >/dev/null; then
  echo "✓ PASS: SilentSession docs expose process_control_failed recovery"
else
  echo "✗ FAIL: SilentSession docs lack process_control_failed guidance" >&2
  exit 1
fi

echo "SPEC96 SilentSession process failure static test: PASS"
