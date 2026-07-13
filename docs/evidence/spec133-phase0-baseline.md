# Spec 133 Phase 0 baseline, scope, authority, and drift evidence

Date: 2026-07-13
Head: d534f852 baseline, updated by local Phase 0 commits after that head
Branch posture: detached HEAD
Remote posture: not fetched by instruction for singleton/Phase0 work; no remote, tag, release, deploy, cargo build/check/test, or push commands were run.

## 0.1 baseline scope

Spec 133 governing spec: `docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md`.

Normative source inventory from §1 was checked for repository presence:

- `docs/G1-detail-03-runtime-daemon.md`
- `docs/core-reducer.md`
- `docs/44-pi-focusa-integration-spec.md`
- `docs/66-affordance-and-execution-environment-ontology.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`
- `docs/72-agent-identity-role-and-self-model-ontology.md`
- `docs/76-retention-forgetting-and-decay-policy.md`
- `docs/77-ontology-governance-versioning-and-migration.md`
- `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`
- `docs/79-focusa-governed-continuous-work-loop.md`
- `docs/83-pi-focusa-rpc-efficiency-spec.md`
- `docs/88-ontology-backed-workpoint-continuity.md`
- `docs/96-trajectory-projection-and-daemon-stability-spec.md`
- `docs/98-project-root-crdt-reconciliation-foundation-spec.md`
- `docs/99-original-intent-vs-implementation-audit.md`
- `docs/100-context-cognition-spec.md`
- `docs/101-focusa-bloatgaurd-spec.md`
- `docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md`
- `docs/106-focusa-vision-tightening-spec.md`
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`
- `docs/111-agent-context-bootstrap-and-delivery-spec.md`
- `docs/116-provider-neutral-work-item-closure-authority-spec.md`
- `docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md`
- `docs/120-adversarial-spec-workbench-and-operator-approval-gates.md`
- `docs/current/AUTHORITY_MODEL.md`
- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`
- `docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md`

## Current implementation drift

Current legacy implementation remains Pi-local and tmux-backed in `apps/pi-extension/src/tools.ts` under `focusa_silent_sessions`.

Drift against Spec 133:

- Canonical daemon-native `/v1/silent-sessions` control plane is not implemented yet.
- Pi tool still directly invokes `tmux`; now explicitly marked legacy/non-canonical.
- Output/logging is compatibility-grade, not Spec 133 durable sequenced stream storage.
- Completion still cannot be canonical; process/tmux status is observation only.
- Model/provider binding, runner protocol, receipts, worktree isolation, and scheduler are future phase work.

## Protected contracts

- `focusa_silent_sessions` tool name is preserved as required by Spec 133 §25.2.
- Existing read/control actions remain available for compatibility.
- Mutating actions still require explicit approval flags.
- Stored legacy registry commands are not automatically reused on restart after this Phase 0 slice.

## Local proof commands

Passing:

```text
tests/spec133_phase0_static_test.sh
bash -n tests/spec133_phase0_static_test.sh
python3 -m json.tool docs/current/focusa-tool-contracts.json
```

Blocked by missing local Node dependencies (not installed in this worktree):

```text
npm --prefix apps/pi-extension run lint
# Error [ERR_MODULE_NOT_FOUND]: Cannot find package '@typescript-eslint/parser'

npm --prefix apps/pi-extension run check
# sh: tsc: command not found
```
