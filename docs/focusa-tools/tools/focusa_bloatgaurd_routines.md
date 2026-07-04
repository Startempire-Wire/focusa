# focusa_bloatgaurd_routines

Spec 101 read-only tool for named routines and automation matrix.

- API: `GET /v1/bloatgaurd/routines/report`
- CLI: `focusa bloatgaurd routines`
- Core: `focusa_core::bloatgaurd::BloatgaurdRoutinesReport`
- Side effects: none (`read_state`)

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
