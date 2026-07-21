# Spec 138 First Implementation Work Order

**Status:** ready after Market Lab Stage 0 baseline  
**Scope:** generic Focusa authority only  
**Prohibited:** market-specific types, brokers, orders, live-market activation

## Objective

Implement the first complete vertical slice of Focusa Spec 138 so external domain engines can submit immutable, temporally bound predictions and receive governed outcome, scoring, calibration, learning, and transfer records.

## Required slice

```text
PredictionQuestion
→ PredictionCommitment
→ InformationSet reference
→ OutcomeClaim
→ OutcomeResolution
→ ScoringPolicy
→ PredictionEvaluation
→ Calibration projection
→ LearningCandidate
→ PromotionDecision
→ LearningRecord
→ TransferPrediction / TransferOutcome
```

## Required architecture

- Move canonical lifecycle types into `focusa-core`; API-local structs are projections or request models only.
- Use append-only typed lifecycle events and scoped durable projections.
- Reference Spec 137 temporal authority for creation, as-of, horizon, resolution, freshness, review, expiry, and transfer windows.
- Freeze outcome resolver and scoring policy before resolution.
- Keep forecast probability, source reliability, evidence confidence, model confidence, and resolution confidence distinct.
- Canonical scores identify scorer, version, direction, range, assumptions, and evidence.
- High-confidence misses and unexpected outcomes create structured learning candidates.
- Promotion requires typed metric values, baselines, sample size, evidence, applicability, review, expiry, and rollback.
- Preserve conflicts, supersession, revocation, negative transfer, and failed learning.
- Maintain backward-compatible projections for current Prediction and Metacognition APIs, CLI, and Pi tools.

## Acceptance

- Core type and schema tests.
- Scorer registry tests.
- Legacy migration and compatibility tests.
- Restart and recovery tests.
- Scope-isolation tests.
- High-confidence-miss learning test.
- Promotion rejection for prose-only metrics.
- Transfer success and negative-transfer tests.
- Evidence and Receipt references.
- No market-domain dependency.

## Coordination

Market Lab implementation directive:

```text
Startempire-Wire/lab.focusa.dev
docs/agent/GPMI-MPMI-IMPLEMENTATION-DIRECTIVE.md
```

Do not begin this slice from assumptions about the server. Consume the accepted Stage 0 capability matrix and baseline commits first.
