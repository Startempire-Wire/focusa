# Ontology Governance, Versioning, and Migration

## Purpose

Define how Focusa’s ontology stack evolves without losing coherence.

This document governs:
- ontology versioning
- compatibility
- migration
- deprecation
- review and approval of ontology changes
- cross-domain conformance over time

---

## Core Thesis

A multi-domain ontology system will drift unless change itself is governed.

Strong primitives are not enough.
Focusa also needs a disciplined process for:
- adding new domains
- changing shared interfaces
- deprecating old fields or statuses
- migrating existing canonical state
- preserving compatibility where needed

---

## Design Laws

1. Ontology changes should be explicit and reviewable.
2. Shared-layer changes are higher risk than domain-local changes.
3. Deprecation should precede destructive removal.
4. Migrations should preserve history and traceability.
5. Compatibility should be declared, not assumed.
6. Conformance to the shared substrate should be re-checked after every major ontology change.

---

# 1. Core Object Types

## OntologyVersion
Represents a named version of an ontology domain or shared layer.

### Required properties
- `id`
- `version_kind`
- `status`

---

## CompatibilityProfile
Represents compatibility guarantees or breakages between versions.

### Required properties
- `id`
- `profile_kind`
- `status`

---

## MigrationPlan
Represents the plan for moving ontology state from one version or schema to another.

### Required properties
- `id`
- `plan_kind`
- `status`

---

## DeprecationRecord
Represents the planned retirement of a field, relation, status, or object type.

### Required properties
- `id`
- `record_kind`
- `status`

---

## GovernanceDecision
Represents an approved ontology change decision.

### Required properties
- `id`
- `decision_kind`
- `status`

---

# 2. Core Relation Types

## versioned_as
Source:
- OntologyDomain
- SharedLayer

Target:
- OntologyVersion

---

## compatible_with
Source:
- OntologyVersion

Target:
- CompatibilityProfile
- OntologyVersion

---

## migrated_by
Source:
- OntologyVersion
- OntologyDomain

Target:
- MigrationPlan

---

## deprecated_by
Source:
- OntologyDomain
- SharedLayer
- OntologyVersion

Target:
- DeprecationRecord

---

## approved_by_governance
Source:
- MigrationPlan
- DeprecationRecord
- OntologyVersion

Target:
- GovernanceDecision
- Actor
- Role

---

# 3. Core Action Types

## create_version
Create a new ontology or shared-layer version.

## declare_compatibility
Declare compatibility guarantees or breakages.

## build_migration_plan
Create a migration plan for state or schema changes.

## execute_migration
Apply the migration plan.

## deprecate_schema_element
Record a planned retirement of a schema element.

## review_governance_change
Review and approve ontology changes.

## verify_post_migration_conformance
Re-check domain conformance after migration.

---

# 4. Success Condition

Ontology Governance, Versioning, and Migration is successful when Focusa can evolve its ontology stack without losing interoperability, traceability, or system-wide consistency.
