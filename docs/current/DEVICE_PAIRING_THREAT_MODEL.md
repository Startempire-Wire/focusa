# Device Pairing Threat Model

Canonical pairing design lives in `docs/53-focusa-device-pairing-spec.md`. This current doc summarizes runtime security requirements for launch/readiness reviews.

## Assets

- Pairing code (`FOCUS-XXXX-XXXX`, 5-minute TTL)
- Device id (UUIDv7)
- Device token (32-byte CSPRNG, base64url-no-pad, 30-day TTL)
- Device ledger (`devices.jsonl`, append-only)
- Pairing URL / public connect URL

## Threats and mitigations

| Threat | Mitigation |
| --- | --- |
| guessed device id | UUIDv7 and rate-limit-ready route boundary |
| reused code | pair-complete is single-use; reuse returns `pair_code_already_used` |
| weak token entropy | 32-byte CSPRNG token encoded base64url-no-pad |
| over-broad scopes | only `read` and `write` scopes accepted |
| malicious URL | pairing URLs must be `https://` or local dev localhost/127.0.0.1 |
| host/path confusion | unsafe agent runtime paths reject with `scope_mismatch` |
| token leak | operator revokes device; append-only revoked ledger record |
| public PWA token exposure | PWA never receives token; joining device polls status |

## Proof

- Static: `tests/device_pairing_threat_model_static_test.sh`
- Live-safe: `tests/device_pairing_endpoint_hardening_live_safe_test.sh`
- Route: `crates/focusa-api/src/routes/device_pairing.rs`
