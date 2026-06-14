# Token and Secret Handling

## Rules

- Never paste tokens, API keys, pairing tokens, cookies, or private credentials into public cards, docs, logs, or final reports.
- Store secrets in environment variables, Keychain/OS secret storage, or approved daemon config.
- Treat pairing tokens as bearer credentials with 30-day TTL and revoke on loss.
- Treat Focusa API auth tokens as deployment secrets.
- Redact secret-like values before public stream/publish surfaces.

## Runtime boundaries

- Pairing token: generated server-side, 32-byte CSPRNG, base64url-no-pad.
- Pairing status may return the token only to the joining-device polling flow after completion.
- Public stream cards default to `publish_allowed=false` and `secret_scan_status=not_required_no_raw_payload` unless a future publisher runs a scan.
- Error envelopes should use `failure_class` and recovery hints, not raw secret-bearing payloads.

## Verification

- Device pairing hardening tests verify token shape and revoke/list behavior.
- Public stream redaction tests verify no raw project path in redacted scope id.
- Release/security reviews should include secret scan output before public release.
