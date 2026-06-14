#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/check-glossary-compliance"
GLOSSARY="$ROOT_DIR/docs/00-glossary.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -x "$SCRIPT" ]] || fail "missing executable glossary compliance gate"

for term in "Focus State" "Focus Stack" "Workpoint" "ProjectIdentity" "Continuity ID" "HLT" "MLG" "STG" "Waypoints" "Evidence Ref" "Context Cognition" "Context Authority" "Call Stack Design" "Focus Gate" "Intuition Engine" "Reference Store" "Expression Engine" "Canonical" "Advisory" "Degraded"; do
  rg -n "$term" "$GLOSSARY" >/dev/null || fail "glossary missing $term"
  rg -n "$term" "$SPEC" >/dev/null || fail "Spec106 missing $term"
done
pass "glossary and Spec106 preserve canonical vocabulary"

cd "$ROOT_DIR"
scripts/check-glossary-compliance >/tmp/focusa-glossary-compliance.out
rg -n 'glossary compliance gate passed' /tmp/focusa-glossary-compliance.out >/dev/null || fail "glossary gate did not report pass"
pass "glossary compliance gate runs"

echo "glossary compliance static test: PASS"
