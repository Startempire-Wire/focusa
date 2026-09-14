# Spec 138B — Temporally Grounded Adversarial Forecasting, Evidence-Responsiveness, and Calibration Assurance Addendum

**Status:** NORMATIVE ADDENDUM — MANDATORY FOR CONSEQUENTIAL PREDICTION ASSURANCE — IMPLEMENTATION NOT IMPLIED  
**Parent:** Spec 138 + Spec 138A  
**Owner:** Focusa Core / Prediction Authority / Epistemic Assurance  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Depends on:** Spec 137 + 137A + 137B, Spec 140 + 140A, Spec 144, Spec 149 champion/challenger infrastructure where active, Spec 152, Spec 152A, Spec 153/153A when physical applicability is active  
**Closure relationship:** A consequential prediction, forecast-derived decision, or prediction-derived learning claim cannot satisfy strict assurance without this addendum.

---

## 0. Constitutional directive

```text
FORECAST FIRST.
SEAL THE INFORMATION SET.
CHALLENGE INDEPENDENTLY.
UPDATE ONLY FOR EVIDENCE.
RESOLVE OUTCOMES THROUGH SEPARATE AUTHORITY.
LEARN ONLY WHEN THE CONTROL IS BEATEN.
```

A second LLM agreeing with the first is not assurance. A persuasive critique is not truth. A favorable outcome is not proof that the reasoning was correct. A correct final probability is not useful when it arrived too late to affect action.

Focusa MUST preserve the complete prediction trajectory: what was knowable, what was predicted, when it was committed, how evidence changed belief, what action threshold applied, how the outcome resolved, how the forecast scored against matched controls, and whether any resulting learning is promotion-eligible.

---

## 1. Purpose

Spec 138 defines prediction, outcome, scoring, calibration, metacognitive learning, transfer, and epistemic governance. This addendum adds the assurance protocol required to make those primitives resistant to:

- anchoring;
- shared blind spots;
- correlated model errors;
- dependent evidence;
- temporal leakage;
- persuasive but inaccurate critique;
- stylistic sensitivity;
- overreaction and underreaction;
- hindsight reconstruction;
- horizon mismatch;
- outcome-correct but mechanism-wrong learning;
- unearned promotion from favorable anecdotes.

---

## 2. Ownership

### 2.1 This addendum owns

- blind independent counterforecast protocol;
- adversarial reveal and exchange;
- disagreement localization;
- evidence-response evaluation;
- semantic/style invariance testing;
- matched temporal control comparisons;
- prediction council roles;
- model disagreement preservation;
- uncertainty decomposition requirements;
- outcome/mechanism/intervention/utility separation;
- high-confidence miss audit;
- prediction-specific promotion proof packet;
- prediction assurance conformance levels.

### 2.2 This addendum does not own

- physical time or clock synchronization;
- generic mathematical or statistical primitives;
- generic verifier routing;
- generic model champion/challenger infrastructure;
- outcome authority;
- Runtime Constitution compilation;
- domain-specific market, finance, engineering, legal, research, or scientific models.

---

## 3. Prediction council

A prediction assurance cycle uses distinct roles. Roles may be served by separate processes, models, deterministic validators, or authorized humans according to assurance tier.

```text
Forecaster
Counterforecaster
Evidence Auditor
Quantitative/Statistical/Physical Model Auditor
Calibrator
Decision Analyst
Resolver
Learning Promotion Verifier
```

The Resolver MUST remain separate from the forecasting roles. A forecaster cannot self-resolve or self-score its own prediction.

Separate prompts or session IDs alone do not prove independence.

---

## 4. Required prediction contract

```yaml
schema: focusa.adversarial_prediction_contract.v1
prediction_id:
question_ref:
outcome_space_ref:
subject_ref:
scope_ref:

forecast_target:
  target_window_ref:
  resolution_eligible_after_ref:
  resolution_deadline_ref:
  censoring_policy_ref:

information:
  information_set_ref:
  information_cutoff_stamp_ref:
  temporal_snapshot_ref:
  evidence_refs: []
  evidence_dependency_graph_ref:
  source_independence_profile_ref:

forecast:
  distribution_ref:
  probability_or_value_ref:
  confidence_dimensions_ref:
  uncertainty_decomposition_ref:
  abstention_policy_ref:
  base_rate_ref:
  reference_class_ref:
  alternative_hypothesis_refs: []
  falsifier_refs: []
  unknown_refs: []

quantitative_binding:
  mathematical_problem_ref:
  statistical_protocol_ref:
  physical_model_ref:
  assumption_set_ref:
  constraint_set_ref:
  computation_run_ref:
  validity_envelope_ref:
  sensitivity_result_ref:

runtime_binding:
  runtime_constitution_ref:
  constitution_hash:
  instruction_graph_hash:
  canonical_amendment_revision:
  active_target_profile_ref:
  temporal_adaptation_envelope_refs: []
  model_identity_ref:
  system_prompt_hash:
  tool_registry_digest:
  environment_snapshot_ref:

resolution:
  resolver_policy_ref:
  resolver_authority_ref:
  scoring_policy_ref:
  utility_policy_ref:

decision:
  action_threshold_refs: []
  expected_utility_ref:
  expected_regret_ref:
  action_deadline_ref:

assurance:
  adversarial_profile_ref:
  required_independence_tier:
  required_verification_obligation_refs: []
  control_strategy_refs: []

committed_at_stamp_ref:
contract_hash:
receipt_ref:
```

---

## 5. Temporal lifecycle

Every prediction has multiple clocks. At minimum, preserve:

```text
question_created
resolution_policy_frozen
scoring_policy_frozen
information_snapshot_sealed
information_cutoff
primary_request_dispatched
primary_response_completed
primary_commitment_sealed
counterforecast_request_dispatched
counterforecast_response_completed
counterforecast_commitment_sealed
mutual_reveal
adversarial_exchange_opened
adversarial_exchange_closed
final_commitment_sealed
evidence_event_occurred
evidence_first_available
evidence_received
evidence_ingested
evidence_authorized_known
probability_update_committed
decision_threshold_crossed
action_authorized
action_dispatched
outcome_occurred
outcome_observed
outcome_adjudicated
score_computed
control_comparison_computed
learning_candidate_created
learning_promoted
learning_effective
learning_expired_or_superseded
```

Each high-resolution stamp is governed by Spec 137B. Ordinary profiles may use coarser stamps only when their precision profile permits it.

---

## 6. Blind counterforecast protocol

### 6.1 Independent formation

The primary and counterforecaster MUST receive:

- the same immutable information-set snapshot;
- the same information cutoff;
- the same target and resolution criteria;
- the same admissible evidence set;
- the same quantitative and statistical contracts;
- equivalent opportunity to use approved tools;
- no visibility into the other forecast before sealing.

### 6.2 Commitment before reveal

```text
primary commitment sealed
counterforecast commitment sealed
→ mutual reveal permitted
```

A forecast edited after reveal is a new revision, not the original forecast.

### 6.3 Independence profile

```yaml
schema: focusa.forecast_independence_profile.v1
primary_actor_ref:
challenger_actor_ref:
primary_session_ref:
challenger_session_ref:
primary_model_ref:
challenger_model_ref:
same_provider:
same_model_family:
same_prompt_compiler:
same_tool_path:
same_retrieval_path:
same_training_or_upstream_dependency_known:
shared_transcript:
shared_hidden_reasoning:
independent_evidence_acquisition:
independent_model_formulation:
information_snapshot_equal:
forecast_sealed_before_reveal:
independence_tier:
degraded_reasons: []
receipt_ref:
```

Cross-family or cross-provider models are preferred for consequential work when policy requires them, but provider difference alone does not prove independence.

---

## 7. Disagreement localization

The system MUST preserve and classify disagreement rather than immediately averaging probabilities.

Disagreement dimensions:

```text
target interpretation
outcome-space interpretation
base rate
reference class
evidence authority
evidence independence
evidence freshness
assumption
mathematical formulation
statistical protocol
physical feasibility
causal mechanism
forecast horizon
uncertainty decomposition
utility or action threshold
model competence
```

A 0.90 forecast and a 0.20 counterforecast cannot be reduced to 0.55 without preserving why they differ and whether aggregation is authorized.

---

## 8. Adversarial exchange

After reveal, each role may challenge:

- assumptions;
- evidence admissibility;
- source dependence;
- quantitative model;
- statistical inference;
- physical mechanism;
- causal interpretation;
- scenario completeness;
- uncertainty;
- action utility;
- resolution criteria.

New evidence is admitted only through a new shared temporal snapshot. It MUST be presented to both forecasting sides simultaneously or excluded until the next update cycle.

The exchange closes at an explicit time. The final forecast is sealed as a new immutable commitment linked to both initial forecasts and all admitted evidence.

---

## 9. Evidence dependency graph

Multiple references do not imply independent support. Every consequential information set MUST identify shared upstream dependencies.

```yaml
schema: focusa.evidence_dependency_graph.v1
nodes:
  - evidence_ref:
    source_ref:
    upstream_source_refs: []
    publication_ref:
    event_stamp_ref:
    first_available_stamp_ref:
    known_stamp_ref:
edges:
  - from_ref:
    to_ref:
    dependency_kind:
independent_support_count:
minimum_required_independent_support:
shared_dependency_clusters: []
status:
receipt_ref:
```

Three articles repeating one press release count as one upstream support cluster unless additional independent evidence exists.

---

## 10. Evidence-response evaluation

Focusa MUST evaluate the update process, not only the final forecast.

### 10.1 Update record

```yaml
schema: focusa.forecast_update.v1
update_id:
prior_commitment_ref:
new_commitment_ref:
new_evidence_refs: []
prior_probability_or_value_ref:
new_probability_or_value_ref:
expected_direction:
expected_magnitude_interval_ref:
actual_direction:
actual_magnitude_ref:
response_latency_ns:
information_cutoff_stamp_ref:
committed_at_stamp_ref:
reason_summary:
receipt_ref:
```

### 10.2 Update failure classes

```text
underreaction
overreaction
sign_error
spurious_update
update_omission
unjustified_certainty
confidence_collapse
source_double_counting
stale_evidence_update
future_evidence_leakage
unsupported_no_change
```

---

## 11. Adversarial perturbation suite

The following tests MUST be available according to profile:

### Semantic sensitivity

Materially change evidence meaning while preserving form. The forecast should respond appropriately.

### Stylistic invariance

Change wording, order, formatting, tone, or presentation without changing meaning. The forecast should remain materially stable.

### Evidence removal

Remove a high-weight evidence item and measure expected probability movement.

### Counterfactual evidence

Supply an authorized hypothetical evidence change and test directional coherence.

### Contradiction injection

Add a credible contradiction and verify uncertainty or probability changes rather than being ignored.

### Source-authority inversion

Swap authoritative and low-authority source labels without changing claims. The system should respond to true authority policy, not presentation prestige.

### Temporal leakage test

Introduce evidence whose known time is after the information cutoff. The forecast MUST reject it.

### Dependency duplication test

Duplicate the same upstream claim through several references. Confidence MUST not rise as though support were independent.

### Quantitative perturbation

Vary high-sensitivity parameters within their uncertainty intervals and inspect decision stability.

### Physical feasibility perturbation

Change physical constraints to cross a feasibility boundary and verify the prediction responds.

---

## 12. Uncertainty decomposition

A single uncertainty score is insufficient. Prediction records should distinguish:

```text
aleatoric
epistemic
data
measurement
source
model
parameter
structural
regime
temporal
resolution
execution
physical-model
numerical
missingness
unknown_other
```

Different uncertainty classes imply different actions:

- gather more evidence;
- improve measurement;
- invoke a different model;
- add controls;
- clarify resolution;
- hedge or abstain;
- change action threshold;
- reject fake precision.

---

## 13. Prediction graph

Complex missions SHOULD decompose forecasts into a directed graph.

```yaml
schema: focusa.prediction_graph.v1
graph_id:
nodes:
  - prediction_ref:
    target_window_ref:
    role: prerequisite | mechanism | intervention | execution | outcome | utility
edges:
  - from_prediction_ref:
    to_prediction_ref:
    relationship:
    temporal_constraint_ref:
    conditional_probability_ref:
root_outcome_prediction_ref:
receipt_ref:
```

Example:

```text
dependency compatibility
→ tests pass
→ canary remains stable
→ deployment succeeds
→ users experience intended outcome
```

This localizes failure and prevents a correct final outcome from promoting an incorrect mechanism.

---

## 14. Four distinct forecast classes

The system MUST distinguish:

1. **Outcome forecast** — what will happen.
2. **Mechanism forecast** — why it will happen.
3. **Intervention-effect forecast** — what changes because of an action.
4. **Decision-utility forecast** — whether taking the action is beneficial under the utility policy.

Correct outcome with incorrect mechanism does not validate the mechanism. Beneficial outcome does not prove the intervention caused it. Accurate forecast does not prove the decision was optimal.

---

## 15. Abstention and intervals

Profiles MUST support:

```text
abstain
insufficient_evidence
not_resolvable_yet
probability_interval
value_interval
time_to_event_distribution
unknown_model_validity
operator_judgment_required
experimental_only
```

The system should reward justified refusal to invent precision.

---

## 16. Ensemble and aggregation

Aggregation MAY be used when:

- independence is classified;
- models are eligible;
- weights are frozen or learned only from resolved prior forecasts;
- weights are cohort-specific;
- disagreement and dispersion remain visible;
- aggregation policy is versioned;
- no valid critical veto is erased.

Performance weighting MUST be domain-, horizon-, model-, strategy-, regime-, and source-versus-transfer-aware.

---

## 17. Time-aware scoring

Every score MUST bind:

```text
prediction commitment time
information cutoff
target window
resolution window
outcome event time
outcome observation time
adjudication time
score computation time
forecast horizon
decision lead time
```

Comparisons must match or normalize:

- forecast horizon;
- information cutoff;
- evidence availability;
- update opportunity;
- target definition;
- resolution policy;
- scoring policy.

A forecast made five minutes before an event cannot be used as the control for one made thirty days before it without explicit horizon adjustment.

---

## 18. Forecast-trajectory metrics

Required metrics may include:

```text
Brier score at each commitment
integrated or cumulative Brier score
log score
calibration error
resolution and sharpness
update-direction accuracy
update-magnitude error
evidence-response latency
time materially miscalibrated
decision lead time
expected utility
realized utility
realized regret
baseline skill
control delta
high-confidence miss rate
abstention quality
```

A forecast that becomes accurate after the action deadline is statistically informative but operationally late.

---

## 19. Matched control and improvement

A prediction-derived strategy or learning cannot claim improvement from raw favorable outcomes.

Required control comparison:

```yaml
schema: focusa.prediction_matched_control_evaluation.v1
evaluation_id:
challenger_strategy_ref:
control_strategy_ref:
matched_information_cutoff:
matched_forecast_horizon:
matched_evidence_availability:
matched_update_opportunity:
matched_resolution_policy:
matched_scoring_policy:
sample_count:
censoring_policy_ref:
cohort_ref:
challenger_score_ref:
control_score_ref:
delta_score:
challenger_utility_ref:
control_utility_ref:
delta_utility:
uncertainty_ref:
statistical_protocol_ref:
evidence_refs: []
receipt_ref:
```

Stronger promotion claims require evidence such as:

```text
ΔBrier < 0
or
ΔUtility > 0
```

with valid uncertainty, sample, control, and applicability analysis.

---

## 20. High-confidence miss protocol

A false forecast at or above the configured high-confidence threshold, or a true outcome assigned below the symmetric low-probability threshold, MUST trigger:

1. automatic audit;
2. information-set integrity check;
3. temporal leakage check;
4. source-dependency check;
5. adversarial replay;
6. quantitative/statistical/physical model audit;
7. resolution-authority audit;
8. dependent learning quarantine;
9. affected strategy and self-model review;
10. rollback or narrowing of promoted learning where required.

Threshold values are policy. The protocol is mandatory when the threshold is crossed.

---

## 21. Learning promotion proof packet

```yaml
schema: focusa.prediction_learning_promotion_packet.v1
candidate_ref:
preregistered_prediction_refs: []
frozen_information_set_refs: []
temporal_integrity_receipt_refs: []
resolution_authority_refs: []
scoring_policy_refs: []
score_refs: []
matched_control_evaluation_refs: []
evidence_response_evaluation_refs: []
style_invariance_result_refs: []
adversarial_review_refs: []
quantitative_verification_refs: []
statistical_validity_refs: []
physical_validity_refs: []
mechanism_validity_refs: []
applicability_ref:
exclusion_refs: []
replication_refs: []
sample_support_ref:
conflict_refs: []
rollback_condition_refs: []
expiry_ref:
review_ref:
independent_promotion_verifier_ref:
evidence_refs: []
receipt_ref:
```

Missing proof leaves the candidate `experimental_only` or rejected.

---

## 22. Assurance levels

```text
138B-L1 Recorded
  Prediction is immutable, temporally grounded, resolvable, and scoreable.

138B-L2 Independently challenged
  Blind counterforecast and disagreement localization are complete.

138B-L3 Adversarially validated
  Perturbation suite, quantitative/statistical verification, and matched controls are complete.

138B-L4 Promotion eligible
  Outcome, score, control improvement, applicability, replication, and rollback proof permit learning review.
```

Suggested policy:

- ordinary low-risk predictions: L1;
- consequential decisions: L2;
- high-risk or high-confidence predictions: L3;
- canonical transferable learning: L4.

---

## 23. Spec 140/140A interlock

Every forecast commitment MUST bind the exact:

```text
Runtime Constitution revision and hash
instruction graph revision and hash
canonical amendment revision
selected target profile
temporal adaptation envelope
model provider/family/version/configuration
system prompt hash
tool registry digest
tool schema hashes
environment snapshot
permission and authority profiles
```

A material runtime or canonical instruction change invalidates affected forecasts, plans, controls, and comparisons until impact is assessed and a new commitment is sealed.

---

## 24. Spec 144 interlock

Spec 144 MUST be able to compile obligations for:

```text
epistemic.information_set_integrity
epistemic.source_authority
epistemic.source_independence
epistemic.temporal_leakage
epistemic.forecast_shape
epistemic.uncertainty
epistemic.scenario_coherence
epistemic.outcome_resolution
epistemic.scoring
epistemic.calibration
epistemic.decision_value
epistemic.causal_claim
epistemic.transfer
epistemic.learning_promotion
quantitative.model_validity
statistics.protocol_validity
physics.model_and_measurement_validity
```

A second LLM cannot substitute for the deterministic and specialist portfolio required by these obligations.

---

## 25. Tool and operation families

```text
prediction.question.create
prediction.information_set.freeze
prediction.commit.primary
prediction.commit.counterforecast
prediction.reveal
prediction.challenge.open
prediction.challenge.evidence.admit
prediction.challenge.close
prediction.update.commit
prediction.disagreement.localize
prediction.perturbation.run
prediction.evidence_response.evaluate
prediction.style_invariance.evaluate
prediction.control.match
prediction.control.evaluate
prediction.resolve
prediction.score
prediction.trajectory.evaluate
prediction.high_confidence_miss.audit
prediction.learning.packet.build
prediction.learning.promotion.verify
prediction.replay
prediction.explain
```

Top-level Pi tools SHOULD remain bounded:

```text
focusa_predict_record
focusa_predict_challenge
focusa_predict_update
focusa_predict_resolve
focusa_predict_evaluate
focusa_prediction_authority
```

Strict authority operations replace generic caller-authored event JSON for consequential profiles.

---

## 26. Legacy migration

Legacy prediction routes using prose outcomes, one confidence value, caller-supplied scores, substring scoring, or mutable evaluation records MUST be classified as legacy advisory behavior.

They MUST NOT:

- satisfy strict outcome resolution;
- satisfy canonical proper scoring;
- operate in high-consequence profiles;
- promote canonical learning automatically;
- bypass temporal snapshot or Runtime Constitution binding.

Migration uses expand-contract:

1. add structured authority operations;
2. migrate Pi/CLI/MCP clients;
3. preserve legacy read compatibility;
4. block strict profiles from legacy writes;
5. remove or quarantine legacy canonical claims only after all consumers migrate.

---

## 27. Required tests

1. primary and challenger receive equal snapshot hashes;
2. neither sees the other before sealing;
3. a shared-transcript challenger is classified as degraded independence;
4. temporally late evidence is rejected;
5. duplicated dependent sources do not increase independent support;
6. semantic evidence changes move probability appropriately;
7. stylistic transformations do not materially move probability;
8. contradictory evidence increases uncertainty or changes probability;
9. source-authority inversion follows policy rather than rhetoric;
10. update sign and magnitude are evaluated;
11. underreaction and overreaction are detected;
12. forecast horizon mismatch blocks naive comparison;
13. matched controls use equal information opportunities;
14. outcome-correct but mechanism-wrong learning is blocked;
15. invalid physical model blocks prediction assurance where applicable;
16. invalid statistical protocol blocks scoring claims;
17. high-confidence miss triggers audit and learning quarantine;
18. abstention is preserved as a valid result;
19. unresolved or censored outcomes are not scored as ordinary failures;
20. material Runtime Constitution change invalidates affected forecasts;
21. immutable replay reconstructs every commitment and update;
22. L4 promotion is impossible without complete proof packet.

---

## 28. Acceptance criteria

Spec 138B is accepted only when:

1. predictions bind complete temporal and runtime provenance;
2. blind counterforecasting operates before reveal;
3. independence is measured rather than assumed;
4. disagreement is localized and preserved;
5. new evidence enters through shared immutable snapshots;
6. update behavior is evaluated separately from final score;
7. semantic sensitivity and stylistic invariance tests exist;
8. evidence dependency and source independence are enforced;
9. uncertainty is decomposed;
10. outcome, mechanism, intervention, and utility forecasts remain distinct;
11. scoring and control comparisons are horizon- and information-matched;
12. high-confidence misses trigger mandatory audits;
13. promotion requires quantified improvement and full proof;
14. Spec 140 runtime identity and Spec 144 verification are bound;
15. legacy bypasses cannot satisfy strict profiles;
16. API, CLI, Pi, MCP, generated-client, replay, Evidence, Receipt, and migration proof are complete.

---

## 29. Canonical summary

```text
Do not ask a second LLM whether a forecast looks good.

Seal independent forecasts against one temporal information set.
Preserve disagreement.
Test reactions to evidence and invariance to style.
Verify mathematical, statistical, physical, semantic, and temporal validity.
Resolve outcomes through separate authority.
Compare against matched controls.
Promote learning only from complete proof.
```
