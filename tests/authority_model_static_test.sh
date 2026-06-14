#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/AUTHORITY_MODEL.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$DOC" ]] || fail "missing docs/current/AUTHORITY_MODEL.md"

for term in \
  "Operator Ask" \
  "ProjectIdentity" \
  "Continuity ID" \
  "Session ID" \
  "HLT" \
  "MLG" \
  "STG" \
  "Waypoints" \
  "Workpoint" \
  "Evidence Ref" \
  "Focus State" \
  "Focus Stack" \
  "Context Cognition" \
  "Context Authority" \
  "Project Card" \
  "Call Stack Design" \
  "Metacognition" \
  "Prediction" \
  "Work-loop" \
  "Operator Steering"; do
  rg -n "\| ${term//-/\\-} \|" "$DOC" >/dev/null || fail "authority table missing $term"
done
pass "authority table covers Spec106 authority surfaces"

for posture in canonical advisory degraded blocked stale; do
  rg -n "\b$posture\b" "$DOC" >/dev/null || fail "authority model missing posture $posture"
done
pass "authority model declares required posture labels"

for file in \
  README.md \
  docs/00-glossary.md \
  docs/current/CONTEXT_AUTHORITY_CURRENT.md \
  docs/focusa-tools/README.md \
  docs/focusa-tools/tools/focusa_context_cognition.md \
  docs/focusa-tools/tools/focusa_workpoint_resume.md \
  docs/focusa-tools/tools/focusa_trajectory_view.md; do
  rg -n "AUTHORITY_MODEL\.md" "$ROOT_DIR/$file" >/dev/null || fail "$file does not reference AUTHORITY_MODEL.md"
done
pass "core docs and authority-bearing tool docs reference Authority Model"

echo "authority model static test: PASS"
