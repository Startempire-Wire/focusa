# `focusa_browser_diagnostics_intake`

Turn UIAI/browser diagnostics JSON into bounded Focusa evidence, active-object hints, a prediction candidate, and a metacog candidate. Use it when Turn UIAI/browser diagnostics JSON into bounded Workpoint evidence, active-object hints, prediction context, and optional metacog learning. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Turn UIAI/browser diagnostics JSON into bounded Workpoint evidence, active-object hints, prediction context, and optional metacog learning.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `diagnostics` (optional; structured): Diagnostics JSON object or browser action failure envelope.
- `diagnostics_ref` (optional; string): Stable file/artifact/URL handle for diagnostics JSON; local files are read best-effort.
- `target_ref` (optional; string): Object/page/endpoint proven by these diagnostics; inferred from diagnostics when omitted.
- `result` (optional; string): Optional bounded result summary override.
- `workpoint_id` (optional; string): Specific Workpoint id; omit to use active Workpoint.
- `project_root` (optional; string): Explicit project root for canonical evidence linkage.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to this Pi session key.
- `continuity_id` (optional; string): Stable logical session/workstream id; defaults to this Pi continuity id.
- `attach_to_workpoint` (optional; boolean): Defaults true; false performs dry intake without canonical evidence linkage.
- `create_prediction` (optional; boolean): Defaults true; records bounded follow-up prediction candidate.
- `create_metacog` (optional; boolean): Defaults false; capture only when this diagnostics pattern should become reusable learning.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_browser_diagnostics_intake`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_browser_diagnostics_intake.md

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
- Side effects: `composite_evidence_prediction_optional_metacog`, `composite_evidence_prediction_optional_metacog`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_active_object_resolve` (likely_next)
- `focusa_evidence_capture` (likely_next)
- `focusa_predict_record` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_active_object_resolve`, `focusa_evidence_capture`, `focusa_predict_record`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-browser-uiai`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_browser_diagnostics_intake`; MCP: `focusa.browser.diagnostics.intake`; OpenAI: `focusa_browser_diagnostics_intake`.
- CLI: `focusa workpoint evidence-link`, `focusa predict record`, `focusa metacognition capture`.
- REST: `POST /v1/workpoint/evidence/link`, `POST /v1/predictions`, `POST /v1/metacognition/capture`.
- Assignable: `true`; parity: `pi_only`.
- Specification: contract registry.
- Descriptor digest: `sha256:7e1200617bf62a1b887a1c9154d2b242cfdda2b655bf5d4a7c42f9dcc3464b55`.
