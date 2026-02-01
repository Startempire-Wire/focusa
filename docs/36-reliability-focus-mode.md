# docs/36-reliability-focus-mode.md
## Reliability Focus Mode (RFM) — AUTHORITATIVE SPEC

Reliability Focus Mode (RFM) is a **selective, cognition-aligned escalation mechanism** in Focusa.
It increases correctness and safety **only when risk, autonomy, or task criticality justify the cost**.

RFM is **not default behavior**, **not brute-force voting**, and **not prompt hacking**.
It is a *system-level reliability governor*.

---

## 1. Purpose & Non-Goals

### Purpose
- Reduce catastrophic or high-regret errors
- Contain error propagation at the step level
- Improve trust for autonomous or semi-autonomous execution
- Enable explainable, auditable correctness checks

### Non-Goals
- Achieve zero-error execution universally
- Replace human judgment
- Make all tasks slow-but-correct
- Turn Focusa into a sampling engine

---

## 2. Conceptual Model

Reliability Focus Mode introduces **Microcells**:
> isolated, narrow-scope sub-agents invoked for *verification*, not creativity.

Microcells:
- have their own context
- do not see full session history
- do not modify Focus State
- return structured evidence

RFM integrates with:
- Focus Gate
- Focus Stack
- CLT
- Telemetry
- Autonomy ladder

---

## 3. Reliability Levels

RFM operates at discrete levels chosen **per Focus Frame**.

| Level | Name | Behavior |
|-----|------|---------|
| R0 | Normal | No reliability escalation |
| R1 | Validation | Spawn validator microcells |
| R2 | Regeneration | Validate → regenerate once on failure |
| R3 | Ensemble | Multiple generators + validators (rare) |

RFM level is **decided by Focus Gate**, not the agent.

---

## 4. Triggers for Reliability Mode

RFM may be activated when one or more signals are present:

### Structural Signals
- Focus Frame marked `risk: high`
- Write or destructive operations
- Security-sensitive tasks
- External system interaction

### Behavioral Signals
- Low gate acceptance rate
- High rework ratio
- Recent cache bust
- CLT branch abandonment spike

### Human Signals
- Rising UFI
- Explicit user override (“be extra careful”)

### Autonomy Signals
- Autonomy level ≥ threshold
- Calibration policy recommends reliability

---

## 5. Microcell Types (MVP)

### 5.1 Validator Microcells (Primary)

Validator microcells **never generate content**.
They only evaluate.

#### Types

1. **Schema Validator**
   - checks formatting, JSON schema, required fields

2. **Constraint Validator**
   - checks explicit constraints (files, scope, tools)

3. **Consistency Validator**
   - checks internal contradictions

4. **Reference-Grounding Validator**
   - checks claims against Reference Store / CLT

Each validator returns:

```json
{
  "result": "pass | fail",
  "reason": "string",
  "citations": [
    { "type": "ref | clt", "id": "uuid" }
  ]
}
```

---

### 5.2 Generator Microcells (Optional, non-MVP)

Used only in R3.

- Multiple minimal-context generators
- Strict output schema
- No access to global memory
- Outputs treated as candidates, not truth

---

## 6. RFM Execution Flow

1. Focus Gate selects RFM level
2. Primary agent produces candidate output
3. Validator microcells are invoked in parallel
4. Validation results are aggregated
5. Gate decision:
   - accept
   - reject + regenerate
   - escalate
6. Outcome recorded in CLT + telemetry

---

## 7. Failure Handling

- Validation failure does NOT mutate Focus State
- Failures create:
  - CLT child nodes
  - Telemetry events
- Regeneration is limited (max 1 in R2)
- R3 is never automatic without policy approval

---

## 8. Telemetry Integration

RFM emits:
- `rfm.invoked`
- `rfm.level`
- `validator.pass | validator.fail`
- `rfm.regeneration`
- `rfm.escalation`

Used by:
- Autonomy calibration
- UXP/UFI attribution
- Reliability scoring

---

## 9. Design Guarantees

- No validator can modify state
- No microcell sees full history
- All decisions are explainable
- Cost is bounded and measurable

---

## 10. Why This Generalizes Beyond Toy Problems

Unlike MDAP-style systems:
- structure is emergent, not predefined
- correctness is contextual, not binary
- reliability is selective, not universal
- humans remain in the loop

---

## 11. Summary

Reliability Focus Mode:
- preserves cognition-first design
- improves trust where it matters
- avoids brute-force economics
- integrates cleanly with Focusa’s architecture

---

---

# UPDATE — Artifact Integrity Scoring

# docs/36-reliability-focus-mode.md (UPDATED)

## Artifact Integrity Scoring (AIS)

### Definition

Artifact Integrity Score (AIS) ∈ [0.0, 1.0]

AIS measures the completeness and correctness of the agent’s internal artifact model
relative to the Reference Store and CLT.

---

### Artifact Categories Tracked

For each task / focus frame:

- files_read
- files_modified
- files_created
- symbols_touched (functions, classes)
- external_refs_used

These are tracked independently of summaries.

---

### Scoring Formula (Conceptual)

AIS =
  known_artifacts_referenced
  ──────────────────────────
  known_artifacts_expected

Where:
- expected artifacts come from Reference Store + CLT
- referenced artifacts come from proposals, prompts, and responses

---

### AIS Penalties

Penalties are applied for:

- Missing file paths
- Incorrect file names
- Lost modification intent
- Validator failures citing missing artifacts
- User re-supplying known files

Each penalty emits:
- artifact.integrity.violation event
- penalty_weight
- CLT citation

---

### AIS Thresholds

- AIS ≥ 0.90 → Safe
- 0.70 ≤ AIS < 0.90 → Degraded
- AIS < 0.70 → Reliability Focus Mode auto-activates

---

### Reliability Focus Mode Behavior (Expanded)

When AIS drops below threshold:

1. Pause autonomy escalation
2. Spin up validator sub-agents
3. Force artifact reconciliation step
4. Re-anchor Focus State with explicit artifact listing
5. Emit explanation to UI/TUI

AIS recovery is tracked over time.

---

### AIS and Autonomy

AIS contributes directly to:
- Autonomy Calibration score
- Proposal confidence weighting
- Trust envelope for long-running execution

An agent cannot earn autonomy while losing artifact integrity.
