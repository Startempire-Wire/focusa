# Focusa API Spec Compliance Audit — 2026-02-28 (Post-remediation)

Spec baseline: `docs/G1-12-api.md` (MVP API contract)
Runtime tested: `focusa-daemon` on `127.0.0.1:8787`
Probe artifact: `/tmp/focusa_api_spec_audit.json`

## Executive status

- **Service health:** ✅ running
- **Endpoint coverage:** ✅ present
- **Schema compliance:** ✅ aligned for P0/P1/P2 docs+fixtures scope
- **Error model compliance:** ✅ JSON envelope enforced for framework + route errors
- **Determinism requirement:** ✅ `turn/complete` idempotent by `turn_id`

---

## Evidence snapshot

- `GET /v1/health`: `200` with `ok`, `version`, `uptime_ms`
- `GET /v1/status`: `200` with `active_frame`, `stack_depth`, `worker_status`, `last_event_ts`, `prompt_stats`
- `POST /v1/prompt/assemble`: `200` with canonical keys (`assembled`, `stats`) and compatibility keys (`assembled_prompt`, `context_stats`)
- Invalid `POST /v1/prompt/assemble`: `422` JSON envelope with `code/message/correlation_id`
- Duplicate `POST /v1/turn/complete` for same `turn_id`: second response includes `duplicate: true`; no second `turn_completed` persistence event
- Contract probe: `scripts/api_contract_probe.py` exits `0` with `pass=true`

---

## Contract checks vs `G1-12-api.md`

### 1) Health
- **Spec:** `/v1/health` includes `ok`, `version`, `uptime_ms`
- **Runtime:** ✅ compliant

### 2) Status
- **Spec:** active frame summary, stack depth, worker status, last event timestamp, prompt stats
- **Runtime:** ✅ compliant (`active_frame`, `stack_depth`, `worker_status`, `last_event_ts`, `prompt_stats`)

### 3) Focus Stack
- **Spec:** clarified to canonical object form (`stack` as `FocusStackState` object, frames in `stack.frames[]`)
- **Runtime:** ✅ aligned

### 4) Focus Gate
- Endpoints present and routable
- Request schemas documented with exact required fields and UUID constraints
- **Runtime:** ✅ compliant for documented shape

### 5) Prompt Assembly
- **Spec:** canonical response keys `assembled`, `stats`, `handles_used`
- **Runtime:** ✅ returns canonical + compatibility keys (`assembled_prompt`, `context_stats`) during migration

### 6) Turn lifecycle
- Request schema now explicitly documented (`adapter_id`, `harness_name`, `timestamp` required for `turn/start`)
- `/v1/turn/complete` idempotency enforced by persistence dedupe
- **Runtime:** ✅ compliant

### 7) ECS/Memory request contracts
- Exact request schema documented for:
  - `/v1/ecs/store`
  - `/v1/memory/semantic/upsert`
  - `/v1/memory/procedural/reinforce`
- Fixture payloads added under `docs/fixtures/api/` and validated (`200`, no `422`)
- **Runtime:** ✅ compliant for documented payloads

### 8) Error model
- **Spec:** all errors JSON `{code,message,details?,correlation_id?}`
- **Runtime:** ✅ enforced via global middleware (including framework/body parse rejections)

### 9) Determinism requirement
- **Spec:** repeated `turn/complete` must not double-apply
- **Runtime:** ✅ enforced and covered by automated test (`turn_complete_is_idempotent_by_turn_id`)

---

## Implemented hardening items

- Global error envelope middleware + correlation ID propagation
- `/v1/turn/complete` dedupe guard backed by SQLite event check
- `/v1/health` `uptime_ms`
- `/v1/status` contract expansion
- Prompt assemble dual key compatibility
- Fixture payload pack (`docs/fixtures/api/`)
- Contract probe script (`scripts/api_contract_probe.py`) + CI wiring (`.github/workflows/ci.yml`)
- Integration test for turn dedupe

---

## Remaining work

- `focusa-d5g.10` (P2): Agent Audit UI compliance panel (operational visibility enhancement, not required for API contract correctness)

## Bottom line

Focusa API is now **spec-compliant for contracted endpoints/error model/determinism** in the audited scope.
Open item is observability UX uplift (`focusa-d5g.10`), not a contract blocker.
