#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ $*"; }

rg -n 'route\("/v1/predictions/capture-outcome"' crates/focusa-api/src/routes/predictions.rs >/dev/null \
  || fail "prediction capture-outcome API route missing"
rg -n 'capture_learning_signal\(' crates/focusa-api/src/routes/predictions.rs >/dev/null \
  || fail "prediction outcomes do not feed metacognition capture"
rg -n 'append_prediction_record\(' crates/focusa-api/src/routes/metacognition.rs >/dev/null \
  || fail "metacognition evaluations do not create follow-up predictions"
rg -n 'ontology_context' crates/focusa-api/src/routes/predictions.rs crates/focusa-api/src/routes/metacognition.rs crates/focusa-cli/src/commands/predict.rs >/dev/null \
  || fail "ontology_context is not wired through prediction/metacog flywheel"
rg -n 'CaptureOutcome' crates/focusa-cli/src/commands/predict.rs >/dev/null \
  || fail "CLI capture-outcome command missing"

pass "prediction/metacog flywheel static contract present"
