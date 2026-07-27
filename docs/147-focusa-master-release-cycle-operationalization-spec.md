# Spec 147 — Focusa Master Release Cycle operationalization

**Status:** active implementation  
**Authority:** operator directive; Beads `focusa-91gtu.1` through `.7`  
**Depends on:** Specs 145–146 and merged PR #85  
**Scope:** Focusa release providers, bounded self-healing, KH/OVH authority, agent-kb-api, and no-spam proof

## 1. Objective

Operationalize the provider-neutral Master Release Cycle without creating another release. Existing release, deployment, OTA, evidence, retry, and healing components must either enter the typed cycle through a bounded adapter or receive an explicit compatibility, quarantine, or retirement boundary.

The system becomes more autonomous only when autonomy is safer than repetition: one durable failure fingerprint, one owner, bounded attempts and cooldown, explicit mutation authority, independent validation, rollback, and append-only settlement.

## 2. Non-goals and hard boundaries

- No Focusa or UIAI production release is created by this work.
- No tag, GitHub release, deployment, rollback, or self-heal PR is created during proof.
- The Master kernel never imports GitHub, KH, OVH, or Focusa-specific behavior.
- A retry is not a repair. Deterministic failures require a patch or operator review.
- Tier 1–2 services, firewall, DNS, secrets, and production credentials are unchanged.
- For Focusa, OVH performs routed build/test/package/provenance in the synchronized `wirebot` workspace. KH retains Pi, production deployment, runtime, and final verification authority.

## 3. Canonical execution chain

1. Operator intent creates one exact-SHA candidate and authority envelope.
2. The kernel validates topology, authority, evidence reuse, and mutation permission.
3. A provider adapter maps typed operations to existing provider workflows or tools.
4. Every operation returns a typed receipt with idempotency key and evidence refs.
5. Failures enter the healing governor before any retry or proposal.
6. The governor fingerprints, deduplicates, budgets, cools down, and claims one action.
7. Independent verification settles success, rollback, exhaustion, or operator review.
8. Calibration may alter only a bounded later plan after measured evaluation.

## 4. Legacy component inventory and disposition

| Component | Existing role | Disposition | Master Cycle boundary |
|---|---|---|---|
| `crates/focusa-core/src/release_cycle.rs` | candidate and topology model | KEEP | compatibility model consumed by protocol/planner |
| `release_protocol.rs`, `release_planner.rs`, `release_orchestrator.rs` | provider-neutral authority/state kernel | KEEP | sole release state authority |
| `release_ledger.rs` | exact-candidate checkpoints | KEEP | append-only resume and settlement |
| `release_calibration.rs` | benchmark/tuning experiments | KEEP | project/profile-scoped learning only |
| `release_adapters.rs` and `config/release-adapters/*.json` | manifest and external process protocol | KEEP | all provider execution enters here |
| `.github/workflows/ci.yml` | source CI evidence | WRAP | provider evidence source; no independent retry authority |
| `.github/workflows/release.yml` | artifact/package/provenance/release provider | WRAP | GitHub adapter observes or dispatches only with execute authority |
| `.github/workflows/deploy-live-daemon.yml` | canary/deploy/rollback provider | WRAP | adapter operation with immutable artifact and rollback receipt |
| `.github/workflows/dev-release-tag.yml` | explicit tag mutation | WRAP | mutation-capable operation; never called in proof mode |
| `.github/workflows/release-pipeline-watchdog.yml` | recurring bounded rerun/redispatch | WRAP | healing governor must authorize each action and emit settlement |
| `.github/workflows/auto-retry-deploy.yml` | second automatic repair actor | QUARANTINE | remove automatic `workflow_run`; manual compatibility drill only |
| `.github/workflows/audit-recorder.yml` | failure intake and repair proposal | WRAP | observer plus atomic durable fingerprint claim; never passive audit spam |
| `.github/workflows/self-heal-failure-injection.yml` | classifier/governor drill | KEEP | mutation-free failure-injection proof |
| `.github/workflows/deploy-self-heal-proof-drill.yml` | deploy healing drill | KEEP | mutation-free rollback/settlement proof |
| `.github/workflows/tauri-updater-signing-proof.yml` | updater signing evidence | KEEP | exact-SHA provenance evidence source |
| `.github/workflows/windows-ota-e2e.yml` | Windows OTA proof | KEEP | platform verification evidence source |
| `.github/workflows/spec132-terminal-matrix.yml` | terminal parity proof | KEEP | preflight evidence source |
| `scripts/release.sh` | local workspace build shim | KEEP | compatibility build helper, not orchestration authority |
| `release-gate.py`, `release-trust-metadata.py`, `generate-supply-chain-artifacts.sh` | gates/provenance | WRAP | adapter-invoked evidence producers |
| `release-deploy-proof.py`, `deploy-smoke-check.sh` | deployment verification | WRAP | independent verify/rollback evidence producers |
| `record-workflow-failure.py`, `classify-ci-failure.py` | failure evidence/classification | KEEP | healing-governor inputs |
| `propose-system-fix.py`, `auto-heal-audit.py` | thresholded proposal path | WRAP | governor fingerprint/claim policy; compatibility filename retained |
| `self-heal-telemetry.py` | healing outcome metrics | KEEP | settlement and calibration input |
| self-heal decision/deploy drill scripts | policy fixtures | KEEP | mutation-free conformance evidence |
| update runtime/CLI/scheduler | OTA install/promotion | WRAP | deployment adapter only; atomic backup/health/rollback required |
| stale per-run `self-heal/<run_id>` branch model | GitHub proposal ownership | RETIRE | replaced by `self-heal/fp-<fingerprint>` atomic claim |
| passive audit-only commits/PRs | noisy pseudo-healing | RETIRE | no concrete deliverable means no GitHub mutation |

## 5. Typed GitHub provider adapter

Create `scripts/master-release-github-adapter.py` as the reference external JSON adapter.

Requirements:

- Read exactly one `focusa.release_execution_envelope.v1` from stdin.
- Validate operation ID, executor ID, exact SHA, idempotency key, and timeout.
- Support `plan`, `observe`, and `execute` provider modes; default to `plan`.
- `plan` and proof modes make no network or filesystem mutation.
- `observe` may query GitHub but cannot dispatch, tag, publish, deploy, or rerun.
- `execute` requires both kernel execute mode and explicit adapter mutation approval.
- Map legacy CI/release/deploy actions to typed provider commands.
- Reuse exact-SHA evidence instead of dispatching duplicate work.
- Emit one bounded `ReleaseStageReceipt`; never emit raw logs or secrets.
- Preserve one artifact identity through package, provenance, draft, canary, verify, and promotion.
- Emit rollback refs for promotion/deployment operations.

Update `config/release-adapters/focusa.json` to identify this executable reference and legacy workflow mappings without putting shell commands inside the kernel.

## 6. Governed self-healing

Create `scripts/self_heal_governor.py` with schemas:

- `focusa.self_heal.failure.v1`
- `focusa.self_heal.decision.v1`
- `focusa.self_heal.claim.v1`
- `focusa.self_heal.settlement.v1`

Fingerprint inputs:

`repository + workflow + failure_class + exact_sha + normalized_action_scope`

Policy:

- File-lock local ledger updates.
- One open claim per fingerprint.
- One automatic attempt per fingerprint by default.
- One repository mutation per governor window by default.
- Minimum cooldown after an attempted action.
- Deterministic/code/security/authority failures never rerun automatically.
- Transient provider failures may rerun only failed jobs.
- Artifact/deploy recovery may redispatch only when immutable artifact identity is known.
- A proposal requires a concrete deliverable and exact-SHA validation command.
- A failed verification invokes rollback when supported, otherwise operator review.
- Every claim settles as `healed`, `rolled_back`, `exhausted`, or `operator_review`.

Update `propose-system-fix.py` to emit the durable fingerprint and governor policy fields. Update `audit-recorder.yml` to use `self-heal/fp-<fingerprint>`, atomically create the remote branch once, reuse an existing branch/PR, cap open proposals, and never force-push. Update `release-pipeline-watchdog.yml` to obtain a governor decision before rerun/redispatch. Convert `auto-retry-deploy.yml` to manual-only compatibility mode.

## 7. KH/OVH dual-server authority

Canonical guidance must state:

| Phase | KH | OVH |
|---|---|---|
| source/spec/task authority | Pi session, canonical operator intent, and deployment authority | consume bounded build intent in synchronized workspace |
| build/test/package/provenance | route and verify expected outputs | execute resource-heavy Focusa build and staging |
| immutable transfer | verify digest/provenance before deployment staging | originate digest + provenance + artifact set |
| canary/deploy | execute production canary/deploy from transferred artifact | prohibited from Focusa production deployment |
| health/version verification | emit live health/version evidence | return build/staging evidence only |
| rollback | atomically restore approved artifact and verify | retain/reproduce immutable build artifact when requested |
| settlement/calibration | append canonical outcome and metrics | return bounded build metrics |


No documentation may infer hostnames, ports, users, paths, credentials, or domains beyond verified agent-kb authority.

## 8. Agent-kb-api and agent bootstrap

Create numbered agent guidance with frontmatter ID `master-release-cycle`. Agent-kb-api must:

- serve the guide through `/v1/doc/master-release-cycle`;
- include a bounded Master Release Cycle route in `/v1/bootstrap`;
- identify the seven-stage dependency chain and current authority boundaries;
- direct agents to Focusa tools for Focusa state interactions;
- prohibit independent workflow retries or self-heal GitHub mutation;
- expose KH/OVH build/deploy responsibilities without secret values.

Update API tests, build the binary, reindex, and verify health/bootstrap/doc responses. Service restart requires backup and post-restart proof.

## 9. Exact implementation files

Focusa repository:

- `docs/147-focusa-master-release-cycle-operationalization-spec.md`
- `docs/145-focusa-canonical-core-release-cycle-fast-release-architecture.md`
- `docs/146-focusa-canonical-release-cycle-operations-and-proof-runbook.md`
- `scripts/master-release-github-adapter.py`
- `scripts/self_heal_governor.py`
- `scripts/propose-system-fix.py`
- `.github/workflows/audit-recorder.yml`
- `.github/workflows/release-pipeline-watchdog.yml`
- `.github/workflows/auto-retry-deploy.yml`
- `config/release-adapters/focusa.json`
- `tests/spec147_master_release_operationalization_test.py`
- `docs/evidence/spec147/spec147-master-release-operationalization.txt`

Agent knowledge/API scope:

- `/root/.agent-kb/docs/02-focusa-master-release-cycle-dual-server-agent-guide.md`
- `/root/.agent-kb/AGENT_KB_API_USAGE.md`
- `/root/agent-kb/cmd/agent-kb-api/main.go`
- `/root/agent-kb/cmd/agent-kb-api/handlers_test.go`

## 10. Acceptance and proof

1. Static matrix test proves every inventoried component has one disposition.
2. Adapter tests prove plan mode is network/mutation free and receipts are deterministic.
3. Governor tests prove two concurrent requests for one fingerprint yield one claim.
4. Duplicate proposal tests prove branch and PR identity derive from fingerprint, not run ID.
5. Retry tests prove deterministic failures and exhausted budgets cannot rerun.
6. Workflow validation proves only the governed watchdog retains automatic retry authority.
7. KH/OVH docs prove OVH build/staging and KH Pi/deploy/runtime authority and contain no secret values.
8. Agent-kb-api tests and live probes return the versioned guide and bootstrap route.
9. Integrated failure injection settles without tag, release, deploy, branch, PR, or remote mutation.
10. Focusa quality gates, docs/runtime parity, and `git diff --check` pass.

Durable receipt: `docs/evidence/spec147/spec147-master-release-operationalization.txt`.

## 11. Rollback

- Focusa changes are one isolated feature branch and one PR.
- `auto-retry-deploy.yml` can be restored to its prior trigger only after governor failure evidence and operator approval.
- The GitHub adapter defaults to plan mode, so disabling it requires no provider mutation.
- Agent-kb-api binary is backed up before replacement; failed health/doc probes restore the backup and restart.
- Agent-kb and dual-server guidance remains local-only; no sensitive push occurs.
