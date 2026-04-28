# Spec88 Workpoint Contract Matrix

**Date:** 2026-04-28  
**Spec:** `docs/88-ontology-backed-workpoint-continuity.md`  
**Decomposition:** `docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md`  
**Beads:** `focusa-a2w2`, `focusa-a2w2.1`  
**Status:** Phase 0 contract matrix / implementation precursor

---

## 1) Purpose

This matrix freezes the first implementation vocabulary and contract surface for Spec88 before code changes.

It binds the Workpoint continuity design to:

- ontology primitives
- action/link parity requirements
- Focusa reducer authority
- API and CLI parity
- Pi first-class tool pickup requirements
- Pi compaction/resume integration points

Primary rule:

```text
Workpoint is an ontology-backed projection, not raw memory.
```

---

## 2) Reference map

| Area | Reference |
|---|---|
| Spec88 source | `docs/88-ontology-backed-workpoint-continuity.md` |
| Decomposition | `docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md` |
| Ontology ownership | `docs/45-ontology-overview.md` |
| Ontology primitives | `docs/46-ontology-core-primitives.md` |
| Working sets/slices | `docs/49-working-sets-and-slices.md` |
| Reducer discipline | `docs/50-ontology-classification-and-reducer.md` |
| Tool/action contracts | `docs/55-tool-action-contracts.md` |
| Recovery/checkpoints | `docs/56-trace-checkpoints-recovery.md` |
| Pi integration | `docs/44-pi-focusa-integration-spec.md` |
| Pi extension contract | `docs/52-pi-extension-contract.md` |
| Behavioral alignment | `docs/53-pi-behavioral-alignment-contract.md` |
| Operator priority | `docs/54a-operator-priority-and-subject-preservation.md` |
| Context routing | `docs/54b-context-injection-and-attention-routing.md` |
| Tool suite quality | `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md` |
| Tool desirability | `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md` |
| Action parity | `docs/84-action-type-parity-spec.md` |
| Relation parity | `docs/85-relation-type-parity-spec.md` |

---

## 3) ObjectType matrix

| ObjectType | Spec88 section | Purpose | Required minimum properties | Canonicalization path | First code target |
|---|---:|---|---|---|---|
| `Mission` | §6.1 | Binds current objective/subobjective. | `id`, `title`, `status`, `source_frame_id` | reducer-promoted ontology object or existing frame-derived mission projection | `crates/focusa-core/src/types.rs` |
| `WorkItem` | §6.1 | Bead/task identity used for continuity and scope. | `id`, `title?`, `status`, `source` | deterministic from Beads/work-loop current task where available | `crates/focusa-core/src/types.rs`, `crates/focusa-api/src/routes/work_loop.rs` |
| `Surface` | §6.1 | User-facing product/UI surface. | `id`, `label`, `url?`, `status` | proposal unless deterministic route/UI mapping exists | ontology routes / future extractor |
| `Endpoint` | §6.1 | API/route/contract target. | `id`, `path`, `method?`, `status` | deterministic route extraction or verification-linked proposal | ontology routes / route extractor |
| `Component` | §6.1 | UI/backend component under change or verification. | `id`, `kind`, `path?`, `status` | deterministic file/component extraction or proposal | ontology/file mapping |
| `Processor` | §6.1 | AI/content/audio processor or pipeline unit. | `id`, `processor_kind`, `status` | deterministic config/code extraction or proposal | ontology extractor |
| `Service` | §6.1 | Runtime dependency. | `id`, `endpoint?`, `health_status?`, `status` | tool-derived health observation + reducer promotion | environment/affordance layer |
| `File` | §6.1 | Source/config/test artifact. | `id`, `path`, `status` | deterministic path mapping | ontology file mapping |
| `Test` | §6.1 | Validation target. | `id`, `path/command`, `status` | deterministic test path/command mapping | tests / ontology extractor |
| `VerificationRecord` | §6.1 | Evidence for object/link/action state. | `id`, `target_ref`, `result`, `evidence_ref`, `verified_at` | reducer event from tool/test/API result | `crates/focusa-core/src/types.rs` |
| `ActionIntent` | §6.1 | Typed next action for continuation. | `id`, `action_type`, `target_ref`, `verification_hooks`, `status` | reducer event from workpoint checkpoint/proposal | `crates/focusa-core/src/types.rs` |
| `Blocker` | §6.1 | Current unresolved obstacle. | `id`, `reason`, `severity`, `target_ref?`, `status` | failure/blocker signal via reducer | `crates/focusa-core/src/types.rs` |

Notes:

- ObjectType names must align with ontology primitive exposure if `/v1/ontology/primitives` projects object vocab.
- New objects created from LLM/tool input start as proposals unless deterministic extraction or verification grants direct promotion.

---

## 4) LinkType matrix

| LinkType | Spec88 section | Source → Target | Evidence policy | Promotion rule | Parity risk |
|---|---:|---|---|---|---|
| `targets` | §6.2 | `Mission`/`ActionIntent` → any target object | checkpoint request or action intent payload | promote when target object refs validate | ensure not duplicated by `acts_on` alias |
| `depends_on` | §6.2 | object/action → object/service/file | deterministic dependency, config, or user/tool evidence | promote with evidence or deterministic extractor | may already exist in relation vocab |
| `consumes` | §6.2 | component/surface → endpoint/data/service | code route use, runtime contract, or explicit checkpoint payload | proposal then verification promotion | key for UI endpoint binding eval |
| `produces` | §6.2 | processor/service/action → artifact/output | processor result or artifact handle | promote with VerificationRecord | align with existing artifact relation terms |
| `verifies` | §6.2 | VerificationRecord → object/link/action | verification event | direct if evidence ref valid | should be canonical relation |
| `blocked_by` | §6.2 | object/action/mission → Blocker | failure/blocker signal | promote with blocker event | align with work-loop blocker vocabulary |
| `implemented_by` | §6.2 | component/action → File | deterministic path mapping or diff | promote from extractor/diff | may overlap `defined_in` |
| `evidence_for` | §6.2 | artifact/handle/result → VerificationRecord/action | ECS/ref handle or test output | promote if evidence handle resolves | required for no raw-output prompt bloat |

Compatibility questions for Phase 1:

- If existing ontology uses `defined_in`, should `implemented_by` become alias or canonical?
- If existing ontology uses `requires`, should `depends_on` remain canonical or alias?
- Relation parity tests must name allowed aliases explicitly.

---

## 5) ActionType matrix

| ActionType | Spec88 section | Target | Side effects | Verification hook | Idempotency |
|---|---:|---|---|---|---|
| `checkpoint_workpoint` | §6.3, §8.1 | Workpoint / Mission / WorkItem | creates/revises workpoint, may propose ontology deltas | post-read current workpoint; reducer event exists | conditional via `workpoint:{work_item_id}:{source_turn_id}:{checkpoint_reason}` |
| `resume_workpoint` | §6.3, §8.1 | Workpoint | renders resume projection; optional telemetry | packet returned with source id | strongly idempotent read path |
| `detect_workpoint_drift` | §6.3, §12.9 | Workpoint + latest turn/action | read-only preview or event emission | drift result includes severity/reason/evidence | read-only in preview; append-style if emitting |
| `verify_ui_endpoint_binding` | §6.3 | Component/Surface + Endpoint | runs/records verification; no direct code mutation | UI consumes endpoint and renders controls | process-executing/non-idempotent unless test-only |
| `patch_component_binding` | §6.3 | Component/File | code edit proposal/execution outside Focusa core | diff + tests + UI verification | conditional; requires file precondition/diff verification |
| `validate_pipeline_output` | §6.3 | Processor/Endpoint/Surface | runs validation command/API probe | VerificationRecord linked to target | process-executing; verify before retry |
| `link_verification_evidence` | §6.3 | VerificationRecord | creates evidence link | evidence handle resolves | conditional; dedupe by target+evidence ref |

Action parity requirement:

- Phase 0/1 must decide whether these are new canonical actions or aliases to existing action vocab.
- Tests must fail on drift between docs, API, CLI, and ontology projections.

---

## 6) Workpoint API contract matrix

| Endpoint | Method | Side effects | Permission | Output status values | Degraded behavior |
|---|---|---|---|---|---|
| `/v1/workpoint/checkpoint` | POST | dispatches reducer event(s), may update active workpoint pointer | workpoint/write or work-loop write equivalent | `accepted`, `completed`, `partial`, `unknown`, `rejected` | return `partial`/`canonical=false` if fallback only |
| `/v1/workpoint/current` | GET | none | read | `completed`, `not_found`, `degraded` | may return last known degraded projection with warning |
| `/v1/workpoint/resume` | POST | read path plus optional `WorkpointResumeRendered` telemetry | read; telemetry write if emitted | `completed`, `not_found`, `degraded` | render from degraded fallback only with `canonical=false` |
| `/v1/workpoint/drift-check` | POST | preview none; event mode emits drift event | read for preview; write for emit | `completed`, `no_drift`, `drift_detected`, `not_found`, `rejected` | preview can run from packet only; event emit requires Focusa |

Endpoint envelope minimum:

```json
{
  "status": "completed",
  "workpoint_id": "...",
  "revision": 1,
  "canonical": true,
  "warnings": [],
  "next_step_hint": "..."
}
```

---

## 7) CLI command matrix

| Command | Endpoint | Human summary | JSON mode | Required flags/options |
|---|---|---|---|---|
| `focusa workpoint checkpoint` | POST `/v1/workpoint/checkpoint` | workpoint id, work item, action, next action, canonical | same as API | `--reason`, optional `--work-item`, `--next-action`, `--idempotency-key` |
| `focusa workpoint current` | GET `/v1/workpoint/current` | active id + concise packet | same as API | optional `--work-item`, `--session`, `--frame` |
| `focusa workpoint resume` | POST `/v1/workpoint/resume` | compact resume packet | same as API | `--mode compact-prompt|full-json|operator-summary` |
| `focusa workpoint drift-check` | POST `/v1/workpoint/drift-check` | drift/no drift + reason | same as API | latest output/action source, optional `--emit` |

CLI acceptance:

- JSON shapes match API.
- Human summaries never imply degraded fallback is canonical.
- Errors expose typed failure reason.

---

## 8) Pi tool matrix

| Pi tool | Endpoint | Pickup trigger | Mutability | Visible summary requirements |
|---|---|---|---|---|
| `focusa_workpoint_checkpoint` | POST checkpoint | before compact/resume/context surgery/overflow/model switch/ambiguous continue | write; conditional idempotency | checkpoint id, work item, action, next action, canonical/degraded, next hint |
| `focusa_workpoint_resume` | POST resume or GET current + render | session resume, after compaction, after overflow, after model switch | read | packet preview, source checkpoint, warnings |
| `focusa_workpoint_drift_check` | POST drift-check | after suspected wrong turn; internal first | read preview or write emit | no drift/drift class/severity/recovery hint |

Spec87 desirability requirements:

- Tool descriptions must say why the model should use the tool instead of guessing.
- Zero-result behavior must say how to create/checkpoint a workpoint.
- Outputs must include explicit next-step hints.
- Mutating operations must expose canonical/degraded status.

---

## 9) Pi integration hook matrix

| Hook/path | Current code anchor | Spec88 behavior | Failure prevented |
|---|---|---|---|
| session start/resume | `apps/pi-extension/src/session.ts` | fetch current WorkpointResumePacket; inject if relevant | raw transcript resume drift |
| before_agent_start | `apps/pi-extension/src/turns.ts` | add trust-workpoint behavioral law | model overriding packet with stale tail |
| context hook | `apps/pi-extension/src/turns.ts:52` | add WORKPOINT / ACTIVE_OBJECT_SET / ACTION_INTENT / VERIFICATION_HOOKS / DRIFT_BOUNDARIES | missing current action in minimal slice |
| session_before_compact | `apps/pi-extension/src/compaction.ts:76` | checkpoint workpoint and include packet in compact instructions | compaction summary omits continuation target |
| session_compact/end | `apps/pi-extension/src/compaction.ts:132`, `:191` | replace generic steer with WorkpointResumePacket | adjacent-thread continuation after compact |
| context overflow | provider/turn error path | checkpoint before retry/slim; resume from packet | context_length_exceeded recovery losing workpoint |
| model switch/fork | session/model lifecycle | refresh packet and provenance | model discontinuity drift |
| turn_end | `apps/pi-extension/src/turns.ts` | drift detection vs ActiveMissionSet/ActionIntent | notes-only or endpoint-only drift |

---

## 10) Tool/action contract summary

| Action | Side effects | Verification | Retry policy | Rollback/recovery |
|---|---|---|---|---|
| checkpoint workpoint | reducer event; active pointer update; optional ontology proposals | GET current confirms id/revision | retry only with same idempotency key after read | supersede with new checkpoint |
| resume workpoint | none, optional telemetry | packet includes source id | safe retry | no rollback needed |
| drift check preview | none | drift result returned | safe retry | no rollback needed |
| drift check emit | append event | event appears in trace/replay | no blind retry after unknown | corrective drift event or supersede |
| degraded fallback | local Pi/scratch entry | visible `canonical=false` | retry promotion after Focusa health returns | reconcile or discard fallback |

---

## 11) Phase 0 unresolved questions

1. Does existing ontology primitive projection already include equivalent ObjectTypes for `ActionIntent`, `VerificationRecord`, or `Blocker`?
2. Should `implemented_by` be canonical or alias to an existing file relation such as `defined_in`?
3. Should checkpoint endpoint directly promote deterministic WorkItem/Mission refs, or always propose a Workpoint first?
4. Should `WorkpointResumeRendered` be telemetry-only or reducer event?
5. Should drift-check event emission be a separate endpoint mode or separate `/emit-drift` endpoint?
6. Which Pi API event exposes provider `context_length_exceeded` reliably enough for automatic checkpoint-before-retry?

These questions do not block Phase 1 if documented as compatibility decisions in the relevant implementation bead.

---

## 12) Phase 0 acceptance result

Phase 0 is satisfied when this matrix is referenced by all implementation child beads and used as the vocabulary/API/tool contract source for Spec88 implementation.
