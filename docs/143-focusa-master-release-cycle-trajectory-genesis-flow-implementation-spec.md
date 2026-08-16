# Spec143 — Focusa Master Release Cycle, Trajectory Ladder, Project Genesis, and Frictionless Flow Implementation

**Status:** LOCKED — W0 IMPLEMENTATION-READINESS REVIEW COMPLETE  
**Authority:** operator-locked next-dev scope in `docs/142-focusa-release-requirement-trace-matrix.md`  
**Root Bead:** `focusa-vbcqu`  
**Active Bead:** `focusa-vbcqu.1.2`  
**Release posture:** no source implementation or candidate creation until this spec and W0 final review close

## 1. Purpose

This specification turns the locked master Release Cycle architecture into one implementable contract. It consolidates existing valid requirements rather than creating competing authority systems.

It governs:

- ProjectIdentity and mandatory Trajectory Ladder integrity;
- greenfield/brownfield Project Genesis and HLT Impasse;
- deliberate MLG/STG/Waypoint inference;
- full Trajectory event history/query/fallback;
- task, Workpoint, Focus Stack, evidence, decision, release, and surface integration;
- Frictionless Project Flow Kernel and ProjectFlowPacket;
- Pi ambient mode and toggleable complete Spec135–135K Mission Canvas journey;
- project-neutral release topology/profiles/state machine;
- intelligent evidence-backed release pages;
- exact-SHA acceptance, deployment truth, rollback, and speed/friction learning.

## 2. Normative sources and supersession

| Source | Preserved authority | Spec143 action |
|---|---|---|
| Spec96 | Project/Trajectory projection, hot-path stability, Workpoint handoff | consolidate schema and remove lazy projection authority |
| Spec102 | HLT persistence, Trajectory Ladder doctrine | preserve HLT durability; replace milestone overlap with Waypoint |
| Spec105 | DXUX scope, doability, recovery, evidence, drift UX | make Flow Kernel acceptance mandatory |
| Spec109/111 | agent-first API and preload | consume one ProjectFlowPacket |
| Spec116/119/131 | provider-neutral closure, receipts, Workpoint timing/closure | integrate into continuous advance transaction |
| Spec125 | mandatory HLT, fallback/history, Pi bootstrap/resume | strengthen to HLT Impasse and full Ladder ledger/query |
| Spec130 | compaction mission/Trajectory recovery | consume marker guard, ledger snapshot, and ProjectFlowPacket |
| Spec135–135K | Project Genesis/Mission Canvas complete frozen journey | protected downstream contract; surgical amendments only |
| Spec137 | temporal authority and release timing | integrate deadlines/urgency into Flow/release decisions |
| Spec138 | prediction/metacognitive governance | advisory learning only; never Trajectory authority |
| Spec141 | generated capability/tool/docs release gate | regenerate and prove every changed contract |
| Spec142 trace matrix | locked scope, issue/Bead DAG, no-pass truth | controls admission and closure |

When wording conflicts, Spec143 governs the locked release implementation while preserving stronger safety, authority, privacy, and no-deferral constraints. W0.3 must record every superseded clause and migration.

## 3. Locked invariants

1. Project and Trajectory Ladder are inseparable.
2. HLT means High Level Trajectory; MLG means Mid Level Goal; STG means Short Term Goal.
3. Waypoint is the only Trajectory Ladder checkpoint term.
4. No lazy, generic, generated-on-read, backfilled, or placeholder Trajectory exists.
5. New and brownfield projects without valid committed HLT enter HLT Impasse.
6. Operator sets or explicitly confirms HLT; lower Ladder inference begins only afterward.
7. Workpoint remains immediate execution authority and links through HLT→MLG→STG→Waypoint→task.
8. Every mutation is scoped, authorized, idempotent, receipt-backed, and event-recorded.
9. Every surface consumes one authoritative project/flow version.
10. Mission Canvas is optional; Pi ambient mode remains complete; both share one substrate.
11. Spec135–135K unknown impact blocks implementation and release.
12. Accepted release candidates are immutable and exact-SHA proven.
13. Build success is not deployment or shipment proof.
14. No release closes while a locked issue/Bead or required proof lacks a disposition.

## 4. Canonical object model

```text
ProjectRecord
├─ ProjectIdentity
├─ TrajectoryBinding
│  ├─ HLT lineage
│  ├─ active MLG
│  ├─ active STG
│  ├─ ordered Waypoints
│  └─ event ledger/snapshot refs
├─ TrajectoryIntegrityGuard
├─ SpecificationBinding
├─ TaskProviderBinding + TaskGraph
├─ WorkpointBinding
├─ FocusStackBinding
├─ Evidence/Decision/Constraint bindings
├─ ReleaseTopologyBinding
└─ ProjectFlowState
```

Directional authority:

```text
operator steering + safety
→ ProjectIdentity
→ HLT
→ MLG
→ STG
→ Waypoints
→ task graph
→ Workpoint immediate action
→ evidence/current-state update
```

## 5. Project marker contract

The canonical marker schema adds:

```json
{
  "project_identity": {},
  "trajectory_binding": {
    "schema_version": "focusa.trajectory_binding.v1",
    "active_trajectory_id": "trajectory:...",
    "active_hlt_id": "hlt:...",
    "active_hlt_version": 1,
    "hlt_lineage": [],
    "active_mlg_id": "mlg:...",
    "active_stg_id": "stg:...",
    "active_waypoint_ids": [],
    "active_workpoint_id": "workpoint:...",
    "event_ledger_ref": "focusa://...",
    "latest_snapshot_ref": "focusa://...",
    "ledger_digest": "sha256:...",
    "authority": "canonical",
    "freshness": {},
    "status": "READY"
  },
  "trajectory_integrity_guard": {
    "required": true,
    "project_scope_fingerprint": "...",
    "expected_trajectory_id": "...",
    "expected_hlt_id": "...",
    "expected_hlt_version": 1,
    "expected_ledger_digest": "...",
    "minimum_ladder_complete": true,
    "causal_chain_valid": true,
    "schema_supported": true,
    "no_placeholder_values": true,
    "ladder_links_complete": true,
    "projection_matches_ledger": true,
    "unresolved_conflicts": [],
    "last_verified_at": "...",
    "verification_receipt_ref": "...",
    "status": "READY",
    "repair_route": null
  }
}
```

Guard states: `READY`, `HLT_IMPASSE`, `ONBOARDING_REQUIRED`, `TRAJECTORY_REVIEW_REQUIRED`, `CONFLICTED`, `MIGRATION_REQUIRED`, `INTEGRITY_REPAIR_REQUIRED`, `ARCHIVED`.

The guard runs at bootstrap commit, project switch, session start/resume, compaction recovery, before durable mutation/release, and after sync/migration/repair/external change. It diagnoses from committed records and never generates Ladder content.

## 6. Project Genesis transaction

```text
DISCOVER PROJECT
→ VERIFY/CREATE IDENTITY
→ INITIALIZE AUTHORIZED ANATOMY
→ INVENTORY SPECS/TASKS/EVIDENCE/CODE/RELEASE FACTS
→ HLT IMPASSE
→ OPERATOR SET/CONFIRM HLT
→ BIND SPEC + ACCEPTANCE + CURRENT/DESIRED STATE
→ INFER/COMMIT MLG
→ INFER/COMMIT STG
→ INFER/COMMIT WAYPOINTS
→ DETECT/BIND TASK PROVIDER
→ IMPORT/DECOMPOSE/RECONCILE TASK GRAPH
→ CREATE FIRST WORKPOINT + INTERNAL OWNERSHIP ATOMICALLY
→ VERIFY MARKER GUARD
→ PROJECT READY
```

Greenfield receives a concise HLT definition journey. Brownfield builds a richer HLT proposal from specs, code/architecture, Git history, tasks, releases/deployments, evidence, decisions, constraints, and prior HLT lineage. A proposal remains staged bootstrap evidence until operator confirmation.

Interrupted Genesis resumes from its last committed transaction receipt. It never leaves a ready marker with partial or synthetic Ladder values.

## 7. Deliberate inference engine

Precondition: verified project, committed HLT, pinned specification/acceptance version, verified current state, explicit transaction.

Inputs:

- HLT version and desired end state;
- authoritative specification and acceptance graph;
- current state evidence;
- task graph and provider metadata;
- prior valid Ladder events;
- Workpoint/Focus Stack context as lower-level evidence only;
- decisions, constraints, risks, releases, and ontology links.

Algorithm:

1. build a bounded evidence packet;
2. generate distinct field-level MLG/STG/Waypoint candidates;
3. validate HLT→MLG→STG→Waypoint hierarchy and acceptance linkage;
4. reject generic, duplicate-level, raw-title substitution, stale/cross-scope, circular, impossible, or unmeasurable candidates;
5. score authority, provenance, HLT relevance, specification coverage, state fit, task/evidence support, specificity, feasibility, freshness, stability, and hierarchy separation;
6. return selected/rejected candidates with reasons, assumptions, uncertainty, and evidence;
7. atomically commit policy-authorized high-confidence lower levels or request one bounded operator decision;
8. emit inference, mutation, marker-guard, and readiness receipts.

Ordered `first_nonempty` substitution and read-time filler Waypoints are prohibited.

## 8. Trajectory event ledger and query

One append-only project ledger records authoritative HLT/MLG/STG/Waypoint, state/gap, spec, task, Workpoint, evidence, decision, Focus Stack, release, pause/archive/reopen, migration, and receipt events.

Every event includes stable IDs, project/continuity, HLT version, causal parent, event type, old/new typed values, actor/source/authority, provenance/confidence/reason, evidence refs, idempotency, schema version, Lamport time, wall time, and non-authoritative session metadata.

Required queries:

- project/continuity/session metadata/Trajectory/HLT version;
- Ladder level and linked spec/task/Workpoint/evidence/release;
- event/source/authority/confidence/status/time filters;
- cursor pagination and deterministic order;
- current, `as_of`, ancestry, descendants, supersession, field history;
- snapshot reconstruction and typed diff;
- drift and fallback-candidate selection/rejection proof.

`focusa trajectory history` returns full Ladder history. `focusa hlt history` is an HLT projection over the same ledger.

Fallback preserves project HLT lineage. Lower levels never leak across continuities by recency. Missing/stale lower levels require an explicit reassessment transaction. Conflicts remain `CONFLICTED` until a resolution event.

## 9. Waypoint migration

Canonical migration:

- `TrajectoryMilestoneRecord` → `TrajectoryWaypointRecord`;
- rich `waypoints` replace parallel string-waypoint/milestone fields;
- milestone IDs/active IDs/recommendations/capacities → Waypoint names;
- Trajectory ontology milestone references → `trajectory_waypoint`;
- canonical Pi/TUI/Canvas/API/CLI/MCP/docs/tests/receipts emit Waypoint only.

A versioned adapter reads legacy fields, performs deterministic ID-aware merge, preserves provenance, emits migration/conflict receipts, and never drops evidence/status. W0.3 must produce the exact field/file/data migration table.

## 10. Frictionless Project Flow Kernel

Flow states:

```text
PROJECT_DISCOVERY → PROJECT_BOUND → HLT_IMPASSE → HLT_COMMITTED
→ LADDER_INFERENCE → LADDER_READY → SPEC_TASK_SYNC → TASK_PATH_READY
→ WORKPOINT_ACTIVE → VERIFYING → ADVANCING
→ NEXT_WORKPOINT | GOAL_COMPLETE | GOVERNED_IMPASSE
```

Natural intents compose existing primitives: `start_resume`, `inspect`, `steer`, `sync`, `advance`, `handoff`, `recover`.

`advance` atomically verifies Workpoint evidence, updates current state/gap, reconciles affected Ladder/tasks, completes the Workpoint, and activates the next ready Workpoint or proven goal-complete state.

Safe reads/preflight/indexing/freshness/evidence capture are automatic. Consequential direction, external mutation, takeover, release, rollback, secrets, or destructive action require bounded authority.

## 11. ProjectFlowPacket

Required fields:

```text
schema/scope/version/freshness
ProjectIdentity + marker guard
flow state/readiness
authoritative HLT/MLG/STG/Waypoints
spec/acceptance authority
task provider/graph/ready tasks
Workpoint/owner/done condition/next action
Focus Stack/evidence/decisions/constraints
release state/drift
doability/safe automatic actions
at most one operator question
recovery/receipts/rehydrate refs
resource/token/surface posture
```

All surfaces consume the same packet and event version. Local shadows are caches, not authority.

## 12. Spec/task/Focus Stack/evidence integration

- specs constrain Ladder and acceptance;
- Ladder prioritizes task graph;
- task-provider changes trigger reconciliation;
- Workpoint evidence updates state/gap;
- Focus Stack reflects the active path without becoming HLT authority;
- code/Git/deployment facts inform current state only;
- releases link to HLT/MLG/STG/Waypoint/Workpoint/acceptance;
- changes refresh marker guard and every subscriber.

Provider outage preserves last verified graph and queues reconciliation; it never silently changes provider or fabricates tasks.

## 13. Mission Canvas Spec135–135K compatibility

The frozen series is a protected downstream contract. No Spec135L is created.

Every locked leaf/change carries:

```text
spec135_impact: none | indirect | direct | unknown
affected_135_specs[]
affected_primitives[]
affected_schemas_apis_events_storage[]
affected_pi_canvas_agent_surfaces[]
compatibility_behavior
migration_behavior
required_doc_amendments[]
required_tests[]
agent_handoff_refs[]
```

`unknown` blocks implementation promotion and release.

A `Spec135CompatibilityPacket` records exact-SHA semantic diff, affected frozen clauses/docs/primitives, ownership/authority/storage/event/API/tool changes, Pi/Canvas/headless impact, migrations/toggles/rollback, files/tests/evidence, unresolved blockers, implementation order, and rehydrate refs.

Amend `135-series-current-manifest.md` and only affected existing 135–135K docs under Spec135E/135D rules. Preserve unaffected wording and ordering. Mission Canvas agents receive the packet and amended refs before affected implementation.

## 14. Release control plane

Release topology is project-owned; kernel behavior is project-neutral.

State machine:

```text
PLANNED → SCOPED → PREFLIGHTED → CANDIDATE_FROZEN → BUILDING
→ BUILT → TESTING → ACCEPTED → PUBLISHING → PUBLISHED
→ DEPLOYING → DEPLOYED → VERIFYING → COMPLETE
```

Failure states: `BLOCKED`, `FAILED`, `ROLLING_BACK`, `ROLLED_BACK`, `RETRACTED`.

Profiles: Full Release, Dev Release, Hotfix, Docs/UI-only, Tauri-only, Dry Run.

One candidate identity is immutable. Exact-SHA proof is reused only when environment/toolchain/input equivalence is proven. Publication/deployment/completion remain distinct receipts.

The Focusa release uses only `scripts/create-dev-release-tag.sh --base 0.9 --push` after acceptance and the full CI→Release→Deploy→audit/self-heal/watchdog chain.

## 15. Intelligent release information

`ReleaseIntelligencePacket` includes release identity/profile/status/SHA/previous tag, purpose and Trajectory refs, material changes and impact, included/resolved work, exact proofs, unproven/failed checks, known issues, breaking changes, compatibility/migrations, install/upgrade/rollback, artifacts/platforms/checksums/signatures, source/artifact/installed/running truth, security/provenance, speed/friction deltas, contributors, traceability, and commits.

Rendered order:

1. release at a glance;
2. why this release exists;
3. material user/agent changes;
4. exact proof;
5. unproven gaps/known issues;
6. compatibility/migration/install/upgrade/rollback;
7. artifacts/deployment truth;
8. security/trust;
9. product/release-flow deltas;
10. issues/PRs/contributors/commits.

Static templates provide structure only. Unsupported generic claims, hidden failures, build-as-shipment wording, missing changed-surface coverage, packet/page disagreement, or commits-as-first-meaningful-content fail the release gate.

## 16. Audited implementation and migration map

Every row requires `spec135_impact`, migration, rollback, tests, and evidence before its implementation Bead may start.

| Current contract | Required canonical contract | Exact owner/surface |
|---|---|---|
| `TrajectoryProjectionRecord.waypoints: Vec<String>` plus `milestones: Vec<TrajectoryMilestoneRecord>` and `active_milestone_id` | one typed `Vec<TrajectoryWaypointRecord>` plus `active_waypoint_id`; legacy read adapter only | `crates/focusa-core/src/types.rs`, `reducer.rs`, serialization/property tests |
| goal-only `FocusaEvent::TrajectoryGoalDefined` projection | typed append-only events for every Ladder/state/link/conflict/migration transition | core event/reducer types and `crates/focusa-api/src/routes/trajectory.rs` dispatch |
| separate projection checkpoint and `hlt-ledger/<project-hash>/hlt.jsonl` | one authoritative project Trajectory ledger with HLT history as a projection; dual-read verification before cutover | daemon persistence, migration utility, snapshots, ledger digests |
| `/v1/trajectory/{view,define-goal,assess,propose-workpoint,checkpoint,resume}` plus `/v1/hlt/history` | add typed set/infer/revise/history/query/resolve and Flow/Genesis operations without breaking old reads | `crates/focusa-api/src/routes/{trajectory,project,workpoint}.rs`, route schemas |
| `focusa trajectory` and `focusa hlt` overlapping command families | preserve aliases, expose full Ladder history/query and explicit Genesis/Flow journeys | `crates/focusa-cli/src/commands/{trajectory,hlt,project}.rs` |
| identity-only `.focusa-project.json` schema `focusa.project.v1` | additive `trajectory_binding` and `trajectory_integrity_guard`; atomic temp-write/rename and rollback copy | project resolver/route plus marker migration and corruption tests |
| Pi tools independently assemble Project/Trajectory/Workpoint context | one versioned ProjectFlowPacket and operation registry, with old tools as compatible projections | `apps/pi-extension/src/{tools,state,session,compaction,awareness-substrate}.ts` |
| menubar `TrajectoryPeek` and existing Mission Canvas generated surfaces | same event/version packet, toggle-only Canvas, complete ambient Pi parity | `apps/menubar/src/lib/components/TrajectoryPeek.svelte`, Canvas/TUI registry/views |
| provider/task and Workpoint routes are separately advanced | one idempotent Flow transaction coordinating provider reconciliation, evidence, task, Workpoint, Ladder, and Focus Stack | `task_plans.rs`, `provider_execution.rs`, `workpoint.rs`, core provider execution |
| release route/CLI/scripts/workflows hold distributed state | project-owned topology plus kernel state/receipt model and exact-SHA gates | `routes/release.rs`, CLI release command, release scripts/workflows/proof |
| static release-note structure | `ReleaseIntelligencePacket` → deterministic evidence-backed renderer/gate | release API/CLI, generator, proof fixtures, published release page |
| frozen Spec135–135K consumes changing primitives | `Spec135CompatibilityPacket`, UNKNOWN blocker, surgical existing-doc amendments | 135 manifest, affected existing specs, generated contracts/tests, `focusa-vbcqu.4.4` |

Protected modified files in Spec142 remain read-only until ownership/diff reconciliation. W1 creates focused core modules for `trajectory::{model,ledger,inference,guard,migration}` and `project_flow`; route and surface layers contain no competing business authority.

## 17. Normative call stacks

| Operation | Entry → handler | Service → adapter/storage | Output and closure |
|---|---|---|---|
| Genesis | Pi/CLI/API `project genesis start|resume|status|commit` → project route | Genesis service → identity/spec/task adapters → marker + Trajectory ledger transaction | Genesis packet, HLT Impasse or READY, receipt, repair route |
| Ladder mutation/query | Pi/CLI/API `trajectory set-hlt|infer-lower|revise|history|query|resolve` → trajectory route | inference/ledger service → reducer + ledger/snapshot + legacy HLT projection | versioned Ladder/query/diff/conflict packet and receipt |
| Flow | Pi/CLI/API `project flow inspect|steer|sync|advance|handoff|recover` → flow route | Flow service → guard/task/Workpoint/Focus Stack/evidence adapters → atomic event batch | ProjectFlowPacket, next action/one question, receipt |
| Marker guard | every bootstrap/switch/resume/mutation/release preflight → project route | guard service → marker/ledger/snapshot verifier; migration/repair append events | typed guard state, mismatch evidence, safe repair route |
| Task reconciliation | Flow sync/provider event → task handler | reconciliation service → provider/task graph + spec/Ladder links | deterministic accepted/rejected delta and idempotency receipt |
| Spec135 gate | change preflight/CI/release preflight → compatibility handler | impact service → Git/schema/API/event/doc/generated-contract diff adapters | Spec135CompatibilityPacket; UNKNOWN or missing amendment fails |
| Release control | Pi/CLI/API release operation → release route | release service → topology, workflow, artifact, deployment, rollback adapters → receipt ledger | state packet with exact SHA and distinct publish/deploy/verify proof |
| Release intelligence | accepted candidate/update event → intelligence handler | evidence assembler → issue/Bead/test/artifact/deployment/security adapters → renderer | packet/page parity result; unsupported claim or hidden gap fails |

All mutation stacks enforce scope/authority/idempotency before service entry, append durable evidence before projecting state, return typed error/recovery envelopes, regenerate capability docs, and have unit/integration/E2E/rollback evidence linked to their Bead.

## 18. Common contracts, transactions, and boundaries

Every mutation accepts `ProjectMutationRequestV1`:

```text
schema_version, operation, project_root, project_id, continuity_id
expected_project_revision, expected_trajectory_revision
idempotency_key, authority_envelope, confirmation_ref
payload, evidence_refs[], source_surface, requested_at
```

Every result returns `ProjectOperationResultV1`:

```text
status, operation, scope, project_revision, trajectory_revision
flow_state, guard_state, changed_object_refs[], event_refs[]
receipt_ref, evidence_refs[], warnings[], blocked_reason
safe_retry, next_action, rehydrate_refs[]
```

Canonical event kinds are `ProjectGenesisStarted|Resumed|Committed`, `HltProposed|Committed|Superseded`, `MlgInferred|Committed|Superseded`, `StgInferred|Committed|Superseded`, `WaypointInferred|Committed|Advanced|Superseded`, `CurrentStateObserved`, `GapAssessed`, `SpecBound`, `TaskGraphReconciled`, `WorkpointActivated|Completed|HandedOff`, `FocusStackReconciled`, `TrajectoryConflictDetected|Resolved`, `MarkerGuardVerified|Failed|Repaired`, `SchemaMigrated`, `ReleaseStateTransitioned`, and `ReceiptLinked`. Each uses the common ledger envelope from §8.

Mutation transaction order is fixed:

1. canonicalize and verify project scope; reject symlink/root/continuity mismatch;
2. verify authority/confirmation and expected revisions;
3. acquire the existing bounded project write lock; preserve the current 1,500 ms safe-retry posture;
4. validate invariants and build the complete event batch without changing projections;
5. append/checkpoint the durable batch and receipt intent;
6. reduce projections, atomically temp-write/fsync/rename marker changes, then finalize receipt;
7. publish one versioned refresh event and release the lock;
8. on any pre-commit failure, persist no partial projection; on post-append projection failure, enter `INTEGRITY_REPAIR_REQUIRED` and replay from ledger.

Typed error codes: `SCOPE_REQUIRED`, `SCOPE_MISMATCH`, `AUTHORITY_REQUIRED`, `CONFIRMATION_REQUIRED`, `HLT_IMPASSE`, `REVISION_CONFLICT`, `IDEMPOTENCY_CONFLICT`, `WRITE_LOCK_SATURATED`, `INFERENCE_REJECTED`, `PROVIDER_UNAVAILABLE`, `TRAJECTORY_CONFLICTED`, `MIGRATION_REQUIRED`, `INTEGRITY_REPAIR_REQUIRED`, `SPEC135_IMPACT_UNKNOWN`, `ACCEPTANCE_INCOMPLETE`, and `RELEASE_STATE_INVALID`.

Security/resource boundaries:

- HLT commitment/supersession, conflict resolution, takeover, external mutation, release, rollback, and destructive repair require explicit authority; inference is advisory until policy permits commit;
- secrets and unrestricted filesystem/user content never enter events, packets, evidence, prompts, diagnostics, or release pages; refs replace raw sensitive payloads;
- query order is deterministic; defaults remain 50 records and hard-cap at 500; larger bodies use cursor/rehydrate refs;
- LowMem mode suppresses optional enrichment, never guard/authority/evidence fields;
- stale caches may serve labeled reads only and never authorize writes;
- route/CLI/Pi aliases remain through one deprecation cycle and emit the canonical schema/version;
- migration phases are inventory → dual-read → shadow-verify → canonical-write cutover → compatibility-read window → evidence-backed retirement; rollback returns to the last verified reader/writer pair without deleting events.

Threat controls are mandatory: canonical path plus project/continuity/revision binding defeats symlink and cross-project replay; event hash/digest and append-only receipts expose truncation/tampering; idempotency plus expected revisions defeat duplicate/lost updates; typed adapters quarantine task-provider and evidence/prompt injection; field allowlists/redaction prevent secret leakage; signed/checksummed exact-SHA artifacts and independent installed/running probes prevent release substitution; confirmation refs protect HLT, conflict, Canvas, release, rollback, and destructive actions; schema support and downgrade rejection prevent migration confusion.

Feasible local-daemon budgets, measured during W0.3 and enforced after implementation: `trajectory view` p95 ≤750 ms normal/≤1,500 ms LowMem; 50-record history query p95 ≤100 ms; internal mutation excluding external inference/provider work p95 ≤2,000 ms; project write-lock acquisition remains 1,500 ms then safe-retry; external/long operations return an accepted receipt/status handle instead of holding an unbounded request; packets stay within Bloatgaurd budgets using rehydrate refs. W7 reports p50/p95/p99, memory, ledger growth, lock saturation, timeout rate, and regression versus the exact pre-change baseline.

## 19. Test and evidence matrix

Required test families:

- schema/event/reducer/migration/property tests;
- HLT Impasse and no-placeholder static/runtime tests;
- greenfield/brownfield Genesis interruption/replay/idempotency tests;
- inference hierarchy/scoring/adversarial/domain-general evals;
- history/query/as-of/diff/fallback/supersession tests;
- marker guard corruption/conflict/stale/cross-scope tests;
- task-provider outage/reconciliation/dedup/migration tests;
- Workpoint advance/handoff/parallel/conflict tests;
- Pi/Canvas/API/CLI/MCP/headless packet/state/version parity;
- Spec135 impact/unknown/amendment/generated-contract drift gates;
- DXUX, accessibility, security/privacy/secrets, LowMem, timeout, daemon reconnect, compaction/resume;
- release profile/state/immutability/proof-reuse/rollback/deployment-truth tests;
- release-intelligence semantic coverage/unsupported-claim/known-gap/page-packet parity;
- complete locked-scope exact-SHA E2E and live dogfood proof.

Evidence links to Beads, GitHub issues, exact SHA, workflow/run IDs, artifacts, receipts, installed/running versions, and benchmark/friction reports.

## 20. Implementation order

1. **W0:** finish this spec, call stacks, migration/security/feasibility/Spec135 impact review, freeze acceptance.
2. **W1:** core schemas/events/storage/Trajectory/marker/temporal/prediction foundations.
3. **W2:** Genesis/Impasse/inference/task/first Workpoint.
4. **W3:** Flow Kernel/packets/surface refresh/Pi/Canvas/Spec135 amendments.
5. **W4:** Pi UX/contracts/runtime regressions.
6. **W5:** release control/intelligence/incident closure.
7. **W6:** remaining open-at-lock Bead dispositions and legacy gates.
8. **W7:** integrated exact-SHA acceptance, canonical dogfood release, truth/speed/friction decision.

Parallel work is allowed only after W0 and only for non-overlapping call stacks with explicit ownership and shared schema versions.

## 21. Rollback and recovery

- schema migrations are additive/read-compatible before canonical-write cutover;
- every migration emits receipt and preserves old data until verification;
- marker/ledger repair appends corrective events, never rewrites history;
- feature toggles alter projection depth, not authority/storage;
- release rollback restores verified prior runtime while preserving failed candidate evidence;
- interrupted flow resumes from receipt;
- incompatible Spec135 impact blocks promotion before code merge/release.

## 22. Terminal acceptance

No release pass if any Spec142 condition remains open, any Spec135 impact is unknown, any canonical terminology/schema conflicts, any lazy/placeholder Trajectory path survives, any locked item lacks evidence disposition, any surface disagrees, any migration/rollback is unproven, any generic release claim lacks evidence, or source/artifact/installed/running truth differs.

`SHIPPED` requires all locked leaves closed, exact-SHA acceptance green, immutable candidate published, live deployment and smoke proof green, rollback/audit/self-heal/watchdog proven, intelligent release information published, and speed/friction/cost evidence attached.

