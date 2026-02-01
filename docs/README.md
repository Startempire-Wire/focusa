# Focusa

**Focusa** is a local-first cognitive governance framework for AI agents.

It is not a chatbot.  
It is not an agent framework.  
It is the system that makes agents *trustworthy over time*.

---

## What Focusa Solves

- Context loss under compaction
- Silent behavioral drift
- Unverifiable autonomy
- Unexplainable learning
- Long-running task incoherence

---

## Core Concepts

- **Focus State** – what the system is currently doing
- **Focus Stack** – structured attention over tasks
- **Focus Gate** – what is allowed to surface
- **Reference Store** – lossless external memory
- **Expression Engine** – language output
- **Intuition Engine** – subconscious signal formation
- **UXP / UFI** – transparent experience calibration
- **Autonomy Reliability Index** – earned trust
- **Agent** – persistent behavioral persona
- **Agent Constitution (ACP)** – immutable reasoning charter
- **Constitution Synthesizer (CS)** – evidence-driven constitution drafting

---

## Agents & Constitutions

Agents in Focusa are **not models**.

An agent defines:
- behavioral defaults
- policy constraints
- learning permissions
- a versioned constitution

The constitution defines **how the agent reasons when uncertain**.

Constitutions:
- do not change at runtime
- are versioned
- are human-ratified
- are rollback-safe

---

## Constitution Synthesizer (CS)

CS is a **design-time assistant**, not a runtime system.

It:
- analyzes UXP, UFI, ARI, and task outcomes
- detects normative tensions
- proposes draft constitution updates
- provides evidence-linked diffs
- never auto-applies changes

This allows agents to improve **without identity drift**.

---

## Interfaces

### CLI (Primary)
- Inspect focus, autonomy, and trust
- Review constitution diffs
- Activate / rollback constitutions
- Control agents explicitly

### Menubar GUI
- Visualize focus and autonomy
- Inspect evidence and calibration
- Adjust preferences via sliders
- Review CS suggestions visually

---

## Design Philosophy

- No silent mutation
- No inferred emotion
- No hidden state
- No unbounded autonomy
- Everything explainable
- Everything reversible

---

## Why This Matters

Focusa enables AI systems that can:
- run for days or weeks
- improve over time
- adapt to users
- remain inspectable
- earn autonomy safely

This is **institutional intelligence**, not novelty AI.

---

## Status

Focusa is under active development.
The MVP focuses on correctness, transparency, and trust.

---

> *Agents grow by learning how to act within their values — not by rewriting them.*

---

# EXTENDED README (Full Detail)

# README.md — Focusa

**Focusa** is a local cognitive runtime that preserves **focus, intent, and meaning** across long-running AI sessions by separating *cognition* from *conversation*.

Focusa sits transparently between an AI harness (Claude Code, Codex CLI, Gemini CLI, Letta, etc.) and a model backend. It does **not** replace agents, models, or frameworks. Instead, it governs *what the system is focused on* and *what meaning must persist* when context inevitably compacts.

---

## The Problem Focusa Solves

Modern AI systems fail in long sessions because:

- Conversation history is treated as memory
- Automatic compaction silently deletes meaning
- Intent, constraints, and decisions drift or vanish
- Models “forget what they were doing”
- Repeated work and regressions occur

This is not a token problem.  
It is a **continuity of mind** problem.

---

## The Core Insight

> **Meaning should never live only in conversation.**

Focusa extracts, structures, and persists meaning *outside* the model so that compaction never destroys intent.

---

## What Focusa Is

- A **cognitive runtime**
- A **focus and intent operating layer**
- **Harness-agnostic**
- **Local-first**
- **Deterministic**
- **Human-aligned**

## What Focusa Is Not

- Not a model
- Not an agent framework
- Not an automation engine
- Not a RAG system
- Not a scheduler
- Not autonomous

---

## Cognitive Architecture

Focusa models cognition explicitly using human-readable components:

### Focus State
The system’s current **state of mind**:
- what it is doing
- why it is doing it
- what has been decided
- what must remain true

This state is injected into every model invocation and survives context compaction.

---

### Focus Stack
A hierarchical structure that models **nested attention**.

- Exactly one active Focus Frame at a time
- Parent frames contribute selectively
- Completed frames are archived, not forgotten

This replaces linear chat history with intentional structure.

---

### Intuition Engine
The **subconscious** layer.

- Runs asynchronously
- Detects patterns, anomalies, repetition, time pressure
- Aggregates weak signals
- Never decides or acts

Its only role is to form intuition.

---

### Focus Gate
The **conscious filter**.

- Receives signals from the Intuition Engine
- Applies decay, pressure, and pinning
- Surfaces *candidates* for review
- Never auto-switches focus

---

### Reference Store
Externalized, lossless memory.

- Holds large artifacts (diffs, logs, outputs)
- Prevents token overload
- Uses handles instead of inlining content
- Explicit rehydration only

---

### Expression Engine
The system’s **voice**.

- Converts Focus State into language
- Enforces token budgets
- Uses deterministic structure
- Applies explicit degradation rules when needed

---

## Canonical Cognitive Flow

```
Intuition Engine
      ↓
  Focus Gate
      ↓
 Focus Stack
      ↓
 Focus State
      ↓
Expression Engine
      ↓
  Model Invocation
```

---

## Why This Works Across Compaction

When a harness or model compacts context:

- Conversation can be lost
- Meaning is not

Because:
- Intent lives in Focus State
- Artifacts live in Reference Store
- Decisions are anchored
- Focus is re-asserted every turn

Compaction becomes harmless.

---

## Integration Model

Focusa runs as a **fast local proxy**.

- Wraps existing CLI or API harnesses
- No harness internals required
- No model modification
- No retraining

Focusa is invisible unless you inspect it.

---

## Interfaces

### CLI
Primary control surface:
- Manage Focus Stack
- Inspect Focus Gate candidates
- Interact with Reference Store
- Debug events and state

### Local API
- JSON over HTTP
- Used by CLI, GUI, adapters

### Menubar GUI
- Passive observability
- Calm, ambient state awareness
- No intrusive alerts

---

## Relationship to Beads

Focusa uses **Beads** as the authoritative system of record for tasks and long-term intent.

- Every Focus Frame maps to a Beads issue
- If work is not in Beads, it does not exist
- Focusa governs *focus*
- Beads governs *what work exists*

---

## Design Principles

- Focus over autonomy
- Structure over prose
- Explicit over inferred
- Advisory over controlling
- Human intent always wins
- Failure must be visible

---

## Status

🚧 **Architecture Locked — Pre-Implementation**

Documentation is sufficient to implement the MVP without guesswork.

---

## One-Sentence Summary

> **Focusa preserves continuity of mind across long AI sessions by separating focus, memory, and expression from fragile conversation history.**
