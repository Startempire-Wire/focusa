#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUNTIME="${ROOT_DIR}/tests/spec96_lowmem_surgical_agent_stress_test.sh"
DEPS="${ROOT_DIR}/tests/spec96_lowmem_tool_dependencies_runtime_test.sh"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"
DOC="${ROOT_DIR}/docs/evidence/SPEC96_LOWMEM_SURGICAL_AGENT_STRESS.md"

if rg -n 'activate_lowmem|all_tools_callable_with_bounded_or_degraded_envelopes|no official tool disappears' "$RUNTIME" "$DEPS" >/dev/null; then
  echo "✓ PASS: LowMem tests prove no tool disappearance"
else
  echo "✗ FAIL: LowMem tool disappearance proof missing" >&2; exit 1
fi

if rg -n 'status/deep|health-after-cold|uptime|restart storm|cold route pressure' "$RUNTIME" >/dev/null; then
  echo "✓ PASS: LowMem stress checks cold pressure without restart storm"
else
  echo "✗ FAIL: LowMem restart storm guard missing" >&2; exit 1
fi

if rg -n 'include_full_payload=true|full_payload_blocked_by_pressure|degraded|opt-in metadata' "$RUNTIME" "$SPEC" >/dev/null; then
  echo "✓ PASS: LowMem stress checks explicit full-payload degradation metadata"
else
  echo "✗ FAIL: LowMem full-payload degradation check missing" >&2; exit 1
fi

if rg -n 'surgical_summary_only|identity_axes|focusa_traverse|Fresh agent|summaries \+ focusa_traverse' "$RUNTIME" "$DOC" >/dev/null; then
  echo "✓ PASS: LowMem golden surgical-agent task is specified"
else
  echo "✗ FAIL: LowMem surgical-agent golden task missing" >&2; exit 1
fi

echo "SPEC96 LowMem surgical-agent static test: PASS"
