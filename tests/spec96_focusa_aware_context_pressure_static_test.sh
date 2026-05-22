#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMPACTION_TS="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
TURNS_TS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
FOCUSA_SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md"
QUICKSTART="${ROOT_DIR}/docs/current/AGENT_AWARENESS_QUICKSTART.md"

if rg -n 'isFocusaContextContinuityHealthy|focusaContinuityReady|Focusa continuity degraded' "$COMPACTION_TS" >/dev/null; then
  echo "✓ PASS: context-pressure path is Focusa continuity aware"
else
  echo "✗ FAIL: context-pressure path lacks Focusa continuity gating" >&2
  exit 1
fi

if rg -n '!focusaContinuityReady[\s\S]*consider /fork|if \(!focusaContinuityReady\)' "$COMPACTION_TS" >/dev/null; then
  echo "✓ PASS: generic fork/new advice is gated to degraded continuity"
else
  echo "✗ FAIL: generic fork/new advice is not visibly degraded-state gated" >&2
  exit 1
fi

if rg -n 'hard compacting\. Consider /fork or /new|Context at \$\{pct\.toFixed\(0\)\}% — consider /fork|critical · fork/new' "$COMPACTION_TS" "$TURNS_TS" >/dev/null; then
  echo "✗ FAIL: stale generic context-pressure warning text remains" >&2
  exit 1
else
  echo "✓ PASS: stale generic fork/new warning text removed"
fi

if rg -n 'Context pressure UX|generic /fork|healthy Workpoint continuity|Focusa continuity' "$FOCUSA_SKILL" "$QUICKSTART" >/dev/null; then
  echo "✓ PASS: skill/docs describe Focusa-aware context-pressure semantics"
else
  echo "✗ FAIL: Focusa-aware context-pressure docs missing" >&2
  exit 1
fi

echo "SPEC96 Focusa-aware context-pressure static test: PASS"
