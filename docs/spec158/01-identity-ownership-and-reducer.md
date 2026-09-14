# Spec 158 Companion 01 — Identity, Ownership, and Reducer Partitioning

**Status:** normative companion to Spec 158  
**Parent:** `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`

---

## 1. Core target types

```rust
pub enum ScopeRef {
    Project(ProjectRootKey),
    Host(HostScopeKey),
}

pub struct WorkstreamKey {
    pub scope: ScopeRef,
    pub workstream_id: WorkstreamId,
}

pub struct AttachmentKey {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub instance_id: InstanceId,
    pub session_id: SessionId,
    pub attachment_id: AttachmentId,
    pub workspace_binding_id: WorkspaceBindingId,
}
```

The target daemon root separates infrastructure from cognition:

```rust
pub struct DaemonState {
    pub infrastructure: DaemonInfrastructureState,
    pub scopes: ScopeRegistry,
    pub sessions: SessionRegistry,
    pub attachments: AttachmentRegistry,
}
```

Project state owns project-wide identity and explicitly shared information:

```rust
pub struct ProjectState {
    pub identity: ProjectIdentity,
    pub genesis: ProjectGenesis,
    pub project_hlt: Option<ProjectHlt>,
    pub desired_end_state: Option<DesiredEndState>,
    pub workspace_bindings: WorkspaceBindingRegistry,
    pub shared_context: ProjectSharedContext,
    pub workstreams: BTreeMap<WorkstreamId, WorkstreamState>,
    pub projection_version: ProjectionVersion,
}
```

Workstream state owns cognition:

```rust
pub struct WorkstreamState {
    pub key: WorkstreamKey,
    pub focus_stack: FocusStackState,
    pub focus_state: FocusState,
    pub workpoints: WorkpointRegistry,
    pub trajectory: WorkstreamTrajectoryState,
    pub work_loop: WorkLoopState,
    pub context: WorkstreamContextState,
    pub memory: WorkstreamMemoryState,
    pub evidence: WorkstreamEvidenceIndex,
    pub claims: WorkstreamClaimIndex,
    pub ontology: WorkstreamOntologyState,
    pub continuity: ContinuityRegistry,
    pub event_head: EventHead,
    pub projection_version: ProjectionVersion,
}
```

---

## 2. Per-Workstream invariants

Where the domain requires one-active semantics, they mean:

- at most one active Focus Frame per Workstream;
- at most one active Workpoint per Workstream;
- at most one active tactical Trajectory per Workstream;
- one Work Loop state machine per Workstream unless a later specification introduces a narrower partition.

They never mean one active object for the entire daemon.

The reducer SHALL reject:

- a Focus Frame parent from another Workstream;
- Workpoint supersession across Workstreams;
- a tactical Trajectory mutation targeting another Workstream;
- a Work Loop command using another Workstream’s active pointer;
- an attachment that resolves to a different Workstream than the event envelope;
- a command whose Workstream cannot be resolved exactly.

---

## 3. WorkstreamContext extraction

Every canonical-capable request SHALL resolve one context before reducer execution:

```rust
pub struct WorkstreamContext {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub attachment: Option<AttachmentKey>,
    pub workspace_binding_id: Option<WorkspaceBindingId>,
    pub actor: ActorRef,
    pub authority: AuthorityContext,
}
```

Resolution sources may include:

1. an explicit WorkstreamKey in the request;
2. an exact AttachmentKey;
3. a stable object reference that resolves uniquely to one Workstream;
4. a versioned compatibility mapping with provenance.

The resolver SHALL NOT use:

- daemon current project;
- daemon current Thread;
- last active Workstream;
- current UI selection;
- process CWD alone;
- ContinuityId alone;
- SessionId alone;
- similarity or nearest-candidate matching;
- first or only record in a global collection.

Ambiguity fails closed.

---

## 4. Reducer shape

The canonical reduction law is partitioned:

```rust
pub fn reduce_workstream(
    state: WorkstreamState,
    event: WorkstreamEvent,
) -> WorkstreamReductionResult
```

The daemon remains one service and may supervise many Workstreams. Logical single-writer discipline is enforced per Workstream through event ordering, revision checks, fencing tokens, idempotency and explicit conflict resolution.

Infrastructure events use separate reducers or state machines and must not acquire cognitive fields as convenience caches.

---

## 5. Thread retirement

The implementation SHALL inventory every use of:

```text
Thread
thread_id
current_thread_id
thread state
thread lifecycle
thread-scoped cache
thread-scoped persistence
```

Each use is classified:

```text
rename-with-proof
migrate-to-WorkstreamId
legacy-compatibility-only
historical metadata
remove
quarantine
not cognitive
```

A broad mechanical rename is forbidden. If one Thread record cannot be proven to represent one unique durable cognitive workspace, it cannot be promoted to a Workstream without migration evidence.

Compatibility APIs may accept legacy Thread input only when they resolve uniquely and return deprecation metadata. New outputs use `workstream_id`.

---

## 6. Subsystem placement

### Project-owned

- ProjectIdentity;
- Genesis/project constitution;
- project HLT and desired end state;
- workspace/host/worktree bindings;
- explicitly project-shared Context;
- project-level registries and aggregate read models.

### Workstream-owned

- Focus Stack and Focus State;
- Workpoints and active pointer;
- tactical Trajectory;
- Work Loop and writer authority;
- Workstream Context and memory;
- ontology working set;
- Evidence/claim/reference visibility and authority;
- Continuity lineage;
- event and projection head.

### Attachment/runtime

- Session and Instance binding;
- runtime transport state;
- terminal/browser process identity;
- temporary diagnostics;
- attachment-scoped caches;
- presentation/runtime handles.

### Daemon infrastructure

- health/resource state;
- peer/device registries;
- shared transports;
- update state;
- licensing transport state;
- explicitly non-cognitive telemetry.

---

## 7. Static audit queries

The implementation task graph SHALL include repository-wide searches for:

```text
current_thread_id
current_instance_id
active_id
active_workpoint_id
active_trajectory_id
FocusaState
snapshots(name = 'focusa')
default_workstream
unscoped_project_root
last_project
latest trajectory
last active
project_root + continuity_id
```

Every match receives an owner, classification, migration action, test and removal gate.
