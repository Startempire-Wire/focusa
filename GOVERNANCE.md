# Focusa Governance

This document describes Focusa’s long-term support, compatibility, deprecation, and contribution policies. It exists so commercial operators and contributors can plan with confidence.

## 1. License

Focusa is released under the **Business Source License 1.1 (BSL 1.1)** with a 4-year change-date to **Apache License 2.0**. After the change date, each release converts to Apache 2.0 automatically.

- **Non-production use** is granted to everyone under the BSL 1.1.
- **Production use** requires a Focusa Operator or Commercial license.

See [`LICENSE`](LICENSE) and [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## 2. Compatibility policy

- **Patch releases** (`0.9.x-y`): bug fixes only; no breaking changes; same `focusa-daemon` and `focusa-cli` binary contract.
- **Minor releases** (`0.y.0`): additive features; no breaking CLI/API changes within the same minor; deprecated APIs emit warnings ≥ 1 minor before removal.
- **Major releases** (`y.0.0`): allowed to break; one major per year at most, with a 90-day deprecation window documented in the release notes.

## 3. Deprecation policy

- APIs marked `#[deprecated]` in the Rust source must remain for at least one minor release before removal.
- CLI subcommands and flags marked deprecated in help text continue to work for one minor release before removal.
- HTTP routes marked deprecated respond with `Deprecation` and `Sunset` headers per RFC 8594.

## 4. Backwards compatibility guarantees

| Surface | Guarantee |
|---|---|
| `focusa-daemon` HTTP API (existing paths, JSON keys) | stable within a minor |
| `focusa-cli` command names and required flags | stable within a minor |
| Pairing protocol (`/v1/device/pair/*`, `/v1/connect/*`, `/v1/connect/room/*`) | stable across majors with documented migration guide |
| Tauri menubar `apps/menubar` URL surfaces | stable within a minor |
| Shell installer contract (`scripts/install-focusa.sh` flags) | stable within a minor |

## 5. Security disclosure

See [`SECURITY.md`](SECURITY.md). Coordinated disclosure window: 90 days from report.

## 6. Contribution policy

- All contributors must sign the Focusa CLA (BSD-3-Clause-compatible).
- Contributions are licensed under BSL 1.1 (the same license as the project) until the change date, after which they convert with the rest of the codebase.
- Trivial fixes (typos, docs, CI) are accepted without CLA on a case-by-case basis.

## 7. Telemetry

Focusa collects **no telemetry by default**. Any future telemetry must be:
- explicitly opt-in per install,
- documented in `docs/current/PRIVACY.md`,
- reviewable in source (`focusa-core/src/telemetry`).

## 8. Commercial support tiers

- **Community**: GitHub issues, best-effort.
- **Operator**: commercial license, signed release artifacts, response-time SLA.
- **Commercial**: dedicated support, custom integrations, indemnification.

## 9. Reproducibility

Every tagged release is built with a pinned Rust toolchain and produces `SHA256SUMS.txt` plus a cosign keyless signature. See [`docs/current/PORTABILITY_AUDIT.md`](docs/current/PORTABILITY_AUDIT.md) Addendum B.

## 10. Change history

- 2026-06-27: governance baseline established (Slice 5 of mass-adoption hardening).