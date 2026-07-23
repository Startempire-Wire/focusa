# `focusa_metacog_retrieve`

Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection. Use it when Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `current_ask` (required; string): Current ask.
- `scope_tags` (optional; array): Optional scope tags.
- `k` (optional; integer; min=1, max=50): Top-k candidates (default 5).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_retrieve`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "current_ask": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_retrieve.md

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_only`, `read_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_capture` (likely_next)
- `focusa_metacog_reflect` (likely_next)
- `focusa_predict_record` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_predict_record`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_retrieve`; MCP: `focusa.metacog.retrieve`; OpenAI: `focusa_metacog_retrieve`.
- CLI: `focusa metacognition retrieve`.
- REST: `POST /v1/metacognition/retrieve`.
- Specification: contract registry.
- Descriptor digest: `sha256:c0eea74663669c78ae8bdb5e0e492be6eccd5aeabb8dd8ab7b9babed8dce974d`.
