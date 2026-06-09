# `focusa_context_cognition_render`

**Family:** `trajectory`
**Label:** Context Cognition Render

## Purpose

Render the **Spec 100 `ContextCognitionPacket`** as compact text (markdown-flavored). Returns bounded lines + the packet's `workpoint_id`, `trajectory_id`, and `rehydrate_id`. Advisory only; never mutates state.

Use when an operator or agent wants a human-readable view of the context cognition packet without parsing JSON — e.g., for a prompt section, a CLI summary, or the macOS menubar peek.

## When to use

- Before composing a prompt section from a packet.
- When the operator wants a copy-paste-ready summary.
- When the menubar or CLI surfaces need a stable, compact text format.

Do not use for high-frequency polling; render is built on demand.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — optional workstream filter.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, `format=compact_text`, the bounded `render` text, `render_lines` count, `workpoint_id`, `trajectory_id`, and `rehydrate_id`.

The render is a fixed-shape bounded output (≤ ~1KB text) suitable for direct embedding in a prompt, a CLI transcript, or a menubar card.

## Example

```json
{
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-root-9b748c41-5185-4530-9f4f-a1f9aee49c47"
}
```

```text
focusa_context_cognition_render ok | context cognition render → 8 lines
ids: rehydrate_id=019eacb8-… workpoint_id=019eacb8-…
fields: render_lines=8 format=compact_text advisory=true
next: focusa_context_cognition → focusa_context_cognition_proof

## Context Cognition (Spec 100) — render for /home/wirebot/focusa
advisory · read-only · canonical=false
schema: focusa.context_cognition_packet.v1
workpoint_id: 019eacb8-c8be-7f63-ae40-a16da6600110
trajectory_id: trajectory:project-fnv1a64:8aab637a4a87e459:defined-goal
authority: workpoint (canonical_mutation_allowed=false)
next_tools: focusa_active_object_resolve, focusa_workpoint_checkpoint, focusa_evidence_capture
do_not_drift: transcript_tail as authority; cross-project scope fallbacks
```

## Scope rules

- `project_root` is **required** — render is scoped to project.
- Agent runtime paths (e.g. `/root/pi-mono`, `/root/.claude`) are rejected with `failure_class=scope_mismatch`.
- Render is read-only; no Workpoint, Trajectory, or HLT mutation occurs.

## Notes

- Per Spec 100 §11 the CLI contract prints the compact packet, proof commands, and diff/context summaries.
- Per Spec 100 §13 the Focusa tool wrappers add callable `focusa_context_cognition_*` outputs.
- The render is a stable shape: the same scope produces the same lines (modulo timestamps).
- The render does not include the full packet JSON; use `focusa_context_cognition` for the JSON.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path; pick a real project folder.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

When `failure_class` is missing, treat the response as a successful render; verify with `focusa_context_cognition` for the JSON.

## Contract summary

- Family: `trajectory`
- Side effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `GET /v1/context-cognition/render`
- CLI commands: `focusa context-cognition render`
- Core surface: `Spec100 §11 CLI contract + §13 tool wrapper`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_context_cognition` — the full packet JSON (when JSON is needed).
- `focusa_context_cognition_proof` — map the packet to proof commands.
- `focusa_project_verify` — verify project identity on `project_root_unverified`.
- `focusa_workpoint_resume` — rehydrate the active Workpoint on discontinuity.
