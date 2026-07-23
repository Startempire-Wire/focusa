# `focusa_preload_build`

Build Preload Packet through the scoped Spec 111 preload API. Use it when Build a scoped agent bootstrap packet without writing it. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Build a scoped agent bootstrap packet without writing it.
- Capability family: `preload`; namespace: `focusa.preload`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `profile` (optional; string | string | string | string): Preload profile id from focusa_preload_profiles. Defaults to rules_and_context.
- `project_root` (optional; string): See the strict descriptor schema.
- `continuity_id` (optional; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_preload_build`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_preload_build.md

## Anti-examples

- writing outside allowlisted paths
- committing receipts without an idempotency key

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"preload"}`
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

- `focusa_preload_render` (likely_next)
- `focusa_preload_verify` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_preload_render`, `focusa_preload_verify`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:preload`
- Pi: `focusa_preload_build`; MCP: `focusa.preload.build`; OpenAI: `focusa_preload_build`.
- CLI: `focusa preload build`.
- REST: `POST /v1/preload/build`.
- Specification: `docs/111-agent-context-bootstrap-and-delivery-spec.md`.
- Descriptor digest: `sha256:45a70143c1801d51a71498e76dd23ff33f9270b1f9dd2d74c55a54e7114aa137`.
