#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MASTER="$ROOT_DIR/docs/135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md"
DELIVERY="$ROOT_DIR/docs/135-series-current-manifest.md"
AUDIT="$ROOT_DIR/docs/current/SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md"
ACCEL="$ROOT_DIR/docs/agent/spec135-implementation-acceleration-directive.md"
GENERATED_UI="$ROOT_DIR/docs/agent/spec135-real-time-generated-ui-directive.md"
UXP="$ROOT_DIR/docs/agent/spec135-uxp-ufi-generated-ui-directive.md"
SPEC_D="$ROOT_DIR/docs/135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md"
SPEC_E="$ROOT_DIR/docs/135e-cross-spec-amendments-migration-and-closure-matrix.md"
SPEC_H="$ROOT_DIR/docs/135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md"
SPEC_I="$ROOT_DIR/docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md"
SPEC_J="$ROOT_DIR/docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md"
SPEC_K="$ROOT_DIR/docs/135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
require_in(){
  local file="$1"
  local needle="$2"
  rg -n -F "$needle" "$file" >/dev/null || fail "$file missing required lock: $needle"
}

ACTIVE_DOCS=(
  "$MASTER" "$DELIVERY" "$AUDIT" "$ACCEL" "$GENERATED_UI" "$UXP"
  "$SPEC_D" "$SPEC_E" "$SPEC_H" "$SPEC_I" "$SPEC_J" "$SPEC_K"
)

for file in "${ACTIVE_DOCS[@]}"; do
  [[ -f "$file" ]] || fail "missing active Spec 135 source: $file"
done

require_in "$DELIVERY" 'Current Authoritative Delivery Contract'
require_in "$DELIVERY" 'The series is frozen at **135K**'
for companion in 135A 135B 135C 135D 135E 135F 135G 135H 135I 135J 135K; do
  require_in "$DELIVERY" "[$companion]"
done

while IFS= read -r filename; do
  if [[ "$filename" =~ ^135[l-z]([-_].*)?\.md$ ]]; then
    fail "Spec 135 series is frozen at 135K; forbidden later companion exists: docs/$filename"
  fi
done < <(find "$ROOT_DIR/docs" -maxdepth 1 -type f -printf '%f\n')
pass "Spec 135 companion series is frozen at 135K"

for needle in \
  'Spec 135 master' \
  '→ Spec 135 Delivery Contract/current manifest' \
  '→ current implementation-reality audit' \
  '→ implementation acceleration directive' \
  '→ generated UI directive' \
  '→ UXP/UFI directive' \
  '→ task-relevant companion specs'; do
  require_in "$GENERATED_UI" "$needle"
done
require_in "$GENERATED_UI" 'MUST NOT create 135L or any later lettered companion'
pass "delivery precedence and no-new-companion rule are explicit"

for needle in \
  'required_native_behavior:' \
  'required_fallback_behavior:' \
  'capability_detection:' \
  'degraded_state_presentation:' \
  'closure_proof:'; do
  require_in "$GENERATED_UI" "$needle"
done
require_in "$GENERATED_UI" 'Normative requirements use only `MUST`, `MUST NOT`, `REQUIRED`, and `FORBIDDEN`.'
require_in "$GENERATED_UI" 'Silent degradation and compatibility guessing are FORBIDDEN.'
pass "normative and capability-dependent closure contract is guarded"

for needle in \
  'openapi-3.0.3.json' \
  'json-schema/' \
  'operation-registry.json' \
  'a2ui-catalog.json' \
  'compatibility-lock.yaml' \
  'sha256sums.txt'; do
  require_in "$GENERATED_UI" "$needle"
done
require_in "$SPEC_E" 'focusa.compatibility_lock.v1'
require_in "$SPEC_E" 'startup version/capability handshake'
require_in "$GENERATED_UI" 'immutable Focusa commit SHA or release digest'
pass "immutable generated-contract bundle and compatibility handshake are guarded"

for needle in \
  'docs/contracts/spec135-complete-feature-ledger.v1.yaml' \
  'docs/contracts/spec135-delivery-dag.v1.yaml' \
  'docs/contracts/spec135-client-parity-matrix.v1.yaml' \
  'docs/contracts/spec135-framework-lock.v1.yaml' \
  'docs/contracts/spec135-proof-matrix.v1.yaml'; do
  rg -n -F "$needle" "$DELIVERY" "$SPEC_D" "$SPEC_E" "$SPEC_H" >/dev/null \
    || fail "machine-readable delivery graph requirement missing: $needle"
done
require_in "$SPEC_D" 'Agents MUST NOT infer the delivery DAG from prose alone.'
pass "machine-readable feature ledger, DAG, parity, framework, and proof requirements are guarded"

for needle in \
  'primitive_submission:' \
  'canonical_owner:' \
  'reusable_primitive:' \
  'crist_specific_projection:' \
  'generated_contract_change:' \
  'uiai_change:'; do
  rg -n -F "$needle" "$SPEC_D" "$SPEC_H" "$ACCEL" >/dev/null \
    || fail "greater primitive-submission contract missing: $needle"
done
require_in "$DELIVERY" 'general reusable Focusa primitive'
require_in "$DELIVERY" '→ reducer and canonical state'
require_in "$DELIVERY" '→ typed Focusa API'
require_in "$DELIVERY" '→ generated cross-language contracts'
pass "greater Focusa primitive submission and implementation order are guarded"

for needle in \
  'backend exists but generated UI is missing' \
  'generated UI uses mock state' \
  'browser proof bypasses UIAI Engine' \
  'scope is ambient or inferred' \
  'action bypasses Operation Registry' \
  'result bypasses shared ToolResult envelope' \
  'no recovery scenario' \
  'no restart/resume scenario' \
  'no nontechnical completion proof' \
  'reusable logic trapped in client or C.R.I.S.T.-local code' \
  'requirement remains open in feature ledger'; do
  require_in "$GENERATED_UI" "$needle"
done
require_in "$GENERATED_UI" 'Decomposing agents MUST NOT present option menus.'
pass "decomposition blockers and no-option-menu rule are guarded"

for needle in \
  'Onboarding' \
  '→ Context' \
  '→ Role' \
  '→ Grill Interview' \
  '→ Project Genesis Spec' \
  '→ Tasks' \
  '→ Workpoint' \
  '→ Evidence' \
  '→ Receipt' \
  '→ UIAI artifact' \
  '→ multiplexed Mission Canvas' \
  '→ pause' \
  '→ restart'; do
  require_in "$DELIVERY" "$needle"
done
require_in "$DELIVERY" '→ resume exact state'
require_in "$DELIVERY" 'Browser portions are proven exclusively through UIAI Engine Eval.'
pass "permanent generated-UI dogfood traversal is guarded"

for needle in \
  'UIAI Engine Eval only' \
  '@a2ui/web_core/v0_9' \
  '@a2ui/lit/v0_9 permanent renderer' \
  'Focusa Svelte Custom Elements' \
  'AG-UI compatibility proceeds in parallel and does not block the native traversal' \
  'OpenAPI 3.0.3' \
  'JSON Schema 2020-12' \
  'Do not add Vercel WorkflowAgent' \
  'Do not build a complete custom Svelte A2UI renderer' \
  'Do not add Playwright'; do
  require_in "$GENERATED_UI" "$needle"
done
require_in "$GENERATED_UI" 'required manual assistive-technology proof'
require_in "$SPEC_I" 'uiai.focusa_ui_eval_scenario.v1'
require_in "$SPEC_I" 'uiai.focusa_ui_eval_result.v1'
pass "browser proof, renderer, stream, runtime, contract, and accessibility ownership are guarded"

PLAYWRIGHT_CONFIGS="$(find "$ROOT_DIR" -type f \( -name 'playwright.config.*' -o -name '.playwright.*' \) -print)"
[[ -z "$PLAYWRIGHT_CONFIGS" ]] || { printf '%s\n' "$PLAYWRIGHT_CONFIGS" >&2; fail "Playwright configuration exists in Focusa"; }

MANIFEST_FILES=()
while IFS= read -r -d '' file; do MANIFEST_FILES+=("$file"); done < <(
  find "$ROOT_DIR" -type f \( \
    -name 'package.json' -o -name 'pnpm-lock.yaml' -o -name 'package-lock.json' -o \
    -name 'yarn.lock' -o -name 'bun.lock' -o -name 'bun.lockb' -o \
    -name 'pyproject.toml' -o -name 'requirements*.txt' \
  \) -print0
)
if ((${#MANIFEST_FILES[@]})) && rg -n "(@playwright/test|playwright-core|(^|[\"'[:space:]])playwright([\"':@[:space:]]|$))" "${MANIFEST_FILES[@]}"; then
  fail "Playwright dependency exists in Focusa package or lock files"
fi

if rg -n --glob '!*.md' --glob '!spec135i_real_time_generated_ui_static_test.sh' \
  --glob '!spec135_delivery_contract_regression_static_test.sh' \
  "(from[[:space:]]+[\"'][^\"']*playwright|require\\([\"']playwright|@playwright/test)" \
  "$ROOT_DIR/apps" "$ROOT_DIR/packages" "$ROOT_DIR/crates" "$ROOT_DIR/scripts" "$ROOT_DIR/tests" 2>/dev/null; then
  fail "Playwright import or executable browser-test usage exists in Focusa source"
fi
pass "Focusa contains no Playwright dependency, config, or executable browser-test usage"

check_positive_conflict(){
  local pattern="$1"
  local label="$2"
  local matches
  matches="$(rg -ni "$pattern" "${ACTIVE_DOCS[@]}" 2>/dev/null \
    | rg -vi 'MUST NOT|must not|Do not|do not|FORBIDDEN|forbidden|not the|does not|no Playwright|without|bypasses|remove|reject|superseded|inventory' || true)"
  [[ -z "$matches" ]] || { printf '%s\n' "$matches" >&2; fail "$label"; }
}

check_positive_conflict '(use|adopt|add|install|require).{0,40}(Playwright Test|Playwright MCP|@playwright/test)' \
  'active Spec 135 instruction reintroduced Playwright'
check_positive_conflict '(build|implement|replace|make).{0,50}(complete|full|custom).{0,30}Svelte.{0,20}A2UI renderer' \
  'active Spec 135 instruction reintroduced a complete custom Svelte A2UI renderer'
check_positive_conflict 'OpenAPI 3\.1.{0,60}(required|canonical|transport contract)' \
  'active Spec 135 instruction reintroduced OpenAPI 3.1 transport ownership'
check_positive_conflict '(use|adopt|add|install|depend on|require).{0,40}(Vercel AI SDK|WorkflowAgent|ToolLoopAgent|@ai-sdk/svelte|Vercel AI Gateway)' \
  'active Spec 135 instruction reintroduced Vercel runtime ownership'
check_positive_conflict '(maintain|write|hand-author|create).{0,40}(handwritten|manual|duplicate).{0,30}(DTO|action registr)' \
  'active Spec 135 instruction reintroduced handwritten generated-contract ownership'
pass "active Spec 135 instructions contain no conflicting framework/runtime adoption"

echo "Spec 135 Delivery Contract regression static test: PASS"
