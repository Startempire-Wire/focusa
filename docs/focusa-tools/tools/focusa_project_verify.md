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

## Expected result

Returns ProjectIdentity plus `verification.verified`, quorum rule, matching independent signal count, and mismatch diagnostics. Pi results include `details.tool_result_v1` with `status`, `failure_class`, `canonical`, `degraded`, recovery posture, and `next_tools`.

## Failure and recovery

- `failure_class=scope_mismatch`: suppress stale packet/context; use current repo/operator scope and retry with corrected expected fields.
- `canonical=false`: do not promote Workpoint/Trajectory carryover as canonical.
- `validation_rejected` or HTTP schema error: fix request fields; do not retry unchanged.

## Example

```text
focusa_project_verify cwd="/home/wirebot/focusa" project_root="/home/wirebot/focusa" project_id="focusa"
```

## Source

Backed by `POST /v1/project/verify`.
