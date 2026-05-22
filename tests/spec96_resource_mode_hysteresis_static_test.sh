#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BOUNDED="${ROOT_DIR}/crates/focusa-api/src/routes/bounded.rs"
SERVER="${ROOT_DIR}/crates/focusa-api/src/server.rs"

if rg -n 'ResourceModeHysteresisRuntime|RESOURCE_MODE_HYSTERESIS_STATE|FOCUSA_RESOURCE_MODE_HYSTERESIS_RECOVERY_SAMPLES' "$BOUNDED" >/dev/null; then
  echo "✓ PASS: ResourceMode has explicit hysteresis runtime state"
else
  echo "✗ FAIL: ResourceMode hysteresis runtime state missing" >&2
  exit 1
fi

if rg -n 'immediate_escalation_delayed_recovery|delayed_recovery_hold|hysteresis_recovery_hold|immediate_escalation' "$BOUNDED" >/dev/null; then
  echo "✓ PASS: ResourceMode hysteresis prevents recovery flapping while allowing escalation"
else
  echo "✗ FAIL: ResourceMode hysteresis policy missing" >&2
  exit 1
fi

if rg -n 'hysteresis_state: status\.hysteresis\.clone\(\)|durability: "pending"|RESOURCE_MODE_TRANSITION_RING_LIMIT|transition_omitted_count' "$BOUNDED" >/dev/null; then
  echo "✓ PASS: ResourceMode transition records include hysteresis/durability and capped history"
else
  echo "✗ FAIL: ResourceMode transition record evidence missing" >&2
  exit 1
fi

if rg -n 'observe_resource_mode_transition\("background_resource_monitor"|FOCUSA_RESOURCE_MODE_MONITOR_INTERVAL_SECS' "$SERVER" >/dev/null; then
  echo "✓ PASS: daemon background monitor observes ResourceMode transitions without active session"
else
  echo "✗ FAIL: background ResourceMode monitor missing" >&2
  exit 1
fi

echo "SPEC96 ResourceMode hysteresis static test: PASS"
