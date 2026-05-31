# Dynamic Local API Security Smoke

Status: local dynamic smoke suite for OWASP API4/CWE-400 and malformed-input posture.

## Script

`tests/security_dynamic_api_smoke_test.sh`

## What it proves

1. Starts a temporary `focusa-daemon` on a random loopback port with an isolated temporary data directory.
2. Sets `FOCUSA_API_MAX_BODY_BYTES=4096` to exercise the request body limit quickly.
3. Verifies `/v1/health` responds.
4. Sends malformed JSON to `/v1/telemetry/trace` and requires a non-2xx response.
5. Sends an oversized JSON body and requires HTTP `413`.
6. Sends schema-level malformed payloads to representative mutation route families and requires HTTP `400` or `422`.
7. Sends a small burst of health requests to verify the daemon remains responsive after rejected inputs.

## Boundaries

- The suite uses loopback only and does not touch the production daemon port.
- The suite writes build output to `CARGO_TARGET_DIR` and state to a temp `FOCUSA_DATA_DIR`.
- It requires `cargo`, `curl`, and `python3`.
- It is intentionally bounded; deeper route fuzzing remains future work.

## CI coverage

`./scripts/ci/run-spec-gates.sh` runs both `tests/security_dynamic_api_smoke_static_test.sh` and `tests/security_dynamic_api_smoke_test.sh`. In CI, the dynamic smoke reuses the already built `focusa-daemon` through `DAEMON_BIN` when available. The gate also runs `tests/security_non_loopback_auth_guard_static_test.sh` and `tests/security_non_loopback_auth_guard_dynamic_test.sh` to prove non-loopback startup fails without `FOCUSA_AUTH_TOKEN` and succeeds with auth enforced.

## Follow-up

- Add repeated mutation burst tests once route-scoped rate limits exist.
