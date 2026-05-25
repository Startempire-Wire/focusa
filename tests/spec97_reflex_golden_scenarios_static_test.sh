#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCENARIOS="$ROOT_DIR/docs/current/spec97-reflex-golden-scenarios.json"
REGISTRY="$ROOT_DIR/docs/current/focusa-reflex-primitives.json"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
TRAVERSE="$ROOT_DIR/crates/focusa-api/src/routes/traverse.rs"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$SCENARIOS" ]] || fail "golden scenarios registry missing"
jq -e '.schema == "focusa.reflex_golden_scenarios.v1" and .scenario_count == (.scenarios|length) and .scenario_count >= 5' "$SCENARIOS" >/dev/null || fail "scenario registry schema/count invalid"
pass "golden scenario registry exists with at least five scenarios"

python3 - "$SCENARIOS" "$REGISTRY" "$ROOT_DIR" <<'PY'
import json, os, sys
scenarios=json.load(open(sys.argv[1]))['scenarios']
registry=json.load(open(sys.argv[2]))['primitives']
ids={p['primitive_id'] for p in registry}
root=sys.argv[3]
required_fields=['scenario_id','family','trigger','context_inputs','primitive_chain','functional_surfaces','evidence_output','escalation_boundary','acceptance_probe']
for s in scenarios:
    missing=[f for f in required_fields if f not in s or not s[f]]
    if missing:
        raise SystemExit(f"{s.get('scenario_id')} missing fields {missing}")
    missing_ids=[p for p in s['primitive_chain'] if p not in ids]
    if missing_ids:
        raise SystemExit(f"{s['scenario_id']} references missing primitives {missing_ids}")
    probe=os.path.join(root, s['acceptance_probe'])
    if not os.path.exists(probe):
        raise SystemExit(f"{s['scenario_id']} probe missing: {s['acceptance_probe']}")
families={s['family'] for s in scenarios}
if len(families) < 5:
    raise SystemExit(f"expected at least five families, got {sorted(families)}")
PY
pass "scenarios reference registry-backed primitives and existing probes"

rg -n 'reflexSuggestionsForFailure|reflex_suggestions' "$TOOLS" >/dev/null || fail "envelope reflex suggestions not implemented"
rg -n 'reflex_primitive_items|reflex_primitives' "$TRAVERSE" >/dev/null || fail "traverse reflex primitive routing not implemented"
pass "scenarios have implemented envelope and traverse substrates"

rg -n 'spec97-reflex-golden-scenarios\.json|G97-golden-reflex-scenarios|Trigger -> Context -> Reflex -> Evidence -> Escalation' "$SPEC" >/dev/null || fail "Spec97 does not reference golden scenario proof"
pass "Spec97 references golden scenario proof path"

echo "SPEC97 reflex golden scenarios static test: PASS"
