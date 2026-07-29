# Spec 149 — Focusa Workset Flow Ledger, Checkpoint Testing, Bounded Preload, and Formal Release Completion

**Status:** DRAFT — OPERATOR REVIEW — NO IMPLEMENTATION OR RELEASE AUTHORITY  
**Project:** Focusa  
**Owner:** Focusa Core, Mission Canvas, Agent Runtime, and Release Engineering  
**Bead:** `focusa-vbcqu.9.8`  
**Call-stack design:** `019faef3-0645-72b2-8524-57dc04debdeb`  
**Depends on:** Specs 100, 104, 109, 111, 116, 119, 130, 131, 135–135K, 140–148  
**Supersedes:** none  
**Implementation baseline:** `locked-release-all-open-spec137-138-140@cf65f34c`  

---

## 1. Executive decision

Focusa SHALL provide a provider-neutral **Workset Flow Ledger** that can represent:

- a few explicitly selected tasks;
- a large finite dependency graph;
- a rolling task set;
- a provider-backed, potentially unbounded task stream;
- typed intermediate test, approval, evidence, and recovery flows;
- explicit next actions after milestones, epochs, or terminal completion; and
- an evidence-gated formal release as a terminal completion flow.

A Workset is not a replacement for a task provider, Workpoint, Trajectory, Context Cognition, Preload, Agent Runtime Constitution, Mission Canvas, or the Canonical Release Cycle. It binds and projects those authorities without merging them.

For the current locked Focusa release Workset, task closure alone is insufficient. The Workset reaches `completed` only after the canonical full-release flow reaches verified completion and its release journal is finalized.

---

## 2. Problem statement

Focusa currently has the necessary pieces but no reusable canonical Workset object:

1. Spec 142 and release-proof artifacts preserve one locked release scope.
2. Beads and the provider-neutral Work Item layer preserve task state and dependencies.
3. Project Genesis can create and materialize task plans.
4. Workpoint carries immediate action authority.
5. Spec 111 Preload carries bounded agent context.
6. Mission Canvas and Work Rail expose generated work surfaces.
7. Spec 140 governs instruction authority, prompt compilation, tool routing, and enforcement.
8. Specs 145–148 govern canonical release execution, proof, deployment, and the release journal.

The missing layer is a durable, provider-neutral ledger that says:

- which work belongs to one coherent Workset;
- how membership was selected and admitted;
- what dependency and checkpoint flows govern it;
- which ready frontier is safe to preload;
- when an epoch or Workset may complete;
- what typed action follows completion; and
- which evidence proves the terminal action actually succeeded.

Without that layer, agents must reconstruct scope from files, Beads, Workpoints, and transcript context. This creates ambiguity between `open`, `in_progress`, `blocked`, and all nonterminal work; encourages token-heavy task dumps; and permits stale or incomplete completion claims.

---

## 3. Normative language

`MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, `SHOULD`, and `MAY` are normative.

A requirement is not complete because a type, route, UI card, or test name exists. Completion requires implementation, positive and negative proof, restart/recovery proof where applicable, cross-surface parity, migration evidence, and stable evidence or Receipt references.

---

## 4. Goals

1. One Workset model for small, finite, rolling, and potentially unbounded work.
2. Immutable history with append-only admission, exclusion, reconciliation, and completion events.
3. Provider-neutral membership over existing `WorkItemRef` and approved task-plan records.
4. Explicit dependency, checkpoint, approval, recovery, and completion flow semantics.
5. Bounded, freshness-aware agent preloads that expose only the useful frontier.
6. Full Mission Canvas, Work Rail, Pi, CLI, API, MCP, and headless parity.
7. Spec 140-governed instruction, operation, permission, and prompt boundaries.
8. Formal release completion through the existing Canonical Release Cycle, not a second release engine.
9. Clear authority separation and fail-closed behavior.
10. Efficient operation under large graphs, provider outages, LowMem, and long-running streams.

---

## 5. Non-goals

Spec 149 does not:

- create another task manager;
- replace Beads, GitHub Issues, or another provider;
- make Workset task text an instruction source;
- make Preload canonical authority;
- promote a Workpoint automatically from arbitrary task ordering;
- redefine HLT, MLG, STG, Waypoint, or Trajectory authority;
- create a second A2UI renderer, Work Rail, Mission Canvas shell, event stream, or Operation Registry;
- amend frozen Specs 135–135K by implication;
- create a second release state machine;
- allow a mathematically infinite task list to be materialized;
- let generated UI silently admit, reorder, close, or release work;
- execute free-form shell text as a checkpoint or completion transition;
- publish, deploy, or release merely because all task nodes appear closed.

---

## 6. Authority model

| Concern | Canonical authority |
| --- | --- |
| Project/workstream scope | `ProjectIdentity` plus `continuity_id` |
| Long-, mid-, and short-term direction | Trajectory |
| Task existence and current provider status | Provider adapter and provider-neutral Work Item layer |
| Approved generated task plan | `ProviderNeutralTaskPlanRecord` |
| Workset membership, admission history, and flow contract | Workset Flow Ledger |
| Immediate executable action | Workpoint |
| Test and acceptance evidence | Evidence records and Spec 119 Receipts |
| Instruction and system-prompt authority | Activated Spec 140 Agent Runtime Constitution |
| Tool/action mutation authority | Operation Registry, permission context, confirmation, and idempotency contract |
| Generated workspace UI | Mission Canvas/A2UI projection only |
| Release execution and truth | Specs 145–147 Canonical Release Cycle |
| Release historical record | Spec 148 Canonical Release Benchmark Journal |
| Agent delivery context | Spec 111 Preload, advisory only |

Authority rules:

1. A Workset SHALL NOT invent provider task status.
2. A task provider SHALL NOT silently mutate locked Workset membership.
3. A Workset SHALL NOT become immediate action authority; it proposes a ready frontier from which a Workpoint is explicitly promoted.
4. A Workset SHALL NOT supersede Trajectory priority without operator steering or canonical Trajectory mutation.
5. A Workset preload SHALL remain advisory and SHALL NOT close, admit, execute, or release work.
6. The Mission Canvas projection SHALL bind the same canonical operations as API, CLI, Pi, and MCP.
7. Operator steering and safety boundaries remain higher authority than automation.

---

## 7. Terminology

### 7.1 Workset

A durable project-scoped flow binding over provider-neutral work items, dependency edges, checkpoint flows, and completion transitions.

### 7.2 Cardinality mode

- `fixed`: membership is explicitly enumerated and expected to become sealed.
- `rolling`: membership may be appended under an admission policy.
- `provider_stream`: items are discovered lazily from a provider query or event cursor.

### 7.3 Admission state

- `draft`: definition is mutable and not executable.
- `active`: admissions may occur under policy.
- `sealing`: reconciliation and final validation are in progress.
- `sealed`: membership revision and graph digest are immutable.

“Locked Workset” means a `sealed` Workset revision. Locked is a state, not a different object type.

### 7.4 Epoch

A bounded, monotonically identified slice of a rolling or provider-stream Workset. Each epoch has a cursor range, membership digest, dependency projection, checkpoint runs, and optional milestone completion flow.

### 7.5 Ready frontier

The bounded set of provider-verified, dependency-ready work items eligible for Workpoint selection under current Workset, Trajectory, policy, and checkpoint state.

### 7.6 Checkpoint flow

A typed intermediate flow that validates, approves, rehearses, settles, or recovers a Workset at a defined trigger.

### 7.7 Completion transition

A typed operation that begins only after a completion contract is satisfied. Examples include starting another Workset, archiving, handoff, or invoking the Canonical Full Release flow.

---

## 8. Canonical object model

```text
WorksetDefinition
├─ WorksetScope
├─ WorksetCardinalityPolicy
├─ WorksetAdmissionPolicy
├─ WorksetProviderBinding[]
├─ WorksetMember[]
│  ├─ WorkItemRef
│  ├─ approved TaskPlan refs
│  ├─ requirement/spec refs
│  └─ status-at-admission provenance
├─ WorksetEdge[]
├─ WorksetEpoch[]
├─ WorksetCheckpointFlow[]
├─ WorksetCompletionContract
├─ WorksetCompletionTransition[]
├─ WorksetConstitutionBinding
├─ WorksetReleaseBinding?
├─ WorksetEvidenceBinding
└─ WorksetLedgerBinding
```

### 8.1 `WorksetDefinition`

```yaml
schema_version: focusa.workset.v1
workset_id:
project_root:
continuity_id:
trajectory_ref:
specification_refs: []
title:
mission:
cardinality_mode: fixed | rolling | provider_stream
admission_state: draft | active | sealing | sealed
lifecycle_state: planned | running | paused | completing | completed | failed | cancelled
revision: 1
membership_digest:
graph_digest:
constitution_ref:
created_at:
created_by:
updated_at:
event_ledger_ref:
provider_bindings: []
admission_policy: {}
members: []
edges: []
epochs: []
checkpoint_flows: []
completion_contract: {}
completion_transitions: []
evidence_refs: []
receipt_refs: []
```

### 8.2 `WorksetProviderBinding`

```yaml
binding_id:
provider: bd | github | linear | jira | imported | none
provider_scope_ref:
provider_revision_ref:
query_semantics: open_only | all_nonterminal | explicit_ids | provider_query | event_stream
query:
cursor:
last_verified_at:
freshness: current | stale | unavailable
capabilities_ref:
```

The query semantics MUST be explicit. `open_only` and `all_nonterminal` are not interchangeable.

### 8.3 `WorksetMember`

```yaml
member_id:
work_item_ref:
provider_binding_ref:
task_plan_ref:
requirement_refs: []
spec_refs: []
mandatory: true
status_at_admission:
provider_revision_at_admission:
admission_event_ref:
admission_reason:
parent_member_ref:
epoch_id:
current_status_projection:
current_status_freshness:
disposition: pending | completed | excluded | superseded | cancelled | blocked
supersedes_member_ref:
evidence_refs: []
receipt_refs: []
```

A Workset stores provider status as a projection. It MUST reconcile rather than claiming ownership of provider status.

### 8.4 `WorksetEdge`

```yaml
edge_id:
from_member_ref:
to_member_ref:
edge_type: blocks | parent_child | evidence_requires | checkpoint_requires | release_requires
source: provider | workset | specification | operator
source_ref:
created_event_ref:
```

Within a sealed finite Workset or one epoch, blocking dependencies MUST form a DAG. Cross-epoch dependencies may point only to the same or an earlier epoch. Recurring behavior is represented by repeated checkpoint runs or new epochs, not dependency cycles.

---

## 9. Cardinality and streaming semantics

### 9.1 Small finite Worksets

A Workset MAY contain one task. The object model and API SHALL not require artificial epics, tranches, or milestones.

### 9.2 Large finite Worksets

Large graphs SHALL use cursor pagination, compact summaries, dependency indices, and bounded projections. Full graph loading requires an explicit cold-path request.

### 9.3 Rolling Worksets

A rolling Workset MAY append members while `active`. Each admission SHALL:

1. resolve and validate the provider reference;
2. capture provider revision and status;
3. evaluate duplicates and supersession;
4. validate dependency references;
5. calculate Spec 135 and Spec 140 impact;
6. produce a preview;
7. require authority appropriate to the admission policy; and
8. append an immutable admission event.

### 9.4 Provider streams

Potentially unbounded Worksets SHALL be lazy. They store:

- provider query identity;
- monotonic cursor or provider event token;
- current bounded horizon;
- epoch membership digest;
- deduplication keys;
- replay checkpoint;
- lag and freshness; and
- rehydrate references.

They MUST NOT claim “all tasks loaded.”

### 9.5 Sealing an open-ended Workset

An open-ended Workset has no terminal end until one of these occurs:

- the operator seals it;
- a pre-authorized, evidence-backed seal condition is satisfied;
- the provider reports a terminal bounded stream and the condition is independently verified; or
- the Workset is cancelled.

A provider-stream Workset MAY execute release or other milestone flows after sealed epochs while the parent Workset remains active.

### 9.6 Reopening

A sealed Workset cannot be edited in place. Reopening appends `workset.reopened`, creates a new revision, invalidates any candidate scope digest and dependent release proof, and requires re-sealing.

---

## 10. Admission policy

```yaml
WorksetAdmissionPolicy:
  mode: operator_only | rule_gated | provider_query | approved_task_plan
  allowed_providers: []
  allowed_query_semantics: []
  required_spec_refs: []
  require_done_condition: true
  require_dependency_resolution: true
  require_impact_assessment: true
  post_seal_admission: deny
  duplicate_policy: reject | link_existing
  unknown_impact_policy: block
  max_batch_size:
  confirmation_required: true
```

Rules:

1. Imported provider text is data, not instruction authority.
2. Unknown Spec 135 or Spec 140 impact blocks admission.
3. Duplicate provider identities fail closed unless explicitly linked to the existing member.
4. Missing done conditions or unresolved dependency refs fail closed for mandatory members.
5. Bulk admissions require bounded preview and aggregate plus per-item reason codes.
6. Admissions SHALL be idempotent.

---

## 11. Task-plan and Work Item integration

The implementation SHALL reuse:

- `WorkItemRef`, `WorkItem`, `WorkItemQuery`, provider adapters, closure claims, and provider capabilities from `crates/focusa-core/src/work_item/`;
- `evaluate_readiness` and `select_next_ready` from the provider-neutral scheduler;
- `ProviderNeutralTaskRecord`, `ProviderNeutralTaskPlanRecord`, and `TaskMaterializationRecord` from current Spec 135 core types; and
- existing Mission Canvas task-plan draft, preview, approval, revision, and materialization operations.

Required flow:

```text
CRIST/spec workbench
→ provider-neutral task-plan draft
→ operator preview/edit/approval
→ governed provider materialization
→ WorkItemRef verification
→ Workset admission preview
→ Workset admission event
→ dependency and ready-frontier projection
→ explicit Workpoint promotion
```

Spec 149 MUST NOT create a second task-plan schema or bypass provider closure authority.

---

## 12. Workset lifecycle

```text
DRAFT
→ ACTIVE
→ SEALING
→ SEALED
→ RUNNING
→ COMPLETING
→ COMPLETED
```

Additional states:

```text
PAUSED
BLOCKED
FAILED
CANCELLING
CANCELLED
REOPENED
```

### 12.1 Transition laws

- `draft → active`: definition validated and authority confirmed.
- `active → sealing`: admission is paused and full reconciliation begins.
- `sealing → sealed`: provider snapshots, membership, dependencies, impacts, and digests verify.
- `sealed → running`: a Workpoint is bound or explicit execution authority exists.
- `running → completing`: all mandatory members have valid dispositions and every blocking checkpoint passed.
- `completing → completed`: all required completion transitions produced verified terminal Receipts.
- Any stale provider, unknown impact, missing evidence, or authority mismatch blocks consequential transition.

### 12.2 Completion is not task closure

`completed` requires both:

1. the Workset completion contract; and
2. all mandatory terminal transitions.

A green build, closed issue, tag, published asset, local binary, or agent statement is not Workset completion.

---

## 13. Checkpoint and intermediate test flows

### 13.1 `WorksetCheckpointFlow`

```yaml
schema_version: focusa.workset_checkpoint_flow.v1
flow_id:
workset_ref:
trigger:
  kind: after_members | before_member | epoch_seal | milestone | risk_change | manual | periodic
  member_refs: []
  epoch_ref:
mode: blocking | advisory
operation_ref:
constitution_ref:
input_template_ref:
validation_matrix_ref:
acceptance_criteria: []
required_evidence_kinds: []
required_receipt_types: []
timeout_ms:
retry_policy:
rollback_ref:
on_pass: []
on_fail: []
on_indeterminate: []
```

### 13.2 Flow requirements

1. `operation_ref` MUST resolve through the canonical Operation Registry.
2. Inputs MUST use typed schemas; free-form shell commands are forbidden.
3. Mutation, external effect, or release-related flows require explicit authority and idempotency.
4. Blocking flows prevent dependent frontier promotion until passed.
5. Advisory flows remain visible but cannot masquerade as blocking acceptance.
6. Test results require stable evidence and Receipt refs.
7. Retry policies require bounded attempts, cooldown, fingerprinting, and rollback where applicable.
8. An indeterminate result fails closed for mandatory acceptance.
9. Re-running a passed flow against changed inputs creates a new run; it does not rewrite prior proof.

### 13.3 Examples

- targeted unit test after one member;
- schema and generated-client drift check after a contract tranche;
- Mission Canvas parity check after UI-affecting members;
- Spec 140 prompt and enforcement evaluation after instruction-affecting members;
- security and secrets scan before epoch sealing;
- release rehearsal before terminal full release;
- prediction settlement and metacognitive capture after a failed checkpoint.

---

## 14. Completion transitions

### 14.1 `WorksetCompletionContract`

```yaml
schema_version: focusa.workset_completion_contract.v1
require_sealed_revision: true
require_all_mandatory_dispositions: true
allowed_terminal_dispositions: [completed, excluded, superseded, cancelled]
require_zero_dependency_cycles: true
require_zero_unresolved_blockers: true
require_zero_unknown_impacts: true
required_checkpoint_refs: []
required_evidence_refs: []
required_receipt_types: []
provider_freshness_max_age_ms:
```

### 14.2 `WorksetCompletionTransition`

```yaml
schema_version: focusa.workset_completion_transition.v1
transition_id:
trigger: epoch_complete | workset_complete
operation_ref:
mode: blocking | advisory
input_binding_ref:
authority_requirement:
confirmation_required:
required_constitution_ref:
required_receipt_types: []
on_success:
on_failure:
rollback_ref:
```

Completion transitions MAY:

- archive and summarize a Workset;
- create or propose a successor Workset;
- hand off to another verified scope;
- run a release or deployment flow;
- settle predictions and capture reusable learning; or
- produce an operator review packet.

They MUST NOT contain arbitrary executable text.

---

## 15. Canonical full-release terminal flow

Spec 149 SHALL invoke, not duplicate, the Canonical Release Cycle defined by Specs 145–147 and journaled by Spec 148.

### 15.1 Binding

```yaml
WorksetReleaseBinding:
  profile_ref: focusa.release.full.v1
  workset_ref:
  sealed_revision:
  membership_digest:
  graph_digest:
  exact_sha:
  topology_ref:
  intended_surfaces: []
  release_cycle_ref:
  release_journal_ref:
  required_receipt_refs: []
```

The Workset digests become inputs to candidate scope freeze. A changed Workset revision invalidates the release binding.

### 15.2 Required terminal sequence

```text
Workset completion preflight
→ locked-scope and requirement disposition audit
→ exact-SHA resolution
→ canonical release-cycle scope lock
→ topology and surface preflight
→ immutable candidate freeze
→ build
→ complete test and proof DAG
→ candidate acceptance
→ publication
→ deployment
→ installed/running surface verification
→ rollback/audit/self-heal/watchdog verification
→ release-journal finalization
→ Workset terminal Receipt
→ workset.completed
```

### 15.3 Current Focusa Workset profile

The current locked Focusa Workset SHALL use this formal full-release terminal transition. It SHALL NOT complete until:

- every locked issue, Bead, spec requirement, addendum requirement, and accepted closure item has an evidence-backed disposition;
- dependency-cycle and mapping audits pass;
- all required targeted and full gates pass;
- Spec 135 impact is known and all required amendments/generated-contract updates are complete;
- Spec 140 conformance, enforcement, delivery, and cross-harness parity are verified;
- the exact candidate SHA is accepted;
- every intended artifact and surface is published and deployed;
- CLI, daemon, TUI, Pi extension, installer, menubar, and agent-context surfaces prove intended parity where applicable;
- rollback, audit, self-heal, and watchdog paths are proven;
- the Spec 148 journal is finalized; and
- release predictions, problems, outcomes, and reusable metacognitive learning are settled.

Release execution still requires the canonical release authority and any required operator confirmation. Workset completion does not grant publication authority by itself.

---

## 16. Workpoint, Trajectory, Focus Stack, and Project Flow

### 16.1 Trajectory

Trajectory prioritizes the Workset and explains why it matters. Workset status updates may inform current state and gap, but SHALL NOT synthesize or supersede HLT.

### 16.2 Workpoint

A Workpoint binds one immediate member, checkpoint, or completion transition. Workset readiness is advisory until a Workpoint is explicitly checkpointed/promoted under current authority.

### 16.3 Focus Stack

Focus Stack may project:

```text
Project → HLT → MLG → STG → Waypoint → Workset → Epoch → Member/Flow → Workpoint
```

This projection does not make Focus Stack canonical membership authority.

### 16.4 ProjectFlowPacket

Spec 143 `ProjectFlowPacket` SHALL add:

- active `workset_id`, revision, digests, and freshness;
- admission and lifecycle states;
- active epoch and cursor;
- bounded ready frontier;
- blocked and stale summaries;
- upcoming checkpoint flows;
- completion contract and next terminal transition;
- provider reconciliation posture;
- evidence, Receipt, and rehydrate refs.

All surfaces SHALL consume the same versioned projection.

---

## 17. Bounded preload

### 17.1 Principle

Preload SHALL include the useful frontier, not the complete Workset.

### 17.2 `WorksetPreloadSlice`

```yaml
schema_version: focusa.workset_preload_slice.v1
workset_id:
workset_revision:
membership_digest:
graph_digest:
freshness:
admission_state:
lifecycle_state:
active_epoch_ref:
cardinality_mode:
loaded_member_count:
known_member_count:
known_count_complete:
ready_frontier: []
active_workpoint_ref:
blocked_summary: []
upcoming_checkpoint_flows: []
completion_contract_summary:
next_completion_transition:
omitted_counts: {}
rehydrate_refs: []
evidence_refs: []
proof_gaps: []
```

### 17.3 Selection laws

1. `ready_frontier` SHALL be bounded by profile and token budget.
2. The default profile SHALL include no more than the active Workpoint plus the highest-value ready and blocking summaries.
3. Full descriptions and cold graph neighborhoods require explicit rehydration.
4. Open-ended Worksets SHALL declare `known_count_complete=false`.
5. Stale provider state SHALL be visible and SHALL block mutation authority.
6. Omitted items SHALL be counted and recoverable by cursor or ref.
7. Workset context SHALL participate in Context Cognition curation rather than bypassing it.
8. Spec 111 Preload remains advisory and cannot execute completion transitions.

### 17.4 Profiles

Existing Spec 111 profiles SHALL be extended rather than replaced:

- `rules_only`: no Workset task payload;
- `rules_and_context`: Workset identity, Workpoint, next checkpoint, and tiny ready frontier;
- `budget_light`: identity, active member, blockers, and rehydrate refs;
- `budget_deep`: bounded neighborhood, checkpoint details, completion contract, and evidence summaries.

---

## 18. Spec 135–135K compatibility and Mission Canvas UI

The Spec 135 series is frozen at 135K. Spec 149 SHALL NOT add Spec 135L or silently change frozen semantics. Implementation MUST follow Spec 135D/135E impact, amendment, generated-contract, migration, and proof rules.

### 18.1 Impact matrix

| Spec | Impact | Required handling |
| --- | --- | --- |
| 135 master | direct | Bind Workset to professional workspace, CRIST, authority, and closure contracts. |
| 135A | direct | Extend Mission Canvas and Work Rail through existing projection and vertical UX patterns. |
| 135B | direct | Reuse CRIST task planning, approval, provider materialization, and Project Genesis. |
| 135C | indirect/direct | Use existing UIAI artifact and live-refresh boundaries; preserve browser/session identity. |
| 135D | direct | Add implementation DAG, performance budgets, and zero-deferral proof. |
| 135E | direct | Record exact amendments, migration, compatibility, and closure matrix. |
| 135F | indirect | Link Worksets, members, requirements, flows, and evidence into ontology without making ontology task authority. |
| 135G | direct | Preserve multiplexed Work Surface, session, attachment, and restoration isolation. |
| 135H | indirect | Allow interview outcomes to propose task plans/admissions; never admit silently. |
| 135I | direct | Generate Workset UI through the approved A2UI path and nontechnical interaction contract. |
| 135J | direct | Bind all actions to the Core API Operation Registry and durable replayable stream. |
| 135K | direct | Apply UXP/UFI, adaptive UI, friction learning, and nontechnical usability proof without authority drift. |

`unknown` impact blocks implementation promotion and release.

### 18.2 Reuse laws

Implementation SHALL reuse:

- the existing Mission Canvas shell and `MissionCanvasView`;
- current Work Rail and Pi Mission Canvas projections;
- `packages/a2ui-renderer` and its Focusa catalog;
- generated Spec 135 TypeScript/OpenAPI contracts;
- current durable event replay/live-tail mechanism;
- current workspace invalidation and restoration semantics;
- current provider-neutral task-plan surfaces; and
- the canonical Operation Registry/provider execution route.

A second renderer, event stream, operation registry, task-plan workbench, or hand-coded authority path is forbidden.

### 18.3 Mission Canvas Workset surface

The generated surface SHALL provide:

1. Workset identity, mission, mode, state, revision, freshness, and digests.
2. A scalable Work Rail/flow view with virtualized finite and epoch-windowed streaming modes.
3. Ready frontier and active Workpoint cards.
4. Provider synchronization, cursor, drift, and outage posture.
5. Admission preview with explicit inclusion semantics and reason codes.
6. Dependency and critical-path neighborhood views.
7. Intermediate test/checkpoint timeline with evidence and retry posture.
8. Completion-contract inspector.
9. Formal release terminal panel with canonical release-cycle and journal state.
10. Evidence, Receipt, prediction, and metacognitive settlement links.
11. Accessible keyboard, screen-reader, reduced-motion, narrow-width, and degraded-mode behavior.
12. Exact replay/restoration from durable server state.

### 18.4 Adaptive UI boundary

UXP/UFI and friction learning MAY change presentation, progressive disclosure, or recommended navigation. They MUST NOT silently:

- admit or exclude members;
- alter dependency edges;
- mark a checkpoint passed;
- change mandatory status;
- seal or reopen a Workset;
- promote a Workpoint;
- execute a completion transition; or
- grant release authority.

Every adaptive suggestion SHALL explain why and bind a canonical operation preview.

### 18.5 Work Surface and browser isolation

Workset artifacts and browser context SHALL remain scoped to the owning Work Surface, session, attachment, origin, project root, continuity ID, and Workset ID. Cross-surface references require explicit governed linking; no ambient browser or attachment context may leak into another Workset.

---

## 19. Spec 140 Agent Runtime Constitution integration

Spec 140 is a primary dependency, not a side integration.

### 19.1 Work item text is data

Provider titles, descriptions, comments, attachments, test logs, and imported plans MUST be treated as untrusted or provenance-scoped data. They SHALL NOT become system instructions merely because they appear in a Workset.

Instruction-like provider content SHALL pass Spec 140 source classification and prompt-injection handling. Quarantined content remains inspectable but not executable.

### 19.2 Prompt architecture

Workset context belongs in the turn-dynamic operational layer, not the stable constitutional system prompt. A Workset slice SHALL carry:

- provenance;
- freshness;
- authority labels;
- scope and revision;
- token budget;
- omitted and rehydrate refs; and
- a distinct context integrity digest.

Changing Workset membership SHALL NOT silently regenerate or activate an Agent Runtime Constitution.

### 19.3 Constitution binding

Each consequential Workset operation SHALL evaluate against the active constitution and target capability profile. The Workset records `constitution_ref` and any `ContractImpactAssessment` required by Spec 140.

### 19.4 Operation and enforcement

Admission, edge mutation, sealing, checkpoint execution, Workpoint promotion, completion execution, release invocation, cancellation, and reopening SHALL:

1. resolve a typed Operation Registry entry;
2. verify permission and mutation authority;
3. validate input schema and scope;
4. enforce confirmation requirements;
5. apply the active `EnforcementPlan` and `ValidationMatrix`;
6. use idempotency keys;
7. emit Evidence and Receipts; and
8. fail closed when capability or policy is missing.

### 19.5 Cross-harness parity

Pi, CLI, MCP, OpenAI-functions, REST, Mission Canvas, menubar, and any future supported harness SHALL preserve identical Workset semantics, authority, failure classes, and confirmation behavior. Unsupported target capabilities must degrade visibly; no harness may invent a local completion path.

### 19.6 Agent Runtime Studio

Runtime Studio SHALL show:

- the constitution bound to the Workset;
- Workset-related instruction sources and quarantines;
- prompt/context composition boundaries;
- operation and enforcement plans;
- target parity and delivery posture;
- Workset-triggered contract impact; and
- drift between intended and effective behavior.

Studio remains an inspection and governed-operation surface, not parallel authority.

### 19.7 Security

Spec 140 secret, prompt-integrity, instruction-injection, path, tool-routing, and delivery constraints apply to every Workset artifact, preload, UI surface, and flow.

---

## 20. Current implementation reality audit

As of `cf65f34c`:

| Surface | Current state | Spec 149 consequence |
| --- | --- | --- |
| Provider-neutral Work Item model | present in `crates/focusa-core/src/work_item/` | reuse and extend by reference only |
| Dependency readiness scheduler | present | reuse for frontier; add Workset/epoch/checkpoint filters |
| Closure authority | present | Workset disposition depends on provider-valid closure/evidence |
| Provider-neutral task plans | present in core Spec 135 types | reuse approved plan and materialization refs |
| Mission Canvas task-plan routes | present | add admission preview after materialization; no duplicate workbench |
| Project Genesis | present with first-task/Workpoint path | allow optional Workset creation/binding without changing HLT rules |
| Workpoint/Trajectory | present | preserve immediate-action and goal authority |
| Spec 111 Preload | present but Workpoint-centric | extend with bounded Workset slice |
| Context Cognition curator | present | score Workset candidates under token budget |
| Mission Canvas/A2UI/Work Rail | present | add generated Workset projection and operations |
| Durable stream/Operation Registry | present under Spec 135J surfaces | register Workset events and actions there |
| Spec 140 core/API/CLI/Pi/Studio | substantially present; ledger still has remaining closure rows | integrate only through verified contracts; unknown gaps block release |
| Canonical release core and CLI | present | invoke existing release services |
| Release REST surface | currently proof-status only | add governed cycle binding/inspection through Specs 145–147 implementation, not ad hoc release commands |
| Spec 148 release journal | specified and current release authority | bind terminal flow and require finalization |
| Canonical Workset object/store/tools/UI | absent | primary implementation gap |

No implementation may claim greenfield freedom. Existing contracts and generated clients are migration inputs.

---

## 21. Persistence, projections, and events

### 21.1 Required stores

```text
worksets
workset_provider_bindings
workset_members
workset_edges
workset_epochs
workset_checkpoint_flows
workset_checkpoint_runs
workset_completion_contracts
workset_completion_transitions
workset_release_bindings
workset_snapshots
workset_idempotency
```

Canonical history SHALL be append-only reducer events. SQL tables and UI packets are projections.

### 21.2 Minimum events

```text
workset.created
workset.definition_revised
workset.activated
workset.provider_bound
workset.provider_reconciled
workset.provider_stale
workset.member_admission_previewed
workset.member_admitted
workset.member_excluded
workset.member_superseded
workset.edge_added
workset.edge_removed
workset.epoch_opened
workset.epoch_sealing
workset.epoch_sealed
workset.sealing_started
workset.sealed
workset.reopened
workset.checkpoint_defined
workset.checkpoint_started
workset.checkpoint_passed
workset.checkpoint_failed
workset.checkpoint_indeterminate
workset.workpoint_proposed
workset.completion_preflighted
workset.completion_started
workset.transition_started
workset.transition_succeeded
workset.transition_failed
workset.release_bound
workset.completed
workset.paused
workset.failed
workset.cancelled
workset.repaired
```

### 21.3 Event laws

- Stable event names and schemas.
- Deterministic identity and idempotency.
- Optimistic revision checks.
- Hash/digest linkage for sealed membership and graph state.
- Corrections append superseding events; history is never rewritten.
- Consequential events link Evidence and Spec 119 Receipts.
- Replay after restart yields the same Workset projection.
- CRDT/distributed merge, where used, SHALL preserve typed scope and reject conflicting sealed revisions.

### 21.4 Projection freshness

Every projection SHALL carry:

```text
project_root
continuity_id
workset_id
workset_revision
event_version
provider_revision
computed_at
freshness
stale_reason
rehydrate_refs
```

---

## 22. API contract

### 22.1 Definition and view

```text
POST /v1/worksets/preview
POST /v1/worksets
GET  /v1/worksets/:workset_id
GET  /v1/worksets/:workset_id/projection
GET  /v1/worksets/:workset_id/members?cursor=&limit=
GET  /v1/worksets/:workset_id/graph?anchor=&depth=&cursor=
GET  /v1/worksets/:workset_id/events?after=&limit=
```

### 22.2 Admission and provider reconciliation

```text
POST /v1/worksets/:workset_id/admissions/preview
POST /v1/worksets/:workset_id/admissions/commit
POST /v1/worksets/:workset_id/reconcile
POST /v1/worksets/:workset_id/epochs/open
POST /v1/worksets/:workset_id/epochs/:epoch_id/seal/preview
POST /v1/worksets/:workset_id/epochs/:epoch_id/seal/commit
```

### 22.3 Scope sealing

```text
POST /v1/worksets/:workset_id/seal/preview
POST /v1/worksets/:workset_id/seal/commit
POST /v1/worksets/:workset_id/reopen/preview
POST /v1/worksets/:workset_id/reopen/commit
```

### 22.4 Checkpoints and completion

```text
POST /v1/worksets/:workset_id/checkpoints/:flow_id/preview
POST /v1/worksets/:workset_id/checkpoints/:flow_id/run
POST /v1/worksets/:workset_id/completion/preview
POST /v1/worksets/:workset_id/completion/execute
GET  /v1/worksets/:workset_id/completion/status
POST /v1/worksets/:workset_id/cancel/preview
POST /v1/worksets/:workset_id/cancel/commit
```

### 22.5 Agent and UI projection

```text
GET  /v1/worksets/:workset_id/preload
GET  /v1/worksets/:workset_id/a2ui
GET  /v1/worksets/:workset_id/stream?after=
POST /v1/worksets/:workset_id/workpoint/propose
```

### 22.6 API laws

- Reads are bounded and cursor-based.
- Mutations require typed scope, expected revision, authority, and idempotency key.
- Preview never mutates.
- Commit responses include event refs, receipts, effective revision, and next tools.
- Scope mismatch fails closed.
- Provider outage returns stale/degraded posture, never fabricated readiness.
- Unknown fields are rejected for consequential mutations.

---

## 23. Operation Registry, Pi tools, CLI, and MCP

### 23.1 Canonical operation families

```text
workset.read
workset.define.preview
workset.define.commit
workset.admit.preview
workset.admit.commit
workset.reconcile
workset.seal.preview
workset.seal.commit
workset.checkpoint.preview
workset.checkpoint.run
workset.workpoint.propose
workset.completion.preview
workset.completion.execute
workset.reopen.preview
workset.reopen.commit
workset.cancel.preview
workset.cancel.commit
```

Each operation declares read/write class, schema, permissions, confirmation, idempotency, evidence, Receipt, retry, and recovery behavior.

### 23.2 Pi tools

Minimum progressively discoverable tools:

```text
focusa_workset_preview
focusa_workset_create
focusa_workset_view
focusa_workset_admit
focusa_workset_reconcile
focusa_workset_seal
focusa_workset_checkpoint_flow
focusa_workset_completion
focusa_workset_preload
focusa_workset_doctor
```

Mutating tools use `action=preview|commit` or another explicit two-phase contract where appropriate. A boolean named `approved` alone is insufficient authority.

### 23.3 CLI

```text
focusa workset preview
focusa workset create
focusa workset view
focusa workset members
focusa workset graph
focusa workset admit preview|commit
focusa workset reconcile
focusa workset epoch open|seal
focusa workset seal preview|commit
focusa workset checkpoint preview|run
focusa workset complete preview|execute|status
focusa workset preload
focusa workset doctor
```

CLI output SHALL support bounded human text and stable JSON.

### 23.4 Cross-harness descriptors

Pi, MCP, OpenAI-functions, CLI, REST, generated docs, Agent Card, capability graph, skills, and runbooks SHALL be generated from one canonical capability descriptor and pass parity gates.

---

## 24. Generated UI contract

### 24.1 A2UI components

The existing catalog SHALL gain bounded Workset components such as:

```text
focusa-workset-header
focusa-workset-mode-badge
focusa-workset-frontier
focusa-workset-member-card
focusa-workset-epoch-strip
focusa-workset-checkpoint-timeline
focusa-workset-provider-status
focusa-workset-admission-preview
focusa-workset-completion-contract
focusa-workset-release-progress
focusa-workset-evidence-summary
```

Names are provisional until the Spec 135 generated-contract amendment is approved. Semantics are normative.

### 24.2 UI behavior

- Virtualize large member collections.
- Window provider streams by epoch and cursor.
- Never rely on color alone.
- Surface stale, blocked, awaiting approval, and degraded states.
- Keep dangerous operations behind preview and explicit confirmation.
- Display why a member is ready or blocked.
- Display exact checkpoint evidence and release truth.
- Preserve state across daemon reconnect and application restart.
- Render useful narrow-width Work Rail summaries without hiding authority or failure posture.

### 24.3 Live refresh

Durable event replay plus live tail is canonical. WebSocket/SSE transport choice follows Spec 135J current implementation. UI invalidations SHALL carry refs and versions, not duplicate semantic authority.

---

## 25. Security and privacy

1. Provider task text, comments, attachments, browser content, and logs are untrusted data.
2. Secrets and secret-like values are excluded, redacted, or represented by approved secret refs.
3. Workset membership MUST be scope-bound to project root plus continuity ID.
4. Cross-project work item refs require explicit transfer/admission authority.
5. Symlink, path traversal, oversized payload, and unsafe file-write rules follow Spec 140.
6. External mutations require permission, confirmation, idempotency, and evidence.
7. Release transitions require release-specific authority; Workset ownership is insufficient.
8. UI annotations and provider-supplied safety claims are untrusted.
9. Preload packets contain bounded summaries, not raw secret-bearing logs.
10. Audit views SHALL avoid exposing sensitive provider metadata beyond operator-authorized scope.
11. Provider tokens and credentials never enter Workset events or Receipts.
12. Prompt injection in task content is quarantined and cannot alter flow semantics.

---

## 26. Concurrency, leases, and idempotency

- All mutations carry `workset_id`, expected revision, writer identity, and idempotency key.
- Sealing and completion use a scoped lease or equivalent single-writer guard.
- Provider reconciliation is safe to retry and deduplicates provider revisions.
- Two admissions of the same provider identity converge to one member or produce a conflict.
- A checkpoint run identity binds Workset revision, flow revision, inputs, environment, and attempt.
- A completion transition cannot run twice for the same effective transition identity.
- Lease loss yields a blocked/pending state, never completion.
- Operator steering can pause or supersede automation through an auditable event.

---

## 27. Resource and token budgets

### 27.1 Hot paths

Hot reads:

- Workset header/status;
- active Workpoint;
- ready frontier;
- blocker summary;
- next checkpoint;
- terminal transition summary.

They SHALL avoid full provider scans and full graph materialization.

### 27.2 Cold paths

Cold operations:

- complete graph export;
- full historical replay;
- cross-epoch analytics;
- complete evidence expansion;
- provider-wide reconciliation.

They require explicit opt-in, pagination, progress posture, and resource budgets.

### 27.3 LowMem

LowMem mode SHALL reduce frontier size, disable nonessential graph expansion, retain authority/freshness fields, and preserve all mutation gates.

### 27.4 Bloatgaurd

No Workset packet may silently exceed prompt/output budgets. Omitted counts, cursor, and rehydrate refs are mandatory when truncation occurs.

---

## 28. Failure, recovery, and rollback

| Failure | Required posture |
| --- | --- |
| Provider unavailable | Preserve last verified graph, mark stale, queue reconciliation, block consequential stale actions. |
| Dependency cycle | Block affected epoch/seal; provide bounded cycle evidence. |
| Duplicate member | Reject or link under explicit duplicate policy. |
| Unknown Spec 135/140 impact | Block admission or promotion. |
| Missing Operation Registry entry | Fail closed; do not execute local fallback. |
| Checkpoint timeout | Record indeterminate/failed run; apply bounded retry policy. |
| UI stream loss | Replay from durable cursor; do not infer completion. |
| Daemon restart | Rebuild projection deterministically from events. |
| Lease loss | Stop mutation and expose writer recovery. |
| Workset reopened after candidate freeze | Invalidate candidate binding and release proof. |
| Release failure | Preserve evidence/journal, invoke canonical recovery/rollback, keep Workset incomplete. |
| Receipt mismatch | Block completion and run receipt diagnostics. |
| Prompt injection | Quarantine source and prevent instruction elevation. |

Repair appends corrective events. It never rewrites canonical history.

---

## 29. Migration

### Phase 0 — Inventory

- Inventory release-specific scope manifests, decomposition proofs, Bead DAGs, task plans, Workpoints, ProjectFlow projections, and release journals.
- Classify canonical source, projection, duplicate, stale, or quarantine.
- Produce no hidden behavior change.

### Phase 1 — Schemas and event ledger

- Add Workset schemas, events, reducers, SQLite migrations, snapshots, and property tests.
- Keep existing providers and Workpoint behavior unchanged.

### Phase 2 — Provider and task-plan binding

- Bind approved task plans and existing WorkItem adapters.
- Import existing locked scope through preview plus operator-confirmed commit.
- Preserve source refs and digests.

### Phase 3 — ProjectFlow, Context Cognition, and Preload

- Add bounded Workset projections.
- Prove token budgets, stale-state handling, and rehydrate behavior.

### Phase 4 — Mission Canvas and Work Rail

- Amend only affected 135-series contracts through Spec 135E rules.
- Regenerate A2UI/OpenAPI/TypeScript artifacts.
- Add accessible generated UI and durable refresh.

### Phase 5 — Spec 140 enforcement

- Register operations, constitution bindings, instruction-data boundaries, prompt/context separation, validation, and target parity.

### Phase 6 — Checkpoint flows

- Add typed intermediate flows, retries, evidence, Receipts, and recovery.

### Phase 7 — Canonical release binding

- Bind sealed Workset revision to Specs 145–148 release cycle and journal.
- Rehearse failure, rollback, restart, and exact-SHA invalidation.

### Phase 8 — Current locked Workset migration

- Preview import from current scope and decomposition artifacts.
- Compare every locked issue, requirement, dependency, and disposition.
- Require zero unmapped mandatory refs and zero cycles.
- Commit only with operator confirmation and stable migration Receipt.

---

## 30. Required tests

### 30.1 Schema and reducer

- round-trip every Workset object;
- reject unknown mutation fields;
- event replay determinism;
- idempotent duplicate event handling;
- snapshot/event parity;
- append-only repair and supersession;
- migration upgrade/downgrade compatibility.

### 30.2 Cardinality

- one-member Workset;
- large finite graph;
- rolling admission;
- provider stream with pagination;
- unknown total count;
- epoch seal and continuation;
- manual seal of an open-ended Workset;
- no materialized-infinite-list path.

### 30.3 Dependencies and readiness

- provider dependency import;
- Workset-added checkpoint dependencies;
- cycle rejection;
- cross-epoch backward-only edges;
- readiness under stale provider;
- mandatory versus optional disposition;
- critical-path and frontier bounds.

### 30.4 Checkpoint flows

- blocking/advisory distinction;
- operation lookup;
- input-schema rejection;
- permission and confirmation;
- pass/fail/indeterminate;
- bounded retry/cooldown;
- rollback;
- changed-input rerun identity;
- evidence and Receipt enforcement.

### 30.5 Completion and release

- task closure without terminal flow remains incomplete;
- unsealed Workset cannot complete;
- unresolved blocker prevents completion;
- reopened revision invalidates candidate;
- exact-SHA binding;
- build is not publication;
- publication is not deployment;
- deployment is not live verification;
- failed release preserves evidence and keeps Workset incomplete;
- rollback proof;
- release-journal finalization;
- terminal Workset Receipt.

### 30.6 Spec 135 UI

- generated contract and client parity;
- A2UI renderer reuse;
- Mission Canvas and Pi Work Rail parity;
- virtualized large Workset;
- epoch-windowed stream;
- keyboard and screen-reader flow;
- narrow-width and reduced-motion;
- reconnect/replay/restoration;
- attachment/session/origin isolation;
- adaptive UI cannot mutate authority;
- UXP/UFI friction evidence.

### 30.7 Spec 140

- task text cannot become system instruction;
- prompt injection quarantine;
- Workset slice appears only in dynamic context layer;
- context integrity digest and provenance;
- missing operation route fails closed;
- Constitution/EnforcementPlan/ValidationMatrix enforcement;
- permissions and mutation confirmation;
- target capability degradation;
- API/CLI/Pi/MCP/Mission Canvas parity;
- no silent artifact regeneration.

### 30.8 Reliability and performance

- restart during admission, seal, checkpoint, and completion;
- concurrent admissions and expected-revision conflicts;
- lease expiry;
- provider outage and recovery;
- LowMem bounded projection;
- hot-path latency budget;
- token budget and truncation proof;
- long-running stream replay;
- no event or UI subscription leaks.

---

## 31. Observability and learning

Metrics SHALL include:

```text
workset_count_by_mode
workset_member_count
workset_frontier_size
provider_reconcile_duration
provider_lag
admission_preview_rejection_rate
dependency_cycle_count
checkpoint_duration_and_outcome
checkpoint_retry_count
completion_preflight_failures
time_from_seal_to_completion
release_transition_duration
preload_tokens_and_omissions
ui_replay_lag
workset_reopen_count
```

Every failed checkpoint or completion transition SHOULD:

- settle relevant predictions;
- capture evidence-backed metacognitive learning;
- identify reusable failure fingerprints;
- avoid duplicate problem spam; and
- preserve bounded next-action guidance.

Learning may recommend changes but cannot self-admit tasks, alter authority, or self-activate prompts.

---

## 32. Implementation order

1. Freeze Spec 149 acceptance, terminology, and cross-spec impact.
2. Produce detailed call-stack designs for ledger, provider binding, preload, UI, checkpoint, and release paths.
3. Decompose every normative requirement into provider tasks and dependency edges.
4. Implement schemas, events, reducers, storage, and replay.
5. Implement provider/task-plan binding and readiness projection.
6. Implement API, capability descriptors, Pi tools, CLI, MCP, and docs.
7. Implement ProjectFlow/Context Cognition/Preload projections.
8. Implement Spec 135 amendments, generated contracts, Mission Canvas, Work Rail, and UI tests.
9. Implement Spec 140 operation/enforcement/prompt boundaries and cross-harness proof.
10. Implement checkpoint flows and recovery.
11. Bind and rehearse Specs 145–148 terminal full release.
12. Preview and migrate the current locked Workset.
13. Run exact-SHA integrated acceptance and only then authorize release through existing authority.

Parallel work is allowed only after shared schemas and operation contracts are frozen, with non-overlapping file ownership.

---

## 33. Normative requirement ledger

| ID | Requirement |
| --- | --- |
| S149-R-001 | One Workset model supports fixed, rolling, and provider-stream cardinality. |
| S149-R-002 | Locked is a sealed revision state, not a separate object type. |
| S149-R-003 | Project root plus continuity ID scope every Workset. |
| S149-R-004 | Provider remains authoritative for current task status. |
| S149-R-005 | Workpoint remains immediate action authority. |
| S149-R-006 | Trajectory remains goal and priority authority. |
| S149-R-007 | Workset membership and flow history are append-only and evidence-backed. |
| S149-R-008 | Query semantics distinguish open-only from all-nonterminal work. |
| S149-R-009 | Provider streams use lazy cursors, epochs, and bounded horizons. |
| S149-R-010 | Open-ended Worksets require explicit/evidence-backed seal before terminal completion. |
| S149-R-011 | Sealed revisions have immutable membership and graph digests. |
| S149-R-012 | Reopening creates a new revision and invalidates dependent candidate proof. |
| S149-R-013 | Existing WorkItem, adapter, scheduler, and closure authority contracts are reused. |
| S149-R-014 | Existing provider-neutral task-plan and materialization contracts are reused. |
| S149-R-015 | Dependency graphs fail closed on cycles. |
| S149-R-016 | Cross-epoch dependencies point only backward or within the epoch. |
| S149-R-017 | Ready frontier is provider-verified, dependency-safe, and bounded. |
| S149-R-018 | Workpoint promotion from the frontier is explicit. |
| S149-R-019 | Checkpoint flows are first-class typed objects. |
| S149-R-020 | Checkpoints support blocking and advisory modes without ambiguity. |
| S149-R-021 | Checkpoint operations resolve through the canonical Operation Registry. |
| S149-R-022 | Checkpoint inputs are typed; arbitrary shell text is forbidden. |
| S149-R-023 | Checkpoint proof uses stable Evidence and Receipts. |
| S149-R-024 | Retry, cooldown, failure fingerprint, and rollback are bounded and typed. |
| S149-R-025 | Completion contract is distinct from task closure. |
| S149-R-026 | Completion transitions are typed operations. |
| S149-R-027 | Current Focusa Workset terminal transition is canonical full release. |
| S149-R-028 | Workset full release invokes Specs 145–147, not a duplicate release engine. |
| S149-R-029 | Workset completion requires release publication, deployment, and live verification. |
| S149-R-030 | Workset completion requires rollback/audit/self-heal/watchdog proof where applicable. |
| S149-R-031 | Workset completion requires Spec 148 journal finalization. |
| S149-R-032 | Workset completion does not itself grant release authority. |
| S149-R-033 | Preload contains a bounded Workset slice, never the complete graph by default. |
| S149-R-034 | Preload exposes omitted counts, cursors, freshness, and rehydrate refs. |
| S149-R-035 | Open-ended preload declares that known count is incomplete. |
| S149-R-036 | Context Cognition curates Workset candidates under token budget. |
| S149-R-037 | Stale provider state blocks consequential action authority. |
| S149-R-038 | ProjectFlowPacket carries one versioned Workset projection to all surfaces. |
| S149-R-039 | Spec 135 series impact is explicit; unknown impact blocks promotion. |
| S149-R-040 | Mission Canvas and Work Rail reuse current A2UI and projection architecture. |
| S149-R-041 | All Workset UI actions bind canonical Operation Registry entries. |
| S149-R-042 | Durable event replay/live tail drives UI refresh and restoration. |
| S149-R-043 | Work Surface session, attachment, origin, and browser isolation are preserved. |
| S149-R-044 | UXP/UFI adaptation cannot mutate Workset authority or semantics. |
| S149-R-045 | Generated UI provides accessible finite and streaming views. |
| S149-R-046 | Provider task content is data, not instruction authority. |
| S149-R-047 | Workset context compiles only into Spec 140 dynamic operational context. |
| S149-R-048 | Consequential Workset operations enforce active Constitution and target profile. |
| S149-R-049 | Spec 140 prompt-injection, secret, path, permission, and integrity gates apply. |
| S149-R-050 | Runtime Studio exposes Workset constitution, enforcement, target parity, and drift. |
| S149-R-051 | API, CLI, Pi, MCP, REST, Mission Canvas, and menubar preserve operation parity. |
| S149-R-052 | Stores and reducers replay deterministically after restart. |
| S149-R-053 | Mutations require expected revision, authority, and idempotency. |
| S149-R-054 | Sealing and completion use single-writer/lease protection. |
| S149-R-055 | Provider outage preserves last verified graph without fabricating readiness. |
| S149-R-056 | LowMem and Bloatgaurd reduce depth without removing authority or recovery fields. |
| S149-R-057 | Migration imports existing locked artifacts through preview and confirmed commit. |
| S149-R-058 | Migration preserves original scope, requirement, dependency, and evidence refs. |
| S149-R-059 | Current locked migration requires zero unmapped mandatory refs and zero cycles. |
| S149-R-060 | Full positive, negative, restart, security, parity, performance, and release tests are mandatory. |
| S149-R-061 | Every consequential transition emits stable Evidence and Spec 119 Receipts. |
| S149-R-062 | No implementation or release pass occurs with deferred or unknown mandatory requirements. |

---

## 34. Terminal acceptance

Spec 149 is implementation-complete only when all of the following are proven:

1. All 62 requirements have `verified` dispositions in a complete ledger.
2. Finite, rolling, and provider-stream Worksets pass end-to-end tests.
3. Intermediate checkpoint flows pass success, failure, retry, restart, and rollback tests.
4. The current locked scope imports with exact membership/requirement/dependency parity.
5. Mission Canvas, Work Rail, Pi, CLI, API, MCP, and headless surfaces agree on one event version.
6. Spec 135 impact/amendment/generated-contract gates are green.
7. Spec 140 instruction, prompt, enforcement, delivery, and cross-harness gates are green.
8. Preload remains bounded under small, large, and open-ended Worksets.
9. Provider outage and stale-state behavior fail closed.
10. Sealed-revision and exact-SHA invalidation tests pass.
11. The Canonical Full Release terminal flow completes through publication, deployment, live verification, rollback/audit/self-heal/watchdog proof, and journal finalization.
12. The Workset terminal Receipt links the sealed Workset revision, candidate SHA, release cycle, deployed/runtime proof, and final journal record.
13. No hidden fallback, duplicate renderer, parallel operation path, prompt-authority leak, or deferred mandatory row remains.

Until then, the status remains **NO IMPLEMENTATION PASS / NO RELEASE PASS**.
