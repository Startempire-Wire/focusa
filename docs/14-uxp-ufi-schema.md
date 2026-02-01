# docs/14-uxp-ufi-schema.md — User Experience Calibration (AUTHORITATIVE)

This document defines the canonical data model for:

- **UXP (User Experience Profile)** — slow-moving calibration
- **UFI (User Friction Index)** — fast-moving interaction measurements

All personalization and autonomy calibration MUST use these structures.

---

# 1. Core Design Rules (Non-Negotiable)

1. No opaque scores
2. No hidden inference
3. No emotion labels
4. All learned values must:
   - be weighted (0.0–1.0)
   - have confidence
   - have citations
   - be user-adjustable
5. Learning is slow, smoothed, and reversible
6. Language signals are secondary to behavior
7. Agent ≠ Model ≠ Harness (always separated)

---

# 2. Entity Separation

Calibration is scoped across **three axes**:

```
User
 ├─ Agent Persona
 │   └─ Model / Harness
```

Every UXP dimension and UFI entry MUST declare its scope.

---

# 3. User Experience Profile (UXP)

UXP represents **how this human prefers the system to behave**.

- Slow-moving
- Trend-based
- Adjustable
- Confidence-weighted

---

## 3.1 UXP Root Object

```json
{
  "user_id": "user_abc123",
  "version": 1,
  "last_updated": "2025-02-14T18:22:00Z",
  "dimensions": [ ... ]
}
```

---

## 3.2 UXP Dimension Object (Canonical)

```json
{
  "dimension_id": "verbosity_preference",

  "value": 0.32,
  "confidence": 0.81,

  "scope": {
    "user": true,
    "agent_id": "focusa-default",
    "model_id": "claude-3.5",
    "harness_id": "claude-code"
  },

  "learning": {
    "source": ["onboarding", "ufi_trend"],
    "alpha": 0.05,
    "window_size": 50,
    "last_adjustment": "2025-02-12T09:41:33Z"
  },

  "citations": [
    {
      "event_id": "evt_91af",
      "interaction_id": "int_3f92",
      "quote": "Just give me the diff, not the explanation",
      "timestamp": "2025-02-11T10:22:04Z"
    }
  ],

  "user_override": {
    "enabled": false,
    "override_value": null,
    "set_at": null
  }
}
```

---

## 3.3 UXP Dimension Semantics

| Field | Meaning |
|---|---|
| `value` | Current calibrated preference (0–1) |
| `confidence` | Evidence strength (not correctness) |
| `scope` | Where this calibration applies |
| `learning.alpha` | Update rate (small by design) |
| `citations` | Exact, inspectable evidence |
| `user_override` | Explicit human control |

---

## 3.4 Canonical UXP Dimensions (Initial Set)

- `autonomy_tolerance`
- `verbosity_preference`
- `interruption_sensitivity`
- `explanation_depth`
- `confirmation_preference`
- `risk_tolerance`
- `review_cadence`

All dimensions are optional but must follow the same schema.

---

# 4. User Friction Index (UFI)

UFI represents **interaction cost**, not emotion.

- Fast-moving
- Per-interaction
- Evidence-based
- Aggregated into trends

---

## 4.1 UFI Interaction Record

```json
{
  "ufi_id": "ufi_482fa",
  "interaction_id": "int_3f92",
  "timestamp": "2025-02-11T10:22:10Z",

  "context": {
    "task_id": "beads-124",
    "agent_id": "focusa-default",
    "model_id": "claude-3.5",
    "harness_id": "claude-code",
    "difficulty_estimate": 0.62
  },

  "signals": [
    {
      "signal_type": "immediate_correction",
      "weight": 0.7
    },
    {
      "signal_type": "rephrase",
      "weight": 0.3
    }
  ],

  "aggregate": 0.54,

  "citations": [
    {
      "event_id": "evt_83ab",
      "quote": "No, that’s not what I meant",
      "timestamp": "2025-02-11T10:21:58Z"
    }
  ]
}
```

---

## 4.2 Canonical UFI Signal Types

### High-Weight (Objective)
- `task_reopened`
- `manual_override`
- `immediate_correction`
- `undo_or_revert`
- `explicit_rejection`

### Medium-Weight
- `rephrase`
- `repeat_request`
- `scope_clarification`
- `forced_simplification`

### Low-Weight (Language-Only)
- `negation_language`
- `meta_language`
- `impatience_marker`

⚠️ Language-only signals may NEVER dominate an aggregate score.

---

## 4.3 UFI Aggregation Rules

- Signals are additive but capped
- Aggregates are clamped `0.0–1.0`
- No single interaction affects UXP
- Trends only, not spikes

---

# 5. UFI → UXP Learning Bridge

UXP updates occur only via **trend windows**:

```
UXP_new =
  clamp(
    UXP_old * (1 - α)
    + mean(UFI_window) * α,
    0.0,
    1.0
  )
```

Constraints:
- α ≤ 0.1
- window_size ≥ 30
- confidence increases with sample size
- user override freezes learning

---

# 6. Cascade Integration Points

| Component | Allowed Influence |
|---|---|
| Intuition Engine | Weak trend signals only |
| Focus Gate | Threshold modulation |
| Expression Engine | Primary consumer |
| Autonomy Scoring | Penalty / stability factor |
| Focus Stack | NO influence |

---

# 7. Storage Requirements

- Local SQLite DB
- Indexed by:
  - user
  - agent
  - model
  - harness
- Append-only for UFI
- Versioned for UXP

---

# 8. Transparency Guarantees

The system MUST be able to answer:

- “Why is this value what it is?”
- “What evidence supports it?”
- “How confident are you?”
- “Can I change it?”

Failure to answer any of these is a violation.

---

# 9. Final Canonical Rule

> **Focusa calibrates behavior through observable friction, not inferred emotion — and always shows its work.**
