# PRD.md — Focusa Product Requirements Document (MVP)

**Product:** Focusa  
**Version:** MVP v1.0  
**Status:** Architecture Locked  
**Category:** Local Cognitive Runtime  
**Audience:** Engineers, Agent Implementers, System Designers  

---

## 1. Product Definition

Focusa is a local cognitive runtime that preserves focus, intent, and meaning across long-running AI sessions by externalizing cognition from conversation.

It integrates transparently with existing AI harnesses and models without modifying them.

---

## 2. Problem Statement

Current AI systems fail at long-running work because:

1. **Conversation is treated as memory**
2. **Automatic compaction silently destroys meaning**
3. **Intent and constraints drift**
4. **Artifacts vanish**
5. **Models repeat work or regress**

This results in:
- lost context
- repeated mistakes
- unstable behavior
- reduced trust

---

## 3. Goals (MVP)

- Preserve meaning across compaction
- Maintain a single coherent focus
- Prevent unbounded context growth
- Surface priorities without interruption
- Remain fast and imperceptible
- Integrate without harness modification

---

## 4. Non-Goals (Explicit)

- Autonomous task execution
- Agent orchestration
- Model training or RL
- Attention kernel optimization
- Multi-device local-first sync (post-MVP; ownership + observations)
- CI/CD automation
- Infinite canvas UI

---

## 5. Core Components & Requirements

### 5.1 Focus State
- Structured representation of intent and decisions
- Incrementally updated
- Anchored and deterministic
- Injected into every model call

**Success Metric:**  
Intent survives compaction without drift.

---

### 5.2 Focus Stack
- Hierarchical focus frames
- Exactly one active frame
- Explicit completion reasons
- Parent context bounded

**Success Metric:**  
Clear “what are we doing now” at all times.

---

### 5.3 Intuition Engine
- Async, non-blocking
- Detects repetition, errors, time pressure
- Emits signals only

**Constraints:**  
May not mutate Focus State or Focus Stack.

---

### 5.4 Focus Gate
- Applies decay and pressure
- Surfaces candidates only
- Supports pinning and suppression
- Never auto-acts

---

### 5.5 Reference Store
- External artifact storage
- Handle-based references
- Explicit rehydration
- Session-scoped

---

### 5.6 Expression Engine
- Deterministic prompt construction
- Slot-based structure
- Token budgeting
- Explicit degradation strategy
- No silent truncation

---

## 6. Trust & Safety Requirements

- No silent prompt changes
- No hidden memory writes
- No autonomous focus switching
- All degradation visible
- Session isolation enforced
- Replayable state via event log

---

## 7. Interfaces

### CLI
- Focus control
- Candidate inspection
- Reference Store access
- Debugging and events

### Local API
- JSON HTTP
- Used by CLI, GUI, adapters

### GUI (Menubar)
- Passive observability only
- No blocking interactions

---

## 8. Performance Requirements

| Area | Requirement |
|---|---|
| Proxy overhead | <20ms typical |
| Prompt assembly | Deterministic |
| Background work | Async only |
| Long sessions | Hours+ stable |

---

## 9. Persistence

Persist locally:
- Focus Stack
- Focus State
- Reference Store artifacts
- Beads mappings
- Sessions
- Event log

Must survive restarts.

---

## 10. Success Criteria (MVP)

The MVP is complete when:

1. Long sessions remain coherent
2. Compaction does not destroy intent
3. Focus never auto-switches
4. Artifacts are never lost
5. Failures are observable
6. Works with real harnesses
7. CLI-only usage is sufficient

---

## 11. Future Directions (Post-MVP)

- Visual focus canvas
- Replay / time-travel debugging
- NavisAI integration
- Advanced intuition heuristics
- Multi-workspace support

---

## 12. Product Statement

> **Focusa ensures that AI systems maintain a stable state of mind across time, even as conversation and context inevitably collapse.**
