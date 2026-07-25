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
5. compute an attention/recall verdict for critical facts that can change action authority
6. inject only the minimal supporting slice plus any required non-droppable memory anchor
7. preserve the newest operator input and continue conversational/read-only reasoning; gate only durable project-scoped mutation until required verification completes

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

## Critical anchor rule

Some facts are too important to leave inside verbose retrieved context. If a fact can change the next action, it must be promoted into a tiny non-droppable `MEMORY_ANCHOR` or equivalent verdict before Workpoint/Trajectory/tool-output detail.

Critical anchors include:
- latest operator correction or project override
- active task invariant / "do not implement yet" boundary
- current-action authority decision
- latest report/spec summary handle
- destructive-risk or scope-conflict warning
- exact next action when tool-output flood or compaction could hide the thread

This is not permission to inject a large always-on block. The anchor must be bounded, current-ask scoped, suppressible when stale, and verified by the same relevance rules as the rest of the slice.

## Relevance Gate

Before injecting context, Focusa must ask:
- does this support the operator’s current request?
- will this change action quality?
- is this needed now?
- is this more likely to help than distract?
- if this can change action authority, has it been pinned, recapped, or explicitly rejected?

If the answer is no, it should not be injected.

## Steering Reset Rule

When operator input clearly changes the task, Focusa must:
- re-rank the current working set
- suppress stale mission context
- suppress unrelated prior focus state
- compute a current-ask scope/action-authority verdict before Workpoint carryover
- rebuild a new task-relevant slice

## Non-Interruption Rule

- Focusa context injection must never cancel, consume, replace, or delay the newest operator prompt.
- Scope uncertainty can gate durable project writes; it cannot suppress operator steering, direct answers, diagnosis, or read-only verification.
- Tool-output pressure may refresh an internal memory anchor, but a visible recap is optional and must not be a prerequisite for the next action.
- Injected context is advisory to the model. Enforcement belongs at the scoped mutation boundary, not in imperative prose that tells the model to stop.

## Success Condition

This document is satisfied when Focusa/Ontology injection improves reasoning quality while leaving operator intent and subject control fully intact.
