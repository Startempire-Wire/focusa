# Runtime Configuration Keys

Current local Focusa runtime configuration keys used by bounded memory/payload paths.

## Pi extension orientation

| Key | Default | Meaning |
|---|---:|---|
| `FOCUSA_PI_VITAL_INFO_PROMPT_MODE` | `prompt` | Controls interruptive Pi prompts for vital project information: `prompt`, `notify`, or `off`. |

## Metacognition store caps

These keys bound the metacognition runtime store and hot index. They mirror `FocusaConfig` fields (`metacog_max_captures`, `metacog_max_reflections`, `metacog_max_adjustments`, `metacog_ttl_minutes`, `metacog_retrieve_max_k`) and are exposed by `GET /v1/metacognition/status`.

| Key | Default | Meaning |
|---|---:|---|
| `FOCUSA_METACOG_MAX_CAPTURES` | `1000` | Maximum retained capture records and capture hot-index entries. |
| `FOCUSA_METACOG_MAX_REFLECTIONS` | `500` | Maximum retained reflection records. |
| `FOCUSA_METACOG_MAX_ADJUSTMENTS` | `500` | Maximum retained adjustment records. |
| `FOCUSA_METACOG_TTL_MINUTES` | `10080` | TTL for metacognition captures/reflections/adjustments before prune/eviction. |
| `FOCUSA_METACOG_RETRIEVE_MAX_K` | `50` | Hard maximum retrieval candidates returned by `/v1/metacognition/retrieve`. |

Eviction telemetry appears in `/v1/metacognition/status` under `eviction_telemetry` and in `/v1/telemetry/memory` store/cap surfaces.

## Bounded route caps

| Key | Default | Meaning |
|---|---:|---|
| `FOCUSA_MEMORY_PRESSURE_RSS_KB` | unset | Enables explicit pressure mode when daemon RSS reaches threshold. |
| `FOCUSA_LOWMEM_RSS_SOFT_MB` | `700` | Enters LowMem posture at audit-warning RSS; hot routes stay callable while cold payloads prune/defer. |
| `FOCUSA_LOWMEM_RSS_HARD_MB` | `1000` | Enters emergency posture near audit-critical RSS; Workpoint routes return bounded/pending envelopes instead of blocking. |
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

`GET /v1/telemetry/memory` exposes current RSS/peak RSS, store counts, caps, pressure status/last transition, route budgets, and response-size histograms.
