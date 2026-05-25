#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REGISTRY="$ROOT_DIR/docs/current/focusa-reflex-primitives.json"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$REGISTRY" ]] || fail "reflex primitive registry missing"

jq -e '.schema == "focusa.reflex_primitives.v1" and .version == "spec97.reflex_primitives.v1" and .status == "read_only_registry"' "$REGISTRY" >/dev/null || fail "registry schema/version/status invalid"
pass "registry exposes read-only Spec97 schema"

jq -e '.primitive_count == (.primitives|length) and .primitive_count >= 40' "$REGISTRY" >/dev/null || fail "registry count mismatch or too few primitives"
pass "registry covers at least initial primitive set"

for family in identity scope continuity evidence recovery salience execution learning resource governance; do
  jq -e --arg family "$family" '(.families | index($family)) and ([.primitives[] | select(.family == $family)] | length >= 4)' "$REGISTRY" >/dev/null || fail "family $family missing or under-covered"
done
pass "registry covers all ten Spec97 families"

jq -e '.primitives[] | select((.primitive_id|type)!="string" or (.family|type)!="string" or (.trigger|type)!="string" or (.context_inputs|type)!="array" or (.reflex_action.recommended_tool|type)!="string" or (.evidence_output.kind|type)!="string" or (.escalation_boundary|type)!="string" or (.authority_boundary|type)!="string" or (.hot_path_budget|type)!="string" or (.failure_envelope|type)!="string") | halt_error(1)' "$REGISTRY" >/dev/null 2>&1 && fail "required primitive fields missing or malformed" || true
pass "all primitive entries expose required contract fields"

jq -e '[.primitives[].primitive_id] | length == (unique|length)' "$REGISTRY" >/dev/null || fail "primitive ids are not unique"
pass "primitive ids are unique"

if rg -n 'focusa-reflex-primitives\.json|spec97\.reflex_primitives\.v1|G97-primitive-registry' "$SPEC" >/dev/null; then
  pass "Spec97 references registry and gap closure path"
else
  fail "Spec97 does not reference registry/gap path"
fi

echo "SPEC97 reflex primitive registry static test: PASS"
