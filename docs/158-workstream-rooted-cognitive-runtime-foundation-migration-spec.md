# Spec 158 — Workstream-Rooted Cognitive Runtime, Canonical State Partitioning, and Foundation Migration

**Status:** draft normative foundation specification  
**Priority:** P0 foundation  
**Date:** 2026-08-04  
**Coordination issue:** `Startempire-Wire/focusa#125`  
**Primary repository:** `Startempire-Wire/focusa`  
**Integrates/supersedes where conflicting:** Specs 39, 40, 41, 98, 99, 104, 116, 133, 135 and all authority-bearing runtime surfaces

---

## 0. Decision

Focusa SHALL support one daemon serving many isolated Projects and Workstreams without storing their cognition inside one daemon-global aggregate.

The canonical durable cognitive boundary is **Workstream**, not Thread, Session, Instance, Continuity, active UI selection, process CWD, last-used project, or daemon-global state.

> **No canonical cognitive object exists outside an exact Scope + Workstream.**

The reducer remains the sole canonical mutation boundary:

> Models, tools, clients, agents, UIs, adapters and external runtimes propose. Only the Focusa reducer canonizes meaning.

The reducer remains deterministic, replayable, side-effect-free and logically single-writer per Workstream.

The correction is not complete until daemon-global cognitive singleton authority has been removed from core state, reducer routing, persistence, replay, API resolution, caches, Pi augmentation, Desktop control and every other authority-bearing consumer.

---

## 1. Normative document set

This master document and the following companion documents form one required specification:

1. [`docs/spec158/01-identity-ownership-and-reducer.md`](spec158/01-identity-ownership-and-reducer.md)
2. [`docs/spec158/02-persistence-migration-and-quarantine.md`](spec158/02-persistence-migration-and-quarantine.md)
3. [`docs/spec158/03-client-runtime-and-desktop-contracts.md`](spec158/03-client-runtime-and-desktop-contracts.md)
4. [`docs/spec158/04-implementation-task-graph-and-closure.md`](spec158/04-implementation-task-graph-and-closure.md)

The active Mission Canvas/Desktop migration is governed by:

- [`docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`](transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md)
- [`docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`](transitions/FOCUSA-TRANSITION-001-task-graph.yaml)

---

## 2. Canonical ownership graph

```text
Focusa Daemon
│
├── Daemon Infrastructure
│   ├── health and resource state
│   ├── peer and device registries
│   ├── shared transport clients
│   ├── licensing/update transport state
│   └── explicitly non-cognitive telemetry
│
├── Scope Registry
│   ├── Project Scope: ProjectRootKey
│   │   ├── ProjectIdentity
│   │   ├── Genesis / project constitution
│   │   ├── project HLT and desired end state
│   │   ├── workspace/host/worktree bindings
│   │   ├── explicitly project-shared Context
│   │   └── Workstreams
│   │       └── WorkstreamId
│   │           ├── Focus Stack and Focus State
│   │           ├── Workpoints
│   │           ├── tactical Workstream Trajectory
│   │           ├── Work Loop and writer authority
│   │           ├── scoped Context and memory
│   │           ├── Evidence, claims and references
│   │           ├── ontology working set
│   │           └── event and projection head
│   └── Explicit Host Scope
│       └── explicitly host-scoped Workstreams only
│
├── Session Registry
└── Attachment Registry
```

The daemon root SHALL NOT own canonical global fields equivalent to:

```text
session
focus_stack
focus_state
active_focus_frame
workpoint
active_workpoint
trajectory
active_trajectory
work_loop
current_thread_id
current_instance_id as a cognition selector
current_project
last_project
latest_workstream
```

---

## 3. Identity hierarchy

```text
ScopeRef / ProjectRootKey
  -> WorkstreamId
    -> ContinuityId
      -> AttachmentKey
        -> SessionId / InstanceId
          -> runtime object identity
            -> WorkSurfaceId
```

### 3.1 Workstream

`WorkstreamId` is the stable durable identity of an independent cognitive workspace.

```rust
pub struct WorkstreamKey {
    pub scope: ScopeRef,
    pub workstream_id: WorkstreamId,
}
```

### 3.2 Continuity

`ContinuityId` is continuation lineage or generation inside a Workstream. It is not Workstream identity.

A rollover, compaction generation, model switch or resume may preserve the Workstream while changing Continuity lineage.

### 3.3 Thread

Thread is historical design lineage and a legacy compatibility term.

New runtime code, schemas, routes, CLI output, Desktop manifests and documentation SHALL use Workstream.

A legacy `thread_id` may exist only as versioned migration input, deprecated compatibility metadata or immutable forensic history. It SHALL NOT remain a canonical owner, selector or permanent dual-authority key.

### 3.4 Session, Instance and Attachment

Session and Instance are temporal runtime identities. They do not own cognition.

Attachment binds runtime identity to one exact Workstream:

```rust
pub struct AttachmentKey {
    pub workstream: WorkstreamKey,
    pub continuity_id: Option<ContinuityId>,
    pub instance_id: InstanceId,
    pub session_id: SessionId,
    pub attachment_id: AttachmentId,
    pub workspace_binding_id: WorkspaceBindingId,
}
```

### 3.5 Work Surface

Work Surface is presentation identity. It does not grant mutation authority.

Every Work Surface SHALL carry or resolve an exact `WorkstreamKey` and, where attached, an exact `AttachmentKey`.

---

## 4. Reducer and routing law

The target reduction shape is Workstream-rooted:

```rust
reduce_workstream(
    state: WorkstreamState,
    event: WorkstreamEvent,
) -> WorkstreamReductionResult
```

A daemon ScopeRouter resolves the exact Workstream partition before reduction.

All canonical-capable events SHALL carry or resolve:

```text
ScopeRef
WorkstreamId
actor identity
causal/idempotency metadata
AttachmentKey where runtime attachment matters
```

No reducer/API path may derive canonical mutation authority from global active/current/latest state.

Cross-Workstream parentage, Workpoint supersession, Focus Frame relationships, tactical Trajectory mutation or Work Loop mutation are forbidden unless represented as an explicit provenance/reference relationship with no authority transfer.

---

## 5. Persistence and migration law

Canonical cognition SHALL move from the mixed global snapshot to Workstream-scoped event streams, snapshots and projections.

The existing global snapshot remains immutable forensic migration input only after cutover.

Migration SHALL:

1. inventory every authority-bearing field and store;
2. classify it as daemon infrastructure, project-shared, Workstream-owned, Attachment/runtime, derived read model, legacy compatibility or quarantine;
3. generate stable Workstream IDs through evidence-backed mappings;
4. preserve Thread mappings with provenance where unique;
5. quarantine ambiguous records rather than guessing;
6. shadow-materialize partitions for bounded parity comparison;
7. cut over one subsystem at a time;
8. disable global cognitive writes;
9. remove global reads and fallbacks;
10. remove singleton fields and global cognition snapshot authority.

Shadow comparison SHALL NOT become permanent dual canonical writes.

---

## 6. Required subsystem corrections

The migration includes, at minimum:

- Focus Stack and Focus State;
- Workpoints, active pointer, resume, supersession, drift and Evidence links;
- project HLT versus tactical Workstream Trajectory;
- Work Loop, budgets, retries, blockers, pause state, writer leases and fencing tokens;
- Sessions, Attachments, Silent Sessions and remote runners;
- Pi events, tools, commands, hooks, compaction, model switch and recovery;
- Context, memory, ontology, Evidence, claims and references;
- API, CLI, MCP, OpenAPI, schemas, generated clients and capability registries;
- Mission Canvas, Work Rails, menubar, Desktop and Focusa.work read models;
- sync, PRE, CRDT, multi-device, remote host and worktree topology;
- authority-sensitive caches, idempotency, telemetry, training, export and portability.

---

## 7. Client and UI law

CLI local selection, Pi current view, menubar selected card, Desktop active tab and Focusa.work route are presentation convenience only.

A canonical operation must carry `ScopeRef + WorkstreamId` or an exact Attachment that resolves to them.

Visual selection SHALL NOT alter canonical Workstream authority.

Aggregate dashboards may display many Workstreams. They remain provenance-labeled read models unless an explicit target is supplied for a governed command.

The GUI, CLI, agent tools and command palette SHALL share one semantic command graph and return operation receipts echoing the resolved WorkstreamKey.

---

## 8. Migration order

```text
0. inventory and freeze new global cognitive authority
1. introduce WorkstreamId, WorkstreamKey and workspace bindings
2. introduce ScopeRouter and WorkstreamContext extraction
3. add scoped event streams and snapshots
4. partition Workpoint and Trajectory; prove parity and cut over
5. partition Focus Stack and Focus State; prove parity and cut over
6. partition Work Loop, writer leases and Silent Sessions
7. partition Context, memory, ontology, Evidence, claims and references
8. cut over API, CLI, MCP, Pi, schemas, Desktop and UI consumers
9. migrate legacy Thread/Continuity records and quarantine ambiguity
10. disable global cognitive writes
11. remove global cognitive reads and fallbacks
12. remove singleton fields and global cognition snapshot authority
13. run closure proof, rollback rehearsal and public-claim audit
```

---

## 9. Closure conditions

Spec 158 is not complete until:

- no core daemon-global cognitive aggregate remains;
- no canonical `current_thread_id` or process-wide current Instance selects cognition;
- one-active semantics are proven per Workstream, not per daemon;
- per-Workstream replay is deterministic;
- ambiguous migration records remain quarantined;
- concurrent Workstreams cannot mutate or expose one another;
- Pi resume, compaction, model switch and Silent Session recovery retain exact Workstream identity;
- API, CLI, MCP, Pi and Desktop fail closed on ambiguous Workstream resolution;
- Work Surfaces carry Workstream identity;
- visual focus does not change authority;
- no permanent dual canonical write path remains;
- backup and rollback rehearsal passes;
- public claims of singleton elimination and multi-project isolation are updated only after end-to-end proof.

---

## 10. Immediate implementation rule

No new feature may add another daemon-global cognitive selector or use `project_root + continuity_id` as the complete permanent identity of canonical cognition.

Agents touching the existing Mission Canvas work SHALL first follow `FOCUSA-TRANSITION-001`, preserve the worktree, produce a migration ledger and correct identity before extracting shared code.
