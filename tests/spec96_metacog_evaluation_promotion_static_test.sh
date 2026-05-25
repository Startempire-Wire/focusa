#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
META="${ROOT_DIR}/crates/focusa-api/src/routes/metacognition.rs"

if rg -n 'struct EvaluationRecord|evaluations: Vec<EvaluationRecord>|metacog_record_path\(&state, "evaluations"|load_evaluation_records_from_disk' "$META" >/dev/null; then
  echo "✓ PASS: metacognition evaluations persist as first-class records"
else
  echo "✗ FAIL: metacognition evaluations are not persisted as records" >&2
  exit 1
fi

if rg -n 'kind: "promoted_learning"\.to_string\(\)|promoted_capture_id|append_capture_index_entry\(&state, &index_entry\)|promoted learning was written back into metacognition retrieval memory' "$META" >/dev/null; then
  echo "✓ PASS: promoted evaluations write learning back into retrieval memory"
else
  echo "✗ FAIL: metacognition evaluate does not promote learning into retrieval memory" >&2
  exit 1
fi

if rg -n '"evaluation_memory"|"evaluations_recorded"|"promoted_evaluations"' "$META" >/dev/null; then
  echo "✓ PASS: metacognition status exposes evaluation memory counters"
else
  echo "✗ FAIL: metacognition status omits evaluation memory counters" >&2
  exit 1
fi

echo "SPEC96 Metacog evaluation promotion static test: PASS"
