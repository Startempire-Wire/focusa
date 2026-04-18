#!/bin/bash
# Consumer-path contract: doc 74 reference-resolution outputs must feed projection and trace-review consumers.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/FIRST_CONSUMER_CANDIDATES_2026-04-13.md"
STATE_FILE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
TELEMETRY_ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/telemetry.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '### Doc 74 — reference resolution' "$DOC_FILE" >/dev/null 2>&1 && rg -n 'Selected first real consumer' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 74 section names a selected first consumer"
else
  log_fail "doc 74 section missing selected first consumer"
fi

if rg -n 'buildCanonicalReferenceAliases|REFERENCE_ALIASES|resolved_reference_count|resolved_reference_aliases' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 74 section anchors projection + trace-review evidence loci"
else
  log_fail "doc 74 section missing projection/trace-review evidence anchors"
fi

if rg -n 'export function buildCanonicalReferenceAliases\(' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "reference alias resolution helper exists"
else
  log_fail "reference alias resolution helper missing"
fi

if rg -n 'buildCanonicalReferenceAliases\(relevantVerifiedDeltas\.items\)' "$TURNS_FILE" >/dev/null 2>&1 && rg -n 'buildSliceSection\("canonical_references", "REFERENCE_ALIASES"' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "projection path consumes resolved references in REFERENCE_ALIASES"
else
  log_fail "projection path missing resolved reference consumer"
fi

if rg -n 'event_type: "verification_result"' "$TURNS_FILE" >/dev/null 2>&1 && rg -n 'resolved_reference_count|resolved_reference_aliases' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "trace emission includes resolved reference evidence"
else
  log_fail "trace emission missing resolved reference evidence"
fi

if rg -n '\.route\("/v1/telemetry/trace", get\(get_trace_events\)\)' "$TELEMETRY_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "trace-review route exists for emitted reference-resolution evidence"
else
  log_fail "trace-review route missing for emitted reference-resolution evidence"
fi

echo "=== DOC 74 REFERENCE RESOLUTION CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
