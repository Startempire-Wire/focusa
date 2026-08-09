# `focusa_project_identity`

Resolve bounded ProjectIdentity from cwd/project_root using marker, git, beads, workspace, daemon, and operator project signals. Use it when Rank explicit, active-worktree, canonical-parent, marker/Beads, persisted-session, and bounded parent-directory project candidates; fail closed on ambiguity before trusting project-bound context. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Rank explicit, active-worktree, canonical-parent, marker/Beads, persisted-session, and bounded parent-directory project candidates; fail closed on ambiguity before trusting project-bound context.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.
- binding_candidates rank active worktree, canonical parent, marker/Beads, resumed-session, and bounded parent-directory evidence; ambiguous_project_binding fails closed.

## Parameters and strict input schema

- `cwd` (optional; string): Optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` (optional; string): Optional expected project root folder.
- `persisted_project_root` (optional; string): Project root retained by the resumed Pi session; advisory candidate only until current worktree/project evidence verifies it.
- `remote_host` (optional; string): Remote SSH host that contains the project root; caller supplies inspected evidence.
- `remote_user` (optional; string): Remote SSH user, if known.
- `remote_port` (optional; integer; min=1, max=65535): Remote SSH port, if known.
- `remote_repo_remote` (optional; string): Git origin/repo remote observed on the remote host.
- `remote_workspace_kind` (optional; string): Workspace kind observed on the remote host.
- `remote_deploy_root` (optional; string): Deployment/site root observed on the remote host.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_identity`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_identity.md

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
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_project_card` (likely_next)
- `focusa_project_verify` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_project_card`, `focusa_project_verify`, `focusa_trajectory_view`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_identity`; MCP: `focusa.project.identity`; OpenAI: `focusa_project_identity`.
- CLI: `focusa project identity`.
- REST: `GET /v1/project/identity`.
- Specification: contract registry.
- Descriptor digest: `sha256:110e6646851b581449447c643a05bf620136c6185034d84c05f3aff3ae8775ef`.
