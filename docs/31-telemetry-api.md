# docs/31-telemetry-api.md — Telemetry Capabilities API (AUTHORITATIVE)

This document defines all API endpoints for querying telemetry.

---

## 1. Capability Domain

```
telemetry.*
```

Read-only by default.

---

## 2. Event Query Endpoint

### GET `/v1/telemetry/events`

Query parameters:
- `type`
- `session_id`
- `agent_id`
- `model_id`
- `since`
- `until`
- `limit`
- `cursor`

Returns:
```json
{
  "events": [ ... ],
  "next_cursor": "optional"
}
```

---

## 3. Token Metrics Endpoint

### GET `/v1/telemetry/tokens`

Parameters:
- `group_by=model|session|agent`
- `window=7d|30d`

Returns aggregated metrics.

---

## 4. Cognitive Process Metrics

### GET `/v1/telemetry/process`

Returns:
- avg focus depth
- abandonment rate
- gate acceptance rate
- summarization frequency

---

## 5. Productivity Metrics

### GET `/v1/telemetry/productivity`

Returns:
- completion ratio
- correction loops
- rework ratio
- time-to-resolution

---

## 6. Autonomy Metrics

### GET `/v1/telemetry/autonomy`

Returns:
- autonomy timeline
- earned score
- reversions

---

## 7. Export Endpoint

### POST `/v1/telemetry/export`

Body:
```json
{
  "format": "jsonl | parquet",
  "purpose": "sft | research",
  "filters": { }
}
```

Returns export job ID.

---

## 8. Permissions

Required scopes:
- `telemetry:read`
- `export:start` (for exports)

---

## 9. Eval Ledger Boundary

Benchmark/eval harnesses may write append-only eval events, but only through the separate Eval Ledger surface:

```http
POST /v1/evals/runs
POST /v1/evals/runs/{run_id}/events
POST /v1/evals/runs/{run_id}/complete
GET  /v1/evals/runs/{run_id}
GET  /v1/evals/compare?baseline=<run_id>&candidate=<run_id>
```

Rules:
- `/v1/telemetry/*` remains read/export only.
- `/v1/evals/*` is append-only, idempotent, eval-mode scoped, and non-cognitive.
- CTL may observe/index eval events for metrics, but eval writes do not mutate prompts, gates, Workpoints, Trajectory, Focus State, or ontology.
- Eval exports must redact secrets/PII and include run metadata: `suite_id`, `run_id`, `task_id`, `scenario_id`, `arm`, `model_provider`, `model_id`, `model_version`, `model_class`, `environment_id`, `prompt_hash`, `pricing_snapshot`, `schema_version`.

## 10. Canonical Rule

> **Telemetry is queryable, never mutable; eval harnesses write only to the append-only `/v1/evals/*` ledger.**
