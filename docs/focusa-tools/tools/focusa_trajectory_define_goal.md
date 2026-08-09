# `focusa_trajectory_define_goal`

Create an advisory per-project Trajectory goal candidate without changing task/execution authority. Use it when Create an advisory per-project Trajectory goal candidate, including HLT/MLG/STG/Waypoints, without changing task or execution authority. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Create an advisory per-project Trajectory goal candidate, including HLT/MLG/STG/Waypoints, without changing task or execution authority.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `long_term_goal` (required; string): Stable project-level long-term goal.
- `desired_end_state` (required; string): Evidence-backed desired project end state.
- `mid_level_goal` (optional; string): Current mid-level goal (MLG) derived from the HLT.
- `short_term_goal` (optional; string): Current short-term goal (STG) derived from the HLT/MLG.
- `waypoints` (optional; array): Concrete HLT-aligned progress markers along the MLG/STG path.
- `current_state` (optional; string): Current verified state if known.
- `current_ask` (optional; string): Explicit current operator intent; satisfies verified state gate (§169-175). Auto-populated from Pi session if omitted.
- `goal_source` (optional; string): operator|durable_supersession|focus_state|workpoint|beads|imported|inferred_context
- `supersedes_trajectory_id` (optional; string): Prior trajectory id if this supersedes one.
- `operator_confirmed` (optional; boolean): True when operator explicitly confirmed a root goal change.
- `supersession_evidence_refs` (optional; array): Durable evidence refs allowing root goal supersession without direct operator prompt.
- `required_evidence_refs` (optional; array): Evidence refs required to prove the desired end state.
- `required_checks` (optional; array): Checks required before the trajectory can be considered done.
- `acceptance_risks` (optional; array): Known false-completion or acceptance risks.
- `not_done_if` (optional; array): Conditions proving the trajectory is not done.
- `project_root` (optional; string): Optional expected project root; defaults to Pi session cwd.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to Pi session key.
- `continuity_id` (optional; string): Optional logical continuity id; defaults to Pi continuity id.
- `idempotency_key` (optional; string): Optional external idempotency key.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_trajectory_define_goal`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "long_term_goal": "example",
  "desired_end_state": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_trajectory_define_goal.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- overriding Workpoint/operator authority
- merging sessions on goal similarity alone

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `advisory_projection`, `advisory_projection`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_trajectory_assess` (likely_next)
- `focusa_trajectory_propose_workpoint` (likely_next)
- `focusa_trajectory_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_trajectory_assess`, `focusa_trajectory_propose_workpoint`, `focusa_trajectory_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_trajectory_define_goal`; MCP: `focusa.trajectory.define.goal`; OpenAI: `focusa_trajectory_define_goal`.
- CLI: `focusa trajectory define-goal`.
- REST: `POST /v1/trajectory/define-goal`.
- Specification: contract registry.
- Descriptor digest: `sha256:af2fb8612ecce461feb74ce7a2e3efdd71d470d4ebcbf1fe42bf036eefddd4f6`.
