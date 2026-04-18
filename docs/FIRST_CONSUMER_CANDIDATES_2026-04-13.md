# First Consumer Candidates — 2026-04-13

Purpose:
- ground sparse/post-substrate docs in their first real runtime consumers
- identify where no honest consumer exists yet

## Provisional first consumers

### Doc 61 — domain-general cognition primitives
Selected first real consumer:
- Branch A routing substrate in `apps/pi-extension/src/turns.ts`, specifically current-ask/scope/relevance selection used to build turn context and exclusion traces

Consumer-path evidence anchors:
- `CURRENT_ASK` / `QUERY_SCOPE` slice construction in the turn context builder
- `relevant_context_selected` and `irrelevant_context_excluded` trace emission
- persisted exclusion reason/labels passed into work-loop context surfaces

Judgment:
- doc 61 primitives are authoritative only where they change routing/relevance behavior; free-floating cognition ontology remains out of scope

### Doc 71 — governing priors
Likely first consumer:
- none strong yet
- candidate future consumer: relevance ranking inside Branch A or bounded autonomy weighting in doc 78

Judgment:
- still mostly blocked until a concrete ranking/weighting consumer is chosen

### Doc 72 — identity / role fields
Likely first consumer:
- trace/routing permission semantics
- possible existing code hints in attachment role types and route validation surfaces

Judgment:
- must stay narrow and consumer-led

### Doc 73 — commitment lifecycle
Selected first real consumer:
- continuity handoff in `scripts/work_loop_watchdog.sh`, driven by `/v1/work-loop/status` → `commitment_lifecycle.release_semantics.state`

Consumer-path evidence anchors:
- watchdog only issues `select-next` when commitment release state is `released_on_completion`, `released_on_blocker`, or `released_or_unbound`
- blocked-task auto-advance path is explicitly gated on `released_on_blocker`
- release state source is `crates/focusa-api/src/routes/work_loop.rs` (`commitment_lifecycle_for_status`)

Judgment:
- doc 73 now has a real continuity consumer that changes task-handoff behavior from commitment state, without inventing a free-floating commitment object

### Doc 74 — reference resolution
Selected first real consumer:
- Focus-slice projection assembly in `apps/pi-extension/src/turns.ts`, where verified handle references are resolved into canonical `REFERENCE_ALIASES`
- trace-review path over `/v1/telemetry/trace` events, where `verification_result` emits `resolved_reference_count` and `resolved_reference_aliases`

Consumer-path evidence anchors:
- `apps/pi-extension/src/state.ts` → `buildCanonicalReferenceAliases` converts verified handle tuples into stable projection aliases
- `apps/pi-extension/src/turns.ts` emits `REFERENCE_ALIASES` in the operator projection slice
- `apps/pi-extension/src/turns.ts` emits `verification_result` trace payload fields `resolved_reference_count` / `resolved_reference_aliases` for review surfaces

Judgment:
- doc 74 now has concrete projection and trace-review consumers without inventing a detached identity-merging subsystem

### Doc 75 — projection / view semantics
Likely first consumer:
- bounded world / ontology projection surfaces
- later Pi bounded-view emission once Branch A matures

Existing anchor:
- `tests/ontology_world_contract_test.sh`

### Doc 76 — retention / decay
Selected first real consumer:
- focus-slice assembly in `apps/pi-extension/src/turns.ts`, where decisions/constraints are tiered into active (`DECISIONS` / `CONSTRAINTS`) vs decayed/historical (`DECAYED_CONTEXT` / `HISTORICAL_CONTEXT`) context surfaces
- command-path `memory.decay_tick` in `crates/focusa-api/src/routes/commands.rs` as the runtime decay substrate hook

Consumer-path evidence anchors:
- `apps/pi-extension/src/state.ts` → `retentionBucketsFromSelection` classifies selected vs decayed vs historical items from ranked relevance
- `apps/pi-extension/src/turns.ts` applies retention buckets for decisions/constraints before slice construction and emits retention-bucket counts in trace metadata
- `crates/focusa-api/src/routes/commands.rs` exposes `memory.decay_tick` for explicit decay execution

Judgment:
- doc 76 now has a concrete projection + command consumer path that reduces active dominance while preserving historical trace

### Doc 78 — bounded secondary cognition / persistent autonomy
Selected first real consumer:
- continuity handoff gate in `scripts/work_loop_watchdog.sh`, driven by `/v1/work-loop/replay/closure-bundle` (fallback `/v1/work-loop/replay/closure-evidence`) + `/v1/work-loop/status`

Consumer-path evidence anchors:
- watchdog reads replay consumer payload status, continuity gate state, and per-task pair flag (`secondary_loop_closure_replay_evidence.evidence.current_task_pair_observed`)
- watchdog auto `select-next` handoff requires `closure_replay_ready=true` before blocker/release-state continuation paths
- watchdog additionally fail-closes on continuation boundaries from status (`decision_context.operator_steering_detected` + `pause_flags.governance_decision_pending`) before issuing handoff
- replay payload source is `crates/focusa-api/src/routes/work_loop.rs` (`secondary_loop_replay_consumer_payload_for_status`)
- Pi operator surfaces (`apps/pi-extension/src/tools.ts` + `apps/pi-extension/src/commands.ts`) project replay consumer state with explicit `continuity_gate=open|fail-closed`
- TUI/API dashboard packaging (`crates/focusa-tui/src/views/work_loop.rs` + `/v1/work-loop/replay/closure-bundle`) projects replay consumer + gate semantics beyond closure-path internals

Judgment:
- doc 78 replay comparative evidence now changes live continuity behavior plus operator/TUI dashboard semantics instead of staying harness-only

### Doc 66 — affordances / execution environment
Likely first consumer:
- tool/action selection and route/tool contract planning

Judgment:
- should not start as a free-floating ontology; start when one action-selection consumer is chosen

### Docs 58-65 — visual/UI track
Likely first consumer:
- none honest yet beyond sparse UI/TUI rendering and generic view code

Judgment:
- still blocked on object/evidence consumer identification
- generic TUI rendering is not enough

## Decomposition rule from this pass

If a doc has no honest first consumer yet:
- it is decomposed but still blocked
- do not inflate it into pseudo-implementation
