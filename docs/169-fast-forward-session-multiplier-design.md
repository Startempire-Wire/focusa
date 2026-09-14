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

`compile_fanout(FanoutInput { work_items, multiplier, policy budgets,
orchestrator_capability_refs, worker_capability_refs }) → FanoutPlan`
— deterministic:

- worker lanes = min(multiplier, work_items), PLUS one dedicated
  ORCHESTRATOR lane;
- round-robin task division across worker lanes (stable for the same
  input);
- per-lane budgets = policy budgets (parallelism never stretches
  policy bounds);
- join spec: wait-for-all over the existing silent-session wait route.

## Role-model architecture (CallGraph spec vocabulary)

Per the CallGraph (docs/155): the orchestrator lane binds to a
`FocusaCallFrame` of kind `agent` routed against the STRONG frontier
capability refs; worker lanes bind to frames of kind `tool` routed
against the WEAKER implementation refs. The CallGraph runtime
(route_frame / route_frame_team + the adapter registry) performs the
model selection — the fan-out only declares each lane's frame kind +
capability contract. Strong frontier model = orchestrator (planning,
division, adjudication); weaker models = task implementation.

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
