# Ontology Addenda Implementation Matrix — 2026-04-12

Scope: docs 45–57

Legend:
- VERIFIED — code + tests/runtime support present
- PARTIAL — some implementation exists, but doc scope exceeds verified behavior
- DOCS-ONLY — concept exists in docs, not yet proven in code/runtime here

---

## 45. Ontology Overview
- Status: PARTIAL
- Evidence:
  - `crates/focusa-core/src/types.rs` contains typed software-world/cognition structures: `FocusState`, `Thread`, `Proposal`, `HandleRef`, `TelemetryEventType`
  - `crates/focusa-core/src/reducer.rs` enforces reducer/event-centric model
  - runtime ontology projection now exists at `GET /v1/ontology/world`
  - `tests/ontology_world_contract_test.sh` passes (15/15)
- Gap:
  - no full end-to-end proof that Pi consumes the broader typed ontology world beyond the minimal slice hot path

## 46. Ontology Core Primitives
- Status: PARTIAL
- Evidence:
  - typed primitives exist in `types.rs`
  - reducer replay invariants enforced in `reducer.rs`
  - proposal primitives exist in `types.rs` / `pre/mod.rs` / `pre/resolution.rs`
  - thread/thesis primitives exist in `types.rs` and API routes under `routes/threads.rs`
  - runtime primitive catalog now exists at `GET /v1/ontology/primitives`
  - `tests/ontology_world_contract_test.sh` verifies object/link/action catalogs, status vocabulary, and provenance classes
- Gap:
  - primitive catalogs are now runtime-visible, but not every primitive family is reducer-native/canonized as a first-class internal struct yet

## 47. Ontology Software World
- Status: PARTIAL
- Evidence:
  - object families exist: frames, artifacts, threads, proposals, memory, handles
  - identity/provenance fields present in `types.rs`
  - thread routes implemented in `crates/focusa-api/src/routes/threads.rs`
  - proposal routes implemented in `crates/focusa-api/src/routes/proposals.rs`
  - proposal runtime verified:
    - `focusa proposals submit --json` → accepted
    - `focusa proposals list --json` → pending proposal visible
    - `POST /v1/proposals/resolve` → returns deterministic resolution outcome
  - thread/proposal runtime regression added and rerun:
    - `tests/thread_runtime_test.sh` → pass (6/6)
    - verifies thread create/list/get consistency plus proposal submit/list/resolve basics
  - broader world projection now exists in `GET /v1/ontology/world`
  - `tests/ontology_world_contract_test.sh` verifies projected goal/active_focus/decision/constraint/failure/verification/artifact objects plus bounded working-set metadata
- Gap:
  - code-world object families like repo/package/module/file/symbol/route/schema/migration are cataloged but not yet populated from live project structure in this pass

## 48. Ontology Links + Actions
- Status: PARTIAL
- Evidence:
  - actions map to reducer-visible events in `reducer.rs`
  - command write model exists in `crates/focusa-api/src/routes/commands.rs`
  - focus/gate/session/ECS routes map to concrete actions in:
    - `routes/focus.rs`
    - `routes/gate.rs`
    - `routes/session.rs`
    - `routes/ecs.rs`
  - SPEC-55 contract gate rerun and passed
  - runtime action/link catalogs now exist in `GET /v1/ontology/primitives` and `GET /v1/ontology/world`
  - `tests/ontology_world_contract_test.sh` verifies typed links (`belongs_to_goal`, `blocks`, `verifies`) and action catalog presence for required action classes
- Gap:
  - verification hooks and ontology delta outputs are proven for some actions, not yet comprehensively for all declared action classes

## 49. Working Sets and Slices
- Status: VERIFIED (Pi hot path) / PARTIAL (broader ontology)
- Evidence:
  - `apps/pi-extension/src/turns.ts` now performs operator-first minimal-slice selection
  - `tests/channel_separation_test.sh` verifies legacy always-on injection removed
  - `tests/behavioral_alignment_test.sh` verifies minimal-slice/operator-first markers
- Gap:
  - broader non-Pi consumers of working sets/slices not exhaustively audited

## 50. Ontology Classification and Reducer
- Status: VERIFIED
- Evidence:
  - reducer is deterministic/event-driven in `crates/focusa-core/src/reducer.rs`
  - worker classification path exists in `crates/focusa-core/src/workers/executor.rs`
  - ambiguous/background/proposal semantics represented via `Proposal`, `PreState`, `pre/mod.rs`, `pre/resolution.rs`
  - runtime proposal path verified end-to-end:
    - `POST /v1/proposals` dispatches `Action::SubmitProposal`
    - daemon persists proposal state in `runtime/daemon.rs`
    - `POST /v1/proposals/resolve` runs deterministic scoring in `pre/resolution.rs`
    - accepted proposals canonically mutate state for focus/thesis/memory/autonomy/constitution paths
  - exact named ontology reducer/audit events now exist in `types.rs`, replay in `reducer.rs`, emit from `routes/proposals.rs`, and are enforced by `tests/ontology_event_contract_test.sh` (13/13)
  - proposal canonical mutation enforced by:
    - `tests/proposal_resolution_enforcement_test.sh` (5/5)
    - `tests/proposal_kind_enforcement_test.sh` (7/7)
    - `tests/proposal_governance_enforcement_test.sh` (7/7)
- Gap:
  - broader worker-classification internals were not separately benchmarked, but the reducer/proposal/event contract itself is runtime-verified

## 51. Ontology Expression and Proxy
- Status: VERIFIED
- Evidence:
  - `apps/pi-extension/src/turns.ts` no longer injects always-on full focus state
  - operator intent is read before slice assembly
  - minimal applicable slice logic present
  - `tests/channel_separation_test.sh` passes with anti-hijack checks
  - prompt assembly route confirmed in `crates/focusa-api/src/routes/turn.rs` (`/v1/prompt/assemble`)
  - Mode B proxy adapters now use shared operator-first minimal-slice assembly in:
    - `crates/focusa-core/src/adapters/openai.rs`
    - `crates/focusa-core/src/adapters/anthropic.rs`
  - strict proxy parity gate added:
    - `tests/proxy_mode_b_parity_test.sh` → pass (4/4)
  - focused unit tests pass in `focusa-core` for both adapters
- Gap:
  - none identified in the verified proxy/Pi expression path for this pass

## 52. Pi Extension Contract
- Status: VERIFIED
- Evidence:
  - `tests/pi_extension_contract_test.sh` passes (20/20)

## 53. Pi Behavioral Alignment
- Status: VERIFIED (strict regression) / PARTIAL (full behavioral thesis)
- Evidence:
  - `tests/behavioral_alignment_test.sh` now passes (17/17)
  - anti-hijack/operator-first checks added
  - Pi hot path now emits runtime consultation traces for:
    - `constraints_consulted`
    - `decisions_consulted`
    - `working_set_used`
    - `prior_mission_reused`
  - strict CI now runs `tests/behavioral_alignment_test.sh`
- Gap:
  - still not a full behavioral/comparative eval against real Pi sessions doing golden tasks

## 54. Pi Visible Output Boundary
- Status: VERIFIED
- Evidence:
  - `tests/channel_separation_test.sh` passes (14/14)
  - explicit anti-echo and anti-hijack checks added

## 54a. Operator Priority and Subject Preservation
- Status: VERIFIED (Pi hot path)
- Evidence:
  - `apps/pi-extension/src/turns.ts` suppresses irrelevant Focusa state on steering/direct-question turns
  - `subject_hijack_prevented` trace emitted
  - focused gates pass

## 54b. Context Injection and Attention Routing
- Status: VERIFIED (Pi hot path)
- Evidence:
  - minimal applicable slice after operator-input interpretation in `turns.ts`
  - broad always-on state injection removed
  - focused gates pass

## 55. Tool and Action Contracts
- Status: VERIFIED (strict gate)
- Evidence:
  - `tests/tool_contract_test.sh` rerun on 2026-04-12 → pass (12/12)
  - runtime surfaces validate schema/failure/idempotency/observability
  - command write model confirmed in `crates/focusa-api/src/routes/commands.rs`

## 55 impl
- Status: PARTIAL
- Evidence:
  - implementation notes reflected in tool contract tests and reducer/action surfaces
- Gap:
  - full doc-to-action matrix not exhaustively enumerated in this pass

## 56. Trace / Checkpoints / Recovery
- Status: VERIFIED
- Evidence:
  - doc file located at `docs/56-trace-checkpoints-recovery.md`
  - `tests/trace_dimensions_test.sh` now passes (23/23)
  - all 18 named trace dimensions are runtime-enforced via `TelemetryEventType`, `/v1/telemetry/trace`, and strict gate coverage:
    - `mission_frame_context`
    - `working_set_used`
    - `constraints_consulted`
    - `decisions_consulted`
    - `action_intents_proposed`
    - `tools_invoked`
    - `verification_result`
    - `ontology_delta_applied`
    - `blockers_failures_emitted`
    - `final_state_transition`
    - `operator_subject`
    - `active_subject_after_routing`
    - `steering_detected`
    - `prior_mission_reused`
    - `focus_slice_size`
    - `focus_slice_relevance_score`
    - `subject_hijack_prevented`
    - `subject_hijack_occurred`
  - `tests/checkpoint_trigger_test.sh` rerun on 2026-04-12 → pass (11/11)
  - `tests/restart_recovery_test.sh` → pass (14/14)
  - `tests/fork_compact_recovery_test.sh` → pass (11/11)
  - checkpoint/resume/runtime recovery verified for:
    - session start
    - focus push / active frame visibility
    - blocker emergence / gate visibility
    - explicit session resume
    - state dump carrying checkpoint-critical state
    - checkpoint file persistence before shutdown
    - frame/ASCC state restoration after daemon restart
    - explicit thread fork point materialization via `/v1/threads/{id}/fork`
    - explicit compact command producing CLT summary nodes while preserving checkpoint-visible state
  - `/v1/telemetry/trace?event_type=...` filtering now verified
  - tool-usage batches now also emit `tools_invoked` trace events
  - command write-model compatibility now covers explicit `ascc.checkpoint`, `compact`, and `micro-compact` paths used by the Pi extension

## 57. Golden Tasks and Evals
- Status: VERIFIED
- Evidence:
  - `tests/golden_tasks_eval.sh` rerun on 2026-04-12 → pass (16/16)
    - infrastructure surfaces remain enforced and explicitly labeled as infrastructure-only
  - `tests/continuous_pruning_test.sh` passes (4/4)
    - bounded-growth continuity/token-use evidence enforced in strict CI
  - `tests/golden_tasks_comparative_eval.sh` → pass (6/6)
    - same-budget `focusa` vs `baseline_raw` comparison is runtime-enforced
    - Focusa retained mission-critical markers under low budget (`4/4`) while raw baseline retained none (`0/4`)
    - Focusa retained more relevant context than raw baseline (`4 > 0`)
    - Focusa reduced irrelevant raw-baseline context markers (`0 < 2` in strict run)
    - weaker-model/low-budget pressure now has explicit proof that the raw baseline truncates earlier while Focusa keeps the bounded mission slice
  - `/v1/prompt/assemble` now supports explicit eval strategies (`focusa`, `baseline_raw`) for auditable with-vs-without comparison
  - prompt assembly now degrades constitution context before mission semantics, preserving active focus/decision/constraint retention more faithfully under tight budgets

---

## Key Bottom Line

Fully verified in this pass:
- 50
- 51
- 52
- 54
- 54a
- 54b
- 56
- 57

Partially verified / still broader than current proof:
- 45
- 46
- 47
- 48
- 49 (outside Pi hot path)
- 53 (full behavioral thesis)
- 55 impl

This matrix should be treated as the current reality baseline, not prior bead/audit claims.
