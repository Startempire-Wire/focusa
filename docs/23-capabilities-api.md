# docs/23-capabilities-api.md — Focusa Capabilities API (AUTHORITATIVE)

This document specifies the **Focusa Capabilities API**: a stable, exhaustive,
introspection-first interface that exposes Focusa’s full internal state and
safe command surfaces to:
- CLIs
- GUIs
- agents
- integrations
- plugins (capabilities)

Focusa’s platform promise depends on this API.

---

## 0. Canonical Principles

1. **Everything observable**: all core data points are readable (subject to policy).
2. **Authority is centralized**: most state is not directly mutable.
3. **Writes are commands**: mutations occur only via validated command endpoints.
4. **Deterministic & auditable**: every write has a reason, provenance, and log entry.
5. **Local-first**: API is served locally by `focusa-core` (daemon), default bind to localhost.
6. **Performance-safe**: streaming + pagination; no expensive scans by default.
7. **Policy-enforced**: access is filtered by capability permissions.

---

## 1. API Transport & Protocol

### 1.1 Transport
MVP: Local HTTP server (loopback only)
- Base URL: `http://127.0.0.1:<port>/v1`
- Default port: configurable (e.g., 4777)

Future: Optional gRPC / Unix socket, but HTTP is canonical for MVP.

### 1.2 Data Formats
- Request/Response: JSON
- Streaming: Server-Sent Events (SSE) OR WebSocket (choose one; SSE recommended for MVP simplicity)
- Errors: structured JSON (see Section 10)

### 1.3 Versioning
- API version in path: `/v1/...`
- Backward-compatible changes only within a major version
- Breaking changes require `/v2`

---

## 2. Authentication & Access Control (Local-First)

### 2.1 Local Token
The daemon issues a local token stored in user config.
Clients present it via:
- `Authorization: Bearer <token>`

### 2.2 Capability Permissions
Every token is bound to a permission set:
- `read:*` wide introspection
- `write:commands` limited command submission
- `admin:*` reserved for local owner UI/CLI

Default:
- CLI (local owner) receives broad permissions
- External clients (agents) can be sandboxed to read-only scopes

---

## 3. Top-Level Resource Domains (Namespaces)

The API is organized by **Capability Domains**:

- `state` (Focus State)
- `lineage` (CLT)
- `references` (Reference Store)
- `gate` (Focus Gate)
- `intuition` (Intuition Engine signals)
- `constitution` (ACP + CS drafts)
- `autonomy` (ARI and autonomy ledger)
- `metrics` (UXP/UFI + telemetry)
- `cache` (cache stats & policy)
- `contribute` (data contribution queue)
- `export` (dataset exports)
- `agents` (agent registry)
- `events` (stream of state changes and logs)

---

## 4. Read Model vs Write Model

### 4.1 Read Model
All read endpoints are safe and side-effect free.

### 4.2 Write Model (Commands)
Writes occur only through the `commands` namespace:

- `/v1/commands/submit`
- `/v1/commands/status/{command_id}`
- `/v1/commands/log/{command_id}`

Commands are validated against:
- policy constraints
- current autonomy level
- authority boundaries
- task system constraints

---

## 5. Canonical Identifiers

All major objects use stable IDs:

- `agent_id`
- `session_id`
- `focus_state_id`
- `focus_state_revision`
- `clt_node_id`
- `ref_id` (Reference Store handle)
- `constitution_version`
- `command_id`

IDs must be URL-safe strings.

---

## 6. Endpoint Specification (MVP)

This section lists **required MVP endpoints**.

### 6.1 Health & Info

- `GET /v1/health`
  - returns: `{ "status": "ok", "version": "x.y.z" }`

- `GET /v1/info`
  - returns build, config, and feature flags

---

## 7. Domain: Agents

### 7.1 List Agents
- `GET /v1/agents`
- query params:
  - `active=true|false`
  - `limit`, `cursor`

### 7.2 Get Agent
- `GET /v1/agents/{agent_id}`

### 7.3 Agent Constitution Summary
- `GET /v1/agents/{agent_id}/constitution`

### 7.4 Agent Capabilities (Effective Permissions)
- `GET /v1/agents/{agent_id}/capabilities`
  - returns what domains are readable/writable for this token

---

## 8. Domain: Focus State (`state`)

### 8.1 Current Focus State
- `GET /v1/state/current`
  - returns:
```json
{
  "focus_state_id": "fs_...",
  "revision": 42,
  "agent_id": "focusa-default",
  "intent": "...",
  "constraints": ["..."],
  "active_frame": "...",
  "lineage_head": "clt_000124",
  "salient_refs": ["ref://..."],
  "confidence": 0.82,
  "timestamp": "iso8601"
}
```

### 8.2 Focus State History
- `GET /v1/state/history`
- query params:
  - `limit`, `cursor`
  - `since`, `until`

### 8.3 Focus Stack
- `GET /v1/state/stack`

### 8.4 Focus State Diff
- `GET /v1/state/diff?from=<rev>&to=<rev>`

---

## 9. Domain: CLT (`lineage`)

### 9.1 Get CLT Head
- `GET /v1/lineage/head?session_id=<id>`

### 9.2 Get Node
- `GET /v1/lineage/node/{clt_node_id}`

### 9.3 Get Path (Head→Root)
- `GET /v1/lineage/path/{clt_node_id}`
- query params:
  - `max_depth`

### 9.4 List Children (Branch Exploration)
- `GET /v1/lineage/children/{clt_node_id}`

### 9.5 Summary Nodes
- `GET /v1/lineage/summaries?session_id=<id>`

---

## 10. Domain: Reference Store (`references`)

### 10.1 List References
- `GET /v1/references`
- query params:
  - `type`
  - `tag`
  - `limit`, `cursor`

### 10.2 Reference Metadata
- `GET /v1/references/{ref_id}/meta`

### 10.3 Retrieve Reference Content
- `GET /v1/references/{ref_id}`
  - large responses MUST support range/chunking:
    - `?offset=&length=`

### 10.4 Search References
- `POST /v1/references/search`
```json
{ "query": "...", "limit": 25 }
```

---

## 11. Domain: Focus Gate (`gate`)

### 11.1 Current Gate Policy
- `GET /v1/gate/policy`

### 11.2 Gate Scores (Last Turn)
- `GET /v1/gate/scores?focus_state_revision=<rev>`

### 11.3 Candidate Explanations
- `GET /v1/gate/explain?candidate_id=<id>`

Read-only in MVP (configuration may be via config files, not API).

---

## 12. Domain: Intuition (`intuition`)

### 12.1 Signals (Recent)
- `GET /v1/intuition/signals`
- query params: `limit`, `cursor`, `since`

### 12.2 Patterns (Derived)
- `GET /v1/intuition/patterns`

### 12.3 Submit Advisory Signal (Restricted)
- `POST /v1/intuition/submit`
```json
{ "kind": "pattern", "payload": {...}, "confidence": 0.7 }
```
Must be tagged as `advisory` and must not mutate Focus State.

---

## 13. Domain: Constitution (`constitution`)

### 13.1 Active Constitution Text
- `GET /v1/constitution/active?agent_id=<id>`

### 13.2 Constitution Versions
- `GET /v1/constitution/versions?agent_id=<id>`

### 13.3 Diff Two Versions
- `GET /v1/constitution/diff?agent_id=<id>&from=1.1.0&to=1.2.0`

### 13.4 CS Drafts (Proposed)
- `GET /v1/constitution/drafts?agent_id=<id>`

---

## 14. Domain: Autonomy (`autonomy`)

### 14.1 Current Autonomy Status
- `GET /v1/autonomy/status?agent_id=<id>`

### 14.2 ARI Ledger
- `GET /v1/autonomy/ledger?agent_id=<id>&limit=...`

### 14.3 Explain Autonomy Decision
- `GET /v1/autonomy/explain?event_id=<id>`

---

## 15. Domain: Metrics (`metrics`)

### 15.1 UXP
- `GET /v1/metrics/uxp?agent_id=<id>&window=30d`

### 15.2 UFI
- `GET /v1/metrics/ufi?agent_id=<id>&window=30d`

### 15.3 Session Metrics
- `GET /v1/metrics/session/{session_id}`

### 15.4 System Performance
- `GET /v1/metrics/perf`

---

## 16. Domain: Cache (`cache`)

### 16.1 Cache Status
- `GET /v1/cache/status`

### 16.2 Cache Policy
- `GET /v1/cache/policy`

### 16.3 Cache Events (Hit/Miss/Bust)
- `GET /v1/cache/events?limit=...`

---

## 17. Domain: Contribution (`contribute`)

### 17.1 Contribution Status
- `GET /v1/contribute/status`

### 17.2 Policy (Read/Update via commands)
- `GET /v1/contribute/policy`

### 17.3 Queue
- `GET /v1/contribute/queue?status=pending|approved|...`

---

## 18. Domain: Export (`export`)

Exports are initiated via **commands** (write surface).
Read endpoints provide history and manifests.

### 18.1 Export History
- `GET /v1/export/history`

### 18.2 Export Manifest
- `GET /v1/export/manifest/{export_id}`

---

## 19. Domain: Events (Streaming)

### 19.1 Server-Sent Events Stream
- `GET /v1/events/stream`
- event types include:
  - `focus_state.updated`
  - `lineage.node_added`
  - `reference.added`
  - `cache.bust`
  - `autonomy.event`
  - `constitution.draft_created`
  - `export.completed`
  - `contribute.queue_updated`

SSE payload shape:
```json
{
  "event": "focus_state.updated",
  "timestamp": "iso8601",
  "data": { ... }
}
```

---

## 20. Commands API (Write Surface)

### 20.1 Submit Command
- `POST /v1/commands/submit`

Canonical command envelope:
```json
{
  "command_type": "string",
  "agent_id": "focusa-default",
  "session_id": "session_42",
  "reason": "human readable",
  "payload": { }
}
```

### 20.2 Command Types (MVP)
- `contribute.set_policy`
- `contribute.pause`
- `contribute.resume`
- `contribute.queue_approve`
- `contribute.queue_reject`
- `export.start`
- `constitution.create_draft`
- `constitution.activate_version`
- `constitution.rollback`

All commands MUST be validated by policy and logged.

### 20.3 Get Command Status
- `GET /v1/commands/status/{command_id}`

### 20.4 Get Command Log
- `GET /v1/commands/log/{command_id}`

---

## 21. Error Model

All errors follow:
```json
{
  "error": {
    "code": "string",
    "message": "string",
    "details": { },
    "hint": "string | null"
  }
}
```

Common codes:
- `unauthorized`
- `forbidden`
- `not_found`
- `invalid_request`
- `policy_violation`
- `conflict`
- `rate_limited`
- `internal_error`

---

## 22. Performance Requirements

- Default list endpoints MUST be paginated
- Large blobs MUST support chunking
- Streaming MUST be optional
- No endpoint should require full-tree traversal by default

Target: **imperceptible** overhead on local machine.

---

## 23. Compatibility & Future Evolution

This API is designed to:
- support multiple harness adapters (Claude Code, Codex, Gemini CLI)
- allow external agents to introspect state safely
- enable rich GUIs and dashboards
- maintain stable semantics as Focusa evolves

---

## 24. Canonical Rule

> **The Capabilities API exposes everything you need to understand Focusa —  
> but only explicit, audited commands may change it.**
