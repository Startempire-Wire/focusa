#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS="${ROOT_DIR}/apps/pi-extension/src/tool-contracts.ts"
TURNS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
DOC="${ROOT_DIR}/docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md"

if rg -n 'FocusaToolAffordance|buildFocusaToolAffordanceCatalog|when_to_use|when_not_to_use|default_inputs|side_effects|failure_classes|recovery|example|expected_result|likely_next_tools' "$CONTRACTS" >/dev/null; then
  echo "✓ PASS: Tool Affordance Catalog exposes Spec96 model-facing fields"
else
  echo "✗ FAIL: Tool Affordance Catalog fields missing" >&2
  exit 1
fi

if rg -n 'selectFocusSliceToolAffordances|catalog_version.*spec96\.tool_affordance_catalog\.v1|scope_mismatch -> focusa_project_verify|resource_exhausted\|cold_path_timeout|canonical=false\|degraded=true' "$CONTRACTS" >/dev/null; then
  echo "✓ PASS: catalog provides best-next/recovery/do-not-use selection"
else
  echo "✗ FAIL: catalog selection/recovery guidance missing" >&2
  exit 1
fi

if rg -n 'selectFocusSliceToolAffordances' "$TURNS" >/dev/null && rg -n 'TOOL_AFFORDANCES|best_next|recovery|do_not_use' "$TURNS" >/dev/null; then
  echo "✓ PASS: Focus Slice uses catalog-selected TOOL_AFFORDANCES"
else
  echo "✗ FAIL: Focus Slice is not wired to catalog-selected affordances" >&2
  exit 1
fi

if rg -n 'Tool Affordance Catalog|when-to-use|failure classes|likely next tools|Focus Slice TOOL_AFFORDANCES' "$DOC" >/dev/null; then
  echo "✓ PASS: docs advertise Tool Affordance Catalog posture"
else
  echo "✗ FAIL: Tool Affordance Catalog docs missing" >&2
  exit 1
fi

echo "SPEC96 Tool Affordance Catalog static test: PASS"
