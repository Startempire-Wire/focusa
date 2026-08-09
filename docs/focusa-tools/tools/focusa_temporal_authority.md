# `focusa_temporal_authority`

Read, commit, revise, observe, forecast, or preflight project-scoped temporal claims without fabricating deadlines or urgency. Use it when Read, commit, revise, observe, forecast, or preflight scoped temporal claims with evidence, confidence, uncertainty, freshness, and no fabricated urgency. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read, commit, revise, observe, forecast, or preflight scoped temporal claims with evidence, confidence, uncertainty, freshness, and no fabricated urgency.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (optional; string | string | string | string | string | string | string | string | string | string | string | string): Temporal operation; defaults to status.
- `project_root` (optional; string): See the strict descriptor schema.
- `continuity_id` (optional; string): See the strict descriptor schema.
- `host_id` (optional; string): See the strict descriptor schema.
- `operator_id` (optional; string): See the strict descriptor schema.
- `workpoint_id` (optional; string): See the strict descriptor schema.
- `item_id` (optional; string): See the strict descriptor schema.
- `task_id` (optional; string): See the strict descriptor schema.
- `idempotency_key` (optional; string): See the strict descriptor schema.
- `confirm` (optional; boolean): See the strict descriptor schema.
- `as_of` (optional; string): See the strict descriptor schema.
- `phase` (optional; string): See the strict descriptor schema.
- `timezone` (optional; string): See the strict descriptor schema.
- `tzdb_version` (optional; string): See the strict descriptor schema.
- `forecast_authority` (optional; object): See the strict descriptor schema.
- `forecast_evaluation` (optional; structured): See the strict descriptor schema.
- `high_consequence_packet` (optional; structured): See the strict descriptor schema.
- `civil_time_packet` (optional; structured): See the strict descriptor schema.
- `temporal_priority_packet` (optional; structured): See the strict descriptor schema.
- `closure_packet` (optional; structured): See the strict descriptor schema.
- `duration_ms` (optional; number): See the strict descriptor schema.
- `outcome` (optional; string): See the strict descriptor schema.
- `actual_ms` (optional; number): See the strict descriptor schema.
- `evidence_refs` (optional; array): See the strict descriptor schema.
- `claim` (optional; object): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_temporal_authority`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_temporal_authority.md

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

- Scope: `{"kind":"write","route_family":"explicit_project_continuity"}`
- Authority: `{"kind":"canonical","path":"daemon:/v1/temporal/commit"}`
- Side effects: `status_preflight_read_or_confirmed_claim_write_or_observation`, `status_preflight_read_or_confirmed_claim_write_or_observation`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_trajectory_view` (likely_next)
- `focusa_workpoint_resume` (likely_next)
- `focusa_project_verify` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_trajectory_view`, `focusa_workpoint_resume`, `focusa_project_verify`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_temporal_authority`; MCP: `focusa.temporal.authority`; OpenAI: `focusa_temporal_authority`.
- CLI: `focusa temporal status|commit|revise|observe|forecast|preflight`.
- REST: `GET /v1/temporal/status`, `POST /v1/temporal/commit`, `POST /v1/temporal/revise`, `POST /v1/temporal/observe`, `POST /v1/temporal/forecast`, `POST /v1/temporal/preflight`.
- Specification: `docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md`.
- Descriptor digest: `sha256:453a5f07a9c079125ef508cf3f0fe773df7c7baaab9c0be261ce51aa018ffe68`.
