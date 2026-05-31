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

rg -n 'recordToolOutputPressure\(toolName, content\.length, tokens\)|visible_recap_required|tool_output_flood_detected|visible_recap_emitted' "$TURNS_TS" "$STATE_TS" >/dev/null \
  || fail "Tool-result path does not set visible_recap_required and telemetry"
pass "Tool-result path records pressure and telemetry"

rg -n 'Focusa Visible Recap Enforcement|Before any tool/file/API action|formatToolOutputVisibleRecapLines\(visibleRecapReason\)|visibleRecapReason' "$TURNS_TS" "$COMPACTION_TS" >/dev/null \
  || fail "Visible recap is not enforced before project-scoped action"
pass "Visible recap enforcement is wired into prompt/context/compaction"

rg -n 'toolOutputPressure.*recapRequired|toolOutputPressure: S\.toolOutputPressure|persistState\(\)' "$STATE_TS" "$ROOT_DIR/apps/pi-extension/src/session.ts" >/dev/null \
  || fail "Visible recap pressure is not persisted across compaction/session transfer"
pass "Visible recap pressure persists until recap clears it"

rg -n 'tool-output flood|visible_recap_required=true|assistant can restate the report summary' "$SPEC" >/dev/null \
  || fail "Spec does not describe tool-output flood recap expectation"
pass "Spec documents tool-output flood recap expectation"

echo "Tool-output flood recap static test: PASS"
