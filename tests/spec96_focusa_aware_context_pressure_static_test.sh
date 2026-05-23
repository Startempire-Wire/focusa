#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMPACTION_TS="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
TURNS_TS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
FOCUSA_SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md"
QUICKSTART="${ROOT_DIR}/docs/current/AGENT_AWARENESS_QUICKSTART.md"

if rg -n 'isFocusaContextContinuityHealthy|focusaContinuityReady|scoped Workpoint anchor not yet confirmed|Focusa anchors are unconfirmed' "$COMPACTION_TS" >/dev/null; then
  echo "✓ PASS: context-pressure path is Focusa anchor aware"
else
  echo "✗ FAIL: context-pressure path lacks Focusa anchor gating" >&2
  exit 1
fi

if rg -n '!focusaContinuityReady[\s\S]*(checkpoint/resume Workpoint|scoped Workpoint anchor not yet confirmed)|if \(!focusaContinuityReady\)' "$COMPACTION_TS" >/dev/null; then
  echo "✓ PASS: operator warning is gated to unconfirmed scoped anchors"
else
  echo "✗ FAIL: operator warning is not visibly scoped-anchor gated" >&2
  exit 1
fi

if rg -n 'Focusa continuity degraded|hard compacting\. Consider /fork or /new|Context at \$\{pct\.toFixed\(0\)\}% — consider /fork|critical · fork/new|consider /fork to preserve context quality|consider /fork or /new before fallback|without healthy Workpoint continuity' "$COMPACTION_TS" "$TURNS_TS" >/dev/null; then
  echo "✗ FAIL: stale or inaccurate context-pressure warning text remains" >&2
  exit 1
else
  echo "✓ PASS: stale/degrading fork-new warning text removed"
fi

if rg -n 'Context pressure UX|Focusa preserves continuity|scoped Focusa anchors|optional UI-isolation|Context pressure is Focusa-aware' "$FOCUSA_SKILL" "$QUICKSTART" >/dev/null; then
  echo "✓ PASS: skill/docs describe Focusa-aware context-pressure semantics"
else
  echo "✗ FAIL: Focusa-aware context-pressure docs missing" >&2
  exit 1
fi

echo "SPEC96 Focusa-aware context-pressure static test: PASS"
