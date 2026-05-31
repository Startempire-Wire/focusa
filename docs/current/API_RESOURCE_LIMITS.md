# API Resource Limits

Status: current resource-exhaustion boundary for Focusa API (OWASP API4 / CWE-400).

## Implemented limit

| Boundary | Default | Override | Enforcement |
| --- | --- | --- | --- |
| HTTP request body size | 1 MiB | `FOCUSA_API_MAX_BODY_BYTES` | Axum `DefaultBodyLimit` applied in `build_router()` before global auth/error middleware. |
| Mutation route rate limit | 120 requests / 1s / route+caller | `FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW`, `FOCUSA_API_MUTATION_RATE_LIMIT_WINDOW_MS`; set count to `0` to disable | `middleware::rate_limit::mutation_rate_limit_layer` returns HTTP `429` before handlers allocate mutation work. |
| Mutation JSON shape guard | max depth 64, array items 2048, object fields 2048 | `FOCUSA_API_JSON_MAX_DEPTH`, `FOCUSA_API_JSON_MAX_ARRAY_ITEMS`, `FOCUSA_API_JSON_MAX_OBJECT_FIELDS` | `middleware::json_guard::mutation_json_guard_layer` parses JSON mutation bodies and returns HTTP `400` for excessive shape before route handlers run. |

The body-size limit covers JSON/body extractors and rejects oversized requests before route handlers allocate unbounded payloads. The default is intentionally small for local-first cognitive state APIs; large artifacts should be stored by reference/handle, not posted as raw payloads.

## Existing bounded-query controls

Several read-heavy routes already use route-level hard caps via `routes::bounded` helpers:

- telemetry event limits
- trace retention limits
- ontology/traverse payload limits
- ECS handle limits
- semantic memory limits

## JSON depth posture

Serde JSON parsing has built-in recursion protection and Focusa also applies an outer byte cap. Focusa now adds a mutation JSON shape guard for JSON requests: excessive object nesting, array length, or object field count returns HTTP `400` before route handlers persist or further process the payload.

## Rate-limit posture

Focusa remains local-first. Non-loopback bind requires auth, and systemd/resource-mode controls limit daemon damage. The daemon now applies an in-process fixed-window rate limit to mutation-style requests by route, method, and caller identity. Caller identity is a bearer-token hash when present, forwarded/real IP hash when supplied by a proxy, or a local anonymous bucket for loopback use. Network-exposed deployments should still add reverse-proxy rate limits as defense in depth.

## Follow-up acceptance criteria

1. Document recommended reverse-proxy rate-limit settings for remote deployments.
2. Include body-size/rate-limit/JSON-shape posture in a future `focusa doctor security` report.
