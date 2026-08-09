# `focusa_call_stack_verify`

Verify a Call Stack Design against bounded implementation surfaces and report drift: entry surface, handlers, services, adapters, storage, output envelope, evidence, and Workpoint/STG alignment. Advisory only. Use it when Verify a Call Stack Design against bounded implementation surfaces and report drift without mutating Focus State. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Verify a Call Stack Design against bounded implementation surfaces and report drift without mutating Focus State.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root for the design. Defaults to Pi session cwd.
- `continuity_id` (optional; string): Optional continuity scope filter.
- `design_id` (optional; string): Specific Call Stack Design id to verify.
- `entry_name` (optional; string): Entry name to verify when design_id is omitted.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_call_stack_verify`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_call_stack_verify.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- broad roots such as /root
- parallel memory outside the active project+continuity scope

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_call_stack_design_verify_drift`, `read_call_stack_design_verify_drift`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_call_stack_design` (likely_next)
- `focusa_workpoint_link_evidence` (likely_next)
- `focusa_trajectory_assess` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_call_stack_design`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-spec-implementation`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_call_stack_verify`; MCP: `focusa.call.stack.verify`; OpenAI: `focusa_call_stack_verify`.
- CLI: `focusa call-stack verify`, `focusa call-stack list`, `focusa call-stack show`.
- REST: `POST /v1/call-stack/verify`, `GET /v1/call-stack/list`, `GET /v1/call-stack/show`.
- Specification: contract registry.
- Descriptor digest: `sha256:b3dd5aa1096eabd052ccc227407c0f423e5e2606eab91a4fb302644f4c2d1aab`.
