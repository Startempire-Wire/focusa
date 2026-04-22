# SPEC82 Pi Extension Perf Reevaluation

**Date:** 2026-04-22  
**Prompt driver:** operator reported Pi still felt laggy with Focusa enabled.

## Live findings before patch

- active daemon RSS observed: `~1.05 GiB`
- live semantic record count: `41`
- live ECS handle count: `906`
- live `/v1/ecs/handles` payload size: `269259` bytes
- Pi extension `context` hook was fetching:
  - `getFocusState()` (`/focus/stack` + `/ascc/state`)
  - full `/memory/semantic`
  - full `/ecs/handles`

## Confirmed hot-path issue

The Pi extension context builder was paying a repeated cost for large, mostly unnecessary payloads on the model hot path. The worst offender was full ECS handle enumeration despite the prompt slice only needing a small recent summary.

## Changes made

### API shaping
- `GET /v1/memory/semantic` now supports:
  - `limit`
  - `summary_only=true`
  - response `count`
- `GET /v1/ecs/handles` now supports:
  - `limit`
  - `summary_only=true`
  - response `count`

### Pi extension hot-path reduction
- `getFocusState()` now uses a short hot cache / inflight dedupe.
- context builder now fetches semantic + ECS summaries in parallel.
- context builder skips auxiliary semantic/ECS fetches entirely when prompt budget is already tight.
- extension now requests only:
  - `/memory/semantic?limit=64&summary_only=true`
  - `/ecs/handles?limit=128&summary_only=true`

## Code references

- API semantic shaping: `crates/focusa-api/src/routes/memory.rs`
- API ECS shaping: `crates/focusa-api/src/routes/ecs.rs`
- extension hot caches: `apps/pi-extension/src/state.ts`
- context hook summary fetches: `apps/pi-extension/src/turns.ts`

## Validation

```bash
cargo test -p focusa-api routes::memory::tests:: -- --nocapture
cargo test -p focusa-api routes::ecs::tests:: -- --nocapture
cargo check -p focusa-api -p focusa-cli
apps/pi-extension/node_modules/.bin/tsc -p apps/pi-extension/tsconfig.json
bash tests/spec81_impl_pi_extension_runtime_contract_test.sh
bash tests/spec87_impl_tool_desirability_test.sh
```

All passed.

## Remaining concern

The daemon RSS is still much higher than the desired steady-state target from Spec82, so this patch reduces extension-side latency pressure but does not fully solve daemon memory residency yet.
