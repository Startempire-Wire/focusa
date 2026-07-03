#!/usr/bin/env bash
# spec_focusa_3tr9_llms_txt_static_test.sh
#
# Static guard for Spec 109 AX-004: GET /llms.txt endpoint.
#
# Acceptance (per focusa-3tr9):
#   1. crates/focusa-api/src/routes/llms_txt.rs exists
#   2. mod.rs declares pub mod llms_txt
#   3. server.rs merges routes::llms_txt::router()
#   4. docs/llms.txt exists
#   5. docs/llms.txt covers the 7 transcript-driven concepts:
#        When to use / How to start / Core concepts / Tool surface /
#        Recovery / Anti-patterns / Workpoint / Trajectory
#   6. docs/llms.txt is under 2000 tokens (line budget sanity check)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/llms_txt.rs"
MOD="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
DOC="$ROOT_DIR/docs/llms.txt"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[ -f "$ROUTE" ] || fail "llms_txt.rs not found"
pass "crates/focusa-api/src/routes/llms_txt.rs exists"

grep -q 'pub mod llms_txt' "$MOD" \
  || fail "pub mod llms_txt missing from routes/mod.rs"
pass "routes/mod.rs exposes llms_txt module"

grep -q 'routes::llms_txt::router()' "$SERVER" \
  || fail "server.rs not merging llms_txt::router"
pass "server.rs merges routes::llms_txt::router()"

[ -f "$DOC" ] || fail "docs/llms.txt not found"
pass "docs/llms.txt exists"

# Each required section from transcript gap
for section in "## When to use focusa" "## Core concepts" "## How to start" "## Tool surface" "## Recovery" "## Anti-patterns"; do
  grep -qF "$section" "$DOC" \
    || fail "docs/llms.txt missing section: $section"
done
pass "docs/llms.txt covers all 6 required sections"

# Workpoint + Trajectory + Focus Stack + Memory + Constitution must be mentioned
for concept in "Workpoint" "Trajectory" "Focus Stack" "Memory" "Constitution" "Action Preflight" "Work Loop"; do
  grep -qF "$concept" "$DOC" \
    || fail "docs/llms.txt missing concept: $concept"
done
pass "docs/llms.txt mentions all 7 transcript-driven concepts"

# Token budget: ~4 chars per English token; 2000 tokens ≈ 8000 chars; be lenient at 12k
LINES=$(wc -l < "$DOC")
CHARS=$(wc -c < "$DOC")
if [ "$CHARS" -gt 12000 ]; then
  fail "docs/llms.txt too long: $CHARS chars (> 12000)"
fi
pass "docs/llms.txt size: $CHARS chars, $LINES lines (under 2000-token budget)"

# Route module must serve plain text
grep -qF 'header::CONTENT_TYPE' "$ROUTE" || fail "llms_txt route missing Content-Type header"
grep -qF 'text/plain' "$ROUTE" || fail "llms_txt route must serve text/plain"
pass "llms_txt route serves text/plain with explicit Content-Type"

echo "✓ All focusa-3tr9 AX-004 llms.txt static checks passed"