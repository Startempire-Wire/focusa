#!/usr/bin/env bash
# AX GAP v3: metacog retrieve must not render long lessons as mid-sentence inline truncations.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$TOOLS" ] || fail "Pi tools missing"

for token in \
  'lessonRequiresRehydrate' \
  'see_rehydrate_ref:' \
  'lesson_inline_omitted' \
  'full_lesson_chars' \
  'rehydrate_full=true lesson_chars='; do
  grep -q "$token" "$TOOLS" || fail "metacog retrieve rehydrate-only lesson guard missing: $token"
done

if grep -q 'truncated inline' "$TOOLS"; then
  fail "metacog retrieve still says long lessons are truncated inline"
fi

pass "metacog retrieve uses rehydrate refs instead of inline truncation"
