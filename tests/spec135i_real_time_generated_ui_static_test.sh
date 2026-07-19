#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-real-time-generated-ui-directive.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$SPEC" ]] || fail "Spec 135I is missing"
[[ -f "$DIRECTIVE" ]] || fail "Spec 135 generated UI agent directive is missing"

for needle in \
  'Every C.R.I.S.T. and onboarding interaction must be presented as a live, incrementally regenerated' \
  'A2UI protocol v0.9.1' \
  '@a2ui/web_core/v0_9' \
  '@a2ui/lit/v0_9' \
  'AG-UI protocol' \
  'openapi-fetch' \
  'focusa.generated_surface.v1' \
  'focusa.ui_action_binding.v1' \
  'No generic mutation escape hatch' \
  'Deterministic shell and generative content boundary' \
  'Trusted Focusa component catalog' \
  'Nontechnical experience constitution'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing decision: $needle"
done
pass "generated UI protocol, authority, and nontechnical decisions are explicit"

for needle in \
  'C — Context generated UI' \
  'R — Role generated UI' \
  'I — Interview generated UI' \
  'S — Spec generated UI' \
  'T — Tasks generated UI' \
  'Operational continuation UI'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing complete stage UI: $needle"
done
pass "all C.R.I.S.T. and continuation stages require generated UI"

for needle in \
  'GET  /v1/ui/catalogs' \
  'POST /v1/ui/surfaces' \
  'GET  /v1/ui/surfaces/:surface_id/stream' \
  'POST /v1/ui/actions/preview' \
  'POST /v1/ui/actions/commit' \
  'focusa-core/src/ui_intent/' \
  'focusa-api/src/ui_projection/' \
  'focusa-api/src/ag_ui/'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing Focusa API integration: $needle"
done
pass "generated UI is integrated through typed Focusa APIs"

for needle in \
  'A2UI Composer/Theater fixtures' \
  'Schemathesis' \
  'Vitest' \
  'Svelte Testing Library' \
  'Playwright' \
  'No custom generated-UI DSL' \
  'A2UI reuse plan' \
  'Catalog-first implementation'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing speed/reuse opportunity: $needle"
done
pass "generated UI speed and non-reinvention stack is explicit"

for needle in \
  'Do not implement:' \
  'static wizard pages as the primary experience' \
  'A2UI v0.9.1' \
  'Every generated action binds to a registered typed Focusa operation' \
  'Required stage surfaces' \
  'Every relevant implementation ticket includes:' \
  'A missing generated-UI section blocks the ticket'; do
  rg -n -F "$needle" "$DIRECTIVE" >/dev/null || fail "agent directive missing generated UI instruction: $needle"
done
pass "decomposing agents receive mandatory generated UI instructions"

for needle in \
  'Alpha 0' \
  'Alpha 1' \
  'Alpha 2' \
  'Alpha 3' \
  'Alpha 4' \
  'Alpha 5' \
  'Alpha 6' \
  'Alpha 7' \
  'Alpha 8'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing Cross-Functional Alpha amendment: $needle"
done
pass "every Alpha slice is amended to include generated UI"

echo "Spec 135I real-time generated UI static test: PASS"
