#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TOOLS="apps/pi-extension/src/tools.ts"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'metacog capture: id=${captureId} lesson="${lessonLine}" why="${relevanceReason}" rehydrate_id=${captureId}' \
  'metacog retrieve: candidates=${total} top_lesson="${topLesson}" why="${topWhy}" rehydrate_id=${topCapture}' \
  'compact_lesson_line' \
  'compact_top_lesson' \
  'rehydrate_id' \
  'why_relevant'; do
  rg -F "$needle" "$TOOLS" >/dev/null || fail "missing metacog compact lesson marker: $needle"
done
pass "Pi metacog compact render includes lesson, relevance reason, rehydrate id"

python3 - <<'PY'
from pathlib import Path
src = Path('apps/pi-extension/src/tools.ts').read_text()
capture = src[src.index('name: "focusa_metacog_capture"'):src.index('name: "focusa_metacog_retrieve"')]
retrieve = src[src.index('name: "focusa_metacog_retrieve"'):src.index('name: "focusa_metacog_reflect"')]
for label, block in [('capture', capture), ('retrieve', retrieve)]:
    for term in ['lesson', 'why', 'rehydrate_id']:
        if term not in block:
            raise SystemExit(f'{label} block missing {term}')
    if 'next_tools=' in block:
        raise SystemExit(f'{label} normal compact output still uses verbose next_tools debug line')
print('✓ PASS: metacog compact blocks are short and action-oriented')
PY

echo "SPEC102 metacog compact lesson-line test: PASS"
