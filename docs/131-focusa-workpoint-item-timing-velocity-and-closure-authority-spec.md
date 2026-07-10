# Spec 131 — Focusa Workpoint Item Timing, Velocity, and Closure Authority

## Status

Draft — operator-directed core Focusa spec. No implementation code changes yet.

## Problem

Focusa has partial elapsed-time and token accounting, but it is not yet accurate enough to measure real implementation speed. Current timing can reset on operator turns and is not reliably bound to Workpoints, beads/tasks, closures, proof, pauses, compactions, handoffs, or scope switches.

Focusa needs first-class timing, token, velocity, and closure authority data so estimates are grounded in completed work instead of heuristics.

## Goals

1. Accurately measure Workpoint and task execution time.
2. Separate wall-clock time from active agent work, blocked time, pause time, and operator-wait time.
3. Track tokens, tool calls, proof runs, commits, changed files, and closure evidence per unit of work.
4. Make Workpoint Items the smallest measurable execution unit.
5. Roll Workpoint Item metrics up into Workpoints, beads/tasks, specs, projects, and trajectories.
6. Add closure authority so work cannot be marked done without the required evidence, checks, and authorization.
7. Use measured history to improve future estimates, velocity reports, and project-card predictions.
8. Prepare the data model for a future SaaS visual timeline interface.

## Non-goals

- No surveillance or keystroke tracking.
- No attempt to bill customers by raw token count in this spec.
- No rewriting historical timing records in place.
- No closure automation that overrides operator authority.
- No visual SaaS timeline implementation in this spec; this spec defines the data substrate.

## Existing partial surfaces

Current code already has pieces that can be reused or extended:

- Pi task timing auto-populates `task_timing.elapsed_ms`, `elapsed_seconds`, and `elapsed_hms` during `focusa_project_card_outcome`.
- Project card outcome records store task timing and token usage.
- Project card summaries compute average elapsed time and token usage from recorded outcomes.
- CLI turn history exposes turn durations.

Current gaps:

- Timing is not consistently bound to a bead/task/Workpoint Item.
- Timing resets on operator input and can include unrelated scope changes.
- Pauses, blocked time, compaction, handoff, and operator-wait time are not separated.
- Closure does not consistently require item-level proof.
- Velocity is not computed from granular completed implementation units.

## Core model

### Rollup hierarchy

```text
WorkpointItem → Workpoint → Bead/Task → Spec → Project → Trajectory
```

A Workpoint Item is the smallest measurable unit of execution. Workpoints contain one or more items. Beads/tasks link to Workpoint Items. Specs and projects roll up completed item metrics.

## Workpoint Item

A Workpoint Item is an actionable, measurable slice of work such as audit, design, implementation, test, proof, docs, or closure.

```json
{
  "schema": "focusa.workpoint_item.v1",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "focusa-wefzg.2",
  "parent_item_id": null,
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "...",
  "session_id": "...",
  "spec_ref": "docs/128-...md#release-manifest",
  "phase": "audit|design|implement|test|proof|docs|closure",
  "title": "Implement release manifest validator",
  "target_objects": [],
  "acceptance_refs": [],
  "required_evidence_refs": [],
  "status": "queued|active|paused|blocked|done|closed",
  "closure_authority": "spec_acceptance|bead_done_condition|operator_override",
  "started_at": null,
  "last_active_at": null,
  "completed_at": null,
  "closed_at": null,
  "timing": {},
  "token_usage": {},
  "tool_usage": {},
  "evidence_refs": [],
  "blockers": [],
  "next_item_ids": []
}
```

## Work Timing Ledger

Timing records are append-only. Corrections append superseding records instead of rewriting history.

```json
{
  "schema": "focusa.work_timing.v1",
  "event_id": "...",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "focusa-wefzg.2",
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "...",
  "session_id": "...",
  "phase": "audit|design|implementation|test|proof|docs|review|closure",
  "event_type": "start|pause|resume|block|unblock|complete|close|correction",
  "started_at": "...",
  "ended_at": "...",
  "wall_clock_elapsed_ms": 0,
  "active_agent_elapsed_ms": 0,
  "paused_ms": 0,
  "operator_wait_ms": 0,
  "blocked_ms": 0,
  "proof_elapsed_ms": 0,
  "compaction_count": 0,
  "scope_switch_count": 0,
  "reason": "..."
}
```

### Timing categories

| Category | Meaning |
| --- | --- |
| `wall_clock_elapsed_ms` | real elapsed time from start to end |
| `active_agent_elapsed_ms` | time the agent was actively working |
| `paused_ms` | intentional pause time |
| `operator_wait_ms` | time waiting on operator input/approval |
| `blocked_ms` | time blocked by external dependency/failure |
| `proof_elapsed_ms` | time spent running validation/proof commands |

## Token and tool accounting

Tokens and tool calls are tracked per Workpoint Item and rolled up.

```json
{
  "schema": "focusa.work_token_usage.v1",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "...",
  "provider_input_tokens": 0,
  "provider_output_tokens": 0,
  "estimated_input_tokens": 0,
  "estimated_output_tokens": 0,
  "total_tokens": 0,
  "tool_call_count": 0,
  "tool_calls_by_family": {
    "bash": 0,
    "read": 0,
    "edit": 0,
    "write": 0,
    "focusa": 0,
    "uiai": 0,
    "web": 0
  }
}
```

## Closure Authority

Closure authority determines whether a Workpoint Item, Workpoint, bead/task, or spec can be marked done.

```json
{
  "schema": "focusa.closure_authority.v1",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "...",
  "closure_requested_by": "agent|operator|daemon|work_loop",
  "closure_authority": "operator|bead_done_condition|spec_acceptance|workpoint_contract",
  "required_evidence_refs": [],
  "provided_evidence_refs": [],
  "required_checks": [],
  "passed_checks": [],
  "closure_status": "authorized|blocked|premature|operator_override",
  "reason": "...",
  "checked_at": "..."
}
```

Rules:

- Workpoint Items cannot close without required evidence or explicit operator override.
- Workpoints cannot close until required Workpoint Items close.
- Beads/tasks cannot close until linked Workpoint Items satisfy done conditions.
- Specs cannot close until required beads/tasks and Workpoint Items have proof.
- Operator override must be explicit, visible, and auditable.
- Closure checks must distinguish `blocked`, `premature`, `authorized`, and `operator_override`.

## Workpoint resume requirements

Workpoint resume packets must expose item state:

- active item;
- blocked items;
- completed but unclosed items;
- next queued items;
- elapsed active time;
- wall-clock time;
- token usage;
- closure authority status;
- missing evidence/checks.

## Velocity metrics

Velocity must be computed from completed Workpoint Items first, then rolled up.

```json
{
  "schema": "focusa.velocity_summary.v1",
  "project_root": "/home/wirebot/focusa",
  "task_family": "spec128-update-system",
  "completed_items": 0,
  "completed_workpoints": 0,
  "completed_tasks": 0,
  "average_active_elapsed_ms": 0,
  "average_wall_clock_elapsed_ms": 0,
  "average_total_tokens": 0,
  "average_tool_calls": 0,
  "proof_failure_rate": 0.0,
  "rollback_rate": 0.0,
  "reopen_rate": 0.0,
  "estimate_accuracy": 0.0
}
```

Useful reports:

- average time per Workpoint Item phase;
- average time per task family;
- average time per spec section;
- proof/test failure rate;
- token burn per successful closure;
- implementation throughput per day/session/week;
- estimate accuracy by task type;
- common blockers and pause reasons.

## Future SaaS timeline interface

This spec prepares data for a visual timeline:

```text
Project
 └─ Spec
    └─ Bead / Task
       └─ Workpoint
          └─ Workpoint Items
             ├─ audit
             ├─ design
             ├─ implementation
             ├─ tests
             ├─ proof
             └─ closure
```

Timeline cards should show:

- elapsed active vs blocked time;
- token burn;
- tool calls;
- files changed;
- commits;
- tests/proofs;
- closure authority;
- agent handoffs;
- compactions/session resumes;
- predictions vs actual outcomes;
- Workpoint lineage.

Future user questions the SaaS timeline should answer:

- Where did this task stall?
- Which item burned most tokens?
- What proof closed this work?
- Was this estimate accurate?
- Which agent/session did what?
- What changed between Workpoints?
- Which specs consume the most implementation time?

## CLI surface

```bash
focusa workpoint item create --workpoint <id> --task <bead> --title "..."
focusa workpoint item list --workpoint <id> --json
focusa workpoint item start <item-id>
focusa workpoint item pause <item-id> --reason blocked
focusa workpoint item resume <item-id>
focusa workpoint item complete <item-id> --evidence <ref>
focusa workpoint item close-check <item-id> --json

focusa work timing status --workpoint <id> --json
focusa work timing status --task <bead> --json
focusa work velocity --project /home/wirebot/focusa --json
focusa task closure check <task-id> --json
```

## API surface

- `POST /v1/workpoint/item/create`
- `GET /v1/workpoint/items`
- `POST /v1/workpoint/item/start`
- `POST /v1/workpoint/item/pause`
- `POST /v1/workpoint/item/resume`
- `POST /v1/workpoint/item/complete`
- `POST /v1/workpoint/item/close-check`
- `GET /v1/work/timing/status`
- `GET /v1/work/velocity`
- `POST /v1/task/closure/check`

## Storage

Recommended append-only ledgers:

- `workpoint-items/{project_hash}/items.jsonl`
- `work-timing/{project_hash}/timing-events.jsonl`
- `work-token-usage/{project_hash}/token-events.jsonl`
- `closure-authority/{project_hash}/closure-checks.jsonl`
- `velocity-summaries/{project_hash}/summaries.jsonl`

All records must include:

- `project_root`;
- `continuity_id`;
- `session_id` when available;
- `workpoint_id`;
- `item_id` where applicable;
- source route/tool;
- created timestamp;
- schema version.

## Acceptance criteria

1. Workpoint Item records are append-only and project-scoped.
2. Workpoint Items can be created, listed, started, paused, resumed, completed, and close-checked.
3. Timing survives compaction/session resume.
4. Timing is bound to project_root + continuity_id + workpoint_id + item_id + task_id.
5. Active time, wall-clock time, blocked time, pause time, and operator-wait time are separated.
6. Token/tool usage aggregates across turns for a Workpoint Item.
7. Workpoint resume packets show active/blocked/next items and missing closure proof.
8. Bead/task closure checks inspect linked Workpoint Items.
9. Closure checks block unsupported completion unless explicit operator override exists.
10. Operator override is recorded with reason and evidence context.
11. Velocity summaries use completed Workpoint Items only.
12. Project-card estimates can use real item/task velocity history.
13. End-of-task reports include elapsed, tokens, item rollup, checks, closure authority, and velocity update.
14. No timing record is silently rewritten; corrections append superseding records.
15. Static and runtime tests cover compaction/resume, item timing, token rollup, closure blocking, operator override, and velocity rollup.

## Implementation slices

### Slice 1 — Audit and schema scaffold

- Audit current Workpoint/task/timing/token fields.
- Add Workpoint Item schema.
- Add timing/token/closure ledger schemas.
- Add static tests for schema fields.

### Slice 2 — Workpoint Item API/CLI

- Add create/list/start/pause/resume/complete/close-check routes.
- Add CLI wrappers.
- Bind items to Workpoints and tasks/beads.

### Slice 3 — Timing and token rollup

- Track elapsed categories per item.
- Aggregate item → Workpoint → bead/task.
- Add token/tool usage rollups.

### Slice 4 — Closure authority enforcement

- Add item-level closure checks.
- Add Workpoint closure checks.
- Add bead/task closure checks.
- Record operator override explicitly.

### Slice 5 — Velocity intelligence

- Compute item/task/spec velocity summaries.
- Feed project-card estimates with real timing history.
- Track estimate accuracy.

### Slice 6 — SaaS timeline readiness

- Add timeline-ready projection API.
- Include Workpoint lineage, item phases, timing, proof, tokens, and closure authority.
- Keep rendering/UI out of this spec unless later superseded.
