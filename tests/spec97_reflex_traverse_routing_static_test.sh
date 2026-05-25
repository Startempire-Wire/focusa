#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAVERSE="$ROOT_DIR/crates/focusa-api/src/routes/traverse.rs"
REGISTRY="$ROOT_DIR/docs/current/focusa-reflex-primitives.json"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'reflex_primitive_items|focusa-reflex-primitives\.json|"reflex" \| "reflexes" \| "reflex_primitives"' "$TRAVERSE" >/dev/null || fail "traverse route lacks reflex primitive surface"
pass "traverse exposes reflex/reflex_primitives surface"

rg -n 'primitive_id|family|context_inputs|reflex_action|escalation_boundary|authority_boundary|hot_path_budget|failure_envelope' "$TRAVERSE" >/dev/null || fail "reflex traverse defaults do not expose required primitive fields"
pass "reflex traverse field projection covers required fields"

rg -n 'reflex_primitive_surface_returns_registry_backed_family_items|route_noncanonical_result|family.*recovery' "$TRAVERSE" >/dev/null || fail "Rust regression test for registry-backed reflex family traversal missing"
pass "Rust regression test covers family-filtered reflex traversal"

jq -e '.primitives | map(select(.family == "recovery" and .primitive_id == "route_noncanonical_result")) | length == 1' "$REGISTRY" >/dev/null || fail "registry missing recovery primitive used by traverse test"
pass "registry contains recovery primitive used by traverse"

rg -n 'G97-ontology-reflex-routing|focusa_traverse|reflex_primitives' "$SPEC" >/dev/null || fail "Spec97 does not track reflex traverse routing"
pass "Spec97 tracks reflex traverse routing"

echo "SPEC97 reflex traverse routing static test: PASS"
