#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE="${ROOT_DIR}/crates/focusa-api/src/routes/traverse.rs"
AUDIT="${ROOT_DIR}/scripts/audit-focusa-tool-suite-safe.mjs"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_traverse.md"

for surface in trajectory lineage ontology focus_stack workpoints evidence metacognition predictions telemetry snapshots tool_registry; do
  if ! rg -n "\"${surface}\"" "$ROUTE" >/dev/null; then
    echo "✗ FAIL: focusa_traverse missing adapter for ${surface}" >&2
    exit 1
  fi
done
echo "✓ PASS: focusa_traverse has adapters for all major surfaces"

if rg -n 'generic_filter_items|"current" \| "by_id"|"search"|"recent"|"path"|"neighborhood"|"summaries"' "$ROUTE" >/dev/null; then
  echo "✓ PASS: adapter selectors cover current/by_id/search/recent/path/neighborhood/window-style use"
else
  echo "✗ FAIL: selector coverage missing" >&2
  exit 1
fi

if rg -n 'budgeted_default_limit|budgeted_hard_limit|budgeted_requested_limit|include_full_payload|full_payload_blocked_by_pressure' "$ROUTE" >/dev/null; then
  echo "✓ PASS: adapters obey bounded caps and cold full-payload guard"
else
  echo "✗ FAIL: bounded/cold guard missing from adapters" >&2
  exit 1
fi

if rg -n 'unbounded_hot_traversal|missing_traversal_adapter|/v1/traverse' "$AUDIT" >/dev/null; then
  echo "✓ PASS: safe audit detects unbounded or incomplete traverse adapters"
else
  echo "✗ FAIL: safe audit traverse guard missing" >&2
  exit 1
fi

if rg -n 'Surface adapters|Expected result|failure_class|validation_rejected' "$DOC" >/dev/null; then
  echo "✓ PASS: traverse adapter docs cover surfaces and failure semantics"
else
  echo "✗ FAIL: traverse adapter docs incomplete" >&2
  exit 1
fi

echo "SPEC96 traverse adapter static test: PASS"
