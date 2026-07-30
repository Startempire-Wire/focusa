# `focusa_instruction_integrity_status`

Read foundational guard availability, amendment authority, and outage posture. Use it when Operate the Spec 140 instruction integrity status surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Operate the Spec 140 instruction integrity status surface with typed scope and evidence.
- Capability family: `agent_runtime`; namespace: `focusa.agent_runtime`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- No arguments.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_instruction_integrity_status`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_instruction_integrity_status.md

## Anti-examples

- unverified prompt sources
- silent prompt replacement
- artifact delivery without a Receipt

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"agent-runtime"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `true`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_instruction_integrity_evaluate` (likely_next)
- `focusa_agent_runtime_doctor` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_instruction_integrity_evaluate`, `focusa_agent_runtime_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-spec-implementation`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:agent_runtime`
- Pi: `focusa_instruction_integrity_status`; MCP: `focusa.instruction.integrity.status`; OpenAI: `focusa_instruction_integrity_status`.
- CLI: `focusa agent-runtime integrity-status`.
- REST: `GET /v1/agent-runtime/instruction-integrity/status`.
- Specification: `docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md`.
- Descriptor digest: `sha256:189b42681fa21a40013f538d3b812fd546d7408a24be5d0de06bde53a1b81222`.
