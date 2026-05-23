#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AWARENESS="${ROOT_DIR}/apps/pi-extension/src/awareness.ts"
TURNS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
SESSION="${ROOT_DIR}/apps/pi-extension/src/session.ts"
STATE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
RUNTIME="${ROOT_DIR}/tests/utility_card_session_isolation_test.mts"

if rg -n 'getScopedWorkpointPacket|Project-bound Workpoint: none verified|Mission: use latest operator instruction only|broad/unsafe' "$AWARENESS" >/dev/null; then
  echo "✓ PASS: Utility Card uses project-bound Workpoint guard"
else
  echo "✗ FAIL: Utility Card project-bound guard missing" >&2
  exit 1
fi

if rg -n 'mode === "visible" && !scopedPacket|split\("\\n"\)\.length <= 7|packageUpdateCommand|REQUIRED FIRST: confirm project_root|folder/container holding project files|current state' "$AWARENESS" "$TURNS" "$RUNTIME" >/dev/null; then
  echo "✓ PASS: unscoped login/update Utility Card stays compact and prioritizes project folder + trajectory"
else
  echo "✗ FAIL: login/update Utility Card remains noisy or does not prioritize project folder + trajectory" >&2
  exit 1
fi

if rg -n 'resolvePiProjectRoot|adoptPiProjectRoot|\.focusa-project\.json|\.beads|\.git' "$STATE" "${ROOT_DIR}/apps/pi-extension/src/session.ts" >/dev/null && bash "${ROOT_DIR}/tests/pi_project_root_inference_test.sh"; then
  echo "✓ PASS: Pi infers safe project root instead of broad cwd when evidence exists"
else
  echo "✗ FAIL: Pi project-root inference missing or unsafe" >&2
  exit 1
fi

if rg -n 'scopedWorkpointForPrompt|Boolean\(getScopedWorkpointPacket\(\)\)|const packet: any = getScopedWorkpointPacket\(\)' "$TURNS" >/dev/null; then
  echo "✓ PASS: Workpoint prompt/Focus Slice injection uses project-bound Workpoint guard"
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
  echo "✗ FAIL: unbound Workpoint/session fallback remains in Utility Card/session restore" >&2
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
