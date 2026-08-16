# `focusa_project_verify`

Verify active project folder against expected ProjectIdentity fields and report mismatches without mutating state. Use it when Verify expected project identity fields against ranked worktree/session candidates and surface ambiguity or project/continuity mismatches without mutating Focusa state. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Verify expected project identity fields against ranked worktree/session candidates and surface ambiguity or project/continuity mismatches without mutating Focusa state.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.
- Verification projects the same binding_candidates and refuses canonical authority when the candidate decision is ambiguous.

## Parameters and strict input schema

- `cwd` (optional; string): Optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` (optional; string): Expected project root.
- `persisted_project_root` (optional; string): Resumed-session project root to compare as an advisory binding candidate.
- `project_id` (optional; string): Expected project id from marker/operator.
- `canonical_name` (optional; string): Expected canonical project name.
- `repo_remote` (optional; string): Expected git origin remote.
- `remote_host` (optional; string): Remote SSH host that contains the project root; caller supplies inspected evidence.
- `remote_user` (optional; string): Remote SSH user, if known.
- `remote_port` (optional; integer; min=1, max=65535): Remote SSH port, if known.
- `remote_repo_remote` (optional; string): Git origin/repo remote observed on the remote host.
- `remote_workspace_kind` (optional; string): Workspace kind observed on the remote host.
- `remote_deploy_root` (optional; string): Deployment/site root observed on the remote host.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_verify`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_verify.md

## Anti-examples

- assuming unsafe broad cwd is canonical
- skipping verify after scope mismatch

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Missing-marker routing (#243)

When verification succeeds but no `.focusa-project.json` marker exists,
route the operator to the formal command — never write the marker JSON
manually: `focusa init --project-root <path> --quickstart` (existing
non-empty projects: `focusa init --project-root <path>`; legacy markers:
inspect then apply enrichment). Marker production is owned by the canonical
core service (atomic, idempotent, ownership-preserving).

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_trajectory_view` (likely_next)
- `focusa_workpoint_resume` (likely_next)
- `focusa_tool_doctor` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_trajectory_view`, `focusa_workpoint_resume`, `focusa_tool_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_verify`; MCP: `focusa.project.verify`; OpenAI: `focusa_project_verify`.
- CLI: `focusa project verify`.
- REST: `POST /v1/project/verify`.
- Specification: contract registry.
- Descriptor digest: `sha256:bb09ec069011fa1dd6e4669076544c84c6fd230c2462b7a6bbede99202efd185`.
