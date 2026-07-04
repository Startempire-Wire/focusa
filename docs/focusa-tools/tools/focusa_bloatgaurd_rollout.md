# focusa_bloatgaurd_rollout

Spec 101 read-only rollout hardening report.

- API: `GET /v1/bloatgaurd/rollout/report`
- CLI: `focusa bloatgaurd rollout`
- Core: `focusa_core::bloatgaurd::BloatgaurdRolloutReport`
- Side effects: none (`read_state`)

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
