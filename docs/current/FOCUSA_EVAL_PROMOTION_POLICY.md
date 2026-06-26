# Focusa Eval Promotion Policy

Status: current implementation contract for Spec 114 Phase 0  
Primary spec: `docs/114-public-benchmark-flywheel-spec.md`  
Related: `docs/113-agent-benchmark-spec.md`, `docs/29-telemetry-spec.md`, `docs/31-telemetry-api.md`

## Purpose

Focusa changes should be promoted when eval evidence shows measurable improvement without hidden regressions, leakage, or false public claims.

## Authority Boundary

- Eval harnesses may write append-only evidence to `/v1/evals/*`.
- `/v1/telemetry/*` remains read/export-only.
- Eval results are evidence, not cognition authority.
- Eval Ledger writes cannot mutate Workpoints, Trajectory, Focus State, ontology, prompts, gates, or routing.

## Promotion Inputs

A promotion decision requires:

- `suite_id`
- `run_id`
- `baseline_run_id`
- `candidate_run_id`
- `focusa_version`
- `model_matrix_digest`
- `environment_digest`
- `scoring_commit`
- `public_snapshot_id` when publication is requested
- Focusa-vs-No-Focusa deltas with confidence intervals
- release-over-release deltas
- failure taxonomy summary
- redaction/publication gate result

## Promotion Decision Schema

```json
{
  "decision_id": "promotion-uuid",
  "run_id": "run-2026-06-25-001",
  "candidate_ref": "git:e081836",
  "decision": "promote | hold | rollback | needs_more_runs",
  "reason": "measured uplift passed thresholds and no publication gate failures",
  "primary_metric": "focusa_uplift_score",
  "thresholds": {
    "min_focusa_uplift_score": 1.05,
    "max_cost_regression": 0.10,
    "max_groundedness_regression": 0.00,
    "max_operator_burden_regression": 0.00
  },
  "confidence": {
    "method": "bootstrap_ci",
    "level": 0.95
  },
  "public_snapshot_id": "optional",
  "schema_version": "focusa.eval_promotion_decision.v1"
}
```

## Default Promotion Thresholds

| Metric | Default Rule |
|--------|--------------|
| Focusa Uplift Score | Must improve vs No-Focusa and not regress release-over-release |
| Resolved % | Must improve or remain neutral with cost/operator-burden win |
| Cost per resolved task | Must not regress >10% unless explicitly justified |
| Groundedness | Must not regress |
| Operator burden | Must not regress |
| Secret/publication gate | Must pass for public release |
| Private holdout | Must not materially diverge from public split without investigation |

## Failure-to-Improvement Loop

Every failed eval may create an improvement candidate with:

- failure type
- affected task/scenario/model
- evidence refs
- suspected subsystem
- candidate fix type
- linked bead/spec/workpoint when accepted

Candidate statuses:

```text
observed -> triaged -> planned -> implemented -> rerun_pending -> verified -> promoted | rejected
```

## Release Rules

A release may say “Focusa improved agent performance” only when:

1. Focusa-vs-No-Focusa measured comparison exists.
2. Model/version and scenario matrix are pinned.
3. Raw artifacts and scoring commit are retained.
4. Public snapshot gate passes.
5. Regression summary is included, including inconclusive or negative slices.

## Rollback Rules

Rollback or hold is required when:

- public snapshot gate fails;
- unsupported claims are detected;
- private holdout regression exceeds threshold;
- groundedness regresses;
- cost worsens without success-rate or burden improvement;
- Eval Ledger hash chain is invalid;
- scoring code or model routing was unpinned.
