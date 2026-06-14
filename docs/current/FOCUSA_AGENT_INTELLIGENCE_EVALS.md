# Focusa Agent Intelligence Evals

Focusa's “makes agents smarter” claim is measured with bounded, repeatable eval cases. The eval suite checks whether agents preserve scope, continuation, proof, context, execution structure, learning, and safety.

## Categories

1. **Continuity** — resume accuracy after compaction, Workpoint recovery after model switch, correct next action after long sessions.
2. **Scope** — wrong-project mutation prevention, same-project/different-continuity rejection, broad-root rejection.
3. **Evidence** — evidence recall rate, claims with linked proof, missing evidence detection.
4. **Context** — context selection precision/recall/F1, under-selected critical file rate, over-budget exclusion correctness.
5. **Execution** — Call Stack Design adherence, implementation drift from blueprint, STG/Waypoint completion rate.
6. **Learning** — prediction calibration improvement, metacog lesson reuse, repeated mistake reduction.
7. **Safety** — risky mutation blocked correctly, planning prompt does not mutate, pairing does not become binary install.

## Runner

```bash
scripts/run-agent-intelligence-evals.sh
```

The runner is intentionally local/static for the first version. It reads `tests/evals/agent_intelligence_cases.json`, validates the schema and category coverage, computes aggregate scores from fixture expectations, and fails if any required category is missing or below threshold.

## Case schema

Each case declares:

- `id`
- `category`
- `goal`
- `input_signal`
- `expected_behavior`
- `required_refs`
- `metric`
- `score`
- `threshold`

## Promotion rule

A benchmark case passes when `score >= threshold`. The suite passes only when every required category has at least one passing case and the aggregate score is at least `0.80`.

## Proof boundary

The benchmark is advisory. It never mutates Focus State and does not replace Workpoint evidence. Failing evals create backlog work; passing evals can support release proof, project card outcome learning, and metacognition capture.
