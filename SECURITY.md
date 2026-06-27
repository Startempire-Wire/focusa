# Security

## Reporting a vulnerability

Please report security issues **privately** to `security@focusa.dev` (placeholder until public launch). Do **not** open a public GitHub issue for suspected vulnerabilities.

## Disclosure window

Focusa follows a **90-day coordinated disclosure** policy. We aim to acknowledge within 3 business days and ship a fix within 90 days, depending on severity.

## Supported versions

| Version | Supported |
|---|---|
| Latest dev tag (e.g. `v0.9.x-dev`) | yes |
| Latest tagged release | yes |
| Older releases | best-effort |

## Severity scale

- **Critical** — unauthenticated RCE, daemon takeover, keychain exfiltration.
- **High** — privilege escalation, signed-binary bypass, persistent pairing bypass.
- **Medium** — local DoS, info disclosure of non-secret debug data, daemon repair-loop bypass.
- **Low** — UI glitches, non-security warnings.

## Hardening checklist (for operators)

- Run `focusa codesign inspect` on macOS; do not ship unsigned `.app` to Apple Silicon users.
- Run `focusa pairing transport setup` only with verified phone-reachable transports; do not rely on account-less quick tunnels for production.
- Use `FOCUSA_REQUIRE_COSIGN=1` when installing from release assets.
- Verify the daemon version matches the CLI version before pairing (`focusa pair` blocks mismatched versions).

## Hardening checklist (for contributors)

- Run `cargo deny check licenses` before submitting PRs.
- Do not introduce GPL/AGPL/LGPL dependencies (see `deny.toml`).
- Pairing/transport code changes must pass `tests/spec_pairing_*_static_test.sh`.