#!/usr/bin/env bash
# GH #3 / focusa-f6sn: Pi wrapper must show supporting-work fields without changing active scope.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$TOOLS" ] || fail "Pi tools missing"

for token in \
  'mismatch_reason=' \
  'packet_age=' \
  'action_authority_for_current_ask=false' \
  'scope_conflict_reason' \
  'safe_recovery: run focusa_project_verify' \
  'mission="' \
  'action="' \
  'return `${summary}${freshnessMarker}`'; do
  grep -Fq "$token" "$TOOLS" || fail "supporting-work wrapper field missing: $token"
done

if grep -Fq 'return `status=${status} id=${id} canonical=${canonical}${freshnessMarker} next=${next}`' "$TOOLS"; then
  fail "summarizeWorkpointResponse still discards mission/action summary"
fi

pass "supporting-work wrapper fields are visible"
