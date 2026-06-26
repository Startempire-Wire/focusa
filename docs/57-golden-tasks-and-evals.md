# Golden Tasks and Evals

## Purpose

This document defines how Focusa proves value.

Architecture elegance is not enough.
The system must show measurable gains on realistic software tasks.

## Eval Thesis

Focusa succeeds only if:
- ontology-backed slices improve action quality
- behavioral alignment improves actual use of decisions/constraints
- continuity is preserved better than raw harness flow
- weaker models benefit from the structured environment

## Golden Tasks

1. resume interrupted refactor
2. debug failing test with bounded working set
3. preserve conventions during feature addition
4. recover after wrong turn
5. maintain continuity across context windows
6. compare weaker model with ontology slice vs without
7. compare Pi with behavioral-alignment rules vs without
8. operator steering under active Focusa state
9. correction handling

## Core Metrics

- mission retention
- working-set precision
- irrelevant-context reduction
- constraint-check rate before risky actions
- decision-consult rate in repeated-pattern zones
- decision-distillation rate
- repeated-mistake rate
- convention adherence
- recovery success rate
- token use
- latency impact
- degraded-mode behavior quality

## Eval Design Rules

1. Use realistic software tasks, not synthetic trivia.
2. Compare against raw harness behavior where possible.
3. Separate infrastructure compliance from behavioral alignment.
4. Preserve replay artifacts for auditability.
5. Track both quality and operational cost.
6. Run matched arms for market evidence: `no_focusa`, `passive_focusa`, `tool_only_focusa`, `full_focusa`.
7. Use blind or deterministic judges where possible; store judge version and evidence refs.
8. Record eval runs through the append-only Eval Ledger (`/v1/evals/*`), not mutable telemetry endpoints.
9. Include multi-model scenario reporting: `model_provider`, `model_id`, `model_version`, `model_class`, `scenario_id`, and pricing snapshot.
10. Public reports must lead with Focusa-vs-No-Focusa, then show model-by-scenario uplift and diagnostic ablations.

## Eval Ledger Requirement

Golden evals write first-class run evidence through:

```http
POST /v1/evals/runs
POST /v1/evals/runs/{run_id}/events
POST /v1/evals/runs/{run_id}/complete
GET  /v1/evals/runs/{run_id}
GET  /v1/evals/compare?baseline=<run_id>&candidate=<run_id>
```

The Eval Ledger is append-only, idempotent, eval-mode scoped, and non-cognitive. CTL may observe/index eval events for aggregate reports, but `/v1/telemetry/*` remains read/export only.

## Success Condition

This document is satisfied when evals show that ontology-backed, behavior-enforced Focusa materially improves continuity, correctness, action quality, time horizon, and cost over raw harness and passive-integration baselines.
