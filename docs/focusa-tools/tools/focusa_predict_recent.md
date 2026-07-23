# `focusa_predict_recent`

List recent predictions from one typed project/workstream scope. Use it when List recent bounded Focusa prediction records. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- List recent bounded Focusa prediction records.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `limit` (optional; number): Recent prediction count, max 100.
- `project_root` (optional; string): Explicit or current verified project root.
- `continuity_id` (optional; string): Explicit or current workstream continuity id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_predict_recent`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_predict_recent.md

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_predict_stats` (likely_next)
- `focusa_predict_evaluate` (likely_next)
- `focusa_metacog_retrieve` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_predict_stats`, `focusa_predict_evaluate`, `focusa_metacog_retrieve`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_predict_recent`; MCP: `focusa.predict.recent`; OpenAI: `focusa_predict_recent`.
- CLI: `focusa predict recent`.
- REST: `GET /v1/predictions/recent`.
- Specification: contract registry.
- Descriptor digest: `sha256:310a5510549063230d3787eb4b8ea5b9dc3a12d52ee90e1b34df6c638a5fc7a1`.
