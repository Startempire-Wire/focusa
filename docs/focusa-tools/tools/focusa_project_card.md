# `focusa_project_card`

Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals. Use it when Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals for bootstrap/re-bootstrap. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals for bootstrap/re-bootstrap.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `cwd` (optional; string): Optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` (optional; string): Optional expected project root folder.
- `current_ask` (optional; string): Optional current ask used to seed bootstrap/re-bootstrap candidate.
- `remote_host` (optional; string): Remote SSH host that contains the project root; caller supplies inspected evidence.
- `remote_user` (optional; string): Remote SSH user, if known.
- `remote_port` (optional; number; min=1, max=65535): Remote SSH port, if known.
- `remote_repo_remote` (optional; string): Git origin/repo remote observed on the remote host.
- `remote_workspace_kind` (optional; string): Workspace kind observed on the remote host.
- `remote_deploy_root` (optional; string): Deployment/site root observed on the remote host.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_card`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_card.md

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

- `focusa_project_card_outcome` (likely_next)
- `focusa_traverse` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_metacog_retrieve` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_project_card_outcome`, `focusa_traverse`, `focusa_trajectory_view`, `focusa_metacog_retrieve`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_card`; MCP: `focusa.project.card`; OpenAI: `focusa_project_card`.
- CLI: `focusa project card`.
- REST: `GET /v1/project/card`.
- Assignable: `true`; parity: `domain`.
- Specification: contract registry.
- Descriptor digest: `sha256:36681a516b0a4b3be4e294a9bb1a5b7f4a8098127624eafcc63038307d9111bb`.
