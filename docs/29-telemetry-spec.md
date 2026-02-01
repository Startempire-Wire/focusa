# docs/29-telemetry-spec.md — Cognitive Telemetry Layer (CTL) Specification (AUTHORITATIVE)

The Cognitive Telemetry Layer (CTL) is a **passive, append-only observability subsystem**
responsible for capturing, aggregating, and exposing **all measurable signals**
produced by Focusa.

CTL does not influence cognition.
CTL does not make decisions.
CTL does not optimize for cost.

CTL exists to **make cognition measurable, researchable, and improvable**.

---

## 0. Canonical Principle

> **Cognition must be observable before it can be improved.**

---

## 1. Scope of CTL

CTL observes:
- model usage
- token economics
- cognitive transitions
- tool usage
- focus dynamics
- gate decisions
- intuition signals
- cache behavior
- human interaction signals
- autonomy evolution

CTL explicitly does **not**:
- modify prompts
- influence gates
- enforce policy
- control agents

---

## 2. Telemetry Design Constraints

1. **Low overhead**
   - async write path
   - batched persistence
   - sampling-capable

2. **Local-first**
   - SQLite / DuckDB default
   - no external dependency

3. **Append-only**
   - no in-place mutation
   - immutable events

4. **Schema-versioned**
   - forward compatible

5. **Queryable**
   - Capabilities API
   - CLI
   - TUI

6. **Exportable**
   - SFT
   - RLHF
   - research datasets

---

## 3. Telemetry Event Classes

### 3.1 Model & Token Telemetry
Tracks:
- token usage
- cache effects
- latency
- cost proxies

### 3.2 Cognitive Process Telemetry
Tracks:
- focus transitions
- CLT branching
- summarization
- abandonment
- rehydration

### 3.3 Tool & Interaction Telemetry
Tracks:
- tool invocations
- retries
- failures
- side effects

### 3.4 Human Experience Telemetry
Tracks:
- UXP
- UFI
- overrides
- corrections

### 3.5 Autonomy Telemetry
Tracks:
- autonomy level
- earned trust
- failures
- reversions

---

## 4. Telemetry Invariants

- Every event MUST be timestamped
- Every event MUST be attributable
- Every metric MUST be derivable from events
- No opaque aggregate-only metrics
- All scores must be explainable

---

## 5. CTL Integration Points

| Subsystem        | Telemetry Hook |
|------------------|----------------|
| Focus State      | transition events |
| CLT              | node creation / summary |
| Gate             | candidate scoring |
| Cache            | hit / miss / bust |
| Intuition Engine | signal generation |
| Constitution     | version changes |
| Agents           | command requests |
| CLI / UI         | interaction events |

---

## 6. Canonical Rule

> **If it cannot be measured, it cannot be trusted.**

---

# UPDATE

# docs/29-telemetry-spec.md (UPDATED)

## Telemetry Goals

Telemetry in Focusa exists to:
1. Measure real cognitive efficiency (not just API cost)
2. Detect degradation caused by compression or context loss
3. Support autonomy calibration and reliability scoring
4. Produce high-quality datasets for downstream training

---

## New Core Metrics (Task-Centric)

### Task Lifecycle Events

These events define the boundaries needed to compute tokens-per-task.

- task.started
- task.completed
- task.abandoned
- task.restarted
- task.refetch_required

A "task" is defined as a Focus Stack frame with status = completed | abandoned.

---

### Tokens Per Task

Derived metric:

tokens_per_task =
  Σ(tokens.input + tokens.output)
  ───────────────────────────────
  count(task.completed)

Tracked per:
- thread
- focus frame
- instance
- model
- harness

This metric is **canonical** for optimization decisions.

---

### Context Recovery Cost

context_recovery_cost =
  tokens_used_after_refetch
  ─────────────────────────
  tokens_used_before_refetch

Triggered by:
- reference reloading
- file re-reading
- clarification prompts
- hallucination recovery

High recovery cost indicates:
- over-aggressive compression
- poor artifact preservation
- misaligned Focus State

---

## Compression Regret Signal

compression_regret_event emitted when:
- refetch_required occurs
- validator failure due to missing artifact
- user explicitly re-provides known info

Stored as:
- regret_score (0–1)
- associated CLT nodes
- triggering compression cycle id

---

## Telemetry Storage Principles

- Append-only
- Never summarized
- Never compacted
- Always queryable

Telemetry is **ground truth**, not cognition.
