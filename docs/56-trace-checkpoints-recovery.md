# Trace, Checkpoints, and Recovery

## Purpose

This document makes the system inspectable and resumable.

If Focusa is meant to preserve continuity, it must be able to explain:
- what happened
- why it happened
- where to resume after interruption or wrong turn

## Trace Model

Every meaningful unit of work should produce traceable events.

Minimum trace dimensions:
- mission/frame context
- working set used
- constraints consulted
- decisions consulted
- action intents proposed
- tools invoked
- verification results
- ontology deltas applied
- blockers/failures emitted
- final state transition
- operator_subject
- active_subject_after_routing
- steering_detected
- prior_mission_reused
- focus_slice_size
- focus_slice_relevance_score
- subject_hijack_prevented
- subject_hijack_occurred

## Checkpoints

A checkpoint should capture:
- active mission
- active frame/thesis
- current working set identity
- recent decisions/constraints in force
- unresolved blockers/open loops
- recent action chain
- most recent verification state

## Checkpoint Triggers

Create checkpoints on:
- session start
- session compact
- high-impact action completion
- verification completion
- blocker/failure emergence
- explicit resume/fork points
- pre-shutdown

## Resume Semantics

On resume, Focusa should restore:
- active mission/frame
- working set identity
- relevant decisions/constraints
- recent blockers/open loops
- recent verified deltas

## Success Condition

This document is satisfied when a user can answer:
- why did Pi act this way?
- what state was it operating from?
- where should work resume?
- what went wrong and how do we recover?
