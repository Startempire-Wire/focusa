#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
HEALTH="${ROOT_DIR}/crates/focusa-api/src/routes/health.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n 'resource_mode_status|"resource_mode"|"latest_transition"|"transition_omitted_count"|"hysteresis"' "$HEALTH" >/dev/null; then
  echo "✓ PASS: /v1/doctor exposes ResourceMode posture and transition summary"
else
  echo "✗ FAIL: /v1/doctor missing ResourceMode transition posture" >&2
  exit 1
fi

if rg -n 'focusaFetchDetailed\("/resource/mode"|resource=\$\{String\(resourceMode\.mode|transition=\$\{transitionLabel\}|resource_mode: resource\.body' "$TOOLS" >/dev/null; then
  echo "✓ PASS: focusa_tool_doctor surfaces ResourceMode and latest transition"
else
  echo "✗ FAIL: focusa_tool_doctor missing ResourceMode/transition output" >&2
  exit 1
fi

echo "SPEC96 Tool Doctor ResourceMode static test: PASS"
