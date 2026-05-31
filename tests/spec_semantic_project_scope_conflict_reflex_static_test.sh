#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY="$ROOT_DIR/docs/current/focusa-reflex-primitives.json"
REFLEX_RS="$ROOT_DIR/crates/focusa-api/src/routes/reflex.rs"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TOOLS_TS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
SCHEMA="$ROOT_DIR/docs/contracts/focusa-tool-result-schema-v1.json"
SPEC97="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

jq -e '
  .primitive_count == (.primitives|length)
  and (.primitives[] | select(.primitive_id == "detect_semantic_project_scope_conflict"
    and .family == "scope"
    and (.trigger | contains("before_tool_api_scope_mismatch"))
    and (.context_inputs | index("CurrentAsk"))
    and (.context_inputs | index("ProjectSwitchLedger"))
    and (.context_inputs | index("Workpoint"))
    and .reflex_action.recommended_tool == "focusa_project_verify"
    and .evidence_output.object == "CurrentScopeVerdict"
    and .evidence_output.authority_field == "action_authority_for_current_ask"
    and (.output_fields | index("action_authority_for_current_ask"))
    and (.failure_envelope | contains("scope_conflict"))))
' "$REGISTRY" >/dev/null || fail "semantic project-scope-conflict primitive missing CurrentScopeVerdict contract"
pass "Registry includes semantic project-scope-conflict CurrentScopeVerdict primitive"

rg -n 'scope_conflict.*detect_semantic_project_scope_conflict|semantic_project_scope_conflict_primitive_outputs_current_scope_verdict' "$REFLEX_RS" >/dev/null \
  || fail "API reflex suggestions/tests do not expose semantic scope-conflict primitive"
pass "API reflex surface suggests and tests semantic scope-conflict primitive"

rg -n 'scope_conflict|detect_semantic_project_scope_conflict|action_authority_for_current_ask=false' "$TOOLS_TS" >/dev/null \
  || fail "Pi tool envelope does not classify/suggest semantic scope conflict"
pass "Pi tool envelopes classify semantic scope conflict"

jq -e '.properties.failure_class.enum | index("scope_conflict")' "$SCHEMA" >/dev/null \
  || fail "tool_result_v1 schema does not allow scope_conflict"
pass "tool_result_v1 schema allows scope_conflict"

rg -n 'interface PiCurrentAskScopeVerdict|buildCurrentAskScopeVerdict|action_authority_for_current_ask.*false|focusa_project_verify.*focusa_project_identity.*focusa_workpoint_checkpoint' "$STATE_TS" >/dev/null \
  || fail "CurrentScopeVerdict path cannot suppress action before API scope_mismatch"
pass "CurrentScopeVerdict can suppress action before API scope_mismatch"

rg -n 'detect_semantic_project_scope_conflict.*CurrentScopeVerdict.*action_authority_for_current_ask=false|G97-primitive-registry' "$SPEC97" >/dev/null \
  || fail "Spec97 does not document semantic project-scope-conflict primitive"
pass "Spec97 documents semantic project-scope-conflict primitive"

echo "Semantic project scope-conflict reflex static test: PASS"
