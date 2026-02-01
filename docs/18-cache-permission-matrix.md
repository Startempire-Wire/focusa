# docs/18-cache-permission-matrix.md — Cache Permission Matrix (AUTHORITATIVE)

This document defines:
1) **What Focusa is allowed to cache**
2) **Where caching is allowed**
3) **How caches are keyed**
4) **When caches MUST be invalidated (“intentional cache busting”)**

Focusa treats caching as a **performance optimization subordinate to cognition**.
Caching is never permitted to distort salience, freshness, authority, or provenance.

---

## 0. Canonical Principle

> **Cache structure and evidence — never conclusions.**  
> Caching must never become a cognitive constraint.

---

## 1. Cache Classes (Standardized)

All caches in Focusa MUST be classified as one of:

### C0 — Immutable Content Cache (Safe)
- Content-addressed blobs (hash)
- Never invalidated (only superseded)
- Examples: stored tool outputs, file snapshots, summaries

### C1 — Deterministic Assembly Cache (Conditionally Safe)
- Deterministic outputs from deterministic inputs
- Must be keyed by all inputs
- Examples: prompt assembly, compiled context packs

### C2 — Ephemeral Compute Cache (Volatile)
- Derived ranking/scoring outputs
- Tied to a specific Focus State revision
- Examples: Focus Gate score tables, retrieval rankings

### C3 — Provider KV/Prompt Cache (Opportunistic)
- External / black-box caching (KV tensors)
- Must never shape cognition
- Useful only for stable scaffolding prefixes

### C4 — Forbidden Cache (Disallowed)
- Anything that can silently drift truth/intent
- Examples: model completions as authoritative outputs

---

## 2. Cache Permission Matrix

Legend:
- ✅ Allowed (recommended)
- ⚠️ Allowed with strict constraints
- ❌ Disallowed
- ⛔ Forbidden (violates Focusa principles)

| Focusa Component | C0 Immutable | C1 Assembly | C2 Ephemeral | C3 Provider KV | C4 Forbidden |
|---|---:|---:|---:|---:|---:|
| Reference Store | ✅ (primary) | ❌ | ❌ | ❌ | ⛔ |
| CLT (Context Lineage Tree) | ✅ (nodes immutable) | ❌ | ❌ | ❌ | ⛔ |
| Focus State (canonical) | ✅ (snapshots/versioned) | ❌ | ❌ | ❌ | ⛔ |
| Focus Gate | ❌ | ⚠️ (deterministic thresholds only) | ✅ | ❌ | ⛔ |
| Expression Engine | ❌ | ⚠️ (prompt assembly only) | ✅ | ⚠️ (prefix only) | ⛔ |
| Intuition Engine | ❌ | ❌ | ✅ (signals only) | ❌ | ⛔ |
| Retrieval / Reference Resolution | ✅ (artifacts) | ⚠️ (context packs) | ✅ (rankings) | ❌ | ⛔ |
| Autonomy Scoring (ARI) | ✅ (ledger) | ⚠️ (report generation) | ✅ | ❌ | ⛔ |
| UXP / UFI | ✅ (append-only facts) | ❌ | ✅ (trend windows) | ❌ | ⛔ |
| CS (Constitution Synthesizer) | ✅ (drafts/evidence) | ⚠️ (diff render) | ✅ (analysis windows) | ❌ | ⛔ |
| Provider Response Output | ❌ | ❌ | ❌ | ❌ | ⛔ |

---

## 3. Cache Key Requirements (Deterministic by Construction)

All caches must use a typed cache key. At minimum:

### 3.1 Mandatory Key Fields
- `agent_id`
- `constitution_version`
- `model_id`
- `harness_id`
- `focus_state_revision` (or hash)
- `token_budget`
- `retrieval_policy_version`

### 3.2 Optional (When Relevant)
- `task_authority_id` (e.g., beads task/epic)
- `repo_fingerprint` (git HEAD or content hash)
- `tool_schema_hash`
- `system_time_bucket` (if time-sensitive behavior is allowed)

### 3.3 Example Cache Key (C1 Prompt Assembly)
```json
{
  "cache_class": "C1",
  "kind": "prompt_assembly",
  "agent_id": "focusa-default",
  "constitution_version": "1.1.0",
  "model_id": "claude-3.5",
  "harness_id": "claude-code",
  "focus_state_hash": "sha256:...",
  "token_budget": 32000,
  "tool_schema_hash": "sha256:...",
  "retrieval_policy_version": 1
}
```

If any required key field is missing → caching is disallowed.

---

## 4. What We Cache Aggressively vs Opportunistically

### 4.1 Cache Aggressively (Preferred)
- C0 Reference Store artifacts
- CLT nodes
- Structured summaries (compaction products)
- Deterministic serialized tool outputs
- Autonomy ledgers, UFI event logs

### 4.2 Cache Opportunistically
- Prompt assembly (C1)
- Context pack compilation (C1)
- Retrieval ranking tables (C2)
- Focus Gate intermediate scoring (C2)

### 4.3 Never Cache
- model completions as reusable truth
- “best answer” snapshots across different Focus State revisions
- inferred intent or inferred emotions

---

## 5. Provider KV/Prompt Cache Policy (C3)

Provider caching is allowed ONLY for:
- static constitution text
- static tool schema blocks
- stable system scaffolding

Provider caching is NOT allowed to influence:
- compaction strategy
- prompt freshness
- inclusion of new evidence

If caching would bias decisions (e.g., avoid adding new info to preserve prefix):
→ **Intentional cache busting MUST occur**.

---

## 6. Hard Invalidation Rules

The following events MUST invalidate all C1/C2 caches:

- Agent ID changed
- Constitution version changed
- Model or harness changed
- Focus State revision changed
- Focus Stack push/pop changed
- Focus Gate threshold/policy changed
- Token budget changed
- Tool schemas changed
- Reference Store new high-priority artifact added
- Task authority changed (Beads: task/epic switched)

C0 caches are immutable and never invalidated.

---

## 7. Observability Requirements

Every cached lookup MUST emit:
- hit/miss
- cache class
- key hash
- invalidation reason (if miss)
- compute time saved estimate (optional)

Caching is only acceptable if observable.

---

## 8. Canonical Rule

> **If caching and cognition disagree, cognition wins.**
