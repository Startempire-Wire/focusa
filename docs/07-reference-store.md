# docs/07-reference-store.md — Reference Store (MVP)

## Purpose

The **Reference Store** is Focusa’s externalized, lossless memory system for large or durable artifacts that must not live in the prompt.

It prevents token overload while preserving full fidelity.

---

## Core Invariants

1. Artifacts are never implicitly injected
2. Artifacts are referenced by handles only
3. Artifacts are immutable once written
4. Rehydration is explicit and auditable
5. Storage is session-scoped by default

---

## Artifact Model

### Artifact Fields
- artifact_id
- type (diff, log, output, file, note)
- summary (≤ 2 lines)
- storage_uri
- created_at
- session_id
- pinned (bool)

---

## Storage Backends (MVP)

- Local filesystem (default)
- Workspace-relative paths
- No remote storage

All writes are atomic.

---

## Handles & References

Artifacts are referenced in Focus State using:

```
@ref:<artifact_id>
```

The handle:
- is stable
- contains no content
- enables explicit rehydration

---

## Rehydration

Rehydration occurs only when:
- explicitly requested by user or agent
- explicitly approved by Focus Gate (if needed)

Rehydrated content:
- is bounded
- may be partially loaded
- is logged as an event

---

## Pinning

Pinned artifacts:
- bypass garbage collection
- persist across sessions
- require explicit unpinning

---

## Garbage Collection

Unpinned artifacts may be GC’d when:
- session is closed
- artifact age exceeds threshold
- no references remain

GC emits events and never deletes pinned artifacts.

---

## Forbidden Behaviors

- Automatic artifact injection
- Semantic inference over artifacts
- Silent deletion
- Cross-session leakage

---

## Acceptance Criteria

- Large artifacts never enter prompts
- Rehydration is always intentional
- Artifacts remain lossless
- Token usage remains bounded

---

## Summary

The Reference Store ensures **nothing important is lost**, while **nothing unnecessary clutters cognition**.
