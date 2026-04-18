#!/bin/bash
# Consumer-path contract: doc 76 retention policy must tier decisions/constraints into active vs decayed/historical surfaces.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/FIRST_CONSUMER_CANDIDATES_2026-04-13.md"
STATE_FILE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
COMMANDS_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/commands.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '### Doc 76 — retention / decay' "$DOC_FILE" >/dev/null 2>&1 && rg -n 'Selected first real consumer' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 76 section names a selected first consumer"
else
  log_fail "doc 76 section missing selected first consumer"
fi

if rg -n 'retentionBucketsFromSelection|DECAYED_CONTEXT|HISTORICAL_CONTEXT|memory\.decay_tick' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 76 section anchors projection retention tiers and decay command hook"
else
  log_fail "doc 76 section missing retention-tier consumer anchors"
fi

if rg -n 'export function retentionBucketsFromSelection\(' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "state helper classifies active/decayed/historical retention buckets"
else
  log_fail "retention bucket helper missing in state layer"
fi

if rg -n 'retentionBucketsFromSelection\(relevantDecisions|retentionBucketsFromSelection\(relevantConstraints' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "turn assembly consumes retention buckets for both decisions and constraints"
else
  log_fail "turn assembly missing retention-bucket consumers for decisions/constraints"
fi

if rg -n 'buildSliceSection\("decayed_context", "DECAYED_CONTEXT"|buildSliceSection\("historical_context", "HISTORICAL_CONTEXT"' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "slice projection exposes decayed and historical context tiers"
else
  log_fail "slice projection missing decayed/historical retention tiers"
fi

if rg -n 'retention_buckets: \{|decisions: \{|constraints: \{' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "trace metadata carries retention bucket counts"
else
  log_fail "trace metadata missing retention bucket counts"
fi

if rg -n '"memory\.decay_tick" => Ok\(Action::DecayTick\)' "$COMMANDS_FILE" >/dev/null 2>&1; then
  log_pass "commands route exposes explicit decay action hook"
else
  log_fail "commands route missing explicit decay action hook"
fi

echo "=== DOC 76 RETENTION POLICY CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
