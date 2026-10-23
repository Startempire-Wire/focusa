# Focusa Agent Intelligence Evals

**Current status: unmeasured.** The existing runner checks scenario definitions; it does not execute agents or measure whether Focusa makes them more effective. Issue353 remains open for execution-backed comparative evaluation.

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
ARM=on scripts/run-agent-intelligence-evals.sh
ARM=off scripts/run-agent-intelligence-evals.sh
ARM=candidate scripts/run-agent-intelligence-evals.sh
```

The existing `tests/evals/agent_intelligence_cases.json` is the single scenario-definition owner. Its v1 schema contains categories, goals, input signals, expected behavior, required references, metric names and expected thresholds—not observed `score` fields. Repository references and declared tool names are checked without executing them. Missing references, self-supplied scores, invalid arms and malformed definitions fail with exit1.

Valid definitions return **exit78 / UNMEASURED**, not PASS. ARM is a validated requested label, not proof that configurations executed or state was isolated. No aggregate score is computed from definitions; lowering a threshold cannot manufacture observations. No model, provider, installation or build is invoked by these definition checks.

## Acceptance still required

A future measured case may apply `score >= threshold` only to metrics independently derived from captured outcomes. Aggregate score and comparative acceptance additionally require isolated ON, OFF/baseline and candidate executions; exact source/runtime/model/harness/configuration provenance; immutable evidence; adequate samples; and hard-safety non-inferiority. None is established by the current definition checker.

## Release-journal boundary

The canonical release journal stops at unavailable agent observations, before unrelated build/release checks. Runner exit0 alone is also insufficient. It cannot publish a newly passed benchmark or treat a historical fixture-derived v1 benchmark as current comparative evidence. Existing append-only records and prior releases are preserved, not edited or revoked.

The benchmark remains advisory and never mutates Focus State. Definition checks and refusal regressions are not model/harness efficacy measurements and do not close353 or comparative acceptance under265.
