#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUDIT="$ROOT_DIR/docs/current/SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md"
DELIVERY="$ROOT_DIR/docs/135-series-current-manifest.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$AUDIT" "$DELIVERY"; do
  [[ -f "$file" ]] || fail "missing Spec 135 audit/delivery file: $file"
done

for needle in \
  'A2UI and AG-UI are not implemented' \
  'Generated contracts are not installed' \
  'Existing API architecture is the correct foundation' \
  'Live SSE is low-latency but not durable' \
  'Canonical SQLite replay already exists' \
  'Shared ToolResult/error middleware exists' \
  'Route-local envelope duplication exists' \
  'Capability and permission systems exist' \
  'UXP/UFI is specified but not implemented' \
  'Existing `visual_workflow` routes are Evidence routes' \
  'Browser execution and proof already belong to UIAI Engine' \
  'Model execution already belongs to governed harness sessions' \
  'Correct core integration path' \
  'Exact fastest Foundation Train' \
  'Nontechnical completion standard'; do
  rg -n -F "$needle" "$AUDIT" >/dev/null || fail "audit missing current-reality finding: $needle"
done
pass "generated UI audit preserves current code reality and mandatory migrations"

for needle in \
  'A2UI instead of a custom generated-UI system' \
  'Generated Operation Registry' \
  'Schema-driven ordinary inputs' \
  'Deterministic UI without model calls' \
  'UIAI Engine Eval instead of browser test reinvention' \
  'Existing UXP/UFI' \
  'Deterministic fixtures before live integration' \
  'Greater primitive submission'; do
  rg -n -F "$needle" "$AUDIT" >/dev/null || fail "audit missing speed/reuse decision: $needle"
done
pass "audit locks high-leverage reuse and primitive ownership"

for needle in \
  'UIAI Engine Eval' \
  'Focusa MUST NOT add Playwright' \
  'OpenAPI 3.0.3' \
  'permanent Lit renderer' \
  'AG-UI compatibility proceeds after' \
  'Vercel WorkflowAgent'; do
  rg -n -F "$needle" "$AUDIT" "$DELIVERY" >/dev/null || fail "audit/delivery missing fixed decision: $needle"
done
pass "browser, renderer, stream, contract, and model decisions are guarded"

rg -n -F 'SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md' "$DELIVERY" >/dev/null || fail "audit missing from Delivery Contract"
pass "audit is discoverable from the authoritative Delivery Contract"

echo "Spec 135 generated UI/core integration audit static test: PASS"
