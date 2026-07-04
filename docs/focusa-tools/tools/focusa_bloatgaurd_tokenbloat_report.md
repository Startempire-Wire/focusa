# focusa_bloatgaurd_tokenbloat_report

Spec 101 read-only tool for Tokenbloat Control domains 5.9–5.10.

- API: `GET /v1/bloatgaurd/tokenbloat/report`
- CLI: `focusa bloatgaurd tokenbloat`
- Core: `focusa_core::bloatgaurd::TokenbloatReport`
- Side effects: none (`read_state`)

Use before prompt/context reduction work to review stable-prefix compression and structured tool-call history elision controls.

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
