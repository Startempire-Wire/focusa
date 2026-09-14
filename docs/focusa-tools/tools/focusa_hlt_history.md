# `focusa_hlt_history`

Read append-only HLT ledger entries with session filters, fallback candidates, and generic HLT tracking. Spec 125 §7.2-7.6. Use it when Read append-only HLT change history with session filters, fallback candidates, and generic HLT tracking. Spec 125 §7.2-7.6. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read append-only HLT change history with session filters, fallback candidates, and generic HLT tracking. Spec 125 §7.2-7.6.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root for HLT history scope.
- `continuity_id` (optional; string): Optional continuity_id filter.
- `session_id` (optional; string): Spec 125 §7.6: filter by session. 'current' resolves to active session.
- `include_cross_session_fallbacks` (optional; boolean): Include cross-session fallback candidates (default false).
- `include_generic` (optional; boolean): Include generic HLT entries (default false).
- `limit` (optional; integer; min=1, max=500): Max entries to return (defaults to 50).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_hlt_history`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_hlt_history.md

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

- `focusa_trajectory_view` (likely_next)
- `focusa_trajectory_define_goal` (likely_next)
- `focusa_project_verify` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_trajectory_view`, `focusa_trajectory_define_goal`, `focusa_project_verify`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_hlt_history`; MCP: `focusa.hlt.history`; OpenAI: `focusa_hlt_history`.
- CLI: `focusa hlt history`, `focusa hlt sessions`, `focusa hlt fallback`.
- REST: `GET /v1/hlt/history`.
- Specification: contract registry.
- Descriptor digest: `sha256:19ec12f728fd64769c4801c1273925e3a47504ca30695ebd8409359f3658e0d3`.
