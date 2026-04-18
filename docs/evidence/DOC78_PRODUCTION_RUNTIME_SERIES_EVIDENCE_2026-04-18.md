# Doc78 Production Runtime Series Evidence — 2026-04-18

Purpose:
- capture sustained (multi-run) production-runtime proof for Doc78 F2/F4 closure review
- retain repeated governance/replay/objective artifacts plus a machine-readable series summary

## Command used

```bash
FOCUSA_BASE_URL=http://127.0.0.1:18799 \
FOCUSA_DOC78_PROD_SERIES_RUNS=6 \
FOCUSA_DOC78_PROD_SERIES_DIR=docs/evidence/doc78-production-runtime-series-2026-04-18-run4 \
bash tests/doc78_production_runtime_series_smoke.sh
```

## Result

- Runs requested: `6`
- Runs completed: `6`
- Runs passed: `6`
- Runs failed: `0`
- Started at: `2026-04-18T02:22:02Z`
- Finished at: `2026-04-18T02:27:25Z`

Source: `docs/evidence/doc78-production-runtime-series-2026-04-18-run4/series-summary.json`

## Artifact bundle

Directory: `docs/evidence/doc78-production-runtime-series-2026-04-18-run4/`
Alias used for follow-on runs: `docs/evidence/doc78-production-runtime-series-latest/`

Top-level artifacts:
- `series.meta`
- `run-records.jsonl`
- `series-summary.json`
- `series-summary.md`
- `series-harness-output.log`
- `daemon.log`
- `run-01/` … `run-06/` (full production harness payload sets)

## Notes

- Each run executes `tests/doc78_production_runtime_governance_replay_smoke.sh` with dedicated artifact directories.
- Series harness seeds non-closure `verification_result` traces before each run for clean-daemon reproducibility.
- Governance continuation-boundary evidence remains valid under retained telemetry through marker-ID-first with retained-marker fallback semantics.
- This run extends sustained proof beyond the earlier five-run baseline to a six-run production series.
- Replay comparative summaries now read raw persisted event rows, so legacy payload variants (including rows missing `handle`) no longer break sustained-run closure evidence.
- Strict replay export (`replay_events`) intentionally stays fail-fast on malformed payload rows to preserve training/export integrity guarantees.
- Governance status verification accepts both blocked and paused continuation-boundary states when governance pause semantics are active.
