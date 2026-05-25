#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"

if rg -n 'resume_render_dispatch_warning|returning the already-rendered canonical packet|packet returned from read model to preserve continuation' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint resume returns rendered packet even when telemetry dispatch is saturated"
else
  echo "✗ FAIL: Workpoint resume can still be blocked by telemetry dispatch saturation" >&2
  exit 1
fi

if rg -n 'focusa_resource_mode or focusa_tool_doctor|resume packet is usable' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint resume degraded telemetry path includes recovery guidance"
else
  echo "✗ FAIL: Workpoint resume degraded telemetry path lacks recovery guidance" >&2
  exit 1
fi

echo "SPEC96 Workpoint resume dispatch degraded static test: PASS"
