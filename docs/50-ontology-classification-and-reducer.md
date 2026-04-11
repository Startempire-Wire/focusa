# Ontology Classification and Reducer Integration

## Purpose

This document defines:
- who performs classification
- what kinds of classification are allowed
- how canonical ontology state is written

## Classification Roles

### Deterministic Classifiers
Responsibilities:
- object typing from code/config/test structure
- package/module/file/symbol mapping
- import and call graph extraction
- route and endpoint extraction
- schema and migration linkage
- diff-to-object mapping
- test/build result linkage
- explicit config relation extraction

### Background Models
Responsibilities:
- candidate working-set membership
- semantic grouping of related changes
- candidate task-to-code linkage
- candidate convention/risk suggestions
- candidate status suggestions
- candidate supersession or blocker inferences

Prohibitions:
- no direct canonical ontology writes
- no invented object types on the fly
- no permanent inferred clutter without expiry
- no silent override of deterministic evidence

### Reducer
Reducer is the sole canonical write boundary.

Responsibilities:
- accept/reject ontology proposals
- apply canonical ontology deltas
- apply verification-backed promotions
- update working-set membership canonically
- write status changes
- persist replayable events

## Proposal Model

Any ambiguous classification must enter as a proposal.

## Background Model Guardrails

Every background-model output must be:
- typed
- bounded
- evidence-linked
- confidence-scored
- expiring
- proposal-only
- reducer-ingestible

## Ontology Reducer Events

Minimum events:
- `ontology_object_upsert_proposed`
- `ontology_link_upsert_proposed`
- `ontology_status_change_proposed`
- `ontology_working_set_membership_proposed`
- `ontology_proposal_promoted`
- `ontology_proposal_rejected`
- `ontology_verification_applied`
- `ontology_working_set_refreshed`

## Failure Discipline

If extraction or proposal systems fail:
- no silent canonical writes may occur
- current canonical ontology state remains authoritative
- the system may continue in degraded mode
- failure signals should be emitted for observability

## Success Condition

This document is satisfied when the ontology can evolve in real time without allowing silent model-driven truth drift.
