#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUDIT="$ROOT_DIR/docs/current/SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md"
MANIFEST="$ROOT_DIR/docs/135-series-current-manifest.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$AUDIT" ]] || fail "Spec 135 generated UI/core integration audit is missing"

for needle in \
  'A2UI and AG-UI are not implemented' \
  'OpenAPI generation is not installed' \
  'Existing API architecture is the correct foundation' \
  'Live SSE is low-latency but not durable' \
  'Canonical event replay already exists' \
  'Shared error/recovery middleware exists' \
  'Route-local envelope duplication exists' \
  'Capability and permission systems exist' \
  'UXP/UFI is canonical but not implemented' \
  'Existing `visual_workflow` routes are evidence routes' \
  'Correct core integration path' \
  'Revised fastest Alpha 0 implementation order' \
  'Nontechnical completion standard'; do
  rg -n -F "$needle" "$AUDIT" >/dev/null || fail "audit missing current-reality finding: $needle"
done
pass "generated UI audit preserves code reality and migration decisions"

for needle in \
  'A2UI instead of a custom generated-UI DSL' \
  'AG-UI middleware instead of a second agent stream' \
  'Generated Operation Registry instead of a manual UI action list' \
  'JSON Schema inputs instead of custom forms' \
  'Deterministic surfaces without model calls' \
  'Existing UXP/UFI instead of a new nontechnical mode' \
  'Existing UIAI Test Lab and evidence plane'; do
  rg -n -F "$needle" "$AUDIT" >/dev/null || fail "audit missing speed/reuse decision: $needle"
done
pass "audit locks high-leverage reuse opportunities"

rg -n -F 'SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md' "$MANIFEST" >/dev/null || fail "audit missing from Spec 135 manifest"
pass "audit is discoverable from the authoritative manifest"

echo "Spec 135 generated UI/core integration audit static test: PASS"
