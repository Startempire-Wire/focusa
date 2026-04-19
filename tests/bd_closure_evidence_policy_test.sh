#!/bin/bash
# Contract: closed beads must include evidence citations in close reason or notes.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ISSUES_FILE="${ROOT_DIR}/.beads/issues.jsonl"
CUTOFF_DATE="${BD_EVIDENCE_POLICY_CUTOFF:-2026-04-18}"

FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

# Enforce for closures on/after policy cutoff.
missing=$(jq -r --arg cutoff "$CUTOFF_DATE" '
  select(.status=="closed")
  | select((.closed_at // .updated_at // "") >= $cutoff)
  | . as $i
  | (($i.close_reason // "") + "\n" + ($i.notes // "")) as $text
  | ( ($text | test("Evidence citations:"; "i")) and
      ($text | test("tests/|docs/|crates/|/v1/|http"; "i")) ) as $ok
  | select($ok | not)
  | [.id, ($i.close_reason // ""), ($i.closed_at // $i.updated_at // "")] | @tsv
' "$ISSUES_FILE")

if [ -z "$missing" ]; then
  log_pass "All closed beads since ${CUTOFF_DATE} include evidence citations"
else
  echo "$missing" | while IFS=$'\t' read -r id reason ts; do
    log_fail "${id} missing evidence citation block (closed_at=${ts})"
    [ -n "$reason" ] && echo "    close_reason=${reason}"
  done
fi

echo "=== BD CLOSURE EVIDENCE POLICY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
