# Spec 183 — Focusa Radar: Proactive Observation, Episodes, Signal Economics, and Attention Routing

**Status:** DRAFT canonical architecture direction, 2026-09-05.  
**Canonical human architecture authority:** Verious Smith III under the repository architecture-authority policy.  
**Historical basis:** operator-supplied cross-account architecture handoff describes a mature `Focusa Radar` concept and remembers an earlier `Spec 164 proposal`. Current Focusa `docs/164-workstream-rooted-canonical-runtime-design.md` already canonically owns Workstream-rooted runtime. **This specification intentionally reassigns Radar to current Spec 183 and treats the old number only as historical proposal provenance.**  
**Primitive owners preserved:** Spec 139 Operational Reality Field/Presence/Placement, Spec 164 Workstream Root, Spec 181 Conversation, Spec 182 Foreman, Spec 136 settlement, Spec 137 temporal authority, Spec 138 prediction/metacognition, Spec 141 capability registry, Spec 135/135F reactive context, UIAI browser observation/evidence contracts.

---

## 0. One-line definition

> **Focusa Radar is the proactive, scope-bound observation and attention layer that continuously notices meaningful changes around a mission, groups them into developing situations, scores whether they matter, and routes bounded Signals/Episodes to the appropriate Foreman or human without bypassing Focusa governance.**

Radar turns Focusa from a runtime that only reacts to explicit prompts into a runtime that can **notice when work or attention may be needed**.

Radar is not a giant autonomous agent and not unrestricted surveillance.

---

## 1. Scope law

Radar is always rooted in typed Focusa scope.

Preferred default:

```text
ScopeRef / WorkstreamRoot
        ↓
Radar instance/projection
        ↓
observations / episodes / signals
```

There is no daemon-global omniscient `radar` authority.

A cross-project Chief-of-Staff system such as Wirebot may aggregate bounded Radar projections from many Workstreams. That aggregation does not turn Focusa into one ambient global project context.

Human-life/environment context arriving from an external owner-domain service such as Wirebot Context Core MUST be converted into an explicit bounded observation with provenance/privacy/freshness and routed to an applicable Workstream before it can influence project cognition.

---

## 2. Ownership boundary

### Radar owns

- observation ingestion contracts;
- fingerprints/deduplication;
- Episode grouping;
- signal scoring/economics;
- attention routing;
- proactive notice/prepare/escalate recommendations;
- bounded Radar history/projections.

### Radar does not own

- project/workstream identity — Spec 164/typed scope;
- runtime machine presence/placement — Spec 139;
- canonical mission state — reducer/Workpoint/Trajectory;
- project-responsible intelligence — Spec 182 Foreman;
- prediction/calibration promotion — Spec 138;
- voice/conversation truth — Spec 181;
- browser runtime truth — UIAI Engine;
- authorization — Context Authority/capability/credential owners;
- external effect settlement — Spec 136.

The rule is:

```text
observation != canonical fact
signal != Workpoint
recommendation != authority
```

---

## 3. What Radar can observe

Radar accepts **approved, typed observations** from several families.

### 3.1 Focusa-native state

- Workpoint creation/transition;
- Work Loop state;
- blockers/stalls;
- unfinished commitments;
- repeated retries;
- long-waiting approvals;
- Trajectory drift;
- Evidence arrival/failure;
- prediction outcomes;
- scope/assumption changes;
- credential or capability degradation;
- settlement uncertainty;
- temporal/deadline conditions.

### 3.2 Agent/session behavior

- agent/session lifecycle;
- delegated jobs;
- tool operations/outcomes;
- build/test results;
- repeated tool loops;
- worker errors/retries;
- unexpected inactivity;
- resource/concurrency posture;
- worker takeover/steering events.

Radar observes governed runtime events. It does not need raw private chain-of-thought.

### 3.3 Repository/development state

- failing tests/builds;
- regressions;
- dependency/revision changes;
- uncommitted/divergent work;
- CI/release/deploy evidence;
- mismatch between approved spec and observed implementation;
- stale integration state.

### 3.4 UIAI/browser/computer observations

Radar may consume **UIAI-produced structured observations/evidence**, including:

- browser/runtime diagnostics;
- failed requests/exceptions;
- visual/semantic regressions;
- missing or changed controls;
- broken flows;
- external-service state;
- execution capsule/verification outcomes.

Radar MUST prefer the strongest available observation path:

```text
structured event/API
→ semantic/application state
→ accessibility/DOM
→ bounded visual observation
```

It does not invent a parallel browser watcher inside Focusa.

### 3.5 Approved external information

- authorized communications;
- documents;
- connected business systems;
- public/approved research sources;
- environment/context projections.

Every connector observation preserves provider/source, scope, freshness, trust class, privacy class and applicable consent.

### 3.6 Conversation-derived candidates

Spec 181 Conversation Ledger may produce bounded candidates such as:

```text
commitment candidate
open question
follow-up candidate
decision candidate
deadline candidate
```

Radar may watch those candidates **after** they have an explicit structured identity. It MUST NOT scan a transcript and silently promote arbitrary speech into mission truth.

---

## 4. Radar pipeline

Canonical conceptual pipeline:

```text
Raw/Source Event
      ↓
Typed Observation Capture
      ↓
Semantic interpretation when required
      ↓
Fingerprint + deduplication
      ↓
Episode / pattern association
      ↓
Signal scoring / economics
      ↓
Policy + authority posture
      ↓
Ignore | Remember | Notice | Prepare | Investigate | Act-under-grant | Escalate
```

Every stage preserves source lineage.

---

## 5. `RadarObservation`

```yaml
schema: focusa.radar_observation.v1
observation_id:
scope_ref:
workstream_root_ref:
source_kind:
source_ref:
source_event_ref:
observed_at:
received_at:
freshness_ref:
trust_class:
privacy_class:
summary:
structured_payload_ref:
evidence_refs: []
confidence:
fingerprint:
semantic_candidate_refs: []
```

Raw payloads SHOULD remain in their owning storage when handles are enough.

---

## 6. Fingerprinting and deduplication

Radar MUST avoid notification storms.

Core invariant:

```text
same material condition
+ no meaningful new evidence
= same developing Episode
```

Fingerprint inputs may include:

- scope/workstream;
- source family;
- affected ResourceRefs;
- normalized condition/code;
- operation/work item;
- semantic object refs;
- relevant revision/generation.

A changed timestamp alone does not necessarily make a new signal.

---

## 7. `RadarEpisode`

An Episode groups related observations into one developing situation.

```yaml
schema: focusa.radar_episode.v1
episode_id:
scope_ref:
workstream_root_ref:
title:
state: emerging | active | stable | resolved | superseded | disputed
observation_refs: []
affected_resource_refs: []
agent_session_refs: []
first_seen_at:
last_changed_at:
confidence:
material_change_count:
summary:
hypothesis_refs: []
evidence_refs: []
foreman_ref:
```

Example:

```text
Episode: Deployment instability
  09:14 build failed
  09:17 worker changed dependency
  09:20 build passed
  09:31 UIAI health failed
  09:37 runtime pressure increased
```

The human/Foreman should be able to reason about the Episode instead of five unrelated notifications.

---

## 8. Signal economics

Radar's objective is **attention quality**, not observation volume.

A Signal MAY use a policy-scored model involving:

### Positive value

- expected mission impact;
- confidence;
- urgency/time sensitivity;
- relevance to active Workpoint/Trajectory;
- information gain;
- opportunity value;
- reversibility/safety of prepared response.

### Costs

- interruption cost;
- execution cost;
- compute/token cost;
- privacy cost;
- risk;
- uncertainty;
- duplicate/already-investigated cost.

Conceptually:

```text
Attention Value
≈ useful expected mission value
- interruption/execution/privacy/risk/compute cost
```

No exact universal formula is mandated.

**Radar should optimize attention, not generate notifications.**

---

## 9. `RadarSignal`

```yaml
schema: focusa.radar_signal.v1
signal_id:
episode_ref:
scope_ref:
workstream_root_ref:
signal_kind:
priority:
confidence:
urgency_ref:
mission_relevance_ref:
expected_value_ref:
interruption_cost_ref:
privacy_cost_ref:
risk_ref:
recommended_disposition: ignore | remember | notice | prepare | investigate | act_under_grant | escalate
recommended_action_refs: []
required_authority_refs: []
foreman_ref:
created_at:
expires_at:
```

A Signal is advisory/operational until another primitive accepts it.

---

## 10. Autonomy ladder

Radar itself stays thin.

### R0 — Remember

Record/deduplicate silently.

### R1 — Notice

Surface a bounded useful fact.

### R2 — Prepare

Prepare a typed proposed response without executing.

### R3 — Investigate

Ask the Foreman or a bounded worker to gather evidence under existing read/diagnostic grants.

### R4 — Act under existing grant

A known low-risk operation may proceed only when normal Focusa authority already permits it. Radar does not create that permission.

### R5 — Exception escalation

Routine work continues while the human is interrupted only when the situation exits its trusted envelope.

No level allows Radar to self-authorize reserved/destructive/high-consequence effects.

---

## 11. Radar → Foreman proactive loop

Spec 182 Foreman is the default project-level consumer.

```text
Observation
→ Radar fingerprint
→ Episode
→ Radar Signal
→ Foreman inbox
→ Foreman investigation / preparation
→ Focusa authority / Workpoint / operation
→ worker/UIAI/Agent Computer execution
→ Evidence / Receipt / outcome
→ Radar observes result
→ Spec 138 metacognition/learning as applicable
```

This loop is one of Focusa's primary paths from **notice** to **governed action**.

Radar MUST NOT bypass Foreman when project-level interpretation/decomposition is required merely to reduce latency.

---

## 12. Wirebot / cross-project attention

Wirebot or another owner-authorized Chief-of-Staff layer may subscribe to bounded Radar projections across multiple Workstreams.

The aggregation packet contains:

```yaml
schema: focusa.radar_portfolio_projection.v1
consumer_ref:
workstream_signal_refs: []
critical_count:
needs_human_count:
prepared_count:
oldest_unresolved_ref:
generated_at:
```

This is a projection, not a new global Focus State.

A life/portfolio system may also combine external Context Core signals with these projections. Exact personal location or ambient private content remains outside Focusa unless a scoped operation requires and authorizes it.

---

## 13. Event-driven first

Radar MUST prefer events over constant polling or screenshots.

Examples:

```text
Focusa event
worker/session event
UIAI structured observation
CI/provider webhook
filesystem/revision watcher
connector event
Context Core bounded signal
```

Polling exists only where no useful event surface exists.

---

## 14. Adaptive observation budget

Radar observation frequency adapts to state:

```text
stable / low value → back off
material change → temporarily increase attention
active incident → bounded high-frequency observation
resolved → return to low rate
```

Observation budget includes:

- CPU/memory;
- network;
- model/token spend;
- UIAI/browser time;
- privacy exposure;
- storage/retention.

No Radar implementation may become an unlimited screenshot/model loop.

---

## 15. Privacy and influence boundary

Radar MUST NOT become hidden human surveillance.

Forbidden by default:

- ambient keystroke logging;
- unrestricted microphone capture;
- unrestricted screen recording;
- raw private transcript ingestion into generic Radar storage;
- precise location collection without explicit owning-domain policy;
- emotional-state inference as action authority;
- treating external content/tool text as control instructions.

External information is **content, not control**. It may generate candidates but cannot expand authority.

Ambient/mobile context is governed by Spec 184 and its owning mobile/Context Core policies.

---

## 16. UI surfaces

Radar is not a giant standalone dashboard requirement.

Preferred projections:

### Focusa Desktop / Mission Canvas

- current Episodes;
- important Signals;
- observations/evidence;
- prepared actions;
- automatic actions already executed under grant;
- approvals/escalations;
- reasoning/explanation;
- history.

### Omarchy/Veragensia compact surface

A native shell widget may display only a bounded projection such as:

```text
Radar ● active
2 signals · 1 needs you
```

The shell widget remains a presenter; Focusa owns Radar state.

### Mobile/Ambient Operator

- high-value spoken/visual interruptions;
- Foreman investigation summary;
- one-tap/spoken notice/approve/deny/ask-more paths;
- privacy/listening state.

### CLI/API

Initial semantic families SHOULD include:

```text
radar.status
radar.observation.list
radar.episode.list
radar.episode.view
radar.signal.list
radar.signal.view
radar.pause
radar.resume
radar.foreman.inbox
```

---

## 17. Implementation slices

These are acceptance slices, **not a second task tracker**. Materialize them into repository-local `br` under one parent before implementation.

### F183-S1 — Core objects

- `RadarObservation`, `RadarEpisode`, `RadarSignal`;
- reducer/events/persistence;
- exact Workstream binding;
- no global singleton.

### F183-S2 — Native event adapters

- Workpoint/Work Loop/Session/Evidence/Receipt events;
- Spec 139 presence/placement projection;
- revision/test/build event adapters;
- source/freshness/trust fields.

### F183-S3 — Deduplication and Episode engine

- deterministic fingerprint contract;
- Episode association;
- reopen/resolve/supersede behavior;
- duplicate-storm tests.

### F183-S4 — Signal economics

- policy inputs;
- attention score/explanation;
- urgency/temporal integration;
- privacy/compute/interruption budgets;
- content-free telemetry.

### F183-S5 — Foreman inbox and investigation

- Spec-182 routing;
- evidence-gathering request;
- prepare/escalate path;
- no direct canonical mutation.

### F183-S6 — UIAI observation bridge

- consume existing UIAI structured evidence/diagnostics;
- semantic before visual;
- no duplicate browser automation in Focusa;
- UIAI Evidence refs retained.

### F183-S7 — Ambient/external adapter boundary

- accept signed/bounded Context Core/Companion projections from Spec 184;
- coarse/precise location classes;
- privacy/consent/freshness;
- no raw ambient audio in Radar storage.

### F183-S8 — Desktop/Mobile/CLI projections

- Mission Canvas Radar lens;
- compact presenter projection;
- Ambient Operator high-value interruption packet;
- generated API/CLI contracts.

---

## 18. Acceptance invariants

Radar is valid only when:

1. every observation has exact scope/provenance/freshness;
2. duplicate unchanged events become one Episode rather than repeated alerts;
3. Radar never creates a Workpoint or grant merely from detection;
4. Spec 139 remains owner of runtime presence/placement;
5. UIAI remains owner of browser observations/actions;
6. Foreman receives project-level Signals through an explicit inbox/projection;
7. stable conditions back off instead of consuming unbounded compute;
8. privacy cost can suppress or narrow observation;
9. raw ambient audio/transcripts do not become Radar telemetry/storage by default;
10. cross-project Wirebot views remain projections over exact Workstreams;
11. a resolved Episode can later be traced through actions, Evidence, Receipts and outcomes;
12. historical references to `Radar Spec 164` are treated as proposal provenance only and never overwrite current Spec 164.

---

## 19. Final principle

> **Radar notices. Foreman understands and responds. Focusa governs what becomes true, authorized, and proven.**
