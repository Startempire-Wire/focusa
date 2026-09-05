# Spec 158 — Workstream-Rooted Cognitive Runtime, Canonical State Partitioning, and Foundation Migration

**Status:** Approved normative direction for future foundation repair — not admitted to the current locked release
**Owner / spec author:** Verious Smith  
**Created:** 2026-08-04  
**Priority:** P0 / post-locked-release foundational program; not a blocker for the current locked release
**Tracking issue:** [#125](https://github.com/Startempire-Wire/focusa/issues/125)  
**Reserved number:** 158; repository search found Spec 152 as the highest checked-in top-level master spec. Existing higher-numbered non-spec documents, including document 153, do not claim those specification numbers. Spec 158 preserves the operator-directed five-number buffer for server-only specifications not yet synchronized to the repository.
**Implementation admission:** Approval of this document does not authorize decomposition or implementation. Spec 158 is not admitted to `workset:focusa-next-locked-release:r7`, does not block the current mangled-release repair, and requires a separately authorized post-release Trajectory, Beads plan, ownership ledger, migration plan, cutover plan, and proof plan before production implementation begins.

---

## Locked-release boundary

- This specification may merge as future architectural authority without admitting its implementation to the current locked release.
- Spec 158 is not a member of `workset:focusa-next-locked-release:r7`; no Spec 158 epic, child task, migration tranche, Mission Canvas requirement, or UIAI requirement may join that release graph without a new explicit operator authorization naming the exact obligation.
- Existing locked-release work for #89, #103, #109, #112, and #118 remains bounded containment and regression repair. Passing those gates does not claim Spec 158 compliance or complete foundational isolation.
- Mission Canvas/Desktop and UIAI/Cockpit requirements in this document govern their future migration. They are not current locked-release publication gates.
- GitHub linkage, P0 priority, architectural importance, issue parentage, labels, code presence, or document approval do not constitute implementation admission.

---

## 0. One-line decision

> **Focusa is one daemon serving many isolated Workstreams. A Workstream is the durable cognitive unit that owns Focus Stack, Focus State, Workpoints, Workstream Trajectory, Work Loop, and canonical tactical context. Sessions attach to Workstreams; Sessions, continuity IDs, cwd, visual focus, cached packets, and daemon-global active values do not define Workstream authority.**

---

## 1. Executive decision

Focusa must replace its current daemon-global cognitive aggregate with a canonical state model physically partitioned by exact Scope and stable Workstream identity.

The existing system contains substantial typed-scope, project identity, continuity, attachment, CRDT, Context retrieval, Workpoint, Trajectory, Silent Session, evidence, and Pi runtime work. Those pieces are useful and must be preserved where their behavior is correct. They currently surround a deeper canonical substrate that still behaves like:

```text
one daemon
→ one FocusaState
→ one active Session
→ one active Focus Stack pointer
→ one active Workpoint pointer
→ one active Trajectory pointer
→ one Work Loop
→ one mixed snapshot
```

This leaves isolation dependent on every route, cache, renderer, recovery path, compaction hook, CLI command, Pi callback, and UI surface remembering to filter global `active`, `current`, `last`, or fallback state correctly.

Spec 158 changes the invariant from:

```text
records are labeled with project and continuity fields
```

into:

```text
foreign Workstream state is structurally unavailable to the reducer,
projection, persistence partition, attachment, and prompt assembly path
unless an explicit governed cross-Workstream reference is requested.
```

This is not a cosmetic refactor and is not another guard layer. It is a source-of-truth migration.

---

## 2. Why this specification exists

### 2.1 Investigation scope

The investigation leading to this specification compared:

- original Focusa daemon, Session, Focus Stack, Thread, Instance, Attachment, proposal-resolution, and multi-device intent;
- current `FocusaState`, reducer, daemon, API `AppState`, SQLite persistence, scoped side stores, Pi extension runtime, recovery sidecars, Context retrieval, Workpoint, Trajectory, Work Loop, Silent Sessions, and Runtime Constitution storage;
- Specs 98, 99, and 104 and their closure evidence;
- recent cross-project and resume incidents, including #89, #98, #103, #109, #112, and #118;
- Focusa Desktop / Mission Canvas plans;
- UIAI Engine Cockpit, browser-context isolation, agent-first browser exchange, evidence, and receipt plans.

### 2.2 Original intent was not daemon-global cognition

The earliest Focusa documents already required isolation:

- `docs/02-runtime-daemon.md` states that one daemon owns mutable state **per Session**, that all state belongs to one Session, and that cross-Session access is forbidden by default.
- `docs/G1-detail-03-runtime-daemon.md` explicitly names multiple harness runs, terminal restarts, and concurrent projects as isolation scenarios and proposes a `sessions: HashMap<SessionId, SessionMeta>`.
- `docs/03-focus-stack.md` defines the Focus Stack as nested human attention with one active frame inside the relevant focused work context.
- `docs/39-thread-lifecycle-spec.md` defines the historical Thread as a durable cognitive workspace that owns its Focus Stack, Thesis, lineage, reference namespace, telemetry, and autonomy history, and states that Threads never share mutable state.
- `docs/40-instance-session-attachment-spec.md` explicitly supports one engineer across many projects, many Instances on one cognitive workspace, and cheap detach/attach switching.
- `docs/41-proposal-resolution-engine.md` defines one canonical Focus State and Thesis **per Thread**, not per daemon.
- `docs/43-multi-device-sync.md` defines ownership, observations, proposals, and deterministic no-silent-merge behavior.

The original architecture therefore anticipated isolation and multiplexing. The implementation failure was not a lack of product vision. It was a failure to make the intended owner of cognition structurally explicit in the canonical state root.

### 2.3 Implementation divergence

The implementation retained the original single-Session MVP shape:

```rust
FocusaState {
    session: Option<SessionState>,
    focus_stack: FocusStackState,
    focus_gate: FocusGateState,
    reference_index: ReferenceIndex,
    memory: ExplicitMemory,
    // later subsystems
    workpoint: WorkpointState,
    trajectory: TrajectoryState,
    work_loop: WorkLoopState,
    instances: Vec<Instance>,
    attachments: Vec<Attachment>,
    threads: Vec<Thread>,
    // many project-labeled records
}
```

Concurrency and workspace entities were added **inside or beside** one global aggregate instead of becoming containers of independent cognition.

The current daemon still owns one `FocusaState`; the reducer still reduces one `FocusaState`; core selectors remain singleton-shaped; and SQLite persists the complete aggregate through one canonical snapshot named `focusa`.

Later project-root, continuity, scope, and attachment work often became validation envelopes and side stores around the global substrate. This produced the current hybrid:

```text
typed scoped vocabulary and selected scoped stores
wrapped around
one daemon-global canonical cognition
```

### 2.4 Observed consequences

The mixture creates several failure classes:

1. **Cross-project selection:** a valid foreign record is selected through a global active/latest pointer before a later scope check.
2. **Transition hazards:** project switch, model switch, compaction, resume, daemon restart, worktree change, or remote binding change causes different identity sources to disagree.
3. **Semantic laundering:** transient foreign context enters a compaction or recovery packet and later appears canonical.
4. **Concurrent interference:** serialization prevents memory races but does not prevent one Workstream from changing the global selector another Workstream reads.
5. **False closure:** scoped component tests pass while end-to-end model-visible context remains unsafe.
6. **Persistence blast radius:** one mixed snapshot makes unrelated Workstreams share startup, serialization, migration, and corruption risk.
7. **Forensic debugging:** the same apparent state may be sourced from daemon state, API side maps, scoped ledgers, Pi attachment runtime, recovery sidecars, compaction packets, or caches.
8. **Remote/worktree ambiguity:** paths and Session IDs are asked to carry identity they cannot uniquely represent.

### 2.5 This is a foundation repair, not a repudiation of Focusa

The following principles remain correct and are preserved:

- meaning belongs in structured Focus State, not conversation;
- Focus Stack models nested attention;
- one writer/reducer owns canonical mutation;
- large artifacts are externalized and referenced;
- Workpoints preserve immediate continuation;
- Trajectory binds destination, current state, gap, evidence, and next step;
- Work Loop governs continuous execution;
- PRE preserves competing decisions and resolves them explicitly;
- UIAI Engine is the isolated actuation and browser execution plane;
- Focusa Desktop / Mission Canvas projects canonical state but does not invent authority.

The correction is cardinality and ownership: **many Workstreams, each with its own cognition, supervised by one daemon.**

---

## 3. Normative precedence, supersession, and historical continuity

### 3.1 Governing precedence

For the containment, identity, persistence, reduction, migration, and isolation questions covered here, Spec 158 supersedes any interpretation that permits daemon-global project or Workstream cognition to remain canonical.

Spec 158 does not erase valid behavior or evidence from earlier specifications. It clarifies the source-of-truth shape under them.

### 3.2 Relationship to foundational documents

| Source | Relationship under Spec 158 |
|---|---|
| Spec 39 / historical Thread lifecycle | Intent retained; **Workstream** becomes the canonical product and code term for the durable cognitive container. |
| Spec 40 Instances/Sessions/Attachments | Retained and strengthened; Attachments bind runtime to Workstreams. |
| Spec 41 PRE | Retained; resolution windows and canonical decisions are Workstream-scoped. |
| Spec 43 multi-device sync | Retained; foreign/unverified observations remain quarantined, same-Workstream updates may reconcile only through declared contracts. |
| Spec 79 Work Loop | Retained; loop state and writer authority move under Workstream. |
| Spec 88 Workpoint | Retained; active Workpoint and records become Workstream-owned. |
| Spec 96 Trajectory | Retained; project HLT and Workstream tactical Trajectory are separated explicitly. |
| Spec 98 Project-root CRDT foundation | Diagnosis retained; stable Workstream ID replaces continuity-only identity, and partitioning precedes broad CRDT expansion. |
| Spec 99 original-intent audit | Adopted as historical evidence of the divergence. |
| Spec 104 typed scope/singleton elimination | Valid scoped work retained; no closure claim is complete until the daemon-global canonical root is removed. |
| Spec 107 lifecycle discipline | Governs this specification and its decomposition, proof, and closure. |
| Specs 119/116 evidence, receipts, closure | Retained; evidence and closure authority become Workstream-addressed. |
| Specs 121/135A/135G Desktop and Mission Canvas | Retained; Work Surfaces become projections over stable Workstream Attachments. |
| Specs 130/130A compaction | Retained; compaction/resume must preserve exact Workstream and may never amplify foreign state. |
| Spec 133 Silent Sessions | Retained; durable autonomous Sessions gain stable Workstream and workspace-binding authority. |
| Spec 135 project Genesis/workspaces | Retained; Project Scope owns project-level identity, Genesis, HLT, and explicitly shared truth. |
| Specs 137–152 and later contracts | Must be audited through the required supersession/integration matrix before closure. |

### 3.3 Thread terminology decision

**Workstream is canonical going forward.**

`Thread` remains:

- a historical architecture term;
- a possible legacy identifier stored as migration provenance;
- a user-facing synonym only where a domain pack deliberately chooses it.

It must not remain a parallel canonical container competing with Workstream.

Existing `thread_id` values may map to a Workstream only when the migration can prove that the legacy Thread represented the same durable cognitive lane. Otherwise they remain preserved legacy references.

---

## 4. Goals and non-goals

### 4.1 Goals

Spec 158 SHALL:

1. Make Scope + stable Workstream identity the mandatory address of canonical cognition.
2. Preserve one daemon and one single-writer governance model while supporting many independent Workstreams.
3. Physically partition Focus Stack, Focus State, Workpoints, Workstream Trajectory, Work Loop, context, evidence linkage, ontology working sets, and replay heads.
4. Separate durable Workstream identity from temporal continuation and Session identity.
5. Make ambiguity block or quarantine rather than adopt prior state.
6. Replace mixed global snapshots with independently replayable Workstream projections and events.
7. Provide a safe, versioned, reversible migration for existing data.
8. Align Core, daemon, persistence, API, CLI, MCP, Pi, Desktop, UIAI Engine, Silent Sessions, sync, exports, and documentation.
9. Enable true multi-project and multi-Workstream Desktop experiences.
10. Enable UIAI Engine browser contexts, actions, evidence, and settlement to attach to exact Workstreams.
11. Prove isolation through adversarial runtime and model-visible tests, not source grep alone.
12. Prevent future features from reintroducing implicit singleton authority.

### 4.2 Non-goals

Spec 158 does NOT:

- replace Focus Stack, Focus State, Workpoint, Trajectory, Work Loop, PRE, Evidence, Receipts, or Project Genesis;
- require a universal multi-master CRDT for every cognitive object before partitioning is complete;
- make visual focus, active Desktop tab, or selected Cockpit pane canonical activity;
- make UIAI Engine or Focusa Desktop a second source of canonical mission truth;
- silently merge legacy records whose Workstream cannot be proven;
- use similarity, embeddings, last-used state, cwd, Session ID, or continuity ID alone as Workstream authority;
- require every daemon-infrastructure record to be duplicated per Workstream;
- move license, entitlement, pairing, update, or process-health authority into Workstream state;
- treat an aggregate dashboard as a mutation target;
- permit indefinite dual canonical writes as a migration strategy.

---

## 5. Canonical vocabulary and identity

### 5.1 Scope

A **Scope** is the verified outer authority boundary under which Workstreams exist.

Canonical scope kinds initially include:

```text
project
host
```

Additional scope kinds require an accepted amendment and cannot be invented by clients.

### 5.2 Project Scope

A **Project Scope** is the verified project source of truth. It owns:

- ProjectIdentity and fingerprint;
- Genesis / project constitution;
- project HLT and desired end state;
- workspace, checkout, remote, deployment, and worktree bindings;
- explicitly project-shared verified context;
- the registry of Workstreams under the project.

Project Scope does not own one global tactical Focus Stack, active Workpoint, active Trajectory, or Work Loop.

### 5.3 Host Scope

A **Host Scope** is used only for explicitly host-level work that has no valid Project Scope, such as daemon maintenance or machine operations.

Broad directories such as `/`, `/root`, a home directory, or a generic server workspace are not automatically Host Scope authority. Host Scope must be explicit, typed, and policy-approved.

### 5.4 Workstream

A **Workstream** is a durable cognitive lane inside exactly one Scope.

It owns:

- one Focus Stack;
- Focus State through its frames;
- Workpoint records and one active Workpoint selector;
- Workstream Trajectory and active tactical goal state;
- one Work Loop initially, or explicitly identified loops under a future amendment;
- Workstream context, constraints, decisions, blockers, and evidence links;
- ontology working set and reactive context for the lane;
- canonical event and projection heads;
- Workstream-specific memory and learned adjustments.

A Workstream survives Session restart, harness change, model switch, compaction, daemon restart, machine change, and compatible workspace rebinding.

### 5.5 WorkstreamId

`WorkstreamId` is the durable identity of a Workstream.

Normative initial representation:

```rust
pub type WorkstreamId = Uuid; // UUIDv7 preferred
```

Human-readable names, issue prefixes, roots, and mission labels are mutable metadata and are not identity.

### 5.6 ContinuityId

`ContinuityId` identifies a continuation lineage or generation **inside a Workstream**.

Continuity may change on governed fork, rollover, restoration generation, or explicit continuity transition. It is not the permanent identity of the Workstream.

### 5.7 WorkspaceBinding

A **WorkspaceBinding** maps a Scope/Workstream to an execution location and topology.

It may include:

```yaml
workspace_binding_id:
scope_id:
workstream_id:
kind: local_checkout | git_worktree | remote_checkout | deploy_root | browser_only | document_space | other
machine_id:
host_fingerprint:
transport:
remote_user:
remote_port:
path:
git_common_dir:
repo_identity:
worktree_identity:
deployment_environment:
verified_signals: []
verified_at:
status:
```

A filesystem path alone is never globally unique identity.

### 5.8 Session

A **Session** is a temporal execution window in an Instance or harness.

A Session may have multiple Attachments but every canonical mutation must identify exactly one target Attachment/Workstream.

A Session does not own durable cognition and does not define Project or Workstream identity.

### 5.9 Attachment

An **Attachment** is the live binding between an Instance/Session and one Workstream.

It owns runtime-local facts such as:

- role and capability posture;
- current provider/harness delivery state;
- runtime correlation and continuation references;
- browser or Silent Session references;
- transient prompt staging;
- UI unread state and delivery queues;
- Attachment-local cache epochs.

Attachments grant read/proposal/mutation posture only through policy; they do not bypass reducer authority.

### 5.10 Work Surface

A **Work Surface** is a Desktop, Pi, TUI, Mission Deck, or Cockpit projection over one primary Attachment and optional supporting references.

Visual focus is presentation state:

```text
focused_work_surface_id
≠ global active project
≠ global active Workstream
≠ global active Session
≠ canonical mutation authority
```

---

## 6. Canonical ownership hierarchy

```text
Focusa Daemon
│
├── Daemon Infrastructure State
│   ├── process health and resource pressure
│   ├── peer/device/pairing registries
│   ├── topology and transport clients
│   ├── license/entitlement client state
│   ├── global schema/migration registry
│   └── explicitly unowned operational telemetry
│
├── Scope Registry
│   │
│   ├── Project Scope
│   │   ├── ProjectIdentity
│   │   ├── Genesis / project constitution
│   │   ├── project HLT / desired end state
│   │   ├── workspace bindings
│   │   ├── project-shared verified context
│   │   ├── project-level evidence and artifact visibility policies
│   │   └── Workstreams
│   │       ├── Focus Stack / Focus State
│   │       ├── Workpoints
│   │       ├── Workstream Trajectory
│   │       ├── Work Loop
│   │       ├── context / memory / ontology working set
│   │       ├── evidence and claim links
│   │       ├── policies and writer leases
│   │       └── event / projection heads
│   │
│   └── Host Scope
│       └── explicitly host-scoped Workstreams
│
├── Sessions
│   └── temporal harness execution records
│
└── Attachments
    └── Session/Instance → Workstream bindings and runtime-local state
```

---

## 7. Foundational laws and invariants

### 7.1 Canonical-address law

No canonical cognitive read or write may occur without an exact `ScopeRef + WorkstreamId`.

### 7.2 No global cognition law

Daemon-global `active`, `current`, `latest`, `last`, remembered, nearest, or default values may not carry project or Workstream cognition.

### 7.3 Fail-closed ambiguity law

If Scope, Workstream, Attachment, workspace binding, or authority is ambiguous, the system returns a blocked/ambiguous envelope containing no foreign canonical payload.

### 7.4 Per-Workstream active-frame law

At most one Focus Frame is active per Workstream. Multiple Workstreams may each have one active frame simultaneously.

### 7.5 Per-Workstream Workpoint law

At most one canonical active Workpoint selector exists per Workstream. Workpoint records cannot supersede records in another Workstream.

### 7.6 Project-HLT / Workstream-Trajectory law

Project Scope owns stable project HLT and desired end state. Each Workstream owns its tactical mission, current state, gaps, milestones, and next bounded route toward that project truth.

A Workstream may diverge from the project HLT only through an explicit governed relationship or fork; divergence is never inferred from stale context.

### 7.7 Session non-ownership law

Session and continuity identities are temporal metadata. They cannot create, merge, rename, or select a Workstream implicitly.

### 7.8 Visual-focus law

Selecting a Desktop tab, Cockpit pane, TUI view, or Pi widget changes client presentation and explicit target selection only. It does not mutate global daemon cognition.

### 7.9 Raw-output integrity law

Tool stdout/stderr, browser action results, and external execution output remain byte-faithful at their raw boundary. Focusa augmentation is separate, provenance-labeled, scope-verified, and removable.

### 7.10 Cross-Workstream reference law

Sharing context, artifacts, evidence, or facts across Workstreams requires an explicit reference edge with provenance and authority semantics. Copying data does not transfer action authority.

### 7.11 Scope placement law

Every authority-bearing field has exactly one declared owner plane:

```text
daemon infrastructure
scope/project
workstream
attachment/runtime
derived projection
legacy/quarantine
```

### 7.12 Single-writer, many-partitions law

Focusa retains one canonical single-writer governance path. The writer routes an event to one exact partition; single writer does not mean single cognition.

### 7.13 Persistence independence law

Each Workstream can be snapshotted, replayed, exported, validated, migrated, restored, and quarantined independently.

### 7.14 No fallback authority law

The following are forbidden as canonical fallback authority:

- daemon active frame;
- daemon active Workpoint;
- daemon active Trajectory;
- daemon current task;
- latest record in an array;
- last verified project or Attachment;
- prior-project trajectory;
- session file path;
- cwd or process cwd alone;
- `/root`, home, repository parent, or broad directory;
- similarity result;
- `default_workstream`;
- `unscoped_project_root`.

### 7.15 Migration evidence law

Legacy records are assigned to a Workstream only when evidence proves the assignment. Ambiguity is quarantined and surfaced; it is never resolved for convenience.

### 7.16 No indefinite dual-authority law

Shadow projections and comparison are allowed during migration. Two canonical write paths are not.

### 7.17 Model-visible closure law

Isolation is not considered complete until the exact model-visible context, compaction packet, tool augmentation, and recovery payload are proven clean under concurrent and alternating Workstreams.

### 7.18 UIAI execution law

UIAI Engine may execute only through an exact Workstream Attachment and matching runtime reference. Browser context identity cannot substitute for Workstream authority.

### 7.19 Desktop projection law

Focusa Desktop and Mission Canvas consume generated Workstream contracts and projections; they do not maintain a parallel canonical cognitive database.

### 7.20 Public-claim law

Focusa may not claim complete multi-project isolation, singleton elimination, deterministic Workstream portability, or safe concurrent cognition until all Spec 158 closure gates pass.

---

## 8. State ownership planes

Spec 158 requires a generated, complete field inventory. The following placement is normative for the major classes.

| State class | Canonical owner |
|---|---|
| Process health, resource pressure, schema version | Daemon infrastructure |
| Peer/device/pairing registry | Daemon infrastructure |
| License and entitlement client state | Daemon infrastructure |
| ProjectIdentity, fingerprint, aliases | Project Scope |
| Genesis / project constitution | Project Scope |
| Project HLT / desired end state | Project Scope |
| Workspace and deployment bindings | Project Scope |
| Explicitly project-shared verified facts | Project Scope |
| Focus Stack and Focus State | Workstream |
| Workpoint index and active Workpoint | Workstream |
| Tactical Trajectory, milestones, gaps | Workstream |
| Work Loop, current task, blockers, budget | Workstream |
| Writer lease and fencing state | Workstream or narrower explicit loop partition |
| Tactical constraints and decisions | Workstream |
| Ontology working set and reactive context | Workstream |
| Workstream memory and metacognitive adjustments | Workstream |
| Session lifecycle and harness metadata | Session registry |
| Provider delivery state, current ask, prompt staging | Attachment/runtime |
| Pi callback routing and UI delivery queues | Attachment/runtime |
| UIAI session/context/target refs | Attachment/runtime |
| Raw content-addressed artifact blobs | Global blob store allowed |
| Artifact visibility, claims, evidence linkage | Project Scope or Workstream, explicitly declared |
| Cross-project dashboard | Derived projection only |
| Legacy singleton selectors | Legacy compatibility, noncanonical, then removed |
| Ambiguous legacy records | Quarantine |

No field may be classified as “global for convenience” when it can influence project/Workstream action, prompt, recovery, completion, or authority.

---

## 9. Canonical runtime type shape

The exact code may evolve during implementation, but the ownership relationships are normative.

```rust
pub struct FocusaRuntimeState {
    pub daemon: DaemonInfrastructureState,
    pub scopes: HashMap<ScopeId, ScopeState>,
    pub sessions: HashMap<SessionId, SessionRecord>,
    pub attachments: HashMap<AttachmentId, AttachmentRecord>,
    pub schema_version: u32,
}

pub enum ScopeState {
    Project(ProjectScopeState),
    Host(HostScopeState),
}

pub struct ProjectScopeState {
    pub scope_id: ScopeId,
    pub identity: ProjectIdentityRecord,
    pub genesis: ProjectGenesisState,
    pub hlt: ProjectHltState,
    pub workspace_bindings: HashMap<WorkspaceBindingId, WorkspaceBinding>,
    pub shared_context: ProjectSharedContext,
    pub workstreams: HashMap<WorkstreamId, WorkstreamState>,
    pub projection_head: ProjectionHead,
}

pub struct WorkstreamState {
    pub key: WorkstreamKey,
    pub metadata: WorkstreamMetadata,
    pub continuities: ContinuityIndex,
    pub focus_stack: FocusStackState,
    pub workpoints: WorkpointState,
    pub trajectory: WorkstreamTrajectoryState,
    pub work_loop: WorkLoopState,
    pub context: WorkstreamContextState,
    pub memory: WorkstreamMemoryState,
    pub ontology: WorkstreamOntologyState,
    pub evidence: WorkstreamEvidenceIndex,
    pub event_head: EventHead,
    pub projection_version: u64,
}

pub struct WorkstreamKey {
    pub scope_id: ScopeId,
    pub workstream_id: WorkstreamId,
}

pub struct AttachmentKey {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub instance_id: InstanceId,
    pub session_id: SessionId,
    pub attachment_id: AttachmentId,
}
```

### 9.1 Legacy compatibility identity

The old concept:

```text
ProjectRootKey + continuity_id = WorkstreamKey
```

becomes a `LegacyWorkstreamLocator`, not the final identity.

```rust
pub struct LegacyWorkstreamLocator {
    pub project_root_key: String,
    pub continuity_id: String,
}
```

Migration maps it to a stable `WorkstreamId` only when evidence is sufficient.

### 9.2 Historical Thread records

```rust
pub struct LegacyThreadRef {
    pub thread_id: Uuid,
    pub mapped_workstream_id: Option<WorkstreamId>,
    pub mapping_evidence_refs: Vec<String>,
}
```

Historical Thread metadata remains inspectable after migration.

---

## 10. Reducer and event contract

### 10.1 Scoped event envelope

All canonical-capable events must enter through a typed envelope:

```rust
pub struct ScopedEventEnvelope {
    pub event_id: EventId,
    pub scope_id: ScopeId,
    pub workstream_id: Option<WorkstreamId>,
    pub continuity_id: Option<ContinuityId>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub instance_id: Option<InstanceId>,
    pub session_id: Option<SessionId>,
    pub attachment_id: Option<AttachmentId>,
    pub actor: ActorRef,
    pub authority: AuthorityEnvelope,
    pub causal: CausalMetadata,
    pub temporal: TemporalActionEnvelope,
    pub payload: FocusaEvent,
}
```

Project-level events may omit `workstream_id` only when their event kind is explicitly declared Project Scope authority, such as Genesis or project HLT revision.

Daemon-infrastructure events use a separate declared envelope class and cannot mutate cognition.

### 10.2 Reducer routing

Conceptually:

```rust
fn reduce_runtime(
    runtime: FocusaRuntimeState,
    envelope: ScopedEventEnvelope,
) -> Result<RuntimeReductionResult, ReducerError> {
    validate_scope_and_authority(&runtime, &envelope)?;
    match event_partition(&envelope.payload) {
        EventPartition::Daemon => reduce_daemon(...),
        EventPartition::Project(scope_id) => reduce_project(...),
        EventPartition::Workstream(key) => reduce_workstream(...),
        EventPartition::Attachment(attachment_id) => reduce_attachment_runtime(...),
    }
}
```

The reducer must never select a target by consulting a global active pointer.

### 10.3 Read contract

Canonical reads require a `WorkstreamContext`:

```rust
pub struct WorkstreamContext {
    pub scope_id: ScopeId,
    pub workstream_id: WorkstreamId,
    pub continuity_id: Option<ContinuityId>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub session_id: Option<SessionId>,
    pub attachment_id: Option<AttachmentId>,
    pub expected_event_head: Option<EventHead>,
    pub authority: ReadAuthority,
}
```

A request lacking exact context returns one of:

```text
scope_required
workstream_required
attachment_required
workspace_binding_required
scope_ambiguous
workstream_ambiguous
stale_event_head
```

The response must contain no canonical foreign Workstream payload.

### 10.4 Per-Workstream invariants

The reducer SHALL enforce:

- active Focus Frame belongs to the addressed Workstream;
- frame parent belongs to the same Workstream;
- active Workpoint belongs to the addressed Workstream;
- Workpoint supersession stays within one Workstream;
- active tactical Trajectory belongs to the Workstream;
- Work Loop and writer lease belong to the Workstream;
- evidence linkage validates owner and target Scope/Workstream;
- ontology mutations cannot cross Workstream without an explicit migration/reference event;
- Attachment role and authority permit the proposed action;
- expected event head/fencing token prevents stale writes where required.

### 10.5 Project-level sharing

Project-level HLT, Genesis, and verified shared context are referenced by Workstreams. Workstream projections may include bounded project-level slices but cannot write them through tactical events.

### 10.6 PRE and conflict resolution

PRE resolution windows are keyed by:

```text
scope_id
workstream_id
canonical target
window
```

Compatible append-only facts may merge through declared policies. Conflicting decisions become visible resolution candidates. Alternatives remain inspectable.

Different Workstreams never resolve into one canonical object.

---

## 11. Persistence, projection, replay, and export

### 11.1 Required canonical tables

The final schema may use normalized tables plus JSON projections, but it must provide equivalent first-class identity and independence.

```sql
CREATE TABLE scope_roots (
    scope_id TEXT PRIMARY KEY,
    scope_kind TEXT NOT NULL,
    identity_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE workstreams (
    workstream_id TEXT PRIMARY KEY,
    scope_id TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    status TEXT NOT NULL,
    legacy_thread_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(scope_id) REFERENCES scope_roots(scope_id)
);

CREATE TABLE workstream_continuities (
    workstream_id TEXT NOT NULL,
    continuity_id TEXT NOT NULL,
    generation INTEGER NOT NULL,
    status TEXT NOT NULL,
    parent_continuity_id TEXT,
    created_at TEXT NOT NULL,
    PRIMARY KEY(workstream_id, continuity_id)
);

CREATE TABLE workspace_bindings (
    workspace_binding_id TEXT PRIMARY KEY,
    scope_id TEXT NOT NULL,
    workstream_id TEXT,
    binding_kind TEXT NOT NULL,
    machine_id TEXT,
    host_fingerprint TEXT,
    path TEXT,
    identity_json TEXT NOT NULL,
    status TEXT NOT NULL,
    verified_at TEXT
);

CREATE TABLE scoped_events (
    event_id TEXT PRIMARY KEY,
    scope_id TEXT NOT NULL,
    workstream_id TEXT,
    continuity_id TEXT,
    workspace_binding_id TEXT,
    instance_id TEXT,
    session_id TEXT,
    attachment_id TEXT,
    event_kind TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    authority_json TEXT NOT NULL,
    causal_json TEXT NOT NULL,
    temporal_json TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    previous_hash TEXT,
    event_hash TEXT NOT NULL
);

CREATE TABLE projection_snapshots (
    scope_id TEXT NOT NULL,
    workstream_id TEXT,
    projection_kind TEXT NOT NULL,
    projection_version INTEGER NOT NULL,
    event_head TEXT NOT NULL,
    state_json TEXT NOT NULL,
    state_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY(scope_id, workstream_id, projection_kind, projection_version)
);

CREATE TABLE scope_migrations (
    migration_id TEXT PRIMARY KEY,
    source_schema_version INTEGER NOT NULL,
    target_schema_version INTEGER NOT NULL,
    status TEXT NOT NULL,
    plan_json TEXT NOT NULL,
    proof_json TEXT,
    started_at TEXT,
    completed_at TEXT,
    rolled_back_at TEXT
);

CREATE TABLE scope_quarantine (
    quarantine_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    reason TEXT NOT NULL,
    candidate_scopes_json TEXT NOT NULL,
    evidence_refs_json TEXT NOT NULL,
    raw_record_json TEXT NOT NULL,
    status TEXT NOT NULL,
    resolved_scope_id TEXT,
    resolved_workstream_id TEXT,
    resolved_at TEXT
);
```

### 11.2 Projection independence

At minimum, Workstream projection kinds include:

```text
focus_stack
workpoint
trajectory
work_loop
context
memory
ontology
Evidence index
Mission Canvas read model
```

A malformed projection for one Workstream must not force unrelated Workstreams to start empty.

### 11.3 Event scope columns are mandatory

Scope cannot be encoded only inside `payload_json` or parsed from `correlation_id` text.

No canonical event may default to `unscoped_project_root` or `default_workstream`.

### 11.4 Global blob store

ECS/UIAI artifacts may remain content-addressed globally. Their references, visibility, claims, Workpoint linkage, Evidence status, and authorization must be scoped.

### 11.5 Workstream export

A Workstream export must be self-contained and inspectable:

```yaml
schema: focusa.workstream_export.v1
scope_identity:
workstream:
continuities: []
workspace_bindings: []
event_range:
projection_heads:
Focus Stack:
Workpoints:
Trajectory:
Work Loop:
context_refs: []
evidence_refs: []
artifact_manifest: []
provider_state_manifest: []
conflicts: []
quarantine_exclusions: []
integrity:
```

Export must not require filtering unrelated records out of one daemon-global snapshot after the fact.

---

## 12. Foundation migration and cutover

### 12.1 Migration principles

- replacement before removal;
- shadow projection before cutover;
- no silent assignment;
- complete backups;
- deterministic migration plan;
- hash-verified parity where applicable;
- explicit difference ledger where semantics change;
- rollback tested before authority cutover;
- no indefinite dual canonical writes;
- migration does not claim feature completion by itself.

### 12.2 Legacy inputs

Migration must inventory:

- global `snapshots(name='focusa')` state;
- append-only events and hash chains;
- CRDT events and scope defaults;
- Workpoint records and selectors;
- Trajectory records, HLT ledgers, and selectors;
- Focus Frames and stack selectors;
- Work Loop state and writer records;
- context sources, claims, embeddings, and retrieval indexes;
- ontology and reactive-context records;
- evidence, receipts, closure claims, and artifacts;
- Threads, Instances, Sessions, and Attachments;
- Silent Session table families and runner state;
- API scoped side stores and filesystem ledgers;
- Pi recovery sidecars, typed scope stores, attachment bindings, and native session references;
- Desktop/Menubar restoration state;
- UIAI Work Surface, browser session/context/target references where integrated;
- caches and adaptive policy state.

### 12.3 Mapping evidence

A legacy record may map to a Workstream through evidence such as:

- explicit stable Workstream ID already present;
- exact verified project fingerprint and continuity mapping;
- exact Attachment binding;
- exact Workpoint/Trajectory owner relationship;
- exact frame/project/continuity relationship;
- migration-approved historical Thread mapping;
- event ancestry proving one Workstream;
- operator-approved assignment.

A matching label or similar mission is insufficient.

### 12.4 Quarantine

Ambiguous or conflicting records enter `scope_quarantine` with:

- raw immutable source;
- reason;
- candidate scopes/Workstreams;
- supporting and conflicting evidence;
- affected projections;
- operator repair options;
- explicit statement that the record is excluded from canonical prompt assembly.

### 12.5 Cutover sequence

```text
Phase 0  Freeze new daemon-global cognitive fields and unscoped routes
Phase 1  Introduce ScopeId, WorkstreamId, WorkspaceBindingId, and migration registry
Phase 2  Add scoped event and snapshot substrate
Phase 3  Shadow-materialize Workpoint and Trajectory partitions
Phase 4  Prove parity and cut over Workpoint/Trajectory authority
Phase 5  Shadow-materialize Focus Stack and Focus State partitions
Phase 6  Prove parity and cut over Focus Stack/Focus State authority
Phase 7  Partition Work Loop, writer authority, Silent Sessions, and runner
Phase 8  Partition Context, memory, ontology, evidence links, and reactive context
Phase 9  Cut over API, CLI, MCP, Pi, Desktop, schemas, and generated clients
Phase 10 Migrate live legacy data and resolve/quarantine ambiguity
Phase 11 Disable global snapshot and active/current/last authority
Phase 12 Run rollback window and adversarial proof
Phase 13 Remove legacy selectors, fallback routes, and obsolete stores
```

### 12.6 Shadow comparison

During shadow phases:

- legacy remains canonical for the bounded tranche;
- scoped projection is computed independently;
- differences are recorded, not overwritten;
- prompts and mutations do not mix legacy and scoped results;
- cutover requires an accepted parity/difference receipt.

### 12.7 Rollback

Rollback must restore the previous executable authority boundary without:

- losing new scoped events;
- reassigning quarantined records;
- corrupting legacy snapshots;
- causing both writers to remain active;
- exposing foreign Workstream content.

---

## 13. Subsystem requirements

### 13.1 Core types and daemon

- Replace one canonical `FocusaState` cognition root with runtime infrastructure plus Scope/Workstream registries.
- Retain one single-writer governance path.
- Remove `current_instance_id` / `current_thread_id` or similar fields from canonical cognitive selection.
- Allow independent Workstream loading, eviction, checkpointing, and recovery.
- Expose bounded daemon status separately from Workstream status.
- Resource management may unload inactive projections but must preserve independent event heads.

### 13.2 Focus Stack and Focus State

- Move `FocusStackState` under Workstream.
- Make root and active frame selectors per Workstream.
- Make frame parent relationships Workstream-local.
- Scope all Focus State writes through frame owner and Workstream context.
- Remove unscoped active-frame fallback from API, CLI, Pi, and Desktop.
- Aggregate views may show several active frames but cannot mutate without an exact target.

### 13.3 Workpoint

- Move Workpoint index, active selector, checkpoint, resume, evidence, drift, supersession, and closure refs under Workstream.
- Require stable Workstream ID in every Workpoint record and packet.
- Make continuity generation metadata, not owner identity.
- Forbid cross-Workstream supersession.
- Make cross-Workstream inspiration/advisory links explicit and non-authoritative.
- Include Workstream event head and Workpoint revision in resume packets.

### 13.4 Project HLT and Workstream Trajectory

Project Scope owns:

```text
root project HLT
desired end state
project-wide stable constraints
Genesis and project truth
```

Workstream owns:

```text
mission and bounded outcome
current verified state
short/medium goals
active gap
milestones and waypoints
active Workpoint relationship
blockers and evidence of progress
```

- Remove global active Trajectory authority.
- Remove prior-project fallback from canonical prompt and tool paths.
- Keep similarity, historical comparison, and cross-project examples advisory and provenance-labeled.
- HLT ledgers must use Scope/Project identity; tactical Trajectory ledgers use Workstream ID.

### 13.5 Work Loop and writer authority

- Move loop status, current task, retries, budgets, temporal state, transport state, blocker package, continuation policy, checkpoints, and writer lease under Workstream.
- Daemon supervises many loops without one global current task.
- Writer lease keys include Scope, Workstream, and narrower root work partition where required.
- One Workstream cannot pause, resume, retry, select, checkpoint, complete, or fence another Workstream.
- Resource scheduling across Workstreams is daemon infrastructure and cannot rewrite Workstream intent.

### 13.6 Context, retrieval, memory, ontology, and reactive context

- Replace project-root + continuity-only indexing with stable Workstream ID while retaining narrower Attachment filters where needed.
- Validate owner scope at write time; downstream filtering cannot repair an originally mis-scoped record.
- Classify memory explicitly as user/global, project-shared, Workstream-owned, or Attachment/runtime.
- Move ontology working sets, proposals, verification, reactive context, and affordances under Workstream unless explicitly project-shared.
- Cross-Workstream retrieval requires explicit query posture and returns advisory provenance, never automatic canonical injection.
- Cache keys include Scope + Workstream.

### 13.7 Evidence, Receipts, closure, and artifacts

- Evidence candidates include Scope, Workstream, Workpoint, action, runtime, and verification target.
- Receipts and closure claims validate Workstream owner relationships.
- An artifact blob may be shared; the claim it proves is not implicitly shared.
- Cross-Workstream evidence reuse creates an explicit reference with source Workstream, target Workstream, rationale, and `authority_transfers=false` by default.
- Completion cannot be inferred from evidence attached to another Workstream.

### 13.8 Sessions, Attachments, Silent Sessions, and runner

- Session registry supports many active Sessions.
- Attachments bind each Session/Instance to exact Workstreams.
- A Session with several Attachments declares explicit primary presentation only; each mutation still names its target.
- Silent Session controls, runs, events, config revisions, checkpoints, leases, approvals, audits, backends, retention, and notifications add stable Workstream and workspace-binding identity.
- Runner protocols verify Workstream and workspace binding before mutation.
- Session restart, process restart, transport fallback, and host change do not create a new Workstream implicitly.

### 13.9 Pi extension

- Add stable Workstream ID and workspace binding to `AttachmentRuntimeRegistry` keys.
- Remove latest-bound Session/Attachment fallback from canonical prompt-visible behavior.
- Scope every event, command, shortcut, tool invocation, callback, model switch, compaction, resume, and lifecycle advisory through an explicit or exactly restored Attachment.
- Scope-less hooks are advisory-only or blocked.
- Key recovery sidecars by Workstream + workspace binding + native Session identity, not only project path and Session.
- Preserve native Pi Session UUID separately from Session file/reopen handle.
- Raw stdout/stderr remains clean; augmentation is a separate typed block containing Workstream and provenance.
- Compaction/resume includes Scope fingerprint, Workstream ID, continuity generation, event head, Workpoint revision, Trajectory revision, canonical/degraded status, and source APIs.
- Adaptive context/compaction policy state is segmented by Workstream/Attachment and cannot leak learned state across incompatible partitions.

### 13.10 API, CLI, MCP, schemas, and generated clients

Canonical route shape should converge toward:

```text
/v1/daemon/status
/v1/scopes
/v1/scopes/{scope_id}
/v1/scopes/{scope_id}/workstreams
/v1/workstreams/{workstream_id}
/v1/workstreams/{workstream_id}/focus
/v1/workstreams/{workstream_id}/workpoints
/v1/workstreams/{workstream_id}/trajectory
/v1/workstreams/{workstream_id}/work-loop
/v1/workstreams/{workstream_id}/context
/v1/workstreams/{workstream_id}/evidence
/v1/workstreams/{workstream_id}/attachments
```

- Compatibility endpoints may remain temporarily but must resolve exact Workstream, emit deprecation metadata, and never fall back.
- CLI selection profiles are client convenience only.
- `focusa status` means daemon infrastructure; `focusa workstream status` means cognition.
- MCP/Pi/tool contracts use one shared Workstream envelope.
- OpenAPI, JSON Schema, Rust, TypeScript, Svelte, Tauri, and UIAI generated contracts must agree.
- Ambiguous requests return bounded candidates and recovery without foreign canonical payload.

### 13.11 Focusa Desktop, Menubar, TUI, Mission Canvas, and Work Rail

Spec 158 turns planned multiplexed UI into real independent cognition.

Desktop SHALL support:

- Projects/Scopes containing many Workstreams;
- Work Surfaces bound to exact Attachments;
- several simultaneous active/paused/blocked/running Workstreams;
- split panes and comparison without authority merge;
- per-Workstream Focus Stack, Workpoint, Trajectory, Work Loop, Evidence, attachments, conflicts, and notifications;
- explicit steering and follow-up targets;
- safe restoration from Workstream event/projection heads;
- read-only aggregate Project and Global Activity views;
- deep links carrying Scope/Workstream identity;
- per-Workstream unread, approval, blocker, and writer indicators.

Desktop SHALL NOT:

- maintain a second canonical cognitive store;
- use the visually selected surface as implicit daemon-global authority;
- restore a Work Surface through last-active project fallback;
- send a mutation from an aggregate view without explicit target selection.

Recommended deep-link form:

```text
focusa://workstreams/{workstream_id}
focusa://workstreams/{workstream_id}/workpoints/{workpoint_id}
focusa://workstreams/{workstream_id}/surfaces/{work_surface_id}
```

Existing project-root aliases may redirect only after exact verification.

### 13.12 UIAI Engine and Cockpit

UIAI Engine remains the execution/actuation plane. Focusa remains canonical mission and continuity authority. Cockpit remains operator presentation and control.

Canonical relationship:

```text
Focusa Project Scope / Workstream
└── Attachment
    └── UIAI Session
        └── Browser Context
            ├── Target / tab
            ├── Observation stream
            ├── Action execution
            └── Evidence / settlement observations
```

Every UIAI browser action and result must carry or resolve:

```yaml
scope_ref:
workstream_id:
attachment_id:
workpoint_ref:
action_proposal_ref:
capability_grant_ref:
uiai_session_id:
browser_context_id:
target_id:
expected_observation_id:
```

UIAI SHALL reject or require resync when:

- Workstream does not match the action proposal;
- Attachment is stale or detached;
- browser context was restored under another Workstream;
- expected observation/document/navigation/frame is stale;
- Workpoint or authority revision no longer matches;
- workspace binding is incompatible;
- evidence would be attributed to another Workstream.

Spec 158 enables:

- concurrent authenticated browser contexts for independent Workstreams;
- safe Pi → UIAI → Pi verification loops;
- Workstream-attributed screenshots, recordings, DOM/AX observations, network evidence, predicates, and settlement watchers;
- Cockpit tabs and split panes that do not create global current authority;
- safe browser-context restoration and explicit incompatibility states;
- multi-project automation without cookie, navigation, target, mission, or Evidence crossover.

UIAI Engine must consume generated Focusa Workstream contracts by version/digest rather than hand-maintaining duplicate authority DTOs.

### 13.13 Sync, CRDT, PRE, multi-device, remote, and worktree topology

Partitioning precedes broad cognitive CRDT expansion.

```text
same Workstream + declared compatible operation
  → deterministic merge

same Workstream + conflicting decision
  → explicit PRE/conflict record/operator resolution

different Workstream
  → never merge

unverified Scope or Workstream
  → quarantine/observation only
```

- CRDT convergence does not prove correct scope assignment.
- Project and Workstream identity includes verified topology, not path alone.
- Same path on two hosts is distinct without proven shared identity.
- Several worktrees are explicit WorkspaceBindings under a Project Scope.
- Worktree execution state cannot silently transfer between Workstreams.
- Remote controller-daemon topology from #89 and native Session/binding work from #118 must be integrated into this foundation.

### 13.14 Caching, idempotency, telemetry, training, and portability

Authority-sensitive keys begin with:

```text
scope_id + workstream_id
```

then add continuity, workspace binding, Session, Attachment, projection, or runtime identity as required.

Forbidden sole cache/idempotency keys include:

- Session ID;
- continuity ID;
- path;
- record ID without Workstream;
- endpoint name;
- `current`;
- `latest`;
- last verified project.

Telemetry is classified by plane and cannot imply global cognition. Training/export datasets may aggregate Workstreams only through explicit dataset intent and provenance. Portable handoff exports exact Workstream state and references rather than provider-bound or daemon-mixed session state.

### 13.15 Installation, update, licensing, entitlements, pairing, and topology

These remain daemon infrastructure but must be audited for hidden single-current-project assumptions.

- Installer/update paths back up, migrate, validate, and roll back the new schema.
- License and entitlement posture cannot grant Workstream authority by itself.
- Paired devices carry exact Workstream authority on cognitive actions.
- Doctor distinguishes daemon readiness, Scope readiness, Workstream readiness, migration status, quarantine count, and compatibility mode.
- Update cannot proceed through a destructive migration without backup and rollback proof.

---

## 14. Product outcomes and new-feature policy

### 14.1 Focusa Desktop outcome

After Spec 158, Focusa Desktop can truthfully become a multi-Workstream mission-control environment rather than a UI that visually multiplexes over one mixed cognition.

High-value outcomes include:

- reliable multi-project tabs;
- several active Workstreams under one project;
- concurrent Pi, UIAI, Silent Session, research, document, and Evidence surfaces;
- safe split panes and comparisons;
- per-Workstream steering, approvals, notifications, Work Rails, and history;
- durable restoration without adopting daemon-last state;
- aggregate dashboards that remain read-only projections;
- portable handoff between Desktop, Pi, mobile, and remote clients.

### 14.2 UIAI Engine outcome

UIAI Engine becomes a safer parallel actuation plane:

- each browser context is attached to one exact Workstream;
- actions are observation-bound and Workstream-authorized;
- several browser automations may run simultaneously without mission crossover;
- Evidence and Receipts retain exact Workstream, Workpoint, browser context, observation, and predicate attribution;
- Cockpit can host Focusa Mission Canvas projections without becoming canonical mission truth;
- browser-context restoration can block rather than silently reattach incompatibly.

### 14.3 New-feature admission rule

Until cutover, new authority-bearing features must be designed against the Spec 158 contract and must not deepen dependency on:

```text
currentProject
activeSession
activeWorkpoint
lastTrajectory
latestAttachment
selectedBrowserContext
```

as shared canonical fields.

Comparatively safe pre-cutover work includes pure presentation components, artifact viewers, themes, domain profiles, browser observation/action primitives, generated-contract tooling, stateless diagnostics, and content-addressed storage—provided they accept Workstream-ready identity contracts.

Features that mutate, restore, schedule, steer, compact, authorize, or attach cognition must wait for or directly participate in the relevant Spec 158 migration tranche.

---

## 15. API and envelope behavior

### 15.1 Canonical result envelope

All cognitive results should expose:

```yaml
schema: focusa.result.vNext
status: ok | blocked | stale | ambiguous | degraded | failed
canonical: true | false
scope_ref:
workstream_id:
continuity_id:
workspace_binding_id:
attachment_id:
event_head:
projection_version:
payload:
provenance:
uncertainty:
conflicts: []
quarantine_refs: []
retry:
recovery:
next_actions: []
receipt_ref:
```

### 15.2 Aggregate result envelope

Aggregate views state:

```yaml
canonical_mutation_target: false
aggregate_kind: project | multi_project | daemon
members:
```

A mutation initiated from an aggregate UI must select one exact target first.

### 15.3 Compatibility metadata

Legacy endpoints expose:

```yaml
compatibility:
  legacy_route: true
  authority_source: scoped_v2
  deprecated_fields: []
  removal_after:
```

They cannot continue reading the old global aggregate after scoped cutover.

---

## 16. Performance and resource requirements

Partitioning must improve or preserve hot-path performance.

Required properties:

- load only addressed Workstream projections on canonical hot paths;
- daemon status does not deserialize every Workstream;
- one Workstream checkpoint does not serialize every project;
- bounded projection caches keyed by Scope + Workstream;
- independent corruption/rebuild domains;
- predictable lock ownership per reducer/partition;
- no network or model call inside reducer;
- no full project/worktree scan in prompt hot path;
- aggregate dashboards use bounded read models;
- Desktop/UIAI subscriptions invalidate only affected Workstreams;
- migration shadow work is rate-limited and does not block active execution.

Performance acceptance compares legacy and scoped operation latency, memory, startup, serialization bytes, lock duration, and prompt assembly cost.

---

## 17. Security, privacy, and authority

- Foreign Workstream content is treated as an authority violation even when the user owns both projects.
- Secrets, browser authentication, private documents, and Evidence inherit the visibility policy of their owner Scope/Workstream and runtime context.
- Cross-Workstream reference edges preserve source visibility and redaction policy.
- Aggregate telemetry contains hashes and metrics, not raw cognition, unless explicitly exported.
- UIAI browser contexts must not share cookie/storage partitions unless explicitly configured and compatible with Workstream policy.
- Quarantined records are excluded from prompt assembly, action authority, completion, and training export.
- Operator overrides are explicit, logged, reversible, and cannot permanently weaken global isolation invariants.
- Break-glass access creates a Receipt and does not silently relabel data.

---

## 18. Required companion artifacts

Before implementation closure, Spec 158 requires these generated or maintained companions:

1. `docs/contracts/spec158-current-head-state-ownership-ledger.v1.yaml`  
   Every authority-bearing field/store/cache/route/UI projection and its current owner, target owner, migration phase, and removal proof.

2. `docs/contracts/spec158-supersession-and-integration-matrix.v1.yaml`  
   Specs 39–152 and later server-synchronized specs, with retained, amended, superseded, or blocked clauses.

3. `docs/contracts/spec158-migration-dag.v1.yaml`  
   Dependency-ordered schema, reducer, subsystem, client, live-data, and removal tranches.

4. `docs/contracts/spec158-api-client-cutover-ledger.v1.yaml`  
   REST, CLI, MCP, Pi, OpenAPI, schemas, generated clients, Desktop, and UIAI contract parity.

5. `docs/contracts/spec158-legacy-record-mapping-policy.v1.yaml`  
   Evidence thresholds, quarantine reasons, operator resolution, and rollback rules.

6. `docs/contracts/spec158-complete-proof-matrix.v1.yaml`  
   Unit, integration, migration, live runtime, model-visible, Desktop, UIAI, remote, worktree, compaction, restart, and rollback evidence.

7. `docs/evidence/spec158-baseline-current-head-audit-2026-08-04.md`  
   Pinned commit SHA, global singleton inventory, contamination reproduction, snapshot inventory, and pre-migration database state.

8. `docs/evidence/spec158-final-foundation-closure.md`  
   Final accepted proof only; cannot be created as a completion artifact until every gate is actual.

---

## 19. Mandatory adversarial proof matrix

Source scans and isolated component tests are necessary but insufficient.

### 19.1 Core isolation

- [ ] Two projects hold active Focus Frames, Workpoints, Trajectories, and Work Loops simultaneously with zero crossover.
- [ ] Two Workstreams under one Project Scope share only explicitly project-level HLT/Genesis/context.
- [ ] Different Workstreams cannot parent/supersede/activate each other’s objects.
- [ ] Unbound queries return no canonical payload.
- [ ] Ambiguous legacy data is quarantined rather than assigned.

### 19.2 Sessions and concurrency

- [ ] Two Sessions attached to one Workstream share canonical cognition while retaining distinct runtime state.
- [ ] One Session attached to several Workstreams must name the target for each mutation.
- [ ] Two concurrent Work Loops maintain independent tasks, budgets, blockers, transport, writer leases, and temporal state.
- [ ] One Workstream’s mutation cannot change another Workstream’s selected state.

### 19.3 Project switch and raw output

- [ ] Alternating Project A/B tool calls never inject foreign Trajectory, Workpoint, Focus State, or advisory context.
- [ ] Raw stdout/stderr remains clean; augmentation is separate and exactly scoped.
- [ ] Desktop/Pi selection changes every relevant projection atomically from the client’s perspective.

### 19.4 Compaction, model switch, and resume

- [ ] Compaction with several active Workstreams includes only the exact bound Workstream packet.
- [ ] Model/provider switch preserves exact Workstream and cannot adopt daemon-last identity.
- [ ] Pi resume, fork, clone, and Session-file recovery preserve native Session identity and exact Attachment.
- [ ] Stale compaction/recovery packets fail closed by event head/revision.
- [ ] Repeated compaction does not semantically launder foreign state.

### 19.5 Restart and migration

- [ ] Daemon restart restores each Workstream independently.
- [ ] One malformed projection cannot erase unrelated Workstreams.
- [ ] Legacy snapshot migration maps only provable records.
- [ ] Quarantine remains excluded after restart.
- [ ] Rollback restores executable authority without data loss or dual writers.
- [ ] Legacy global snapshot becomes noncanonical and then removable.

### 19.6 Remote and worktree topology

- [ ] Same path on two remote hosts resolves to distinct bindings unless shared identity is proven.
- [ ] Two worktrees under one project are explicit bindings and cannot exchange tactical execution state silently.
- [ ] Controller daemon can supervise multiple remote Project Scopes and Workstreams without local-path existence assumptions.
- [ ] Remote resume uses typed locator and host fingerprint.

### 19.7 Desktop / Mission Canvas

- [ ] Several Work Surfaces remain independently attributable, steerable, recoverable, and scope-safe.
- [ ] Visual focus does not pause or retarget background Workstreams.
- [ ] Aggregate Project/Global views cannot mutate without explicit target.
- [ ] Deep links open exact Workstream and reject ambiguous aliases.
- [ ] Workspace layout restoration does not restore cognition from client storage.

### 19.8 UIAI Engine / Cockpit

- [ ] Several browser contexts operate concurrently for different Workstreams without cookie, target, observation, mission, or Evidence crossover.
- [ ] Observation-bound action rejects Workstream mismatch.
- [ ] Browser-context restoration blocks incompatible Attachment binding.
- [ ] Screenshot/DOM/AX/network/recording Evidence is attributed to the exact Workstream, Workpoint, action, observation, and predicate.
- [ ] Pi → UIAI → Pi fix/verify loop preserves one Workstream throughout.
- [ ] Cockpit visual focus is not canonical mission authority.

### 19.9 Sync and conflict

- [ ] Same-Workstream compatible replicated events converge deterministically.
- [ ] Same-Workstream conflicting decisions create visible resolution records.
- [ ] Different-Workstream events never reconcile into one object.
- [ ] Unverified remote records remain observations/quarantine.

### 19.10 Export and portability

- [ ] Workstream export is independently inspectable and replayable.
- [ ] Export excludes unrelated Workstream data by construction.
- [ ] Import maps Scope/Workstream explicitly and preserves conflicts/quarantine.
- [ ] Provider-opaque state is preserved as a referenced accelerator, not the sole canonical continuation.

### 19.11 Incident regression

- [ ] Re-run and close the foundation intersections of #89, #98, #103, #109, #112, and #118 without bypassing their original evidence.

---

## 20. Implementation decomposition

The parent implementation epic shall be derived from Spec 158 only after separate operator authorization admits implementation into a post-release Trajectory and Beads plan.

Minimum tranches:

```text
158.0  Current-head inventory, baseline reproduction, freeze guards
158.1  Canonical identity: ScopeId, WorkstreamId, WorkspaceBindingId
158.2  Scoped event, projection, snapshot, migration, and quarantine substrate
158.3  Workpoint and Trajectory shadow/cutover
158.4  Focus Stack and Focus State shadow/cutover
158.5  Work Loop, writer leases, Silent Sessions, and runner
158.6  Context, memory, ontology, evidence, receipts, and artifacts
158.7  API, CLI, MCP, OpenAPI, schemas, and generated clients
158.8  Pi attachment, recovery, compaction, and raw-output cutover
158.9  Focusa Desktop, Menubar, TUI, Mission Canvas, Work Rail, deep links
158.10 UIAI Engine/Cockpit Workstream Attachment and browser isolation alignment
158.11 Sync, CRDT, PRE, remote, worktree, and multi-device reconciliation
158.12 Live legacy-data migration, quarantine review, and rollback drill
158.13 Adversarial proof, incident regression, and performance comparison
158.14 Legacy global authority removal and documentation/public-claim alignment
```

Every tranche requires:

- implementation work item;
- migration/compatibility work item;
- automated tests;
- live proof;
- rollback or reversibility proof;
- evidence classification;
- explicit closure check against Spec 107/116/119.

No tranche may close because another tranche’s proof looks similar.

---

## 21. Closure policy

Spec 158 and issue #125 MUST remain open while any of the following is true:

- canonical cognition still depends on one global `FocusaState` snapshot;
- a canonical route can omit Workstream and fall back to global state;
- any prompt-visible path can select latest/last/prior foreign state;
- one global active frame, Workpoint, Trajectory, current task, or Work Loop remains authoritative;
- continuity ID remains the only durable Workstream identity;
- Session or cwd can implicitly select or create Workstream authority;
- legacy records are silently assigned rather than quarantined;
- Desktop or UIAI maintains parallel canonical cognition;
- model-visible multi-Workstream proof is missing;
- migration rollback has not been run against real data;
- global snapshot authority is merely labeled legacy but still used;
- a scoped side store exists but the global aggregate remains the source of truth;
- only static source checks prove closure;
- any required companion ledger is incomplete;
- incident regressions have not been rerun;
- public documentation claims stronger isolation than runtime proof supports.

Final closure requires:

1. operator approval of this specification;
2. complete decomposition and dependency graph;
3. exhaustive current/target ownership ledger;
4. canonical Scope + Workstream physical partitioning;
5. reducer enforcement of exact authority;
6. independently scoped events, snapshots, replay, export, and migration;
7. no canonical global fallback;
8. successful live migration, quarantine, rollback, and legacy removal;
9. full Desktop, Pi, UIAI, remote, worktree, compaction, resume, restart, sync, and export proof;
10. stable evidence receipts for every acceptance tranche;
11. an operator-approved final foundation closure record.

---

## 22. Required actions after separate implementation admission

Document approval alone does not authorize any action in this section. These actions begin only after separate operator authorization of the post-release implementation program.

1. Pin the exact current `main` SHA and produce the current-head ownership inventory.
2. Add CI guards forbidding new daemon-global cognitive fields and unscoped canonical routes.
3. Generate the Spec 158 supersession/integration matrix before implementation.
4. Create the parent/child Beads decomposition.
5. Record a live baseline reproduction of cross-project contamination and mixed snapshot state.
6. Define schema version, backup, shadow, cutover, rollback, and quarantine operations.
7. Start only with identity and persistence substrate; do not patch another symptom route as “the foundation fix.”

---

## 23. Final constitutional statement

Focusa’s differentiating promise is not merely that it remembers more than a provider Session. It is that it preserves intent, continuity, evidence, authority, recovery, and transfer **without allowing unrelated work to become one cognition**.

The canonical architecture is therefore:

```text
one governed daemon
many verified Scopes
many isolated Workstreams
many Sessions and Attachments
many Desktop Work Surfaces
many UIAI browser contexts
one explicit authority path per mutation
zero implicit cross-Workstream cognition
```

> **A Workstream is not a label attached to global state. It is the container in which canonical cognitive state exists.**
