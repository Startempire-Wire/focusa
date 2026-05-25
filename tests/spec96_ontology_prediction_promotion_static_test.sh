#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ONT="${ROOT_DIR}/crates/focusa-api/src/routes/ontology.rs"

if rg -n 'record_memory_pipeline_prediction\(|ontology_memory_pipeline_promotion|write_predictions\(|read_predictions\(' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology memory pipeline records predictive follow-up signal"
else
  echo "✗ FAIL: ontology memory pipeline does not write prediction records" >&2
  exit 1
fi

if rg -n 'prediction_record|artifact_ref|source": "ontology_memory_pipeline"' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology memory pipeline response links prediction record to artifact"
else
  echo "✗ FAIL: ontology memory pipeline response lacks prediction linkage" >&2
  exit 1
fi

echo "SPEC96 Ontology prediction promotion static test: PASS"
