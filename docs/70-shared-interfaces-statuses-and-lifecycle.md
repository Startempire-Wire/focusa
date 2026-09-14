# Shared Interfaces, Statuses, and Lifecycle

## Purpose

Define the shared cross-cutting consistency layer that every Focusa ontology domain should implement.

This document exists to prevent drift between domains by standardizing:
- shared interfaces
- shared status vocabulary
- shared value kinds
- lifecycle/state-transition rules
- evidence/provenance normalization
- actor/role/trust semantics
- temporal/event semantics

This is the consistency substrate for the ontology stack.

---

## Core Thesis

Once Focusa has multiple ontology domains, quality is no longer determined only by how good each domain model is.

Quality is determined by whether the domains remain:
- structurally compatible
- semantically interoperable
- lifecycle-consistent
- traceable
- governable

Without a shared consistency layer, domains become parallel good ideas instead of one coherent operating ontology.

---

## Design Laws

1. Shared interfaces must be domain-neutral.
2. Shared statuses must not be reinvented per domain.
3. Lifecycle transitions must be explicit and governable.
4. Evidence and provenance semantics must be normalized.
5. Actor/role/trust semantics must be reusable across domains.
6. Time and events must be represented consistently.
7. This layer should constrain domain ontologies without overfitting them.

---

# 1. Shared Interfaces

An interface is a reusable behavioral contract that multiple ontology objects may implement.

## Verifiable
Meaning:
This object can be checked against evidence or verification procedures.

### Required fields
- `verification_status`
- `verification_refs`
- `verification_kind`

### Typical implementers
- Decision
- Constraint
- Task
- Component
- LayoutRule
- ValidationRule
- Affordance

---

## Actionable
Meaning:
This object can participate in action selection or execution planning.

### Required fields
- `allowed_actions`
- `action_constraints`
- `action_priority`

### Typical implementers
- Task
- Component
- Affordance
- ToolSurface
- WorkingSet

---

## Scoped
Meaning:
This object is valid only within a defined scope or subject boundary.

### Required fields
- `scope_ref`
- `scope_kind`
- `scope_status`

### Typical implementers
- CurrentAsk
- QueryScope
- Constraint
- WorkingSet
- VariationSet

---

## ArtifactBacked
Meaning:
This object has linked evidence or source artifacts.

### Required fields
- `artifact_refs`
- `artifact_kind_summary`

### Typical implementers
- EvidenceArtifact
- VisualArtifact
- ComparisonResult
- CritiqueResult
- Verification
n
---

## Ownable
Meaning:
This object has ownership or authority semantics.

### Required fields
- `owner_ref`
- `owner_kind`
- `authority_boundary_refs`

### Typical implementers
- ToolSurface
- Resource
- Permission
- Ownership
- Deployment surface

---

## Reversible
Meaning:
This object or associated action has rollback/recovery semantics.

### Required fields
- `reversibility_kind`
- `rollback_refs`
- `recovery_notes`

### Typical implementers
- ActionIntent
- Affordance
- ReversibilityProfile
- ImplementationPlan

---

## Costed
Meaning:
This object carries cost/latency/resource implications.

### Required fields
- `cost_refs`
- `latency_refs`
- `resource_refs`

### Typical implementers
- Affordance
- ToolSurface
- ActionIntent
- ExecutionContext

---

## RiskBearing
Meaning:
This object has meaningful risk semantics.

### Required fields
- `risk_refs`
- `severity`
- `mitigation_refs`

### Typical implementers
- Task
- ActionIntent
- Affordance
- Deployment operation
- UI release state

---

# 2. Shared Status Vocabulary

All ontology domains should draw from a shared status vocabulary.

## Core statuses
- `proposed`
- `candidate`
- `active`
- `blocked`
- `verified`
- `failed`
- `stale`
- `superseded`
- `retired`
- `completed`
- `canonical`
- `experimental`

## Status rules
1. Domain ontologies may extend but should not contradict this base vocabulary.
2. Cross-domain workflows should prefer shared statuses for interoperability.
3. Status meaning should remain stable across domains.

---

# 3. Shared Value Kinds

Value kinds should be normalized where practical.

## Recommended shared value kinds
- `severity`
- `priority`
- `confidence`
- `freshness`
- `verification_kind`
- `scope_kind`
- `owner_kind`
- `authority_kind`
- `artifact_kind`
- `cost_kind`
- `latency_kind`
- `reliability_kind`
- `reversibility_kind`

This enables cleaner interoperability and less domain drift.

---

# 4. Lifecycle Model

Every major ontology object should be governable by a common lifecycle model.

## Canonical lifecycle stages
- `proposed`
- `active`
- `verified`
- `blocked`
- `completed`
- `superseded`
- `retired`

## Transition rules
### proposed -> active
Allowed when the object becomes the operative subject of work.

### proposed -> verified
Allowed when direct evidence immediately confirms the object.

### active -> verified
Allowed when the object is confirmed or satisfied.

### active -> blocked
Allowed when progress is prevented.

### blocked -> active
Allowed when blocker conditions are cleared.

### active -> completed
Allowed when required completion checks pass.

### verified -> superseded
Allowed when a newer verified object replaces it.

### completed -> retired
Allowed when the object is no longer operative.

### active -> failed
Allowed when the object is invalidated or execution collapses.

## Lifecycle law
Every domain object should either:
- explicitly use the shared lifecycle,
or
- define a narrower lifecycle that maps cleanly back to it.

---

# 5. Evidence and Provenance Normalization

Focusa now has multiple evidence-bearing domains.

All domains should normalize:
- provenance source
- evidence refs
- confidence
- timestamp/freshness
- verification status

## Shared provenance classes
- `user_asserted`
- `operator_asserted`
- `artifact_derived`
- `screenshot_derived`
- `tool_derived`
- `runtime_observed`
- `model_inferred`
- `verification_confirmed`
- `reducer_promoted`

---

# 6. Actor, Role, and Trust Primitives

These should be shared across domains.

## Actor
Represents an entity that can observe, decide, act, approve, or own.

### Actor kinds
- operator
- agent
- reviewer
- admin
- external_system

## Role
Represents a functional responsibility.

### Role kinds
- requester
- approver
- executor
- reviewer
- owner
- observer

## TrustProfile
Represents trust expectations or earned trust state.

### Trust kinds
- untrusted
- limited
- trusted_for_scope
- fully_trusted
- needs_review

These primitives let Focusa reason consistently about who may do what and how much autonomy to grant.

---

# 7. Temporal and Event Primitives

Multiple ontology domains need a shared temporal/event layer.

## Event
Represents something that happened.

### Required properties
- `id`
- `event_kind`
- `timestamp`
- `status`

## TemporalWindow
Represents a meaningful time boundary.

### Required properties
- `id`
- `window_kind`
- `start_time`
- `end_time`

## CausalLink
Represents a meaningful cause/effect relation between events or state changes.

### Required properties
- `id`
- `causal_kind`
- `status`

These primitives allow domains to align around:
- what happened
- when it happened
- what changed because of it

---

# 8. Conformance Rules for Domain Ontologies

Every new ontology domain should answer:
1. Which shared interfaces do its objects implement?
2. Which shared statuses does it use?
3. How do its lifecycle transitions map to the shared lifecycle?
4. How does it normalize provenance and evidence?
5. Which actor/role/trust semantics apply?
6. Which events and temporal windows matter?

If a domain cannot answer these, it is not fully integrated into Focusa.

---

# 9. What This Enables

With this layer, Focusa can:
- keep multiple ontologies interoperable
- reduce schema drift
- preserve system-wide consistency
- reason across domains with shared lifecycle semantics
- reuse actor/trust/evidence rules instead of reinventing them per domain
- make integration cleaner for future ontologies

---

# 10. Success Condition

Shared Interfaces, Statuses, and Lifecycle is successful when Focusa’s ontology domains behave like one coherent operating ontology rather than a collection of separate domain models.
