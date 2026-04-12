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
- Gap:
  - no full end-to-end proof that Pi consumes a fully typed bounded working world beyond the minimal slice hot path

## 46. Ontology Core Primitives
- Status: PARTIAL
- Evidence:
  - typed primitives exist in `types.rs`
  - reducer replay invariants enforced in `reducer.rs`
  - proposal primitives exist in `types.rs` / `pre/mod.rs` / `pre/resolution.rs`
  - thread/thesis primitives exist in `types.rs` and API routes under `routes/threads.rs`
- Gap:
  - not all primitive families were exercised end-to-end in runtime during this pass

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
- Gap:
  - broader software-world coverage remains unproven beyond these thread/proposal/frame/memory/handle surfaces
  - source patch in `crates/focusa-api/src/routes/threads.rs` has not been rebuild-verified separately in this pass

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
- Gap:
  - verification hooks and ontology delta outputs are proven for some actions, not comprehensively for all canonical actions

## 49. Working Sets and Slices
- Status: VERIFIED (Pi hot path) / PARTIAL (broader ontology)
- Evidence:
  - `apps/pi-extension/src/turns.ts` now performs operator-first minimal-slice selection
  - `tests/channel_separation_test.sh` verifies legacy always-on injection removed
  - `tests/behavioral_alignment_test.sh` verifies minimal-slice/operator-first markers
- Gap:
  - broader non-Pi consumers of working sets/slices not exhaustively audited

## 50. Ontology Classification and Reducer
- Status: PARTIAL
- Evidence:
  - reducer is deterministic/event-driven in `crates/focusa-core/src/reducer.rs`
  - worker classification path exists in `crates/focusa-core/src/workers/executor.rs`
  - ambiguous/background/proposal semantics represented via `Proposal`, `PreState`, `pre/mod.rs`, `pre/resolution.rs`
  - runtime proposal path verified at the surface level:
    - `POST /v1/proposals` dispatches `Action::SubmitProposal`
    - daemon persists proposal state in `runtime/daemon.rs`
    - `POST /v1/proposals/resolve` runs deterministic scoring in `pre/resolution.rs`
- Gap:
  - doc-50's named ontology reducer events (`ontology_object_upsert_proposed`, `ontology_link_upsert_proposed`, `ontology_proposal_promoted`, etc.) were **not** found in code/tests during this pass; that part of the contract appears docs-only/unproven
  - current resolve path returns a scoring outcome but does not prove canonical ontology mutation from accepted proposals end-to-end
  - full classification → proposal → resolution → state mutation chain remains unproven in runtime during this pass

## 51. Ontology Expression and Proxy
- Status: VERIFIED (Pi extension hot path) / PARTIAL (Mode B proxy semantics)
- Evidence:
  - `apps/pi-extension/src/turns.ts` no longer injects always-on full focus state
  - operator intent is read before slice assembly
  - minimal applicable slice logic present
  - `tests/channel_separation_test.sh` passes with anti-hijack checks
  - prompt assembly route confirmed in `crates/focusa-api/src/routes/turn.rs` (`/v1/prompt/assemble`)
  - proxy adapters confirmed in:
    - `crates/focusa-core/src/adapters/openai.rs`
    - `crates/focusa-core/src/adapters/anthropic.rs`
- Gap:
  - Mode B proxy adapters still inject assembled prompt as system message wholesale; full operator-first/minimal-slice parity there was not exhaustively reworked in this pass

## 52. Pi Extension Contract
- Status: VERIFIED
- Evidence:
  - `tests/pi_extension_contract_test.sh` passes (20/20)

## 53. Pi Behavioral Alignment
- Status: VERIFIED (focused regression) / PARTIAL (full behavioral thesis)
- Evidence:
  - `tests/behavioral_alignment_test.sh` passes (12/12)
  - anti-hijack/operator-first checks added
- Gap:
  - still not a full behavioral eval against real Pi sessions doing golden tasks

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
- Status: VERIFIED (trace + checkpoint trigger surface) / PARTIAL (full recovery breadth)
- Evidence:
  - doc file located at `docs/56-trace-checkpoints-recovery.md`
  - `tests/trace_dimensions_test.sh` passes (17/17)
  - `tests/checkpoint_trigger_test.sh` rerun on 2026-04-12 → pass (11/11)
  - trace event types present in `types.rs`
  - checkpoint/resume runtime verified for:
    - session start
    - focus push / active frame visibility
    - blocker emergence / gate visibility
    - explicit session resume
    - state dump carrying checkpoint-critical state
- Gap:
  - full recovery semantics across restart/fork/compact/pre-shutdown cases were not re-audited in this pass

## 57. Golden Tasks and Evals
- Status: PARTIAL
- Evidence:
  - `tests/golden_tasks_eval.sh` rerun on 2026-04-12 → pass (16/16)
  - script explicitly labels itself as infrastructure verification only
  - current runtime confirms metrics/state/gate/steering surfaces required for eval harness presence
- Gap:
  - current script still proves infrastructure presence, not the full success condition from doc 57
  - comparative evals (`with Focusa` vs `without`, weaker-model comparisons, measurable action-quality improvement) remain unproven here

---

## Key Bottom Line

Fully verified in this pass:
- 51
- 52
- 54
- 54a
- 54b
- 56 (trace dimensions surface)

Partially verified / still broader than current proof:
- 45
- 46
- 47
- 48
- 49 (outside Pi hot path)
- 50
- 53 (full behavioral thesis)
- 55 impl
- 56 (checkpoint/recovery breadth)
- 57

This matrix should be treated as the current reality baseline, not prior bead/audit claims.
