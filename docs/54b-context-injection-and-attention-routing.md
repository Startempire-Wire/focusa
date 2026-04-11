# Context Injection and Attention Routing

## Purpose

This document defines how injected Focusa/Ontology context should be selected and routed so that it supports the current task without dominating it.

The problem is not only visible echoing.
The problem is that overly prominent injected context can distort model attention and change the subject.

## Core Principle

Injected context must be:
- relevant
- bounded
- secondary
- task-scoped
- suppressible when operator steering changes

It must not be a large always-on block that competes with the operator's newest input.

## Injection Model

Focusa should not inject a monolithic state block into every turn.

Instead, it should inject a **minimal applicable slice** chosen after operator-input interpretation.

### Injection sequence
1. read newest operator input
2. determine current subject/task intent
3. determine whether prior mission/frame is still applicable
4. compute applicable constraints/decisions/working-set members
5. inject only the minimal supporting slice
6. generate response/action

## Minimal Applicable Slice

A minimal applicable slice may include:
- current mission if still relevant
- only applicable constraints
- only relevant prior decisions
- only relevant working-set objects
- only recent verified deltas that matter to the current ask

A minimal applicable slice must exclude:
- full focus-state blocks
- unrelated open questions
- unrelated decisions
- unrelated telemetry
- irrelevant daemon summaries
- broad metacognitive prose

## Relevance Gate

Before injecting context, Focusa must ask:
- does this support the operator’s current request?
- will this change action quality?
- is this needed now?
- is this more likely to help than distract?

If the answer is no, it should not be injected.

## Steering Reset Rule

When operator input clearly changes the task, Focusa must:
- re-rank the current working set
- suppress stale mission context
- suppress unrelated prior focus state
- rebuild a new task-relevant slice

## Success Condition

This document is satisfied when Focusa/Ontology injection improves reasoning quality while leaving operator intent and subject control fully intact.
