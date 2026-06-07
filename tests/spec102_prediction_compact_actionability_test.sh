#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TOOLS="apps/pi-extension/src/tools.ts"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'prediction record → ${body.status || "accepted"} id=${predictionId} confidence=${predictionConfidence} scope=(${predictionScope}) eval_hint="${predictionEvalHint}"' \
  'predictions recent → ${count}${actionLine}' \
  'next_id=${String(actionable.prediction_id)} confidence=${String(actionable.confidence ?? "unknown")} scope=(project=${String(actionable.project_root || "unknown")} continuity=${String(actionable.continuity_id || "unknown")}) eval_hint="focusa_predict_evaluate prediction_id=${String(actionable.prediction_id)}"' \
  'compact_actionability' \
  'focusa_predict_evaluate prediction_id='; do
  rg -F "$needle" "$TOOLS" >/dev/null || fail "missing prediction compact actionability marker: $needle"
done
pass "Pi prediction compact render includes id/confidence/scope/evaluation hint"

python3 - <<'PY'
from pathlib import Path
src = Path('apps/pi-extension/src/tools.ts').read_text()
record = src[src.index('name: "focusa_predict_record"'):src.index('name: "focusa_predict_recent"')]
recent = src[src.index('name: "focusa_predict_recent"'):src.index('name: "focusa_predict_evaluate"')]
for label, block in [('record', record), ('recent', recent)]:
    for term in ['prediction_id', 'confidence', 'project_root', 'continuity_id', 'focusa_predict_evaluate']:
        if term not in block:
            raise SystemExit(f'{label} block missing {term}')
    if block.count('\n') > 55:
        raise SystemExit(f'{label} compact implementation too large/noisy')
print('✓ PASS: prediction record/recent blocks are compact and self-evaluable')
PY

echo "SPEC102 prediction compact actionability test: PASS"
