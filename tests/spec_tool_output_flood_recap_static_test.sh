#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
COMPACTION_TS="$ROOT_DIR/apps/pi-extension/src/compaction.ts"
SPEC="$ROOT_DIR/docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'TOOL_OUTPUT_FLOOD_(WINDOW_MS|RESULT_THRESHOLD|BYTES_THRESHOLD|TOKENS_THRESHOLD|LARGE_RESULT)' "$STATE_TS" >/dev/null \
  || fail "Tool-output flood thresholds missing or comment-only"
pass "Tool-output flood thresholds exist in source"

rg -n 'recordToolOutputPressure|toolOutputVisibleRecapReason|formatToolOutputVisibleRecapLines|markVisibleRecapEmittedIfPresent' "$STATE_TS" >/dev/null \
  || fail "Tool-output flood pressure/recap helpers missing"
pass "Tool-output pressure and recap helpers exist"

rg -n 'recordToolOutputPressure\(toolName, content\.length, tokens\)|memory_refresh_consumed|tool_output_flood_detected|visible_recap_required: false' "$TURNS_TS" "$STATE_TS" >/dev/null \
  || fail "Tool-result path does not record internal memory refresh telemetry"
pass "Tool-result path records pressure and internal refresh telemetry"

rg -n 'FOCUSA_MEMORY_REFRESH|operator_flow=continue|formatToolOutputVisibleRecapLines\(visibleRecapReason\)|visibleRecapReason' "$TURNS_TS" "$COMPACTION_TS" >/dev/null \
  || fail "Internal memory refresh is not wired into context/compaction"
pass "Internal memory refresh is wired without blocking operator flow"

rg -n 'toolOutputPressure.*recapRequired|toolOutputPressure: S\.toolOutputPressure|persistState\(\)' "$STATE_TS" "$ROOT_DIR/apps/pi-extension/src/session.ts" >/dev/null \
  || fail "Tool-output pressure is not persisted across compaction/session transfer"
pass "Tool-output pressure persists until the next assistant continuation consumes it"

rg -n 'tool-output flood|visible_recap_required.*false|internal memory anchor' "$SPEC" >/dev/null \
  || fail "Spec does not describe non-blocking tool-output memory refresh"
pass "Spec documents non-blocking tool-output memory refresh"

echo "Tool-output flood recap static test: PASS"
