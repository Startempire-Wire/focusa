# docs/06-focus-state.md — Focus State (MVP)

## Purpose

The **Focus State** represents the system’s current state of mind.

It is the **authoritative carrier of meaning** across turns and survives context compaction intact.

---

## Core Invariants

1. Focus State is explicit and structured
2. Focus State is deterministic
3. Focus State is incrementally updated
4. Focus State is injected every turn
5. Focus State never inferred implicitly

---

## Structure

### Required Sections
- intent
- decisions
- constraints
- artifacts (references only)
- failures
- next_steps
- current_state

Each section may be empty but must exist.

---

## Update Rules

### Incremental Updates
- Only changed sections are updated
- No full regeneration
- Anchored to frame lifecycle

### Conflict Handling
- Contradictions must be logged
- Prior decisions preserved
- Resolution recorded explicitly

---

## Artifacts

Artifacts are stored in the Reference Store.

Focus State contains:
- artifact handles
- brief descriptors
- no large content

---

## Persistence

Focus State is persisted:
- per Focus Frame
- per Session
- on every mutation

---

## Injection Policy

Every model invocation includes:
- serialized Focus State
- deterministic ordering
- bounded token budget

If budget exceeded:
- lower-priority sections truncated first
- truncation is explicit and logged

---

## Forbidden Behaviors

- Implicit summarization
- Silent overwrites
- Hidden inference
- Mixing conversation with state

---

## Acceptance Criteria

- Focus State always reflects reality
- Compaction does not destroy intent
- Decisions never vanish
- Failures remain visible

---

## Summary

The Focus State is the **single source of truth for meaning**, ensuring continuity of mind regardless of context loss.
