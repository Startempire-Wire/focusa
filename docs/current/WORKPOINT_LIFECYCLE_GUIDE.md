# Workpoint Lifecycle Guide

Workpoints are the current build's typed continuation contract for Focusa/Pi continuity.

## Use Workpoints when

- Before compaction, model switch, context overflow, fork, or risky handoff.
- After compaction/resume/overflow when transcript tail may be unreliable.
- When linking release proof, test results, API evidence, or file evidence to active work.
- When drift needs to be checked against an expected action/object.

## Lifecycle

1. **Checkpoint** with `focusa_workpoint_checkpoint` or `POST /v1/workpoint/checkpoint`.
2. **Promote/current**: current API waits for reducer-visible active state before returning accepted in the current build.
3. **Resume** with `focusa_workpoint_resume` or `POST /v1/workpoint/resume`.
4. **Link evidence** with `focusa_workpoint_link_evidence` or `POST /v1/workpoint/evidence/link`.
5. **Check drift** with `focusa workpoint drift-check` or `POST /v1/workpoint/drift-check`.

## Canonical/degraded rules

- `canonical=true`: authoritative current Focusa state.
- `canonical=false` or degraded fallback: bounded recovery hint only.
- `pending`: accepted into command path but not yet safe to rely on; retry current/resume.

## Minimal CLI flow

```bash
focusa workpoint current
focusa workpoint resume
focusa workpoint drift-check --latest-action 'release verify Spec89FocusaToolSuite live_api cli pi_tool' --expected-action-type release_verify
```
