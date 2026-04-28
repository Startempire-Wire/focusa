# Spec 88 Implementation Decomposition — Ontology-backed Workpoint Continuity

**Date:** 2026-04-28  
**Parent spec:** `docs/88-ontology-backed-workpoint-continuity.md`  
**Spec bead:** `focusa-ql1o`  
**Purpose:** Convert Spec 88 into implementation beads with concrete code targets, API/CLI/tool contracts, Pi extension lifecycle hooks, compaction behavior, and verification gates.

---

## 0) Implementation doctrine

Spec 88 must be implemented as a Focusa-native continuity layer, not as Pi-local memory.

Canonical rule:

```text
Meaning lives in the typed workpoint, not in transcript tail, session_info entries, or provider cache metadata.
```

Authority boundaries:

- Focusa reducer owns canonical Workpoint state, ontology objects, ObjectSet membership, ActionIntent binding, VerificationRecord links, drift events, and recovery checkpoint identity.
- Pi extension owns lifecycle timing, tool UX, context/steer projection, and degraded local fallback only.
- Pi must never directly canonize ontology truth.
- Large raw outputs must become ECS/reference handles; workpoints carry concise evidence refs.

Primary references:

- Spec 88: `docs/88-ontology-backed-workpoint-continuity.md`
- Ontology overview: `docs/45-ontology-overview.md`
- Ontology primitives: `docs/46-ontology-core-primitives.md`
- Working sets/slices: `docs/49-working-sets-and-slices.md`
- Ontology reducer discipline: `docs/50-ontology-classification-and-reducer.md`
- Tool/action contracts: `docs/55-tool-action-contracts.md`
- Trace/checkpoint recovery: `docs/56-trace-checkpoints-recovery.md`
- Pi integration: `docs/44-pi-focusa-integration-spec.md`
- Pi extension contract: `docs/52-pi-extension-contract.md`
- Pi behavioral alignment: `docs/53-pi-behavioral-alignment-contract.md`
- Operator priority: `docs/54a-operator-priority-and-subject-preservation.md`
- Minimal slice routing: `docs/54b-context-injection-and-attention-routing.md`
- First-class tools: `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`
- Tool desirability: `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`
- Action parity: `docs/84-action-type-parity-spec.md`
- Relation parity: `docs/85-relation-type-parity-spec.md`

Important code anchors:

- Pi compaction hook: `apps/pi-extension/src/compaction.ts:76`
- Pi post-compaction hook: `apps/pi-extension/src/compaction.ts:132`
- Current generic resume steer: `apps/pi-extension/src/compaction.ts:191`
- Compact instruction builder: `apps/pi-extension/src/state.ts:1003`
- Pi state persistence: `apps/pi-extension/src/state.ts:1184`
- Minimal applicable slice hook: `apps/pi-extension/src/turns.ts:52`
- Work-loop decision context type: `crates/focusa-core/src/types.rs:363`
- Work-loop resume payload: `crates/focusa-api/src/routes/work_loop.rs:1119`
- Work-loop decision context route: `crates/focusa-api/src/routes/work_loop.rs:2163`
- Work-loop checkpoint route: `crates/focusa-api/src/routes/work_loop.rs:2756`
- Focus slot validation boundary: `crates/focusa-api/src/routes/focus.rs:292`

---

## 1) Bead tree overview

Parent epic to create:

- `Implement Spec88 ontology-backed workpoint continuity`

Child implementation beads:

1. `Spec88 Phase 0 — vocabulary/parity and API contract matrix`
2. `Spec88 Phase 1 — core Workpoint types and reducer events`
3. `Spec88 Phase 2 — workpoint persistence, active pointer, and replay/status projections`
4. `Spec88 Phase 3 — Workpoint API endpoints`
5. `Spec88 Phase 4 — CLI parity for focusa workpoint commands`
6. `Spec88 Phase 5 — Pi first-class workpoint tools`
7. `Spec88 Phase 6 — Pi lifecycle and compaction integration`
8. `Spec88 Phase 7 — context-overflow, model-switch, and degraded fallback recovery`
9. `Spec88 Phase 8 — drift detection and telemetry/replay signals`
10. `Spec88 Phase 9 — golden evals, prompt-pickup tests, and evidence packet`
11. `Spec88 Phase 10 — operator docs, skill updates, and rollout gate`

Execution order is mostly linear through Phase 6; Phases 7–10 can parallelize after API/tool skeletons exist.

---

## 2) Phase 0 — vocabulary/parity and API contract matrix

### Goal

Freeze the minimum ontology/action/relation vocabulary and tool/API contracts before code changes.

### Implementation details

Create evidence/contract artifact:

```text
docs/evidence/SPEC88_WORKPOINT_CONTRACT_MATRIX_2026-04-28.md
```

Matrix sections:

1. ObjectType vocabulary from Spec 88 §6.1:
   - `Mission`
   - `WorkItem`
   - `Surface`
   - `Endpoint`
   - `Component`
   - `Processor`
   - `Service`
   - `File`
   - `Test`
   - `VerificationRecord`
   - `ActionIntent`
   - `Blocker`
2. LinkType vocabulary from Spec 88 §6.2:
   - `targets`
   - `depends_on`
   - `consumes`
   - `produces`
   - `verifies`
   - `blocked_by`
   - `implemented_by`
   - `evidence_for`
3. ActionType vocabulary from Spec 88 §6.3:
   - `checkpoint_workpoint`
   - `resume_workpoint`
   - `detect_workpoint_drift`
   - `verify_ui_endpoint_binding`
   - `patch_component_binding`
   - `validate_pipeline_output`
   - `link_verification_evidence`
4. Endpoint contract:
   - `POST /v1/workpoint/checkpoint`
   - `GET /v1/workpoint/current`
   - `POST /v1/workpoint/resume`
   - `POST /v1/workpoint/drift-check`
5. CLI contract:
   - `focusa workpoint checkpoint`
   - `focusa workpoint current`
   - `focusa workpoint resume`
   - `focusa workpoint drift-check`
6. Pi tool contract:
   - `focusa_workpoint_checkpoint`
   - `focusa_workpoint_resume`
   - future `focusa_workpoint_drift_check`

### Required checks

- Confirm action types align with `docs/84-action-type-parity-spec.md` expectations.
- Confirm relation names align with `docs/85-relation-type-parity-spec.md` expectations.
- Confirm every tool follows `docs/55-tool-action-contracts.md`: input schema, output schema, side effects, idempotency, failure modes, verification hooks, degraded fallback.

### Acceptance criteria

- Contract matrix exists with object/link/action/API/CLI/Pi-tool rows.
- Every row cites the owning spec section.
- Open compatibility questions are explicit, not hidden.
- No implementation claims are made without code/evidence placeholders.

---

## 3) Phase 1 — core Workpoint types and reducer events

### Goal

Add Focusa-owned typed Workpoint state and reducer-visible event vocabulary.

### Code targets

- `crates/focusa-core/src/types.rs`
- `crates/focusa-core/src/reducer.rs`
- `crates/focusa-core/src/runtime/daemon.rs`
- possible module: `crates/focusa-core/src/workpoint/mod.rs`

### Type additions

Add bounded structs:

```rust
pub struct WorkpointState {
    pub active_workpoint_id: Option<Uuid>,
    pub entries: Vec<WorkpointRecord>,
}

pub struct WorkpointRecord {
    pub workpoint_id: Uuid,
    pub revision: u32,
    pub status: WorkpointStatus,
    pub mission_id: Option<String>,
    pub work_item_id: Option<String>,
    pub active_object_set_id: Option<String>,
    pub current_action_intent_id: Option<String>,
    pub verification_record_ids: Vec<String>,
    pub blocker_object_ids: Vec<String>,
    pub slice_policy_id: Option<String>,
    pub source_turn_id: Option<String>,
    pub session_id: Option<String>,
    pub checkpoint_reason: WorkpointCheckpointReason,
    pub confidence: WorkpointConfidence,
    pub canonical: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resume_projection: Option<String>,
}
```

Add enums:

```rust
WorkpointStatus = Active | Superseded | Degraded | Rejected | Archived
WorkpointCheckpointReason = Compaction | Resume | ContextOverflow | OperatorCheckpoint | PreSurgery | ModelSwitch | VerificationComplete
WorkpointConfidence = High | Medium | Low
WorkpointDriftSeverity = Low | Medium | High
```

Add events:

```rust
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

### Reducer rules

- New active workpoint supersedes older active workpoint for same work item unless explicit parallel branch/session id exists.
- Unknown object/link/action data is proposal-only unless already canonical.
- Bounded arrays enforced at reducer boundary.
- `canonical=false` records may be retained as degraded fallback but not promoted without verification/provenance.
- Operator steering can suppress projection but should not silently delete the old workpoint.

### Acceptance criteria

- Types compile.
- Reducer can apply proposed/promoted/superseded/drift events.
- State defaults are backward-compatible.
- Unit tests cover bounded vectors, active pointer transitions, and supersession.

---

## 4) Phase 2 — persistence, active pointer, replay/status projections

### Goal

Persist and expose Workpoint state across daemon restart and replay.

### Code targets

- `crates/focusa-core/src/runtime/persistence_sqlite.rs`
- `crates/focusa-core/src/runtime/persistence.rs`
- `crates/focusa-core/src/runtime/daemon.rs`
- `crates/focusa-core/src/replay/mod.rs`
- `crates/focusa-api/src/routes/work_loop.rs` for status/resume integration

### Implementation details

Persistence:

- serialize Workpoint events through existing event log path where possible
- add migration/table only if current event persistence cannot reconstruct state
- ensure restart restores active workpoint pointer

Status projection:

- augment existing work-loop status/resume payload with:

```json
"workpoint": {
  "active_workpoint_id": "...",
  "work_item_id": "...",
  "current_action_intent_id": "...",
  "active_object_set_id": "...",
  "checkpoint_reason": "compaction",
  "confidence": "high",
  "canonical": true
}
```

Replay:

- include workpoint events in trace/replay filters
- expose drift events in replay quality summaries

### Acceptance criteria

- Daemon restart preserves active workpoint.
- Replay reconstructs active/superseded state.
- Work-loop status includes active workpoint summary.
- Tests prove persistence and replay behavior.

---

## 5) Phase 3 — Workpoint API endpoints

### Goal

Add endpoint-backed Workpoint operations.

### Code targets

- new: `crates/focusa-api/src/routes/workpoint.rs`
- update: `crates/focusa-api/src/routes/mod.rs`
- update router registration in API main/server as needed
- related: `crates/focusa-api/src/routes/work_loop.rs`

### Endpoints

#### `POST /v1/workpoint/checkpoint`

Accepts structured checkpoint request. Returns typed envelope:

```json
{
  "status": "accepted|completed|partial|unknown|rejected",
  "workpoint_id": "...",
  "revision": 1,
  "active_object_set_id": "...",
  "current_action_intent_id": "...",
  "verification_record_ids": [],
  "checkpoint_id": "...",
  "resume_preview": "...",
  "canonical": true,
  "warnings": [],
  "next_step_hint": "..."
}
```

#### `GET /v1/workpoint/current`

Query params:

- `workpoint_id`
- `work_item_id`
- `session_id`
- `frame_id`

Returns active matching workpoint or `not_found`.

#### `POST /v1/workpoint/resume`

Renders `compact_prompt`, `full_json`, or `operator_summary`.

#### `POST /v1/workpoint/drift-check`

Preview mode by default; optional event emission with permission.

### Failure mapping

- `400 validation_failed`
- `403 permission_denied`
- `404 not_found`
- `409 stale_or_superseded`
- `422 reducer_rejected`
- `503 dependency_unavailable`

### Acceptance criteria

- All endpoints return typed envelopes.
- Endpoint tests cover success, validation failure, not found, stale/superseded, and degraded dependency.
- Permission behavior matches existing work-loop write rules.

---

## 6) Phase 4 — CLI parity

### Goal

Expose Workpoint workflows as first-class operator CLI commands.

### Code targets

- new: `crates/focusa-cli/src/commands/workpoint.rs`
- update: `crates/focusa-cli/src/commands/mod.rs`
- update: `crates/focusa-cli/src/main.rs`
- API client updates if needed: `crates/focusa-cli/src/api_client.rs`

### Commands

```bash
focusa workpoint checkpoint --work-item <id> --reason compaction --next-action "..." --json
focusa workpoint current --work-item <id> --json
focusa workpoint resume --mode compact-prompt --json
focusa workpoint drift-check --latest-output-file <path> --json
```

Human output must show:

- active workpoint id
- work item
- action intent
- next action
- canonical/degraded status
- warnings

### Acceptance criteria

- CLI JSON shape matches API shape.
- Human summaries are useful but not canonical.
- Error path tests cover validation and not-found.
- CLI docs/help text explain when to use each command.

---

## 7) Phase 5 — Pi first-class workpoint tools

### Goal

Add desirable low-friction Pi tools that models will choose over guessing.

### Code targets

- `apps/pi-extension/src/tools.ts`
- `apps/pi-extension/src/state.ts`
- tests under `tests/spec88_*`

### Tools

#### `focusa_workpoint_checkpoint`

Description should emphasize payoff:

```text
Best tool before compaction, resume, context surgery, or ambiguous continuation; creates an ontology-backed continuation anchor so work survives context loss.
```

Parameters:

- `current_ask?: string`
- `work_item_id?: string`
- `checkpoint_reason`
- `mission?: string`
- `target_objects?: []`
- `current_action?: { action_type, target_object_id, verification_hooks? }`
- `verified_evidence?: []`
- `blockers?: []`
- `next_action`
- `do_not_drift?: []`
- `source_turn_id?: string`
- `idempotency_key?: string`

Visible output:

```text
Workpoint checkpoint → completed
Work item: ...
Action: ...
Next: ...
Resume id: ...
Canonical: true
Next: use focusa_workpoint_resume after compact/resume.
```

#### `focusa_workpoint_resume`

Description:

```text
Best tool when resuming after compaction, model switch, or context overflow; fetches the safest Focusa-owned continuation packet.
```

### Acceptance criteria

- Tools are registered and discoverable.
- Runtime contract test verifies schemas and endpoint payloads.
- Outputs include next-step hints per Spec 87.
- Failure envelopes distinguish rejected/degraded/offline.

---

## 8) Phase 6 — Pi lifecycle and compaction integration

### Goal

Make Pi automatically preserve and re-inject Workpoint at the actual failure boundary.

### Code targets

- `apps/pi-extension/src/session.ts`
- `apps/pi-extension/src/turns.ts`
- `apps/pi-extension/src/compaction.ts`
- `apps/pi-extension/src/state.ts`
- `apps/pi-extension/src/config.ts`

### Hook changes

#### Session start/resume

- after frame recovery, call `/v1/workpoint/current`
- if found, store projected packet in S state
- inject via custom hidden message or context section
- if no workpoint and active work-loop task exists, optionally create low-confidence checkpoint

#### `before_agent_start`

Add behavioral rule:

```text
If a Focusa Workpoint Resume Packet is present, trust it as continuation anchor unless the operator explicitly steers elsewhere.
```

#### `context`

Add ordered sections:

```text
WORKPOINT
ACTIVE_OBJECT_SET
ACTION_INTENT
VERIFICATION_HOOKS
DRIFT_BOUNDARIES
```

Budget rule: Workpoint outranks historical/decayed context.

#### `session_before_compact`

- call Workpoint checkpoint with `checkpoint_reason=compaction`
- refresh ActiveMissionSet and CurrentActionIntent where possible
- request resume packet
- inject packet into compaction instructions

#### `session_compact` / compaction end

Replace generic `Last Active Focus` steer with Focusa Workpoint Resume Packet.

### Acceptance criteria

- Workpoint packet appears in compaction instructions and post-compact steer.
- If Focusa offline, local fallback is marked non-canonical.
- Compaction tests prove no generic-only resume steer remains.
- Operator steering still suppresses stale workpoint carryover.

---

## 9) Phase 7 — context overflow, model switch, degraded fallback

### Goal

Handle provider overflow and discontinuities without losing workpoint.

### Code targets

- `apps/pi-extension/src/turns.ts`
- `apps/pi-extension/src/session.ts`
- `apps/pi-extension/src/compaction.ts`
- `apps/pi-extension/src/state.ts`

### Behaviors

Context overflow:

- detect `context_length_exceeded` or provider-equivalent error
- checkpoint workpoint before blind retry
- request compact/slim recovery
- preserve raw session as backup/reference if possible
- resume from workpoint packet

Model switch/fork:

- retrieve current workpoint
- record provenance old/new model if available
- inject resume packet

Degraded fallback:

- local `focusa-workpoint-fallback` entry in Pi session
- scratch mirror with full details
- `canonical=false` in visible output
- promotion/reconciliation suggestion on reconnect

### Acceptance criteria

- Simulated context overflow triggers checkpoint path.
- Model switch/fork refreshes workpoint packet.
- Degraded fallback is clearly non-canonical.
- No automatic destructive/session-trimming action occurs without existing safety policy.

---

## 10) Phase 8 — drift detection and telemetry/replay

### Goal

Detect and explain workpoint drift after resume/compaction.

### Code targets

- `apps/pi-extension/src/turns.ts`
- `apps/pi-extension/src/state.ts`
- `crates/focusa-core/src/runtime/daemon.rs`
- `crates/focusa-api/src/routes/telemetry.rs`
- replay summaries in `crates/focusa-core/src/replay/mod.rs`

### Drift inputs

- latest assistant output preview
- latest tool action metadata
- active WorkpointResumePacket
- ActiveMissionSet object ids
- CurrentActionIntent type/target
- `do_not_drift` boundaries
- operator steering state

### Drift classes

- `notes_only_drift`
- `wrong_object_drift`
- `repeated_validation_drift`
- `work_item_switch_drift`
- `missing_verification_hook_drift`
- `stale_workpoint_overcarry`

### Acceptance criteria

- Preview drift-check is read-only.
- Event-emitting drift-check is permissioned.
- Turn-end drift emits trace with severity/reason/recovery hint.
- Operator steering prevents false positives.
- Replay/status surfaces recent drift count and latest reason.

---

## 11) Phase 9 — golden evals and evidence packet

### Goal

Prove Spec88 prevents the observed failure class.

### Test targets

- `tests/spec88_workpoint_api_contract_test.sh`
- `tests/spec88_cli_workpoint_contract_test.sh`
- `tests/spec88_pi_tool_runtime_contract.ts`
- `tests/spec88_compaction_survival_test.sh`
- `tests/spec88_context_overflow_recovery_test.sh`
- `tests/spec88_drift_detection_test.sh`
- evidence doc: `docs/evidence/SPEC88_WORKPOINT_CONTINUITY_EVIDENCE_2026-04-28.md`

### Golden scenario

ASAP Digest-like scenario:

```text
Mission: homepage renders real AI processed content and playable TTS audio.
Active objects: homepage surface, /api/audio/today endpoint, audio widget component, Kokoro TTS service.
Verified evidence: processors OK, TTS OK, endpoint 200 available=true.
Next action: patch/verify audio widget play/pause/loading/error controls.
Failure to prevent: notes-only work or generic endpoint validation after compaction.
```

### Acceptance criteria

- Workpoint survives compaction summary and post-compact steer.
- Agent/tool simulated resume selects UI binding task.
- Drift detector catches notes-only/generic validation drift.
- Context overflow recovery produces bounded resume packet.
- Evidence file cites code lines and command outputs.

---

## 12) Phase 10 — docs, skill, rollout

### Goal

Make the feature usable and discoverable.

### Targets

- update `/root/.pi/skills/focusa/SKILL.md` only if installed skill sync is intended
- update project skill: `apps/pi-extension/skills/focusa/SKILL.md`
- add docs section to `docs/44-pi-focusa-integration-spec.md` or cross-reference Spec88
- add operator examples to evidence/usage doc
- update Pi tool descriptions for pickup/desirability

### Rollout gates

- existing Spec81/87 tool contract tests still pass
- Rust checks pass
- Pi extension TypeScript compile passes
- no context injection subject hijack regressions
- degradation behavior visible and non-canonical

---

## 13) Dependency graph

```text
Phase 0 contract matrix
  → Phase 1 core types/events
    → Phase 2 persistence/status
      → Phase 3 API
        → Phase 4 CLI
        → Phase 5 Pi tools
          → Phase 6 lifecycle/compaction
          → Phase 7 overflow/model/degraded recovery
          → Phase 8 drift detection
            → Phase 9 evals/evidence
              → Phase 10 rollout docs
```

Parallel-safe lanes after Phase 3:

- CLI can proceed once endpoint schemas stabilize.
- Pi tool registration can proceed against mock/stub endpoint envelopes.
- Drift detector can start in preview-only mode while event emission is finalized.

---

## 14) Closure definition for the epic

The Spec88 implementation epic is complete only when:

1. Workpoint state is reducer-owned and replayable.
2. API/CLI/Pi tools expose checkpoint and resume parity.
3. Pi compaction checkpoints and re-injects the workpoint in both summary and steer paths.
4. Context overflow creates a workpoint checkpoint before recovery.
5. Drift detection flags the observed notes-only/generic-validation lapse.
6. Golden eval evidence is checked in with citations.
7. Operator docs explain when to use workpoint tools and how degraded fallback appears.
