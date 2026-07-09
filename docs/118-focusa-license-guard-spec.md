# Spec 118 — focusa-license: LicenseGuard Module

**Status:** Implemented (bead focusa-nbai)
**Crate:** `crates/focusa-license/`
**Owner:** Verious Smith
**Created:** 2026-07-09

## 1. Purpose

LicenseGuard is a small, focused crate that gives the Focusa daemon a typed way to
evaluate whether a given operation is permitted under the operator's current
Focusa license tier. It enforces the BSL 1.1 boundary: personal/eval/educational
use is free; commercial use requires a paid license; hosted/multi-tenant and
product-embedding are commercial-only.

It is intentionally separate from `focusa-core` (which owns state) and
`focusa-cli` (which owns commands). LicenseGuard has no I/O dependencies; it is a
pure-function decision module that any crate can use.

## 2. Tier model

Three tiers:

| Tier     | Issued by         | Commercial use | Hosted / embedding | Telemetry | Local eval | Expiry  |
|----------|-------------------|----------------|---------------------|-----------|------------|---------|
| `eval`   | Self (default)    | warning only   | denied              | denied    | permitted  | 7 days  |
| `licensed` | `FOCUSA_LICENSE_KEY` validated | permitted | permitted           | denied*   | permitted  | none    |
| `open`   | post-BSL change   | permitted      | permitted           | denied*   | permitted  | none    |

\*Focusa is no-telemetry by default; even Licensed/Open tiers cannot send telemetry.

## 3. Capability map

`LicenseGuard::check(Capability)` returns one of:

- `Permitted` — capability allowed.
- `PermittedWithWarning { warning }` — allowed but operator should be informed (eval+commercial).
- `Denied { reason }` — capability refused; caller must escalate.

Caller can use `require(capability)` to short-circuit denial into `LicenseError::Denied`.

## 4. Resolution order

`resolve_license_guard()` reads in this order:

1. `FOCUSA_LICENSE_KEY` env var → `LicenseGuard::licensed(...)` (commercial tier).
2. `~/.config/focusa/license.json` — durable cached record.
3. `~/.focusa/license.toml` — per-project override.
4. Self-issued `LicenseGuard::eval(7)` (default 7-day offline grace).

## 5. BSL boundary

`focusa-license` does NOT embed any BSL-1.1 text; it only enforces the tier
behavior. The actual license text is in `LICENSE.md` at the repo root. LicenseGuard
checks tier × capability, not BSL clauses directly. This keeps the crate
operator-facing and avoids legal-text drift.

## 6. Tests

`cargo test -p focusa-license` runs 9 unit tests covering:
- tier × capability matrix
- `require()` returning warning/denied
- JSON round-trip of Tier enum
- `Tier::Eval` expiring correctly
- tier-label mapping

All pass as of 2026-07-09.

## 7. Static test (BSL boundary + tier enforcement)

The `tests::require_returns_warning_or_denied` test pins the most security-critical
contract: commercial use is allowed under eval (with warning) but hosted_mode is
hard-denied under eval. This is the BSL-1.1 boundary in code.

## 8. Daemon integration

The `focusa-api` daemon calls `focusa_license::resolve_license_guard()` at
startup, logs the tier + expiry + key fingerprint, and serves `/v1/license/status`
for runtime inspection. Soft-warns when commercial use is requested under eval.

The `focusa-cli` exposes license status via `focusa license status` (existing).

The `focusa-tui` can gate commercial-only features behind `LicenseGuard` checks.
The `focusa-installer` writes the eval license.json on first install and respects
a 7-day cap.

## 9. Future

When the BSL change date is reached, the daemon will automatically upgrade existing
eval/license records to `Open` tier via a one-time migration. Until then,
`Open` is a placeholder used only for tests.
