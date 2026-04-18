# Doc78 Production Runtime Evidence — 2026-04-17

Purpose:
- capture a non-isolated runtime proof run for Doc78 F2/F4/F5 closure evidence
- retain replay/governance/checkpoint payload artifacts from the same run

## Command used

```bash
FOCUSA_BASE_URL=http://127.0.0.1:18799 \
FOCUSA_DOC78_PROD_ARTIFACT_DIR=docs/evidence/doc78-production-runtime-2026-04-18-run4 \
bash tests/doc78_production_runtime_governance_replay_smoke.sh
```

## Result

- Tests passed: `16`
- Tests failed: `0`
- Finished at: `2026-04-18T02:22:01Z`

Source: `docs/evidence/doc78-production-runtime-2026-04-18-run4/result.meta`

## Artifact bundle

Directory: `docs/evidence/doc78-production-runtime-2026-04-18-run4/`
Alias used by scorecard verify commands: `docs/evidence/doc78-production-runtime-latest/`

Included payload snapshots:
- `health.json`
- `status_before.json`
- `scope_failure_trace_before.json`
- `pause_flags_set_governance.json`
- `select_next_round1.json`
- `heartbeat_round1.json`
- `select_next_round2.json`
- `heartbeat_round2.json`
- `status_governance_blocked.json`
- `scope_failure_trace_after.json`
- `verification_result_trace_snapshot.json`
- `status_after.json`
- `closure_bundle.json`
- `closure_evidence.json`
- `checkpoints.json`
- `pause_flags_restore.json`
- `harness-output.log`
- `daemon.log`
- `run.meta`
- `result.meta`

## Notes

- Run executed against an already-running daemon endpoint contract (`FOCUSA_BASE_URL`), not the isolated smoke harness ports.
- Governance boundary validation prefers new `scope_failure_recorded` marker event IDs; when telemetry retention/dedup keeps marker IDs stable, the harness falls back to marker presence plus live governance continuation-boundary status semantics (blocked or paused with governance boundary fields).
- Pause flags were restored at the end of the run (`pause_flags_restore.json`) to avoid leaving governance boundary state active.
- Sustained multi-run follow-up evidence is tracked separately in `docs/evidence/DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md`.
