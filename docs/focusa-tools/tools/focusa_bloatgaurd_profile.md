# focusa_bloatgaurd_profile

Spec 101 read-only tool for one Bloatgaurd profile preset.

- API: `GET /v1/bloatgaurd/profiles/profile/{name}`
- CLI: `focusa bloatgaurd profile <name>`
- Core: `focusa_core::bloatgaurd::BloatgaurdProfile`
- Side effects: none (`read_state`)

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
