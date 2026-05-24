#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"

if rg -n 'send_work_loop_action|work_loop_dispatch_timeout|tokio::time::timeout\(Duration::from_millis\(1500\), state\.command_tx\.send\(action\)\)' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop command dispatch uses bounded helper"
else
  echo "✗ FAIL: work-loop dispatch helper or timeout missing" >&2
  exit 1
fi

if rg -n 'resource_exhausted|work-loop dispatch timed out before enqueue|command backlog may be saturated|StatusCode::ACCEPTED' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop dispatch timeout returns typed pending recovery envelope"
else
  echo "✗ FAIL: work-loop dispatch timeout lacks typed recovery envelope" >&2
  exit 1
fi

if rg -n 'work_loop_ingest_transport_lock|state\.write_serial_lock\.lock\(\)' "$WORK_LOOP" >/dev/null && ! rg -n 'let _guard = state\.write_serial_lock\.lock\(\)\.await;' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop transport ingest write-lock wait is bounded"
else
  echo "✗ FAIL: work-loop transport ingest can wait unbounded on write_serial_lock" >&2
  exit 1
fi

if rg -n 'work_loop_pause|work_loop_resume|work_loop_select_next|work_loop_context|work_loop_checkpoint|work_loop_stop' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: public work-loop write routes name bounded dispatch actions"
else
  echo "✗ FAIL: public work-loop write route dispatch names missing" >&2
  exit 1
fi

echo "SPEC96 work-loop dispatch timeout static test: PASS"
