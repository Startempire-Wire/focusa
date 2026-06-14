# Eval Metrics Dashboard

The eval metrics dashboard is an operator-facing read model for Focusa quality signals. It summarizes evaluation outcomes without replacing Workpoint/Trajectory authority.

## Metric sources

- Context Cognition curator eval runs (`precision`, `recall`, `F1`)
- Cognition optimizer artifacts (`baseline_score`, `eval_score`, promotion/rollback decision)
- Prediction records (`confidence`, evaluation score, calibration accuracy)
- Metacognition adjustments (`observed metrics`, promotion outcome)
- Tool contract validation counts and failures
- Release proof / runtime status gates
- Agent intelligence evals (`FOCUSA_AGENT_INTELLIGENCE_EVALS.md`)

## Dashboard cards

| Card | Purpose |
| --- | --- |
| Curator quality | precision/recall/F1 by eval case and latest promoted artifact |
| Prediction calibration | evaluated vs open predictions, accuracy, score trend |
| Optimizer promotions | promoted/rolled-back artifacts and reasons |
| Tool contract health | tool count, contract count, missing contracts |
| Release proof health | latest version/status/proof command result |
| Agent intelligence | scope verification, Workpoint pickup, evidence quality, drift avoidance |

## Minimum filters

- project root
- continuity id
- date/time window
- module name (`curator`, `optimizer`, `prediction`, `metacog`)
- status (`promoted`, `rolled_back`, `passed`, `failed`, `open`)

## Authority and privacy

Dashboard metrics are advisory. They do not promote artifacts, close beads, mutate Workpoints, or publish public data. Public display must use redacted project identity and pass `PUBLIC_STREAM_REDACTION_POLICY.md` gates.

## Suggested proof commands

```bash
focusa predict stats
focusa context-cognition optimizer-artifacts --module-name curator --json
focusa context-cognition curate-eval --help
node scripts/validate-focusa-tool-contracts.mjs
scripts/verify-doc-version-consistency
```

## Proof

- Static guard: `tests/eval_metrics_dashboard_static_test.sh`
- Related docs: `FOCUSA_AGENT_INTELLIGENCE_EVALS.md`, `PUBLIC_PROOF_BUNDLE_VIEWER.md`, `CURRENT_RUNTIME_STATUS.md`
