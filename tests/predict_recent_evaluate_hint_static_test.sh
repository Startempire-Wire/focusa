#!/usr/bin/env bash
# AX GAP v3: focusa_predict_recent must explain whether to evaluate or record a prediction.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PRED="$ROOT_DIR/crates/focusa-api/src/routes/predictions.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$PRED" ] || fail "predictions route missing"
[ -f "$TOOLS" ] || fail "Pi tools missing"

for token in \
  'fn prediction_evaluate_hint' \
  'age_hours' \
  'high confidence unevaluated prediction should be checked' \
  'prediction is old enough' \
  'next_prediction_id' \
  '"evaluate_hint"'; do
  grep -q "$token" "$PRED" || fail "API predict recent evaluate hint missing: $token"
done

for token in \
  'apiEvaluateHint' \
  'evaluation_hint' \
  'evaluate_hint' \
  'age='; do
  grep -q "$token" "$TOOLS" || fail "Pi predict recent evaluate hint missing: $token"
done

pass "predict recent evaluate hint is visible"
