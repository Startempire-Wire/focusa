# `focusa_context_cognition`

**Family:** `trajectory`
**Label:** Context Cognition

## Purpose

Build the bounded, advisory **Spec 100 `ContextCognitionPacket`** for the current project. Returns a typed packet describing the agent's current context: scope, authority, freshness, selected context (files/diffs/docs/codemaps), ontology frame, evidence frame, reasoning frame, optimization frame, and route frame. **Never mutates state.**

The packet is the single artifact an agent or operator can read to understand the agent's working context without digging into raw state.

## When to use

- Before a multi-step decision: "what does the agent know right now?"
- When an agent or operator needs a structured view of project context.
- As the first step of `focusa_trajectory_assess` or `focusa_workpoint_resume`.
- When the operator wants to verify the agent is operating in the right scope.

Do not use for high-frequency polling; the packet is built on demand.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — optional workstream filter.
- `session_id` — optional session id filter.
- `include_rehydrate_refs` — when `true`, return rehydrate refs for each surface.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, `scope_status=matched` (when project identity is verified), the full `ContextCognitionPacket` envelope, `next_tools` (default: `focusa_active_object_resolve`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`), and `rehydrate_id` (the active workpoint id when present, else `ctx_cognition:v0`).

The packet is read-only and never mutates Workpoint or Trajectory.

## Example

```json
{
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-root-9b748c41-5185-4530-9f4f-a1f9aee49c47"
}
```

```text
focusa_context_cognition ok | context cognition → focusa.context_cognition_packet.v1 scope=matched
ids: rehydrate_id=019eacb8-… workpoint_id=019eacb8-… trajectory_id=trajectory:… action_authority=workpoint
fields: schema=focusa.context_cognition_packet.v1 scope_status=matched evidence_refs=1 advisory=true canonical=false
note: advisory only; never mutates Workpoint or Trajectory
next: focusa_active_object_resolve → focusa_workpoint_checkpoint → focusa_evidence_capture
```

## Scope rules

- `project_root` is **required** — packet is scoped to project.
- Agent runtime paths (e.g. `/root/pi-mono`, `/root/.claude`) are rejected with `failure_class=scope_mismatch`.
- Packet is read-only; `canonical_mutation_allowed` is always `false`.
- The packet's `scope_status` is one of `matched` (verified), `partial` (unverified), `missing` (no identity).
- The `next_tools` are advisory only; the operator decides what to call next.

## Notes

- Per Spec 100 §6 the packet has bounded shape: `schema_version`, `status`, `advisory`, `canonical`, `scope_status`, `freshness`, `scope`, `authority`, `selected_context`, `ontology_frame`, `evidence_frame`, `reasoning_frame`, `optimization_frame`, `route_frame`, `side_effects`, `evidence_refs`, `recommended_packet_use`.
- The Context Curator (Spec 100 P3) and Cognition Optimizer (Spec 100 P5) are deferred to follow-up slices. v0 builds the envelope from existing read models (workpoint, trajectory, HLT, evidence).
- The packet is part of the 66-tool surface; it composes with `focusa_trajectory_assess`, `focusa_workpoint_resume`, and `focusa_active_object_resolve`.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path; pick a real project folder.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.
- `trajectory_not_hydrated` — call `focusa_trajectory_view` first, then retry.

When `failure_class` is missing, treat the response as a successful packet; verify with `focusa_traverse` on the `trajectory` surface.

## Contract summary

- Family: `trajectory`
- Side effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `GET /v1/context-cognition`
- CLI commands: `focusa context-cognition view`
- Core surface: `Spec100 bounded ContextCognitionPacket envelope (advisory)`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_active_object_resolve` — bind the active object for the next action.
- `focusa_workpoint_checkpoint` — record a typed checkpoint for the active work.
- `focusa_evidence_capture` — link evidence to the active Workpoint.
- `focusa_trajectory_view` — refresh the trajectory ladder before deeper work.
- `focusa_project_verify` — verify project identity on `project_root_unverified`.
- `focusa_workpoint_resume` — rehydrate the active Workpoint on discontinuity.
