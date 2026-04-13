# Identity and Reference Resolution

## Purpose

Define how Focusa determines when multiple observations, artifacts, proposals, or domain objects refer to the same underlying entity.

This document exists to prevent fragmentation across domains and iterations.

Without identity resolution, the same:
- task
- component
- artifact
- affordance
- decision
- actor

may appear as multiple disconnected objects, weakening consistency.

---

## Core Thesis

A coherent ontology needs more than typed objects.

It also needs a canonical way to say:
- these refer to the same thing
- this is the authoritative representative
- this newer representation supersedes or refines the older one

Identity resolution is therefore a foundational consistency mechanism.

---

## Design Laws

1. Canonical identity must be explicit.
2. Resolution should preserve provenance rather than erase history.
3. Identity merges should be reviewable when ambiguous.
4. Supersession and equivalence are not the same thing.
5. Cross-domain identity must be possible.

---

# 1. Core Object Types

## CanonicalEntity
Represents the authoritative identity for an underlying entity.

### Required properties
- `id`
- `entity_kind`
- `status`

---

## ReferenceAlias
Represents an alternate name, handle, path, or representation that may point to a canonical entity.

### Required properties
- `id`
- `alias_kind`
- `status`

---

## ResolutionCandidate
Represents a possible identity match awaiting confirmation.

### Required properties
- `id`
- `candidate_kind`
- `status`

---

## ResolutionDecision
Represents the outcome of a reference-resolution judgment.

### Required properties
- `id`
- `decision_kind`
- `status`

---

## SupersessionRecord
Represents a relationship where one representation supersedes another.

### Required properties
- `id`
- `record_kind`
- `status`

---

# 2. Core Relation Types

## aliases
Source:
- ReferenceAlias

Target:
- CanonicalEntity

---

## candidate_for
Source:
- ResolutionCandidate

Target:
- CanonicalEntity

---

## resolved_as
Source:
- ResolutionDecision

Target:
- CanonicalEntity
- ResolutionCandidate

---

## equivalent_to
Source:
- CanonicalEntity

Target:
- CanonicalEntity

Use only when true equivalence is established.

---

## supersedes_entity
Source:
- SupersessionRecord
- CanonicalEntity

Target:
- CanonicalEntity

---

# 3. Core Action Types

## detect_aliases
Find potential aliases and alternative references.

## build_resolution_candidates
Construct possible matches across observations/domains.

## resolve_identity
Select the canonical entity or create one.

## verify_resolution
Check whether the resolution is sufficiently supported.

## record_supersession
Record when a newer representation supersedes an older one.

---

# 4. Success Condition

Identity and Reference Resolution is successful when Focusa can maintain canonical identity across domains, turns, and iterations without losing provenance or creating silent fragmentation.
