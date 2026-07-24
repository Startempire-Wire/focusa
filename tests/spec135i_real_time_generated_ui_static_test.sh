#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-real-time-generated-ui-directive.md"
DELIVERY="$ROOT_DIR/docs/135-series-current-manifest.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$SPEC" "$DIRECTIVE" "$DELIVERY"; do
  [[ -f "$file" ]] || fail "missing required Spec 135 generated UI file: $file"
done

for needle in \
  'live, incrementally regenerated, plain-language' \
  'A2UI protocol v0.9.1' \
  '@a2ui/web_core/v0_9' \
  '@a2ui/lit/v0_9' \
  'Focusa Svelte Custom Elements' \
  'OpenAPI 3.0.3' \
  'JSON Schema 2020-12' \
  'external adapters generated from published OpenAPI outside Focusa core' \
  'focusa.generated_surface.v1' \
  'focusa.ui_action_binding.v1' \
  'UIAI Engine Eval contract' \
  'Pi RPC AgentExecutionAdapter' \
  'Greater Focusa primitive'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing decision: $needle"
done
pass "generated UI, contracts, runtime, and browser-proof decisions are explicit"

for needle in \
  '### Context' \
  '### Role' \
  '### Interview' \
  '### Spec' \
  '### Tasks' \
  '### Operational continuation'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing stage surface: $needle"
done
pass "all C.R.I.S.T. and continuation stages require generated UI"

for needle in \
  'GET  /v1/ui/catalogs' \
  'GET  /v1/ui/operations' \
  'POST /v1/ui/surfaces' \
  'GET  /v1/ui/surfaces/:surface_id/stream' \
  'POST /v1/ui/actions/preview' \
  'POST /v1/ui/actions/commit' \
  'focusa-core/src/ui_intent/' \
  'focusa-api/src/ui_projection/' \
  'focusa-api/src/ag_ui/'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135I missing API integration: $needle"
done
pass "generated UI is integrated through typed Focusa APIs"

for needle in \
  'UIAI Engine Eval only' \
  'MUST NOT use Playwright Test' \
  'complete custom Svelte A2UI renderer' \
  'AG-UI implementation proceeds in parallel' \
  'Vercel WorkflowAgent' \
  'A missing generated-UI or UIAI Eval section blocks the ticket'; do
  rg -n -F "$needle" "$SPEC" "$DIRECTIVE" "$DELIVERY" >/dev/null || fail "missing hardening decision: $needle"
done
pass "browser, renderer, streaming, and model-runtime ownership is guarded"

if rg -n 'playwright_flow_ref|OpenAPI 3\.1|make Svelte renderer primary|Playwright tests full' "$SPEC" "$DIRECTIVE"; then
  fail "stale Spec 135I implementation decision remains"
fi

PLAYWRIGHT_CONFIGS="$(find "$ROOT_DIR" -path '*/node_modules' -prune -o -type f \( -name 'playwright.config.*' -o -name '.playwright.*' \) -print)"
if [[ -n "$PLAYWRIGHT_CONFIGS" ]]; then
  printf '%s\n' "$PLAYWRIGHT_CONFIGS" >&2
  fail "Playwright configuration exists in Focusa"
fi

MANIFEST_FILES=()
while IFS= read -r -d '' file; do MANIFEST_FILES+=("$file"); done < <(
  find "$ROOT_DIR" -path '*/node_modules' -prune -o -type f \( \
    -name 'package.json' -o \
    -name 'pnpm-lock.yaml' -o \
    -name 'package-lock.json' -o \
    -name 'yarn.lock' -o \
    -name 'bun.lock' -o \
    -name 'bun.lockb' -o \
    -name 'pyproject.toml' -o \
    -name 'requirements*.txt' \
  \) -print0
)
if ((${#MANIFEST_FILES[@]})) && rg -n "(@playwright/test|playwright-core|(^|[\"'[:space:]])playwright([\"':@[:space:]]|\$))" "${MANIFEST_FILES[@]}"; then
  fail "Playwright dependency exists in Focusa package or lock files"
fi

if rg -n --glob '!*.md' --glob '!**/node_modules/**' --glob '!spec135i_real_time_generated_ui_static_test.sh' \
  --glob '!spec135_delivery_contract_regression_static_test.sh' \
  "(from[[:space:]]+[\"'][^\"']*playwright|require\\([\"']playwright|@playwright/test)" \
  "$ROOT_DIR/apps" "$ROOT_DIR/packages" "$ROOT_DIR/crates" "$ROOT_DIR/scripts" "$ROOT_DIR/tests" 2>/dev/null; then
  fail "Playwright import or executable test usage exists in Focusa source"
fi
pass "Focusa contains no Playwright dependency, config, or executable browser-test usage"

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
  rg -n -F "$needle" "$SPEC" "$DELIVERY" >/dev/null || fail "missing Cross-Functional Alpha requirement: $needle"
done
pass "every Alpha slice includes generated UI and proof"

echo "Spec 135I real-time generated UI static test: PASS"
