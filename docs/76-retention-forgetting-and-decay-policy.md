# Retention, Forgetting, and Decay Policy

## Purpose

Define how Focusa decides what should remain:
- canonical
- active
- decayed
- superseded
- archived
- pruned from active use

This document exists because quality degrades not only from missing knowledge, but also from stale knowledge that remains too active for too long.

---

## Core Thesis

A coherent ontology needs memory discipline.

Focusa should explicitly model:
- what should remain permanently important
- what should decay in relevance
- what should leave active working memory
- what should remain historically available but no longer govern behavior

Forgetting is not failure.
Uncontrolled retention is failure.

---

## Design Laws

1. Canonical truth and active relevance are not the same thing.
2. Superseded knowledge should remain traceable without remaining behaviorally dominant.
3. Decay should be explicit and evidence-aware.
4. Pruning should prefer active-use reduction over destructive loss.
5. Historical trace should remain available where governance or recovery requires it.

---

# 1. Core Object Types

## RetentionPolicy
Represents how long and how strongly an object should remain active or preserved.

### Required properties
- `id`
- `policy_kind`
- `status`

---

## DecayProfile
Represents how an object's relevance declines over time or after supersession.

### Required properties
- `id`
- `decay_kind`
- `status`

---

## ArchiveState
Represents an object that is no longer active but still retained historically.

### Required properties
- `id`
- `archive_kind`
- `status`

---

## PruningDecision
Represents a deliberate reduction of active-use visibility.

### Required properties
- `id`
- `decision_kind`
- `status`

---

# 2. Core Relation Types

## retained_under
Source:
- CanonicalEntity
- Decision
- Constraint
- WorkingSet
- EvidenceArtifact

Target:
- RetentionPolicy

---

## decays_via
Source:
- CanonicalEntity
- Decision
- Constraint
- WorkingSet

Target:
- DecayProfile

---

## archived_as
Source:
- CanonicalEntity
- Checkpoint
- EvidenceArtifact

Target:
- ArchiveState

---

## pruned_by
Source:
- PruningDecision

Target:
- WorkingSet
- Projection
- RelevantContextSet

---

# 3. Core Action Types

## evaluate_retention
Decide whether an object should remain active, canonical, decayed, or archived.

## apply_decay
Reduce an object's active relevance according to policy.

## archive_object
Preserve historical trace while removing active dominance.

## prune_active_context
Reduce what remains in active working memory or answer scope.

## restore_from_archive
Return an archived object to active use when justified.

---

# 4. Success Condition

Retention, Forgetting, and Decay Policy is successful when Focusa can preserve what matters, decay what no longer should dominate, and keep active reasoning clear without destroying historical trace.
