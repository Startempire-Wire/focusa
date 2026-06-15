# focusa_bloatgaurd_domain

Spec 101 read-only tool for one Bloatgaurd budget domain.

- API: `GET /v1/bloatgaurd/domain/{name}`
- CLI: `focusa bloatgaurd domain <name>`
- Core: `focusa_core::bloatgaurd::BloatgaurdDomainState`
- Side effects: none (`read_state`)

Use when a gap maps to one budget domain such as `output-firewall`, `docs-diet`, or `dead-code-safety`.
