# docs/38-thread-thesis-spec.md
## Thread Thesis (Conversation Thesis) — AUTHORITATIVE SPEC

This document defines the **Thread Thesis**, a first-class cognitive object in Focusa.
The Thread Thesis represents the **current, distilled meaning of an ongoing interaction** —
what the system believes the conversation *is really about* at this moment in time.

It is **not a summary**, **not a transcript**, and **not a prompt**.
It is a **living semantic anchor** that preserves cohesion over long horizons.

---

## 1. Core Definition

> **Thread Thesis**  
> A structured, continuously refined representation of:
> - user intent
> - goals
> - constraints
> - open questions
> - confidence level  
> that governs how Focusa interprets and prioritizes all subsequent input.

The Thread Thesis is the **top-level cognitive anchor** for a session.

---

## 2. Why a Thread Thesis Exists

Long-running interactions fail when:
- intent drifts
- goals fragment
- constraints are forgotten
- relevance erodes

LLMs cannot reliably hold this internally over hundreds or thousands of turns.

**Thread Thesis externalizes this responsibility**.

---

## 3. Design Principles

1. **Meaning over words**  
   Thesis captures *semantic intent*, not phrasing.

2. **Stable but revisable**  
   It changes deliberately, not continuously.

3. **Structured, not free-form**  
   Machine-evaluable, not prose.

4. **Explainable**  
   Every update has provenance.

5. **Non-authoritative**  
   It informs decisions but does not enact them.

---

## 4. Thread Thesis Schema (Canonical)

```json
{
  "thesis_id": "uuid",
  "version": "int",
  "created_at": "timestamp",
  "updated_at": "timestamp",

  "primary_intent": "string",
  "secondary_goals": ["string"],

  "explicit_constraints": ["string"],
  "implicit_constraints": ["string"],

  "open_questions": ["string"],

  "assumptions": ["string"],

  "confidence": {
    "score": 0.0,
    "rationale": "string"
  },

  "scope": {
    "domain": "string",
    "time_horizon": "short | medium | long",
    "risk_level": "low | medium | high"
  },

  "sources": [
    { "type": "clt | user | system", "id": "uuid" }
  ]
}
```

---

## 5. Lifecycle

### 5.1 Creation

The Thread Thesis is created:
- at session start
- after onboarding
- after explicit goal-setting

Initial confidence is low.

---

### 5.2 Update Triggers

The Thesis may be revised when:

- User explicitly redefines goals
- Focus Stack root changes
- Repeated clarifications occur
- UFI spikes (indicating misalignment)
- Calibration recommends re-centering
- Long sessions exceed thresholds
- Autonomy level changes

Updates are **event-driven**, not per-turn.

---

### 5.3 Update Process

1. Reducer proposes a thesis update
2. Focus Gate evaluates:
   - alignment
   - evidence
   - stability impact
3. If accepted:
   - version increments
   - old version archived
4. Change recorded in CLT

---

## 6. What the Thread Thesis Is *Not*

- ❌ Not a chat summary
- ❌ Not injected verbatim into prompts
- ❌ Not an agent persona
- ❌ Not immutable
- ❌ Not hidden from the user

---

## 7. Interaction with Other Systems

### Focus State
- Focus State must be consistent with the Thesis
- Conflicts trigger clarification or re-centering

### Focus Gate
- Uses Thesis to:
  - score relevance
  - detect drift
  - justify rejections

### CLT
- Thesis updates become lineage nodes

### Reliability Focus Mode
- High risk thesis → higher reliability defaults

### Autonomy Calibration
- Confidence impacts autonomy ceilings

---

## 8. Prompt Assembly Rules

Thread Thesis is **never injected raw**.

Instead:
- distilled signals (intent, constraints) may influence:
  - system instructions
  - tool selection
  - validator constraints

This prevents prompt bloat and cache pollution.

---

## 9. Telemetry & Audit

Each thesis update emits:

- `thesis.updated`
- `thesis.version`
- `thesis.confidence_delta`
- citations to supporting CLT nodes

These are visible in:
- UI
- TUI
- Capabilities API

---

## 10. UI / TUI Requirements

Thread Thesis must be:

- visible
- inspectable
- versioned
- diffable
- revertible

Users must be able to:
- confirm
- edit
- reject
- roll back

---

## 11. Failure Modes & Safeguards

### Drift Prevention
- Minimum confidence delta required for change
- Cooldown between updates

### Overfitting Prevention
- Do not absorb single anomalous turns
- Require corroboration over time

---

## 12. Example (Conceptual)

```text
Primary Intent:
Design and formalize Focusa architecture for long-horizon agent cognition.

Secondary Goals:
- Ensure autonomy is earned safely
- Enable explainability
- Maintain model-agnosticism

Constraints:
- Local-first
- Model-agnostic
- Inspectable cognition

Open Questions:
- Optimal validator coverage?
- Best autonomy thresholds?

Confidence:
0.82 — stable intent over 120+ turns
```

---

## 13. Strategic Insight

The Thread Thesis is the **answer to long-context collapse**.

It allows Focusa to:
- reason coherently without reading everything
- remain aligned without repetition
- scale conversations indefinitely

---

## 14. Summary

The Thread Thesis:
- anchors meaning
- stabilizes cognition
- enables explainability
- bridges human intent and machine reasoning

It is the **semantic backbone** of Focusa’s long-horizon intelligence.

---
