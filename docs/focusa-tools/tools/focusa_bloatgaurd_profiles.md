# focusa_bloatgaurd_profiles

Spec 101 read-only tool for profile presets and operator switches.

- API: `GET /v1/bloatgaurd/profiles/report`
- CLI: `focusa bloatgaurd profiles`
- Core: `focusa_core::bloatgaurd::BloatgaurdProfilesReport`
- Side effects: none (`read_state`)

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
