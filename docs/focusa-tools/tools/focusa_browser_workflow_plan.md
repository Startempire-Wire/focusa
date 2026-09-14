# `focusa_browser_workflow_plan`

Build the governed UIAI/WebMCP sequence for one browser operation before action. Returns health, read/source, diagnostics, snapshot refs, mutation confirmation, bound execution, evidence intake, Workpoint linkage, and session cleanup steps. Use it when Plan a governed UIAI/WebMCP read, action, diagnostics, evidence, and cleanup sequence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Plan a governed UIAI/WebMCP read, action, diagnostics, evidence, and cleanup sequence.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `operation` (required; string): Bounded browser action intent.
- `mutation` (optional; boolean; default=false): See the strict descriptor schema.
- `webmcp_available` (optional; boolean; default=false): See the strict descriptor schema.
- `session_id` (optional; string): See the strict descriptor schema.
- `origin` (optional; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_browser_workflow_plan`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "operation": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_browser_workflow_plan.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- hiding failures behind null/unknown
- silent deletion or cleanup

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"browser"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_browser_capabilities_intake` (likely_next)
- `focusa_browser_diagnostics_intake` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_browser_capabilities_intake`, `focusa_browser_diagnostics_intake`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-browser-uiai`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_browser_workflow_plan`; MCP: `focusa.browser.workflow.plan`; OpenAI: `focusa_browser_workflow_plan`.
- CLI: `focusa help all --json`.
- REST: `POST /v1/browser/workflow/plan`.
- Specification: `docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md`.
- Descriptor digest: `sha256:cf4418b2bd9038a3624033764926e38fff19903175c5312db13fa9441a9065da`.
