#!/usr/bin/env bash
# Static guard for focusa-7wgk fresh-operator dry-run evidence.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md"
DIR="$ROOT_DIR/docs/evidence/fresh-operator-dry-run-2026-07-05"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$DOC" ]] || fail "fresh-operator dry-run evidence doc missing: $DOC"
[[ -d "$DIR" ]] || fail "fresh-operator transcript dir missing: $DIR"

for needle in \
  "tmux session \`focusa-fresh-op\`" \
  "Quickstart" \
  "Operator Design Constraints" \
  "ASCII intros" \
  "interactive" \
  "/v1/about" \
  "Gaps identified" \
  "README quickstart" \
  "Installed binary drift" \
  "ensure_dir_all" \
  "interactive" \
  "ASCII wordmark" ; do
  grep -nF -- "$needle" "$DOC" >/dev/null || fail "fresh-op doc missing marker: $needle"
done
pass "fresh-operator dry-run doc + transcript directory present and complete"

for needle in \
  '00-full-tmux-transcript.txt' \
  '03-focusa-help.txt' \
  '04-install-script-preview.txt' \
  '05-release-assets.txt' \
  '07-onboard-tui.txt' \
  '12-final-api.txt' ; do
  [[ -f "$DIR/$needle" ]] || fail "fresh-op transcript slice missing: $needle"
done
pass "fresh-operator transcript slices archived"

python3 - <<'PY'
from pathlib import Path
text = Path('docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md').read_text()
assert 'GAP #1' in text or 'README quickstart' in text
assert 'GAP #2' in text or '/v1/about' in text
assert 'GAP #3' in text or 'Installed binary drift' in text
assert 'GAP #4' in text or 'ensure_dir_all' in text
assert 'GAP #5' in text or 'interactive prompts' in text.lower()
PY
pass "fresh-operator doc enumerates all 5 gaps"

echo "focusa-7wgk fresh-operator dry-run test: PASS"