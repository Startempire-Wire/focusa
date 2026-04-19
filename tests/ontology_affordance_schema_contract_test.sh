#!/bin/bash
# Runtime contract test: affordance ontology schemas must be projected through public endpoints.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

PRIMES_JSON="$(curl -sS "${BASE_URL}/v1/ontology/primitives")"
CONTRACTS_JSON="$(curl -sS "${BASE_URL}/v1/ontology/contracts")"

if echo "$PRIMES_JSON" | jq -e '.object_types | any((.type_name // .) == "capability") and any((.type_name // .) == "tool_surface") and any((.type_name // .) == "permission") and any((.type_name // .) == "authority_boundary") and any((.type_name // .) == "execution_context") and any((.type_name // .) == "affordance")' >/dev/null 2>&1; then
  log_pass "affordance object families projected"
else
  log_fail "affordance object families missing from primitives projection"
fi

if echo "$PRIMES_JSON" | jq -e '.link_types | any((.name // .) == "enabled_by") and any((.name // .) == "requires_permission") and any((.name // .) == "bounded_by_authority") and any((.name // .) == "consumes_resource") and any((.name // .) == "available_in_context") and any((.name // .) == "supports_execution_of")' >/dev/null 2>&1; then
  log_pass "affordance execution relation types projected"
else
  log_fail "affordance execution relation types missing"
fi

if echo "$PRIMES_JSON" | jq -e '.action_types | any((.name // .) == "detect_affordances") and any((.name // .) == "verify_permissions") and any((.name // .) == "verify_preconditions") and any((.name // .) == "estimate_reliability") and any((.name // .) == "choose_execution_path") and any((.name // .) == "mark_unavailable")' >/dev/null 2>&1; then
  log_pass "affordance execution action types projected"
else
  log_fail "affordance execution action types missing"
fi

if echo "$CONTRACTS_JSON" | jq -e '.contracts | any(.name=="detect_affordances" and (.target_types | index("affordance") and index("execution_context"))) and any(.name=="verify_permissions" and (.target_types | index("permission") and index("authority_boundary"))) and any(.name=="choose_execution_path" and (.target_types | index("affordance") and index("task")))' >/dev/null 2>&1; then
  log_pass "affordance action contracts preserve typed target semantics"
else
  log_fail "affordance action contract target semantics missing"
fi

echo "=== ONTOLOGY AFFORDANCE SCHEMA CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
