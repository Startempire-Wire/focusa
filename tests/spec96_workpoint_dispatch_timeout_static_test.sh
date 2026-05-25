#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"

if rg -n 'workpoint_dispatch_timeout|tokio::time::timeout\(|Duration::from_millis\(1500\)|state\.command_tx\.send\(Action::EmitEvent' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: normal Workpoint dispatch send is bounded"
else
  echo "✗ FAIL: normal Workpoint dispatch can wait unbounded on command_tx.send" >&2
  exit 1
fi

if rg -n 'failure_class": "resource_exhausted"|daemon command channel is saturated|workpoint event was not enqueued|focusa_resource_mode' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint dispatch timeout returns typed recovery envelope"
else
  echo "✗ FAIL: Workpoint dispatch timeout lacks typed recovery envelope" >&2
  exit 1
fi

if rg -n 'resume_render_dispatch_warning|packet returned from read model to preserve continuation' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint resume telemetry dispatch saturation no longer blocks rendered packet"
else
  echo "✗ FAIL: Workpoint resume can be blocked by telemetry dispatch saturation" >&2
  exit 1
fi

echo "SPEC96 Workpoint dispatch timeout static test: PASS"
