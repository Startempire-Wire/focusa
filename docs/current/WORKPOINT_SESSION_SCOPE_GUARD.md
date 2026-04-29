# Workpoint Session Scope Guard

Focusa Workpoint resume packets are scoped to the project/session that created them.

## Problem prevented

A compaction/resume packet from another project/session must not become canonical continuation in the current session. If the operator is working in `asapdigest`, a Focusa repo packet must be rejected instead of injected.

## Current behavior

- Workpoint checkpoints include `session_id` and `project_root`.
- Pi extension checkpoint/resume calls send the current Pi `session_id` and `project_root`.
- `/v1/workpoint/resume` rejects mismatched packets with `status: rejected_scope_mismatch`.
- Pi clears the active packet and tells the agent to follow the latest operator instruction/current repo.

## Recovery

```text
Ignore the rejected packet. Follow latest operator instruction and local git/beads for the current project.
```

Then create a new scoped checkpoint:

```text
focusa_workpoint_checkpoint mission="..." next_action="..." checkpoint_reason="manual"
```

## Tests

```bash
cargo test -p focusa-api workpoint
```
