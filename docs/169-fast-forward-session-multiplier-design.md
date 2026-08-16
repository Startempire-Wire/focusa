# Fast-forward session multiplier — design (GUI vision)

Status: design + core slice. ORIGIN: operator-conceived on the fly
(2026-08-16); formalized as issue #312 — no pre-existing spec. The
future GUI (2x/4x/6x/8x controls) is a projection of this typed
fan-out primitive.

## Vision

A workbench control that multiplies parallel workloop-bound silent
sessions and divides tasks across them — the operator "fast-forwards"
session completion by adding lanes, not by skipping work.

## Model

`compile_fanout(FanoutInput { work_items, multiplier, policy budgets })
→ FanoutPlan` — deterministic:

- session_count = min(multiplier, work_items);
- round-robin task division (stable for the same input);
- per-session budgets = policy budgets (parallelism never stretches
  policy bounds);
- join spec: wait-for-all over the existing silent-session wait route.

The GUI renders the FanoutPlan lanes; each lane IS one silent session
(work_item_ref bound, workloop-compatible — docs/168). Completion
joins through the silent-session completion stream + bg receipts.

## Safety

- Budget multiplication is additive across lanes (2x lanes = 2x total
  policy budget), each lane individually capped.
- The multiplier never creates sessions without work.
- Settlement is adjudicable (acceptance atoms + #276 verdicts).

## Slices

1. Core FanoutPlan compiler (LANDED — session_fanout.rs, 4/4 tests).
2. Daemon route: POST /v1/silent-sessions/fanout {work_items,
   multiplier} → creates + starts the sessions (existing routes).
3. Pi tool `focusa_fast_forward {multiplier, work_items?}`.
4. GUI: Mission Canvas fast-forward lane control (2x/4x/6x/8x).
