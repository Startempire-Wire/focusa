#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BOUNDED_RS="${ROOT_DIR}/crates/focusa-api/src/routes/bounded.rs"

if rg -n 'CursorWindow|cursor_window|bounded_window|next_cursor|previous_cursor' "$BOUNDED_RS" >/dev/null; then
  echo "✓ PASS: shared cursor/window helpers exist"
else
  echo "✗ FAIL: cursor/window helpers missing" >&2
  exit 1
fi

if rg -n 'FieldProjection|field_projection|project_json_fields|requested.*applied.*omitted' "$BOUNDED_RS" >/dev/null; then
  echo "✓ PASS: field projection helpers exist"
else
  echo "✗ FAIL: field projection helpers missing" >&2
  exit 1
fi

if rg -n 'TraversalBounds|traversal_bounds|max_depth|max_path_segments|omitted_path_segments' "$BOUNDED_RS" >/dev/null; then
  echo "✓ PASS: path/depth traversal guard helpers exist"
else
  echo "✗ FAIL: path/depth traversal guard helpers missing" >&2
  exit 1
fi

if rg -n 'BoundedReadMetadata|bounded_metadata|full_payload_blocked_by_pressure|include_full_payload|RehydrateHint' "$BOUNDED_RS" >/dev/null; then
  echo "✓ PASS: metadata and cold full-payload guards exist"
else
  echo "✗ FAIL: metadata/full-payload guards missing" >&2
  exit 1
fi

# Representative large/list surfaces must consume the bounded helper layer.
declare -A surfaces=(
  [lineage]="crates/focusa-api/src/routes/clt.rs:bounded_window|field_projection|traversal_bounds|bounded_metadata"
  [ontology]="crates/focusa-api/src/routes/ontology.rs:BoundedReadOptions|bounded_metadata|full_payload_blocked_by_pressure"
  [references_evidence]="crates/focusa-api/src/routes/ecs.rs:BoundedReadOptions|bounded_metadata|include_full_payload"
  [metacog]="crates/focusa-api/src/routes/metacognition.rs:budgeted_requested_limit|cursor|next_cursor"
  [telemetry]="crates/focusa-api/src/routes/telemetry.rs:budgeted_requested_limit|bounded_metadata|metadata"
  [snapshots]="crates/focusa-api/src/routes/snapshots.rs:budgeted_requested_limit|bounded_metadata|metadata"
  [workpoint]="crates/focusa-api/src/routes/workpoint.rs:bounded_metadata|active_object_refs_metadata|verification_records_metadata"
)

for surface in "${!surfaces[@]}"; do
  file="${surfaces[$surface]%%:*}"
  pattern="${surfaces[$surface]#*:}"
  if rg -n "$pattern" "${ROOT_DIR}/${file}" >/dev/null; then
    echo "✓ PASS: $surface surface uses bounded traversal helpers"
  else
    echo "✗ FAIL: $surface surface does not use bounded traversal helpers" >&2
    exit 1
  fi
done

echo "SPEC96 bounded traversal helpers static test: PASS"
