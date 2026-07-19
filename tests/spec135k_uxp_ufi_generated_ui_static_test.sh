#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-uxp-ufi-generated-ui-directive.md"
MANIFEST="$ROOT_DIR/docs/135-series-current-manifest.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$SPEC" ]] || fail "Spec 135K is missing"
[[ -f "$DIRECTIVE" ]] || fail "Spec 135 UXP/UFI directive is missing"

for needle in \
  'Spec 14' \
  'Default nontechnical baseline' \
  'verbosity_preference' \
  'explanation_depth' \
  'confirmation_preference' \
  'interruption_sensitivity' \
  'review_cadence' \
  'No friction from normal completion' \
  'Why this presentation?' \
  'User override' \
  'Safety invariants' \
  'PlainLanguageProjection integration' \
  'Nontechnical completion benchmark'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135K missing UXP/UFI decision: $needle"
done
pass "canonical UXP/UFI reuse and nontechnical baseline are explicit"

for needle in \
  'Do not create a simple mode' \
  'Use the existing canonical UXP/UFI system' \
  'safe nontechnical baseline' \
  'Record only observable' \
  'Every adaptation must answer' \
  'uxp_ufi:' \
  'A missing UXP/UFI section blocks'; do
  rg -n -F "$needle" "$DIRECTIVE" >/dev/null || fail "UXP/UFI agent directive missing: $needle"
done
pass "agents receive canonical adaptive-UI rules"

rg -n -F '135K' "$MANIFEST" >/dev/null || fail "Spec 135K missing from current manifest"
rg -n -F 'Spec 135 UXP/UFI generated UI directive' "$MANIFEST" >/dev/null || fail "UXP/UFI directive missing from manifest"
pass "Spec 135K and directive are discoverable"

echo "Spec 135K UXP/UFI generated UI static test: PASS"
