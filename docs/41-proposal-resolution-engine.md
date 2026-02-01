# docs/41-proposal-resolution-engine.md
## Proposal Resolution Engine (PRE) — AUTHORITATIVE SPEC

This document specifies the Proposal Resolution Engine (PRE), which enables
**timestamped, async concurrency** across multiple Instances and Sessions without locks.

PRE resolves competing **decisional proposals** into a single canonical outcome,
while preserving all alternatives for audit, explanation, and forking.

---

## 1. Purpose

PRE exists to:
- allow concurrent work across many instances
- prevent silent overwrites
- preserve explainability and provenance
- maintain a singular canonical Focus State / Thesis version per thread
- support background progress without blocking

---

## 2. Key Concepts

### 2.1 Observations vs Decisions

#### Observations (always concurrent, append-only)
- CLT nodes
- reference additions
- validator results
- telemetry events

Observations never conflict.

#### Decisions (subject to resolution)
- focus change
- focus stack mutation
- thesis update
- autonomy adjustment
- constitution update

Decisions are expressed as **proposals**.

---

## 3. Proposal Schema (Canonical)

```json
{
  "proposal_id": "uuid",
  "thread_id": "uuid",
  "instance_id": "uuid",
  "session_id": "uuid",
  "attachment_id": "uuid|null",

  "timestamp": "timestamp",
  "type": "focus.change | thesis.update | autonomy.change | constitution.propose",
  "payload": { "key": "value" },

  "confidence": 0.0,
  "evidence": [
    { "type": "clt|ref|telemetry", "id": "uuid" }
  ],

  "status": "pending | accepted | rejected | superseded",
  "resolution": {
    "resolved_at": "timestamp|null",
    "winner": "proposal_id|null",
    "reason": "string|null",
    "citations": [{ "type": "clt|telemetry", "id": "uuid" }]
  }
}
```

---

## 4. Resolution Windows (No Locks)

PRE groups proposals into **Resolution Windows** by thread and proposal target.

### 4.1 Window Definition

A window is:
- per thread
- per target class (e.g., Focus State, Thesis)
- time bounded (default 500ms–2000ms configurable)

Within a window:
- multiple proposals may arrive
- all remain pending until resolution

### 4.2 Window Keying

Key tuple:
- `thread_id`
- `target` (one of: focus, thesis, autonomy, constitution)
- `window_start`

---

## 5. Resolution Algorithm (High-Level)

At window close:
1. Gather all pending proposals in the window
2. Compute a score for each proposal
3. Select:
   - a single winner (default)
   - or no-winner (request clarification)
4. Emit resolution events
5. Apply winner to canonical state via reducer
6. Record outcome in CLT + telemetry

---

## 6. Scoring Inputs (Deterministic)

PRE scoring MUST be deterministic given inputs.

Recommended inputs:

### 6.1 Evidence Strength
- validator pass rate
- grounding evidence present
- references cited

### 6.2 Alignment
- Thread Thesis alignment
- active Focus Frame consistency

### 6.3 Risk & Reliability Policy
- if risk high, require validator support
- if RFM active, weight validator outcomes strongly

### 6.4 Source Trust
- instance role (active > assistant > background > observer)
- autonomy level ceiling for that agent/harness

### 6.5 Recency
- slight bias to later proposals within window (configurable)

---

## 7. Outcomes

### 7.1 Accept One
- winner applied
- others rejected as conflicting

### 7.2 Reject All (Clarification Required)
- when proposals are too divergent
- or evidence insufficient
- or policy requires human confirmation

### 7.3 Supersede
- new window proposal supersedes earlier pending proposals

---

## 8. Canonical State Invariants

Even with concurrency:
- Focus State is singular per thread
- Thesis version is linear per thread
- Autonomy level is singular per thread

History is never erased.

---

## 9. Integration Points

### 9.1 Focus Gate
- PRE may call Gate for explanation generation and policy enforcement

### 9.2 Focus State Reducer
- winner proposal is submitted as a reducer action
- reducer updates canonical state and emits CLT node

### 9.3 Reliability Focus Mode
- RFM can raise required evidence thresholds for acceptance

### 9.4 Telemetry
PRE emits:
- `proposal.submitted`
- `proposal.window.opened`
- `proposal.window.closed`
- `proposal.resolved`
- `proposal.rejected`
- `proposal.clarification_required`

---

## 10. UI / TUI Expectations

The UI must surface:
- pending proposals per thread
- timeline (by instance/session)
- resolution countdown (optional)
- resolution reasons + citations
- ability to fork from rejected proposals

---

## 11. Failure Modes & Safeguards

- Window sizes bounded (avoid latency)
- Hard ceiling on pending proposals (backpressure)
- Deterministic scoring prevents oscillation
- Full audit trail prevents hidden drift

---

## 12. Summary

PRE replaces locking with **async, timestamped governance**.
It is the concurrency backbone for:
- multiplexing engineers
- background intuition work
- safe autonomy growth
