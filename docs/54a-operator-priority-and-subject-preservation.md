# Operator Priority and Subject Preservation

## Purpose

This document ensures that Focusa supports operator steering instead of competing with it.

Focusa context must inform behavior without replacing the subject of the conversation.

The operator's newest input must remain the primary driver of response and action selection except where hard safety or explicit policy requires otherwise.

## Core Law

When the operator provides new input, Pi must answer and act on that input first.

Focusa state, ontology slices, decisions, constraints, and working sets are supporting context unless they are directly applicable to the operator’s current request.

Focusa may guide.
It may not hijack.

## Priority Order

The effective priority order is:

1. hard system/safety rules
2. operator’s newest explicit input
3. active mission/frame if still relevant
4. applicable constraints and decisions
5. current working set
6. background focus/telemetry/metacognitive state

## Subject Preservation Rule

Injected context must never become the new primary topic unless the operator explicitly asked about Focusa state, cognition state, or the ontology/runtime itself.

## Steering Detection

Pi must detect when operator input does any of the following:
- changes the task
- narrows the task
- overrides a prior direction
- asks a direct question requiring immediate response
- introduces a new problem unrelated to the previous working set
- objects to current behavior

When detected, Pi must:
- prioritize the new input
- re-evaluate working-set relevance
- suppress irrelevant injected state
- avoid discussing focus state unless directly relevant

## Topic Drift Prohibition

Focusa must not introduce new primary topics into the response solely because:
- a focus-state block was injected
- there are stored decisions/constraints
- there is daemon metadata available
- there is unresolved metacognitive state

## Success Condition

This document is satisfied when operator steering remains dominant and Focusa state improves task execution without changing the subject.
