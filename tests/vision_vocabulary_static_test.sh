#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in README.md BENEFITS.md docs/106-focusa-vision-tightening-spec.md docs/current/GOLDEN_WORKFLOW.md docs/current/AUTHORITY_MODEL.md docs/00-glossary.md; do
  for term in "ProjectIdentity" "Continuity ID" "HLT" "MLG" "STG" "Waypoints" "Workpoint" "Evidence Ref" "Context Cognition" "Context Authority"; do
    rg -n "$term" "$ROOT_DIR/$file" >/dev/null || fail "$file missing canonical term $term"
  done
done
pass "primary docs preserve canonical trajectory/scope vocabulary"

rg -n 'HLT → MLG → STG → Waypoints → Workpoint' "$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md" "$ROOT_DIR/BENEFITS.md" >/dev/null \
  || fail "canonical trajectory ladder sequence missing from primary docs"
pass "canonical trajectory ladder sequence preserved"

for forbidden in \
  'simplify the trajectory ladder into' \
  'replace HLT with' \
  'replace MLG with' \
  'replace STG with' \
  'generic mission/milestone/next step'; do
  if rg -n "$forbidden" "$ROOT_DIR/README.md" "$ROOT_DIR/BENEFITS.md" "$ROOT_DIR/docs/current" "$ROOT_DIR/docs/focusa-tools" >/dev/null; then
    fail "forbidden vocabulary dilution found: $forbidden"
  fi
done
pass "no canonical vocabulary dilution found in primary/current/tool docs"

echo "vision vocabulary static test: PASS"
