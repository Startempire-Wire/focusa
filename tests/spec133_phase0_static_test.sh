#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_TS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
CONTRACTS_TS="$ROOT_DIR/apps/pi-extension/src/tool-contracts.ts"
TOOL_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_silent_sessions.md"
BASELINE="$ROOT_DIR/docs/evidence/spec133-phase0-baseline.md"
TRACE="$ROOT_DIR/docs/evidence/spec133-phase0-traceability.md"
RELEASE="$ROOT_DIR/docs/evidence/spec133-phase0-release-gate.md"
SPEC="$ROOT_DIR/docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$SPEC" ]] || fail "Spec 133 doc missing"
[[ -f "$BASELINE" && -f "$TRACE" && -f "$RELEASE" ]] || fail "Phase 0 evidence files missing"
pass "Phase 0 evidence files exist"

rg -n 'legacy/non-durable|not the canonical Spec133|SILENT_SESSION_LEGACY_POSTURE|legacy_silent_session_wrapper_used' "$TOOLS_TS" "$TOOL_DOC" "$CONTRACTS_TS" >/dev/null \
  || fail "legacy/non-durable labels or telemetry missing"
pass "legacy wrapper is labeled and telemetry-instrumented"

rg -n 'stored_legacy_command_rejected|stored legacy shell `command` values are not auto-executed' "$TOOLS_TS" "$TOOL_DOC" >/dev/null \
  || fail "stored legacy command rejection not documented/implemented"
if rg -n 'p\.command \|\|\s*priorMeta\.command|priorMeta\.command \|\|' "$TOOLS_TS" >/dev/null; then
  fail "restart can still auto-reuse priorMeta.command"
fi
pass "stored legacy shell commands are not auto-reused on restart"

rg -n 'focusa-a6yq6\.2\.1|focusa-a6yq6\.10\.9|Gap closure mapping|0\.1:|0\.2:|0\.3:|0\.4:' "$TRACE" >/dev/null \
  || fail "traceability matrix incomplete"
pass "traceability maps Phase 0 through final gate and gap closures"

rg -n 'No release, deploy, tag, push, remote fetch/pull, cargo build/check/test|focusa-slxpz\.5\.6.*open|Phase 0 gate must remain open' "$RELEASE" >/dev/null \
  || fail "release/deploy freeze or Spec132 blocker not recorded"
pass "release/deploy freeze and Spec132 blocker recorded"

required_docs=(
  docs/G1-detail-03-runtime-daemon.md
  docs/core-reducer.md
  docs/44-pi-focusa-integration-spec.md
  docs/66-affordance-and-execution-environment-ontology.md
  docs/70-shared-interfaces-statuses-and-lifecycle.md
  docs/72-agent-identity-role-and-self-model-ontology.md
  docs/76-retention-forgetting-and-decay-policy.md
  docs/77-ontology-governance-versioning-and-migration.md
  docs/78-bounded-secondary-cognition-and-persistent-autonomy.md
  docs/79-focusa-governed-continuous-work-loop.md
  docs/83-pi-focusa-rpc-efficiency-spec.md
  docs/88-ontology-backed-workpoint-continuity.md
  docs/96-trajectory-projection-and-daemon-stability-spec.md
  docs/98-project-root-crdt-reconciliation-foundation-spec.md
  docs/99-original-intent-vs-implementation-audit.md
  docs/100-context-cognition-spec.md
  docs/101-focusa-bloatgaurd-spec.md
  docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md
  docs/106-focusa-vision-tightening-spec.md
  docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md
  docs/111-agent-context-bootstrap-and-delivery-spec.md
  docs/116-provider-neutral-work-item-closure-authority-spec.md
  docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md
  docs/120-adversarial-spec-workbench-and-operator-approval-gates.md
  docs/current/AUTHORITY_MODEL.md
  docs/current/CONTEXT_AUTHORITY_CURRENT.md
  docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md
)
for rel in "${required_docs[@]}"; do
  [[ -f "$ROOT_DIR/$rel" ]] || fail "missing governing source: $rel"
done
pass "Spec 133 governing source files are present"
