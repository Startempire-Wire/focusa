# 81 — Focusa LLM Tool Suite + CLI Development Reset Spec

**Date:** 2026-04-21  
**Status:** active  
**Priority:** critical

## 1) Why this spec exists

We temporarily drifted into system-memory optimization discussion.
This spec resets execution back to the original mission:

1. deliver a **high-quality first-class LLM tool suite** in Focusa (Pi extension surface), and
2. deliver **deeply intertwined Focusa CLI development** (not thin wrappers only).

Memory/perf hardening remains important, but is **secondary lane** and not the primary delivery focus for this cycle.

---

## 2) Primary mission (authoritative)

Ship a crafted, operator-grade tool system where:

- LLM tools are discoverable, reliable, typed, and useful for real workflows.
- CLI is a first-class operator interface for the same workflows.
- Metacognition supports compound learning loops, not just point actions.

---

## 3) Scope

## 3.1 In-scope now (implemented-now target)

### A. First-class LLM tools (Pi extension)

Required tools:

- `focusa_tree_head`
- `focusa_tree_path`
- `focusa_tree_snapshot_state`
- `focusa_tree_restore_state`
- `focusa_tree_diff_context`
- `focusa_metacog_capture`
- `focusa_metacog_retrieve`
- `focusa_metacog_reflect`
- `focusa_metacog_plan_adjust`
- `focusa_metacog_evaluate_outcome`

Quality requirements:

- strict parameter schemas (bounded + validated)
- canonical typed result envelope
- explicit error-code mapping
- writer-safe routing for write operations
- transient retry behavior for recoverable failures
- runtime contract tests (not grep-only)

### B. CLI parity + deep coupling

Required CLI surfaces:

- lineage commands (`focusa lineage ...`)
- snapshot commands (`focusa state snapshot ...`)
- metacognition commands (`focusa metacognition ...`)

CLI quality requirements:

- endpoint-backed behavior with stable JSON output
- command semantics aligned with tool semantics
- typed failure visibility
- operator-friendly non-json summaries

### C. Compound-learning workflow uplift (CLI-focused)

Add high-order CLI workflows beyond primitive actions:

- `focusa metacognition loop run` (capture→retrieve→reflect→adjust→evaluate)
- `focusa metacognition promote` (policy-gated promotion decision)
- `focusa lineage compare` (branch/snapshot comparison for learning deltas)
- `focusa metacognition doctor` (signal quality + confidence diagnostics)

These are required to satisfy the “deeply intertwined with CLI” ask.

## 3.2 Out-of-scope for this reset (planned-extension)

- broad daemon RAM redesign as primary lane
- unrelated service/process tuning outside Focusa tool+CLI delivery

(Perf/memory work may continue in parallel but cannot displace this mission.)

---

## 4) Delivery contracts

## 4.1 Code-reality contract

No claim without citation:

- `implemented-now` claims require `code: file:line`
- runtime claim requires executable test evidence

## 4.2 Test contract

Minimum proof pack must include:

1. extension runtime contract test (all 10 tools)
2. live flow test for snapshot + metacognition chains
3. CLI runtime contract test for new high-order commands
4. typed error-path tests for each new high-order command

## 4.3 Operator quality contract

Reject “done” if any of these fail:

- tools exist but are thin pass-through with weak semantics
- CLI lacks high-order metacognition workflows
- only shell grep checks are used as proof
- no typed failure contract for user-facing flows

---

## 5) Execution plan

### Phase 1 — Harden existing tool quality (now)
- schema strictness
- envelope consistency
- retry + failure mapping
- runtime request-shape assertions

### Phase 2 — Build CLI high-order metacognition suite
- loop run / promote / compare / doctor
- JSON + human modes
- endpoint + policy wiring

### Phase 3 — Evidence + signoff packet
- tool-to-cli traceability matrix
- command examples + failure examples
- line-cited completion report

---

## 6) Acceptance criteria

Done means all true:

1. all required tools are registered + discoverable as first-class surfaces
2. all required CLI domains are implemented and endpoint-backed
3. high-order CLI metacognition workflows are shipped
4. runtime tests pass (tool contract + live flow + CLI workflows)
5. evidence doc maps every claim to `file:line` + test output

---

## 7) Immediate next task

Implement CLI high-order metacognition commands (`loop run`, `promote`, `lineage compare`, `doctor`) with full runtime contract tests, then produce completion matrix.
