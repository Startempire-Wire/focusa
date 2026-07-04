# focusa_bloatgaurd_routine

Spec 101 read-only tool for one named routine.

- API: `GET /v1/bloatgaurd/routines/routine/{name}`
- CLI: `focusa bloatgaurd routine <name>`
- Core: `focusa_core::bloatgaurd::BloatgaurdRoutine`
- Side effects: none (`read_state`)

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
