# `focusa_project_verify`

Verify active project folder against expected ProjectIdentity fields and report mismatches without mutating state. Use it when Verify expected project identity fields against ranked worktree/session candidates and surface ambiguity or project/continuity mismatches without mutating Focusa state. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

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

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- assuming unsafe broad cwd is canonical
- skipping verify after scope mismatch

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Advisory source hints (not dispatch/replay guarantees): read-only `true`; destructive `false`; idempotent `true`; open-world `true`. Verify selected action/schema/policy separately.
- Confirmation required: `unknown (null)`; preview supported: `unknown (null)`. Inspect the selected action, strict installed schema and policy; unknown is not permission.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

Complete journey: `docs/agent/02-focusa-cohesive-project-flow.md` (Bootstrap/Genesis → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance). Reuse valid state; select capabilities by condition rather than running a tool list as a script.
A rejected operation calls for exact-cause diagnosis and supported scoped recovery, not automatic mission abandonment; preserve genuine authority boundaries and resume only after verification.
Shared boundaries and conditional crosswalk: [AUTHORITY_MODEL.md](../../current/AUTHORITY_MODEL.md), [GOLDEN_WORKFLOW.md](../../current/GOLDEN_WORKFLOW.md), and [AGENT_ADAPTER_CONTRACT.md](../../current/AGENT_ADAPTER_CONTRACT.md).

- `focusa_project_bootstrap` (likely_next)
- `focusa_project_genesis` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_workpoint_resume` (likely_next)
- `focusa_tool_doctor` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_project_bootstrap`, `focusa_project_genesis`, `focusa_trajectory_view`, `focusa_workpoint_resume`, `focusa_tool_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_verify`; MCP: `focusa.project.verify`; OpenAI: `focusa_project_verify`.
- CLI: `focusa project verify`.
- REST: `POST /v1/project/verify`.
- Specification: contract registry.
- Descriptor digest: `sha256:21755584ed92bc9d67c73155805bf68f1e1fe1bc768ab36cf6e7fcc56e8e73eb`.
