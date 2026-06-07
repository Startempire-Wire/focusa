#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
PI_CARD="apps/pi-extension/src/awareness.ts"
API_CARD="crates/focusa-api/src/routes/awareness.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$PI_CARD" "$API_CARD"; do
  for section in NOW_CARD WHY_CARD HEALTH_CARD DO_CARD; do
    rg -F "$section" "$file" >/dev/null || fail "$file missing $section"
  done
  for term in authority scope readiness why exact_next_action mutates rollback rehydrate_refs source_authority_order; do
    rg -F "$term" "$file" >/dev/null || fail "$file missing card contract term: $term"
  done
  pass "$file exposes Now/Why/Health/Do contract terms"
done

# Happy path contract: no repair/history/scar wording in the card section source.
python3 - <<'PY'
from pathlib import Path
for path in ['apps/pi-extension/src/awareness.ts','crates/focusa-api/src/routes/awareness.rs']:
    src = Path(path).read_text()
    card_terms = ['NOW_CARD', 'WHY_CARD', 'HEALTH_CARD', 'DO_CARD']
    windows = []
    for term in card_terms:
        i = src.index(term)
        windows.append(src[i:i+2200].lower())
    text = '\n'.join(windows)
    banned = ['previous issue', 'fixed', 'repair history', 'temporary warning', 'debug label']
    for b in banned:
        if b in text:
            raise SystemExit(f'{path} card section contains happy-path scar wording: {b}')
    if ' scar ' in text or 'scar text' in text:
        raise SystemExit(f'{path} card section contains happy-path scar wording: scar')
print('✓ PASS: card contracts avoid repair/scar narration')
PY

echo "SPEC102 Now/Why/Health/Do card contract test: PASS"
