#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

fail() {
  echo "✗ FAIL: $*" >&2
  exit 1
}

pass() {
  echo "✓ PASS: $*"
}

EVIDENCE_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_evidence_capture.md"
TRAVERSE_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_traverse.md"
METACOG_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_metacog_capture.md"

rg -n 'Trajectory-aware evidence|trajectory_id.*hlt.*mlg.*stg|proof alignment metadata' "$EVIDENCE_DOC" >/dev/null \
  || fail "evidence capture docs do not describe trajectory-aware proof alignment context"
pass "evidence capture docs describe trajectory-aware proof context"

rg -n 'evidence.*ecs.*references.*trajectory|HLT/STG-aligned|without requesting full payloads' "$TRAVERSE_DOC" >/dev/null \
  || fail "traverse docs do not describe trajectory context in evidence/ECS/reference projections"
pass "traverse docs describe bounded trajectory context on evidence/ECS/reference slices"

rg -n 'hot-index tags.*trajectory|HLT/MLG/STG alignment|project_root \+ continuity_id' "$METACOG_DOC" >/dev/null \
  || fail "metacog capture docs do not describe trajectory-context retrieval semantics"
pass "metacog capture docs describe trajectory-context retrieval semantics"
