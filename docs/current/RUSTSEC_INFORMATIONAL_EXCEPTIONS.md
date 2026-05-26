# RustSec Informational Exceptions

Status: accepted informational exceptions for `tests/security_cargo_audit_gate.sh`.

## Current exceptions

| Advisory | Package | Via | Class | Rationale | Review trigger |
| --- | --- | --- | --- | --- | --- |
| RUSTSEC-2024-0436 | `paste` 1.0.15 | `parquet` 58.3.0 → `focusa-cli` export writer | unmaintained | Parquet export is a documented Focusa CLI requirement; latest `parquet` still depends on `paste`; advisory has no patched `paste` version and is informational/unmaintained, not a vulnerability. | Revisit when `parquet` removes `paste`, a maintained fork is adopted upstream, or Focusa replaces the Parquet writer. |

## Non-exceptions fixed in this pass

- Menubar `cookie` low advisory fixed by npm `overrides.cookie=^1.1.1`; `npm audit --audit-level=low` reports 0 vulnerabilities.
- `lru` unsound warning fixed by updating `ratatui` to 0.30.0, which pulls patched `lru` 0.16.4.
- `parquet` updated from 53 to 58.3.0; the only remaining RustSec signal is the documented `paste` informational exception.

## Gate behavior

`tests/security_cargo_audit_gate.sh` fails on vulnerabilities and non-accepted informational warnings. Accepted informational exceptions are counted separately so they stay visible without blocking release/security gates.
