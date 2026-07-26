# `focusa_project_genesis`

Stage, resume, inspect, or atomically commit the verified project journey from HLT and specification through tasks, first Workpoint, coordination, and readiness.

## When to use

- After `focusa_project_verify` when a project has no complete chain through its first Workpoint.
- To resume an interrupted Genesis journal without duplicating Trajectory, tasks, or Workpoints.
- To enter explicit HLT Impasse and ask at most one concise operator intent question.

## Parameters

- `action`: `start`, `resume`, `status`, or `commit`.
- `project_root`: verified absolute project root.
- `continuity_id`: stable project workstream id.
- `idempotency_key`: stable transaction replay key.
- `hlt`, `hlt_confirmed`: operator HLT and confirmation authority.
- `specification_ref`, `acceptance_criteria`, `current_state`, `desired_end_state`: required specification/state evidence.
- `mid_level_goal`, `short_term_goal`, `waypoints`: optional explicit lower levels; otherwise deliberate bounded inference supplies provenance and confidence.
- `task_provider`: detected/adopted provider name; Beads is detected automatically.
- `allow_task_decomposition`: permits packet-local decomposition when no valid task path exists.
- `confirm`: required `true` for commit or takeover.
- `takeover`: replaces a conflicting active workstream only with `confirm:true`; otherwise the packet returns four plain-language coordination choices.

Unknown properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_genesis`.

## Output

Returns `focusa.project_genesis.v1` with:

`project_identity → bootstrap_receipt → hlt → specification_and_acceptance → current_and_desired_state → mlg → stg → waypoints → task_provider_and_task_graph → first_workpoint → coordination_owner → readiness_receipt`.

Every packet includes scope, authority, freshness, provenance, confidence, idempotency, evidence refs, next action, and recovery tools.

## Example

```json
{
  "action": "start",
  "project_root": "/workspace/project",
  "continuity_id": "project-main",
  "idempotency_key": "genesis-project-main",
  "hlt": "Ship the verified product",
  "hlt_confirmed": true,
  "specification_ref": "docs/01-product-spec.md",
  "acceptance_criteria": ["First Workpoint is active"],
  "current_state": "Genesis incomplete",
  "desired_end_state": "Project ready",
  "allow_task_decomposition": true
}
```

## Authority and side effects

- Scope: `{"kind":"write","route_family":"explicit_project_continuity"}`.
- Authority: canonical only after `commit` with `confirm=true`.
- `start`, `resume`, and `status` never report the project ready.
- Commit writes the Genesis journal first, persists Trajectory plus Workpoint in one reducer batch, writes the ready packet, then updates the project marker last.
- Idempotent replay returns the same Workpoint and receipt.

## Failure and recovery

- `project_identity_missing` → `focusa_project_verify`.
- `hlt_impasse` → answer one concise HLT question, then resume.
- `incomplete` → supply listed specification/state/task links.
- coordination conflict → show View current work, Coordinate, Take over with confirmation, or Continue read-only; never expose writer-lease jargon.
- interrupted `preparing` transaction → call `resume` with the same idempotency key.

## Dependencies and workflow position

- Prerequisite: `focusa_project_verify`.
- Likely next: `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_project_verify`.
- Pi: `focusa_project_genesis`; MCP: `focusa.project.genesis`; OpenAI: `focusa_project_genesis`.
- CLI: `focusa project genesis start|resume|status|commit`.
- REST: `/v1/project/genesis/{start,resume,status,commit}`.
- Specification: `docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md` §6–7.
