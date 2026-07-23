# `focusa_bloatgaurd_tokenbloat_domain`

Spec 101 — read one Tokenbloat Control domain and its prompt-visible fields/boundaries. Use it when Spec 101 — read one Tokenbloat Control domain and its prompt-visible fields/boundaries. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Spec 101 — read one Tokenbloat Control domain and its prompt-visible fields/boundaries.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `name` (required; string): Tokenbloat domain slug or title.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_bloatgaurd_tokenbloat_domain`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "name": "focusa_workpoint_resume"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_domain.md

## Anti-examples

- hiding failures behind null/unknown
- silent deletion or cleanup

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

- `focusa_bloatgaurd_tokenbloat_report` (likely_next)
- `focusa_traverse` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_bloatgaurd_tokenbloat_report`, `focusa_traverse`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_bloatgaurd_tokenbloat_domain`; MCP: `focusa.bloatgaurd.tokenbloat.domain`; OpenAI: `focusa_bloatgaurd_tokenbloat_domain`.
- CLI: `focusa bloatgaurd token-domain <name>`.
- REST: `GET /v1/bloatgaurd/tokenbloat/domain/{name}`.
- Specification: contract registry.
- Descriptor digest: `sha256:c038a7b0807a426f2cdc6371082622d37d09720bab9bfd676d668e5dffcbca1e`.
