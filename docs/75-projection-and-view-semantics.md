# Projection and View Semantics

## Purpose

Define how Focusa materializes different ontology-backed views for different purposes without changing canonical truth.

This document exists because different actors and processes need different read models, such as:
- planner view
- executor view
- reviewer view
- operator view
- compressed low-budget view
- evidence-heavy diagnostic view

A coherent system needs to distinguish:
- canonical ontology state
from
- purpose-specific projections of that state

---

## Core Thesis

Truth should be canonical.
Views should be contextual.

Focusa should not give every actor the same representation of truth at all times.
Instead, it should project the right bounded view for the current role, task, and resource constraints.

---

## Design Laws

1. Canonical ontology state and projections must be distinct.
2. Projections must be traceable back to canonical state.
3. Different roles may need different views of the same truth.
4. Compression is a projection operation, not a truth mutation.
5. Projections must respect scope, permissions, and affordances.

---

# 1. Core Object Types

## Projection
Represents a derived view over canonical ontology state.

### Required properties
- `id`
- `projection_kind`
- `status`

---

## ViewProfile
Represents the intended audience or purpose of a projection.

### Required properties
- `id`
- `view_kind`
- `status`

### Examples
- planner_view
- executor_view
- reviewer_view
- operator_view
- low_budget_view
- diagnostics_view

---

## ProjectionRule
Represents how canonical state should be transformed into a projection.

### Required properties
- `id`
- `rule_kind`
- `status`

---

## ProjectionBoundary
Represents limits applied to a projection.

### Required properties
- `id`
- `boundary_kind`
- `status`

### Examples
- token_budget_boundary
- permission_boundary
- scope_boundary
- role_boundary

---

# 2. Core Relation Types

## derived_from_canonical
Source:
- Projection

Target:
- CanonicalEntity
- WorkingSet
- QueryScope
- CurrentAsk

---

## shaped_by_view
Source:
- Projection

Target:
- ViewProfile
- ProjectionRule
- ProjectionBoundary

---

## allowed_for_role
Source:
- ViewProfile

Target:
- RoleProfile
- Actor
n
---

# 3. Core Action Types

## build_projection
Construct a projection for a given purpose.

## compress_projection
Reduce a projection under boundary constraints without mutating canonical truth.

## verify_projection_fidelity
Check whether a projection still preserves the required governing semantics.

## switch_view_profile
Change which view is active for a given actor or process.

---

# 4. Success Condition

Projection and View Semantics is successful when Focusa can show different actors and processes the right bounded representation of truth without fragmenting or mutating canonical ontology state.
