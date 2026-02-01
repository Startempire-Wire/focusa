# docs/03-focus-stack.md — Focus Stack & Focus Frames (MVP)

## Purpose

The **Focus Stack** models human attention by representing work as a hierarchy of nested Focus Frames.

It replaces:
- linear conversation history
- implicit task switching
- unstructured “what are we doing?” state

---

## Core Invariants

1. Exactly one active Focus Frame exists at any time
2. Every Focus Frame has a concrete intent
3. Every Focus Frame maps to a Beads issue
4. Frames are entered and exited explicitly
5. Completed frames are archived, not forgotten

---

## Focus Frame Definition

A **Focus Frame** represents a single unit of focused work.

### Required Fields
- frame_id
- title
- goal
- beads_issue_id
- focus_state
- created_at
- status: active | suspended | completed
- completion_reason (when completed)

---

## Completion Reasons

When a frame is closed, it MUST include a reason:

- goal_achieved
- blocked
- abandoned
- superseded
- error

This reason is persisted and reflected in Focus State.

---

## Stack Behavior

### Push Frame
- Validates Beads issue
- Suspends current frame (if any)
- Creates new active frame

### Pop / Complete Frame
- Requires completion reason
- Archives Focus State
- Restores parent frame as active

### Suspend Frame
- Temporarily deactivates frame
- Preserves state
- Used for interruptions

---

## Parent Context Rules

When assembling Focus State:
- Active frame is always included
- Parent frames contribute selectively:
  - intent
  - decisions
  - constraints
- Artifacts from parent frames included only if referenced

---

## Invalid Operations (Forbidden)

- Multiple active frames
- Implicit frame switching
- Editing archived frames
- Frames without Beads linkage
- Skipping completion reasons

---

## Interaction with Other Components

### Intuition Engine
- May observe frame duration
- May emit time-based signals

### Focus Gate
- May surface candidates related to inactive frames
- Never auto-resumes frames

### Expression Engine
- Receives serialized Focus State derived from stack

---

## Failure Modes & Protections

- Frame push fails if Beads issue invalid
- Frame pop fails if reason missing
- Stack corruption triggers hard error

---

## Acceptance Criteria

- Stack always reflects actual focus
- Frame transitions are explicit
- Parent context remains bounded
- Long task nesting remains readable

---

## Summary

The Focus Stack ensures the system is **always doing exactly one thing, on purpose**, and remembers why.
