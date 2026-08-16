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

rg -ni 'daemon-native Spec133|legacy action compatibility' "$TOOLS_TS" "$TOOL_DOC" "$CONTRACTS_TS" >/dev/null \
  || fail "daemon-native migration posture or bounded legacy compatibility missing"
pass "tool is daemon-native and legacy compatibility is explicitly bounded"

if rg -n 'p\.command \|\|\s*priorMeta\.command|priorMeta\.command \|\||tmux new-session|spawn\(' "$TOOLS_TS" >/dev/null; then
  fail "daemon facade can still auto-reuse or execute a stored legacy shell command"
fi
rg -n '/v1/silent-sessions|daemon facade|Daemon-native' "$TOOLS_TS" "$TOOL_DOC" >/dev/null \
  || fail "canonical daemon facade route is not documented/implemented"
pass "legacy shell commands are not executed by the daemon facade"

rg -n 'focusa-a6yq6\.2\.1|focusa-a6yq6\.10\.9|Gap closure mapping|0\.1:|0\.2:|0\.3:|0\.4:' "$TRACE" >/dev/null \
  || fail "traceability matrix incomplete"
pass "traceability maps Phase 0 through final gate and gap closures"

rg -n 'No Spec 133 tag, release, deploy, live sync, push, merge|focusa-slxpz\.6\.6.*closed|work_loop_conformance\.py --mode release|expected exit 3' "$RELEASE" >/dev/null \
  || fail "release freeze, resolved Spec132 gate, or fail-closed conformance gate not recorded"
pass "release freeze, resolved Spec132 gate, and fail-closed conformance gate recorded"

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
