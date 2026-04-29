# Efficiency Guide

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

This page documents the current Spec92 token-budget telemetry slice. Cache metadata now has an initial bounded Spec92 metadata surface.

## Current token-budget surfaces

Pi extension hook:

```text
before_provider_request
```

Daemon API:

```bash
curl -sS http://127.0.0.1:8787/v1/telemetry/token-budget/status | jq .
curl -sS -X POST http://127.0.0.1:8787/v1/telemetry/token-budget \
  -H 'Content-Type: application/json' \
  -d '{"budget_class":"ok","input_token_estimate":1200,"payload_hash":"example"}' | jq .
```

CLI:

```bash
focusa telemetry token-budget
focusa --json telemetry token-budget --limit 10
focusa tokens doctor
focusa tokens compact-plan
```

Pi doctor:

```text
focusa_tool_doctor scope="spec92"
```

## What is recorded

The provider hook records bounded metadata only:

- payload hash
- repeated prefix hash
- payload byte size
- input token estimate
- message count
- tool-schema token estimate
- budget class: `ok`, `watch`, `high`, or `critical`
- cache eligibility

Raw provider payloads and secrets are not stored by default.

## Current recovery guidance

If budget class is `high` or `critical`:

```bash
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
focusa telemetry token-budget
```

Then compact large tool-result history or store large evidence in ECS handles before continuing.

## Cache phase status

Current cache metadata commands:

```bash
focusa cache doctor
focusa --json cache doctor --limit 10
```


## Current cache metadata surfaces

Daemon API:

```bash
curl -sS http://127.0.0.1:8787/v1/telemetry/cache-metadata/status | jq .
curl -sS -X POST http://127.0.0.1:8787/v1/telemetry/cache-metadata \
  -H 'Content-Type: application/json' \
  -d '{"cache_key":"example","cache_eligible":true,"payload_hash":"example"}' | jq .
```

CLI:

```bash
focusa cache doctor
focusa --json cache doctor --limit 10
```


## Full doctor command center

```bash
focusa doctor
focusa --json doctor
```

The full doctor aggregates daemon health, daemon executable path, API route/capability inventory, Spec90/Spec91 proof surfaces, Pi skill paths, Workpoint canonicality, Work-loop writer state, token telemetry, cache metadata, Mac app package presence, release docs, Guardian scanner presence, recovery commands, and evidence refs.
