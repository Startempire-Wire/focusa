# Spec 150 — Focusa Guided Install, First Project, and Lifecycle Master Specification

- Status: Draft planning authority; implementation and publication are gated
- Date: 2026-07-29
- Scope: host installation through first canonical Workpoint, repair, update, rollback, and uninstall
- Directive: complete install-process documentation and planning without weakening existing authority, safety, or compatibility contracts

---

## 1. Purpose

Focusa has strong component-level installation and onboarding contracts, but no single end-to-end authority connecting them safely. This specification orchestrates:

1. verified Focusa host installation;
2. explicitly selected optional integrations;
3. explicitly selected and verified project;
4. governed Project Bootstrap and Project Genesis;
5. first canonical Workpoint and optional Mission Canvas mode;
6. durable acceptance evidence; and
7. safe repair, update, rollback, uninstall, and purge paths.

Spec 150 coordinates existing owners; it does not replace their lower-level contracts.

## 2. Non-negotiable laws

1. Host installation and project mutation are separate transactions.
2. Installation never infers a project from cwd and never creates a remote.
3. Existing user, project, provider, and harness state is preserved by default.
4. Every mutation has preview, confirmation, idempotency, evidence, and recovery semantics.
5. The installer orchestrates owned components instead of duplicating them.
6. Pi retains authority over Pi prompts, queues, providers, compaction, and continuation.
7. UIAI is optional and governed, never a core-install prerequisite.
8. Provider secrets remain provider- or harness-owned and never enter Focusa receipts.
9. Generated instructions obey Specs 140/140A; silent overwrite is forbidden.
10. Project Bootstrap and Project Genesis retain their own confirmation gates.
11. User data remains unless destructive purge is explicitly and separately confirmed.
12. Documentation advertises only behavior proven by code or conformance evidence.
13. Platform limitations are explicit; parity is never implied.
14. Binary copy alone does not prove successful Focusa installation.
15. GitHub #14 remains the first locked-release implementation task.

## 3. Scope and exclusions

### 3.1 Included

- host preflight, target selection, artifact trust, license/evaluation/development authority;
- CLI, daemon, TUI, skills, Pi, optional UIAI, and Mac menubar compatibility;
- interactive/headless, dependency, service, release-channel, and integration choices;
- provider capability discovery without provider-secret custody;
- existing/new/skip-project paths, Git and task-provider choices;
- governed project instructions, Bootstrap, Genesis, first Workpoint, and Canvas mode;
- acceptance receipts, rerun, repair, update, rollback, uninstall, and purge;
- declared Linux, macOS, and Windows capability boundaries.

### 3.2 Excluded

- inferred remote, organization, visibility, deployment, DNS, firewall, port, or system service;
- provider account creation or Focusa custody of provider credentials;
- bypassing BSL/commercial authority, Pi/UIAI lifecycle ownership, or project confirmation;
- project mutation before exact identity and explicit selection;
- silently converting or overwriting an existing project.

## 4. Authority and compatibility matrix

| Owner | Existing authority | Spec 150 relationship | Conflict rule |
|---|---|---|---|
| Spec 112 | Binary architecture, artifacts, targets, checksums, rollback foundation | Import | Spec 112 owns binary mechanics |
| Spec 128 | OTA/update, installer intelligence, development licensing | Import | Spec 128 owns update/license mechanics |
| Spec 132 | Terminal animation and TTY behavior | Import | UX cannot weaken gates |
| Specs 135/135i | Workspace, CRIST, Genesis, generated Mission Canvas | Import | Genesis/Canvas remain project-scoped |
| Spec 140 | Instruction authority and cross-harness compilation | Import | Generated instructions stay governed |
| Spec 140A | Integrity, canonical amendment, headless enforcement | Import | Headless preserves equivalent gates |
| Spec 142 | Pi continuation and onboarding dependencies | Import | Pi owns queues/continuation |
| Spec 143 | Release-cycle trajectory and Genesis implementation | Import | Release proof differs from install proof |
| Specs 145–147 | Canonical release architecture and operations | Reference | Consume only verified release artifacts |
| Spec 148 | Benchmark journal | Reference | Install failures may settle predictions |
| Spec 149 | Workset Flow Ledger | Reference | Implementation needs admitted workset |
| GitHub #14 | Pi lifecycle/compaction authority | Blocking prerequisite | Complete before Pi-guided install implementation |

Spec 150 supersedes no existing owner by default. A contradiction fails closed and is resolved by the owner or a named amendment.

## 5. Current-code compatibility matrix

| Surface | Current reality | Issue | Required disposition before publication |
|---|---|---|---|
| Bash bootstrapper | Dry-run, eval, target, channel, dependencies, service, force, uninstall, purge-data | Broadest public lifecycle contract | Keep authoritative until parity is proven |
| Bash uninstall | Adds `--keep-data` unless purge is explicit | Safer than direct CLI | Preserve default and prove it |
| `focusa uninstall` | Needs `--keep-data` to preserve customer state | Contradicts universal preservation claim | Unify default or distinguish entrypoints visibly |
| PowerShell bootstrapper | Thin Windows installer | Missing Bash uninstall/dependency/service/force/license parity | Implement parity or publish exact limitations |
| CLI install | Installs trusted artifacts to resolved target | Does not own provider/project onboarding | Host stage only |
| CLI update | Uses release/update metadata | Compatibility spans daemon, extension, skills, UI | Require version-set rollback proof |
| Project Bootstrap API | Preview/apply/status/repair plus confirm/idempotency | Must not merge into host install | Run only after explicit project selection |
| Project Genesis API | Start/resume/status/commit plus confirm/takeover | HLT impasse/takeover governed | Preserve native state machine |
| Pi extension | Project, Workpoint, Canvas, preload, continuation | Scope/compaction regressions can corrupt setup | P0 compatibility issues are prerequisites |
| Mission Canvas | Project-scoped generated interaction and modes | Incoherent before identity/Genesis | Offer after project acceptance gate |
| UIAI | Optional governed browser capability | Availability/pressure vary | Detect and verify; do not require |
| Providers | Harness/provider-owned | Focusa must not ingest secrets | Discover then provider-native handoff |
| Instructions | Specs 140/140A compiler/governance | Existing files may be operator-owned | Preview, diff, provenance, confirm, receipt |
| Task provider | Bootstrap accepts explicit policy | Existing stores can be damaged | Detect, preview, preserve |
| Git | Bootstrap accepts explicit initialization | Remote/org/visibility not inferable | Explicit init; remote excluded |
| Mac menubar | Separate pairing/install surface | Signing/update/pairing platform-specific | Optional independent receipt |

### 5.1 Publication blockers

| Priority | Blocker |
|---|---|
| P0 | Bash and direct CLI disagree on uninstall preservation defaults |
| P0 | Windows lifecycle parity is incomplete or undocumented |
| P0 | Host and project scopes must not collapse into one transaction |
| P0 | GitHub #14 Pi compaction/queue compatibility remains unresolved |
| P0 | Exact-project scope regressions must not contaminate onboarding |
| P1 | Repair is not uniformly defined across entrypoints |
| P1 | Provider/UIAI optionality needs explicit capability contracts |
| P1 | Cross-artifact update/rollback needs one compatible-version proof |
| P1 | Existing user guides can drift from implementation truth |

## 6. Transaction boundaries

### 6.1 HostInstallTransaction

```text
preflight → license posture → channel/target → artifact trust
→ dependency policy → binary install → service policy → daemon health
→ optional host integrations → host receipt
```

Host installation cannot create, initialize, bind, or modify a project.

### 6.2 ProjectOnboardingTransaction

```text
explicit project candidate → exact identity → verification → existing/new classification
→ discipline preview → confirmed bootstrap apply → Genesis start/resume
→ confirmed Genesis commit → first Workpoint → optional Canvas mode → project receipt
```

### 6.3 LifecycleMaintenanceTransaction

Exactly one explicit action: `inspect | rerun | repair | update | rollback | uninstall | purge`. Purge cannot hide inside uninstall.

## 7. Lifecycle state machine

Primary states:

```text
uninspected → preflighted → artifact_selected → artifact_verified
→ host_installed → daemon_ready → host_accepted → integrations_selected
→ integrations_verified → project_selected → project_verified
→ project_bootstrap_previewed → project_bootstrapped → genesis_started
→ genesis_committed → first_workpoint_ready → experience_selected → accepted
```

Recovery states:

```text
blocked_unsupported_host | blocked_license | blocked_artifact_trust
blocked_permission | blocked_scope | blocked_confirmation
blocked_provider_handoff | blocked_dependency | partial_host_install
partial_integration | partial_project_bootstrap | degraded_daemon
rollback_required | operator_action_required
```

Every transition records transaction id, typed scope, prior/new state, typed action, status, evidence refs, bounded recovery, and timestamp. Unknown completion requires inspection before retry.

## 8. Explicit selection dimensions

Spec 150 does not invent unsupported monolithic install profiles. It coordinates existing dimensions independently.

| Dimension | Choices | Default rule |
|---|---|---|
| Interaction | interactive, headless | TTY detection; no hidden headless consent |
| Authorization | evaluation, commercial, authorized development | Never infer entitlement |
| Channel | stable, preview, nightly where supported | Stable unless explicit |
| Target | detected supported target or explicit override | Ambiguity blocks |
| Dependencies | approved install, verify-only | Interactive confirmation unless pre-approved |
| Service | supported user service, no service | No system-wide assumption |
| Integrations | individually selected Pi, UIAI, menubar, declared harness, none | Optional/capability-gated |
| Project | skip, explicit existing path, explicit new path | Skip until explicit |
| Git | preserve, explicit initialize, skip | Never create/configure remote |
| Task provider | preserve, explicit supported provider, skip | Never overwrite |
| Instructions | preserve, governed preview/generate, skip | Never silently write |
| Canvas | guided, full, off, leave unchanged | Offer after project readiness |
| Maintenance | inspect, rerun, repair, update, rollback, uninstall, purge | Preview precedes mutation |

## 9. Guided decision graph

```text
START
├─ supported host? no/unknown → stop with evidence
├─ existing install? healthy → inspect path; degraded → repair proposal; absent → install
├─ interactive? explain choices; headless? explicit config and machine receipt
├─ optional integrations? detect each; absence cannot fail core
├─ configure project now? no → resumable continuation; yes → explicit path
├─ existing project? verify/preserve; new project? require explicit location/end state
├─ bootstrap preview accepted? no → preserve; yes → confirmed apply
├─ sufficient HLT? no → one intent question or blocked receipt; yes → confirmed Genesis
├─ first canonical Workpoint? absent → completion blocked
└─ Canvas guided/full/off/unchanged → final proof
```

## 10. Preflight contract

Read-only preflight reports:

- OS/architecture, user/home boundary, shell/TTY, supported target;
- existing binaries, daemon, services, extension, skills, state, and version set;
- writable user-safe targets without implicit escalation;
- required/optional dependencies and network/offline posture;
- artifact/update metadata reachability and license posture without secret values;
- Pi/UIAI/menubar capabilities;
- no project inspection until an explicit path is supplied.

Each finding is `required`, `optional`, `already_satisfied`, `operator_choice`, `unsupported`, or `blocked`.

## 11. Artifact trust and activation

Production installation requires declared version/channel, supported target, complete metadata, checksums/signatures, provenance, staged extraction, rollback metadata before replacement, atomic activation where supported, and post-activation version/health proof. No fallback may silently substitute a local build, stale asset, different channel, or partial version set.

## 12. Integration contracts

### 12.1 Pi

- detect Pi compatibility before mutation and install matching extension/skills;
- never submit lifecycle prompts during startup or compaction;
- use non-triggering next-turn guidance when guidance is required;
- preserve provider, queue, cancellation, compaction, and reconnect ownership;
- prove tool discovery with one read-only scoped call;
- keep core CLI/daemon acceptance independent of Pi.

### 12.2 UIAI

UIAI is optional. Verify health, pressure, and capability; bind actions to session/origin; preserve normal confirmation/evidence; degrade to core installation when absent; never install unrelated browser software silently.

### 12.3 Providers

Discover through the selected harness, present provider-neutral choices, use provider-native OAuth/device/auth, never persist provider secrets, and allow resumable project setup when no provider is available.

### 12.4 Mac menubar

macOS-only integration requires separate signature/update verification, explicit device pairing, revocation/expiry behavior, and independent health/uninstall receipt.

## 13. Project Bootstrap and Genesis

Required order:

```text
project_identity → project_verify → bootstrap preview → operator review
→ bootstrap apply(confirm) → Genesis start/resume → Genesis commit(confirm)
→ Workpoint checkpoint/resume → optional Canvas selection
```

Scope mismatch blocks project mutation without invalidating a healthy host installation.

### 13.1 Git and task provider

- preserve existing repository, dirty worktree, and task store;
- initialize only when explicitly selected and previewed;
- never infer repo name, branch, host, organization, visibility, or remote;
- detect and preserve existing task provider; never invent tasks;
- return optional remote setup as a continuation, not an install side effect.

### 13.2 Instructions and documentation

Governed generation requires canonical source/version, target harness, proposed files, preview/diff, conflict classification, explicit confirmation, atomic write/rollback, hashes/authority receipt, and no overwrite of operator-owned instructions without approved reconciliation.

### 13.3 First Workpoint and Canvas

Project onboarding is incomplete until an exact-scoped canonical Workpoint exists or a typed blocked receipt explains why. Canvas selection follows project verification, Genesis context, and first Workpoint readiness:

| Mode | Meaning |
|---|---|
| `guided` | bounded assisted interaction |
| `full` | full generated Mission Canvas |
| `off` | no Canvas interaction |
| `leave_unchanged` | preserve persisted preference |

Headless mode uses equivalent authority and evidence.

## 14. Secrets, permissions, and data

Secret classes: Focusa license, provider credentials, device tokens, signing/update verification material, and discovered project secrets. Never include values in previews, logs, receipts, diagnostics, or Workpoints; prefer approved environment/device/provider-native handoff; warn about command-line exposure; preserve least-privilege ownership.

Data classes must be independently itemized:

```text
managed binaries | services | integrations | Focusa state | logs/caches
license state | provider/harness state | project files | project task data
operator-authored instructions
```

## 15. Maintenance semantics

| Action | Contract |
|---|---|
| Inspect | Read-only version-set, health, drift, capabilities, next actions |
| Rerun | Replay declared intent idempotently; no implicit upgrade/downgrade/repair/purge |
| Repair | Diagnosed drift, bounded proposal, preserved state, confirmation, rollback, proof |
| Update | Trusted complete compatible version set with rollback authority |
| Rollback | Restore last verified set while preserving newer user/project data by default |
| Uninstall | Remove managed executable/service/integration artifacts; preserve user/project data |
| Purge | Explicit, destructive, separately confirmed, itemized |

The desired preservation contract is not implemented until direct CLI and public entrypoints agree. Provider-secret and project deletion remain outside purge unless their owner explicitly authorizes them.

## 16. Platform capability matrix

| Capability | Linux | macOS | Windows | Publication rule |
|---|---:|---:|---:|---|
| Core CLI/daemon | prove | prove | prove | Signed per-target artifact |
| User service | systemd-user where supported | launchd where supported | declare actual mechanism | No inferred parity |
| Bash lifecycle | supported targets | supported targets | N/A | Shell contract |
| PowerShell lifecycle | not primary | not primary | current install path | List gaps |
| Pi extension | compatible Pi | compatible Pi | compatible Pi | Independent proof |
| UIAI | optional | optional | optional | Absence cannot fail core |
| Menubar | no | signed app | no | macOS-only language |
| Data-preserving uninstall | prove | prove | prove or declare absent | Ambiguity blocks publication |
| Update/rollback | prove set | prove set | prove set | No partial-success claim |

Architecture/version support derives from release metadata, not this table alone.

## 17. Failure and recovery matrix

| Failure | Response | Forbidden |
|---|---|---|
| unsupported_host | stop with target evidence | force another target |
| artifact_incomplete | block | install partial assets |
| trust_failure | preserve prior install | bypass verification |
| permission_boundary | user-safe proposal or explicit escalation | silently use root/system scope |
| daemon_degraded | diagnose/preserve host transaction | begin project mutation |
| integration_incompatible | mark optional integration blocked | fail healthy core |
| scope_mismatch | stop project transaction | infer cwd/stale packet |
| confirmation_missing | preview/continuation | mutate |
| provider_unavailable | resumable handoff | capture provider secret |
| project_conflict | preserve and diff | overwrite |
| update_partial | rollback or degraded receipt | report success |
| uninstall_ambiguous | itemized preview | delete data |
| unknown_completion | inspect receipts/state | blind retry |

One primary classification survives; cleanup noise cannot replace root cause.

## 18. Receipts and false-completion gates

Host receipt proves target/version set, trust refs, daemon health, service posture, selected integration outcomes, preserved-data policy, and recovery/update/uninstall actions. Project receipt proves exact identity, Bootstrap refs, Git/task/instruction choices, Genesis/HLT status, Workpoint ref, Canvas outcome, and deferred optional work.

Completion is false if artifact trust is absent, version set incompatible, daemon unhealthy, required service failed, project mutated without exact scope, confirmation missing, Workpoint absent, secret leaked, uninstall preservation ambiguous, rollback unavailable after replacement, or platform behavior lacks evidence.

## 19. Planned implementation sequence

Planning may proceed while implementation remains blocked behind compaction compatibility.

0. Complete GitHub #14 Pi compaction/lifecycle compatibility.
1. Ratify Spec 150 matrices.
2. Create the requirement ledger and Beads decomposition.
3. Resolve P0 code/document contradictions.
4. Implement host orchestration and receipts.
5. Implement optional integration capability flow.
6. Implement project Bootstrap/Genesis orchestration.
7. Implement maintenance parity and destructive-data gates.
8. Run platform and interactive/headless conformance.
9. Reconcile and publish user-facing guides.
10. Settle predictions, evidence, and release gate.

This file does not admit Spec 150 implementation into the locked release. Admission requires explicit operator authorization and a new lock revision.

## 20. Required conformance coverage

- clean, healthy-existing, degraded, and partial install;
- interactive/headless; evaluation/commercial/development authority;
- supported channels/targets; dependency install/verify-only; service/no-service;
- Pi absent/present/incompatible/busy/compacting;
- UIAI absent/healthy/saturated; provider configured/missing/auth-blocked;
- project skipped/existing/new/conflicting;
- Git existing/explicit-init/skip; task provider preserve/init/skip;
- instructions preserve/generate/conflict;
- Genesis sufficient/HLT impasse/takeover conflict;
- Canvas guided/full/off/unchanged;
- rerun/repair/update/rollback/uninstall/purge;
- each declared Linux/macOS/Windows target;
- offline/interrupted network, incomplete artifact, trust failure, unknown completion.

## 21. Documentation plan

After code-compatible requirements and conformance evidence:

1. reconcile `docs/current/FOCUSA_FRIENDLY_ONBOARDING.md` as truthful quickstart;
2. reconcile `docs/current/INSTALLER_UPDATE_POLICY.md` with maintenance semantics;
3. cross-link owners rather than duplicating them;
4. publish exact platform and destructive-data tables;
5. include recovery commands and receipt interpretation;
6. prevent user guides from leading implementation evidence.

### 21.1 Guided CLI lifecycle contract (phase 4)

The direct `focusa install`, `focusa update ...`, and `focusa uninstall` commands remain compatible. A guided transaction is selected with the global `--lifecycle-action` option:

```text
focusa install --lifecycle-action inspect
focusa install --lifecycle-action preview
focusa install --lifecycle-action apply --confirm
focusa install --lifecycle-action resume --confirm
focusa install --lifecycle-action repair --confirm
focusa install --lifecycle-action rerun --confirm
focusa update apply --lifecycle-action preview
focusa update apply --lifecycle-action apply --confirm
focusa update rollback --lifecycle-action rollback --confirm
focusa uninstall --lifecycle-action uninstall --confirm
focusa uninstall --lifecycle-action purge --confirm --confirm-purge-data
```

`inspect`, `preview`, and `confirm` do not mutate. A mutation without `--confirm` returns an `operator_required` receipt and an exact recovery command. Purge remains distinct from uninstall and requires the additional `--confirm-purge-data`; uninstall continues to preserve user data by default. `--json` emits `focusa.cli.lifecycle.receipt.v1`, including the typed operation, transaction transition, adapter contract, status, and recovery without credential values. The ordinary command's existing result and degraded/operator-required handling remain authoritative after a guided action starts applying; the guidance receipt never claims completion.

## 22. Planning acceptance criteria

- [ ] every imported owner acknowledges its boundary;
- [ ] each code path is supported, unsupported, or planned;
- [ ] Bash, PowerShell, and direct CLI differences are resolved or gated;
- [ ] host/project transactions are independently modeled;
- [ ] mutation, confirmation, idempotency, rollback, and receipts are explicit;
- [ ] provider, Pi, UIAI, Canvas, Git, task, and instruction boundaries are explicit;
- [ ] uninstall and purge data classes are proven;
- [ ] platform claims have executable evidence;
- [ ] a complete ledger maps requirements to Beads and proof;
- [ ] guides reconcile only after implementation truth;
- [ ] locked scope changes only with explicit authorization;
- [ ] GitHub #14 remains first implementation task.

## 23. Source references

Specifications: `docs/112-install-binary-architecture-spec.md`, `docs/112-install-binary-architecture-audit.md`, `docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md`, `docs/132-focusa-installer-animated-terminal-experience-spec.md`, `docs/135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md`, `docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md`, `docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md`, `docs/140a-foundational-instruction-integrity-temporal-adaptation-canonical-amendment-and-headless-enforcement-addendum.md`, `docs/142-focusa-seamless-pi-continuation-and-workflow-dependency-onboarding-spec.md`, `docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md`, and `docs/149-focusa-workset-flow-ledger-and-release-completion-spec.md`.

Implementation: `scripts/install-focusa.sh`, `scripts/install-focusa.ps1`, `scripts/magic/install.sh`, `crates/focusa-cli/src/commands/install.rs`, `crates/focusa-cli/src/commands/update.rs`, `crates/focusa-cli/src/commands/uninstall.rs`, `crates/focusa-api/src/routes/project_bootstrap.rs`, `crates/focusa-api/src/routes/project_bootstrap_support.rs`, `crates/focusa-api/src/routes/project_genesis.rs`, `crates/focusa-api/src/routes/project_genesis_support.rs`, `apps/pi-extension/src/mission-canvas-model.ts`, and `apps/pi-extension/src/commands.ts`.

Current guides: `docs/current/FOCUSA_FRIENDLY_ONBOARDING.md` and `docs/current/INSTALLER_UPDATE_POLICY.md`.

Issue authority: <https://github.com/Startempire-Wire/focusa/issues/14> and <https://github.com/Startempire-Wire/focusa/issues/14#issuecomment-5123704895>.
