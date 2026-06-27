## Bug Description

When Focusa operates on a remote project (via SSH/VPS) that lacks a `.focusa-project.json` marker, it cannot write the marker file to the remote host. This causes:

1. **Focus State tools blocked**: `focusa_current_focus`, `focusa_next_step`, `focusa_workpoint_checkpoint`, etc. all fail with "Attentive and awaiting operator direction" validation rejections
2. **Scope conflict on resume**: Every new session re-detects the project as low-confidence `cwd_only`
3. **No persistent project identity**: The project has no way to establish itself as a known Focusa project

## Root Cause

**Location**: `crates/focusa-api/src/routes/project.rs` — `discover_identity()` function

Focusa has **no remote write capability**:
- Line 1197: `let remote_nonlocal = remote_hint.is_present() && !start.exists();`
- When `remote_nonlocal = true`, the marker search returns `None` (no local file exists)
- Focusa **never attempts to write** the marker — it only searches locally
- There is no SSH exec/write mechanism in Focusa's remote project handling

For local projects, the operator can run `focusa project init` or similar, but for remote/VPS projects accessed via SSH, Focusa cannot create the marker file itself.

## Expected Behavior

When operating on a verified remote project (confirmed via `remote_host`, `remote_port`, `remote_user` in `RemoteProjectHint`), Focusa should:

1. **Option A (SSH write)**: Use SSH to write `.focusa-project.json` to the remote host automatically
2. **Option B (SSH mkdir + init command)**: Run `focusa project init` on the remote host via SSH
3. **Option C (Bootstrap packet)**: When no marker exists on a verified remote project, surface a clear bootstrap action with SSH details pre-filled

## Repro Steps

1. SSH to a remote VPS project without `.focusa-project.json` (e.g., `/home/planmarr/plan-the-marriage` on `67.222.16.22:2200`)
2. Run any Focusa tool that requires project scope: `focusa_project_identity`, `focusa_trajectory_define_goal`
3. Tools succeed but project is `cwd_only` / low-confidence
4. Focus State tools (current_focus, next_step, etc.) **reject with validation error**
5. Every session restart requires re-verification

## Impact

- **High**: All Focus State tools unusable for remote/VPS projects without existing markers
- **Medium**: Every session requires manual scope verification
- **Medium**: Project trajectory/workpoint state does not persist across sessions for remote projects

## Suggested Fix

Add remote write capability to `discover_identity()` or add a new `focusa_project_bootstrap` tool that:
1. Takes remote_host, remote_port, remote_user as inputs
2. SSHs to the remote host
3. Creates `.focusa-project.json` with inferred/operator-supplied fields
4. Also creates `.beads/` directory

Alternatively, document a workaround: "For remote projects, manually create `.focusa-project.json` first before using Focusa tools."

## Environment

- Focusa version: latest
- Remote project: VPS via SSH (root@67.222.16.22:2200)
- Project root: /home/planmarr/plan-the-marriage
- No `.focusa-project.json` present