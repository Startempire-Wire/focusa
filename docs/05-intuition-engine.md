# docs/05-intuition-engine.md — Intuition Engine (MVP)

## Purpose

The **Intuition Engine** is the subconscious layer responsible for forming weak signals from ongoing activity.

It:
- observes
- aggregates
- correlates
- suggests

It never decides, acts, or speaks.

---

## Core Invariants

1. Runs asynchronously only
2. Cannot block the hot path
3. Cannot mutate Focus State or Focus Stack
4. Emits signals, not commands
5. All signals are explainable

---

## Signal Sources (MVP)

### Temporal Signals
- Frame duration exceeds expected bounds
- Prolonged inactivity

### Repetition Signals
- Repeated errors
- Repeated edits
- Repeated tool invocations

### Consistency Signals
- Contradictory decisions
- Drift between stated intent and actions

### Structural Signals
- Deep stack nesting
- Frequent frame switching

---

## Signal Model

Each signal includes:
- signal_id
- signal_type
- severity
- related_frame_id
- metadata
- timestamp

Signals are ephemeral until promoted by Focus Gate.

---

## Aggregation

Signals are aggregated by:
- type
- related frame
- time window

Aggregation produces:
- cumulative pressure
- summarized description

---

## Emission

Aggregated signals are emitted to the Focus Gate.

Emission:
- is idempotent
- updates existing candidates where possible
- creates new candidates only when necessary

---

## Observability

The Intuition Engine emits:
- intuition.signal.created
- intuition.signal.updated
- intuition.signal.expired

---

## Performance Constraints

- Zero blocking
- Bounded memory
- O(1) per signal processing target

---

## Forbidden Behaviors

- Writing memory
- Altering focus
- Triggering actions
- Injecting prompt content

---

## Acceptance Criteria

- Signals reflect real patterns
- No noise storms
- No runaway pressure
- Fully explainable output

---

## Summary

The Intuition Engine forms **subconscious awareness**, quietly shaping what *might* matter without demanding attention.
