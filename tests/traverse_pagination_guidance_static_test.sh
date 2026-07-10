#!/usr/bin/env bash
# AX GAP v3: focusa_traverse must tell callers whether/how to paginate.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRAVERSE="$ROOT_DIR/crates/focusa-api/src/routes/traverse.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$TRAVERSE" ] || fail "traverse route missing"
[ -f "$TOOLS" ] || fail "Pi tools missing"

for token in \
  'more_available' \
  'pagination_guidance' \
  'Re-run focusa_traverse with cursor=' \
  'same surface/selector/query/limit' \
  'No additional page is available'; do
  grep -q "$token" "$TRAVERSE" || fail "API traverse pagination guidance missing: $token"
done

for token in \
  'more_available=' \
  'guidance=' \
  'pagination_guidance'; do
  grep -q "$token" "$TOOLS" || fail "Pi traverse text pagination guidance missing: $token"
done

pass "traverse pagination guidance is visible"
