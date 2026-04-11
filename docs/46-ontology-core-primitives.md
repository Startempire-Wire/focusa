# Ontology Core Primitives

## Purpose

This document defines the irreducible primitive set required to make the Ontology operational inside Focusa.

These primitives must be:
- reducer-expressible
- persistence-friendly
- replayable
- bounded
- auditable

## Primitive Set

## 1. ObjectType
A typed class of entity in the software/work world.

Required fields:
- `type_name`
- `id_strategy`
- `required_properties`
- `allowed_links`
- `allowed_actions`
- `status_vocabulary`

## 2. Property
A typed field belonging to an object.

## 3. LinkType
A typed directional relationship between two objects.

Required fields:
- `name`
- `source_types`
- `target_types`
- `multiplicity`
- `directionality`
- `evidence_policy`
- `promotion_policy`

## 4. ActionType
A typed operation over ontology objects.

Required fields:
- `name`
- `target_types`
- `input_schema`
- `preconditions`
- `side_effects`
- `verification_hooks`
- `revertability`
- `emitted_events`

## 5. Status
A typed state label applied to objects, links, actions, or proposals.

Minimum statuses:
- active
- speculative
- blocked
- verified
- stale
- deprecated
- canonical
- experimental

## 6. ObjectSet
A bounded set of ontology objects used for current cognition.

## 7. Constraint
A rule that should shape action selection.

## 8. Mission
A typed objective context that binds work together.

## 9. ProvenanceRecord
Tracks where a fact came from.

Provenance classes:
- parser_derived
- tool_derived
- user_asserted
- model_inferred
- reducer_promoted
- verification_confirmed

## 10. VerificationRecord
Captures whether an object, link, or action result has been verified.

## 11. OntologyDelta
The unit of ontology change.

## 12. SlicePolicy
Defines how ontology data is selected for expression.

## Primitive Design Laws

1. No primitive may require freeform interpretation to be valid.
2. Every canonical primitive must be replayable through reducer events.
3. Every proposal primitive must be bounded and expirable.
4. Provenance and verification are mandatory for long-lived truth.
5. Object sets are first-class, not derived afterthoughts.
