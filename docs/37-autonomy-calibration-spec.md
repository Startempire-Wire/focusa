# docs/37-autonomy-calibration-spec.md
## Autonomy Calibration — AUTHORITATIVE SPEC

Autonomy Calibration is the mechanism by which Focusa **measures, earns, and governs agent autonomy** over time.

It replaces intuition and gut-feel with **evidence-based progression**.

---

## 1. Purpose

Autonomy Calibration exists to:

- quantify agent reliability
- measure improvement over time
- determine safe autonomy ceilings
- tune Focus Gate and RFM defaults
- provide explainable trust metrics

Autonomy is **never granted implicitly**.

---

## 2. Core Principle

> **Autonomy is earned through observed performance, not assumed capability.**

---

## 3. Autonomy Dimensions

Autonomy is multi-dimensional, not a single score.

### Core Dimensions

| Dimension | Description |
|--------|------------|
| Correctness | Constraint compliance, validation pass rate |
| Stability | Low rework, low abandonment |
| Efficiency | Tokens, time, tool economy |
| Trust | UXP/UFI-adjusted satisfaction |
| Grounding | Reference correctness |
| Recovery | Error correction behavior |

Each dimension is tracked independently.

---

## 4. Calibration Modes

### 4.1 On-Demand Calibration
- Explicitly triggered by user
- Short, bounded task suite

### 4.2 Continuous Background Calibration
- Passive observation
- Rolling metrics
- No disruption to workflow

---

## 5. Calibration Suite

A calibration suite is a set of **controlled tasks** with known evaluation criteria.

Each task defines:
- allowed tools
- risk level
- expected invariants
- success checks
- max budget (tokens/time)

Suites are:
- model-specific
- harness-specific
- domain-specific

---

## 6. Calibration Execution Flow

1. Select calibration suite
2. Execute tasks under observation
3. Collect telemetry + validator outcomes
4. Score per dimension
5. Aggregate results
6. Generate recommendations

---

## 7. Scoring Model (Conceptual)

Scores are normalized [0.0 – 1.0].

Example composite:

```text
Autonomy Score =
  0.30 * Correctness
+ 0.20 * Stability
+ 0.15 * Efficiency
+ 0.20 * Trust
+ 0.15 * Grounding
```

Weights are configurable.

---

## 8. Outputs of Calibration

Calibration produces **recommendations**, never automatic changes.

### Possible Outputs
- Suggested autonomy ceiling (level N)
- RFM default per task category
- Gate weight adjustments
- Constitution draft proposals
- Warning flags (“autonomy regression”)

---

## 9. Autonomy Levels (Example)

| Level | Capabilities |
|-----|-------------|
| 0 | Advisory only |
| 1 | Assisted execution |
| 2 | Conditional autonomy |
| 3 | Limited unattended runs |
| 4 | Extended autonomous operation |
| 5 | Long-horizon autonomy (future) |

Levels are **per agent + model + harness**.

---

## 10. Integration with Other Systems

Calibration feeds into:
- Focus Gate policy
- Reliability Focus Mode triggers
- Constitution evolution
- Capability permissions
- UI trust indicators

---

## 11. Telemetry Schema (Key Fields)

- calibration_run_id
- task_id
- agent_id
- model_id
- harness_id
- dimension_scores
- validator_stats
- rfm_usage
- uxp / ufi deltas

---

## 12. UI / TUI Expectations

Calibration UI must show:
- scores with explanations
- citations to CLT / telemetry
- deltas over time
- recommended vs active autonomy
- rollback options

---

## 13. Safety Guarantees

- No self-escalation
- No silent changes
- Human approval required
- Full audit trail

---

## 14. Why This Matters

Without calibration:
- autonomy is guesswork
- failures are surprises
- trust erodes

With calibration:
- autonomy is earned
- failures are bounded
- progress is measurable

---

## 15. Summary

Autonomy Calibration:
- turns reliability into data
- aligns autonomy with evidence
- enables safe long-running agents
- completes Focusa’s cognitive loop

---
