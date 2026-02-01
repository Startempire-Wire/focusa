# docs/39-thread-lifecycle-spec.md
## Thread Lifecycle — AUTHORITATIVE SPEC

This document defines **Threads** as first-class cognitive containers in Focusa
and specifies their lifecycle, guarantees, and integration points.

---

## 1. Definition

> A **Thread** is a persistent cognitive workspace that binds:
> - a Thread Thesis
> - a Context Lineage Tree (CLT)
> - a Focus Stack
> - a Reference Store namespace
> - telemetry and autonomy history

Threads are the **unit of continuity** in Focusa.

---

## 2. Thread Identity

Each Thread has a globally unique identity.

```json
{
  "thread_id": "uuid",
  "name": "string",
  "status": "active | paused | archived",
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

Thread identity is stable across restarts, compaction, and exports.

---

## 3. Thread Creation

### 3.1 New Thread

Triggered by:
- explicit user action
- API call
- CLI command

Effects:
- create new thread_id
- create new CLT root
- initialize empty Focus Stack
- initialize new Thread Thesis (low confidence)
- create isolated Reference Store namespace
- reset telemetry counters

No state is inherited unless explicitly requested.

---

### 3.2 New Thread With Inheritance

Optional inheritance flags:
- constitution
- preferences
- reference subset
- calibration profile

Inheritance is **explicit**, never implicit.

---

## 4. Thread Continuation (Resume)

Resuming a thread rehydrates:
- latest Thread Thesis version
- CLT active head
- Focus Stack
- autonomy profile
- cache permission state

Resuming does NOT:
- replay conversation
- re-inject full history
- auto-escalate autonomy

---

## 5. Thread Save (Checkpoint)

Saving a thread:
- commits Focus Stack head
- persists Thesis version
- snapshots autonomy + telemetry state
- records checkpoint marker in CLT

Save is **idempotent** and lightweight.

---

## 6. Thread Rename

Rename:
- updates human-readable metadata only
- does not alter cognition
- does not affect lineage or autonomy

Rename is always reversible.

---

## 7. Thread Fork

Forking creates a **new thread** from an existing CLT node.

Effects:
- new thread_id
- selected CLT node becomes new root
- Thesis is cloned (with reduced confidence)
- Focus Stack resets
- Reference Store optionally pruned

Forking preserves exploration while preventing cognitive contamination.

---

## 8. Thread Archive

Archiving:
- freezes thread state
- disallows new Focus Frames
- allows inspection and export
- preserves telemetry for training

Archived threads are immutable.

---

## 9. Guarantees

- Threads never share mutable state
- One active Thread per agent session
- CLT nodes belong to exactly one Thread
- Telemetry is thread-scoped
- Autonomy is thread-specific

---

## 10. Why Threads Matter

Threads:
- prevent goal entanglement
- enable long-running autonomy
- make cognition navigable
- provide clean training boundaries

Threads are **the spine of Focusa**.

---
