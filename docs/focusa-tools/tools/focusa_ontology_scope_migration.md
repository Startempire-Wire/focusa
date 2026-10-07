# `focusa_ontology_scope_migration`

Dry-run, apply, inspect, or roll back granular legacy ontology scope migration. Apply/rollback require explicit confirmation and per-record evidence; ownership is never inferred. Use it when Dry-run, apply, inspect, and roll back granular evidence-backed migration of quarantined legacy ontology records into one verified workstream. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Dry-run, apply, inspect, and roll back granular evidence-backed migration of quarantined legacy ontology records into one verified workstream.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (required; string | string | string | string): See the strict descriptor schema.
- `migration_id` (optional; string): Stable UUID for apply/retry or rollback target.
- `rollback_id` (optional; string): Stable UUID for idempotent rollback/retry.
- `selections` (optional; array): See the strict descriptor schema.
- `evidence_refs` (optional; array): See the strict descriptor schema.
- `confirm` (optional; boolean): Required true for apply or rollback mutation.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_ontology_scope_migration`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "action": "dry_run"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_ontology_scope_migration.md

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

- Scope: `{"kind":"write","route_family":"ontology"}`
- Authority: `{"kind":"canonical","path":"reducer event plus append-only receipt"}`
- Side effects: `confirmed_append_only_scope_migration`, `confirmed_append_only_scope_migration`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `null`; preview supported: `null`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

Complete journey: `docs/agent/02-focusa-cohesive-project-flow.md` (Bootstrap/Genesis → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance). Reuse valid state; select capabilities by condition rather than running a tool list as a script.
A rejected operation calls for exact-cause diagnosis and supported scoped recovery, not automatic mission abandonment; preserve genuine authority boundaries and resume only after verification.
Shared boundaries and conditional crosswalk: [AUTHORITY_MODEL.md](../../current/AUTHORITY_MODEL.md), [GOLDEN_WORKFLOW.md](../../current/GOLDEN_WORKFLOW.md), and [AGENT_ADAPTER_CONTRACT.md](../../current/AGENT_ADAPTER_CONTRACT.md).

- `focusa_project_verify` (likely_next)
- `focusa_evidence_capture` (likely_next)
- `focusa_workpoint_link_evidence` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_project_verify`, `focusa_evidence_capture`, `focusa_workpoint_link_evidence`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_ontology_scope_migration`; MCP: `focusa.ontology.scope.migration`; OpenAI: `focusa_ontology_scope_migration`.
- CLI: `focusa ontology scope-migration-dry-run`, `focusa ontology scope-migration-status`, `focusa ontology scope-migration-apply`, `focusa ontology scope-migration-rollback`.
- REST: `POST /v1/ontology/scope-migrations`.
- Specification: `docs/151-focusa-emergency-cross-project-scope-isolation-locked-release-addendum.md`.
- Descriptor digest: `sha256:8748a85fa2f1ede90ef5389daf86aea80eae079e5a6eacfe60936137ae5a192f`.
