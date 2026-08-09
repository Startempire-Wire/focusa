# `focusa_evidence_capture`

Capture a bounded evidence ref/result and optionally link it to the active Workpoint. Use it when Capture a bounded evidence ref/result and optionally link it to the active Workpoint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Capture a bounded evidence ref/result and optionally link it to the active Workpoint.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.
- Trajectory-aware evidence supplies proof alignment metadata for the active trajectory and its HLT, MLG, and STG context without expanding the evidence payload.

## Parameters and strict input schema

- `target_ref` (required; string): Object/file/test/endpoint/work item proven by this evidence.
- `result` (required; string): Bounded result summary.
- `evidence_ref` (required; string): Stable evidence handle/path/test id.
- `workpoint_id` (optional; string): Specific Workpoint id; omit to use active Workpoint.
- `project_root` (optional; string): Explicit safe project folder/root; use after compaction if Pi cwd is broad like /root.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to this Pi session key.
- `continuity_id` (optional; string): Stable logical session/workstream id; defaults to this Pi continuity id.
- `attach_to_workpoint` (optional; boolean): Defaults true.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_evidence_capture`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "target_ref": "example",
  "result": "example",
  "evidence_ref": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_evidence_capture.md

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
- Side effects: `evidence_link`, `evidence_link`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workpoint_link_evidence` (likely_next)
- `focusa_trajectory_assess` (likely_next)
- `focusa_recent_result` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`, `focusa_recent_result`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-evidence-outcomes`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_evidence_capture`; MCP: `focusa.evidence.capture`; OpenAI: `focusa_evidence_capture`.
- CLI: `focusa workpoint evidence-link`.
- REST: `POST /v1/workpoint/evidence/link`.
- Specification: contract registry.
- Descriptor digest: `sha256:6d457a7005c1c46c20f9d410b5871e1eee125dc5c13f2c4ba4945a9eefef15af`.
