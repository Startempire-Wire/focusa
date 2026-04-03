# Focusa Meta-Cognition

Use this skill when managing complex multi-step tasks or needing to preserve decisions across compaction.

Focusa is a cognitive runtime that tracks focus, decisions, constraints, and failures across sessions.

## Available Tools

When Focusa is active, you have these tools:
- `focusa_decide` — Record a significant decision (architecture, approach, library choice)
- `focusa_constraint` — Record a constraint (limitation, requirement, hard rule)
- `focusa_failure` — Record a failure (build error, test failure, wrong assumption)

## When to Use

- **focusa_decide**: Choosing between approaches, selecting libraries, architectural choices, committing to a direction
- **focusa_constraint**: Discovering file size limits, API limitations, compatibility requirements, performance bounds
- **focusa_failure**: Build errors, test failures, wrong assumptions, operator corrections

## Commands

- `/focusa-status` — Show connection status, frame, decisions/constraints/failures counts
- `/focusa-stack` — Show Focus Stack frames
- `/focusa-checkpoint` — Create ASCC checkpoint (snapshot cognitive state)
- `/focusa-rehydrate <handle>` — Retrieve externalized content from ECS
- `/focusa-explain-decision [query]` — Search recorded decisions
- `/focusa-lineage` — Show CLT lineage path
- `/wbm on` — Enable Wirebot Mode (cross-surface identity bridge)

## Check Status

```bash
curl -s http://127.0.0.1:8787/v1/focus/stack | jq .
```

## Record a Decision

```bash
curl -X POST http://127.0.0.1:8787/v1/focus/update \
  -H 'Content-Type: application/json' \
  -d '{"decisions": ["Chose X because Y"]}'
```

## Rules

- Decisions survive compaction — they live in Focus State, not conversation
- Check constraints before acting — do not violate recorded constraints
- Do not contradict prior decisions without explanation
- After compaction, Focus State is your source of truth
