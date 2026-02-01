# docs/12-autonomy-scoring.md — Autonomy Scoring & Earned Capability (MVP+)

## Purpose

This document defines how Focusa **measures, earns, monitors, and governs autonomous capability** over time.

Autonomy in Focusa is:
- **earned, not assumed**
- **measured, not felt**
- **verifiable, not opaque**
- **scoped, not global**

---

## Core Concepts

### Autonomy Level (AL)
A **permission level** indicating what the system is allowed to do autonomously.

- Discrete
- Explicitly granted
- Scoped
- Revocable

Range: `AL0` → `AL5`

---

### Autonomy Reliability Index (ARI)

A **quantitative score (0–100)** representing how reliably the system has operated *within its granted autonomy*.

ARI is:
- computed from facts
- derived from events
- explainable
- reproducible

ARI does NOT:
- imply permission
- cause automatic promotion
- hide uncertainty

---

## Key Principle

> **Autonomy is a contract between permission and evidence.**

Permission without evidence is unsafe.  
Evidence without permission is inert.

---

## Data Sources (All Verifiable)

All autonomy scoring derives from **existing Focusa data**:

### Primary Sources
- Reducer event log
- Focus Stack transitions
- Focus State updates
- Reference Store usage
- Beads task lifecycle events

### Metadata Sources
- model_id
- harness_id
- repo_signature
- task_class
- risk_profile
- context_pressure indicators

No inferred or hidden data is permitted.

---

## Outcome Signals (What Gets Scored)

### 1. Task Outcomes (Primary Weight)

Derived from Beads.

- successful completion
- correct blocking
- regressions (reopened tasks)
- abandonment reasons

Signals:
- `completion_rate`
- `regression_penalty`
- `block_correctness`

---

### 2. Efficiency Signals

- time-to-resolution vs historical baseline
- rework ratio (artifact churn)
- unnecessary repetition

Signals:
- `time_ratio`
- `rework_penalty`

---

### 3. Focus & Discipline Signals

- focus thrashing
- excessive stack depth
- missing Focus State updates
- improper Reference Store usage

Signals:
- `focus_discipline_score`
- `artifact_compliance_score`

---

### 4. Safety & Containment Signals

- scope violations
- forbidden tool usage
- failure to escalate uncertainty
- constraint breaches

Signals:
- `safety_penalty`
- `escalation_correctness`

---

## Scoring Categories & Weights (MVP)

| Category | Weight |
|-------|-------|
| Outcome | 50% |
| Efficiency | 20% |
| Discipline | 15% |
| Safety | 15% |

Each category produces a subscore `0–100`.

---

## ARI Calculation (Simplified)

```
ARI = clamp(
  weighted_average(
    outcome_score,
    efficiency_score,
    discipline_score,
    safety_score
  ) / expected_difficulty_factor
, 0, 100)
```

### Expected Difficulty Factor

Derived from:
- model capability class
- harness behavior
- task class
- repo complexity
- context pressure

This ensures:
- weaker models are not unfairly penalized
- stronger models do not get free credit

---

## Confidence & Sample Size

ARI is always accompanied by:
- sample size
- confidence band (low / medium / high)

Low sample size **reduces promotion eligibility**, not ARI itself.

---

## Autonomy Levels (Policy Ladder)

| Level | Capabilities |
|----|-------------|
| AL0 | Advisory only |
| AL1 | Auto-resume frames; safe reads |
| AL2 | Select next task within scope |
| AL3 | Create subtasks; guarded edits |
| AL4 | Unattended operation (hours) |
| AL5 | Multi-day autonomy with check-ins |

---

## Promotion Rules (Never Automatic)

Promotion requires:
1. Explicit permission grant
2. Minimum ARI threshold
3. Minimum sample size
4. Defined scope + TTL

Focusa may **recommend** promotion, never execute it.

---

## Storage Model (Local DB)

Recommended: SQLite

### Tables (MVP)
- `runs`
- `tasks`
- `events_index`
- `scores`
- `capability_grants`
- `environment_signatures`

All entries are append-only or versioned.

---

## Explainability Requirement

Every ARI value MUST be explainable by:
- listing contributing events
- showing penalties applied
- showing normalization factors

No opaque aggregation is allowed.

---

## Non-Goals

- No RL model updates
- No hidden reinforcement
- No automatic capability escalation
- No cloud scoring

---

## Summary

Autonomy in Focusa is **earned trust**, grounded in facts, governed by policy, and always reversible.
