# `focusa_browser_capabilities_intake`

Validate and govern a UIAI or WebMCP browser capability manifest. Binds page tools to one session and origin, treats page safety annotations as untrusted, requires confirmation/evidence for mutation, and returns Focusa browser capability descriptors without executing them. Use it when Validate and session/origin-bind UIAI or WebMCP page capabilities under Focusa governance. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Validate and session/origin-bind UIAI or WebMCP page capabilities under Focusa governance.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `session_id` (required; string): Exact active UIAI browser session identifier.
- `origin` (required; string): Absolute http(s) page origin bound to these capabilities.
- `source` (optional; string | string | string): See the strict descriptor schema.
- `trusted_origin` (optional; boolean; default=false): See the strict descriptor schema.
- `tools` (required; array): See the strict descriptor schema.
- `project_root` (optional; string): See the strict descriptor schema.
- `continuity_id` (optional; string): See the strict descriptor schema.
- `workpoint_id` (optional; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_browser_capabilities_intake`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "session_id": "example",
  "origin": "https://example.com",
  "tools": []
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_browser_capabilities_intake.md

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

- Scope: `{"kind":"write","route_family":"browser"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_browser_capability_evidence`, `write_browser_capability_evidence`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_browser_workflow_plan` (likely_next)
- `focusa_browser_diagnostics_intake` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_browser_workflow_plan`, `focusa_browser_diagnostics_intake`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-browser-uiai`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_browser_capabilities_intake`; MCP: `focusa.browser.capabilities.intake`; OpenAI: `focusa_browser_capabilities_intake`.
- CLI: `focusa help all --json`.
- REST: `POST /v1/browser/capabilities/intake`.
- Specification: `docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md`.
- Descriptor digest: `sha256:a22694368089c7eb212cf761a09349c10ed6704b98ee0db49ba9ef291475b505`.
