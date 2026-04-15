# Rebaseline Single-Writer Summary — 2026-04-13

## Goal
Rebase Focusa back to spec authority: daemon-owned single-writer state transitions, reducer-expressed canonical mutation, no mixed API/daemon canonical writers.

## Checkpoints

### 1. Root cause identified
- Mixed canonical writers existed: sync API routes mutated shared canonical state while daemon retained its own stale in-memory snapshot.
- This violated the single-writer reducer contract from `docs/core-reducer.md` and the daemon ownership model from `docs/G1-detail-03-runtime-daemon.md`.
- Symptom pattern: accepted/applied responses followed by stale canonical reads in CI.

### 2. Immediate stabilization shipped
- Added serialized writer coordination and daemon reconciliation so stale daemon state could not overwrite externally mutated shared state.
- Commit previously landed and passed remote CI.
- This restored read-after-write consistency but was recognized as an interim architectural correction, not full spec purity.

### 3. Rebaseline bead tranche created
- Parent epic: `focusa-iqz8`.
- Tranche covers:
  - canonical route rebasing
  - sync transfer rebasing
  - canonical-vs-ephemeral classification
  - daemon-side reducer bypass cleanup
  - regression guardrails/docs

### 4. Canonical route rebasing landed
Converted these route mutations to daemon/reducer-backed events:
- `POST /v1/proposals` → `FocusaEvent::ProposalSubmitted`
- `POST /v1/proposals/resolve` → reducer/event flow for proposal status + accepted domain effects
- `POST /v1/constitution/load` → `FocusaEvent::ConstitutionLoaded`
- `POST /v1/threads/:id/fork` → `FocusaEvent::ThreadForked`

New/expanded reducer-backed events introduced:
- `ProposalSubmitted`
- `ProposalStatusChanged`
- `ConstitutionLoaded`
- `ThreadForked`
- `ThreadThesisUpdated`
- `AutonomyAdjusted`

### 5. Sync transfer rebasing landed
- `POST /v1/sync/transfer` no longer applies canonical state in-route via direct reducer + shared-state write.
- Route now persists inbound ownership transfer event and replays it through daemon-owned `Action::EmitEvent`.
- Visibility is polled from canonical state after dispatch.

### 6. Redundant turn/proxy direct writes reduced
Removed/reduced redundant direct route writes that duplicated reducer-managed `active_turn` mutation:
- `turn_start` now waits for reducer-applied visibility instead of writing `active_turn` directly.
- proxy turn-start/turn-complete redundant `active_turn` direct writes/clears removed where reducer events already own them.

### 7. Verification checkpoints passed
Verified during this tranche:
- `cargo test`
- `tests/fork_compact_recovery_test.sh`
- `tests/proposal_submit_contract_test.sh`
- `tests/proposal_governance_enforcement_test.sh`
- `tests/proposal_resolution_enforcement_test.sh`
- `tests/proposal_kind_enforcement_test.sh`
- full `./scripts/ci/run-spec-gates.sh`

### 7b. Additional proxy/turn checkpoint
- Messages proxy redundant `active_turn` direct start/clear writes removed where `TurnStarted`/`TurnCompleted` events already own the canonical lifecycle.
- Proxy pre-turn Mem0/wiki memory injection now dispatches `Action::UpsertSemantic` instead of mutating `FocusaState.memory` directly in route code.
- This continues moving proxy-side durable cognition updates under daemon/reducer ownership while leaving unresolved ephemeral prompt-assembly staging for later classification.

### 7c. Guardrail checkpoint
- Added `tests/canonical_writer_guardrail_test.sh`.
- Added the guardrail test to `scripts/ci/run-spec-gates.sh`.
- Guardrail asserts rebased canonical routes (`proposals`, `constitution`, `threads`, `sync_transfer`) do not regress to direct shared-state writes and that daemon thesis/semantic updates remain event-backed.
- Full strict spec gates still pass with the guardrail enabled.

### 7d. Additional proxy cleanup checkpoint
- Removed another redundant proxy-side `active_turn` clear at completion where `TurnCompleted` already owns canonical turn teardown.
- Marked proxy-side contradiction resolution as an explicit remaining maintenance-path deviation to address in a later reducer-backed cleanup tranche.
- Proxy and turn prompt-assembly writes that remain are now explicitly treated as ephemeral session staging, not canonical cognition mutations.

### 8. Remaining rebaseline work in flight
Still to finish for full alignment:
- classify remaining direct writes in `proxy`, `turn`, `training`, `telemetry`
- convert canonical ones to daemon/reducer flow
- isolate/document truly ephemeral state
- convert remaining daemon-side direct mutations (telemetry counters, stale active_turn expiry, procedural memory maintenance where required by spec) into reducer-backed events where spec requires
- add regression guardrails so mixed canonical writers cannot re-enter

### 9. Additional checkpoint — daemon reducer bypasses reduced
- `Action::UpdateThesis` now emits `ThreadThesisUpdated` instead of mutating thread state directly inside daemon translation.
- `Action::UpsertSemantic` now emits `SemanticMemoryUpserted` and relies on reducer application instead of mutating memory directly in `translate_action`.
- Proposal resolution now waits for kind-specific canonical visibility after dispatch so strict gates verify reducer-owned state after acceptance, not transient in-flight command enqueue.
- This further narrows daemon-side spec deviations after the route-level rebases.

## Current classification snapshot
### Clearly canonical / must be daemon-owned
- proposals + proposal resolution
- constitution load/revision
- thread fork
- thread ownership transfer sync import
- proxy paths that upsert semantic memory or mutate durable cognition
- daemon-side thesis updates
- daemon semantic upserts

### Likely redundant or reducer-owned already
- turn start/complete `active_turn` initialization/clearing when mirrored by `TurnStarted`/`TurnCompleted`
- messages/openai proxy direct start/clear writes that duplicated turn lifecycle events

### Currently classified as ephemeral/session-local staging
- `turn` prompt assembly writes to `active_turn.raw_user_input` / `active_turn.assembled_prompt`
- `turn_append` streaming chunk accumulation in `active_turn.assembled_prompt`
- proxy prompt-assembly staging into `active_turn.assembled_prompt`

### Currently classified as non-canonical but still living in FocusaState
- telemetry route counters + trace buffers in `telemetry`
- training contribution toggles/approvals/submission queue mutations in `contribution`

### Rebaseline implication
- Canonical cognition mutations continue to be moved under daemon/reducer ownership immediately.
- Ephemeral/non-canonical state still living inside `FocusaState` needs a follow-up architectural decision: either formalize as reducer-managed runtime state or extract from canonical state entirely.

### 10. Live remaining-write inventory
Current direct route writes still present after this tranche slice:
- `proxy.rs`
  - contradiction resolution over memory
  - prompt-assembly staging into `active_turn.assembled_prompt`
- `turn.rs`
  - prompt assembly / append staging into `active_turn`
- `telemetry.rs`
  - telemetry counters and trace buffers
- `training.rs`
  - contribution queue + enable/pause state

Interpretation:
- canonical writer regressions already addressed by guardrails
- remaining writes are either explicitly classified ephemeral/non-canonical or queued for later architectural extraction/reducer formalization

### 11. Continuous-work master spec checkpoint
- Added `docs/79-focusa-governed-continuous-work-loop.md`.
- Spec establishes daemon-owned continuation policy, Pi RPC/SDK as transport, reducer purity boundaries, extension-thin role, canonical-vs-runtime split, stop conditions, policy bundle, and implementation phases.
- Spec is explicit that unattended continuation must not rely on etiquette or manual prodding; the system needs an outer runtime loop.

### 12. Continuous-work citation/enrichment checkpoint
- Ran a second-pass audit across Pi RPC/extensions docs and Focusa specs/codebase.
- Enriched `docs/79-focusa-governed-continuous-work-loop.md` with source-backed claims covering: Pi RPC event/command model, extension lifecycle limits, current-ask/scope/minimal-slice rules, operator-priority law, trace/checkpoint recovery, eval expectations, existing `active_turn`/`TurnStarted`/`TurnCompleted`/`PromptAssembled` surfaces, and the Pi extension two-file scratchpad model.
- Spec now cites each major section so implementation can stay aligned with both Pi harness capabilities and Focusa philosophy/runtime boundaries.

### 13. Continuous-work anti-footgun refinement checkpoint
- Refined the master spec to require intelligent and intuitive behavior, not merely persistence.
- Added preset modes, sensible defaults, low-productivity detection, adaptive pacing, explainable continuation reasons, no-busy-loop rules, visible status requirements, and graceful operator-interrupt handling.
- Acceptance criteria now explicitly require usability without heavy tuning and pausing instead of spinning when work is no longer productive.

### 14. Continuous-work blocker-intelligence checkpoint
- Added blocker taxonomy and intelligent blocker handling requirements to `docs/79-focusa-governed-continuous-work-loop.md`.
- Spec now requires self-recovery, fallback, blocked-task deferral, alternate-ready-task advancement, and high-quality blocker packages before whole-project stop/escalation.
- Safety/acceptance criteria now explicitly forbid declaring the whole project blocked when only one task is blocked and valid alternate work remains.

### 15. Continuous-work project-executor checkpoint
- Expanded `docs/79-focusa-governed-continuous-work-loop.md` from turn-loop framing to project-executor framing.
- Spec now explicitly covers single-tranche and multi-tranche execution, ordered `bd` work-graph traversal, automatic task/tranche advancement, no-reprompt guarantee, and effectively unlimited turns while valid scoped work remains.
- Acceptance criteria now require end-to-end project completion behavior rather than merely turn-to-turn continuation.

### 16. Docs-first + BD-first integration checkpoint
- Integrated the project workflow directly into the master spec.
- `docs/79-focusa-governed-continuous-work-loop.md` now explicitly treats authoritative docs/specs as correctness authority and ordered `bd` items as the default execution graph.
- Added BD execution contract, docs-first completion/verification rules, ready-item traversal requirements, and a prohibition against treating `bd` as optional decoration in BD-first workflow mode.

### 17. Anti-drift spec-supremacy checkpoint
- Tightened the master spec so clearly defined linked specs outrank worker-preferred implementation paths.
- Completion semantics now require implementation behavior to match authoritative specs, not just partial functional progress.
- Added anti-drift rule forbidding the executor from inventing a substitute target and declaring success against it.

### 18. Spec-over-BD hierarchy checkpoint
- Clarified that `bd` items are decomposed from the supreme spec and should normally be trusted as the operational guide.
- Also made the hierarchy explicit: when bead guidance is ambiguous or appears inconsistent, the executor must return to the authoritative spec rather than improvising from the bead alone.

### 19. Absolute spec-supremacy checkpoint
- Tightened the hierarchy language to the operator's intended absolute rule.
- `docs/79-focusa-governed-continuous-work-loop.md` now states that `bd` NEVER outranks spec and that all bead/spec conflicts are resolved by the authoritative spec.

### 20. Supreme-spec authority checkpoint
- Further strengthened the master spec so the authoritative spec is explicitly the supreme normative source for execution, interpretation, completion, and conflict resolution.
- Added language forbidding any worker judgment, convenience, momentum, or local heuristic from superseding a clearly defined spec requirement.

### 21. Operator/spec-author cascade checkpoint
- Added the authority ladder explicitly: operator/spec author → spec → `bd` decomposition → code composition → app functionality.
- Spec now requires operator/spec-author amendments to cascade downward through decomposition, implementation targets, verification, and resulting behavior rather than leaving stale beads or stale code plans in place.

### 22. No-LLM-decision-authority checkpoint
- Added an explicit rule that the LLM/worker has execution capability and proposal capability, but no governing decision authority.
- The master spec now forbids the LLM from overruling the operator/spec author, the spec, or consistent spec-derived decomposition, and forbids self-granted completion or deviation authority.

### 23. Direct-author communication checkpoint
- Clarified that the LLM does have direct communication with the operator/spec author for clarification, confirmation, and amendment requests.
- Also clarified that this direct communication channel does not confer governing authority; authoritative decisions still flow from operator/spec author into spec and then cascade downward.

### 24. Delegated-authorship trust-level checkpoint
- Added future delegated spec authorship as an explicit governed possibility.
- The master spec now allows spec-independent authorship only when explicitly delegated by the operator/current spec author and permitted by Focusa trust levels and scope policy.
- Without explicit delegation, the LLM remains a non-authoring executor/proposal source.

### 25. Practical-gap refinement checkpoint
- Ran another refinement pass to close practical execution gaps that could make the system feel stupid despite good architecture.
- Added spec-linked task packets, task-class verification matrix, task-switch context reset, run identity model, worker capability/model-routing guidance, budget hierarchy, blocker-package quality requirements, and git/worktree discipline.
- Acceptance criteria now also require fidelity-first worker selection, correct context reset on task switches, and preservation of unrelated worktree state without destructive git behavior.

### 26. Cross-layer Focusa integration checkpoint
- Ran another integration-focused pass to ensure the continuous-work master spec is anchored at every Focusa layer rather than existing as a floating loop design.
- Added explicit layer contracts (reducer/daemon/API/ontology/harness), shared lifecycle-status alignment, proposal-aware continuation rules, ontology-shaped execution requirements, API/daemon boundary rules, and degraded-mode integration behavior.
- Result: the spec now reads more like a Focusa-native runtime architecture than a Pi-centric auto-continue wrapper.

### 27. Highest-value simplification checkpoint
- Ran a simplification pass to compress the master spec toward the smallest implementation that still delivers the operator outcome.
- Added an explicit highest-value kernel, a deferred-by-default complexity section, kernel-first implementation phases, and a “keep the first cut minimal” rule in the module-shape section.
- The spec now more clearly separates must-have end-to-end autonomy from lower-priority sophistication that can wait until after the kernel works.

### 28. Readiness-to-decompose checkpoint
- Added a decomposition-readiness section to `docs/79-focusa-governed-continuous-work-loop.md`.
- The spec now explicitly states the readiness criteria, the Phase-1 implementation slices, the decomposition guardrails, and the first end-to-end proof target.
- Result: implementation can now follow from a concrete kernel decomposition rather than another round of architecture debate.

### 29. Full BD decomposition checkpoint
- Created full `.beads/` decomposition for spec 79 under root epic `focusa-3d7d`.
- Decomposition includes epic → child BD → grandchild BD hierarchy across authority substrate, daemon supervisor, transport/API, verification/anti-drift, recovery/observability, safety/governance, and deferred sophistication.
- Added index document `docs/79_CONTINUOUS_WORK_LOOP_BD_DECOMPOSITION_2026-04-13.md` mapping the full hierarchy back to the master spec.

### 30. Implementation start checkpoint
- Began implementation under `focusa-3d7d.1` / `focusa-3d7d.1.1`.
- Added first continuous-work core substrate to `crates/focusa-core/src/types.rs`: run identities, policy/preset types, task/blocker/work-loop enums, worker capability profile, spec-linked task packet, and canonical `FocusaState.work_loop`.
- `cargo test` remains green after the new type substrate landed.

### 31. Next tranche implementation checkpoint
- Continued implementation through `focusa-3d7d.1.3` / `focusa-3d7d.1.6`.
- Added canonical continuous-work lifecycle events to `crates/focusa-core/src/types.rs` and reducer-backed handling in `crates/focusa-core/src/reducer.rs` for loop enable/disable, request/start/observe/complete, pause/block/escalate, tranche complete, budget exhaustion, transport degradation, resume, and recovery checkpoint updates.
- `cargo test` remains green after reducer/event integration.

### 32. Daemon supervisor tranche checkpoint
- Continued into `focusa-3d7d.2.1` / `focusa-3d7d.2.5`.
- Added first daemon supervisor action plumbing in `crates/focusa-core/src/types.rs` and `crates/focusa-core/src/runtime/daemon.rs` for enabling, pausing, resuming, stopping, requesting next turn, observing turn outcome, checkpointing, and degraded transport signaling.
- These daemon actions now translate into reducer-backed `Continuous*` events; `cargo test` remains green.

### 33. BD traversal/spec-packet tranche checkpoint
- Continued into `focusa-3d7d.2.6` / `focusa-3d7d.2.7`.
- Added `ContinuousWorkItemSelected` plus daemon `SetContinuousWorkItem` plumbing so the loop can canonically select/store the current spec-linked work item.
- Added minimal authoritative-spec grounding enforcement: daemon rejects continuous work items that lack `linked_spec_refs` before selection; `cargo test` remains green.

### 34. Transport/API tranche checkpoint
- Continued into `focusa-3d7d.3.1` / `focusa-3d7d.3.2`.
- Added `crates/focusa-api/src/routes/work_loop.rs` with minimal `/v1/work-loop` control/status routes and wired it into `routes/mod.rs` + `server.rs`.
- API now exposes status plus enable/pause/resume/stop dispatch into daemon-owned loop actions; `cargo test` remains green.

### 35. Verification/anti-drift tranche checkpoint
- Continued into `focusa-3d7d.4.1` / `focusa-3d7d.4.3`.
- Added spec-linked task packet helper methods plus outcome-time enforcement in daemon action translation so required verification and spec conformance are checked before emitting `ContinuousTurnCompleted`.
- Non-verified outcomes now block with `BlockerClass::Verification`; spec-nonconformant outcomes now block with `BlockerClass::SpecGap`; `cargo test` remains green.

### 36. Recovery/observability tranche checkpoint
- Continued into `focusa-3d7d.5.1` / `focusa-3d7d.5.3`.
- Added reducer-backed `last_observed_summary` state plus API checkpoint/status improvements in `crates/focusa-api/src/routes/work_loop.rs`.
- `/v1/work-loop` now exposes richer operator status, and `/v1/work-loop/checkpoint` can persist loop checkpoints through daemon-owned actions; `cargo test` remains green.

### 37. Automatic subtask-selection tranche checkpoint
- Extended the BD traversal kernel with `SelectNextContinuousSubtask` in `crates/focusa-core/src/types.rs` and `crates/focusa-core/src/runtime/daemon.rs`.
- Daemon can now shell `bd show --json`, select the next open/in-progress dependent under a parent work item, infer a task class, ground the packet to spec 79, and emit `ContinuousWorkItemSelected` canonically.
- Added `POST /v1/work-loop/select-next` in `crates/focusa-api/src/routes/work_loop.rs`; `cargo test` remains green.

### 38. Safety/governance tranche checkpoint
- Continued directly on `focusa-3d7d.6`, `focusa-3d7d.6.1`, `focusa-3d7d.6.2`, and `focusa-3d7d.6.3` with docs/79-backed API guards.
- `crates/focusa-api/src/routes/work_loop.rs` now enforces explicit approval for enable, clean-git-worktree checks for autonomous mutations, and `x-focusa-writer-id` single-writer claims to block parallel authority.
- `crates/focusa-api/src/server.rs` plus API test builders now track `active_writer` in shared app state; `GET /v1/work-loop` exposes current writer claim; `cargo test` remains green.

### 39. Richer work-loop status tranche checkpoint
- Continued on `focusa-3d7d.7.4` using docs/79 outcome 6 (visible status + checkpoint/resume) without introducing new authority paths.
- `GET /v1/work-loop` now reports governance preflight requirements and live git worktree readiness, so operators can see approval/writer constraints and dirty-worktree blockers before mutating the loop.
- `cargo test` remains green.

### 40. Degraded-mode control tranche checkpoint
- Continued on `focusa-3d7d.7.1` using docs/79 `TransportDegraded` / `ContinuousLoopTransportDegraded` lifecycle surfaces.
- Added `POST /v1/work-loop/degraded` so the API can dispatch canonical `MarkContinuousLoopTransportDegraded` actions into the daemon/reducer path instead of mutating state ad hoc.
- `cargo test` remains green.

### 41. Degraded execution-path tranche checkpoint
- Continued on `focusa-3d7d.7.5`, `focusa-3d7d.7.5.1`, and `focusa-3d7d.7.5.2` using docs/79 degraded-mode behavior.
- `crates/focusa-core/src/runtime/daemon.rs` now adapts selected task packets while the loop is `TransportDegraded`: scope is narrowed, verification tier is heightened, and checkpoint guidance is injected before autonomous execution continues.
- `cargo test` remains green.

### 42. Safe degraded ready-work tranche checkpoint
- Completed `focusa-3d7d.7.5.3` as part of the degraded execution bundle.
- While `TransportDegraded`, BD dependent selection now prefers safe open/in-progress work and defers risky/destructive titles instead of continuing blindly.
- `cargo test` remains green.

### 43. Fidelity-first worker routing tranche checkpoint
- Completed `focusa-3d7d.7.2` using docs/79 degraded-mode guidance to select a better-fit worker/model.
- `crates/focusa-core/src/reducer.rs` now assigns an `active_worker` recommendation by task class whenever continuous work is selected, and degraded mode biases the profile toward bounded slower/safer execution.
- `cargo test` remains green.

### 44. Advanced worker selection tranche checkpoint
- Completed `focusa-3d7d.7.6`, `focusa-3d7d.7.6.1`, `focusa-3d7d.7.6.2`, and `focusa-3d7d.7.6.3` using docs/79 guidance on better-fit worker/model selection under degradation.
- `crates/focusa-core/src/reducer.rs` now tracks consecutive task-class failures, upgrades worker recommendations to fallback profiles after repeated blocked outcomes, and keeps routing bounded to kernel-authoritative signals while leaving spec-linked task packets untouched.
- `GET /v1/work-loop` now exposes `consecutive_failures_for_task_class`; `cargo test` remains green.

### 45. Delegated-authorship tranche checkpoint
- Completed `focusa-3d7d.7.7`, `focusa-3d7d.7.7.1`, `focusa-3d7d.7.7.2`, and `focusa-3d7d.7.7.3` using docs/79 delegated authorship constraints.
- Added approved delegation control routes in `crates/focusa-api/src/routes/work_loop.rs`, canonical delegation events/state in `crates/focusa-core/src/types.rs` + `crates/focusa-core/src/reducer.rs`, and downward cascade of authoritative amendment summaries into selected task packets in `crates/focusa-core/src/runtime/daemon.rs`.
- `GET /v1/work-loop` now exposes `delegated_authorship`; `cargo test` remains green.

### 46. Governance pause-flags tranche checkpoint
- Completed `focusa-3d7d.6.6`, `focusa-3d7d.6.6.1`, `focusa-3d7d.6.6.2`, and `focusa-3d7d.6.6.3` to represent destructive confirmations, governance-sensitive proposal decisions, and explicit operator overrides as canonical pause flags.
- Added `pause_flags` state plus `POST /v1/work-loop/pause-flags`, and `GET /v1/work-loop` now surfaces the current governance pause state directly.
- `cargo test` remains green.

### 47. No-parallel-authority tranche checkpoint
- Completed `focusa-3d7d.6.7`, `focusa-3d7d.6.7.1`, `focusa-3d7d.6.7.2`, and `focusa-3d7d.6.7.3` by making role boundaries explicit in the work-loop governance surface.
- `GET /v1/work-loop` now states that daemon owns policy, API is dispatch/observability only, extension is bridge-only, and the LLM remains executor-only unless explicit delegation is approved.
- `cargo test` remains green.

### 48. Git/worktree discipline tranche checkpoint
- Completed `focusa-3d7d.6.5`, `focusa-3d7d.6.5.1`, `focusa-3d7d.6.5.2`, `focusa-3d7d.6.5.3`, and `focusa-3d7d.6.4` using work-loop preflight/status surfaces.
- `GET /v1/work-loop` now includes `git diff --stat`, sample unrelated changes, explicit forbidden destructive git operations, and explicit operator-override supersedence in governance metadata.
- `cargo test` remains green.

### 49. High-quality blocker package tranche checkpoint
- Completed `focusa-3d7d.5.8`, `focusa-3d7d.5.8.1`, `focusa-3d7d.5.8.2`, and `focusa-3d7d.5.8.3` using docs/79 blocker package quality requirements.
- `GET /v1/work-loop` now emits a `blocker_package` containing blocker class, affected work item, linked spec requirement, recovery/fallback attempts, alternate-ready-work availability, exact operator decision needed, and recommended next action.
- `cargo test` remains green.

### 50. Operator-visible status tranche checkpoint
- Completed `focusa-3d7d.5.7`, `focusa-3d7d.5.7.1`, `focusa-3d7d.5.7.2`, and `focusa-3d7d.5.7.3` to fill out the status model promised by docs/79.
- `GET /v1/work-loop` now includes explicit project/tranche status, last checkpoint id, grouped blocker context, transport health, and remaining budget alongside current work item state.
- `cargo test` remains green.

### 51. Run identity tranche checkpoint
- Completed `focusa-3d7d.5.5`, `focusa-3d7d.5.5.1`, `focusa-3d7d.5.5.2`, and `focusa-3d7d.5.5.3` to strengthen run identity continuity.
- `crates/focusa-core/src/reducer.rs` now assigns task/tranche/worker identities during work selection and clears worker session identity on completion, while `GET /v1/work-loop` exposes an explicit `identity_summary` surface.
- `cargo test` remains green.

### 52. Checkpoint/resume tranche checkpoint
- Completed `focusa-3d7d.5.1`, `focusa-3d7d.5.2`, `focusa-3d7d.5.6`, `focusa-3d7d.5.6.1`, `focusa-3d7d.5.6.2`, and `focusa-3d7d.5.6.3`.
- `crates/focusa-core/src/runtime/daemon.rs` now emits automatic checkpoint events on enable, work-item switch, pause, verification success, and blocker transitions; `crates/focusa-core/src/reducer.rs` stores last safe re-entry basis and restored context summaries; `GET /v1/work-loop` now exposes `resume_payload` for safe resume.
- `cargo test` remains green.

### 53. Close-check + anti-drift tranche checkpoint
- Completed `focusa-3d7d.4.6`, `focusa-3d7d.4.6.1`, `focusa-3d7d.4.6.2`, `focusa-3d7d.4.6.3`, `focusa-3d7d.4.7`, `focusa-3d7d.4.7.1`, `focusa-3d7d.4.7.2`, and `focusa-3d7d.4.7.3`.
- `crates/focusa-core/src/runtime/daemon.rs` now blocks completion for stale or underspecified work items, auto-populates acceptance criteria on selection, and checkpoints stale bead/spec mismatch detection; `crates/focusa-core/src/reducer.rs` records BD transition ids and forces replan-required pause semantics when authoritative spec amendments arrive.
- `GET /v1/work-loop` now exposes `last_recorded_bd_transition_id`; `cargo test` remains green.

### 54. Verification matrix + completion semantics tranche checkpoint
- Completed the remaining verification/completion epic: `focusa-3d7d.4`, `focusa-3d7d.4.1`, `focusa-3d7d.4.2`, `focusa-3d7d.4.3`, `focusa-3d7d.4.4`, `focusa-3d7d.4.5`, `focusa-3d7d.4.5.1`, `focusa-3d7d.4.5.2`, and `focusa-3d7d.4.5.3`.
- `crates/focusa-core/src/runtime/daemon.rs` now assigns task-class-specific verification tiers and emits `ContinuousTrancheCompleted` when verified completion exhausts ready work in a tranche; `GET /v1/work-loop` maps project/tranche progression more explicitly.
- `cargo test` remains green.

### 55. Transport session + event ingestion tranche checkpoint
- Completed the remaining transport/API kernel work: `focusa-3d7d.3.5`, `focusa-3d7d.3.5.1`, `focusa-3d7d.3.5.2`, `focusa-3d7d.3.5.3`, `focusa-3d7d.3.6`, `focusa-3d7d.3.6.1`, `focusa-3d7d.3.6.2`, `focusa-3d7d.3.6.3`, `focusa-3d7d.3.7`, `focusa-3d7d.3.7.1`, `focusa-3d7d.3.7.2`, `focusa-3d7d.3.7.3`, `focusa-3d7d.3.3`, `focusa-3d7d.3.4`, `focusa-3d7d.3.1`, and `focusa-3d7d.3.2`.
- Added attach/abort/event-ingestion transport routes plus canonical transport session/event state in the reducer; event ingestion is serialized through `write_serial_lock`, and status now exposes transport adapter/session health, last event, and ordered event sequence.
- `cargo test` remains green.

### 56. Supervisor packet/progression tranche checkpoint
- Completed the remaining daemon supervisor epic work: `focusa-3d7d.2`, `focusa-3d7d.2.7`, `focusa-3d7d.2.7.1`, `focusa-3d7d.2.7.2`, `focusa-3d7d.2.7.3`, `focusa-3d7d.2.8`, `focusa-3d7d.2.8.1`, `focusa-3d7d.2.8.2`, and `focusa-3d7d.2.8.3`.
- Spec-linked task packets now always carry linked spec refs, acceptance/verification/scope payloads, and authoritative-grounding rejection; blocker-aware progression now recommends ordered self-recovery, sibling-ready-work deferral, and escalation only when retries and alternate work are exhausted.
- `cargo test` remains green.

### 57. Supervisor lifecycle + auto-advance tranche checkpoint
- Completed the remaining lifecycle/traversal work: `focusa-3d7d.2.5`, `focusa-3d7d.2.5.1`, `focusa-3d7d.2.5.2`, `focusa-3d7d.2.5.3`, `focusa-3d7d.2.6`, `focusa-3d7d.2.6.1`, `focusa-3d7d.2.6.2`, and `focusa-3d7d.2.6.3`.
- `crates/focusa-core/src/runtime/daemon.rs` now resolves next ready packets from BD, best-effort claims them into `in_progress`, and auto-advances to the next ready sibling after verified completion without reprompt; status/blocker surfaces distinguish task-blocked vs project-blocked contexts.
- `cargo test` remains green.

### 58. Authority/policy substrate completion checkpoint
- Completed the remaining substrate work: `focusa-3d7d.1`, `focusa-3d7d.1.1`, `focusa-3d7d.1.2`, `focusa-3d7d.1.3`, `focusa-3d7d.1.4`, `focusa-3d7d.1.5`, `focusa-3d7d.1.5.1`, `focusa-3d7d.1.5.2`, `focusa-3d7d.1.5.3`, `focusa-3d7d.1.6`, `focusa-3d7d.1.6.1`, `focusa-3d7d.1.6.2`, `focusa-3d7d.1.6.3`, `focusa-3d7d.1.7`, `focusa-3d7d.1.7.1`, `focusa-3d7d.1.7.2`, and `focusa-3d7d.1.7.3`.
- The canonical type substrate now covers work-loop state, policy defaults, run/checkpoint identities, event schema, transport/delegation state, and operator-supremacy / executor-only governance controls.
- `cargo test` remains green.

### 59. Frame-write fidelity checkpoint
- Reopened and repaired spec-79 closure drift through `focusa-3d7d.8`: `/v1/focus/update` now honors explicit `frame_id`/`turn_id`, returns accepted `frame_id`, and the Pi bridge now treats `accepted`/`no_active_frame`/`rejected` as distinct outcomes instead of assuming success from any JSON body.
- Added `tests/focus_frame_write_contract_test.sh` and wired it into `scripts/ci/run-spec-gates.sh`; the live daemon on `:8787` was restarted and verified with the new frame-write contract.
- `cargo test` remains green.

### 60. Spec-79 dependency-fidelity reconciliation checkpoint
- Completed `focusa-3d7d.9.1` through `focusa-3d7d.9.4`: Pi session persistence now uses resumable `focusa-wbm-state` with `sessionId`; operator-priority/minimal-slice routing now gates mission reuse by explicit continuation/relevance and emits `subject_hijack_prevented`; tool contracts now strictly gate write-failure taxonomy, status-envelope handling, and scratchpad fallback mirroring.
- Strengthened proof surfaces in `tests/pi_extension_contract_test.sh`, `tests/behavioral_alignment_test.sh`, `tests/scope_routing_regression_eval.sh`, and `tests/tool_contract_test.sh`, and updated `docs/IMPLEMENTATION_STATUS_MATRIX_2026-04-13.md` to reflect the now-implemented 52/53/54a/54b/55/57/67/68 surfaces.
- `cargo test` and the stricter live script gates remain green.

### 61. Spec-79 direct audit continuation-input checkpoint
- Completed `focusa-3d7d.10`: added canonical `WorkLoopDecisionContext` to core state, a daemon action/event path for current-ask/scope/excluded-context updates, and `/v1/work-loop/context` so Pi can publish continuation inputs into daemon-owned state.
- `/v1/work-loop` status now exposes `continuation_inputs` with current ask/scope, proposal pressure, autonomy level, next-work risk class, budget caps, operator overrides, and checkpoint posture; `tests/work_loop_continuation_inputs_test.sh` was added and wired into `scripts/ci/run-spec-gates.sh`.
- `cargo test` and the live continuation-input gate remain green.

### 62. Literal spec-79 coverage closure checkpoint
- Completed `focusa-3d7d.11.1` through `focusa-3d7d.11.4`: daemon continuation policy now visibly consumes canonical §11 inputs; daemon-owned Pi RPC driver routes exist for start/prompt/abort/stop; Pi exposes `/focus-work` and richer loop status visibility; and the daemon checks worktree cleanliness before requesting each new continuous turn.
- Added `tests/work_loop_policy_consumption_test.sh`, `tests/pi_rpc_driver_contract_test.sh`, `tests/focus_work_command_surface_test.sh`, and `tests/worktree_discipline_guardrail_test.sh`, all wired into `scripts/ci/run-spec-gates.sh` and passing against the live daemon on `:8787`.
- This tranche closes the previously identified direct code-vs-spec79 mismatches, leaving the spec79 tree legitimately closed.

## Rule of record
When code/spec conflict, spec wins.
