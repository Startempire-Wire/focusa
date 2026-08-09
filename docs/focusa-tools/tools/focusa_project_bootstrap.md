# `focusa_project_bootstrap`

Preview, apply, inspect, or repair the idempotent local project-discipline baseline before Project Genesis. Use it when Preview, apply, inspect, or repair an idempotent local project-discipline baseline with explicit Git/task choices, receipts, rollback, and Project Genesis handoff. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Preview, apply, inspect, or repair an idempotent local project-discipline baseline with explicit Git/task choices, receipts, rollback, and Project Genesis handoff.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (optional; string | string | string | string): Bootstrap operation; defaults to status.
- `project_root` (required; string): Explicit safe absolute project root.
- `project_id` (optional; string): See the strict descriptor schema.
- `canonical_name` (optional; string): See the strict descriptor schema.
- `continuity_id` (optional; string): See the strict descriptor schema.
- `idempotency_key` (optional; string): See the strict descriptor schema.
- `discipline_profile` (optional; string): Defaults to standard_software_project.
- `initialize_git` (optional; boolean): See the strict descriptor schema.
- `initialize_task_provider` (optional; boolean): See the strict descriptor schema.
- `task_provider` (optional; string): See the strict descriptor schema.
- `hlt` (optional; string): See the strict descriptor schema.
- `hlt_confirmed` (optional; boolean): See the strict descriptor schema.
- `desired_end_state` (optional; string): See the strict descriptor schema.
- `current_state` (optional; string): See the strict descriptor schema.
- `specification_ref` (optional; string): See the strict descriptor schema.
- `acceptance_criteria` (optional; array): See the strict descriptor schema.
- `confirm` (optional; boolean): See the strict descriptor schema.
- `repair_action` (optional; string): retry or rollback; rollback requires confirm=true.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_bootstrap`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "project_root": "/tmp/focusa-project"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_bootstrap.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- assuming unsafe broad cwd is canonical
- skipping verify after scope mismatch

## Authority, permissions, and side effects

- Scope: `{"kind":"write","route_family":"explicit_project_continuity"}`
- Authority: `{"kind":"canonical","path":"daemon:/v1/project/bootstrap/apply"}`
- Side effects: `preview_read_or_confirmed_local_bootstrap_repair`, `preview_read_or_confirmed_local_bootstrap_repair`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_project_genesis` (likely_next)
- `focusa_project_verify` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_project_genesis`, `focusa_project_verify`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_bootstrap`; MCP: `focusa.project.bootstrap`; OpenAI: `focusa_project_bootstrap`.
- CLI: `focusa project bootstrap preview|apply|status|repair`.
- REST: `POST /v1/project/bootstrap/preview`, `POST /v1/project/bootstrap/apply`, `GET /v1/project/bootstrap/status`, `POST /v1/project/bootstrap/repair`.
- Specification: `docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md`.
- Descriptor digest: `sha256:2c84b8714a3ecc0b1d97c9fcbe0acb9ce36865ff6cc44c6d4f8fc76ce13d49b1`.
