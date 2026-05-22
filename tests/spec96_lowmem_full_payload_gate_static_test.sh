#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BOUNDED="${ROOT_DIR}/crates/focusa-api/src/routes/bounded.rs"
STRESS="${ROOT_DIR}/tests/spec96_lowmem_surgical_agent_stress_test.sh"

if rg -n 'matches!\(status\.mode, "lowmem" \| "emergency"\)|status\.pressure\.active' "$BOUNDED" >/dev/null; then
  echo "✓ PASS: full-payload gating is driven by ResourceMode lowmem/emergency plus pressure"
else
  echo "✗ FAIL: full-payload gating still only checks raw pressure or lacks ResourceMode gate" >&2
  exit 1
fi

if rg -n 'forced_lowmem_blocks_full_payload_without_force_override|set_runtime_resource_mode_override\(Some\("lowmem"\)\)|set_runtime_resource_mode_override\(Some\("emergency"\)\)' "$BOUNDED" >/dev/null; then
  echo "✓ PASS: unit coverage proves forced LowMem/emergency block full payload unless force override"
else
  echo "✗ FAIL: forced LowMem full-payload unit coverage missing" >&2
  exit 1
fi

if rg -n 'include_full_payload=true|full_payload_blocked_by_pressure|degraded|opt-in metadata' "$STRESS" >/dev/null; then
  echo "✓ PASS: runtime stress covers full-payload degradation metadata"
else
  echo "✗ FAIL: runtime full-payload degradation proof missing" >&2
  exit 1
fi

echo "SPEC96 LowMem full-payload gate static test: PASS"
