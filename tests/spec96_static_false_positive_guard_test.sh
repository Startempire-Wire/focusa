#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUNTIME_FOCUS="${ROOT_DIR}/tests/spec96_focus_slice_runtime_injection_test.mts"
TRAVERSE_RUNTIME="${ROOT_DIR}/tests/spec96_traverse_schema_runtime_test.sh"
LOWMEM_STATIC="${ROOT_DIR}/tests/spec96_lowmem_focus_slice_wpv2_static_test.sh"
TRAJ_STATIC="${ROOT_DIR}/tests/spec96_trajectory_focus_slice_static_test.sh"
GOLDEN="${ROOT_DIR}/tests/spec96_tool_affordance_catalog_golden_eval_test.sh"

if rg -n 'pi.emit\("context"|injected\.includes\("PROJECT_IDENTITY|injected\.includes\("TRAJECTORY_GOALS|injected\.includes\("RESOURCE_MODE|injected\.includes\("TOOL_AFFORDANCES' "$RUNTIME_FOCUS" >/dev/null; then
  echo "✓ PASS: Focus Slice runtime test asserts emitted sections, not just source strings"
else
  echo "✗ FAIL: Focus Slice runtime emitted-section assertions missing" >&2
  exit 1
fi

if rg -n 'spec96_focus_slice_runtime_injection_test\.sh' "$LOWMEM_STATIC" "$TRAJ_STATIC" >/dev/null; then
  echo "✓ PASS: static LowMem/Trajectory guards invoke mocked runtime injection proof"
else
  echo "✗ FAIL: static Focus Slice guards can still pass without runtime proof" >&2
  exit 1
fi

if rg -n 'curl -fsS.*v1/traverse|jq -e.*items\[0\]|/v1/traverse/verify-tags|validation_rejected' "$TRAVERSE_RUNTIME" >/dev/null; then
  echo "✓ PASS: traverse schema proof asserts live response schema and validation behavior"
else
  echo "✗ FAIL: traverse schema runtime proof missing live assertions" >&2
  exit 1
fi

if rg -n 'baseline|outperform baseline|without source|source-code|SPEC96 Tool Affordance Catalog golden eval' "$GOLDEN" >/dev/null; then
  echo "✓ PASS: tool-choice golden eval compares affordance surface against no-source baseline"
else
  echo "✗ FAIL: tool-choice golden eval no-source baseline missing" >&2
  exit 1
fi

echo "SPEC96 static false-positive guard test: PASS"
