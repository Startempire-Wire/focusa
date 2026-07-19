#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-uxp-ufi-generated-ui-directive.md"
MANIFEST="$ROOT_DIR/docs/135-series-current-manifest.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$SPEC" "$DIRECTIVE" "$MANIFEST"; do
  [[ -f "$file" ]] || fail "missing required Spec 135K file: $file"
done

for needle in \
  'Spec 14' \
  'Default nontechnical baseline' \
  'verbosity_preference' \
  'explanation_depth' \
  'confirmation_preference' \
  'interruption_sensitivity' \
  'review_cadence' \
  'Completion time' \
  'Why is Focusa presenting this this way?' \
  'User override' \
  'Safety invariants' \
  'PlainLanguageProjection' \
  'Nontechnical completion benchmark' \
  'UIAI Engine Eval scenarios'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135K missing UXP/UFI decision: $needle"
done
pass "canonical UXP/UFI and nontechnical baseline are explicit"

for needle in \
  'Do not create a simple mode' \
  'canonical UXP/UFI system' \
  'safe baseline' \
  'Record only observable' \
  'Every adaptation must answer' \
  'UIAI Engine Eval' \
  'Do not add Playwright' \
  'uxp_ufi:' \
  'uiai_eval_scenarios:' \
  'A missing UXP/UFI or UIAI Eval section blocks'; do
  rg -n -F "$needle" "$DIRECTIVE" >/dev/null || fail "UXP/UFI agent directive missing: $needle"
done
pass "agents receive canonical adaptive-UI and browser-proof rules"

if rg -n 'Playwright tests|playwright_flow_ref' "$SPEC" "$DIRECTIVE"; then
  fail "stale browser-test ownership remains in Spec 135K"
fi

rg -n -F '135K' "$MANIFEST" >/dev/null || fail "Spec 135K missing from Delivery Contract"
rg -n -F 'UIAI Engine Eval' "$MANIFEST" >/dev/null || fail "UIAI Engine Eval decision missing from Delivery Contract"
pass "Spec 135K and proof ownership are discoverable"

echo "Spec 135K UXP/UFI generated UI static test: PASS"
