# Spec 182 — Focusa Project Foreman: Workstream Intelligence Projection and Project-Responsible Agent Role

**Status:** DRAFT canonical architecture direction, 2026-09-05.  
**Canonical human architecture authority:** Verious Smith III under the repository architecture-authority policy.  
**Historical basis:** operator-supplied cross-account architecture handoff describing Project Foreman across prior design conversations. That handoff did not identify one current canonical standalone Foreman spec. This document is the current reconciliation and does not inherit stale proposal numbering.  
**Primitive owners preserved:** Spec 164 Workstream Root, Spec 72 Agent Identity/Role, Spec 135/135B C.R.I.S.T., Spec 140 Runtime Constitution, Spec 133 Silent Sessions, Spec 79 Work Loop, Spec 139 Presence/Placement, Spec 136 settlement, Spec 141 capabilities, Spec 151 Program Design Runtime, Spec 181 Voice/Conversation.

---

## 0. One-line definition

> **The Project Foreman is the persistent, Workstream-scoped projection of Focusa project intelligence through a project-responsible agent role. It is not a separate chatbot, model, session, memory store, or authority system.**

The Foreman gives one project/Workstream a continuous operating identity across Pi sessions, model switches, desktop/mobile/voice surfaces, worker turnover, daemon restart, and execution placement changes while all canonical state remains in the primitive that already owns it.

---

## 1. Why this specification exists

Focusa already contains the pieces of persistent project intelligence:

```text
Workstream Root
ProjectIdentity
C.R.I.S.T. Project Genesis
Runtime Constitution
Trajectory
Workpoint
Spec / Work Items
Evidence / Receipts
History / decisions / constraints
Context Cognition
Presence / environment
Sessions / Work Loop
Capabilities / permissions
promoted learning
```

Prior architecture discussions converged on a product identity called **Project Foreman**. The missing step was to state exactly what persists and what does not.

The Foreman MUST NOT become another agent between the human and Pi. It MUST NOT duplicate Focusa state merely to feel persistent.

Correct relationship:

```text
Canonical Workstream intelligence
        |
        | projected through a stable role binding
        v
PROJECT FOREMAN
        |
        +-- Pi session
        +-- Focusa Desktop
        +-- Focusa Mobile / Ambient Operator
        +-- Voice
        +-- browser / Cockpit
        +-- API / CLI
        +-- delegated workers
```

Those are different projections and runtime attachments to the same project-responsible intelligence.

---

## 2. Scope decision

### 2.1 One default Foreman per Workstream Root

The canonical default is:

```text
one WorkstreamRoot
        ↓
one Foreman role projection
```

Spec 164 remains owner of Workstream Root identity and persistence. A Session, Attachment, browser context, worker, model process, UI window, or Continuity tail MUST NOT create a new Foreman implicitly.

A project may intentionally define additional specialized project-responsible roles later, but they require explicit typed role/scope rather than accidental session proliferation.

### 2.2 No daemon-global Foreman singleton

There is no universal `current_foreman` authority.

Every Foreman operation resolves:

```text
ScopeRef / WorkstreamRoot
→ Foreman binding
→ current Workpoint / relevant operation
```

before mutation.

### 2.3 Cross-project intelligence belongs above Foreman

Wirebot/Chief-of-Staff style systems may reason across many Workstreams and delegate to exact Foreman refs. They do not collapse those project intelligences into one global Focusa Foreman.

---

## 3. Foreman identity is not model identity

The Foreman is a **role projection**, not a provider/model process.

Reuse Spec 72 identity primitives. Do not create a second agent-identity authority.

Conceptually:

```yaml
schema: focusa.foreman_binding.v1
foreman_ref:
workstream_root_ref:
agent_identity_ref:
role_profile_ref:
runtime_constitution_ref:
project_genesis_ref:
created_at:
state_revision_ref:
```

`foreman_ref` is a stable binding/projection identifier. The canonical agent principal remains the Spec-72-owned `AgentIdentity`.

Changing:

```text
Claude → Codex → local model → another Pi/provider session
```

does not create a different Foreman when the same approved role binding remains active.

A model/session switch creates or changes `ActorInstance`/runtime attachment and MUST preserve provenance.

---

## 4. C.R.I.S.T. forms the Foreman's project understanding

The Foreman is not initialized by a giant persona prompt.

Its project understanding is grounded by Spec 135/135B:

```text
Context
→ Role
→ Interview
→ Spec
→ Tasks
```

combined with the living runtime:

```text
Trajectory
Workpoint
Evidence
Receipts
history
constraints
current environment
promoted learning
```

The result is a project-responsible intelligence that can explain not merely repository contents, but **what the project is, why it exists, what governs it, what is happening now, and what safe move comes next**.

C.R.I.S.T. does not automatically grant execution permission. Role/Project Genesis describe responsibility and meaning; capability/authority remain separately evaluated.

---

## 5. Foreman Hydration Packet

A Foreman projection MUST hydrate from a bounded, source-bearing packet rather than transcript memory.

```yaml
schema: focusa.foreman_hydration_packet.v1
foreman_ref:
workstream_root_ref:
project_identity_ref:
runtime_constitution_ref:
project_genesis_ref:
trajectory_ref:
current_workpoint_ref:
active_spec_refs: []
ready_work_refs: []
constraint_refs: []
decision_refs: []
recent_evidence_refs: []
recent_receipt_refs: []
active_session_refs: []
presence_projection_ref:
radar_episode_refs: []
capability_projection_ref:
credential_requirement_refs: []
recovery_refs: []
source_revision:
generated_at:
fresh_until:
```

Rules:

1. minimal applicable slice first;
2. no raw transcript tail as canonical authority;
3. no broad cross-project memory injection by default;
4. stale/unknown fields remain marked stale/unknown;
5. every authority-bearing field references its primitive owner;
6. the packet is reconstructible after model/session loss.

---

## 6. Responsibilities

The Foreman role may be configured as builder, orchestrator, or hybrid.

Its project-responsible responsibility set includes:

```text
Inspect
Understand
Plan
Prepare work
Prioritize within approved mission
Delegate
Execute within grants
Test / verify
Observe
Review
Correct
Recover
Explain
Report
Learn through existing promotion paths
```

The invariant is:

> **Foreman != orchestrator only. Foreman != worker only. Foreman = project-responsible operating role.**

---

## 7. Authority boundary

A Foreman role does not self-mint capability.

It may:

- observe scoped canonical/operational state;
- prepare proposals;
- invoke approved Focusa operations;
- execute actions covered by current grants;
- create/delegate bounded work through existing Work Loop/Silent Session mechanisms;
- ask for approval/clarification;
- reconcile Evidence/Receipts;
- recommend spec/Trajectory/Workpoint changes as typed proposals.

It may not merely because it is Foreman:

- approve its own reserved/consequential operation;
- broaden project scope;
- create architecture authority;
- rewrite canonical spec without the governed authoring path;
- access another Workstream's private state;
- bypass Context Authority, credential policy, settlement, writer leases, placement, or Veragensia enforcement;
- convert Radar observation into canonical truth directly.

Role responsibility is not permission.

---

## 8. Worker relationship

Workers are replaceable runtime attachments beneath project continuity.

```text
Project Foreman
   |
   +-- Pi worker
   +-- Codex worker
   +-- Claude worker
   +-- UIAI/browser worker
   +-- test/review worker
   +-- Silent Sessions
   +-- future specialist agents
```

Spec 133 owns durable agent sessions/runs. Spec 79 owns continuous Work Loop policy. Spec 139 owns placement/presence. Veragensia owns machine/workcell topology and enforcement when execution occurs on an Agent Computer.

If a worker dies, the Foreman does not die. Focusa retains assignment, Workpoint, attempts, Evidence, failure state, and continuation; the Foreman may rehydrate and reassign according to authority.

---

## 9. Model and harness switching

A Foreman may switch model/provider/harness only through a typed runtime transition.

Required record:

```yaml
schema: focusa.foreman_runtime_attachment.v1
foreman_ref:
actor_instance_ref:
harness_ref:
provider_ref:
model_ref:
session_ref:
attached_at:
detached_at:
reason:
authority_ref:
hydration_packet_ref:
```

A new attachment must not inherit stale hidden harness memory as canonical state.

Pi remains the default/reference Focusa harness; other harnesses consume the same Workstream-rooted state through thin adapters.

---

## 10. Surface parity

The same Foreman must be addressable through:

- Pi/terminal;
- Focusa Desktop / Mission Canvas;
- Focusa Mobile / Ambient Operator;
- spoken voice;
- PWA;
- UIAI Cockpit/browser-associated surfaces where useful;
- API/CLI;
- future compatible surfaces.

Surfaces may present different amounts of detail. They MUST NOT create different project identities or separate Foreman memories.

### 10.1 Voice address resolution

Examples:

```text
"Foreman, what's blocking this project?"
"Foreman, give that verification to another worker."
"Foreman, switch this Workpoint to Codex."
```

Voice resolution rules:

1. explicit Workstream/project reference wins;
2. an already-bound active Ambient/Conversation scope may supply the Workstream;
3. if more than one Workstream is plausible, clarify rather than route globally;
4. saying `Foreman` is address selection, not authorization;
5. the resulting operation follows Spec 181 plus normal Focusa authority.

---

## 11. Radar relationship

Spec 183 owns **Radar** proactive observation/signal/episode semantics.

The Foreman consumes bounded Radar Signals/Episodes relevant to its Workstream and may:

```text
inspect
→ investigate
→ request more evidence
→ prepare response
→ create/propose work
→ execute within grant
→ escalate
```

Radar does not become the Foreman's memory, and the Foreman does not mutate Radar raw observations into truth without the normal reducer/promotion path.

---

## 12. Wirebot / Chief-of-Staff relationship

Cross-project intelligence such as Wirebot may hold a broader operator-authorized portfolio/life view.

The relationship is:

```text
Human
  ↓
Wirebot / Chief of Staff
  ↓
exact Workstream/Foreman delegation
  ↓
Foreman
  ↓
workers / tools / Agent Computers
```

Wirebot may ask one or many Foremen for status, work, evidence, or action. It MUST NOT impersonate their Workstream state or make one Foreman's assumptions canonical in another Workstream.

---

## 13. Evidence-backed answers

Foreman claims about completed or consequential work SHOULD resolve to Evidence/Receipt refs.

Examples:

```text
"Did CI pass?"          → test/workflow Evidence
"Did it deploy?"        → settlement/Receipt
"What changed?"         → revision/diff Evidence
"Why are we blocked?"   → Workpoint/blocker + supporting Evidence
"What happened overnight?" → session/receipt/radar episode timeline
```

The Foreman must distinguish observed, inferred, stale, unknown, and proven state.

---

## 14. Recovery

Foreman continuity must survive:

- model crash;
- Pi/session exit;
- phone disconnect;
- UI restart;
- daemon restart;
- worker death;
- remote-node replacement;
- conversation interruption.

Recovery sequence:

```text
resolve WorkstreamRoot
→ resolve Foreman binding
→ reconstruct hydration packet
→ reconcile active sessions/effects
→ attach compatible runtime
→ continue from canonical Workpoint
```

No transcript replay is required to re-create project identity.

---

## 15. Operations

Initial operation family SHOULD include:

```text
foreman.resolve
foreman.view
foreman.hydrate
foreman.status
foreman.explain
foreman.work.list
foreman.worker.list
foreman.worker.delegate
foreman.worker.steer
foreman.worker.pause
foreman.worker.stop
foreman.runtime.attach
foreman.runtime.switch
foreman.radar.inbox
foreman.recovery.plan
```

Operation Registry/Spec 141 generation rules apply. Names may be refined before implementation, but semantic ownership may not be duplicated in client code.

---

## 16. Implementation slices

These are implementation/acceptance slices, **not a second task tracker**. Materialize them into repository-local `br` under one parent when implementation begins.

### F182-S1 — Core binding types

- define `ForemanBinding` and `ForemanRuntimeAttachment` using existing Spec-72 identities;
- bind exactly to Spec-164 Workstream Root;
- persistence/reducer/events;
- no new global singleton.

**Done when:** two Workstreams can resolve distinct Foremen under one daemon with zero state bleed.

### F182-S2 — Hydration packet

- generated bounded packet;
- exact source refs/freshness;
- restart reconstruction;
- minimal applicable slice tests.

**Done when:** a fresh model/session can answer project status from Focusa state without transcript replay.

### F182-S3 — Operation Registry/API/CLI/Pi

- generated operation schemas;
- REST/CLI/Pi reference adapter;
- non-Pi adapter parity;
- authority errors preserved.

### F182-S4 — Worker supervision

- Spec-133 session list/delegate/steer/pause/stop;
- Work Loop integration;
- placement and writer-lease checks;
- no raw shell spawning as Foreman authority.

### F182-S5 — Runtime/model transfer

- switch model/provider/harness;
- preserve Foreman binding;
- detach stale ActorInstance;
- rehydrate exact current Workpoint;
- Evidence of transition.

### F182-S6 — Desktop/Mobile/Voice projection

- one shared presenter contract;
- Focusa Desktop/Mission Canvas projection;
- Ambient Operator projection;
- Spec-181 voice address/response;
- voice/text parity acceptance.

### F182-S7 — Radar inbox

- consume Spec-183 Signals/Episodes;
- investigate/prepare/escalate loop;
- no direct raw-observation mutation.

### F182-S8 — Wirebot delegation adapter

- exact Workstream/Foreman refs;
- cross-project Chief-of-Staff routing;
- scoped response/evidence packet;
- no cross-Workstream context bleed.

---

## 17. Acceptance invariants

A Foreman implementation is valid only when:

1. the same Workstream resolves the same Foreman binding across UI/model/session replacement;
2. different Workstreams do not share canonical Foreman state;
3. model switching changes runtime attachment without changing project identity;
4. Foreman restart rehydrates from Focusa, not hidden session memory;
5. Foreman role alone grants no permission;
6. worker death does not destroy project continuity;
7. status/completion claims link to Evidence/Receipts where applicable;
8. mobile/voice/desktop projections reach the same canonical operations;
9. ambiguous spoken `Foreman` routing fails to exact-scope clarification;
10. Wirebot delegation preserves exact Foreman/Workstream identity;
11. Radar input remains proposal/evidence until governed promotion/action;
12. architecture authority remains with the Canonical Owner Principal, never the Foreman role.

---

## 18. Final principle

> **The Foreman is the project taking responsibility through an agent role—not a chat session pretending to remember the project.**
