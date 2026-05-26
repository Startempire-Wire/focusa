# Autonomic Coding Workflow Governor

Status: proposed next layer. Scope: coding-agent workflow regulation, not a replacement for Workpoint, Trajectory, Work Loop, Reflex, or operator steering.

## Purpose

The Autonomic Coding Workflow Governor is a Focusa-native control loop for continuous coding agents. It reads a normalized project-vitals card, detects stuckness/risk/pressure, then recommends or triggers bounded workflow regulation: checkpoint, compact, narrow scope, retry differently, gather evidence, pause, or escalate.

It answers: **"Given the current coding physiology, what should the agent do next to stay productive and safe?"**

## Non-goals

- It does not choose the project goal; Trajectory remains the north star.
- It does not mutate canonical task state without existing Workpoint/Work Loop authority.
- It does not run destructive commands or service restarts by itself.
- It does not replace tests, CI, Beads, or git; it summarizes their signals.
- It does not use raw transcript tail as authority.

## Inputs: project vitals card

`project_vitals` should be a hot-path bounded card scoped by `project_root + continuity_id`.

```json
{
  "schema": "focusa.project_vitals.v1",
  "project_root": "/repo",
  "continuity_id": "focusa-cont-...",
  "freshness": { "generated_at": "...", "stale": false },
  "git": {
    "branch": "main",
    "dirty_files": 3,
    "untracked_files": 1,
    "last_commit": "abc123",
    "risk": "clean|dirty|large_diff|unknown"
  },
  "checks": {
    "last_test": "pass|fail|timeout|not_run|unknown",
    "last_lint": "pass|fail|timeout|not_run|unknown",
    "last_typecheck": "pass|fail|timeout|not_run|unknown",
    "ci": "pass|fail|pending|not_available|unknown"
  },
  "beads": {
    "active": "focusa-...",
    "blocked_by": [],
    "ready_count": 2
  },
  "workpoint": {
    "canonical": true,
    "current_action": "...",
    "next_action": "...",
    "last_checkpoint_age_minutes": 12
  },
  "trajectory": {
    "posture": "verify_first|execute|plan|blocked|unknown",
    "gap": "bounded summary"
  },
  "daemon": {
    "health": "ok|degraded|down|unknown",
    "rss_kb": 944588,
    "hot_path_warnings": 1,
    "registry_drift": false
  },
  "agent_loop": {
    "turns_since_checkpoint": 8,
    "repeated_commands": 0,
    "same_file_churn": 2,
    "failure_streak": 0,
    "compaction_pressure": "low|medium|high"
  }
}
```

## Detectors

### 1. Stuck detector

Signals:

- Same command or same failing test repeated ≥ 2 without strategy change.
- Same file edited/reverted repeatedly.
- Tool fallback loop: same failure class and same next tool repeated.
- Investigation loop: many reads/searches with no new evidence or code/docs change.
- Workpoint next action unchanged across multiple checkpoints.

Output:

```json
{
  "stuckness": "none|watch|stuck|hard_stuck",
  "reasons": ["repeated_command_failure"],
  "recommended_shift": "narrow_scope|switch_tool|write_checkpoint|ask_operator|create_blocker_bead"
}
```

### 2. Safety immune check

Signals:

- Destructive operations, service restarts, permission changes, secrets, `/home/*` root writes, broad git resets.
- Cross-project identity mismatch.
- Dirty git state with high-risk command.
- Production daemon restart or deploy action.

Output:

```json
{
  "immune_state": "clear|caution|blocked_requires_approval",
  "approval_required": true,
  "reason": "service_restart"
}
```

### 3. Resource/homeostasis detector

Signals:

- Daemon RSS/CPU above thresholds.
- Hot-path latency warnings/timeouts.
- Token/context pressure.
- Too many turns since checkpoint.
- Live/static registry drift after code changes.

Output actions favor consolidation and proof isolation before more work.

### 4. Evidence/proof detector

Signals:

- Code changed without tests.
- Docs changed without link/static checks.
- Runtime path changed without smoke proof.
- Feature marked complete while blocker/warning remains.

Output actions favor targeted gates and evidence capture.

## Governor decision output

`coding_governor` returns advisory regulation, with optional Work Loop action only when writer authority and safety allow it.

```json
{
  "schema": "focusa.coding_governor.v1",
  "status": "completed|blocked|degraded",
  "canonical": false,
  "advisory_only": true,
  "decision": "continue|checkpoint|compact|narrow|retry_differently|capture_evidence|create_blocker|pause|approval_required",
  "why": ["turns_since_checkpoint=8", "dirty_files=3"],
  "recommended_tools": ["focusa_workpoint_checkpoint", "focusa_evidence_capture"],
  "blocked_by": [],
  "next_action": "Run focused test, capture evidence, then checkpoint.",
  "do_not_do": ["restart service without approval"],
  "confidence": "low|medium|high"
}
```

## Regulation policy

| Condition | Decision | Notes |
| --- | --- | --- |
| No active Workpoint | `checkpoint` | Create/resume canonical Workpoint before major edits. |
| High compaction pressure | `compact` | Workpoint checkpoint first; trajectory checkpoint if goal/gap matters. |
| Same failure twice | `retry_differently` | Change strategy/tool/input; capture failure class. |
| Same failure three times | `create_blocker` | Stop looping; record blocker bead/evidence. |
| Dirty diff + no test proof | `capture_evidence` | Run minimal relevant gate. |
| Service restart/destructive op | `approval_required` | Never auto-execute. |
| Daemon hot-path/resource warning | `narrow` | Prefer static/local proof; avoid broad live stress. |
| Goal/gap unclear | `trajectory_view` | Advisory orientation before acting. |
| Large diff or many turns | `checkpoint` | Preserve continuation and shrink context. |

## Integration points

### API

Proposed routes:

- `GET /v1/project/vitals?project_root=...&continuity_id=...`
- `POST /v1/coding-governor/assess`
- `POST /v1/coding-governor/checkpoint-policy`

### CLI

Proposed commands:

- `focusa project vitals --project-root . --continuity-id ... --json`
- `focusa coding-governor assess --project-root . --continuity-id ... --json`

### Pi tools

Proposed tools:

- `focusa_project_vitals`
- `focusa_coding_governor_assess`

### Focus Slice

Add bounded advisory section:

```text
CODING_GOVERNOR:
  decision: checkpoint
  why: dirty_files=4, turns_since_checkpoint=9
  next: run focused gate then checkpoint
  approval_required: false
```

## Relationship to existing Focusa systems

- **Trajectory**: supplies desired state and gap; governor does not redefine goals.
- **Workpoint**: supplies canonical continuation; governor recommends checkpoint/resume/evidence actions.
- **Work Loop**: supplies writer ownership; governor may propose pause/resume/select-next only through Work Loop authority.
- **Reflex primitives**: supply recovery affordances; governor decides when a reflex class should fire.
- **Prediction/metacog**: learns which regulation decisions improved outcomes.
- **Resource mode**: informs pressure and LowMem choices.
- **State hygiene**: handles stale signals; governor can recommend hygiene plan, not destructive cleanup.

## MVP build order

1. `project_vitals` read-only card from git status, Beads, Workpoint, Trajectory, resource, telemetry, and last known checks.
2. Static stuck detector over recent tool/failure summaries, not raw transcript.
3. `coding_governor_assess` advisory route with deterministic policy table.
4. Pi tool + Focus Slice section for advisory decision and do-not-do list.
5. Evidence capture integration: record gates and regulation outcomes.
6. Prediction/metacog loop: predict regulation success, evaluate actual outcome, promote useful policies.
7. Menubar cockpit card for vitals/governor status.

## Acceptance criteria

- Hot-path project vitals returns in <500ms for normal repo state or degrades with `failure_class` and next tools.
- Governor never recommends unsafe execution without approval when safety immune check is blocked.
- Repeated failure loop produces `retry_differently` by second repeat and `create_blocker` by third repeat.
- Dirty code plus no proof recommends a minimal targeted gate before completion.
- Context pressure recommends Workpoint checkpoint before compaction.
- Runtime/static registry drift recommends daemon rebuild/restart only as approval-required action.
- All outputs include `tool_result_v1` and bounded `next_tools` when implemented as tools.

## Open implementation questions

- Where should last test/lint/typecheck evidence be stored: Workpoint evidence refs, telemetry, or a small project-vitals cache?
- How should same-file churn be detected without reading full git diff or transcript tail?
- Which thresholds should be learned from prediction outcomes versus fixed by policy?
- Should `project_vitals` be part of ProjectIdentity, Workpoint resume, or a separate route?
