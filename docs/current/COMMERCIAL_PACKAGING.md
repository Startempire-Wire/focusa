# Commercial Packaging

Focusa packaging is local-first by default: the product is not a cloud memory service, and commercial packaging must preserve operator-controlled data, explicit scope authority, and redaction-first public surfaces.

## Editions

| Edition | Audience | Packaging | Support boundary |
| --- | --- | --- | --- |
| Community source | individual developers | source checkout + CLI/daemon build | best-effort docs/tests |
| Pro local | professional operators | signed CLI/daemon + menubar bundle | install/update support, local data ownership |
| Team self-hosted | teams/multi-agent workstreams | daemon service + adapters + policy docs | multi-agent scope, backup/migration, security review |
| Enterprise | regulated orgs | self-hosted bundle + support contract | deployment review, SSO/auth integration planning, audit artifacts |

## Package artifacts

- Focusa daemon binary
- Focusa CLI binary
- Mac menubar app bundle
- Pi extension package
- generated current docs (`docs/current/*`)
- release proof bundle (`focusa release prove --tag <tag>`)
- security/trust docs
- installer/update policy
- migration/backup policy

## Commercial readiness gates

- Version consistency passes.
- Tool-surface summary is current.
- Security/trust docs exist.
- Installer/update policy exists.
- Migration/backup policy exists.
- Public proof artifacts are redacted and `publish_allowed=true` only after review.
- License and billing terms are explicit before paid distribution.

## License and billing placeholders

Current repository badge indicates BSL-1.1 licensing. Any paid packaging must include current license text, allowed commercial use terms, support SLA, billing owner, refund/cancellation policy, and data-processing boundaries before launch.

## Non-goals

- No hosted cloud-memory claim by default.
- No automatic public publishing of private project state.
- No adapter-specific fork of cognitive authority.
- No installer replacing live production binaries without Context Authority preflight.

## Proof

- Static guard: `tests/commercial_packaging_static_test.sh`
- Related docs: `SECURITY_MODEL.md`, `FIRST_RUN_FLOW.md`, `GOLDEN_WORKFLOW_PUBLIC_DEMO.md`
