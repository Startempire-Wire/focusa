# `focusa_project_verify`

**Family:** `project_identity`  
**Label:** Project Verify

## Purpose

Verify expected ProjectIdentity fields and surface project mismatches without mutating Focusa state.

## When to use

- Before treating a packet as canonical after compaction/model switch/session resume.
- When operator supplied an expected project root, id, name, or remote.
- When Focusa reports `scope_mismatch`, `read_model_lag`, or degraded ProjectIdentity. (`scope_mismatch` is the legacy failure-class name for project/continuity context mismatch.)

## Parameters

- `cwd` — optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` — expected project root.
- `project_id` — expected project id.
- `canonical_name` — expected canonical project name.
- `repo_remote` — expected git origin remote.
- `remote_host`, `remote_user`, `remote_port` — optional remote SSH context for a project that lives outside the local daemon filesystem.
- `remote_repo_remote`, `remote_workspace_kind`, `remote_deploy_root` — optional caller-supplied remote evidence to verify against the expected project root.
- `persisted_project_root`, `persisted_project_fingerprint`, `persisted_project_id`, `persisted_canonical_name` — optional prior-session ProjectIdentity signal used to detect stale/cross-session scope before canonical trust.

## Expected result

Returns ProjectIdentity plus `verification.verified`, quorum rule, matching independent signal count, aliases, Beads issue-prefix evidence, persisted-session signal diagnostics, and mismatch diagnostics. Remote SSH verification may return `remote_context` and `authority_boundary=remote_host_plus_project_root_plus_fingerprint` when caller-supplied remote evidence forms the quorum. Pi results include `details.tool_result_v1` with `status`, `failure_class`, `canonical`, `degraded`, recovery posture, and `next_tools`.

## Failure and recovery

- `failure_class=hot_path_timeout` or `status=timeout_preserved`: cached ProjectIdentity can be returned as noncanonical advisory only; retry verification after `focusa_tool_doctor`/`focusa_resource_mode` before trusting scope.

- `failure_class=scope_mismatch`: suppress stale packet/context; use current repo/operator scope and retry with corrected expected fields.
- `canonical=false`: do not promote Workpoint/Trajectory carryover as canonical.
- `validation_rejected` or HTTP schema error: fix request fields; do not retry unchanged.

## Example

```text
focusa_project_verify cwd="/home/wirebot/focusa" project_root="/home/wirebot/focusa" project_id="focusa"
focusa_project_verify project_root="/home/site/project" remote_host="site.example" remote_repo_remote="git@github.com:org/project.git" remote_workspace_kind="wordpress"
```

## Contract summary

- Family: Project Identity.
- Side effects: `read_state`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/project/verify`
- CLI commands: `focusa project verify`
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Spec96 ProjectIdentity quorum and project-folder safety.
- Live check: contract_static plus /v1/project/verify safe probe and mismatch diagnostics.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Backed by `POST /v1/project/verify`.
