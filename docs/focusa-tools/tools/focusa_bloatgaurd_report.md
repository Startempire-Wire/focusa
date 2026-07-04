# focusa_bloatgaurd_report

Spec 101 read-only tool for the compact Bloatgaurd budget report.

- API: `GET /v1/bloatgaurd/report`
- CLI: `focusa bloatgaurd report`
- Core: `focusa_core::bloatgaurd::BloatgaurdReport`
- Side effects: none (`read_state`)

Use before cleanup or context-budget work to see domains 5.1-5.8 and their budget checks.

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
