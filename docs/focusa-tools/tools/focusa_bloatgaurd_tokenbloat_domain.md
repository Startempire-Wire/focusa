# focusa_bloatgaurd_tokenbloat_domain

Spec 101 read-only tool for one Tokenbloat Control domain.

- API: `GET /v1/bloatgaurd/tokenbloat/domain/{name}`
- CLI: `focusa bloatgaurd token-domain <name>`
- Core: `focusa_core::bloatgaurd::TokenbloatControl`
- Side effects: none (`read_state`)

Use when a gap maps to `tokenbloat-control` or `tool-call-history-elision`.

- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
