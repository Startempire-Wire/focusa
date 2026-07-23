# `focusa_state_hygiene_plan`

Create a proposal-style hygiene plan; does not mutate Focus State. Use it when Create a proposal-style hygiene plan; does not mutate Focus State. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Create a proposal-style hygiene plan; does not mutate Focus State.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `reason` (optional; string): Why hygiene is being considered.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_state_hygiene_plan`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_state_hygiene_plan.md

## Anti-examples

- hiding failures behind null/unknown
- silent deletion or cleanup

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

- `focusa_state_hygiene_apply` (likely_next)
- `focusa_state_hygiene_doctor` (likely_next)
- `focusa_tool_doctor` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_state_hygiene_apply`, `focusa_state_hygiene_doctor`, `focusa_tool_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_state_hygiene_plan`; MCP: `focusa.state.hygiene.plan`; OpenAI: `focusa_state_hygiene_plan`.
- CLI: none.
- REST: Pi-local only.
- Specification: contract registry.
- Descriptor digest: `sha256:b569aa92de713270233891c2371bdf2a219d08867d5052f0799623278aa3f88d`.
