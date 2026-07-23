# `focusa_metacog_reflect`

Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes. Use it when Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `turn_range` (required; string): Turn range expression.
- `failure_classes` (optional; array): Failure class tags.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_reflect`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "turn_range": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_reflect.md

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_state`, `write_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_plan_adjust` (likely_next)
- `focusa_metacog_capture` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_plan_adjust`, `focusa_metacog_capture`, `focusa_workpoint_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_reflect`; MCP: `focusa.metacog.reflect`; OpenAI: `focusa_metacog_reflect`.
- CLI: `focusa metacognition reflect`.
- REST: `POST /v1/metacognition/reflect`.
- Specification: contract registry.
- Descriptor digest: `sha256:673cd7e1b75f9dabd16bf4022f4aea6de1f72b1d67198e010edb9744bd211716`.
