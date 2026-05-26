# API Resource Limits

Status: current resource-exhaustion boundary for Focusa API (OWASP API4 / CWE-400).

## Implemented limit

| Boundary | Default | Override | Enforcement |
| --- | --- | --- | --- |
| HTTP request body size | 1 MiB | `FOCUSA_API_MAX_BODY_BYTES` | Axum `DefaultBodyLimit` applied in `build_router()` before global auth/error middleware. |

The body-size limit covers JSON/body extractors and rejects oversized requests before route handlers allocate unbounded payloads. The default is intentionally small for local-first cognitive state APIs; large artifacts should be stored by reference/handle, not posted as raw payloads.

## Existing bounded-query controls

Several read-heavy routes already use route-level hard caps via `routes::bounded` helpers:

- telemetry event limits
- trace retention limits
- ontology/traverse payload limits
- ECS handle limits
- semantic memory limits

## JSON depth posture

Serde JSON parsing has built-in recursion protection and Focusa now adds an outer byte cap. For public or remote deployments, deeper schema-level validation should reject oversized arrays, excessive object nesting, and raw payload fields before persistence.

## Rate-limit posture

Focusa remains local-first. Non-loopback bind requires auth, and systemd/resource-mode controls limit daemon damage. Network-exposed deployments still need per-client/request-rate controls at reverse proxy or future daemon middleware.

## Follow-up acceptance criteria

1. Add route-specific JSON depth/array-count validation for mutation endpoints.
2. Add per-token/per-IP rate limiting or document required reverse-proxy limits for remote deployments.
3. Add dynamic API fuzz/smoke tests for malformed JSON, oversized bodies, and repeated mutation bursts.
4. Include body-size and rate-limit posture in a future `focusa doctor security` report.
