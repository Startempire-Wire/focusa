# Prediction Algorithms Implemented

Focusa now includes a dependency-free prediction math core surfaced through `GET /v1/project/card` as `algorithmic_intelligence`. Project-card algorithm runs are persisted as a portable JSONL ledger and learned signal weights are stored as compact JSON.

## Implemented formulas

- Weighted/normalized weighted score
- Sigmoid
- Logit
- Softmax
- Expected value
- Exponential decay
- Exponential moving average
- Z-score
- Brier score
- Log loss

## Current use

Project cards compute bounded signals for:

- trajectory presence
- ontology object coverage
- evidence/proof coverage
- prediction accuracy/calibration
- open prediction pressure
- blocker penalty

Then they produce:

- `readiness_to_execute`
- `need_to_bootstrap_or_rebootstrap`
- `need_to_learn_or_evaluate`
- `risk_probability`
- softmax action probabilities
- expected utility

## Persistence and learning

- Algorithm-run ledger: `project_card_algorithm_runs.jsonl`
- Algorithm-outcome ledger: `project_card_algorithm_outcomes.jsonl`
- Learned weights: `project_card_signal_weights.json`
- Each project-card call records signals, weights, scores, probabilities, expected utility, project root, current ask, and formula version.
- `POST /v1/project/card/outcome` / `focusa project card-outcome` attaches final results to a specific `algorithm_run_id`.
- `success_sequence.ranking_basis` includes outcome count, average score, and outcome bias; outcome scores now influence readiness/refresh/learn probabilities and expected utility.
- Hot-path project-card GET projects prediction-informed weights without persisting them; explicit algorithm-run outcomes persist learned weights and remain clamped for stability.
- Prediction read paths use an mtime/size keyed in-process store cache, and prediction evaluation returns a compact result summary to reduce tool-wrapper payload cost.
- Storage stays portable and append-friendly: no DB migration required for local-first installs.

## Why these first

These algorithms are interpretable, cheap, deterministic, and useful immediately for project intelligence, trajectory refresh decisions, and end-of-task learning loops. Heavier models like ARIMA, Kalman filters, random forests, gradient boosting, and transformers can be added later when Focusa has richer numeric time-series datasets.
