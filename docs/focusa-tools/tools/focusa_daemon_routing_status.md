# `focusa_daemon_routing_status`

Resolve one explicit project/worktree/continuity/native-session scope against a supplied daemon registry. Never infers a global or foreign daemon. Use it when Resolve one explicit project/worktree/continuity/native-session scope against a supplied daemon registry. Never infers a global or foreign daemon. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Resolve one explicit project/worktree/continuity/native-session scope against a supplied daemon registry. Never infers a global or foreign daemon.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `registry` (required; structured): Canonical daemon registry projection from the controller.
- `project_root` (required; string): See the strict descriptor schema.
- `continuity_id` (required; string): See the strict descriptor schema.
- `working_subpath_id` (required; string): See the strict descriptor schema.
- `native_session_id` (required; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_daemon_routing_status`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "registry": {},
  "project_root": "/tmp/focusa-project",
  "continuity_id": "continuity-demo",
  "working_subpath_id": "example",
  "native_session_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_daemon_routing_status.md

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

- Scope: `{"kind":"write","route_family":"daemon-routing"}`
- Authority: `{"kind":"canonical","path":"/v1/daemon-routing/resolve"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `true`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_project_identity` (likely_next)
- `focusa_tool_doctor` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_project_identity`, `focusa_tool_doctor`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_daemon_routing_status`; MCP: `focusa.daemon.routing.status`; OpenAI: `focusa_daemon_routing_status`.
- CLI: `focusa daemon-routing status`.
- REST: `POST /v1/daemon-routing/resolve`.
- Specification: `docs/158-focusa-daemon-routing-surface-parity-spec.md`.
- Descriptor digest: `sha256:732fbc5685a9c5b98b34a60141aba7f968530285fd97bcba95fbbd3e4a8e6e06`.
