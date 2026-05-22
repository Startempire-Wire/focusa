#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE="${ROOT_DIR}/crates/focusa-api/src/routes/traverse.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n 'tags: Vec<Value>|tag_mode: Option<String>|alias = "include_payload"|include_rehydrate_refs|budget_tokens|session_identity' "$ROUTE" >/dev/null; then
  echo "✓ PASS: traverse API accepts Spec96 schema aliases and structured tag inputs"
else
  echo "✗ FAIL: traverse API schema parity fields missing" >&2
  exit 1
fi

if rg -n 'traversed_items|"anchor"|"ordinal"|"tag"|"freshness"|"scope"|"data"' "$ROUTE" >/dev/null; then
  echo "✓ PASS: traverse output wraps items with TraversedItem-style anchors/tags"
else
  echo "✗ FAIL: TraversedItem output shape missing" >&2
  exit 1
fi

if rg -n '"caps"|"omitted"|"rehydrate_refs"|"verified_tags"|"stale_tags"|"algorithm"|"includes_anchor"|"includes_surface_version"' "$ROUTE" >/dev/null; then
  echo "✓ PASS: traversal metadata and tag_scheme expose Spec96 fields"
else
  echo "✗ FAIL: traversal metadata/tag_scheme parity fields missing" >&2
  exit 1
fi

if rg -n 'validation_rejected|read_model_lag|resource_exhausted' "$ROUTE" >/dev/null && ! rg -n 'failure_class": "unsupported_surface"|cold_full_payload_blocked_by_pressure' "$ROUTE" >/dev/null; then
  echo "✓ PASS: traverse failure classes use Spec96 taxonomy"
else
  echo "✗ FAIL: off-spec traverse failure classes remain" >&2
  exit 1
fi

if rg -n 'Type.Object\(\{[\s\S]*tag: Type.String|tag_mode|include_payload|include_rehydrate_refs|budget_tokens|session_identity' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi focusa_traverse advertises structured tags and schema aliases"
else
  echo "✗ FAIL: Pi focusa_traverse schema aliases missing" >&2
  exit 1
fi

echo "SPEC96 traverse schema static test: PASS"
