# Focusa Tool Reliability Audit

**Purpose:** keep official Focusa tools reliable, model-usable, and minimally dependent on `focusa_scratch` fallback.

## Current safe audit command

```bash
node scripts/audit-focusa-tool-suite-safe.mjs
```

This audit is read-only for daemon/API probes. It validates all 58 registered tool contracts and docs, probes safe GET routes, and classifies warnings/failures with `failure_class`.

## Latest safe audit result

- Static contracts: passed (`tools=58`, `contracts=58`).
- Safe GET routes: passed for health, ontology tool contracts, focus current frame, lineage, metacog recent, predictions, project identity, resource mode, trajectory, work-loop summary, Workpoint current.
- Warnings:
  - `stale_runtime_registry`: running daemon serves old registry payload after source/docs edits; requires rebuild/restart to clear.

## Confirmed failurepoints from this session

| Failurepoint | Evidence | Repair path |
|---|---|---|
| daemon OOM/restart | kernel log: `OOM killed process ... focusa-daemon ... anon-rss≈1072644kB`; systemd restarted daemon | add memory caps, bounded hot routes, cached last-known-good fallbacks, and memory-pressure diagnostics |
| full/cold routes can hang | `/v1/telemetry/memory` timed out under pressure | keep hot readiness separate from telemetry/deep diagnostics; make cold routes bounded/degraded |
| stale frame writes | daemon reducer logged `FocusStateUpdated for <old frame> but active is <new frame>` | refresh Pi frame identity before Focus State writes; recover stale frame and retry once |
| focus current-frame null | `/v1/focus/frame/current` returned active id but no frame | source patch makes unscoped route fall back to active frame |
| result-adjacent slot max mismatch | API rejects `recent_results`, `notes`, `open_questions` over 180 chars while Pi tool advertised 200/300 | source/docs now align public tool limits to 180 chars |
| stale runtime registry | live proof `payload_equal=false` while static/safe fixtures pass | classify as `stale_runtime_registry`; static validation is source until approved daemon reload |
| null/unknown tool failures | older wrappers hide upstream status/body as null/unavailable | preserve raw status/body in `tool_result_v1.raw`; classify `null_response`/retry posture |
| Workpoint status projection mismatch | REST `/v1/workpoint/current` returned envelope `status=completed` while nested canonical workpoint was `status=active`; wrapper evidence/checkpoint tools blocked | classify as `read_model_lag`; wrappers should use nested canonical object state and avoid blocking solely on envelope status |

## Reliability requirements

- Tool results must expose `failure_class`, retry posture, `canonical/degraded`, side effects, evidence refs, `next_tools`, and raw status/body when safe.
- Read-only hot tools must return bounded data or degraded cached data, not block on cold diagnostics.
- Mutating tools must retry only when idempotent or after checking side effects.
- `focusa_scratch` is fallback for working notes and degraded write recovery, not the normal path for durable state.
- Stale/cross-project/frame-mismatched data must be advertised in `do_not_use`.

## Open repair items for decomposition

1. Implement and release source patches for active-frame fallback, frame refresh before writes, and result-adjacent slot limit alignment.
2. Add memory-pressure guardrails for Focus Stack/path growth, telemetry stores, Workpoint records, CLT, and ontology payloads.
3. Add last-known-good caches for hot tool reads when daemon is restarting or under pressure.
4. Add `resource_exhausted` and `null_response` across docs, schema, and all wrappers.
5. Keep golden tool-choice tasks current; `tests/spec96_tool_affordance_catalog_golden_eval_test.sh` now proves catalog-driven tool choice without source-code inspection.

## Low-memory reliability caveats

Focusa operating principle: **low memory = still reliable; high memory = opportunistic and performant without being a hog.**

- Rich context is opportunistic; core tool availability is mandatory.
- Hot tools must return bounded live data or cached degraded data instead of blocking on cold stores.
- Low-memory mode should preserve health, project identity, Trajectory summary, Workpoint summary, Focus State compact writes, evidence summary, Tool Doctor summary, work-loop summary, and tool contract reads.
- Cold routes such as telemetry memory, deep replay, full lineage, full ontology, and release proof should degrade before risking OOM.
- If daemon RSS/store pressure is high, tools should return `resource_exhausted` with recovery guidance rather than bare `null`/`unknown`.
- `focusa_scratch` is last-resort fallback; preferred recovery is cached project-bound Focusa summaries plus clear failure classes.


## Latest all-tools implementation/spec audit

- Contracts: passed (`tools=58`, `contracts=58`).
- Static implementation/spec audit: passed (`failures=0`, `warnings=0`).
- Filled gaps: concrete Focus State CLI update, scoped Workpoint CLI flags, metacog recent CLI, lineage extract CLI, snapshot API/CLI contract parity.
- Live probes: 15 hot/safe endpoints passed with no failures in the safe suite; stale runtime registry can appear until daemon rebuild/restart reloads embedded registry.
- Safe audit skips cold `GET /v1/lineage/tree` by default for low-memory reliability; set `FOCUSA_AUDIT_INCLUDE_COLD_GET=1` for explicit cold-route probing.
- Evidence: `/tmp/focusa-tool-implementation-spec-audit.json`, `/tmp/focusa-tool-suite-safe-audit-final.json`, `/tmp/focusa-cli-parity-smoke.log`, `/tmp/focusa-tool-contracts-live-smoke.json`, `/tmp/focusa-tool-stress-smoke.log`.

