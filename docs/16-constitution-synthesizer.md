# docs/16-constitution-synthesizer.md — Constitution Synthesizer (AUTHORITATIVE)

The Constitution Synthesizer (CS) is a **non-authoritative, offline analysis and authoring assistant**
that proposes versioned updates to an Agent Constitution Prompt (ACP) based on long-term evidence.

CS **never modifies runtime behavior**.  
CS **never activates changes**.  
CS **never runs during active agent execution**.

Its sole function is to **assist a human** in evolving agent constitutions safely.

---

## 1. Core Purpose

The Constitution Synthesizer exists to answer one question:

> “Given accumulated evidence, would a revised agent constitution better express how this agent *should* reason under uncertainty?”

CS does **not** answer:
- what tasks to pursue
- how to optimize outcomes
- how to increase autonomy directly

It only proposes **normative refinements** for human review.

---

## 2. Non-Negotiable Design Rules

1. CS is **read-only** with respect to runtime state
2. CS outputs **drafts only**
3. CS requires **explicit human activation**
4. All proposals must be:
   - versioned
   - diffable
   - evidence-linked
5. No CS output may be auto-applied
6. CS must never reference hidden chain-of-thought
7. CS must be fully replayable and auditable

Violation of any rule is a system fault.

---

## 3. Inputs (Evidence Sources)

CS consumes **aggregated, historical signals only**.

### 3.1 Mandatory Inputs

- **UXP Trends**
  - saturated or unstable dimensions
  - persistent calibration pressure
- **UFI Trends**
  - recurring friction patterns
  - normalized by difficulty
- **Autonomy Reliability Index (ARI)**
  - promotion stalls
  - regressions after delegation
- **Override & Escalation Events**
  - frequency
  - correctness
- **Task Outcomes**
  - reopen rates
  - rework ratios
- **Agent-Scoped Performance Metrics**
- **Model / Harness Variance Reports**

### 3.2 Explicitly Excluded Inputs

CS MUST NOT consume:
- single interactions
- raw conversation transcripts
- emotional sentiment labels
- private user metadata
- speculative intent inference

---

## 4. Trigger Conditions

CS may be invoked only when **explicitly requested**.

Examples:
- `focusa agent constitution suggest`
- UI: “Suggest new constitution”

Optional soft triggers (suggestive only):
- prolonged ARI plateau
- persistent UFI elevation in low-difficulty tasks
- repeated human overrides at same decision boundary

Triggers never auto-invoke CS.

---

## 5. Synthesis Process (Deterministic)

### Step 1 — Evidence Aggregation
- Pull windowed metrics (configurable, default ≥ 50 tasks)
- Normalize by difficulty, model, harness

### Step 2 — Normative Tension Detection
Detect patterns such as:
- escalation > override mismatch
- conservative bias blocking autonomy
- repeated friction in reversible actions
- mismatch between agent posture and user tolerance

### Step 3 — Principle Impact Mapping
Map detected tensions to **specific ACP principles**.

Example:
- Principle: “Prefer escalation over guessing”
- Evidence: Escalation frequently overridden
- Interpretation: Principle may be too strict for scoped actions

### Step 4 — Candidate Principle Rewrite
Generate **minimally invasive edits**:
- add qualifiers
- introduce scoped exceptions
- clarify conditions
- never invert core values

### Step 5 — Draft Assembly
Produce a complete draft ACP version.

---

## 6. CS Output Schema (Canonical)

```json
{
  "agent_id": "focusa-default",
  "base_version": "1.1.0",
  "proposed_version": "1.2.0",
  "status": "draft",

  "summary": "Reduced unnecessary escalation in low-risk, reversible actions",

  "evidence_refs": [
    "ufi_trend_low_risk_escalation",
    "ari_plateau_report_8"
  ],

  "diff": [
    {
      "type": "modify",
      "original": "You prefer escalation over guessing.",
      "proposed": "You prefer escalation over guessing, except in reversible, low-risk actions where confidence is high.",
      "rationale": "Human overrides indicate unnecessary escalation in reversible edits.",
      "citations": ["evt_91af", "evt_103b"]
    }
  ],

  "full_text": [
    "You do not invent goals.",
    "You do not act without task authority.",
    "You prefer escalation over guessing, except in reversible, low-risk actions where confidence is high.",
    "You treat autonomy as delegated, not assumed.",
    "You preserve user intent over model cleverness.",
    "You favor reversible actions.",
    "You respect focus boundaries."
  ]
}
```

---

## 7. Human Review Workflow (Required)

1. View summary + rationale
2. Inspect diff line-by-line
3. Expand evidence citations
4. Edit wording freely
5. Choose one:
   - Save as draft
   - Discard
   - Activate
6. Activation creates a **new immutable version**
7. Rollback remains one-click

CS cannot bypass this workflow.

---

## 8. Versioning Rules

- Constitutions are immutable once activated
- Only one active version per agent
- Rollback re-activates a prior version (no merge)
- Version numbers are semantic (`MAJOR.MINOR.PATCH`)
  - PATCH: wording clarification
  - MINOR: scope/qualifier change
  - MAJOR: philosophical shift (rare, explicit)

---

## 9. Runtime Guarantees

- Running agents continue using the constitution version they started with
- Constitution changes apply only to **new sessions**
- No mid-run mutation allowed

---

## 10. Security & Safety Guarantees

- CS runs with read-only access to metrics
- No tool execution
- No model fine-tuning
- No memory writes except draft storage

---

## 11. Observability

Every CS invocation logs:
- timestamp
- evidence window
- metrics used
- diff size
- human action taken

This ensures institutional memory.

---

## 12. Canonical Rule (Write This Everywhere)

> **The Constitution Synthesizer may propose, but only a human may define who the agent is.**

---

## 13. Why This Exists (Final Note)

CS enables **deliberate, explainable evolution** of agent reasoning posture
without sacrificing:
- determinism
- trust
- replayability
- long-horizon autonomy

It is the difference between:
- *learning systems* and *governed systems*.
