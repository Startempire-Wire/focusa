# API Resource Limits

Status: current resource-exhaustion boundary for Focusa API (OWASP API4 / CWE-400).

## Implemented limit

| Boundary | Default | Override | Enforcement |
| --- | --- | --- | --- |
| HTTP request body size | 1 MiB | `FOCUSA_API_MAX_BODY_BYTES` | Axum `DefaultBodyLimit` applied in `build_router()` before global auth/error middleware. |
| Mutation route rate limit | 120 requests / 1s / route+caller | `FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW`, `FOCUSA_API_MUTATION_RATE_LIMIT_WINDOW_MS`; set count to `0` to disable | `middleware::rate_limit::mutation_rate_limit_layer` returns HTTP `429` before handlers allocate mutation work. |
| Mutation JSON shape/path guard | max depth 64, array items 2048, object fields 2048; no `../` traversal in path-like JSON fields | `FOCUSA_API_JSON_MAX_DEPTH`, `FOCUSA_API_JSON_MAX_ARRAY_ITEMS`, `FOCUSA_API_JSON_MAX_OBJECT_FIELDS` | `middleware::json_guard::mutation_json_guard_layer` parses JSON mutation bodies and returns HTTP `400` for excessive shape or `json_path_traversal` before route handlers run. |

The body-size limit covers JSON/body extractors and rejects oversized requests before route handlers allocate unbounded payloads. The default is intentionally small for local-first cognitive state APIs; large artifacts should be stored by reference/handle, not posted as raw payloads.

## Existing bounded-query controls

Several read-heavy routes already use route-level hard caps via `routes::bounded` helpers:

- telemetry event limits
- trace retention limits
- ontology/traverse payload limits
- ECS response limits plus a strict 2,048-record hot state/snapshot index; durable cold metadata remains exact-ID rehydratable
- semantic memory limits

## JSON depth posture

Serde JSON parsing has built-in recursion protection and Focusa also applies an outer byte cap. Focusa now adds a mutation JSON shape/path guard for JSON requests: excessive object nesting, array length, object field count, or `../` traversal in path-like JSON fields returns HTTP `400` before route handlers persist or further process the payload (`json_path_traversal`).

## Rate-limit posture

Focusa remains local-first. Non-loopback bind requires auth, and systemd/resource-mode controls limit daemon damage. The daemon now applies an in-process fixed-window rate limit to mutation-style requests by route, method, and caller identity. Caller identity is a bearer-token hash when present, forwarded/real IP hash when supplied by a proxy, or a local anonymous bucket for loopback use. Network-exposed deployments should still add reverse-proxy rate limits as defense in depth.

## Reverse-proxy rate-limit guidance

Use these settings only when Focusa is intentionally exposed beyond loopback, and keep `FOCUSA_AUTH_TOKEN` enabled. Prefer binding Focusa to loopback or Tailscale-only addresses; public Internet exposure should remain exceptional.

Recommended reverse-proxy buckets:

| Route class | Examples | Suggested proxy limit | Notes |
| --- | --- | --- | --- |
| Health | `GET /v1/health` | 120 rpm per IP | Public health can be lenient; no sensitive state. |
| Read/status | `/v1/project/*`, `/v1/workpoint/resume`, `/v1/trajectory/view`, `/v1/predictions/recent` | 60 rpm per token/IP | Reads may reveal P2 project context; require auth outside loopback. |
| Mutation | `/v1/focus/*`, `/v1/workpoint/checkpoint`, `/v1/trajectory/define-goal`, `/v1/metacognition/capture`, `/v1/predictions` | 30 rpm per token/IP | Aligns with daemon mutation guard; lower for shared/team endpoints. |
| Work-loop/control/admin | `/v1/work-loop/*`, `/v1/commands*`, `/v1/release*`, `/v1/tokens*`, `/v1/sync*` | 10 rpm per token/IP plus explicit allowlist | High-impact operations; prefer VPN/Tailscale allowlists. |
| Proxy/model invocation | `/proxy/*`, `/v1/proxy/*` | 10 rpm per token/IP and upstream quota | Protects provider spend and provider payload boundaries. |

Proxy implementation requirements:

- Key limits by bearer token when the proxy can see `Authorization`; otherwise key by client IP plus `X-Forwarded-For`/`X-Real-IP` hygiene.
- Preserve `X-Forwarded-For` or `X-Real-IP` only from trusted proxy hops so daemon-side caller buckets are not spoofed.
- Return HTTP `429` with a small body; avoid reflecting request payloads.
- Keep request-body caps at or below `FOCUSA_API_MAX_BODY_BYTES` unless an endpoint explicitly stores artifacts by handle.
- Treat `FOCUSA_AUTH_TOKEN` as deployment/API auth only, not a license or product entitlement token.

## Doctor security posture

`focusa doctor security` and `focusa --json doctor security` report the current body-size, mutation rate-limit, JSON shape/path, non-loopback auth, and reverse-proxy guidance posture. This command reports deployment/API safety only; it is not a license-plan or entitlement command.

## Follow-up acceptance criteria

No current API resource-limit follow-up remains in this document; keep this section for future audit items.
