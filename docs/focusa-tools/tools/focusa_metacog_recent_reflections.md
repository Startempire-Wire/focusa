# `focusa_metacog_recent_reflections`

Best safe helper for finding recent reflection ids and update sets before adjust or promote work. Use it when Best safe helper for finding recent reflection ids and update sets before adjust or promote work. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Best safe helper for finding recent reflection ids and update sets before adjust or promote work.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `limit` (optional; integer; min=1, max=20): How many recent reflections to return (default 5).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_recent_reflections`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_recent_reflections.md

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

- `focusa_metacog_plan_adjust` (likely_next)
- `focusa_metacog_doctor` (likely_next)
- `focusa_metacog_reflect` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_plan_adjust`, `focusa_metacog_doctor`, `focusa_metacog_reflect`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_recent_reflections`; MCP: `focusa.metacog.recent.reflections`; OpenAI: `focusa_metacog_recent_reflections`.
- CLI: `focusa metacognition recent-reflections`.
- REST: `GET /v1/metacognition/reflections/recent`.
- Specification: contract registry.
- Descriptor digest: `sha256:d86f43402703bfbf9758f28d6fb148a95c41b140e88af57d7f4b917fad8818d1`.
