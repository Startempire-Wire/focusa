# Ontology Links and Actions

## Purpose

This document defines:
- the typed relationships between ontology objects
- the typed actions that may be performed over them

This is what turns the ontology from storage into an operational world model.

## Link Types

Required link types:
- imports
- calls
- renders
- persists_to
- depends_on
- configured_by
- tested_by
- implements
- violates
- blocks
- supersedes
- belongs_to_goal
- verifies
- derived_from

## Link Policies

1. Every canonical link must have evidence.
2. Model-inferred links must start as proposals unless policy says otherwise.
3. Links may expire if their evidence goes stale.
4. Working-set builders may rank by link confidence and freshness.

## Action Types

Required action types:
- refactor_module
- modify_schema
- add_route
- add_test
- verify_invariant
- promote_decision
- mark_blocked
- resolve_risk
- complete_task
- rollback_change

For every action define:
- target types
- preconditions
- verification hooks
- side effects
- revert behavior
- emitted reducer-visible events

## Action Policies

1. Every action must map to concrete toolable behavior.
2. Every action must emit reducer-visible events.
3. Every action should have verification hooks.
4. High-risk actions require applicable-constraint checks first.
5. Every action result should be able to produce ontology deltas.
