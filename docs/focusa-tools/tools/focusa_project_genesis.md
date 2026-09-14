# `focusa_project_genesis`

Start, resume, inspect, or atomically commit the Project Genesis chain from verified identity and HLT through the first Workpoint. Use it when Stage, resume, inspect, or atomically commit the verified project journey from HLT and specification through tasks, first Workpoint, coordination, and readiness receipt. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Stage, resume, inspect, or atomically commit the verified project journey from HLT and specification through tasks, first Workpoint, coordination, and readiness receipt.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (optional; string | string | string | string): Genesis operation; defaults to status.
- `project_root` (optional; string): Verified absolute project root.
- `continuity_id` (optional; string): Stable project workstream continuity id.
- `idempotency_key` (optional; string): Stable transaction replay key.
- `hlt` (optional; string): Operator-confirmed High Level Trajectory.
- `hlt_confirmed` (optional; boolean): See the strict descriptor schema.
- `desired_end_state` (optional; string): See the strict descriptor schema.
- `current_state` (optional; string): See the strict descriptor schema.
- `specification_ref` (optional; string): See the strict descriptor schema.
- `acceptance_criteria` (optional; array): See the strict descriptor schema.
- `mid_level_goal` (optional; string): See the strict descriptor schema.
- `short_term_goal` (optional; string): See the strict descriptor schema.
- `waypoints` (optional; array): See the strict descriptor schema.
- `task_provider` (optional; string): See the strict descriptor schema.
- `allow_task_decomposition` (optional; boolean): See the strict descriptor schema.
- `confirm` (optional; boolean): Required true for commit or takeover.
- `takeover` (optional; boolean): Take over a conflicting active project workstream; requires confirm=true.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_genesis`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_genesis.md

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
- Authority: `{"kind":"canonical","path":"daemon:/v1/project/genesis/commit"}`
- Side effects: `start_resume_read_or_confirmed_atomic_commit`, `start_resume_read_or_confirmed_atomic_commit`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workpoint_resume` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_project_verify` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_project_verify`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_genesis`; MCP: `focusa.project.genesis`; OpenAI: `focusa_project_genesis`.
- CLI: `focusa project genesis start|resume|status|commit`.
- REST: `POST /v1/project/genesis/start`, `POST /v1/project/genesis/resume`, `GET /v1/project/genesis/status`, `POST /v1/project/genesis/commit`.
- Specification: `docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md`.
- Descriptor digest: `sha256:1296287658b1ac0d4657d7569ada92c9b9f20111517a451f1aeb9dcce5dcfb57`.
