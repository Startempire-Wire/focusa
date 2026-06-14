#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n -F 'Status: implemented-and-final-audited' "$SPEC" >/dev/null || fail "Spec106 top status is not implemented-and-final-audited"
if rg -n 'Status: (partial|gap)' "$SPEC" >/tmp/spec106-status-gaps.txt; then
  cat /tmp/spec106-status-gaps.txt >&2
  fail "Spec106 still contains partial/gap status markers"
fi
pass "Spec106 status map has no partial/gap markers"

for marker in \
  'tests/pi_extension_final_toolset_audit_static_test.sh' \
  'tests/pi_extension_contract_test.sh' \
  'tests/vision_vocabulary_static_test.sh' \
  'tests/glossary_compliance_static_test.sh' \
  'tests/authority_model_static_test.sh' \
  'tests/authority_scope_static_test.sh' \
  'tests/golden_workflow_static_test.sh' \
  'tests/golden_workflow_live_safe_test.sh' \
  'docs/current/PI_EXTENSION_FINAL_TOOLSET_AUDIT.md'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing completion proof marker $marker"
done
pass "Spec106 references completion proof gates"

if command -v bd >/dev/null 2>&1 && [ -d "$ROOT_DIR/.beads" ]; then
  parent_status=$(cd "$ROOT_DIR" && bd show focusa-husq --json | jq -r '(if type=="array" then .[0] else . end) | .status')
  [ "$parent_status" = "closed" ] || fail "focusa-husq parent bead is not closed"
  open_children=$(cd "$ROOT_DIR" && bd list --json | jq -r '[.[] | select(.id|startswith("focusa-husq.") and .status != "closed")] | length')
  [ "$open_children" = "0" ] || fail "open focusa-husq child beads remain: $open_children"
  pass "bead closure state has closed parent and zero open children"
else
  echo "SKIP: bd unavailable; static Spec106 status checks already passed"
fi

echo "Spec106 completion status static test: PASS"
