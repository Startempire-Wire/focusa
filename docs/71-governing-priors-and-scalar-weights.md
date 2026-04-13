# Governing Priors and Scalar Weights

## Purpose

Define how Focusa orders competing influences on behavior.

This document exists to prevent inconsistency when multiple valid forces are active at once, such as:
- hard safety and policy boundaries
- identity and role
- the current ask and scope
- long-running mission commitments
- execution reality and affordances
- local optimization or convenience

This layer determines not just what matters, but **what wins** when these forces conflict.

---

## Core Thesis

Behavior should not be governed by one flat weighted soup.

Focusa should distinguish between:
- **governing priors** — upstream frames that shape behavior class-wide
- **scalar weights** — local comparative values used inside a permitted decision space

Identity, role, hard boundaries, and current ask should behave more like governing priors than ordinary weighted preferences.

---

## Design Laws

1. Hard prohibitions outrank all scalar tradeoffs.
2. Identity should be upstream, not merely heavier.
3. The current ask should remain purer than adjacent mission carryover unless relevance is proven.
4. Scalar weights should only operate after higher-order constraints and priors are applied.
5. Weighting must be inspectable and traceable.
6. Conflicts between priors must have explicit precedence rules.

---

# 1. Core Object Types

## GoverningPrior
Represents an upstream behavioral frame that shapes downstream reasoning.

### Required properties
- `id`
- `prior_kind`
- `status`

### Examples
- hard_safety_prior
- identity_prior
- current_ask_prior
- mission_commitment_prior
- affordance_reality_prior

---

## ScalarWeight
Represents a local weighting factor used within an allowed decision space.

### Required properties
- `id`
- `weight_kind`
- `value`
- `status`

### Examples
- urgency weight
- reversibility weight
- cost weight
- reliability weight
- latency weight
- completion weight

---

## PriorityBand
Represents a rank band in the precedence system.

### Required properties
- `id`
- `band_kind`
- `rank`
- `status`

### Canonical bands
- non_overridable
- constitutional
- scope_governing
- mission_governing
- execution_governing
- optimization

---

## ConflictSet
Represents a set of competing priors or weighted options.

### Required properties
- `id`
- `conflict_kind`
- `status`

---

## ResolutionOutcome
Represents the resolved winner and rationale.

### Required properties
- `id`
- `resolution_kind`
- `status`

---

# 2. Precedence Model

## Band 1: Non-overridable
Includes:
- hard safety
- absolute policy boundaries
- explicit forbidden actions

## Band 2: Constitutional
Includes:
- agent identity
- role
- trust boundaries
- enduring value constraints

## Band 3: Scope-governing
Includes:
- current ask
- query scope
- operator correction
- explicit subject narrowing

## Band 4: Mission-governing
Includes:
- mission
- commitments
- long-running goals
- active working-set obligations

## Band 5: Execution-governing
Includes:
- affordances
- permissions
- dependencies
- resources
- cost, latency, reversibility, reliability

## Band 6: Local optimization
Includes:
- convenience
- elegance
- optional improvement
- secondary preference tuning

---

# 3. Scalar Weight Types

These weights operate only after higher bands have constrained the choice space.

## Recommended weight kinds
- urgency
- severity
- importance
- relevance
- completion_value
- reversibility
- reliability
- cost
- latency
- operator_attention_cost
- trust_requirement

---

# 4. Core Relation Types

## belongs_to_band
Source:
- GoverningPrior
- ScalarWeight

Target:
- PriorityBand

---

## competes_in
Source:
- GoverningPrior
- ScalarWeight

Target:
- ConflictSet

---

## resolved_by
Source:
- ConflictSet

Target:
- ResolutionOutcome

---

## shapes
Source:
- GoverningPrior
- ScalarWeight

Target:
- ActionIntent
- WorkingSet
- QueryScope
- Affordance
- ResolutionOutcome

---

# 5. Core Action Types

## build_priority_stack
Construct the active precedence stack for the current turn or action.

## normalize_scalar_weights
Normalize local weights inside the currently allowed decision space.

## resolve_conflict_set
Resolve conflicts between priors and/or scalar weights.

## verify_precedence_application
Check whether the system obeyed the active precedence rules.

## trace_resolution_rationale
Emit why the chosen behavior won.

---

# 6. Success Condition

Governing Priors and Scalar Weights is successful when Focusa can produce behavior that is consistent, inspectable, and stable because it knows which forces govern first and which only optimize within permitted bounds.
