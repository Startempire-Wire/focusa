#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in formatWorkLoopBudgetRemaining remaining_turn_budget remaining_wall_clock_ms remaining_failure_budget remaining_low_productivity_budget remaining_same_subproblem_budget; do
  rg -F "$term" apps/pi-extension/src/tools.ts >/dev/null || fail "Pi work-loop budget renderer missing $term"
done
pass "Pi extension declares explicit budget renderer fields"

if rg -n 'String\(loopStatus\?\.budget_remaining\)' apps/pi-extension/src/tools.ts >/dev/null; then
  fail "budget_remaining still rendered via raw String(object)"
fi
pass "budget_remaining is not stringified as raw object"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
curl -fsS --max-time 15 "$BASE/v1/work-loop/status?summary_only=true" >/tmp/spec102-work-loop-status.json
jq -e '.budget_remaining | type == "object" and has("remaining_failure_budget")' /tmp/spec102-work-loop-status.json >/dev/null || fail "daemon status missing explicit budget object fields"
if jq -r '.budget_remaining' /tmp/spec102-work-loop-status.json | rg -F '[object Object]' >/dev/null; then
  fail "daemon budget response contains [object Object]"
fi
pass "daemon budget response is explicit object without [object Object]"

echo "SPEC102 work-loop budget render schema test: PASS"
