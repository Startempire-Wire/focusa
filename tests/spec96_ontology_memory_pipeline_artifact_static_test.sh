#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ONT="${ROOT_DIR}/crates/focusa-api/src/routes/ontology.rs"

if rg -n 'ontology_runtime_dir\(|persist_ontology_artifact\(|runtime.*ontology.*memory-pipeline' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology memory pipeline has durable runtime artifact persistence"
else
  echo "✗ FAIL: ontology memory pipeline lacks durable artifact persistence" >&2
  exit 1
fi

if rg -n 'focusa\.ontology\.memory_pipeline_artifact\.v1|promotion_target|durable_artifact|storage_path|semantic_metacog_candidate|procedural_playbook_candidate' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology memory pipeline emits durable promotion artifact metadata"
else
  echo "✗ FAIL: ontology memory pipeline lacks promotion artifact metadata" >&2
  exit 1
fi

if rg -n 'semantic_ready.*persist_ontology_artifact|persist_ontology_artifact\(&state, "memory-pipeline"' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology memory pipeline writes artifacts only after semantic/eval gate"
else
  echo "✗ FAIL: ontology memory pipeline write is not tied to semantic/eval gate" >&2
  exit 1
fi

echo "SPEC96 Ontology memory pipeline artifact static test: PASS"
