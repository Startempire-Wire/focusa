# Security Policy

Please report security issues to `security@focusa.dev`.

Do not open public issues for suspected vulnerabilities.

Include:

- affected version or commit
- affected command/API surface
- reproduction steps
- impact
- suggested mitigation, if known

## Disclosure window

Focusa follows coordinated disclosure. We aim to acknowledge reports within 3 business days and provide status updates as investigation proceeds.

## Supported versions

| Version | Supported |
|---|---|
| Latest tagged release | yes |
| Latest dev tag | best-effort |
| Older releases | best-effort |

## Security expectations

- Keep the daemon bound to loopback or a trusted private network unless you intentionally expose it.
- Treat pairing links, device tokens, license keys, and proof exports as sensitive.
- Use signed release assets where available and verify the CLI/daemon versions match.
- Do not include secrets, private paths, customer data, or full chat logs in public bug reports.
