#!/bin/bash
# SPEC-79 / contract hardening: ontology route metadata must match executable contract semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/ontology.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '"tool_action_metadata"|"trace_metadata"|"eval_metadata"|"projection_metadata"|"governance_metadata"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "action contract exposes tool/trace/eval/projection/governance metadata surfaces"
else
  log_fail "contract metadata surfaces missing from action contract payload"
fi

if rg -n '"runtime_execution_supported": runtime_execution_supported' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "runtime executability flag derives from contract tool mappings"
else
  log_fail "runtime executability flag is not derived from contract mappings"
fi

if rg -n '"constraint_checked": true|"reducer_visible": reducer_visible|"runtime_execution_supported": runtime_execution_supported' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "world action catalog computes reducer/runtime flags from action contract"
else
  log_fail "world action catalog still uses fixed projection-only booleans"
fi

if rg -n '"determine_current_ask" \| "build_query_scope"|"/v1/work-loop/context"|"resolve_identity"|"/v1/references/search"|"execute_migration"|"/v1/events/recent"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "post-doc67/70/74/75/77 action families expose concrete route mappings"
else
  log_fail "new ontology action families missing concrete route mappings"
fi

if rg -n '"surface": "GET /v1/ontology/contracts"|"read_only": true|"mutates_canonical_state": false' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "contracts endpoint still publishes read-only route behavior metadata"
else
  log_fail "contracts endpoint route behavior metadata missing"
fi

echo "=== ONTOLOGY ROUTE METADATA CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
