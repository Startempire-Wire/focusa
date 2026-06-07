#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TURNS="apps/pi-extension/src/turns.ts"
RUNTIME="tests/spec96_focus_slice_runtime_injection_test.mts"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in CONTEXT_RECEIPT included excluded omitted_bytes rehydrate_refs reason current_ask+Workpoint+trajectory_gap stale_or_advisory; do
  rg -F "$term" "$TURNS" >/dev/null || fail "Focus Slice builder missing context receipt term $term"
done
pass "Focus Slice builder declares context receipt fields"

rg -F 'CONTEXT_RECEIPT:' "$RUNTIME" >/dev/null || fail "runtime Focus Slice test must assert emitted context receipt"
pass "runtime test asserts context receipt emission"

python3 - <<'PY'
from pathlib import Path
src=Path('apps/pi-extension/src/turns.ts').read_text()
if 'contextReceiptHelpful' not in src:
    raise SystemExit('missing conditional receipt gate')
if '...contextReceiptLines' not in src:
    raise SystemExit('receipt not injected into Focus Slice lines')
receipt = src[src.index('const contextReceiptLines'):src.index('// §Prompt Serialization')]
for banned in ['previous issue', 'repair history', 'scar', 'debug']:
    if banned in receipt.lower():
        raise SystemExit(f'receipt contains repair/scar narration: {banned}')
print('✓ PASS: receipt is conditional and clean')
PY

echo "SPEC102 context receipt Focus Slice test: PASS"
