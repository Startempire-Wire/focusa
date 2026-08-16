# Spec 155 — Focusa CallGraph Workflow and Flow Mesh Execution Integration

**Status:** Operator draft; normative architecture proposal; no implementation authority  
**Date:** 2026-08-03  
**Owner:** Focusa  
**Execution integration:** Flow Mesh  
**Interaction surface:** UIAI Engine Cockpit with Focusa Mission Canvas  
**Parent architecture:** Spec 103 Call Stack Architecture Blueprint  
**Companion architecture:** Flow Mesh product portability and UIAI integration draft (operator-local; non-normative)  
**Typed design:** `019fc871-24be-7942-b46e-443b972f0ab2`

---

## 1. Executive directive

Focusa SHALL own a first-class **CallGraph** primitive that applies traditional call-stack discipline to mission-aligned workflow and task graphs.

Focusa is canonical for:

- CallGraph definition and revision;
- Project/continuity/mission scope;
- Trajectory, MLG, STG, waypoint, gap, Workpoint, and frontier linkage;
- semantic CallFrames and CallEdges;
- semantic run, CallPath, frontier, waiting, failure, unwind, and settlement state;
- operator authority, approval, evidence, receipts, and acceptance;
- recovery, replay, and continuation.

Flow Mesh is the preferred tightly woven execution substrate for task-backed frames. It remains canonical for:

- its independent project spaces, tasks, dependencies, providers, and synchronization;
- bound task execution and attempts;
- connector effects;
- runtime retries, joins, cancellation, and compensation effects when authorized;
- immutable execution observations, receipts, and evidence handles.

UIAI Engine Cockpit is the flagship interaction shell. Focusa Mission Canvas supplies the canonical CallGraph workflow surface within Cockpit. UIAI renders and invokes typed operations; it does not own graph or execution truth.

---

## 2. Problem

Traditional task graphs capture dependencies but often lose the operational discipline provided by a software call stack:

- who called whom;
- why a child exists;
- what inputs were passed;
- what local context belongs to one invocation;
- what result returned;
- where execution is waiting;
- how errors propagate;
- where errors are caught;
- what unwinds or compensates;
- which active path produced an effect;
- how to reconstruct a useful stack trace.

Focusa already owns mission authority, Workpoint continuity, evidence, and recovery. Flow Mesh already owns broad task/provider execution. The missing primitive is a typed semantic bridge that gives graph execution call-stack-grade lineage and control without making either product duplicate the other.

---

## 3. Goals

1. Add CallGraph as a native Focusa workflow primitive.
2. Preserve Focusa's project/continuity and operator-authority boundaries.
3. Preserve Flow Mesh as a portable independent task product.
4. Bind Focusa frames to Flow Mesh tasks without copying canonical state.
5. Provide stack-like active paths through branching/concurrent graphs.
6. Make every input, return, effect, failure, retry, and unwind inspectable.
7. Integrate CallGraph into Trajectory, Workpoint, Mission Canvas, evidence, recovery, prediction, and context cognition.
8. Expose bounded progressive APIs, CLI commands, Pi tools, and generated contracts.
9. Render the integrated workflow in UIAI Cockpit without a parallel graph authority.
10. Support deterministic replay, outage independence, and migration.

---

## 4. Non-goals

CallGraph SHALL NOT:

- replace Focusa Trajectory or Workpoint;
- turn Focus State into a general task database;
- make Focusa a provider synchronization engine;
- make Flow Mesh authoritative for Focusa mission/workflow semantics;
- require Flow Mesh for human-only or direct-tool CallGraphs;
- classify every Flow Mesh TaskGraph as a Focusa CallGraph;
- allow Cockpit caches to grant mutation authority;
- infer ownership from matching titles or paths;
- hide fallback, retries, skipped work, or compensation;
- treat configured providers as operational;
- erase attempts or mutate historical graph revisions.

---

## 5. Ontological placement

CallGraph participates in the Focusa chain:

`Project → HLT → MLG → STG → waypoint → gap → Workpoint → CallGraph slice → frontier → FrameInvocation → evidence → settlement`

Rules:

1. A CallGraph belongs to exactly one typed Focusa project/continuity scope.
2. A CallGraph definition references one mission and may reference a Trajectory revision.
3. A Workpoint authorizes or observes one bounded CallGraph slice.
4. The CallGraph frontier refines—not replaces—the Workpoint next action.
5. A frame invocation may bind to a Flow Mesh task execution, human action, agent operation, tool call, approval, timer, join, or subgraph.
6. Evidence settles frame returns and Workpoint acceptance through Focusa authority.
7. A graph run may outlive one temporal Pi session but not escape its continuity scope.

---

## 6. Call Stack architecture mapping

Spec 103 defines:

`entry → handlers → services → adapters → storage → output`

CallGraph applies that architecture as follows:

| Spec 103 layer | CallGraph realization |
|---|---|
| Entry | Entry frame(s), run request, mission/Workpoint authority |
| Handlers | Validation, preflight, control, execution-observation handlers |
| Services | Reducer, eligibility engine, CallPath/frontier, join/unwind services |
| Adapters | Flow Mesh, human, agent, tool, provider, timer, subgraph adapters |
| Storage | Definitions, revisions, events, snapshots, invocations, evidence refs |
| Output | Typed return, evidence, receipts, settlement, recovery guidance |

Traditional call-stack semantics map as follows:

| Call-stack concept | Focusa CallGraph concept |
|---|---|
| Function | CallFrame template |
| Stack frame | FrameInvocation |
| Caller/callee | Parent/child invocation through CallEdge |
| Arguments | Typed invocation inputs |
| Local variables | Invocation-local context reference |
| Program counter | Invocation phase/checkpoint |
| Return value | Typed semantic return |
| Exception | Structured failure |
| Try/catch | Failure boundary and catch policy |
| Stack trace | CallPath and immutable invocation lineage |
| Push | Invocation declared/activated |
| Pop | Invocation settled and returned |
| Unwind | Cancellation/compensation propagation |
| Recursion | Bounded frame/subgraph re-entry |
| Thread | Execution lane |
| Fork/join | Spawn edges and join barrier |
| Tail call | Continuation edge releasing the caller waiter |
| Timeout | Frame/edge temporal policy |

---

## 7. Vocabulary

### CallGraph

A versioned Focusa workflow definition containing frames, edges, policies, authority, evidence requirements, and execution bindings.

### CallFrame

A reusable semantic operation contract. It declares purpose, inputs, return schema, pre/postconditions, authority, side-effect class, acceptance, recovery, and optional execution binding.

### FrameInvocation

One runtime activation of one CallFrame in one graph run.

### CallEdge

A typed relationship controlling invocation, data mapping, waiting, joining, continuation, retry, or compensation.

### CallPath

An ordered active stack projection for one execution lane. A graph may have several concurrent CallPaths.

### Frontier

The bounded set of ready, running, waiting, blocked, or approval-required invocations relevant to the current Workpoint.

### Semantic return

Focusa's validated interpretation of an invocation result. A Flow Mesh success observation is evidence toward a semantic return; it is not automatically the return.

### TaskGraph

A Flow Mesh-owned task/dependency graph. A TaskGraph becomes a CallGraph execution substrate only through an explicit typed binding.

### Execution binding

A versioned mapping from a Focusa frame/invocation to Flow Mesh task definitions, executions, attempts, inputs, outputs, effects, and receipts.

---

## 8. Identity and scope

Every canonical object MUST carry:

- `project_root`;
- `continuity_id`;
- `graph_id`;
- `graph_revision` where applicable;
- `run_id` where applicable;
- `frame_id` and `invocation_id` where applicable;
- schema/reducer version;
- causation and correlation IDs;
- observed/effective timestamps;
- authority/evidence references.

Flow Mesh-bound objects additionally carry:

- `flowmesh_endpoint_id`;
- `flowmesh_workspace_id`;
- `binding_id` and revision;
- Flow Mesh project-space/work-item/execution/attempt identifiers.

Titles, names, URLs, repository remotes, and filesystem fingerprints are advisory—not identity.

Scope mismatch fails closed.

---

## 9. Definition schema

```ts
interface FocusaCallGraphDefinition {
  schema: "focusa.callgraph.v1";
  graph_id: string;
  revision: number;
  scope: {
    project_root: string;
    continuity_id: string;
  };
  mission_ref: string;
  trajectory_ref?: string;
  workpoint_refs: string[];
  title: string;
  description: string;
  entry_frame_ids: string[];
  frames: FocusaCallFrame[];
  edges: FocusaCallEdge[];
  policies: FocusaCallGraphPolicies;
  required_evidence: EvidenceRequirement[];
  created_at: string;
  created_by: AuthorityRef;
  supersedes_revision?: number;
}

interface FocusaCallFrame {
  frame_id: string;
  name: string;
  purpose: string;
  kind:
    | "human"
    | "agent"
    | "tool"
    | "approval"
    | "timer"
    | "join"
    | "subgraph"
    | "flowmesh_task";
  input_schema: JsonSchemaRef;
  return_schema: JsonSchemaRef;
  preconditions: PredicateRef[];
  postconditions: PredicateRef[];
  side_effect_class:
    | "none"
    | "local"
    | "external"
    | "destructive"
    | "financial"
    | "security";
  capability_refs: string[];
  authority_requirement?: AuthorityRequirement;
  timeout_policy?: TimeoutPolicy;
  retry_policy?: RetryPolicy;
  failure_boundary?: FailureBoundary;
  compensation_frame_id?: string;
  resource_budget?: ResourceBudget;
  acceptance: AcceptanceContract;
  execution_binding?: ExecutionBindingRef;
}

interface FocusaCallEdge {
  edge_id: string;
  from_frame_id: string;
  to_frame_id: string;
  kind:
    | "call"
    | "spawn"
    | "await"
    | "join"
    | "continue"
    | "condition"
    | "retry"
    | "catch"
    | "compensate";
  condition?: PredicateRef;
  input_mapping?: DataMapping[];
  return_mapping?: DataMapping[];
  join_policy?: JoinPolicy;
  cycle_policy?: CyclePolicy;
  authority_requirement?: AuthorityRequirement;
}
```

---

## 10. Runtime schema

```ts
interface FocusaCallGraphRun {
  schema: "focusa.callgraph_run.v1";
  run_id: string;
  graph_id: string;
  graph_revision: number;
  scope: { project_root: string; continuity_id: string };
  mission_ref: string;
  active_workpoint_id?: string;
  status: CallGraphRunStatus;
  root_invocation_ids: string[];
  frontier_invocation_ids: string[];
  active_paths: FocusaCallPath[];
  started_at: string;
  settled_at?: string;
  authority_refs: string[];
  evidence_refs: string[];
}

interface FocusaFrameInvocation {
  invocation_id: string;
  run_id: string;
  frame_id: string;
  parent_invocation_id?: string;
  caller_edge_id?: string;
  lane_id: string;
  depth: number;
  attempt: number;
  state: FrameState;
  input_ref: string;
  local_context_ref?: string;
  return_ref?: string;
  failure?: StructuredFailure;
  execution_binding_ref?: string;
  execution_observation_refs: string[];
  effect_receipt_refs: string[];
  evidence_refs: string[];
  idempotency_key: string;
  created_at: string;
  started_at?: string;
  settled_at?: string;
}

interface FocusaCallPath {
  path_id: string;
  lane_id: string;
  invocation_ids: string[];
  waiting_on: string[];
  state: "ready" | "running" | "waiting" | "unwinding" | "settled";
}
```

---

## 11. State machines

### 11.1 Definition

`draft → validated → proposed → approved → active → superseded | retired`

Only an approved active revision can start a canonical run.

### 11.2 Run

`draft → validated → preflighted → awaiting_approval → queued → running → waiting | degraded → succeeded | failed | cancelled | compensated`

### 11.3 Frame invocation

`declared → eligible → ready → dispatched → running → waiting → returned → accepted`

Terminal/exception states:

`failed | rejected | cancelled | skipped | compensated | quarantined`

Rules:

1. `eligible` requires graph, data, scope, authority, and prerequisite satisfaction.
2. `ready` requires resource/admission readiness.
3. `dispatched` records the selected adapter and idempotency key before effects.
4. `running` requires adapter acknowledgment or human acceptance.
5. `waiting` names an exact child, approval, timer, join, connector, or external dependency.
6. `returned` requires a typed result or structured failure observation.
7. `accepted` requires postconditions and acceptance evidence.
8. Flow Mesh may report execution success, failure, cancellation, or effect receipts; Focusa alone settles `returned`/`accepted` semantic state.
9. `skipped` is never implicit success.
10. `compensated` is a new invocation/evidence chain, not deletion.

---

## 12. Eligibility and dispatch algorithm

For each candidate invocation, Focusa deterministically evaluates:

1. scope and graph revision;
2. caller and edge settlement;
3. required input availability/schema;
4. preconditions and condition edges;
5. cycle/depth/iteration bounds;
6. Workpoint/frontier alignment;
7. required capability availability;
8. adapter health/freshness;
9. operator/authority requirements;
10. temporal and resource budgets;
11. idempotency and prior effect receipts;
12. join/barrier policy.

The reducer emits one reasoned disposition:

- `eligible`;
- `waiting_input`;
- `waiting_parent`;
- `waiting_join`;
- `waiting_authority`;
- `waiting_capability`;
- `blocked_scope`;
- `blocked_stale`;
- `blocked_budget`;
- `blocked_cycle_policy`;
- `rejected`.

No adapter call occurs before a dispatch event is durably committed.

---

## 13. Flow Mesh binding

### 13.1 Binding schema

```ts
interface FlowMeshExecutionBinding {
  schema: "focusa.flowmesh_execution_binding.v1";
  binding_id: string;
  revision: number;
  scope: { project_root: string; continuity_id: string };
  graph_id: string;
  graph_revision: number;
  frame_id: string;
  flowmesh_endpoint_id: string;
  flowmesh_workspace_id: string;
  project_space_ref?: string;
  task_template_ref?: string;
  input_mapping: DataMapping[];
  return_mapping: DataMapping[];
  effect_policy: EffectPolicy;
  retry_delegation: "focusa" | "flowmesh_bounded";
  join_delegation: "focusa" | "flowmesh_runtime";
  compensation_binding_ref?: string;
  required_health: HealthRequirement[];
}
```

### 13.2 Ownership protocol

1. Focusa validates semantic eligibility.
2. Focusa preflights the Flow Mesh binding.
3. Flow Mesh returns endpoint, task, provider, health, side-effect, and receipt expectations.
4. Focusa obtains required authority.
5. Focusa emits a durable dispatch event.
6. Flow Mesh creates or invokes the bound task with the idempotency key.
7. Flow Mesh owns task execution and connector effects.
8. Flow Mesh emits immutable observations/receipts.
9. Focusa verifies scope, mapping, freshness, and postconditions.
10. Focusa records the semantic return and acceptance result.

### 13.3 Execution observation

```ts
interface FlowMeshExecutionObservation {
  schema: "focusa.flowmesh_execution_observation.v1";
  scope: { project_root: string; continuity_id: string };
  binding_id: string;
  graph_id: string;
  graph_revision: number;
  run_id: string;
  frame_id: string;
  invocation_id: string;
  execution_id: string;
  attempt_id: string;
  task_ref: string;
  status: "accepted" | "running" | "waiting" | "succeeded" | "failed" | "cancelled" | "compensated";
  return_ref?: string;
  failure?: StructuredFailure;
  effect_receipt_refs: string[];
  evidence_refs: string[];
  observed_at: string;
  source_event_cursor: string;
}
```

### 13.4 Standalone Flow Mesh

Flow Mesh may continue to create and execute standalone TaskGraphs. Such graphs:

- are not Focusa CallGraphs;
- do not gain Focusa mission/authority claims implicitly;
- can later be imported as inactive CallGraph candidates;
- require operator review of scope, edges, side effects, and acceptance before activation.

---

## 14. Workpoint and Trajectory integration

### 14.1 Workpoint

A Workpoint CallGraph link contains:

- `graph_id` and revision;
- `run_id`;
- selected frame/invocation slice;
- current frontier;
- exact next semantic action;
- evidence requirements;
- Flow Mesh binding/health summary where relevant;
- drift boundaries.

A Workpoint checkpoint captures these references without embedding entire graphs or logs.

### 14.2 Trajectory

Trajectory assessment may:

- propose a CallGraph candidate for a verified gap;
- assess progress using accepted frame returns;
- detect graph drift from HLT/MLG/STG;
- recommend a new Workpoint at the frontier.

Trajectory does not mutate the graph or schedule tasks.

### 14.3 Context cognition

Context cognition selects bounded graph/frame/task/evidence slices under budget. Full definitions, histories, and provider payloads are cold surfaces with rehydrate references.

### 14.4 Prediction and metacognition

Predictions can estimate:

- eligibility;
- adapter success;
- join completion;
- retry value;
- stale-state risk;
- compensation success;
- operator-intervention likelihood.

Predictions guide but never grant authority.

---

## 15. Concurrency, joins, and recursion

### 15.1 Lanes

Each concurrent branch has a lane and CallPath. Parent/child lineage remains explicit across lanes.

### 15.2 Join policies

- `all_success`;
- `all_settled`;
- `any_success`;
- `quorum`;
- `reduce` with versioned deterministic reducer;
- `manual` with operator receipt.

Every join declares ordering, cancellation, partial-return, timeout, and failure behavior.

### 15.3 Cycles and recursion

Cycles are forbidden unless a `CyclePolicy` declares:

- maximum depth;
- maximum iterations;
- total duration;
- resource/effect budget;
- termination predicate;
- repeated-effect policy;
- unwind behavior.

Unbounded recursive task creation fails validation.

---

## 16. Failure, catch, retry, and unwind

Structured failure classes:

- validation;
- scope/authority;
- capability unavailable;
- connector authentication/authorization;
- rate limit;
- timeout;
- resource budget;
- execution;
- return mapping;
- postcondition/acceptance;
- compensation;
- stale dependency;
- operator rejection.

Failure routes:

- retry under Focusa policy;
- bounded Flow Mesh runtime retry delegated by binding;
- alternate execution binding;
- catch frame;
- pause for operator;
- compensate then fail;
- cancel descendants;
- continue with explicit degraded return;
- abort run.

Silent fallback is prohibited. Every retry is a new attempt. Every effect retains its receipt.

---

## 17. Evidence and authority

Each frame declares:

- required authority;
- required evidence before dispatch;
- expected effect receipts;
- return evidence;
- acceptance checks;
- recovery evidence.

Side-effect policy:

| Class | Default authority |
|---|---|
| None/read | automatic when scope/currentness passes |
| Local reversible | preview and idempotency |
| External mutation | explicit confirmation or durable delegated authority |
| Destructive | explicit operator confirmation and rollback evidence |
| Financial | explicit operator confirmation and amount/scope evidence |
| Security/auth | explicit confirmation, SecretRef boundary, rotation/rollback |

Flow Mesh provider annotations are untrusted hints until Focusa policy validates them.

---

## 18. Persistence, events, and replay

### 18.1 Persistence

Focusa persists:

- definitions/revisions;
- semantic runs/invocations/CallPaths/frontier;
- authority/evidence/settlement references;
- execution-binding definitions;
- normalized execution observations;
- snapshots and reducer checkpoints.

Flow Mesh persists its own task/execution/provider events. Focusa stores references and normalized observations, not a duplicate unbounded log.

### 18.2 Event schema

Each `FocusaCallGraphEvent` contains:

- event/sequence IDs;
- project/continuity scope;
- graph/revision/run/frame/invocation/attempt IDs;
- prior/next state;
- actor and authority reference;
- causation/correlation IDs;
- bounded input/return/evidence handles;
- binding/execution observation references;
- side-effect/receipt references;
- observed/effective timestamps;
- schema/reducer versions.

### 18.3 Replay

Replay MUST reproduce semantic scheduling and state when given the same:

- graph revision;
- ordered Focusa events;
- Flow Mesh observations;
- authority decisions;
- external evidence;
- reducer version.

Replay never re-executes effects by default.

---

## 19. Focusa API, CLI, and Pi tools

### 19.1 Routes

- `GET /v1/callgraphs`
- `POST /v1/callgraphs/validate`
- `POST /v1/callgraphs/preview`
- `POST /v1/callgraphs`
- `GET /v1/callgraphs/{graph_id}/revisions/{revision}`
- `POST /v1/callgraphs/{graph_id}/runs/preflight`
- `POST /v1/callgraphs/{graph_id}/runs`
- `GET /v1/callgraph-runs/{run_id}`
- `GET /v1/callgraph-runs/{run_id}/paths`
- `GET /v1/callgraph-runs/{run_id}/frontier`
- `GET /v1/callgraph-runs/{run_id}/events`
- `POST /v1/callgraph-runs/{run_id}/control`
- `POST /v1/callgraph-runs/{run_id}/flowmesh-bindings/preflight`
- `POST /v1/callgraph-runs/{run_id}/flowmesh-bindings/execute`
- `POST /v1/callgraph-runs/{run_id}/evidence/link`

### 19.2 Pi tools

- `focusa_callgraph_search`
- `focusa_callgraph_describe`
- `focusa_callgraph_validate`
- `focusa_callgraph_preview`
- `focusa_callgraph_preflight`
- `focusa_callgraph_run`
- `focusa_callgraph_observe`
- `focusa_callgraph_control`
- `focusa_callgraph_flowmesh_binding_preflight`
- `focusa_callgraph_flowmesh_dispatch`
- `focusa_callgraph_link_evidence`

Progressive discovery and bounded traversal are mandatory. Mutation tools require confirmation/idempotency/receipts according to side-effect class.

---

## 20. Mission Canvas and UIAI Cockpit

### 20.1 Mission Canvas

Mission Canvas is the Focusa-native CallGraph workflow surface. It displays:

- mission/Trajectory/Workpoint linkage;
- graph/revision/run;
- active CallPaths;
- frontier and eligibility reasons;
- caller/callee lineage;
- waiting approvals/joins/connectors;
- Flow Mesh binding/task/attempt summaries;
- return/evidence/acceptance state;
- stale/degraded/conflicted/quarantined states;
- exact recovery action.

Graph definition editing and semantic workflow operations are Focusa operations.

### 20.2 UIAI Cockpit

Cockpit composes two typed planes:

- Focusa plane: CallGraph semantics, authority, Workpoint, evidence, control;
- Flow Mesh plane: bound task/execution/provider detail.

Cockpit joins the planes only by typed scope/binding/invocation IDs. It cannot infer links by title.

Cockpit operations:

- read/inspect immediately when authorized;
- preview local reversible changes;
- confirm external mutations;
- route destructive/financial/security operations through Focusa authority;
- display receipt/evidence/rollback after execution.

### 20.3 Offline behavior

Cockpit may cache bounded projections. Cached state:

- is labeled with source/freshness;
- cannot grant authority;
- cannot dispatch unapproved work;
- reconciles by revision/event cursor after reconnect.

---

## 21. Security and tenancy

- Every object is tenant/workspace/project/continuity scoped.
- CallGraph definitions cannot reference cross-scope frames without explicit bridge authority.
- Flow Mesh endpoints use paired/scoped credentials.
- Connector secrets remain Flow Mesh SecretRefs; Focusa stores presence/reference only.
- Cockpit credentials remain in OS keychain.
- Payload sizes/depths are bounded.
- List/search endpoints never return prompt/tool-output blobs by default.
- Webhook/event observations are signed or Tailscale/mTLS authenticated and replay-protected.
- Operator confirmation cannot be synthesized by an adapter.
- Audit events are append-only and tamper-evident.
- Revocation blocks new dispatch and marks affected waiting frames degraded.

---

## 22. Performance and resource control

Hot paths:

- current Workpoint CallGraph slice;
- active paths/frontier;
- bounded recent events;
- binding health summary.

Cold paths:

- full graph history;
- all attempts;
- full provider payloads;
- replay diagnostics;
- cross-run analytics.

Required controls:

- cursor pagination;
- field projection;
- token/byte budgets;
- event compaction through verified snapshots;
- backpressure between Focusa and Flow Mesh;
- bounded fan-out and recursion;
- scheduler/resource budgets;
- no loss of audit/evidence authority under LowMem mode.

---

## 23. Degraded and outage behavior

### Flow Mesh unavailable

- Focusa keeps graph/workflow truth.
- Bound frames become `waiting_capability` or degraded.
- Human/direct-tool frames may proceed if independent.
- No execution success is inferred.

### Focusa unavailable

- Flow Mesh completes already dispatched work under the authority envelope.
- No new approval-required frame is dispatched.
- Observations spool durably for later intake.
- Standalone Flow Mesh graphs remain independently governed by Flow Mesh policy.

### UIAI unavailable

- Focusa and Flow Mesh remain operable headlessly.
- CLI/Pi/API recovery remains available.

### Stale or conflicting observations

- Focusa quarantines the observation.
- The semantic frame does not settle.
- Recovery identifies exact source/binding/cursor mismatch.

---

## 24. Migration

1. Introduce CallGraph schemas and read-only validation in Focusa.
2. Add definitions/revisions/events/snapshots without changing Workpoint semantics.
3. Add inactive import of existing task dependencies as candidate frames/edges.
4. Add Flow Mesh endpoint identity and binding validation.
5. Add read-only execution observation intake.
6. Add one preflight-only dispatch path.
7. Pilot one non-destructive graph.
8. Add approval/evidence settlement.
9. Add concurrency/join/retry/compensation incrementally.
10. Add Mission Canvas and Cockpit composition.

Migration cannot activate imported graphs automatically.

---

## 25. Verification

### Unit/property tests

- referential integrity/reachability;
- schema mappings;
- state-machine legality;
- deterministic eligibility;
- cycle/recursion bounds;
- join determinism;
- retry/idempotency;
- cancellation/unwind;
- compensation ordering;
- scope isolation;
- evidence/authority gates.

### Integration tests

- Focusa definition → Flow Mesh binding → task execution → observation → semantic return;
- duplicate/reordered observation intake;
- stale endpoint/binding revision;
- credential expiry/revocation;
- Flow Mesh retry delegated vs Focusa retry;
- Workpoint checkpoint/resume;
- Mission Canvas/Cockpit projection parity;
- independent product restarts.

### Adversarial tests

- unbounded cycle;
- forged execution success;
- cross-project IDs;
- stale approval;
- replayed effect receipt;
- connector lies about side effects;
- partial join failure;
- compensation failure;
- oversized/deep payload;
- UI cache mutation attempt.

### End-to-end proof

A proof graph must demonstrate:

1. Focusa mission/Workpoint binding;
2. CallGraph preview and approval;
3. human, agent/tool, Flow Mesh task, fork, join, and approval frames;
4. typed arguments and returns;
5. visible active CallPaths;
6. Flow Mesh execution observations;
7. evidence settlement;
8. failure/catch/retry;
9. cancellation/unwind/compensation;
10. restart/replay equivalence;
11. headless and Cockpit parity.

---

## 26. Granular decomposition

### Native model

- **CG-001** Add glossary and ontology registry entries.
- **CG-002** Define graph/revision JSON Schemas.
- **CG-003** Define frame/edge schemas.
- **CG-004** Define run/invocation/CallPath/frontier schemas.
- **CG-005** Define structured failure and semantic return.
- **CG-006** Define authority/evidence contracts.
- **CG-007** Define binding/observation contracts.
- **CG-008** Generate language bindings.

### Validation/reducer

- **CG-009** Implement scope/revision validator.
- **CG-010** Implement referential/reachability validator.
- **CG-011** Implement input/return mapping validator.
- **CG-012** Implement cycle/recursion validator.
- **CG-013** Implement state transition reducer.
- **CG-014** Implement eligibility reasons.
- **CG-015** Implement CallPath projection.
- **CG-016** Implement frontier projection.
- **CG-017** Implement join policies.
- **CG-018** Implement failure/catch routing.
- **CG-019** Implement unwind/compensation semantics.
- **CG-020** Implement replay equivalence.

### Persistence/runtime

- **CG-021** Add definition/revision persistence.
- **CG-022** Add run/invocation persistence.
- **CG-023** Add append-only events.
- **CG-024** Add snapshots/checkpoints.
- **CG-025** Add idempotency/effect receipts.
- **CG-026** Add resource/temporal budgets.
- **CG-027** Add bounded traversal.
- **CG-028** Add recovery/doctor projections.

### Flow Mesh binding

- **CG-029** Add endpoint identity/pairing.
- **CG-030** Add binding validate/preview.
- **CG-031** Add task/work-item mapping.
- **CG-032** Add authority envelope.
- **CG-033** Add preflight dispatch.
- **CG-034** Add idempotent execution dispatch.
- **CG-035** Add observation normalization.
- **CG-036** Add receipt/evidence intake.
- **CG-037** Add stale/conflict quarantine.
- **CG-038** Add spool/replay during Focusa outage.
- **CG-039** Add retry/compensation delegation.
- **CG-040** Verify standalone Flow Mesh independence.

### Focusa workflow integration

- **CG-041** Link Project/Trajectory/mission.
- **CG-042** Link Workpoint/current frontier.
- **CG-043** Add checkpoint/resume fields.
- **CG-044** Add context cognition selection.
- **CG-045** Add prediction types/outcome evaluation.
- **CG-046** Add evidence/acceptance settlement.
- **CG-047** Add authority/confirmation receipts.
- **CG-048** Add LowMem/bounded projections.

### API/CLI/Pi

- **CG-049** Add read/search/describe APIs.
- **CG-050** Add validate/preview APIs.
- **CG-051** Add run preflight/start APIs.
- **CG-052** Add observe/path/frontier APIs.
- **CG-053** Add control/evidence APIs.
- **CG-054** Add CLI parity.
- **CG-055** Add progressive Pi tools.
- **CG-056** Add capability catalog/docs/skills.
- **CG-057** Add strict error/recovery envelopes.

### Mission Canvas/UIAI

- **CG-058** Add Mission Canvas graph summary.
- **CG-059** Add CallPath/frontier view.
- **CG-060** Add frame/return/evidence inspector.
- **CG-061** Add Flow Mesh execution detail projection.
- **CG-062** Add graph diff/compatibility view.
- **CG-063** Add confirmed control operations.
- **CG-064** Add Cockpit composite adapter.
- **CG-065** Add offline/freshness behavior.
- **CG-066** Add keyboard/screen-reader/visual regression proof.

### Release proof

- **CG-067** Add migration dry-run/rollback.
- **CG-068** Add deterministic stress/property suite.
- **CG-069** Add outage/recovery suite.
- **CG-070** Add security/tenant suite.
- **CG-071** Add end-to-end proof graph.
- **CG-072** Add docs/examples/gallery.
- **CG-073** Add release/rollback receipts.

---

## 27. Acceptance criteria

CallGraph is accepted when:

- Focusa is the unambiguous canonical owner;
- Flow Mesh execution binding is typed, idempotent, and independently recoverable;
- standalone Flow Mesh TaskGraphs remain valid and distinct;
- active CallPaths/frontier are deterministic;
- failures/retries/unwind/compensation preserve lineage;
- Workpoint/Trajectory/evidence integration is complete;
- UIAI Cockpit and Mission Canvas render the same canonical state;
- all mutations pass authority/preflight/receipt gates;
- replay is equivalent after restart;
- outage independence passes;
- security, accessibility, performance, and migration gates pass.

---

## 28. Not-done conditions

CallGraph is not done if:

- Flow Mesh can mutate Focusa semantic frame state directly;
- Focusa fabricates Flow Mesh execution success;
- Cockpit grants authority from cached state;
- task titles/paths are used as identity;
- cycles lack explicit bounds;
- retries erase attempts or duplicate effects;
- semantic success lacks acceptance evidence;
- skipped/degraded work appears successful;
- compensation removes history;
- Workpoint and CallGraph frontier disagree silently;
- full graphs/logs are injected into hot prompts by default;
- one product outage corrupts another;
- imported TaskGraphs activate without operator review.

---

## 29. Operator decisions

1. Approve the exact CallGraph ontology placement in the Focusa chain.
2. Approve whether graph definitions are immutable after activation or permit constrained amendment revisions only.
3. Approve default retry ownership: Focusa versus bounded Flow Mesh delegation.
4. Approve default join and degraded-return policies.
5. Approve standalone Flow Mesh TaskGraph import semantics.
6. Approve the first non-destructive pilot graph.
7. Approve Mission Canvas graph-authoring depth for initial release.
8. Approve whether CallGraph ships in the next Focusa release or behind an experimental entitlement.

No implementation decomposition becomes active until these decisions and the official documentation sweep are approved.
