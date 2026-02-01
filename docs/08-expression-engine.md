# docs/08-expression-engine.md — Expression Engine (MVP)

## Purpose

The **Expression Engine** converts the current Focus State into language suitable for model invocation.

It governs *what is said now*, not *what is known*.

---

## Core Invariants

1. Deterministic output
2. Explicit structure
3. Bounded token usage
4. No silent truncation
5. No reasoning or planning

---

## Input

- Focus State (active frame)
- Selected parent frame context
- Optional surfaced candidates (annotated)
- Invocation metadata

---

## Output Structure (Canonical)

1. System framing
2. Active intent
3. Constraints
4. Decisions
5. Relevant artifacts (handles only)
6. Failures (if relevant)
7. Next steps
8. Invocation-specific instructions

---

## Token Budgeting

### Priority Order
1. Intent
2. Constraints
3. Decisions
4. Current state
5. Next steps
6. Failures
7. Artifacts

Lower-priority sections are truncated first.

All truncation is:
- explicit
- logged
- reversible

---

## Degradation Strategy

If budget exceeded:
- emit degradation event
- annotate missing sections
- never silently drop meaning

---

## Forbidden Behaviors

- Implicit summarization
- Dynamic prompt shaping
- Content inference
- Memory mutation

---

## Acceptance Criteria

- Output is reproducible
- Token usage predictable
- Meaning preserved
- Failures visible

---

## Summary

The Expression Engine ensures **clarity without overload**, expressing only what matters *now*.
