# `focusa_work_loop_checkpoint`

Create a manual continuous-loop checkpoint. Use it when Create a manual continuous-loop checkpoint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Create a manual continuous-loop checkpoint.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `summary` (optional; string): Checkpoint summary (max 240 chars).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_work_loop_checkpoint`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_work_loop_checkpoint.md

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `checkpoint`, `checkpoint`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_work_loop_select_next` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_work_loop_select_next`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_work_loop_checkpoint`; MCP: `focusa.work.loop.checkpoint`; OpenAI: `focusa_work_loop_checkpoint`.
- CLI: none.
- REST: `POST /v1/work-loop/checkpoint`.
- Specification: contract registry.
- Descriptor digest: `sha256:c3e146ea5f4abb7193aabe79759354b6126028f51fa28426eeb1495660e31160`.
