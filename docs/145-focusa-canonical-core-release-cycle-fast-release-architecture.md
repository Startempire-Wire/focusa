# Spec145 — Focusa Canonical Core Release Cycle and Fast Release Architecture

**Status:** implementation in progress under GitHub #56 / Bead `focusa-vbcqu.6.1`  
**Supersedes:** implicit and fragmented release behavior; does not supersede Spec143  
**Extends:** Spec107, Spec128, Spec131, Spec133, Spec136, Spec137, Spec143  
**Call-stack design:** `019fa147-b0ff-7751-a578-029bdd131c8d`

## 1. Purpose

Define one provider-neutral release control plane that can release one or many
surfaces quickly without weakening scope, evidence, provenance, rollback, or
installed/runtime truth.

The release cycle is a governed state machine. GitHub Actions is the first
provider adapter, not the architecture itself. Focusa is the first complex
multi-surface profile, not a special-case truth model.

## 2. Operator contract

One operator command to release means:

1. bind the exact project and workstream;
2. resolve intended version, SHA, topology, surfaces, and gates;
3. freeze the candidate scope;
4. reject unrelated work;
5. execute the shortest valid evidence-producing DAG;
6. promote one immutable artifact set;
7. verify every intended installed/running surface;
8. unlock only after completion, rollback, cancel, or approved replan;
9. record speed, retries, wasted time, and reusable learning.

A tag, draft, green partial workflow, uploaded asset, local binary, or agent
statement is never release completion.

## 3. Non-negotiable invariants

- Candidate identity is `project_root + continuity_id + workpoint_id + version + exact_sha`.
- Artifacts are immutable and identified by SHA-256.
- Evidence is accepted only for the exact candidate SHA.
- Source truth, artifact truth, installed truth, and running truth are distinct.
- Scope expansion after lock is rejected unless it is a bounded proven blocker fix or an operator amendment.
- A blocker fix names the failed gate, affected surfaces, expected proof, invalidated evidence, and candidate-retag requirement.
- Release-managed state never overwrites user data, `.env`, license, project, Workpoint, evidence, ownership, permissions, xattrs, or capabilities.
- Failed apply/deploy exits nonzero after rollback; rollback success does not convert the failed operation into success.
- No release mutates an existing remote tag or published artifact.
- Provider APIs execute work; they do not redefine release authority.
- No parallel version, timing, Workpoint, evidence, artifact, or release-truth store.

## 4. Authority hierarchy

Highest to lowest:

1. current operator release/ship instruction;
2. verified ProjectIdentity;
3. release-scoped Workpoint and Bead graph;
4. active Trajectory HLT/MLG/STG/Waypoints;
5. ReleaseCandidate lock and admitted fix lanes;
6. exact-SHA evidence and provider receipts;
7. workflow/adaptor observations;
8. transcript and inferred intent.

When Workpoint writer authority is unavailable, release execution must use the
operator instruction plus verified GitHub issue/Bead/spec and record degraded
continuation explicitly. Missing Workpoint authority never permits broad scope.

## 5. Canonical state machine

```text
PLAN
  -> LOCKED
  -> CANDIDATE_SNAPSHOTTED
  -> PREFLIGHTED
  -> BUILT
  -> PACKAGED
  -> PROVENANCED
  -> DRAFT_PUBLISHED
  -> CANARY_DEPLOYED
  -> VERIFIED
  -> PROMOTED
  -> CLOSED

any nonterminal stage
  -> bounded fix lane -> revalidate affected stage and descendants
  -> ROLLED_BACK
  -> CANCELLED
```

Rules:

- Normal transitions advance exactly one stage.
- Every transition carries evidence refs, exact SHA, and observed timestamp.
- Terminal candidates cannot accept fixes or additional promotion.
- A fix that changes source requires a new SHA and therefore a new candidate.
- A fix to provider infrastructure may reuse the candidate only when artifacts and source are unchanged and invalidated evidence is named.
- Promotion requires all global and surface gates green.

## 6. Canonical core model

Owner: `crates/focusa-core/src/release_cycle.rs`.

### 6.1 `ReleaseTopology`

```text
schema
project_id
profile
provider
surfaces[]
  surface_id
  kind
  depends_on[]
  required_gates[]
  artifact_identity
  deployment_target?
  canary_required
  rollback_required
global_gates[]
```

Validation rejects:

- unsupported schemas;
- empty project/profile/provider;
- no surfaces;
- duplicate or empty surface IDs;
- unknown or self dependencies;
- dependency cycles;
- surfaces without artifact identity or gates;
- rollback-required surfaces without deployment targets.

### 6.2 `ReleaseCandidate`

```text
schema
candidate_id
project_root
continuity_id
workpoint_id
version
exact_sha
topology_ref
stage
locked_scope_refs[]
evidence[]
admitted_fixes[]
benchmark?
```

### 6.3 `ReleaseEvidence`

Every evidence item contains:

- transition stage;
- candidate exact SHA;
- observed timestamp;
- stable evidence refs;
- evidence invalidated by this observation.

### 6.4 `ReleaseFixLane`

A fix lane is valid only when it contains:

- one failed gate;
- known affected surfaces;
- required proof;
- invalidated prior evidence;
- whether a new candidate is mandatory;
- optional operator amendment reference.

### 6.5 `ReleaseBenchmark`

```text
total_elapsed_ms
useful_work_ms
queue_ms
retry_ms
human_interventions
retries
first_pass_gate_success_rate
flow_efficiency
critical_path[]
missed_target_reason_codes[]
stage timings[]
```

## 7. Provider-neutral topology profiles

### 7.1 Single package

```text
source -> test -> package -> provenance -> registry publish -> install verify
```

### 7.2 Multi-platform binary

```text
source/preflight
  -> linux gnu build
  -> linux musl build
  -> macOS arm64 build
  -> macOS x64 build
  -> Windows x64/arm64 build
  -> checksums/signatures/provenance
  -> publish
  -> platform install probes
```

### 7.3 Focusa multi-surface

```text
source/preflight
  -> core/API/CLI tests + clippy
  -> Spec132 terminal/target matrix
  -> daemon/CLI/TUI target builds
  -> Pi extension package/typecheck/lint
  -> menubar build/sign/update proof
  -> installer + agent-context packaging
  -> checksums/signatures/provenance
  -> draft release
  -> daemon canary/live deploy
  -> all-surface OTA/install proof
  -> promote and close
```

### 7.4 Service/container/web

```text
source -> test -> image/SBOM/provenance -> staging -> canary -> observe -> promote -> rollback receipt
```

### 7.5 UIAI Engine

UIAI supplies its own topology and adapters. It shares the kernel, release
lock, evidence contract, benchmarks, and provider interface; it does not copy
Focusa-specific workflows.

## 8. Call-stack design

### 8.1 Entry surfaces

- CLI: `focusa release cycle ...`
- API: `/v1/release/cycles/...`
- Pi: governed release tools discovered progressively
- operator shorthand: `/ Canonical Core Release Cycle`

### 8.2 Handler layer

Handlers:

1. parse operator intent;
2. verify ProjectIdentity;
3. resolve topology/profile/version/SHA;
4. require Workpoint or explicit degraded authority;
5. dispatch release service command;
6. render bounded state/evidence/next action.

Handlers never invoke provider commands before a valid lock exists.

### 8.3 Service layer

- `ReleasePlanner`: topology resolution and DAG validation.
- `ReleaseLocker`: candidate snapshot and scope freeze.
- `ReleasePreflight`: exhaustive target/gate discovery.
- `ReleaseExecutor`: ready-node scheduling and bounded parallelism.
- `ReleaseEvidenceService`: exact-SHA proof acceptance/reuse.
- `ReleaseFixLaneService`: blocker classification and invalidation.
- `ReleasePromotionService`: draft/canary/verify/promote.
- `ReleaseRollbackService`: artifact/service/data-safe restore.
- `ReleaseBenchmarkService`: temporal pulse and learning packet.
- `ReleaseIntelligenceService`: evidence-backed release page.

### 8.4 Adapter layer

Provider interface:

```text
resolve_source(candidate)
start_gate(node)
observe_gate(node)
collect_evidence(node)
publish_draft(candidate)
deploy(surface, artifact)
verify(surface, target)
promote(candidate)
rollback(surface, receipt)
```

First adapter: GitHub Actions + GitHub Releases. Future adapters: local CI,
GitLab, package registries, container registries, Tauri updater, deployment
systems, and UIAI-specific delivery.

### 8.5 Storage layer

Canonical references:

- Workpoint/Beads: release task and authority graph;
- Trajectory: goal/waypoint alignment;
- Focusa evidence: proof handles;
- existing `release-proof/`: immutable release proof artifacts;
- GitHub runs/releases: provider execution receipts;
- Spec137 temporal ledger: benchmark events;
- update history/rollback journal: installed-surface promotion receipts.

No new store duplicates those owners. ReleaseCandidate materializations are
indexes over canonical refs and must remain reconstructable.

### 8.6 Output layer

Every output includes:

- candidate ID, version, SHA, and stage;
- locked scope and topology ref;
- current/failed/ready gates;
- installed/running truth by surface;
- evidence refs and unproven claims;
- elapsed/queue/retry/critical-path pulse;
- exact next action;
- rollback/cancel path.

## 9. GitHub Actions adapter DAG

### 9.1 Pull-request/pre-candidate

- CI runs on PR and main.
- Spec132 path ownership includes every terminal/runner/updater surface.
- Stale PR runs cancel automatically; locked-candidate `main` runs never cancel when audit commits arrive.
- Cross-target failures are discovered before immutable tag creation.
- Rust caches are keyed by lockfile, toolchain, target, job, and relevant env.

### 9.2 Candidate/tag

- `Release` triggers only on `v*-dev` tags.
- No branch-push no-op Release workflow pollutes badge/history.
- Open release-gate issues and open PRs block the candidate.
- Version surfaces and release topology are verified.
- CI/Spec132 evidence is reused when exact source SHA and inputs match.
- Candidate-specific changes run only invalidated gates.

### 9.3 Build/package

Independent target builds fan out. Fail-fast applies to a candidate, while
logs/artifacts from completed siblings remain evidence. Packaging never
rebuilds a binary already produced for the exact SHA/toolchain/input digest.

### 9.4 Provenance/publish

Checksums, detached signatures, manifest signature, provenance, and expected
asset inventory must agree before release promotion. Draft publication does
not imply completion.

### 9.5 Deploy/verify

Deploy consumes the immutable release asset, never `target/release` or moving
`main`. It takes a rollback backup, installs atomically, restarts, checks health
version and API contract, and emits signed deploy proof. Failed deploy exits
nonzero even when rollback succeeds.

### 9.6 Close/learn

The final receipt records stage timings, critical path, cache hit/miss, retries,
interventions, failed gates, recovery time, and next calibrated improvement.

## 10. Operations, proof, and rollout

Detailed OTA operation, speed controls, benchmark, security, migration, acceptance, proof commands, and rollback are normative in [`146-focusa-canonical-release-cycle-operations-and-proof-runbook.md`](146-focusa-canonical-release-cycle-operations-and-proof-runbook.md).
