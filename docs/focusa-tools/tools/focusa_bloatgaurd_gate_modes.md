# focusa_bloatgaurd_gate_modes

Spec 101 read-only tool for Bloatgaurd gate modes A/B/C.

- API: `GET /v1/bloatgaurd/gate-modes/report`
- CLI: `focusa bloatgaurd gate-modes`
- Core: `focusa_core::bloatgaurd::BloatgaurdGateModesReport`
- Side effects: none (`read_state`)

Use before enabling Bloatgaurd enforcement to inspect deterministic modes, thresholds, allowlist entries, and report schema fields.
