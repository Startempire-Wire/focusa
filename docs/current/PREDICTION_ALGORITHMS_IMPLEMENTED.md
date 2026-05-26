# Prediction Algorithms Implemented

Focusa now includes a dependency-free prediction math core surfaced through `GET /v1/project/card` as `algorithmic_intelligence`.

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

## Why these first

These algorithms are interpretable, cheap, deterministic, and useful immediately for project intelligence, trajectory refresh decisions, and end-of-task learning loops. Heavier models like ARIMA, Kalman filters, random forests, gradient boosting, and transformers can be added later when Focusa has richer numeric time-series datasets.
