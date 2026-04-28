# Focusa Meta-Cognition

Use this skill when managing complex multi-step tasks, preserving decisions across compaction, or resuming work after context overflow/model switch/fork.

Focusa is the cognitive runtime that tracks focus, decisions, constraints, failures, and typed Workpoint continuity across sessions.

## Available Tools

When Focusa is active, use these tools:

- `focusa_scratch` — working notes only; not Focus State.
- `focusa_decide` — record a significant architectural decision with why.
- `focusa_constraint` — record a discovered hard requirement or boundary.
- `focusa_failure` — record a specific failure diagnosis and recovery.
- `focusa_workpoint_checkpoint` — checkpoint typed continuation before compact/resume/overflow/model switch/fork/risky work.
- `focusa_workpoint_resume` — fetch the active `WorkpointResumePacket` before continuing after compact/resume/overflow/model switch/fork.

## Workpoint Continuity Rules

1. Meaning lives in the typed Workpoint, not in raw transcript tail.
2. After compaction or uncertainty, call `focusa_workpoint_resume` before choosing the next task.
3. Before risky discontinuities, call `focusa_workpoint_checkpoint` with mission, current action, evidence, blockers, and exact next action.
4. Treat `canonical: false` / `focusa-workpoint-fallback` as degraded recovery hint only; do not silently promote it.
5. Drift warnings mean the latest action may not match the active Workpoint action intent; resume the packet’s `next_slice` unless the operator steers otherwise.

## Commands

- `/focusa-status` — Show connection status, frame, decisions/constraints/failures counts
- `/focusa-stack` — Show Focus Stack frames
- `/focusa-checkpoint` — Create ASCC checkpoint
- `/focusa-rehydrate <handle>` — Retrieve externalized content from ECS
- `/focusa-explain-decision [query]` — Search recorded decisions
- `/focusa-lineage` — Show CLT lineage path
- `/wbm on` — Enable Wirebot Mode

## Operator Examples

Checkpoint before compact or fork:

```text
focusa_workpoint_checkpoint current_ask="continue Spec88 rollout" checkpoint_reason="before_compact" next_action="run rollout gate and close bead" canonical=true
```

Resume after overflow or compaction:

```text
focusa_workpoint_resume mode="compact_prompt"
```

Degraded fallback display:

```text
NON-CANONICAL WORKPOINT FALLBACK: resume from local bounded next action
```

Safe recovery command:

```bash
curl -s http://127.0.0.1:8787/v1/workpoint/current | jq .
```

## Rules

- Decisions survive compaction because they live in Focusa, not conversation.
- Workpoint continuity survives compaction because it lives in reducer-owned typed state, not prompt history.
- Check constraints before acting; do not violate recorded constraints.
- Do not contradict prior decisions without explicit operator steering or new evidence.

## Spec89 Tool Result Envelope

Every `focusa_*` Pi tool preserves its visible text summary and adds `details.tool_result_v1` with common fields: `ok`, `status`, `canonical`, `degraded`, `summary`, `retry`, `side_effects`, `evidence_refs`, `next_tools`, `error`, and `raw`.

Use `status`, `retry.posture`, and `next_tools` for recovery decisions instead of parsing prose. Treat `canonical=false` or `degraded=true` as non-authoritative fallback unless a later canonical Focusa read confirms it.
