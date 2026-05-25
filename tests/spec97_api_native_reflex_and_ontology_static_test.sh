#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FOCUS="$ROOT_DIR/crates/focusa-api/src/routes/focus.rs"
WORKPOINT="$ROOT_DIR/crates/focusa-api/src/routes/workpoint.rs"
TRAJECTORY="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"
TRAVERSE="$ROOT_DIR/crates/focusa-api/src/routes/traverse.rs"
REFLEX="$ROOT_DIR/crates/focusa-api/src/routes/reflex.rs"
ONTOLOGY="$ROOT_DIR/crates/focusa-api/src/routes/ontology.rs"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'pub fn reflex_suggestions_for_failure|diagnose_scope_mismatch|resource_mode_fallback|retry_safe_pending|preflight_writer_ownership' "$REFLEX" >/dev/null || fail "API-native reflex suggestion helper missing"
pass "API-native reflex suggestion helper exists"

for f in "$FOCUS" "$WORKPOINT" "$TRAJECTORY" "$TRAVERSE"; do
  rg -n 'reflex_suggestions_for_failure|"reflex_suggestions"' "$f" >/dev/null || fail "$(basename "$f") lacks API-native reflex_suggestions"
done
pass "Core API envelopes expose reflex_suggestions"

rg -n 'reflex_primitive|reflex_trigger|reflex_action|reflex_risk|reflex_affordance|route_reflex_primitive|suggest_reflex_recovery|inspect_reflex_registry' "$ONTOLOGY" >/dev/null || fail "Ontology lacks Spec97 reflex object/action classes"
pass "Ontology exposes Spec97 reflex object/action classes"

rg -n 'API-native|ontology classes|runtime dogfood|hot-index|live runtime dogfood' "$SPEC" >/dev/null || fail "Spec97 gap closures not documented"
pass "Spec97 documents gap closure surfaces"

echo "SPEC97 API-native reflex and ontology static test: PASS"
