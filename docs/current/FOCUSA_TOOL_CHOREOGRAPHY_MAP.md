# Focusa Tool Choreography Map

Current contract registry: **58 Focusa tools** across project identity, trajectory, Workpoint, evidence, traversal, Focus State, work-loop, diagnostics, lineage, prediction, and metacognition.

Machine-readable registry: [`focusa-tool-choreography.json`](focusa-tool-choreography.json), also embedded at `GET /v1/ontology/tool-choreography`. It contains five route templates, 174 weighted per-tool edges, and exact `per_tool_next_tools` for all 58 tools. The live API also exposes `runtime_weight_adjustments` from evaluated predictions that cite `tool_edge:from->to`, so route ordering can improve from measured outcomes without overriding operator steering or safety gates.

This map links tools by **model intent** so agents get compounding project results instead of using only basic note tools.


## No-deadend failure recovery

Every Focusa tool result should be read as a small recovery contract, not just prose:

```text
failure_class → why it failed
retry.posture → whether retry is safe
recovery_hint / misuse_hint → what to fix first
next_tools → the next safe route
```

Common out-of-order fixes:

| Symptom | Likely cause | Safe next move |
|---|---|---|
| `scope_mismatch` | Broad cwd, cross-project packet, stale continuity | `focusa_project_identity` / `verify` → `focusa_workpoint_checkpoint` → `resume` |
| `frame_unavailable` | Focus State slot used before active Pi frame | Stay `Attentive and awaiting operator direction`; checkpoint/resume before durable writes |
| `validation_rejected` | Verbose/task/debug text in durable slot | Put full text in `focusa_scratch`; retry one compact declarative slot |
| `read_model_lag` | Just-written packet not visible yet | Wait/read once with same scope; avoid duplicate writes |
| `hot_path_timeout` | Daemon/resource pressure on bounded route | `focusa_tool_doctor` → `focusa_resource_mode`; avoid full/cold payloads |
| `unknown_ambiguous_completion` | Result does not prove side effect | Check canonical state/side effects before retrying |

Rule: if a tool blocks, do not stop at the error. Follow `next_tools` unless the operator steers otherwise.

## Route graph

### 1) Orient the project

```text
focusa_project_identity
  → focusa_project_verify (when expected root/name/remote matters)
  → focusa_traverse (when project infra/architecture facts are needed)
  → focusa_trajectory_view
  → focusa_workpoint_resume (if continuing) OR focusa_workpoint_checkpoint (if starting a slice)
```

Use when: project start, resume, compaction recovery, cross-project risk, stale transcript uncertainty, or before making architecture/infrastructure assumptions.

### 2) Set or repair the goal route

```text
focusa_trajectory_view
  → focusa_trajectory_define_goal (only when operator/new evidence changes goal)
  → focusa_trajectory_assess
  → focusa_trajectory_propose_workpoint
  → focusa_workpoint_checkpoint
```

Use when: goals/mission/destination are unclear, current state changed after proof, or Workpoint needs to be derived from project gap.

### 3) Execute a focused slice

```text
focusa_active_object_resolve
  → focusa_workpoint_checkpoint
  → tree/snapshot tools when rollback or lineage matters
  → do the work
  → focusa_evidence_capture OR focusa_workpoint_link_evidence
```

Use when: editing files, touching endpoints/components, testing, release proof, or preserving continuation before risky work.

### 4) Prove and update confidence

```text
focusa_evidence_capture / focusa_workpoint_link_evidence
  → focusa_trajectory_assess
  → focusa_recent_result OR focusa_decide/constraint/failure if durable
  → focusa_workpoint_checkpoint for the next slice
```

Use when: a test/API/file/result proves project state changed.

### 5) Predict, learn, and compound

```text
focusa_predict_record
  → execute risky/uncertain action
  → focusa_predict_evaluate
  → focusa_metacog_retrieve / focusa_metacog_capture
  → focusa_metacog_reflect
  → focusa_metacog_plan_adjust
  → focusa_metacog_evaluate_outcome
```

Use when: risk exists, tool choice is uncertain, release may fail, stale state may mislead, or a lesson should survive future sessions.

### 6) Traverse instead of guessing

```text
focusa_tool_doctor (if degraded/uncertain)
  → focusa_resource_mode (if hot/lowmem)
  → focusa_traverse (bounded surface slices)
  → lineage/tree/snapshot helpers when ancestry or rollback matters
  → focusa_workpoint_resume
```

Use when: context is large, daemon is degraded, Focus State is stale, or raw transcript memory is insufficient.

### 7) Continuous/background work

```text
focusa_work_loop_writer_status
  → focusa_work_loop_status
  → focusa_work_loop_context
  → focusa_work_loop_checkpoint
  → focusa_work_loop_select_next
  → focusa_silent_sessions (only for explicit background session management)
```

Use when: continuing autonomous work, coordinating writer ownership, selecting next ready work, or managing tmux-backed SilentSessions.

### 8) Hygiene and recovery

```text
focusa_tool_doctor
  → focusa_state_hygiene_doctor
  → focusa_state_hygiene_plan
  → focusa_state_hygiene_apply (approved=true only)
```

Use when: state is stale/duplicated/confusing. Apply never silently deletes; it requires explicit approval.

## Focus State tools: where they fit

Focus State tools are lightweight memory slots:

- `focusa_scratch` — working notes; safe fallback when scoped frame is unavailable.
- `focusa_decide` — one crystallized architectural choice.
- `focusa_constraint` — discovered hard requirement.
- `focusa_failure` — specific failure + diagnosis.
- `focusa_intent`, `focusa_current_focus`, `focusa_next_step`, `focusa_open_question`, `focusa_recent_result`, `focusa_note` — bounded operator-facing state.

Use them **after** project/trajectory/workpoint orientation, or as local scratch fallback when scoped Focus State is unavailable.

## Family adjacency matrix

| Family | Usually comes after | Usually leads to | Compounding value |
|---|---|---|---|
| Project identity | session start, tool doctor | trajectory, Workpoint | prevents cross-project contamination |
| Trajectory | project identity, evidence | proposed Workpoint, assessment | aligns mission with destination/waypoints |
| Workpoint | trajectory, active object | evidence, resume, checkpoint | preserves exact next slice across compaction |
| Evidence | work/test/proof | trajectory assessment, recent result | turns claims into durable handles |
| Prediction | before risky/uncertain action | evaluation, metacog | calibrates future decisions |
| Metacognition | prediction/evidence/outcome | reusable lessons, adjustments | compounds behavior across projects |
| Traversal/tree | low context/large state | targeted evidence/workpoint recovery | avoids raw transcript guessing |
| Work-loop/SilentSession | writer/status check | next ready work/background control | supports continuous execution safely |
| Diagnostics/hygiene | uncertainty/degraded state | recovery plan/resource mode | repairs state/tool trust boundaries |
| Focus State | any route, after orientation | operator-visible memory | concise durable notes, not the whole workflow |

## Machine-readable next-tool policy

- `per_tool_next_tools` is the exact next-tool shortlist used by the Pi affordance catalog.
- `edges[]` carries `from`, `to`, `rank`, `weight`, `from_family`, and `to_family`.
- `dynamic_weight_policy` documents how evaluated prediction scores adjust live `effective_edges`; base JSON remains the canonical static registry.
- Operator steering/current ask outranks choreography; choreography is route guidance, not authority.
- Focus State tools intentionally route back toward project identity, trajectory, and Workpoint instead of forming a note-only loop.

## Model hint template

```text
If task is project work: use project_identity → trajectory_view → workpoint_resume/checkpoint first.
If task changes code/state: active_object_resolve → evidence_capture/link after proof.
If task is risky: predict_record before, predict_evaluate after.
If task should improve future agents: metacog_retrieve/capture/reflect.
If context is large or stale: tool_doctor → traverse/tree → workpoint_resume.
```
