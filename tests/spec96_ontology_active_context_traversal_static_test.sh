#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ONT="${ROOT_DIR}/crates/focusa-api/src/routes/ontology.rs"
TRAVERSE="${ROOT_DIR}/crates/focusa-api/src/routes/traverse.rs"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'selector.*working_set|selector.*active_context|selector.*adjacency|traversal_metadata' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology/active context exposes traversal selectors and metadata"
else
  echo "✗ FAIL: ontology traversal selectors/metadata missing" >&2
  exit 1
fi

if rg -n 'field_projection|limit|cursor|next_cursor|summary_only|cold_full_payload_opt_in' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology hot context exposes limits/cursors/projections"
else
  echo "✗ FAIL: ontology limits/cursors/projections missing" >&2
  exit 1
fi

if rg -n 'rehydrate_refs|rehydrate_routes|/v1/ontology/adjacency|/v1/ecs/rehydrate' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology context exposes targeted rehydrate refs"
else
  echo "✗ FAIL: ontology rehydrate refs missing" >&2
  exit 1
fi

if rg -n 'do_not_use.*full_ontology_graph|surgical_summary_only|uncertainty_flags|uncertainty_label' "$ONT" "$SPEC" >/dev/null; then
  echo "✓ PASS: ontology context marks uncertainty and forbids full graph default"
else
  echo "✗ FAIL: ontology uncertainty/do_not_use posture missing" >&2
  exit 1
fi

if rg -n '"ontology"|surface.*ontology|adapter.*ontology|bounded_json_items' "$TRAVERSE" "$ONT" >/dev/null; then
  echo "✓ PASS: focusa_traverse ontology adapter remains bounded"
else
  echo "✗ FAIL: focusa_traverse ontology bounded adapter missing" >&2
  exit 1
fi

echo "SPEC96 ontology active-context traversal static test: PASS"
