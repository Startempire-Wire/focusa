# Tool Result Envelope v1

Current Focusa Pi tools preserve visible summaries and attach structured `tool_result_v1` metadata when available.

## Purpose

Agents should not parse prose to decide whether a Focusa tool succeeded. They should inspect structured fields such as status, failure class, retry posture, recovery/misuse hints, canonical/degraded flags, evidence refs, side effects, and next-tool hints.

## Common fields

- `ok` — boolean success indication when available.
- `status` — current result state such as completed, accepted, pending, blocked, unavailable, validation_rejected, or degraded.
- `canonical` — true when Focusa says the result is authoritative.
- `degraded` — true when output is a fallback or partial result.
- `summary` — short human-readable result.
- `retry` — retry safety, posture, and reason.
- `recovery_hint` — plain next recovery action; use this before retrying.
- `misuse_hint` — likely out-of-order, scope, validation, or resource-use mistake to fix.
- `side_effects` — whether state was read, written, linked, checkpointed, or left unchanged.
- `evidence_refs` — stable proof refs associated with the result.
- `next_tools` — recommended next Focusa tools.
- `error` — structured error details when applicable.
- `raw` — compatibility copy of the underlying response.

## Agent usage rule

Use `status`, `failure_class`, `canonical`, `degraded`, `retry`, `recovery_hint`, `misuse_hint`, and `next_tools` for recovery decisions. Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as a recovery state, not as a final success.

## No-deadend rule

If a tool fails or blocks:

1. Read `failure_class` to understand the cause.
2. Check `retry.posture`; only retry unchanged when posture says it is safe.
3. Follow `recovery_hint` and fix `misuse_hint` before retrying.
4. Use `next_tools` as the safe route; do not stop at the error unless the operator asks.

Contract fixture: `tests/fixtures/spec89_tool_result_failure_recovery_sample.json` proves failed/out-of-order results carry `recovery_hint`, `misuse_hint`, and non-empty `next_tools`.

## API route failure envelopes

Public Focusa API routes should avoid bare HTTP status failures for caller-actionable errors. Validation, permission, dispatch, persistence, lookup, and upstream failures should return the same no-deadend fields (`status`, `failure_class`, `why`, `recovery_hint`, `misuse_hint`, `next_tools`, and `details.tool_result_v1`) so Pi, CLI, and non-Pi agents can recover without guessing.

Sync routes follow this contract for local persistence failures, missing peers, malformed receive/transfer payloads, delegated receive/transfer rejections, and remote peer upstream failures.

Instance, token, visual workflow, memory mutation, session resume, and sync receive/transfer implementation routes follow the same contract for missing inputs, invalid enum values, missing tokens, malformed visual evidence content, malformed sync timestamps, and daemon/persistence dispatch failures.
