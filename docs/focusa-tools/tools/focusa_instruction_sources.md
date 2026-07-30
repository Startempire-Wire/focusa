# `focusa_instruction_sources`

Discover bounded, registered project instruction sources with trust and authority metadata. Use it when Operate the Spec 140 instruction sources surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Operate the Spec 140 instruction sources surface with typed scope and evidence.
- Capability family: `agent_runtime`; namespace: `focusa.agent_runtime`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (required; string): See the strict descriptor schema.
- `max_source_bytes` (optional; number; min=1): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_instruction_sources`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "project_root": "/tmp/focusa-project"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_instruction_sources.md

## Anti-examples

- unverified prompt sources
- silent prompt replacement
- artifact delivery without a Receipt

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"agent-runtime"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_instruction_conflicts` (likely_next)
- `focusa_instruction_explain` (likely_next)
- `focusa_agent_runtime_effective` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_instruction_conflicts`, `focusa_instruction_explain`, `focusa_agent_runtime_effective`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Runbooks: `runbook:agent_runtime`
- Pi: `focusa_instruction_sources`; MCP: `focusa.instruction.sources`; OpenAI: `focusa_instruction_sources`.
- CLI: `focusa agent-runtime sources`.
- REST: `GET /v1/agent-runtime/instructions/sources`.
- Specification: `docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md`.
- Descriptor digest: `sha256:ef221a86d39c69d2efdcf989baca720125d0d5ac312fe7940ca4e7d5ba45cc48`.
