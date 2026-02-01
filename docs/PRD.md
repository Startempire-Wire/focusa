# Focusa — Product Requirements Document (PRD)
**Version:** MVP v1.0  
**Status:** Architecture Locked  
**Audience:** Engineers, Agent Implementers, Technical Stakeholders  
**Product Type:** Local Cognitive Runtime / Harness-Agnostic Proxy  
**Relationship:** Standalone now; future NavisAI subsystem  

---

## 1. Product Overview

### 1.1 Product Definition
**Focusa** is a local, harness-agnostic cognitive runtime that sits between an AI harness (Letta, Claude Code, Codex CLI, Gemini CLI, etc.) and an LLM backend to govern **focus, context fidelity, and priority emergence** across long-running sessions.

Focusa does **not** replace:
- agent frameworks
- harnesses
- models

Focusa **augments** them by providing:
- hierarchical focus control
- deterministic context compression
- lossless artifact offloading
- advisory salience surfacing
- minimal, explicit memory

---

## 2. Problem Statement

Long-running AI-assisted workflows fail due to:

1. **Linear context growth**
   - Token bloat
   - Lossy summarization
   - Quadratic attention cost

2. **Loss of task continuity**
   - Nested subtasks collapse into chat history
   - No structured notion of “current focus”

3. **Priority confusion**
   - Everything is treated as equally important
   - No organic surfacing of what *matters now*

4. **Lack of trust**
   - Silent prompt truncation
   - Hidden memory writes
   - Autonomous behavior that hijacks user intent

---

## 3. Goals & Non-Goals

### 3.1 Goals (MVP)
- Maintain stable focus over hours/days
- Prevent unbounded context growth
- Preserve task meaning with structured compression
- Surface priorities without auto-executing them
- Integrate transparently with existing harnesses
- Remain fast and imperceptible

### 3.2 Explicit Non-Goals
- No model training or RL
- No kernel-level attention optimizations
- No autonomous task switching
- No silent memory writes
- No harness replacement
- No cloud dependency

---

## 4. Core Product Principles

1. **Focus over autonomy**
2. **Structure over prose**
3. **Advisory systems over control**
4. **Determinism over magic**
5. **Human intent always wins**
6. **Failure must be visible**

---

## 5. User Personas

### 5.1 Primary
- Developers using AI for:
  - coding
  - debugging
  - system design
  - long technical sessions

### 5.2 Secondary
- Tool builders integrating AI into workflows
- Power users running CLI-based AI tools
- Future NavisAI users

---

## 6. High-Level Architecture (Product View)

```
User / Harness (CLI, Agent Framework)
            |
            v
     Focusa Proxy Layer
            |
   -----------------------
   | Focus Stack (HEC)  |
   | Focus Gate         |
   | ASCC               |
   | ECS                |
   | Memory             |
   | Prompt Assembly    |
   | Background Workers |
   -----------------------
            |
            v
      Model / Provider
```

---

## 7. Core Features (MVP)

### 7.1 Focus Stack (HEC)
**What:**  
A hierarchical execution context that models nested work.

**Requirements:**
- Push, pop, complete frames
- Single active frame at all times
- Completion reasons recorded
- Parent context selectively included in prompts

**Success Metric:**
- Long sessions maintain clear “what we are doing now”

---

### 7.2 Focus Gate (Salience Engine)
**What:**  
RAS-inspired advisory system that surfaces what *might* matter.

**Requirements:**
- Ingest signals (errors, repetition, time, user input)
- Accumulate surface pressure
- Never auto-switch focus
- Allow suppression, resolution, pinning

**Success Metric:**
- Relevant priorities surface without interruption

---

### 7.3 ASCC (Anchored Structured Context Checkpointing)
**What:**  
Incremental, structured summaries per focus frame.

**Requirements:**
- Fixed semantic slots
- Delta-only updates using anchors
- Deterministic merge rules
- Section pinning
- Degradation-safe digest fallback

**Success Metric:**
- Prompt size stabilizes while meaning is preserved

---

### 7.4 ECS (Externalized Context Store)
**What:**  
Lossless offloading of large artifacts with handle-based indirection.

**Requirements:**
- Store large blobs locally
- Replace prompt content with handles
- Explicit rehydration only
- Session-scoped access
- Pinning support

**Success Metric:**
- Zero large blobs inline in prompts

---

### 7.5 Minimal Memory
**What:**  
Small, explicit memory system.

**Requirements:**
- Semantic memory (facts/preferences)
- Procedural memory (rules)
- Explicit user writes only
- Pinning & decay
- Bounded injection into prompts

**Success Metric:**
- Predictable behavior, no personality drift

---

### 7.6 Prompt Assembly Engine
**What:**  
Deterministic construction of minimal prompts.

**Requirements:**
- Slot-based assembly
- Token budgeting
- Explicit degradation strategy
- No silent truncation
- Harness-agnostic formatting

**Success Metric:**
- Stable prompt size and reproducible outputs

---

### 7.7 Background Workers
**What:**  
Async “subconscious” processes.

**Requirements:**
- Never block hot path
- Classification & detection only
- Advisory outputs
- Time-bounded jobs

**Success Metric:**
- No latency impact on user flow

---

## 8. Interfaces

### 8.1 CLI (Primary Control)
- Focus manipulation
- Candidate management
- Memory inspection
- Artifact access
- Event inspection
- JSON mode for scripting

### 8.2 Local API
- HTTP JSON
- CLI, GUI, adapters
- Deterministic endpoints
- Event streaming optional

### 8.3 Menubar GUI
- Passive observability
- Focus stack view
- Candidate surfacing
- Calm, ambient UX

---

## 9. Performance Requirements

| Area | Requirement |
|----|----|
| Proxy overhead | < 20ms typical |
| Prompt assembly | Deterministic, bounded |
| Workers | Async, non-blocking |
| Storage | Local, fast I/O |
| Long sessions | Hours/days without reset |

---

## 10. Trust & Safety Requirements

- No silent prompt changes
- No hidden memory writes
- No autonomous focus switching
- Explicit degradation warnings
- Session isolation guaranteed
- Replayable event log

---

## 11. Persistence & State

**Stored Locally:**
- Focus stack
- ASCC checkpoints
- ECS artifacts
- Memory
- Sessions
- Event log

**Requirements:**
- Survive restarts
- No cross-session leakage
- Deterministic recovery

---

## 12. Success Criteria (MVP)

The MVP is successful when:

1. Long sessions remain coherent
2. Prompt size plateaus
3. Focus never auto-shifts
4. Priorities surface meaningfully
5. Failures are visible
6. Works with a real harness as a proxy
7. CLI-only usage is sufficient

---

## 13. Out of Scope (MVP)

- Multi-agent orchestration
- Model fine-tuning
- Attention kernel optimization
- Cloud sync
- Infinite canvas visualization
- Autonomous planning

---

## 14. Risks & Mitigations

| Risk | Mitigation |
|----|----|
| Over-complexity | Strict MVP scope |
| Performance regression | Async workers + budgets |
| Loss of trust | Explicit invariants |
| Harness incompatibility | Adapter capability declaration |

---

## 15. Roadmap (Post-MVP)

- Multi-session workspace management
- Visual infinite canvas
- Replay & time-travel debugging
- NavisAI integration
- Advanced salience heuristics
- Optional semantic retrieval

---

## 16. One-Sentence Product Statement

> **Focusa is a local cognitive runtime that governs focus, compresses context without loss, and integrates transparently with existing AI harnesses—so long-running work stays clear, stable, and human-directed.**

---

# UPDATE — Cognitive Governance Reframe

# PRD — Focusa (UPDATED)

## Product Vision

Focusa is a **cognitive governance layer** that preserves meaning, intent,
and trust across long-running AI work — even under context compression and autonomy.

It enables:
- earned autonomy
- verifiable learning
- explicit human control
- long-horizon operation (days to weeks)

---

## Core Problem

AI agents today:
- drift silently over time
- lose context under compaction
- cannot explain why behavior changed
- cannot earn trust in a measurable way

Focusa solves this by separating:
- cognition from execution
- learning from identity
- autonomy from authority

---

## Key Differentiator (NEW)

**Explicit Constitutional Evolution**

Agents do not silently change how they reason.
Instead, Focusa introduces a **Constitution Synthesizer (CS)** that:

- analyzes long-term evidence
- proposes constitutional refinements
- requires human review
- preserves full version history
- allows one-click rollback

This enables growth **without drift**.

---

## In-Scope for MVP (Updated)

### Core
- Focus State + Focus Stack
- Focus Gate
- Reference Store
- Expression Engine
- Intuition Engine
- Task Authority integration (Beads)

### Learning & Trust
- UXP (User Experience Profile)
- UFI (User Friction Index)
- Autonomy Reliability Index (ARI)

### Agent System
- Persistent Agent abstraction
- Versioned Agent Constitutions
- CS draft generation (read-only)
- Constitution diff + evidence view
- Manual activation + rollback

### Interfaces
- CLI (authoritative)
- Menubar GUI (inspection + control)

---

## Explicitly Out of Scope (MVP)

- Automatic constitution activation
- Runtime constitution mutation
- Emotional modeling
- Self-generated goals
- Cross-user constitution sharing

---

## Success Metrics

- Ability to explain **why** behavior changed
- Ability to run autonomously for extended periods
- Zero silent changes to agent identity
- User trust and inspectability
- Deterministic replay of decisions

---

## Product Principle (Updated)

> Focusa allows agents to **grow in capability**
> without **drifting in identity**.

---

## Long-Term Direction

- Multi-agent systems with distinct constitutions
- Constitution templates per domain
- Cross-model calibration using shared evidence
- Institutional-grade AI governance
