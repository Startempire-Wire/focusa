#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
SESSION_TS="$ROOT_DIR/apps/pi-extension/src/session.ts"
TEST_MTS="$ROOT_DIR/tests/spec_report_summary_capture_runtime_test.mts"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'PiReportSummaryHandle|latestReportSummary|maybeCaptureReportSummaryFromAssistantOutput|report-summary' "$STATE_TS" >/dev/null \
  || fail "report-summary handle state/capture helpers missing"
pass "report-summary handle helpers exist"

rg -n 'latest_report_summary_ref|S\.latestReportSummary\?\.handle' "$STATE_TS" >/dev/null \
  || fail "AttentionRecallVerdict does not replay latest report summary handle"
pass "AttentionRecallVerdict replays latest report summary ref"

rg -n 'report_summary_captured|maybeCaptureReportSummaryFromAssistantOutput' "$TURNS_TS" >/dev/null \
  || fail "turn_end does not capture assistant-produced report summaries"
pass "turn_end captures report summaries"

rg -n 'latestReportSummary' "$SESSION_TS" >/dev/null \
  || fail "session restore does not persist/reload latest report summary handle"
pass "session persistence covers latest report summary"

bun "$TEST_MTS"

echo "Report replay static/runtime test: PASS"
