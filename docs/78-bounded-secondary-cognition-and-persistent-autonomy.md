# Bounded Secondary Cognition and Persistent Autonomy

## Purpose

Define how Focusa should use secondary LLM cognition and persistent improvement loops **without** violating operator priority, ontology authority, bounded scope, or verification discipline.

This document integrates and extends:
- `docs/51-ontology-expression-and-proxy.md`
- `docs/52-pi-extension-contract.md`
- `docs/54a-operator-priority-and-subject-preservation.md`
- `docs/54b-context-injection-and-attention-routing.md`
- `docs/56-trace-checkpoints-recovery.md`
- `docs/57-golden-tasks-and-evals.md`
- `docs/61-domain-general-cognition-core.md`
- `docs/62-visual-ui-evidence-and-workflow.md`
- `docs/67-query-scope-and-relevance-control.md`
- `docs/68-current-ask-and-scope-integration.md`
- `docs/69-scope-failure-and-relevance-tracing.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`
- `docs/71-governing-priors-and-scalar-weights.md`
- `docs/72-agent-identity-role-and-self-model-ontology.md`
- `docs/73-intention-commitment-and-self-regulation.md`
- `docs/74-identity-and-reference-resolution.md`
- `docs/75-projection-and-view-semantics.md`
- `docs/76-retention-forgetting-and-decay-policy.md`
- `docs/77-ontology-governance-versioning-and-migration.md`
- `docs/G1-14-reflection-loop.md`

This document also adopts selected loop-design principles from Karpathy's `autoresearch` experiment:
- immutable evaluation harness
- cheap bounded experiments
- explicit keep/discard advancement
- exhaustive result logging
- persistent execution under hard loop guardrails

External reference:
- upstream conceptual reference: `karpathy/autoresearch`
  - `README.md:7`
  - `program.md:21-114`

Citation note:
- `autoresearch` references in this document point to the upstream project and are not assumed to be vendored into this repository
- local normative authority for Focusa still comes from the Focusa docs cited throughout this spec

### Implementation status note

This document is primarily **normative**.

It should not be read as a claim that current runtime behavior already satisfies all requirements here.
Recent audit evidence warns that several newer Focusa/Pi behavioral requirements are still **DOCS-ONLY** in the hot path, especially around operator priority, minimal-slice injection, and non-coercive adoption.

Authority:
- `docs/TRUST_RESTORATION_AUDIT_2026-04-12.md`
- `docs/51-ontology-expression-and-proxy.md`
- `docs/54a-operator-priority-and-subject-preservation.md`
- `docs/54b-context-injection-and-attention-routing.md`

Accordingly, statements in this spec should be interpreted as one of:
- current verified behavior
- current code-backed partial behavior
- normative target requirement
- migration guidance

Where current implementation is known to lag the requirement, that lag should be called out explicitly rather than smoothed over.

---

## Core Thesis

Focusa should operate a **closed-loop improvement system** that gets sharper over time through:
- bounded observation
- structured extraction
- verification
- reflection
- evaluation
- promotion/rejection
- checkpoint/recovery
- retention/decay

But this loop is **not sovereign**.

Secondary cognition and persistent autonomy remain subordinate to:
1. hard system and safety rules
2. the operator's newest explicit input
3. ontology-governed scope, role, permissions, and verification boundaries
4. reducer/governance paths for canonical truth changes

This follows from:
- operator-first priority (`docs/54a-operator-priority-and-subject-preservation.md`)
- Pi as thin harness adapter, not second authority (`docs/52-pi-extension-contract.md`)
- ontology slices instead of broad self-injection (`docs/51-ontology-expression-and-proxy.md`)
- current-ask/scope governance (`docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`)
- inspectable self-regulation (`docs/73-intention-commitment-and-self-regulation.md`)
- bounded reflection (`docs/G1-14-reflection-loop.md`)

---

## 1. Why This Spec Exists

Focusa already contains several model-backed secondary cognition paths in `focusa-core`, including:
- deep validation
- post-turn evaluation
- worker-based extraction
- thesis refinement
- anticipatory query generation
- CLT summarization

These paths are useful, but without a unifying contract they risk:
- semantic drift
- scope contamination
- over-trusting speculative outputs
- canonical truth mutation without sufficient verification
- runaway metacognition
- stale context dominance

The newer Focusa docs establish the necessary missing constraints:
- ontology slices must be bounded and operator-first (`docs/51-ontology-expression-and-proxy.md`)
- Pi must not become a parallel cognitive authority (`docs/52-pi-extension-contract.md`)
- current ask and scope must govern relevance (`docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`)
- failures must be named, traced, and made improvable (`docs/69-scope-failure-and-relevance-tracing.md`)
- shared interfaces must normalize verification and provenance (`docs/70-shared-interfaces-statuses-and-lifecycle.md`)
- canonical truth, projections, and active relevance must be distinguished (`docs/75-projection-and-view-semantics.md`, `docs/76-retention-forgetting-and-decay-policy.md`)

---

## 2. Closed-Loop Improvement Model

Focusa's improvement loop should be modeled as:

1. **Observe**
2. **Extract / Propose**
3. **Verify / Critique**
4. **Evaluate against fixed success criteria**
5. **Promote / retain-as-projection / reject / archive**
6. **Checkpoint and trace**
7. **Recover / resume / continue**
8. **Apply decay and retention policy**
9. **Repeat if continuation conditions still hold**

This loop is directly grounded in:
- domain-general cognition requirements: verification, blocker detection, recovery, bounded working sets (`docs/61-domain-general-cognition-core.md`)
- trace + resumability requirements (`docs/56-trace-checkpoints-recovery.md`)
- measurable eval discipline (`docs/57-golden-tasks-and-evals.md`)
- explicit evidence-chain workflow (`docs/62-visual-ui-evidence-and-workflow.md`)
- bounded reflection overlay (`docs/G1-14-reflection-loop.md`)

### 2.1 Not a generate-and-hope loop

Focusa must not treat improvement as vague recursive self-talk.

Relevant prohibitions and supporting principles:
- reflection is overlay workflow, not autonomous control mode (`docs/G1-14-reflection-loop.md:6-13`)
- visual/UI work must not be a vague generate-and-hope loop (`docs/62-visual-ui-evidence-and-workflow.md`)
- slices must remain bounded and relevant (`docs/54b-context-injection-and-attention-routing.md`)

---

## 3. Operator and Ontology Authority

### 3.1 Operator-first override

When the operator provides new input, it overrides loop momentum.

Focusa may guide.
It may not hijack.

Authoritative support:
- `docs/54a-operator-priority-and-subject-preservation.md`
- `docs/52-pi-extension-contract.md`
- `docs/54b-context-injection-and-attention-routing.md`

### 3.2 Governing priors outrank loop persistence

Persistent autonomy must be downstream of governing priors, not above them.

In particular:
- hard prohibitions outrank local optimization
- identity/role outrank convenience weights
- current ask outranks adjacent mission carryover unless relevance is proven

Authority:
- `docs/71-governing-priors-and-scalar-weights.md`

### 3.3 Ontology-governed scope is mandatory

Every active improvement cycle must remain consistent with:
- `CurrentAsk`
- `QueryScope`
- `RelevantContextSet`
- `ExcludedContextSet`
- role/capability/permission constraints
- verification state
- handoff boundaries

Authority:
- `docs/67-query-scope-and-relevance-control.md`
- `docs/68-current-ask-and-scope-integration.md`
- `docs/72-agent-identity-role-and-self-model-ontology.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`

---

## 4. Core Design Laws

1. **Secondary cognition is subordinate.** It may assist reasoning; it may not become a parallel sovereign authority.
2. **Canonical truth, projection, and active relevance are distinct.** (`docs/75-projection-and-view-semantics.md`, `docs/76-retention-forgetting-and-decay-policy.md`)
3. **Verification precedes promotion.** (`docs/61-domain-general-cognition-core.md`, `docs/70-shared-interfaces-statuses-and-lifecycle.md`)
4. **Every consequential output must be traceable.** (`docs/56-trace-checkpoints-recovery.md`)
5. **Scope purity is part of intelligence.** (`docs/67-query-scope-and-relevance-control.md`, `docs/69-scope-failure-and-relevance-tracing.md`)
6. **Self-regulation must reduce drift, not increase introspective noise.** (`docs/73-intention-commitment-and-self-regulation.md`)
7. **Retention discipline is part of improvement.** Better forgetting is part of getting smarter. (`docs/76-retention-forgetting-and-decay-policy.md`)
8. **Improvement claims require fixed evals.** (`docs/57-golden-tasks-and-evals.md`; `autoresearch/program.md:28-33`)
9. **Persistent autonomy is conditional, not absolute.** (`docs/G1-14-reflection-loop.md`; `docs/54a-operator-priority-and-subject-preservation.md`)
10. **Policy itself is part of the loop surface.** `autoresearch` explicitly treats `program.md` as the programmable research-org layer (`README.md:7`). Focusa should do the equivalent through specs/contracts/governance docs.
11. **Workers remain advisory at the contract level.** Older worker specs still define workers as advisory-only results flowing through reducer acceptance, even where current runtime uses LLM-backed worker execution. (`docs/G1-10-workers.md`)
12. **Prompt assembly remains deterministic critical-path infrastructure.** Secondary cognition may inform future state or projections, but it must not turn prompt assembly into an unconstrained generative step. (`docs/G1-detail-11-prompt-assembly.md`)
13. **RFM microcells are validators, not generators.** Reliability mode exists to verify and selectively regenerate under policy, not to create a second creative authority. (`docs/36-reliability-focus-mode.md`)

---

## 4.1 Foundational older-spec constraints still in force

The newer ontology/governance docs do not erase the still-binding architectural constraints established in older foundational specs.

These older constraints materially shape this document:
- workers are advisory and return results; reducer decides state effects (`docs/G1-10-workers.md`)
- `extract_ascc_delta` is defined as a structured delta proposal, not direct worker-owned state mutation (`docs/G1-10-workers.md`, `docs/G1-07-ascc.md`)
- prompt assembly is deterministic, bounded, and on the critical path (`docs/G1-detail-11-prompt-assembly.md`)
- RFM validator microcells never generate content and do not see full history (`docs/36-reliability-focus-mode.md`)
- reflection is an overlay workflow and recommendations remain advisory unless applied through existing action paths (`docs/G1-14-reflection-loop.md`)
- reducer/event discipline remains the authoritative state-transition mechanism (`docs/core-reducer.md`)

Implementation drift note:
- current runtime appears to have evolved beyond parts of the older worker MVP spec by executing worker jobs through LLM-backed code paths with longer timeouts and direct model API calls
- this older-spec/current-code mismatch must be treated as an explicit reconciliation item, not silently glossed over
- related bead: `focusa-jbp2`

Accordingly, any newer secondary-cognition design in this spec must be interpreted through those older binding constraints unless an explicit superseding spec says otherwise.

## 5. Secondary Cognition Roles

Secondary cognition is allowed only in clearly typed roles.

### 5.1 Verification

Purpose:
- check consistency
- check grounding
- inspect scope fidelity
- inspect constraint compliance
- inspect implementation quality

Ontology/interface posture:
- verification outputs should be modeled as **Verifiable** objects with at least:
  - `verification_status`
  - `verification_refs`
  - `verification_kind`
- where evidence artifacts exist, they should also be **ArtifactBacked**

Examples:
- RFM validation
- post-turn quality evaluation
- visual/UI critique
- scope verification

#### 5.1a Adversarial closure-veracity verification
For work-item closure authority, secondary verification should support an adversarial verifier mode that attempts to falsify closure claims.

Contract expectations:
- closure verdict is `Verifiable` and evidence-linked
- verifier emits explicit objections and sufficiency status
- verifier output is advisory to reducer authority, but may veto close transitions via verification-blocked outcome
- verifier-unavailable states are fail-closed for closure authority unless operator/governance override is explicit

This aligns with:
- `docs/61-domain-general-cognition-core.md`
- `docs/60-visual-ui-verification-and-critique.md`
- `docs/G1-14-reflection-loop.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`

### 5.2 Proposal / Extraction

Purpose:
- extract candidate decisions/constraints/failures/blockers
- derive structured candidate deltas from evidence
- produce resolution candidates, not final truth

Authority posture:
- proposals are not canonical truth
- proposal outputs should be framed as candidate objects analogous to:
  - `OntologyProposal`
  - `DecisionCandidate`
  - `EvidenceLinkedObservation`
  - `VerificationRequest`
- extraction may support promotion, but extraction alone is not promotion

This aligns with:
- `docs/52-pi-extension-contract.md`
- `docs/61-domain-general-cognition-core.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`
- `docs/74-identity-and-reference-resolution.md`

### 5.3 Projection

Purpose:
- create bounded role/task-specific views
- compress for budget without mutating canonical truth
- produce low-budget, executor, reviewer, or diagnostic perspectives

Authority posture:
- a projection is a derived view over canonical state, not a replacement for canonical state
- projection outputs should be explicitly shaped by a view profile, projection rules, and projection boundaries
- compression remains a projection operation, not a truth mutation

This aligns with:
- `docs/75-projection-and-view-semantics.md`

### 5.4 Reflection

Purpose:
- periodically review work quality and focus trajectory
- surface observations, risks, and recommended actions

This aligns with:
- `docs/G1-14-reflection-loop.md`

### 5.5 Prediction / Anticipation

Purpose:
- suggest next likely retrieval targets
- anticipate blockers or needed context
- propose likely future relevance, without forcing context import

Authority posture:
- predictive outputs are lower-authority than verification outputs
- predictive outputs should be treated as scoped, decaying hints unless later re-validated
- prediction must not silently cross scope boundaries or outrank the current ask

This aligns with:
- future-facing use of working sets/slices and loop continuity
- but must remain subordinate to current-ask and scope controls (`docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`)
- and retention/decay discipline (`docs/76-retention-forgetting-and-decay-policy.md`)

---

## 6. Forbidden Authority

Secondary cognition must not:
- auto-switch active focus frame outside allowed policy path (`docs/G1-14-reflection-loop.md`)
- auto-write memory/canonical ontology without reducer/governance path (`docs/G1-14-reflection-loop.md`, `docs/52-pi-extension-contract.md`)
- bypass operator priority (`docs/54a-operator-priority-and-subject-preservation.md`)
- redefine the eval harness or its success criteria (`docs/57-golden-tasks-and-evals.md`; `autoresearch/program.md:28-33`)
- inject broad unbounded self-state into every turn (`docs/51-ontology-expression-and-proxy.md`, `docs/54b-context-injection-and-attention-routing.md`)
- continue stale mission context when current ask no longer supports it (`docs/52-pi-extension-contract.md`, `docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`)
- erase failed attempts from historical trace (`docs/56-trace-checkpoints-recovery.md`, `docs/76-retention-forgetting-and-decay-policy.md`)

---

## 7. Persistent Autonomy Under Operator and Ontology Authority

### 7.1 Definition

Persistent autonomy means Focusa may continue bounded improvement activity across successive iterations without awaiting fresh confirmation, **but only as a subordinate execution mode**.

This section formalizes the safe adaptation of `autoresearch`'s persistent experiment loop (`program.md:90-114`) into Focusa.

### 7.2 Continuation conditions

A new autonomous cycle may begin only if all are true:
- no newer operator steering has superseded the objective
- the active `CurrentAsk` still governs the loop or validly authorizes continuing work
- the `QueryScope` still permits the current search/refinement path
- role/capability/permission boundaries still allow the work
- the loop has evidence delta or a justified next experiment
- no stop condition has fired
- required eval/verification surfaces remain available

### 7.3 Invalidation triggers

Persistent autonomy must halt, pause, or rebuild when any occur:
- new operator instruction
- scope contamination or wrong-question risk detected (`docs/69-scope-failure-and-relevance-tracing.md`)
- no new evidence delta (`docs/G1-14-reflection-loop.md`)
- repeated recommendation set (`docs/G1-14-reflection-loop.md`)
- degraded confidence beyond threshold (`docs/G1-14-reflection-loop.md`)
- permission or handoff boundary reached (`docs/72-agent-identity-role-and-self-model-ontology.md`)
- eval harness unavailable or compromised

### 7.4 Allowed outputs during persistent autonomy

Each cycle may emit only:
- proposal
- verification result
- evidence-linked observation
- failure/blocker signal
- projection update
- retention/decay recommendation
- checkpoint / trace event
- eval record

Each cycle may not emit:
- direct canonical truth write without policy path
- topic takeover
- stale mission continuation without relevance proof
- new unscoped context injection block

---

## 8. Immutable Evaluation Harness

A core imported principle from `autoresearch` is that the agent may not rewrite the benchmark while trying to improve against it.

In `autoresearch`:
- `prepare.py` is read-only
- `evaluate_bpb` is ground truth
- the agent may modify `train.py`, not the benchmark
- source: `program.md:28-33`

Focusa should adopt the same law:

### 8.1 Eval surfaces must be fixed by policy

For any improvement loop, the following must be externally defined and non-self-modifiable within the loop:
- golden tasks
- acceptance metrics
- scoring formulas
- replay artifacts
- stop conditions
- promotion thresholds

Authority:
- `docs/57-golden-tasks-and-evals.md`
- `docs/G1-14-reflection-loop.md`

### 8.2 Reflection and proposals cannot redefine success

Secondary cognition may recommend changes to eval policy **only via governance path**, never inline as a loop-local self-exemption.

Authority:
- `docs/77-ontology-governance-versioning-and-migration.md`

---

## 9. Promotion, Rejection, and Archival

Another key imported principle from `autoresearch` is explicit keep/discard advancement:
- if improved, keep
- if equal/worse, revert
- if crash, log and move on
- source: `program.md:103-110`

Focusa should adapt this into ontology-native outcome classes.

### 9.1 Outcome classes

Every secondary-cognition output must end in one of:
- **promoted** — verified and allowed to shape canonical state
- **retained_as_projection** — useful bounded view, not canonical truth
- **rejected** — insufficiently supported or harmful if used
- **archived_failed_attempt** — historically preserved, behaviorally inactive
- **deferred_for_review** — ambiguous, requires human or stronger policy path

Current continuous-outcome mapping emits `promoted`, `rejected`, `archived_failed_attempt`, and `deferred_for_review`; `retained_as_projection` remains a valid target class for secondary outputs that are useful but intentionally non-canonical.

### 9.2 Do not erase failure history

Unlike a pure branch reset, Focusa should preserve failed attempts in historical trace while reducing their active influence.

Authority:
- `docs/56-trace-checkpoints-recovery.md`
- `docs/76-retention-forgetting-and-decay-policy.md`
- `docs/74-identity-and-reference-resolution.md`

---

## 10. Proposal Advancement Ledger

Focusa should maintain a durable result log for secondary cognition analogous to `autoresearch/results.tsv` (`program.md:64-88`).

### 10.1 Purpose

This ledger makes the improvement loop auditable and comparable over time.
Current runtime wiring records secondary-loop outcome entries in durable telemetry (`secondary_loop_ledger`), tracks active-window archival roll-off (`secondary_loop_archived_events`), and exposes recent rows through work-loop status eval artifacts.

### 10.2 Minimum fields

Each ledger row/event should capture:
- `proposal_id`
- `source_function`
- `actor_instance_id`
- `role_profile_id`
- `current_ask_id`
- `query_scope_id`
- `input_window_ref`
- `evidence_refs[]`
- `proposed_delta`
- `verification_status`
- `promotion_status`
- `confidence`
- `impact_metrics`
- `failure_class` if harmful or rejected
- `description`
- `trace_id`
- `correlation_id`
- `created_at`

### 10.3 Benefits

This supports:
- comparison of near-miss ideas
- historical audit
- repeated-mistake reduction
- smarter retention/decay
- governance review

Authority:
- `docs/56-trace-checkpoints-recovery.md`
- `docs/57-golden-tasks-and-evals.md`
- `docs/69-scope-failure-and-relevance-tracing.md`
- `autoresearch/program.md:64-88`

---

## 11. Trace Requirements

Every meaningful secondary cognition cycle must emit traceable events.

Minimum trace should include:
- loop objective
- operator subject
- active subject after routing
- current ask
- scope used
- excluded context
- working set used
- constraints and decisions consulted
- evidence refs
- proposal/projection/verification kind
- continuation decision
- promotion/rejection outcome
- stop_reason if applicable

For closure-veracity workflows, this includes verifier-specific traces for focus-slice size/relevance, steering detection, prior-mission reuse, and close-committed vs close-blocked transitions.
Continuous-loop continuation boundaries (request-next, select-next, and auto-advance) should emit `scope_verified` / `scope_failure_recorded` path markers when operator/governance boundaries block continuation.
API-side auto-dispatch/autoselection helpers should short-circuit prompt dispatch while these boundaries are active.

Authority:
- `docs/56-trace-checkpoints-recovery.md:12-35`
- `docs/69-scope-failure-and-relevance-tracing.md:47-58`
- `docs/G1-14-reflection-loop.md:63-74`

### 11.1 Current implementation-gap note

Some required trace dimensions appear to be specified more clearly in docs than in current runtime observability surfaces.
Current closure-veracity and continuous-outcome quality traces emit:
- `active_subject_after_routing`
- `steering_detected`
- `prior_mission_reused`
- `focus_slice_size`
- `focus_slice_relevance_score`
- `subject_hijack_prevented`
- `subject_hijack_occurred`

Remaining gap: prove these dimensions are emitted consistently across all secondary-cognition loop kinds, not only closure paths.

Related reconciliation bead:
- `focusa-fs2m`

### 11.2 Acceptance hooks

This section should not be considered satisfied unless reviewers can point to:
- emitted runtime events or persisted trace artifacts containing the required dimensions
- at least one replayable example where a secondary loop can be followed from trigger to outcome
- at least one negative example showing rejection, suppression, or decay rather than only successful promotion

---

## 12. Evidence and Verification Requirements

### 12.1 Shared verifiable interface

Any output that can affect future behavior should implement the equivalent of a `Verifiable` contract with:
- `verification_status`
- `verification_refs`
- `verification_kind`

Authority:
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`

### 12.2 Evidence-first rule

Secondary cognition must prefer evidence-linked observations over uncited semantic conclusions.

Where practical, outputs should be both:
- **Verifiable** (`verification_status`, `verification_refs`, `verification_kind`)
- **ArtifactBacked** (`artifact_refs`, `artifact_kind_summary`)

This is especially important for:
- decision extraction
- constraint extraction
- thesis refinement
- visual critique
- scope failure diagnosis
- closure authority decisions (emit closure certificate artifacts before durable close transitions)

Authority:
- `docs/62-visual-ui-evidence-and-workflow.md`
- `docs/60-visual-ui-verification-and-critique.md`
- `docs/69-scope-failure-and-relevance-tracing.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`

---

## 13. Retention and Decay Policy for Loop Outputs

Not every useful output should remain behaviorally dominant forever.

### 13.1 Core distinction

Focusa must distinguish:
- canonical truth
- active relevance
- archived historical trace

Authority:
- `docs/76-retention-forgetting-and-decay-policy.md`
- `docs/75-projection-and-view-semantics.md`

### 13.2 Default retention posture

Suggested defaults:
- verification results: active for current working set, then decay
- projection artifacts: bounded by view and scope, then archive or regenerate
- rejected proposals: archive with low active weight
- promoted canonical deltas: retain per domain retention policy
- predictive hints: aggressive decay unless re-validated

---

## 14. Mapping Current Secondary LLM Functions into This Spec

This section does not freeze implementation, but establishes intended roles.

### 14.1 RFM deep validation

Current role:
- verifier
- narrow reliability microcell for R1+ content checking

Current implementation notes:
- implemented in `crates/focusa-core/src/rfm/mod.rs` via `validate_llm()`
- prompt checks internal consistency, grounding, and constraint compliance
- returns booleans + issues/details, not freeform semantic state
- fallback is fail-closed by default when API key is absent, response is unparseable, or timeout occurs
- optional permissive mode exists via `FOCUSA_RFM_LLM_FAIL_OPEN=1` for controlled diagnostics

Fit:
- strong on role
- stronger trust semantics with strict-default fallback

Required constraints:
- fixed eval semantics
- explicit verification record
- verification outcome should remain distinct from canonical truth mutation
- permissive fallback usage should remain explicit and observable

### 14.2 Post-turn quality evaluation

Current role:
- verifier / critique
- async review of the just-completed assistant turn

Current implementation notes:
- implemented in `crates/focusa-core/src/runtime/daemon.rs`
- prompt scores whether the assistant answered the question, remained consistent, and violated active constraints
- parsed output is logged and written to semantic memory as `eval.last_turn`
- logs confidence telemetry and emits `verification_result` traces for `post_turn_eval` / `thesis_refinement`
- does **not** directly rewrite canonical Focus State fields

Fit:
- strong
- one of the cleaner current secondary-cognition paths because authority is mostly observational

Required constraints:
- trace association
- current-ask / scope association
- evidence refs where possible
- promotion boundary should remain observational unless separately verified
- suppress this loop when operator steering or governance-decision-pending boundaries are active

### 14.3 LLM-backed worker jobs

Status note:
- this section describes the **current runtime behavior** of workers
- it does **not** claim that current worker runtime is fully aligned with the older MVP worker spec
- that reconciliation remains open under bead `focusa-jbp2`

Current role:
- extractor / classifier / detector / suggestion generator

Current implementation notes:
- implemented in `crates/focusa-core/src/workers/executor.rs` and invoked from `runtime/daemon.rs`
- active job kinds include:
  - `ClassifyTurn`
  - `ExtractAsccDelta`
  - `DetectRepetition`
  - `ScanForErrors`
  - `SuggestMemory`
- worker jobs are actually enqueued at turn completion and run through `execute_job_llm()` with heuristic fallback
- downstream effects differ by subtype:
  - `ExtractAsccDelta` can produce `FocusStateDelta` application
  - `DetectRepetition` and `ScanForErrors` emit worker-origin signals
  - `SuggestMemory` creates procedural-rule candidates / semantic suggestions
  - `ClassifyTurn` is lower-authority classification metadata

Fit:
- mixed
- strongest fit: classification/error/repetition detection
- highest-risk fit: `ExtractAsccDelta`, because it is closest to live semantic authority

Required constraints:
- distinguish candidate/proposal outputs from promoted truth
- require verification or stronger reducer rules before durable promotion where impact is high
- preserve provenance
- separate low-authority signal generation from higher-authority semantic delta application

### 14.4 Thread thesis refinement

Current role:
- continuity projection / semantic anchor maintenance

Current implementation notes:
- implemented in `crates/focusa-core/src/runtime/daemon.rs`
- triggered every 3rd turn by event-count cadence
- prompt consumes current thesis, frame state, and latest turn
- parsed output updates thesis fields directly via `Action::UpdateThesis`
- therefore this path has more authority than observational/verifier-only paths

Fit:
- medium
- useful for continuity, but currently underspecified relative to newer ask/scope/projection docs

Required constraints:
- treat as projection or tightly bounded semantic update
- tie to current ask/scope
- trace changed fields and evidence basis
- support no-change as a first-class valid outcome
- suppress refinement when operator steering or governance-decision-pending boundaries are active

### 14.5 Anticipatory query generation

Current role:
- prediction / speculative retrieval guidance

Current implementation notes:
- implemented in `crates/focusa-core/src/runtime/daemon.rs`
- prompt predicts the user's next likely question/topic and returns 3 search queries
- resulting queries are used to prefetch Mem0 context
- retrieved items are stored as semantic memory under `anticipated.*`
- this does not directly mutate canonical state, but it can bias future context selection

Fit:
- medium-risk
- potentially useful, but highly vulnerable to scope contamination if not gated by current-ask relevance

Required constraints:
- speculative only
- blocked from polluting prompt assembly unless relevance gate passes
- aggressive decay
- explicit scope association and exclusion handling
- suppress prediction while operator steering or governance-decision-pending boundaries are active

### 14.6 CLT summarization

Current role:
- compression / continuity aid

Current implementation notes:
- implemented in `crates/focusa-core/src/clt/mod.rs`
- triggered when CLT interaction-node volume exceeds threshold and oldest uncovered interactions are summarized
- output is inserted as a summary node covering prior node ids
- this is a continuity/compression artifact, not a canonical decision source

Fit:
- strong if treated as projection/compression, not truth mutation

Required constraints:
- preserve traceability to covered interactions
- do not treat summary as canonical replacement for evidence
- keep lineage from summary back to covered interactions explicit

---

## 15. Evaluation Requirements

A secondary cognition loop is only successful if it measurably improves Focusa under fixed evals.

Minimum measured dimensions should include:
- mission retention
- working-set precision
- irrelevant-context reduction
- repeated-mistake rate
- decision-consult rate
- recovery success rate
- degraded-mode behavior quality
- scope contamination rate
- verification coverage
- latency/cost impact

Authority:
- `docs/57-golden-tasks-and-evals.md`
- `docs/69-scope-failure-and-relevance-tracing.md`

### 15.1 Acceptance hooks

A secondary-cognition implementation should not be called compliant with this spec unless it can demonstrate, under replay or controlled task runs:
- at least one task where bounded secondary cognition improves action quality over a no-secondary baseline
- at least one task where a tempting but irrelevant secondary suggestion is correctly suppressed
- at least one task where verification rejects a proposal that looked locally plausible
- at least one task where predictive or reflective output decays or is archived instead of remaining behaviorally dominant

Current coverage includes (a) controlled runtime tests driving `ObserveContinuousTurnOutcome` through useful vs low-quality outcomes, subject-hijack suppression, deferred/archived outcomes, and same-task comparative baseline-vs-bounded outcomes, and (b) replay-log comparative tests over persisted `ContinuousSecondaryLoopOutcomeRecorded` events; status-level `secondary_loop_eval_bundle` and `secondary_loop_acceptance_hooks` surfaces include `comparative_improvement_pairs`.
Controlled proof bundle: `tests/doc78_secondary_loop_comparative_eval.sh`; replay proof bundle: `tests/doc78_secondary_loop_replay_comparative_eval.sh`.

### 15.2 Minimum eval artifact set

Each eval run should preserve or emit enough data to audit:
- task id / scenario id
- model/runtime configuration
- secondary loop kind invoked
- traces or event handles for consulted scope, evidence, and outcome
- promotion/rejection/archival result
- latency and token-cost impact where applicable
- final task outcome

Current runtime status now emits `secondary_loop_eval_bundle` containing these dimensions from ledger + telemetry surfaces, and ledger impact metrics include per-outcome latency (`latency_ms_since_turn_request`) plus token totals.

### 15.3 Implementation status update (2026-04-18)

The previously noted instrumentation gap for doc78 closure surfaces is now satisfied for the implemented runtime path set.
`verification_result` traces, replay-log comparative summaries, per-task `secondary_loop_closure_replay_evidence`, `secondary_loop_objective_profile`, and fail-closed replay consumer projections are all live in status/consumer/dashboard surfaces and validated by executable harnesses.

Production closure evidence is archived in:
- `docs/evidence/DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17.md`
- `docs/evidence/DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md`
- `docs/evidence/DOC78_COMPLETION_CERTIFICATE_2026-04-18.md`

### 15.4 Bead/test surfaces

Concrete work items generated from this section should usually land as:
- instrumentation beads for missing trace fields
- replay/eval beads for golden-task comparisons
- reconciliation beads where current runtime behavior diverges from stated measurement surfaces

---

## 16. Governance and Migration

Because this spec changes authority boundaries for secondary cognition, any implementation change must be treated as ontology-governed and reviewable.

Required:
- explicit compatibility declaration
- migration plan for existing secondary outputs if schemas change
- conformance checks against shared interfaces after major change

Authority:
- `docs/77-ontology-governance-versioning-and-migration.md`

### 16.1 Reconciliation status

For doc78 closure scope, required reconciliations are now backed by live code paths and executable proof surfaces.

Remaining broader ontology/runtime reconciliation items outside doc78 closure scope may still exist, but they no longer block doc78 implementation conformance claims.

---

## 17. Success Condition

This document is satisfied when Focusa can run bounded, persistent, evidence-backed secondary cognition loops that:
- measurably improve continuity, correctness, relevance, and recovery over time
- remain fully subordinate to operator steering and ontology-governed scope
- produce traceable, reviewable, eval-backed outcomes
- promote only sufficiently verified changes into canonical state
- preserve failed attempts historically without allowing them to dominate active reasoning
- get sharper through governed iteration rather than unconstrained recursive self-talk

---

## Appendix A — Imported Principles from `autoresearch`

### Keep
- immutable eval harness (`program.md:28-33`)
- cheap bounded iterations (`program.md:23`, `108-114`)
- explicit keep/discard advancement (`program.md:103-110`)
- exhaustive experiment/result logging (`program.md:64-88`)
- policy/context as part of the programmable loop surface (`README.md:7`)

### Adapt
- persistent autonomy / “never stop” (`program.md:112-114`) → in Focusa this becomes continue-by-default **only while** operator, scope, verification, and stop conditions allow
- revert semantics (`program.md:104`) → in Focusa this becomes reject for promotion while preserving historical trace (`docs/56-trace-checkpoints-recovery.md`, `docs/76-retention-forgetting-and-decay-policy.md`)

### Reject
- single scalar metric as total truth (`program.md:31-33`) → Focusa requires multidimensional evals (`docs/57-golden-tasks-and-evals.md`)
