# `focusa_context_cognition_curate`

Spec 100 Phase 3 — token-budgeted context selection. Takes candidates (files/docs/diffs/snippets/codemaps/evidence) and selects the highest-scoring subset under a token budget. Returns selected_context + excluded_context (with reasons). Use it when Spec 100 Phase 3 — token-budgeted context selection. Ranks candidates by workpoint target + evidence overlap and selects the highest-scoring subset under a token budget. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Spec 100 Phase 3 — token-budgeted context selection. Ranks candidates by workpoint target + evidence overlap and selects the highest-scoring subset under a token budget.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root. Defaults to Pi session cwd.
- `continuity_id` (optional; string): Optional continuity id filter.
- `target` (optional; string): Curator target string (workpoint next_slice, mission, query). Defaults to the active workpoint's next_slice/mission.
- `token_budget` (optional; integer; min=1, max=1000000): Token budget for the selection. Defaults to 2000.
- `candidates` (optional; array): Candidates to curate. Each is a {kind, path, body?, evidence_ref?, tokens?} object.
- `evidence_refs` (optional; array): Evidence refs that boost candidate ranking when matched.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_context_cognition_curate`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_context_cognition_curate.md

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
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_context_cognition` (likely_next)
- `focusa_context_cognition_render` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_context_cognition`, `focusa_context_cognition_render`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_context_cognition_curate`; MCP: `focusa.context.cognition.curate`; OpenAI: `focusa_context_cognition_curate`.
- CLI: `focusa context-cognition curate`.
- REST: `POST /v1/context-cognition/curate`.
- Specification: contract registry.
- Descriptor digest: `sha256:c94350bd76d6f6dfdf59ba8498d112750acd279ca2287667b6b21440953f2127`.
