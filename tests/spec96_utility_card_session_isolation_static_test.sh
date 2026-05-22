#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AWARENESS="${ROOT_DIR}/apps/pi-extension/src/awareness.ts"
TURNS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
SESSION="${ROOT_DIR}/apps/pi-extension/src/session.ts"
STATE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
RUNTIME="${ROOT_DIR}/tests/utility_card_session_isolation_test.mts"

if rg -n 'getScopedWorkpointPacket|Scoped Workpoint: none verified|Mission: use latest operator instruction only' "$AWARENESS" >/dev/null; then
  echo "✓ PASS: Utility Card uses scoped Workpoint guard"
else
  echo "✗ FAIL: Utility Card scoped guard missing" >&2
  exit 1
fi

if rg -n 'scopedWorkpointForPrompt|Boolean\(getScopedWorkpointPacket\(\)\)|const packet: any = getScopedWorkpointPacket\(\)' "$TURNS" >/dev/null; then
  echo "✓ PASS: Workpoint prompt/Focus Slice injection uses scoped Workpoint guard"
else
  echo "✗ FAIL: Workpoint prompt or Focus Slice still uses unscoped packet" >&2
  exit 1
fi

if rg -n 'resetPiSessionScopedState\("session_start"\)|resetPiSessionScopedState\("session_switch"\)|String\(e\.data\.sessionId \|\| ""\) === eventSessionId|String\(switchEntries\[i\]\.data\.sessionId \|\| ""\) === eventSessionId|persistedSessionId !== eventSessionId' "$SESSION" "$STATE" >/dev/null; then
  echo "✓ PASS: session start/switch reset scoped state and restore persisted data only by session id"
else
  echo "✗ FAIL: session scoped reset/adoption guard missing" >&2
  exit 1
fi

if rg -n 'activeWorkpointPacket\?\.continuity_id|if \(e\.data\.sessionId\) S\.sessionFrameKey|if \(d\.sessionId\) S\.sessionFrameKey|Mission: .*S\.currentAsk|Next anchor: .*S\.lastCompactDecision' "$SESSION" "$AWARENESS" >/dev/null; then
  echo "✗ FAIL: unscoped Workpoint/session fallback remains in Utility Card/session restore" >&2
  exit 1
else
  echo "✓ PASS: stale Workpoint/session fallback patterns absent"
fi

if [[ -f "$RUNTIME" ]] && bun "$RUNTIME"; then
  echo "✓ PASS: runtime Utility Card isolation proof passed"
else
  echo "✗ FAIL: runtime Utility Card isolation proof failed" >&2
  exit 1
fi

echo "SPEC96 Utility Card session-isolation static test: PASS"
