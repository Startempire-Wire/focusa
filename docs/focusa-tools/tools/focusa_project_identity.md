# `focusa_project_identity`

**Family:** `project_identity`  
**Label:** Project Identity

## Purpose

Resolve the active ProjectIdentity before trusting project-bound Workpoints, Trajectory packets, evidence, or carryover context.

## When to use

- At project start/resume when the project folder is unclear.
- Before accepting a Workpoint/Trajectory packet as canonical.
- When a packet, cwd, Beads root, git root, or operator-provided project folder might point at different projects.

## Parameters

- `cwd` — optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` — optional expected project folder/root.

## Expected result

Returns a bounded ProjectIdentity with `status`, `project_id`, `canonical_name`, `project_root`, `fingerprint`, `confidence`, `signals`, `mismatches`, `verified_at`, and quorum details. It marks unsafe broad roots such as `/root` as `status=unsafe_project_root`, `canonical=false`. Pi results include `details.tool_result_v1` with `status`, `failure_class`, `canonical`, `degraded`, recovery posture, and `next_tools`.

## Failure and recovery

- `failure_class=scope_mismatch`: resolve mismatched marker/git/beads/cwd/operator signals, then retry unchanged only after scope is corrected.
- `canonical=false` or `confidence=low|medium`: treat identity as advisory/degraded and verify before canonical resume.
- `daemon_unavailable`: continue from current repo and Beads as noncanonical fallback, then retry `/v1/project/identity`.

## Example

```text
focusa_project_identity cwd="/home/wirebot/focusa"
```

## Source

Backed by `GET /v1/project/identity`.
