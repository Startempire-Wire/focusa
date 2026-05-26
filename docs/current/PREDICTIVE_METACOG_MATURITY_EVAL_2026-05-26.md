# Predictive & Metacognitive Maturity Evaluation — 2026-05-26

Verdict: **not yet 9/10**.

## Current scores

| Feature set | Score | Why |
| --- | ---: | --- |
| Predictive power | 8.3/10 | Record/recent/evaluate/stats tools exist across API/CLI/Pi; live store has 56 predictions and 21 evaluated; predictions carry context and can feed metacognition. Still needs stronger calibration UI, automatic project-level rollups, and more routine dogfood enforcement before 9/10. |
| Metacognition | 8.5/10 | Capture/retrieve/reflect/adjust/evaluate loop exists with durable storage, recent readbacks, promotion, disk fallback, and composite Pi loop. Doctor returned 5 relevant candidates with confidence. Still needs richer outcome metrics, Focus Slice surfacing, and automatic consolidation/promotion cadence before 9/10. |

## Evidence checked

- `focusa_predict_stats`: 56 predictions, 21 evaluated.
- `focusa_metacog_doctor`: 5 relevant candidates, all with confidence.
- `tests/spec80_impl_metacognition_api_contract_test.sh`: pass.
- `tests/spec96_metacog_evaluation_readback_static_test.sh`: pass.
- `tests/spec96_ontology_prediction_promotion_static_test.sh`: pass.
- `tests/spec98_prediction_metacog_flywheel_static_test.sh`: pass.
- Prior maturity audit rated both prediction and metacognition at 8/10 before the current hardening pass.

## What keeps them below 9/10

1. Predictive calibration is present but not yet a first-class daily cockpit/Focus Slice surface.
2. Metacog outcomes can promote learning, but promotion quality is still mostly heuristic instead of statistically learned from repeated tasks.
3. There is no automatic project-level maturity rollup from prediction outcomes, metacog evaluations, CI/test results, and user value signals.
4. Agents still must remember to run retrieve → predict → act → evaluate; this is not yet consistently enforced by workflow gates.
5. Dataset/export/feed tooling exists adjacent to the loop, but not enough to continuously train or benchmark the predictive/metacog system.

## Path to 9/10

- Add `PREDICTIVE_CONTEXT` / `METACOG_CONTEXT` to Focus Slice with recent accuracy, relevant lessons, pending evaluations, and recommended next prediction.
- Add an automatic end-of-task evaluator that closes predictions and proposes metacog captures from test/evidence outcomes.
- Add a maturity scoreboard generated from tests, contracts, docs, evidence, runtime probes, prediction outcomes, and metacog evaluations.
- Add a recurring consolidation job that prunes weak signals, promotes strong lessons, and creates follow-up predictions.

## Bottom line

Predictive and metacognitive features are now **strong, usable, and real**—roughly high-8 maturity—but **not honestly 9/10 yet** until calibration, surfacing, and automatic outcome compounding become routine rather than agent-driven.
