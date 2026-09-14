# Runtime Configuration Keys

Current local Focusa runtime configuration keys used by bounded memory/payload paths.

## Pi extension orientation

| Key | Default | Meaning |
|---|---:|---|
| `FOCUSA_PI_VITAL_INFO_PROMPT_MODE` | `prompt` | Controls vital project info handling: `prompt`, `warn_only`, or `off` (`notify` remains a legacy alias for `warn_only`). |
| `FOCUSA_PI_VITAL_INFO_PROMPT_SURFACES` | `project_root,project_verify,workpoint,trajectory` | Comma-list of prompt surfaces. Project identity surfaces plus Workpoint/Trajectory are default-on; all other tool surfaces are opt-in only. Confirmed roots, project_verify state, Workpoint packet, and Trajectory clarity persist in Pi session entries across reload/resume. |

## Metacognition store caps

These keys bound the metacognition runtime store and hot index. They mirror `FocusaConfig` fields (`metacog_max_captures`, `metacog_max_reflections`, `metacog_max_adjustments`, `metacog_ttl_minutes`, `metacog_retrieve_max_k`) and are exposed by `GET /v1/metacognition/status`. Evaluations use the adjustment cap/TTL family and are visible through `evaluation_memory` plus `/v1/metacognition/evaluations/recent`.

| Key | Default | Meaning |
|---|---:|---|
| `FOCUSA_METACOG_MAX_CAPTURES` | `1000` | Maximum retained capture records and capture hot-index entries. |
| `FOCUSA_METACOG_MAX_REFLECTIONS` | `500` | Maximum retained reflection records. |
| `FOCUSA_METACOG_MAX_ADJUSTMENTS` | `500` | Maximum retained adjustment records and evaluation records. |
| `FOCUSA_METACOG_TTL_MINUTES` | `10080` | TTL for metacognition captures/reflections/adjustments/evaluations before prune/eviction. |
| `FOCUSA_METACOG_RETRIEVE_MAX_K` | `50` | Hard maximum retrieval candidates returned by `/v1/metacognition/retrieve`. |

Eviction telemetry appears in `/v1/metacognition/status` under `eviction_telemetry` and in `/v1/telemetry/memory` store/cap surfaces.

## Bounded route caps

| Key | Default | Meaning |
|---|---:|---|
| `FOCUSA_MEMORY_PRESSURE_RSS_KB` | unset | Enables explicit pressure mode when daemon RSS reaches threshold. |
| `FOCUSA_LOWMEM_RSS_SOFT_MB` | `700` | Canonical soft RSS limit. Enters LowMem posture at audit-warning RSS; hot routes stay callable while cold payloads prune/defer. |
| `FOCUSA_LOWMEM_RSS_HARD_MB` | `1000` | Canonical hard RSS limit. Enters emergency posture near audit-critical RSS; Workpoint routes return bounded/pending envelopes instead of blocking. |
| `FOCUSA_MEMORY_BUDGET_MB` | unset | Deprecated compatibility alias for the hard RSS limit. Used only when `FOCUSA_LOWMEM_RSS_HARD_MB` is absent; status reports the resolved ResourceMode budget. |
| `FOCUSA_ONTOLOGY_WORLD_DEFAULT_OBJECT_LIMIT` | `256` | Default object page for `/v1/ontology/world`. |
| `FOCUSA_ONTOLOGY_WORLD_DEFAULT_LINK_LIMIT` | `512` | Default link page for `/v1/ontology/world`. |
| `FOCUSA_ONTOLOGY_WORLD_FULL_OBJECT_LIMIT` | `10000` | Hard object ceiling for explicit full ontology world reads. |
| `FOCUSA_ONTOLOGY_WORLD_FULL_LINK_LIMIT` | `20000` | Hard link ceiling for explicit full ontology world reads. |
| `FOCUSA_ONTOLOGY_WORKSPACE_SCAN_LIMIT` | `128` | Workspace discovery scan cap used by ontology world projection. |
| `FOCUSA_ECS_HANDLES_DEFAULT_LIMIT` | `100` | Default ECS handle list page. |
| `FOCUSA_ECS_HANDLES_FULL_LIMIT` | `512` | Hard ECS handle list ceiling for explicit full reads. |
| `FOCUSA_MEMORY_SEMANTIC_DEFAULT_LIMIT` | `100` | Default semantic memory page. |
| `FOCUSA_MEMORY_SEMANTIC_FULL_LIMIT` | `512` | Hard semantic memory list ceiling for explicit full reads. |
| `FOCUSA_REFERENCES_SALIENT_DEFAULT_LIMIT` | `50` | Default salient-reference page. |
| `FOCUSA_REFERENCES_SALIENT_FULL_LIMIT` | `512` | Hard salient-reference full-read ceiling. |

`GET /v1/telemetry/memory` exposes current RSS/peak RSS, store counts, caps, pressure status/last transition, route budgets, and response-size histograms. `GET /v1/status` consumes the same resolved `LowMemBudget`; its legacy `runtime_memory.memory_budget_mb` field mirrors `rss_hard_mb`, while `rss_soft_mb`, `rss_hard_mb`, and `budget_authority=resource_mode` make the canonical policy explicit. If a legacy hard limit is below the default soft limit, the resolved soft limit is clamped to that hard limit.
