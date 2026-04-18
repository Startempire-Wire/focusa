#!/bin/bash
# Contract test: QueryScope/reset semantics persist from input -> context boundary.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TURNS_TS="${REPO_ROOT}/apps/pi-extension/src/turns.ts"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

require_rg() {
  local pattern="$1"
  local file="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then
    log_pass "$label"
  else
    log_fail "$label"
  fi
}

echo "=== QueryScope boundary contract test ==="

require_rg 'const sourceTurnId = `pi-turn-\$\{S\.turnCount \+ 1\}`;' "$TURNS_TS" 'input uses next-turn id for QueryScope sourceTurnId'
require_rg 'const scopeSourceTurnId = S\.queryScope\?\.sourceTurnId \|\| S\.currentAsk\?\.sourceTurnId \|\| contextTurnId;' "$TURNS_TS" 'context reuses persisted QueryScope/currentAsk source turn id'
require_rg 'source_turn_id: scopeSourceTurnId' "$TURNS_TS" 'work-loop context sync uses persisted source_turn_id'
require_rg 'const resetReason = scopeKind === "fresh_question"' "$TURNS_TS" 'context defines explicit reset reason for fresh/correction scopes'
require_rg ': resetReason \|\| \(irrelevantExcludedLabels\.length \? "irrelevance" : "none"\);' "$TURNS_TS" 'reset reason survives into exclusionReason before irrelevance fallback'


echo ""
echo "=== Results ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"

if [ "$FAILED" -eq 0 ]; then
  exit 0
else
  exit 1
fi
