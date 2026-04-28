# 88 — Ontology-backed Workpoint Continuity and Pi Compaction Integration

**Date:** 2026-04-28  
**Status:** implemented through Phase 10 rollout gate  
**Priority:** critical  
**Bead:** `focusa-ql1o`  
**Owner:** Focusa + Pi integration

---

## 1) Why this spec exists

A Pi session can retain a large raw transcript and still lose the operational continuation point after compaction, context overflow recovery, model switch, or manual session slimming.

The observed failure class:

1. Pi session history grew large enough to trigger provider `context_length_exceeded`.
2. Raw transcript slimming preserved recent conversation, but not a strong canonical continuation contract.
3. Focusa Focus State contained little or no actionable workpoint content.
4. The agent resumed adjacent work instead of the intended next step.

This spec closes that gap by making Focusa preserve continuation as typed working-world state, not as raw chat history.

---

## 2) Core thesis

Focusa should preserve the typed workpoint, not the transcript tail.

A workpoint is the active continuation contract projected from ontology state:

```text
Mission → ActiveMissionSet → ActionIntent → VerificationRecords → Blockers/OpenLoops → Next Slice
```

Pi must resume from this projection when it exists. Pi must not rely on broad raw history, stale compaction summaries, or cache metadata to infer where work should continue.

---

## 3) Authority model

### 3.1 Focusa owns

- canonical ontology objects, links, statuses, action intents, and verification records
- ActiveMissionSet membership
- reducer promotion/rejection of workpoint-relevant ontology deltas
- checkpoint identity and provenance
- resume packet generation
- drift/failure classification
- trace, replay, and recovery state

### 3.2 Pi extension owns

- lifecycle hook timing
- prompt/steer delivery
- local non-canonical fallback entries
- tool UX and visible summaries
- context budget observation
- local detection hints forwarded to Focusa

### 3.3 Pi extension must not own

- canonical workpoint truth
- parallel long-lived project memory
- reducer bypass writes
- unverified ontology object/link/action state

---

## 4) Non-goals

This spec does not:

- preserve full raw transcripts in prompts
- weaken Focus State validation rules
- make Pi a second cognitive runtime
- replace ASCC, Focus State, CLT, or work-loop primitives
- require broad ontology expansion beyond the minimum objects/actions needed for continuity
- automate destructive recovery or git/session mutation without existing safety policy

---

## 5) Terminology

### Workpoint

A compact, bounded continuation projection derived from canonical or proposed Focusa state.

### WorkpointCheckpoint

A reducer-visible event/state transition that records the current workpoint projection and its provenance.

### WorkpointResumePacket

A prompt-safe projection rendered from the latest active workpoint for Pi or another harness.

### ActiveMissionSet

The bounded ontology ObjectSet containing the task, target objects, dependencies, evidence, blockers, and verification hooks needed for the next action.

### CurrentActionIntent

A typed operation over ontology objects that represents the next concrete continuation step.

### Workpoint drift

A turn/action that materially diverges from the active workpoint without explicit operator steering or reducer-authorized scope change.

---

## 6) Ontology model

The workpoint must be ontology-backed. The human-readable resume text is a projection, not canonical truth.

### 6.1 Minimum object types

| ObjectType | Purpose |
|---|---|
| `Mission` | Binds the current objective/subobjective. |
| `WorkItem` | Bead/task/issue identity. |
| `Surface` | User-facing UI or product surface. |
| `Endpoint` | API/route/contract target. |
| `Component` | UI/backend component under change or verification. |
| `Processor` | AI/content/audio processor or pipeline unit. |
| `Service` | Runtime service dependency. |
| `File` | Source/config/test artifact. |
| `Test` | Validation target. |
| `VerificationRecord` | Evidence that an object/action/link is verified or failed. |
| `ActionIntent` | Typed next action. |
| `Blocker` | Current unresolved obstacle. |

### 6.2 Minimum link types

| LinkType | Source → Target |
|---|---|
| `targets` | Mission/ActionIntent → object |
| `depends_on` | object/action → object/service/file |
| `consumes` | component/surface → endpoint/data/service |
| `produces` | processor/service/action → artifact/output |
| `verifies` | VerificationRecord → object/link/action |
| `blocked_by` | object/action/mission → blocker |
| `implemented_by` | component/action → file |
| `evidence_for` | artifact/handle/result → VerificationRecord/action |

### 6.3 Minimum action types

| ActionType | Purpose |
|---|---|
| `checkpoint_workpoint` | Create/update workpoint projection. |
| `resume_workpoint` | Render the active reentry packet. |
| `detect_workpoint_drift` | Compare latest turn/action to active workpoint. |
| `verify_ui_endpoint_binding` | Prove a UI component consumes and renders endpoint output. |
| `patch_component_binding` | Modify a component to use intended data/control path. |
| `validate_pipeline_output` | Verify end-to-end pipeline output. |
| `link_verification_evidence` | Attach evidence to object/link/action. |

Action and link names must remain aligned with action/relation parity specs.

---

## 7) Workpoint projection schema

### 7.1 Canonical-ish API shape

```json
{
  "workpoint_id": "wp_...",
  "revision": 1,
  "status": "active",
  "mission_id": "mission:...",
  "work_item_id": "focusa-ql1o",
  "active_object_set_id": "objectset:...",
  "current_action_intent_id": "actionintent:...",
  "verification_record_ids": ["verify:..."],
  "blocker_object_ids": [],
  "slice_policy_id": "slicepolicy:workpoint_resume_v1",
  "source_turn_id": "pi-turn-...",
  "session_id": "...",
  "checkpoint_reason": "compaction",
  "confidence": "high",
  "canonical": true,
  "created_at": "2026-04-28T00:00:00Z",
  "updated_at": "2026-04-28T00:00:00Z"
}
```

### 7.2 Prompt-safe resume projection

```text
WORKPOINT:
  Work item: <id/title>
  Mission: <bounded objective>
  Active objects: <typed ids only>
  Current action: <ActionType target>
  Verified evidence: <VerificationRecord refs + short summaries>
  Blockers/open loops: <typed ids + short summaries>
  Next action: <one concrete action>
  Do not drift: <bounded negative boundaries>
```

### 7.3 Example

```text
WORKPOINT:
  Work item: asap-99p7.4
  Mission: Homepage renders real AI processed content and playable TTS audio.
  Active objects: Surface:asapdigest.homepage, Endpoint:app.audio.today, Component:homepage.audio_widget
  Current action: verify_ui_endpoint_binding → Component:homepage.audio_widget
  Verified evidence: VerificationRecord:openrouter_processors_ok, VerificationRecord:kokoro_tts_ogg_ok, VerificationRecord:audio_today_200_ok
  Blockers/open loops: UI audio widget controls not verified.
  Next action: Inspect and patch homepage audio widget play/pause/loading/error states against /api/audio/today.
  Do not drift: No notes-only work; endpoint-only validation is insufficient.
```

---

## 8) First-class tool surface

### 8.1 MVP Pi tools

#### `focusa_workpoint_checkpoint`

Create or update an ontology-backed workpoint checkpoint.

Inputs:

```ts
{
  current_ask?: string;
  work_item_id?: string;
  checkpoint_reason: "compaction" | "resume" | "context_overflow" | "operator_checkpoint" | "pre_surgery" | "model_switch" | "verification_complete";
  mission?: string;
  target_objects?: Array<{ type: string; id: string; role?: string; status?: string }>;
  current_action?: {
    action_type: string;
    target_object_id: string;
    verification_hooks?: string[];
  };
  verified_evidence?: string[];
  blockers?: string[];
  next_action: string;
  do_not_drift?: string[];
  source_turn_id?: string;
  idempotency_key?: string;
}
```

Output:

```ts
{
  status: "accepted" | "completed" | "partial" | "unknown" | "rejected";
  workpoint_id?: string;
  revision?: number;
  active_object_set_id?: string;
  current_action_intent_id?: string;
  verification_record_ids?: string[];
  checkpoint_id?: string;
  resume_preview?: string;
  canonical: boolean;
  warnings?: string[];
  next_step_hint?: string;
}
```

#### `focusa_workpoint_resume`

Fetch/render the safest current resume packet.

Inputs:

```ts
{
  workpoint_id?: string;
  work_item_id?: string;
  mode?: "compact_prompt" | "full_json" | "operator_summary";
}
```

Output:

```ts
{
  status: "completed" | "not_found" | "degraded";
  workpoint_id?: string;
  resume_packet?: string;
  canonical: boolean;
  source_checkpoint_id?: string;
  warnings?: string[];
}
```

### 8.2 Later tools

#### `focusa_workpoint_drift_check`

Public read/observation tool after internal drift detection has evidence.

#### `focusa_workpoint_guard`

Composite tool that checkpoints, snapshots, renders resume, and optionally drift-checks in one call. Defer until MVP pickup evidence shows the workflow repeats often enough.

---

## 9) API and CLI parity

### 9.1 API

```text
POST /v1/workpoint/checkpoint
GET  /v1/workpoint/current
POST /v1/workpoint/resume
POST /v1/workpoint/drift-check
```

### 9.2 CLI

```bash
focusa workpoint checkpoint
focusa workpoint current
focusa workpoint resume
focusa workpoint drift-check
```

CLI must support:

- stable JSON output
- human-readable summary
- typed failure output
- idempotency key option for checkpoint
- work item filter
- mode selection for resume projection

---

## 10) Reducer events

Minimum events:

```text
WorkpointCheckpointProposed
WorkpointCheckpointPromoted
WorkpointCheckpointRejected
WorkpointSuperseded
WorkpointResumeRendered
WorkpointDriftDetected
WorkpointDegradedFallbackRecorded
OntologyWorkingSetRefreshed
OntologyActionIntentBound
OntologyVerificationLinked
```

Reducer responsibilities:

- validate object/action/link types
- enforce bounded field limits
- promote only evidence-linked canonical state
- write replayable events
- update active workpoint pointer
- supersede older workpoints when scope changes
- preserve operator steering precedence

---

## 11) Tool/action contract

### 11.1 Side effects

`focusa_workpoint_checkpoint` may:

- create/update workpoint state
- propose ontology objects/links/action intent
- refresh ActiveMissionSet membership
- link VerificationRecords
- create/update work-loop checkpoint
- optionally update concise Focus State slots
- create non-canonical local fallback if Focusa unavailable

`focusa_workpoint_resume` is read-only unless telemetry records `WorkpointResumeRendered`.

### 11.2 Idempotency

Checkpoint is conditionally idempotent with:

```text
workpoint:{work_item_id}:{source_turn_id}:{checkpoint_reason}
```

Resume is strongly idempotent read-only.

Drift-check is read-only in preview mode and append-style when emitting observation events.

### 11.3 Failure modes

| Mode | Meaning | Retry posture |
|---|---|---|
| validation failure | malformed object/action/workpoint payload | do not retry unchanged |
| dependency failure | daemon/ontology/reducer unavailable | retry after health change |
| permission failure | caller lacks authority | do not blind retry |
| partial success | local checkpoint or Focus State wrote but ontology promotion failed | reconcile/promote later |
| unknown | timeout or ambiguous completion | read current workpoint before retry |
| reducer rejection | proposed ontology delta invalid or unsupported | revise payload/spec |

### 11.4 Degraded fallback

If Focusa is unavailable:

- Pi may write local session entry `focusa-workpoint-fallback`
- scratchpad may store full details
- output must say `canonical=false`
- fallback must never be treated as reducer-approved ontology truth
- on reconnect, Pi may propose promotion/reconciliation

---

## 12) Pi extension integration

### 12.1 Session start/resume

On Pi session start/resume:

1. ensure or recover Focusa frame
2. call `GET /v1/workpoint/current` scoped by session/work item when possible
3. render `WorkpointResumePacket` if present
4. inject it as compact supporting context
5. if no workpoint exists, optionally create low-confidence checkpoint from active Focus State/work-loop state/current ask

Rule:

```text
Pi resume must prefer Focusa WorkpointResumePacket over raw transcript tail.
```

### 12.2 `before_agent_start`

Add behavioral law:

```text
If a Focusa Workpoint Resume Packet is present, treat it as the authoritative continuation anchor unless the operator explicitly steers elsewhere.
Do not use raw transcript tail to override the active workpoint.
```

### 12.3 `context` hook

The minimal applicable slice should include workpoint sections when relevant:

```text
WORKPOINT
ACTIVE_OBJECT_SET
ACTION_INTENT
VERIFICATION_HOOKS
DRIFT_BOUNDARIES
```

Priority order inside the injected slice:

1. operator current ask
2. workpoint when mission carryover remains relevant
3. applicable constraints/decisions
4. verified evidence/object handles
5. recent results
6. historical/decayed context

If operator input changes task, suppress or downgrade the workpoint and mark the old workpoint as superseded only through reducer-approved transition.

### 12.4 `session_before_compact`

Before Pi compacts:

1. flush local Focusa shadow writes through validation
2. call `focusa_workpoint_checkpoint` internally with `checkpoint_reason=compaction`
3. refresh ActiveMissionSet
4. bind CurrentActionIntent
5. link recent VerificationRecords and blockers
6. request `WorkpointResumePacket`
7. include that packet in compaction instructions

Required compaction instruction:

```text
Preserve the Focusa Workpoint Resume Packet: mission, work item, active objects, action intent, verified evidence, blockers, next action, and do-not-drift boundaries. Prefer object ids and handles over raw transcript.
```

### 12.5 Pi compaction semantics

Pi session context after compaction is built from compaction summary plus kept/post-compaction messages. Therefore:

```text
Workpoint projection must be present in both the compaction summary path and the post-compaction steer path.
```

This prevents loss when either mechanism is partially truncated or skipped.

### 12.6 `session_compact` / compaction end

Replace generic post-compact steer with:

```md
# Focusa Workpoint Resume Packet
Authority: Focusa ontology/reducer
Work item: ...
Mission: ...
Active object set: ...
Current action intent: ...
Verified evidence: ...
Blockers/open loops: ...
Next action: ...
Do not drift: ...
Directive: Execute the next action only. If context conflicts, trust this packet unless operator steers differently.
```

### 12.7 Context-length overflow

When a provider returns `context_length_exceeded` or equivalent:

1. stop blind retry
2. create workpoint checkpoint if possible
3. create tree/session snapshot if available
4. request compact/slim recovery
5. resume from workpoint packet
6. preserve full raw session as backup or reference handle when possible

Overflow is a mandatory workpoint checkpoint trigger.

### 12.8 Model switch/fork

On model switch, fork, or branch resume:

1. retrieve current workpoint
2. inject resume packet
3. record provenance with old/new model and session/CLT node when available

### 12.9 Turn-end drift detection

At turn end, compare latest assistant output/tool intent against active workpoint.

Signal drift when:

- latest action touches no active target object
- notes-only work happens while current action requires implementation/verification
- endpoint validation repeats while UI binding remains unverified
- action intent is ignored without operator steering
- work item changes without scope-change approval

Emit `WorkpointDriftDetected` with severity, evidence, and recovery hint.

---

## 13) Large output and cache discipline

The workpoint must not inline large raw outputs.

Rules:

- raw bash/tool output becomes ECS/reference handle
- workpoint stores concise verification conclusions plus handles
- session_info/toolResult noise must never be used as primary resume source
- context slice must prefer object ids, action ids, verification ids, and reference handles
- context budget pressure should increase checkpoint frequency and reduce raw context reliance

---

## 14) Automatic triggers

Create or refresh workpoint checkpoint on:

- session start/resume when no active workpoint exists
- `session_before_compact`
- `session_compact`
- provider context overflow
- manual session slimming/surgery
- model switch/fork
- operator says `continue` after long or ambiguous context
- major verification completion
- blocker emergence
- work-loop current task change
- ActiveMissionSet membership refresh

---

## 15) Acceptance tests and evals

### 15.1 Golden compaction survival

Scenario:

1. Long session with adjacent work threads.
2. Active workpoint targets homepage audio widget UI binding.
3. Compaction occurs.
4. Agent resumes.

Pass:

- resume packet contains mission, active objects, action intent, evidence, next action, do-not-drift boundaries
- agent chooses UI binding work, not notes-only or generic validation
- packet survives through compaction summary and post-compaction steer

### 15.2 Context overflow recovery

Scenario:

1. Simulate `context_length_exceeded`.
2. Trigger workpoint checkpoint and slim/compact recovery.
3. Resume.

Pass:

- full raw session can be backed up or referenced
- active context is bounded
- workpoint remains intact
- no blind retry with same oversized context

### 15.3 Drift detection

Scenario:

1. Workpoint next action requires patching/verifying a UI audio widget.
2. Agent attempts notes-only update or repeats endpoint-only validation.

Pass:

- `WorkpointDriftDetected` emitted
- severity/reason visible
- recovery hint points back to action intent

### 15.4 API/CLI/tool parity

Pass:

- Pi tools, API, and CLI expose equivalent semantics
- JSON schemas match
- typed failures are stable
- idempotency behavior is tested

### 15.5 Ontology reducer discipline

Pass:

- no Pi tool directly canonizes ontology truth
- ambiguous object/action/link data enters as proposal
- reducer promotion/rejection is replayable
- verification records link evidence to targets

---

## 16) Implementation plan

### Phase 0 — Spec and decomposition

- finalize this spec
- create bead decomposition
- map object/action/link vocabulary to parity specs

### Phase 1 — Core/API model

- add Workpoint state/types
- add reducer events
- add API endpoints
- add persistence/replay handling

### Phase 2 — CLI parity

- implement `focusa workpoint ...`
- provide JSON and human summaries
- add typed error tests

### Phase 3 — Pi MVP tools

- add `focusa_workpoint_checkpoint`
- add `focusa_workpoint_resume`
- wire to endpoint-backed behavior
- visible outputs include next-step hints

### Phase 4 — Pi lifecycle integration

- session start/resume injection
- context hook section ordering
- `session_before_compact` checkpoint
- `session_compact` resume steer
- context overflow recovery path
- model switch/fork refresh

### Phase 5 — Drift detection

- internal turn-end drift detector
- trace/failure emission
- optional public `focusa_workpoint_drift_check`

### Phase 6 — Evidence and polish

- golden evals
- runtime tests
- prompt-level pickup tests
- evidence doc with file:line and command output

---

## 17) Success criteria

Spec 88 is complete when:

1. Focusa can create and resume from an ontology-backed workpoint without raw transcript dependence.
2. Pi compaction preserves workpoint in both compaction summary and post-compact steer paths.
3. Context overflow recovery checkpoints the workpoint before trimming/slimming.
4. Pi tools, API, and CLI expose checkpoint/resume parity.
5. Drift detection flags notes-only or adjacent-thread continuation when an active ActionIntent exists.
6. Golden evals prove the ASAP-style homepage audio widget scenario survives compaction and overflow.
7. All workpoint canonical writes pass through reducer-governed ontology/event paths.

---

## 18) Design law

Meaning lives in the typed workpoint, not in the transcript.

Focusa should not hope compaction preserves meaning. Focusa should checkpoint the typed working world before compaction, project it after compaction, and verify future turns against it.
