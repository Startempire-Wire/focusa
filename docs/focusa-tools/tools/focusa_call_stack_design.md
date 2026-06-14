# `focusa_call_stack_design`

**Family:** `workpoint`
**Label:** Call Stack Design

## Purpose

Write a typed, append-only **Call Stack Design** for a feature before implementation. Returns the standard Focusa call stack scaffold (entry → handlers → services → adapters → storage → output) that the operator/agent fills in for the specific feature. Per **Spec 103 — Call Stack Architecture Blueprint**.

A call stack design is the answer to the question: "given this feature, what is the exact end-to-end call flow from operator/agent input all the way to storage and back, and how does each layer compose with the next?" It is the highest-leverage artifact an agent can be given before writing code.

## When to use

- Before implementing a new feature that an AI agent will write.
- Before a multi-layer refactor where the call surface will change.
- When handing off work to another agent or a teammate who needs the call shape upfront.
- When you want the design to be linkable as `focusa_evidence` to an active Workpoint.

Do not use for ad-hoc single-line edits; the design is overhead for trivial changes.

## Parameters

- `mission` — short description of the feature this design covers (≤ 200 chars). Required.
- `entry_surface` — `pi_tool` | `cli_command` | `http_route`. Default: `pi_tool`.
- `entry_name` — proposed tool/command/route name (≤ 120 chars). Required.
- `project_root` — project scope. Defaults to the Pi session cwd.
- `continuity_id` — optional workstream filter.
- `workpoint_id` — Workpoint to attach the design to (required when `attach_to_workpoint=true`).
- `attach_to_workpoint` — default `false`; when `true`, the design becomes `focusa_evidence` linked to the active Workpoint.
- `attach_to_stg` — default `false`; when `true`, the design sets the active STG of the active Trajectory.
- `parent_design_id` — chain a refinement onto an existing design.
- `notes` — bounded free-form notes (≤ 2KB).

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, `scope_status=matched`, `design_id`, the `CallStackDesign` envelope, the standard scaffold (handlers, services, adapters, output envelope), `next_tools`, `ledger_file`, and `evidence_refs` (empty unless `attach_to_workpoint=true`).

The returned `design.handlers`, `design.services`, `design.adapters` are the **standard Focusa call stack shape**. The operator/agent is expected to refine the per-feature details (e.g., entry parameter schema, storage path, evidence refs). The tool does not invent those.

## Example

```json
{
  "mission": "Add focusa_call_stack_design tool",
  "entry_surface": "pi_tool",
  "entry_name": "focusa_call_stack_design",
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-root-9b748c41-5185-4530-9f4f-a1f9aee49c47",
  "notes": "Spec 103 v0 implementation"
}
```

```text
focusa_call_stack_design ok | call stack design → mission="Add focusa_call_stack_design tool"
ids: design_id=019eadb2-… rehydrate_id=019eadb2-… entry_name=focusa_call_stack_design entry_surface=pi_tool
fields: mission=Add focusa_call_stack_design tool project_root=/home/wirebot/focusa attach_to_workpoint=no attach_to_stg=no ledger_file=/home/wirebot/focusa/data/.focusa/call-stack-designs/…/designs.jsonl
next: focusa_call_stack_verify → focusa_workpoint_link_evidence → focusa_trajectory_assess
```

## Scope rules

- `project_root` is **required** — designs are scoped to project.
- `mission` is **required** — bounded to 200 chars.
- `entry_surface` must be one of `pi_tool` | `cli_command` | `http_route`.
- `entry_name` is **required**, bounded to 120 chars.
- Agent runtime paths (e.g. `/root/pi-mono`, `/root/.claude`) are rejected with `failure_class=scope_mismatch`.
- File path is deterministic: `{data_dir}/call-stack-designs/{project_root_hash}/designs.jsonl`.
- Designs are append-only — old entries are never modified or deleted.
- Entries are ordered by timestamp (oldest first, most recent last).
- Per Spec 103: no singleton, scope-bounded, advisory by default, promotion requires explicit `attach_to_*` opt-in.

## Notes

- A call stack design is **not** a substitute for code review. Use `focusa_call_stack_verify` for drift detection.
- The standard scaffold (handlers: validation, scope_binding, workpoint_link; services: spec80_envelope, trajectory_assess; adapters: focusa_fetch, persistence_jsonl; output: tool_result_v1) is the **standard Focusa call stack shape**; deviate only with reason.
- The design is *advisory* — it never overrides canonical Workpoint or Trajectory envelopes.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path; pick a real project folder.
- `mission_missing` / `mission_too_long` — provide a `mission` ≤ 200 chars.
- `entry_surface_invalid` — use one of `pi_tool` | `cli_command` | `http_route`.
- `entry_name_missing` / `entry_name_too_long` — provide an `entry_name` ≤ 120 chars.
- `workpoint_unavailable` — `attach_to_workpoint=true` requires an explicit `workpoint_id`.
- `trajectory_unclear` — `attach_to_stg=true` requires an explicit `continuity_id`.
- `notes_too_long` — keep `notes` ≤ 2KB.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.
- `storage_unwritable` — inspect daemon logs and retry.

When `failure_class` is missing, treat the response as a successful design; verify with `focusa_call_stack_verify` using the returned `design_id`.

## Contract summary

- Family: `workpoint`
- Side effects: `write_call_stack_design`
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/call-stack/design`
- Core surface: `Spec103 per-project append-only Call Stack Design ledger`
- Spec: `docs/103-call-stack-architecture-blueprint-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_workpoint_link_evidence` — link the design to the active Workpoint as `focusa_evidence`.
- `focusa_trajectory_assess` — check whether the design's mission aligns with the active STG.
- `focusa_project_verify` — verify project identity before retrying on `project_root_unverified`.
- `focusa_workpoint_resume` — rehydrate the active Workpoint when a `workpoint_id` is missing.
- `focusa_call_stack_verify` — compare a design against the actual call surface and report drift.
- `focusa_traverse` — walk the `call_stack_designs` surface (planned v0.5).
