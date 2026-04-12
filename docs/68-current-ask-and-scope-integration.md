# Current Ask and Scope Integration

## Purpose

Define how `CurrentAsk`, `QueryScope`, and scope-control concepts integrate with the Domain-General Cognition Core.

This document exists so scope control is not treated as an isolated patch. It should become part of how Focusa models:
- what the agent is doing
- what the agent is answering
- what context is allowed to shape the answer
- what context must be excluded

---

## Core Thesis

The current ask is not just another note in context.

It is a governing object that should shape:
- working-set construction
- answer boundaries
- context inclusion
- context exclusion
- scope verification before response

---

## Integration Rules

1. Every active answer path should have a `CurrentAsk`.
2. Every `CurrentAsk` should be governed by a `QueryScope`.
3. Working sets used for answering should be filtered through current-ask relevance.
4. Adjacent but irrelevant prior context should be explicitly excludable.
5. Scope verification should occur before final answer generation on sensitive or ambiguity-prone turns.

---

## Integration with Core Objects

### Mission / Goal / Task
A `CurrentAsk` may align with, refine, or temporarily override the currently active task focus.

### WorkingSet
The answering working set should be a relevance-filtered subset of the broader active working set.

### Decision / Constraint
Prior decisions and constraints should only influence an answer when the `CurrentAsk` makes them relevant.

### Verification
Scope verification is a first-class verification type.

### Checkpoint
The current ask and scope should be checkpointable when continuity across interruption matters.

---

## Success Condition

Current Ask and Scope Integration is successful when Focusa helps the agent answer the exact question asked, using only relevant prior context, without accidental carryover from neighboring threads.
