# 83 — Pi × Focusa RPC Efficiency Spec (A/B/C)

**Date:** 2026-04-21  
**Status:** active (planning complete; implementation gated)  
**Priority:** critical  
**Epic:** `focusa-pyc1`  
**Tasks:** `focusa-pyc1.1` (A), `focusa-pyc1.2` (B), `focusa-pyc1.3` (C)

---

## 1) Why this spec exists

Pi remains perceptibly laggy under Focusa integration. Observed runtime behavior indicates the integration path can add wall-time overhead even when Focusa daemon is healthy. We need a resource-efficient architecture that preserves correctness and the intended authority model.

---

## 2) Non-negotiable authority model

This spec is constrained by:
- `docs/44-pi-focusa-integration-spec.md`
- `docs/52-pi-extension-contract.md`
- `docs/79-focusa-governed-continuous-work-loop.md`

Hard rules:
1. Focusa is the single cognitive authority.
2. Pi RPC driver is transport-only.
3. Pi extension is thin UX/observability glue.
4. Pi extension must not become a parallel persistent cognitive runtime.

---

## 3) Scope

This spec covers only A/B/C:

### A) Event-driven bridge default
- Make bridge sync default to event-driven.
- Disable periodic polling unless explicitly configured.
- Keep optional polling mode for fallback environments.

### B) Supervisor/RPC backpressure observability
- Add explicit daemon supervisor counters.
- Expose counters through `/v1/status` runtime perf schema.
- Preserve bounded driver lifecycle behavior.

### C) Benchmark proof
- Build reproducible benchmark flow.
- Produce before/after evidence with p50/p95 latency + CPU/RSS.

Out of scope:
- New cognition policy models.
- Major work-loop functional redesign.
- Provider/model routing redesign outside measured perf path.

---

## 4) Bead decomposition

## 4.1 Epic
- `focusa-pyc1` — Spec83: Pi×Focusa RPC efficiency redesign (A/B/C)

## 4.2 Task A
- `focusa-pyc1.1` — Event-driven bridge mode default
- Description: remove periodic polling as default sync strategy.
- Key deliverables:
  - config keys for sync mode + poll interval
  - event-driven default behavior
  - polling mode opt-in path

## 4.3 Task B
- `focusa-pyc1.2` — Supervisor/RPC counters + status exposure
- Description: instrument control loop backpressure and expose telemetry.
- Key deliverables:
  - counter struct in daemon app state
  - counter increments at supervisor control points
  - `/v1/status` runtime perf projection

## 4.4 Task C
- `focusa-pyc1.3` — Benchmark harness + evidence
- Description: prove performance deltas with reproducible bounded commands.
- Key deliverables:
  - benchmark script/command set
  - before/after captures
  - evidence markdown report

---

## 5) Detailed requirements

## 5.1 A — Event-driven default
1. Config surface MUST include:
   - `bridgeSyncMode`: `event-driven|polling`
   - `bridgePollMs`: integer, minimum 5000
2. Defaults MUST be:
   - `bridgeSyncMode=event-driven`
   - `bridgePollMs=15000`
3. Session startup MUST NOT start periodic polling timer unless mode=`polling`.
4. Existing on-demand sync points remain functional.

## 5.2 B — Counters + status projection
1. Supervisor counters MUST include at least:
   - ticks_total
   - driver_start_attempts
   - driver_stop_attempts
   - dispatch_attempts
   - dispatch_skipped_disallowed
   - dispatch_recovery_restarts
2. Counters MUST be lock-safe and low-overhead.
3. `/v1/status` MUST expose counters in stable JSON fields.
4. Behavior MUST keep disallowed-state guardrails (no uncontrolled transport churn).

## 5.3 C — Benchmark/evidence
1. Benchmark commands MUST be timeout-bounded.
2. Capture metrics:
   - Pi command wall time
   - Focusa API p50/p95 latency
   - Pi CPU%
   - Pi RSS
3. Use fixed scenario for before/after comparability.
4. Write evidence artifact under `docs/evidence/` with raw command outputs and summary.

---

## 6) Validation gates

Mandatory pass gates before closure:
1. `apps/pi-extension` TypeScript compile passes.
2. `focusa-api` cargo check/tests pass.
3. `/v1/status` includes runtime perf counters.
4. Benchmark evidence produced and linked in bead closure reasons.

---

## 7) Execution order (strict)

1. Finalize detailed spec + detailed beads. ✅
2. Implement A (`focusa-pyc1.1`).
3. Implement B (`focusa-pyc1.2`).
4. Implement C harness + evidence (`focusa-pyc1.3`).
5. Close A/B/C with evidence citations.
6. Close epic.

---

## 8) Risk register

1. **Risk:** overly sparse sync causes stale UI labels.  
   **Mitigation:** polling fallback mode remains available.

2. **Risk:** telemetry overhead affects hot path.  
   **Mitigation:** atomic counters only; no heavy per-tick allocations.

3. **Risk:** benchmark polluted by host noise.  
   **Mitigation:** fixed command set, timestamped runs, repeated samples.

---

## 9) Closure criteria

Spec83 is complete when:
1. A/B/C tasks are implemented and closed with citations.
2. Evidence report demonstrates measured delta or clearly documents non-improvement.
3. Runtime behavior remains aligned with thin-extension + transport-only + single-authority principles.
