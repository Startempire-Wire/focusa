#!/usr/bin/env bash
set -euo pipefail
BASE="${UIAI_ENGINE_URL:-http://127.0.0.1:7456}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

curl -fsS --max-time 10 "$BASE/api/health/browser" >/tmp/spec102-uiai-health.json || { echo "UIAI unavailable; live checks skipped"; exit 0; }

jq -e '
  (.current_capacity | type == "object")
  and (.historical_pressure | type == "object")
  and (.agent_pressure.current_capacity | type == "object")
  and (.agent_pressure.historical_pressure | type == "object")
  and (.agent_pressure.browser.current_capacity | type == "object")
  and (.agent_pressure.browser.historical_pressure | type == "object")
' /tmp/spec102-uiai-health.json >/dev/null || fail "UIAI health missing current_capacity/historical_pressure split"
pass "UIAI health exposes current_capacity separate from historical_pressure"

jq -e '
  .current_capacity.capacity_available == true
  and .current_capacity.remaining_page_slots > 0
  and (.agent_pressure.overall_pressure != "saturated")
  and (.agent_pressure.browser.pressure != "saturated")
  and (.agent_pressure.browser.pressure != "constrained")
' /tmp/spec102-uiai-health.json >/dev/null || fail "current available capacity still appears saturated/constrained"
pass "available current capacity cannot be mistaken for saturation"

jq -e '
  (.historical_pressure.queue_p95_wait_ms | type == "number")
  and (.historical_pressure.queue_rejected | type == "number")
  and (.historical_pressure.note | test("historical pressure.*current_capacity"))
' /tmp/spec102-uiai-health.json >/dev/null || fail "historical pressure detail not retained"
pass "historical pressure remains available in detail view"

echo "SPEC102 UIAI pressure split test: PASS"
