#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TOOLS="apps/pi-extension/src/tools.ts"
SCOPED="apps/pi-extension/src/scoped-state.ts"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'content: [{ type: "text", text: renderScopedResultHuman(body) }]' \
  'evaluation_hint: `focusa_predict_evaluate prediction_id=${predictionId}`' \
  'next_prediction_id: hint.prediction_id || null' \
  'evaluate_hint: hint' \
  'focusa_predict_evaluate prediction_id='; do
  rg -F "$needle" "$TOOLS" >/dev/null || fail "missing prediction human/actionability marker: $needle"
done
for needle in \
  'human_readable?: string' \
  'Human readable: ${body.human_readable}'; do
  rg -F "$needle" "$SCOPED" >/dev/null || fail "missing scoped human-readable marker: $needle"
done
pass "Pi prediction render preserves human-readable output and evaluation hints"

python3 - <<'PY'
from pathlib import Path
src = Path('apps/pi-extension/src/tools.ts').read_text()
record = src[src.index('name: "focusa_predict_record"'):src.index('name: "focusa_predict_recent"')]
recent = src[src.index('name: "focusa_predict_recent"'):src.index('name: "focusa_predict_evaluate"')]
required = {
    'record': [
        'prediction_id', 'confidence', 'project_root', 'continuity_id', 'evaluation_hint',
        'renderScopedResultHuman', 'object_refs', 'action_refs', 'tool_refs', 'evidence_refs',
        'relation_refs',
    ],
    'recent': ['predictions', 'next_prediction_id', 'evaluate_hint', 'scope', 'renderScopedResultHuman'],
}
for label, block in [('record', record), ('recent', recent)]:
    for term in required[label]:
        if term not in block:
            raise SystemExit(f'{label} block missing {term}')
print('✓ PASS: prediction record/recent blocks are human-readable, rolling-compatible, and self-evaluable')
PY

echo "SPEC102 prediction compact actionability test: PASS"
