# focusa_bloatgaurd_gate_mode

Spec 101 read-only tool for one Bloatgaurd gate mode.

- API: `GET /v1/bloatgaurd/gate-modes/mode/{name}`
- CLI: `focusa bloatgaurd gate-mode <name>`
- Core: `focusa_core::bloatgaurd::BloatgaurdGateMode`
- Side effects: none (`read_state`)

Use to inspect `A`/`advisory`, `B`/`warning`, or `C`/`fail-candidate` thresholds and allowlist behavior.
