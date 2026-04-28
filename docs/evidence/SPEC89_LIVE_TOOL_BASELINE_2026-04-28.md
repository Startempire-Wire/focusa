# Spec89 Live Focusa Tool Baseline — 2026-04-28

Active bead: `focusa-bcyd.1.4`.

Safe live probes were run against daemon `http://127.0.0.1:8787/v1` plus the full stress harness.

| Family | Probe | Result shape / status | Pain points |
|---|---|---|---|
| health | `GET /health` | keys=ok,uptime_ms,version; status=True | none |
| focus_state | `GET /focus/stack + stress focus writes` | keys=active_frame_id,stack; status=n/a | needs unified envelope in Phase 1 |
| workpoint | `GET /workpoint/current + POST /workpoint/resume` | keys=canonical,next_step_hint,status,warnings,workpoint,workpoint_id; status=completed | needs shared envelope but canonical fields present |
| lineage_intelligence | `GET /lineage/head and /lineage/tree` | keys=head,session_id; status=n/a | bounded output/perf remains Phase 6 gate |
| tree_snapshot_lineage | `GET /focus/snapshots?limit=3` | keys=code,correlation_id,details,message; status=n/a; code=method_not_allowed | snapshot endpoint currently returned error envelope in baseline; route/contract clarity belongs in Phase 1/6 |
| metacognition | `POST /metacog/retrieve` | keys=code,correlation_id,details,message; status=n/a; code=not_found | retrieve endpoint currently returned error envelope with this body shape; quality gates remain Phase 4 |
| work_loop | `GET /work-loop/status` | keys=active_worker,active_writer,authorship_mode,blocker_package,budget_remaining,commitment_lifecycle,consecutive_failures_for_task_class,continuation_inputs,current_task,decision_context,delegated_authorship,enabled; status=idle | writer clarity/preflight remains Phase 4 |

## Full stress baseline

```text
✓ PASS: work_loop_status
✓ PASS: work_loop_context
✓ PASS: work_loop_checkpoint
✓ PASS: work_loop_pause
✓ PASS: work_loop_resume
✓ PASS: work_loop_stop
✓ PASS: ontology_primitives
✓ PASS: ontology_world
✓ PASS: ontology_slices
✓ PASS: cli_workpoint_current
✓ PASS: cli_workpoint_resume
✓ PASS: cli_workpoint_drift_check
=== FOCUSA TOOL STRESS RESULTS ===
passed=37 failed=0 artifacts=/tmp/focusa-tool-stress-1951164
```

## Baseline conclusion

- Current live surfaces are healthy enough to enter Phase 1.
- Main gaps are coherent shared envelope, Workpoint-spine linkage, clearer writer/preflight UX, metacog quality gates, and broader pickup/parity proof.
