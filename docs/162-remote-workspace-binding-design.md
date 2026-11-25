# Remote Workspace Binding — Controller-Daemon Multiplexing Design (#89)

**Status:** design spec (IR1) — implementation pending an executable patch packet per #294.
**Issue:** Startempire-Wire/focusa#89

## Problem

Canonical bootstrap/Genesis/Workpoint authority assumes `project_root` exists
on the daemon host. One controller daemon cannot manage SSH/VPS checkouts or
multiple remote worktrees without installing a daemon per target host. The
PTM reproduction: a verified remote project's HLT persists while bootstrap
is local-only and the first Workpoint enters a writer-lease cycle.

## Design: `RemoteWorkspaceBinding`

A controller-owned typed binding that multiplexes local projects, SSH/VPS
projects, repositories, worktrees, and team sessions under one daemon.

```json
{
  "schema": "focusa.remote_workspace_binding.v1",
  "binding_id": "uuid",
  "controller": {
    "daemon_identity": "anchor-server",
    "controller_origin": "agent-kb:host-philoveracity-com"
  },
  "project": {
    "project_id": "plan-the-marriage",
    "repo_remote": "git@github.com:planmarr/plan-the-marriage.git"
  },
  "transport": {
    "kind": "ssh",
    "host": "100.x.y.z",
    "user": "planmarr",
    "port": 22,
    "host_reference": "agent-kb:planmarr-vps",
    "verified_at": "2026-08-15T00:00:00Z",
    "verification_evidence": ["ssh_fingerprint", "agent-kb_host_record"]
  },
  "roots": {
    "canonical_remote_root": "/home/planmarr/plan-the-marriage",
    "deploy_root": "/home/planmarr/app.planmarr.dev",
    "working_subpath": null,
    "worktree_identity": null
  },
  "session": {
    "continuity_id": "ptm-main",
    "principal": "team:planmarr"
  },
  "state": {
    "status": "verified",
    "freshness": "2026-08-15T00:00:00Z",
    "revocation": null
  }
}
```

### Authority invariants

1. Identity never changes silently: `project_id` + `repo_remote` +
   `continuity_id` are immutable once verified; mutation requires a new
   binding with an explicit supersede record.
2. Transport verification is required before any remote write path:
   host/user/port from agent-kb (authority) or verified SSH fingerprint.
3. Controller-side canonical authority: Workpoint/Trajectory state remains
   on the controller daemon; the remote host is only a working surface
   (no daemon, no local authority).
4. Freshness + revocation: bindings age out of "verified" without a
   successful reachability probe within the freshness window; revocation is
   a typed state transition, never a delete.
5. Writer leases: lease acquisition consults the binding's verified state —
   no checkpoint-without-lease cycles because the binding itself satisfies
   the bootstrap precondition (replaces the fabricated-local-checkout
   workaround).

### Bootstrap and Genesis admission

The general binding lookup supports recovery and visibility. A root match or a
client-supplied `verified` status is not sufficient for onboarding. Before
starting Bootstrap or Genesis, the controller must select the exact binding ID
and independently verify the SSH target, project, worktree, principal and
continuity. Use the transport verification time, not the binding row's update
time, when checking freshness. Pending, stale, revoked, changed or mismatched
bindings cannot authorize effects. Recheck the binding at effect boundaries and
serialize against revocation or replacement for the same binding.

Keep Bootstrap and Genesis journals and canonical readiness on the controller.
Inspect the actual checkout and its existing task provider through bounded
owner-side operations. The adoption receipt records the verified Git directory;
Genesis rejects a changed linked checkout even when its path and origin match. Do not overwrite an existing task queue or require an
optional remote marker as proof of readiness. Local onboarding retains its
existing filesystem and ownership checks.

The controller exposes remote Bootstrap preview, adoption and receipt status,
and Genesis plan, confirmed commit and readback via `--remote-binding-id`
(CLI) or `remote_binding_id` (Pi/API). Include the exact checkout path and
continuity. Non-status operations require the original replay key where
applicable. Neither adapter guesses a remote binding or substitutes a local
workspace. Local markers and local Genesis are unchanged.

Remote registration alone is pending, not proof. In the controller daemon,
`FOCUSA_REMOTE_ONBOARDING_TARGETS` selects exact owner-approved binding
records (the `focusa.remote_workspace_binding.v1` schema, requested identity
and transport; no secret values or asserted verification). Missing or duplicate
IDs fail closed. An administrator can preview the registered checkout, then
explicitly confirm existing-provider adoption to record the actual controller
SSH probe and adoption receipt. The generic binding POST cannot mint a probe.
A re-apply with the original key refreshes an expired proof without creating
another receipt. The controller SSH configuration must already pin the target
host key and authorize the exact workspace owner.

Genesis stages a bounded frozen plan from the owner-verified queue. Confirmed
commit rechecks the exact linked checkout and first task, then persists its
binding-qualified HLT, Ladder, Trajectory, and first Workpoint with the
publication receipt in one SQLite writer transaction. The remote projection
key is not a filesystem path and is rejected by local selectors. Status and
resume require the current probe and recheck the active records. Staging,
publication, and execution are distinct: publication is not an attached
session, evidence-write grant, or writer lease. Attach and verify the exact
remote workstream before any automated action; do not substitute a local
marker or a root-only consumer for that proof.

### Execution slices (IR2+)

1. Core type + persistence (`remote_workspace_bindings` table + CRUD route).
2. SSH transport probe (host reachability, fingerprint verification,
   bounded commands via the controller).
3. Bind remote checkout to controller scope (identity without daemon).
4. Writer-lease + checkpoint bootstrap through the binding.
5. CLI: `focusa remote bind --host ... --user ... --repo ...` +
   `focusa remote status` + revocation.
6. Pi/skill surfaces: project-switch ledger consumes bindings.

### Acceptance sketch

- PTM scenario: verify remote project → HLT persists → first Workpoint
  checkpoints without a lease cycle, all from the controller.
- No daemon installed on the remote host.
- Revocation blocks remote write paths within one freshness window.
- Typed receipts for bind/verify/revoke with evidence refs.

### Non-goals

- No remote daemon installation.
- No unverified SSH credential storage (agent-kb owns host records;
  bindings reference them).
