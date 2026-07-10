# Spec 131 — Focusa Workpoint Item Timing, Velocity, and Closure Authority

## Status

Draft — operator-directed core Focusa spec. No implementation code changes yet.

Canonical label: Spec 131 Workpoint Item Timing and Velocity  
Depends on: Spec 88, Spec 96, Spec 98, Spec 100, Spec 101, Spec 104, Spec 119, Spec 125, Spec 130  
Primary implementation surfaces: Focusa core, API, CLI, Pi extension, Workpoint, beads/tasks, Trajectory, CompactionMissionPacket, Bloatgaurd, Evidence/ECS, Receipts, Closure Authority, project cards, tests, future SaaS timeline.

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
9. Preserve Workpoint Item timing and closure state across Spec 130 compaction, model switches, forks, handoffs, and provider overflow.
10. Include HLT posture, receipt posture, and Bloatgaurd omission/rehydration data in closure and velocity records.

## Non-goals

- No surveillance or keystroke tracking.
- No attempt to bill customers by raw token count in this spec.
- No rewriting historical timing records in place.
- No closure automation that overrides operator authority.
- No visual SaaS timeline implementation in this spec; this spec defines the data substrate.
- No replacement of Spec 130 compaction authority rules; this spec consumes Spec 130 packets and adds item-level measurement/closure data.

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

## Relationship to Spec 130

Spec 130 defines the HLT-aware Compaction Mission Packet and Bloatgaurd Context Firewall. Spec 131 extends that architecture by making Workpoint Items measurable and closeable across compaction boundaries.

Spec 131 imports these Spec 130 rules:

```text
Compaction packets are not authority.
Transcript tails are not authority.
Generic HLT is not authority.
HLT posture must be visible before durable work.
Evidence and receipt expectations must survive compaction.
Raw bulky context belongs behind ECS/Evidence handles.
Closure claims require receipt/evidence posture.
```

Spec 131 adds this item-level invariant:

```text
A Workpoint Item cannot be measured, completed, closed, or used for velocity unless its timing, token usage, evidence refs, HLT posture, receipt posture, and closure authority survive compaction or are explicitly marked degraded.
```

### Spec 130 data consumed by Spec 131

Workpoint Item and closure records must be able to reference:

- `CompactionMissionPacket.packet_id`;
- `TrajectoryResumePacketV3.packet_id`;
- `WorkpointResumePacketV2.workpoint_id`;
- `HLT_STATUS`;
- `GENERIC_BOOTSTRAP`;
- `FALLBACK_SOURCE`;
- Bloatgaurd omitted-context receipt;
- ECS/Evidence rehydrate refs;
- active blocker excerpt/rehydrate handle;
- receipt expectation;
- closure authority result.

### Durable-work gate

Workpoint Item closure is durable work. It must obey Spec 130 durable-work rules:

```text
HLT_STATUS=canonical_explicit
OR HLT_STATUS=previous_valid_fallback with refreshed session-specific state
OR explicit degraded-mode receipt posture
OR operator override with recorded reason where allowed.
```

Generic HLT can never become canonical closure authority through override alone.

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
  "hlt_status": "canonical_explicit|previous_valid_fallback|supersession_pending|missing_required|generic_degraded|conflicted",
  "trajectory_packet_ref": null,
  "compaction_packet_ref": null,
  "receipt_refs": [],
  "bloatgaurd": {
    "omitted_context_refs": [],
    "rehydrate_refs": [],
    "raw_context_externalized": false
  },
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
  "compaction_elapsed_ms": 0,
  "handoff_elapsed_ms": 0,
  "compaction_count": 0,
  "scope_switch_count": 0,
  "reason": "...",
  "compaction_packet_ref": null,
  "trajectory_hlt_status": null
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
  "closure_status": "authorized|blocked|premature|operator_override|degraded_allowed",
  "hlt_status": "canonical_explicit|previous_valid_fallback|supersession_pending|missing_required|generic_degraded|conflicted",
  "receipt_posture": "canonical|advisory|degraded|blocked|stale",
  "compaction_packet_ref": null,
  "trajectory_packet_ref": null,
  "bloatgaurd_rehydrate_refs": [],
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
- Closure checks must distinguish `blocked`, `premature`, `authorized`, `degraded_allowed`, and `operator_override`.
- Closure checks must include HLT posture from Spec 125/130.
- Closure checks must include receipt posture from Spec 119/130.
- Closure checks must preserve Bloatgaurd rehydrate refs when proof/evidence context was omitted from the hot prompt.

## Compaction and resume requirements

Workpoint Item state must survive Spec 130 compaction. Compaction may elide raw context, but it must preserve item ids, status, timing rollups, HLT posture, closure posture, active blockers, and rehydrate refs.

When `CompactionMissionPacket.status=degraded|blocked`, Workpoint Items may remain active, but closure is blocked unless degraded receipt posture or explicit operator override is recorded.

Workpoint resume packets must expose item state:

- active item;
- blocked items;
- completed but unclosed items;
- next queued items;
- elapsed active time;
- wall-clock time;
- token usage;
- closure authority status;
- missing evidence/checks;
- associated `CompactionMissionPacket` refs;
- HLT status and receipt posture affecting closure.

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
- Workpoint lineage;
- CompactionMissionPacket boundaries;
- HLT warning intervals;
- Bloatgaurd omitted-context/rehydration events;
- receipt/closure-gate transitions.

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
- `work-compaction-links/{project_hash}/item-compaction-links.jsonl`

All records must include:

- `project_root`;
- `continuity_id`;
- `session_id` when available;
- `workpoint_id`;
- `item_id` where applicable;
- source route/tool;
- created timestamp;
- schema version;
- `hlt_status` when closure or durable work is involved;
- `compaction_packet_ref` when record was created before/after compaction;
- Bloatgaurd/ECS rehydrate refs for omitted proof/context.

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
16. Workpoint Item closure is blocked when HLT status is missing_required, generic_degraded, conflicted, or stale unless degraded receipt posture or explicit operator override is recorded.
17. Workpoint Item state survives Spec 130 compaction with item id, timing rollup, HLT posture, active blocker, evidence refs, and rehydrate refs intact.
18. Bloatgaurd-elided proof/tool context remains reachable by rehydrate refs from item and closure records.
19. Velocity summaries exclude items whose closure authority is blocked, premature, or missing receipt posture.
20. Future timeline projection can display compaction boundaries, HLT warnings, omitted-context handles, and closure transitions for each Workpoint Item.

## Implementation slices

### Slice 1 — Audit and schema scaffold

- Audit current Workpoint/task/timing/token fields.
- Audit Spec 130 CompactionMissionPacket, TrajectoryResumePacketV3, Bloatgaurd, ECS/Evidence, and receipt surfaces that must link to Workpoint Items.
- Add Workpoint Item schema.
- Add timing/token/closure ledger schemas.
- Add compaction-link ledger schema.
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
- Include HLT posture and receipt posture in every closure check.
- Block closure when Spec 130 marks resume degraded/blocked unless degraded receipt posture or operator override permits it.
- Record operator override explicitly.

### Slice 5 — Velocity intelligence

- Compute item/task/spec velocity summaries.
- Feed project-card estimates with real timing history.
- Track estimate accuracy.

### Slice 6 — SaaS timeline readiness

- Add timeline-ready projection API.
- Include Workpoint lineage, item phases, timing, proof, tokens, and closure authority.
- Include Spec 130 compaction boundaries, HLT warning intervals, omitted-context handles, rehydrate refs, and receipt posture transitions.
- Keep rendering/UI out of this spec unless later superseded.
