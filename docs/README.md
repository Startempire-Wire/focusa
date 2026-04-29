# Focusa Docs

**Focusa** is a local-first cognitive continuity and governance runtime for AI agents.

This docs index describes the current development snapshot. Focusa is implemented across Rust core/API/CLI plus the Pi extension, and it remains under active development.

---

## What Focusa Solves

- Context loss under compaction
- Silent behavioral drift
- Unverifiable autonomy
- Unexplainable learning
- Long-running task incoherence

---

## Current Runtime Concepts

- **Focus State** – bounded current cognitive state: intent, focus, decisions, constraints, failures, next steps, open questions, recent results, notes, artifacts.
- **Workpoint** – typed continuation contract for compaction/model-switch/fork/retry recovery.
- **Evidence refs** – stable proof handles linked to Workpoints instead of raw transcript blobs.
- **Focus Stack** – structured attention over tasks and frames.
- **Context Lineage Tree (CLT)** – branch-aware interaction lineage.
- **Ontology** – objects, links, working sets, action intent, and verification relations.
- **Metacognition** – capture/retrieve/reflect/adjust/evaluate loop for reusable learning.
- **Work-loop** – continuous execution state with writer ownership and preflight controls.
- **Tool Result Envelope** – common status/canonical/degraded/retry/evidence/next-tool metadata for `focusa_*` tools.
- **UXP/UFI, autonomy, constitutions** – governance design surfaces with partial runtime support and ongoing development.

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


## Current-build references

- [../CHANGELOG.md](../CHANGELOG.md) — current snapshot change history.
- [current/CURRENT_RUNTIME_STATUS.md](current/CURRENT_RUNTIME_STATUS.md) — current implemented runtime status and limits.
- [current/API_REFERENCE_CURRENT.md](current/API_REFERENCE_CURRENT.md) — current API route inventory.
- [current/CLI_REFERENCE_CURRENT.md](current/CLI_REFERENCE_CURRENT.md) — current CLI command inventory.
- [current/PI_EXTENSION_AND_SKILLS_GUIDE.md](current/PI_EXTENSION_AND_SKILLS_GUIDE.md) — Pi extension and skills guide.
- [current/WORKPOINT_LIFECYCLE_GUIDE.md](current/WORKPOINT_LIFECYCLE_GUIDE.md) — Workpoint lifecycle guide.
- [current/TOOL_RESULT_ENVELOPE_V1.md](current/TOOL_RESULT_ENVELOPE_V1.md) — structured tool result contract.
- [current/TROUBLESHOOTING_CURRENT.md](current/TROUBLESHOOTING_CURRENT.md) — current troubleshooting runbook.
- [current/VALIDATION_AND_RELEASE_PROOF.md](current/VALIDATION_AND_RELEASE_PROOF.md) — validation and real runtime proof.
- [current/PRODUCTION_RELEASE_COMMANDS.md](current/PRODUCTION_RELEASE_COMMANDS.md) — release, restart, GitHub proof, and cleanup commands.
- [90-ontology-backed-tool-contracts-parity-spec.md](90-ontology-backed-tool-contracts-parity-spec.md) — Spec90 tool contract/parity hardening plan.
- [current/FOCUSA_TOOL_CONTRACT_REGISTRY.md](current/FOCUSA_TOOL_CONTRACT_REGISTRY.md) — current tool contract registry table.
- [91-live-tool-contract-proof-harness-spec.md](91-live-tool-contract-proof-harness-spec.md) — Spec91 live runtime proof harness.
- [current/LIVE_TOOL_CONTRACT_PROOF.md](current/LIVE_TOOL_CONTRACT_PROOF.md) — live proof command and expected result.

## Focused tool and skill docs

- [focusa-tools/README.md](focusa-tools/README.md) — index for all current `focusa_*` tool family docs.
- [focusa-tools/workpoint.md](focusa-tools/workpoint.md) — Workpoint continuity tools.
- [focusa-tools/focus-state.md](focusa-tools/focus-state.md) — Focus State and scratchpad tools.
- [focusa-tools/work-loop.md](focusa-tools/work-loop.md) — continuous work-loop tools.
- [focusa-tools/metacognition.md](focusa-tools/metacognition.md) — metacognition tools.
- [focusa-tools/tree-lineage.md](focusa-tools/tree-lineage.md) — tree, lineage, snapshot tools.
- [focusa-tools/diagnostics-hygiene.md](focusa-tools/diagnostics-hygiene.md) — troubleshooting and state hygiene tools.

Companion Pi skills mirror these docs: `focusa-workpoint`, `focusa-metacognition`, `focusa-work-loop`, `focusa-cli-api`, `focusa-troubleshooting`, and `focusa-docs-maintenance`.

## Status

Focusa is under active development.
The current `v0.9.0-dev` snapshot focuses on correctness, transparency, continuity, and live proof over marketing claims.

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

### GUI / TUI
- TUI crate exists as a runtime surface.
- Menubar/Tauri material in older docs is design direction unless a current release note or evidence file says it is shipped in the active snapshot.

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

🚧 **Current snapshot: v0.9.0-dev**

The Rust daemon/API/CLI, Pi extension, Workpoint continuity, tool result envelopes, evidence linking, metacognition surfaces, state hygiene tools, and live release proof are implemented in the current snapshot. Focusa remains under active development; older design docs may describe planned or partial surfaces.

---

## One-Sentence Summary

> **Focusa preserves continuity of mind across long AI sessions by separating focus, memory, and expression from fragile conversation history.**
