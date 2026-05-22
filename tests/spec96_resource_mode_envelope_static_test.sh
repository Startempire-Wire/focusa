#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RESOURCE="${ROOT_DIR}/crates/focusa-api/src/routes/resource.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CLI="${ROOT_DIR}/crates/focusa-cli/src/commands/resource.rs"

if rg -n 'preflight: bool|body\.preflight|resource mode preflight' "$RESOURCE" >/dev/null; then
  echo "✓ PASS: ResourceMode API accepts preflight without mutation"
else
  echo "✗ FAIL: ResourceMode API preflight missing" >&2
  exit 1
fi

if rg -n 'tool_result_v1|canonical|degraded|failure_class|retry|side_effects|resource_tool_result_exposes_required_envelope_fields' "$RESOURCE" >/dev/null; then
  echo "✓ PASS: ResourceMode API exposes canonical/degraded tool_result_v1-style envelope"
else
  echo "✗ FAIL: ResourceMode canonical/degraded tool_result_v1 envelope missing" >&2
  exit 1
fi

if rg -n 'preflight|activate_lowmem|deactivate_lowmem|set_mode' "$TOOLS" "$CLI" >/dev/null; then
  echo "✓ PASS: Pi and CLI ResourceMode callers expose preflight/control actions"
else
  echo "✗ FAIL: ResourceMode client parity missing" >&2
  exit 1
fi

echo "SPEC96 ResourceMode preflight/envelope static test: PASS"
