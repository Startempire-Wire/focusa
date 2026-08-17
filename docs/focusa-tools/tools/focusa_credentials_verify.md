# `focusa_credentials_verify`

Ask the Credential Authority whether a requirement is satisfied by the given grants — secret-free: the verdict and reasons only, never secret values. Use before touching any provider seam. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done.
- Capability family: `credential`; namespace: `focusa.credential`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `requirement` (required; object): See the strict descriptor schema.
- `grants` (required; array): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_credentials_verify`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "requirement": {
    "schema": "example",
    "credential_role_ref": "example",
    "required_operation": "example",
    "required_exposure_mode": "example",
    "exact_consumer_ref": "example",
    "exact_target_refs": [],
    "use_count_required": 0,
    "evidence_requirement_refs": []
  },
  "grants": []
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_credentials_verify.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- when another narrower tool is explicitly indicated

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_verdict`, `read_verdict`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_tool_doctor` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tool_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`
- Runbooks: `runbook:credential`
- Pi: `focusa_credentials_verify`; MCP: `focusa.credentials.verify`; OpenAI: `focusa_credentials_verify`.
- CLI: none.
- REST: `/v1/credentials/verify-requirement `.
- Specification: contract registry.
- Descriptor digest: `sha256:c4a885359a9ba261e4f5f6c8267de11d5f62b99caa8662dfd9fb16f6a6743ba1`.
