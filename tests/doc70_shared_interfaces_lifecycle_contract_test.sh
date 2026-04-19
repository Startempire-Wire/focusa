#!/bin/bash
# Semantic ontology contract bundle (higher-signal, runtime-first)
# Covers Doc70 shared lifecycle substrate + Doc72 identity/role + Doc75 projection/view semantics.
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

# Doc70: shared statuses/provenance/lifecycle substrate
if echo "$PRIMES_JSON" | jq -e '.status_vocabulary | index("proposed") and index("verified") and index("retired") and index("superseded")' >/dev/null 2>&1; then
  log_pass "doc70 shared status vocabulary projected at runtime"
else
  log_fail "doc70 shared status vocabulary incomplete"
fi

if echo "$PRIMES_JSON" | jq -e '.provenance_classes | index("runtime_observed") and index("verification_confirmed") and index("reducer_promoted")' >/dev/null 2>&1; then
  log_pass "doc70 shared provenance classes projected at runtime"
else
  log_fail "doc70 provenance classes incomplete"
fi

if echo "$PRIMES_JSON" | jq -e '.link_types | any((.name // .) == "transitions_to") and any((.name // .) == "allowed_for_role")' >/dev/null 2>&1; then
  log_pass "doc70 lifecycle/role link semantics projected"
else
  log_fail "doc70 lifecycle/role link semantics missing"
fi

# Doc72: identity-role-self-model vocabulary + actions
if echo "$PRIMES_JSON" | jq -e '.object_types | any((.type_name // .) == "agent_identity") and any((.type_name // .) == "role_profile") and any((.type_name // .) == "identity_state")' >/dev/null 2>&1; then
  log_pass "doc72 identity/role objects projected"
else
  log_fail "doc72 identity/role objects missing"
fi

if echo "$PRIMES_JSON" | jq -e '.link_types | any((.name // .) == "serves_role") and any((.name // .) == "governed_by_identity") and any((.name // .) == "bounded_by_handoff")' >/dev/null 2>&1; then
  log_pass "doc72 identity/role relations projected"
else
  log_fail "doc72 identity/role relations missing"
fi

if echo "$PRIMES_JSON" | jq -e '.action_types | any((.name // .) == "establish_identity") and any((.name // .) == "load_role_profile") and any((.name // .) == "restore_identity_continuity")' >/dev/null 2>&1; then
  log_pass "doc72 identity/role actions projected"
else
  log_fail "doc72 identity/role actions missing"
fi

# Doc75: projection/view semantics + contract target behavior
if echo "$PRIMES_JSON" | jq -e '.object_types | any((.type_name // .) == "projection") and any((.type_name // .) == "view_profile") and any((.type_name // .) == "projection_boundary")' >/dev/null 2>&1; then
  log_pass "doc75 projection/view objects projected"
else
  log_fail "doc75 projection/view objects missing"
fi

if echo "$PRIMES_JSON" | jq -e '.action_types | any((.name // .) == "build_projection") and any((.name // .) == "compress_projection") and any((.name // .) == "verify_projection_fidelity") and any((.name // .) == "switch_view_profile")' >/dev/null 2>&1; then
  log_pass "doc75 projection/view actions projected"
else
  log_fail "doc75 projection/view actions missing"
fi

if echo "$CONTRACTS_JSON" | jq -e '.contracts | any(.name == "build_projection" and (.target_types | index("projection_boundary") and index("view_profile"))) and any(.name == "switch_view_profile" and (.target_types | index("role_profile") and index("view_profile")))' >/dev/null 2>&1; then
  log_pass "doc75 action contracts preserve projection/view target semantics"
else
  log_fail "doc75 action contract target semantics missing"
fi

echo "=== DOC70/72/75 SEMANTIC ONTOLOGY CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
