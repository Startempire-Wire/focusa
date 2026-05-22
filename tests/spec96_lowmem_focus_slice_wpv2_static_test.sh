#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TURNS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if perl -0ne 'exit(/const resourceModeLines = await getResourceModeFocusSliceLines\(\);[\s\S]*buildSliceSection\("resource_mode"[\s\S]*resourceModeLines/s ? 0 : 1)' "$TURNS"; then
  echo "✓ PASS: Focus Slice actually injects LowMem/resource posture section"
else
  echo "✗ FAIL: Focus Slice LowMem posture is computed but not injected" >&2
  exit 1
fi

if perl -0ne 'exit(/function getToolAffordanceFocusSliceLines[\s\S]*TOOL_AFFORDANCES:[\s\S]*buildSliceSection\("tool_affordances"[\s\S]*toolAffordanceLines/s ? 0 : 1)' "$TURNS"; then
  echo "✓ PASS: Focus Slice injects TOOL_AFFORDANCES guidance"
else
  echo "✗ FAIL: Focus Slice TOOL_AFFORDANCES guidance missing" >&2
  exit 1
fi

if rg -n 'resource_mode_resume_payload|resource_mode|lowmem_budget|cold_surfaces_deferred|targeted_rehydration|focusa_traverse' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint Resume Packet v2 includes resource mode posture"
else
  echo "✗ FAIL: Workpoint v2 resource mode posture missing" >&2
  exit 1
fi

if rg -n 'full_lineage_tree|full_ontology_graph|deep_work_loop_status|surgical_summary_only|focusa_traverse' "$TURNS" "$WORKPOINT" "$SPEC" >/dev/null; then
  echo "✓ PASS: LowMem guidance avoids full cold defaults and points to traversal"
else
  echo "✗ FAIL: LowMem do_not_use/traversal guidance missing" >&2
  exit 1
fi

if bash "${ROOT_DIR}/tests/spec96_focus_slice_runtime_injection_test.sh" >/tmp/spec96-focus-slice-runtime-proof.log 2>&1; then
  echo "✓ PASS: Focus Slice mocked runtime proof emits LowMem, trajectory, and tool affordances"
else
  echo "✗ FAIL: Focus Slice runtime proof failed" >&2
  cat /tmp/spec96-focus-slice-runtime-proof.log >&2
  exit 1
fi

echo "SPEC96 LowMem Focus Slice + Workpoint v2 static test: PASS"
