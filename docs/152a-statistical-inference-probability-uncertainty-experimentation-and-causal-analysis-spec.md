# Spec 152A — Statistical Inference, Probability, Uncertainty, Experimentation, and Causal Analysis

**Status:** NORMATIVE DRAFT — PRIMITIVE-OWNING — IMPLEMENTATION NOT IMPLIED  
**Owner:** Focusa Core / Statistical Cognition  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Parent substrate:** Spec 152  
**Depends on:** Spec 137 family, Spec 138 family, Spec 140 family, Spec 144  
**Primary consumers:** predictions, research, markets, software evaluation, operations, finance, physical experiments, metacognition, verifier calibration

---

## 0. Constitutional directive

```text
DEFINE THE POPULATION.
FREEZE THE PROTOCOL.
PRESERVE THE SAMPLE PATH.
MODEL DEPENDENCE AND MISSINGNESS.
SEPARATE ASSOCIATION, PREDICTION, AND CAUSATION.
REPORT UNCERTAINTY AND PRACTICAL CONSEQUENCE.
```

A p-value is not a conclusion. A posterior is not a fact. A high correlation is not a mechanism. A large sample cannot repair a biased sampling process. A model fit on future information cannot prove predictive skill.

---

## 1. Purpose

This specification provides one reusable statistical substrate for:

- probabilistic forecasts;
- calibration;
- experiment design;
- observational studies;
- time-series and event-stream analysis;
- model and verifier evaluation;
- decision analysis;
- uncertainty propagation;
- causal reasoning;
- scientific and operational learning.

It prevents each Vertical from inventing incompatible definitions of probability, samples, confidence, evidence, or statistical validity.

---

## 2. Ownership

### 2.1 This specification owns

- events, random variables, outcomes, and probability distributions;
- populations, sampling frames, samples, observations, and cohorts;
- estimands, estimators, estimates, likelihoods, priors, and posteriors;
- uncertainty intervals and prediction intervals;
- descriptive statistics and distribution summaries;
- hypotheses, tests, effect sizes, errors, and statistical power;
- experiment, treatment, control, randomization, blocking, and stopping protocols;
- multiple testing and selective reporting controls;
- missingness, censoring, truncation, and survivorship semantics;
- dependence, correlation, clustering, autocorrelation, and repeated measures;
- time-series, regime, and drift semantics;
- causal graphs, interventions, counterfactuals, identification, confounding, mediation, and moderation;
- sequential and online inference;
- statistical protocol freezing, validation, correction, and replay.

### 2.2 This specification does not own

- forecast commitment or learning promotion;
- domain-specific outcome definitions;
- market or business risk appetite;
- physical measurement authority;
- general mathematical quantities and solvers;
- semantic verification routing;
- authority to claim legal, medical, financial, or scientific conclusions without applicable domain review.

---

## 3. Core ontology

```text
ProbabilitySpace
Event
Outcome
RandomVariable
Distribution
JointDistribution
ConditionalDistribution
StochasticProcess
Population
TargetPopulation
SamplingFrame
Sample
Observation
Cohort
Panel
TimeSeries
Estimand
Estimator
Estimate
Likelihood
Prior
Posterior
Hypothesis
NullHypothesis
AlternativeHypothesis
EffectSize
TestStatistic
ConfidenceInterval
CredibleInterval
PredictionInterval
Experiment
Treatment
Control
Intervention
Randomization
Block
Stratum
StoppingRule
MissingnessMechanism
CensoringPolicy
Confounder
Mediator
Moderator
Instrument
CausalGraph
CausalEffect
CalibrationProfile
DriftProfile
```

---

## 4. Probability representation

Probabilities MUST be finite, bounded, semantically attached to an event and information set, and governed by a representation policy.

```yaml
schema: focusa.probability_value.v1
probability_id:
event_ref:
conditional_on_refs: []
information_set_ref:
representation: binary_float | decimal | interval | symbolic | distribution_derived
value:
lower_bound:
upper_bound:
source_method_ref:
temporal_cutoff_ref:
uncertainty_ref:
status:
```

A model confidence, evidence confidence, source reliability, resolution confidence, and event probability remain separate dimensions.

---

## 5. Distribution contract

```yaml
schema: focusa.distribution_definition.v1
distribution_id:
variable_refs: []
distribution_family:
parameter_refs: []
support_ref:
normalization_status:
mixture_component_refs: []
truncation_ref:
censoring_ref:
fit_method_ref:
validation_ref:
validity_envelope_ref:
evidence_refs: []
receipt_ref:
```

Supported forms include:

```text
parametric
nonparametric
empirical
mixture
hierarchical
multivariate
conditional
time_to_event
posterior
predictive
scenario_weighted
sample_based
```

---

## 6. Population, sample, and observation

```yaml
schema: focusa.statistical_sample.v1
sample_id:
target_population_ref:
sampling_frame_ref:
sampling_method_ref:
inclusion_criteria_refs: []
exclusion_criteria_refs: []
observation_refs: []
sample_size:
cluster_refs: []
weight_policy_ref:
missingness_profile_ref:
censoring_policy_ref:
collection_window_ref:
information_cutoff_ref:
representativeness_assessment_ref:
evidence_refs: []
receipt_ref:
```

The system MUST preserve:

- who or what could have entered the sample;
- how units were selected;
- why observations are missing;
- whether observations are independent;
- whether repeated measurements belong to the same unit;
- when each observation became available;
- whether outcomes influenced inclusion.

---

## 7. Statistical protocol

A consequential inference requires a frozen protocol before outcome inspection unless explicitly classified as exploratory.

```yaml
schema: focusa.statistical_protocol.v1
protocol_id:
question_ref:
analysis_class: confirmatory | exploratory | monitoring | predictive | causal
estimand_ref:
population_ref:
sample_design_ref:
variable_roles: []
model_refs: []
assumption_refs: []
primary_metric_refs: []
secondary_metric_refs: []
hypothesis_refs: []
error_rate_policy_ref:
multiple_testing_policy_ref:
stopping_rule_ref:
missingness_policy_ref:
outlier_policy_ref:
censoring_policy_ref:
transformation_refs: []
sensitivity_plan_refs: []
subgroup_policy_ref:
validation_plan_ref:
reporting_policy_ref:
frozen_at_stamp_ref:
protocol_hash:
status:
receipt_ref:
```

Post-outcome protocol changes create a new revision and are reported as exploratory unless a governing correction protocol applies.

---

## 8. Descriptive statistics

Focusa SHOULD support governed summaries including:

```text
count
sum
mean
weighted_mean
median
mode
minimum
maximum
range
variance
standard_deviation
median_absolute_deviation
quantiles
interquartile_range
skewness
kurtosis
covariance
correlation
rank_correlation
frequency_table
histogram
density_estimate
```

Every summary identifies:

- sample and weights;
- missing observations;
- units;
- time window;
- cohort and regime;
- robust versus nonrobust method;
- uncertainty where required.

---

## 9. Inference contract

```yaml
schema: focusa.statistical_inference.v1
inference_id:
protocol_ref:
sample_ref:
estimand_ref:
estimator_ref:
model_ref:
estimate_ref:
standard_error_ref:
interval_ref:
test_statistic_ref:
p_value_ref:
posterior_ref:
effect_size_ref:
practical_significance_ref:
power_or_precision_ref:
assumption_check_refs: []
sensitivity_result_refs: []
correction_refs: []
conclusion_class:
validity_envelope_ref:
evidence_refs: []
receipt_ref:
```

Conclusion classes include:

```text
descriptive_only
association_supported
predictive_skill_supported
causal_effect_supported
insufficient_evidence
inconclusive
protocol_violated
not_identified
experimental_candidate
```

---

## 10. Hypothesis testing

Hypothesis tests MUST preserve:

- null and alternative;
- directionality;
- statistic;
- reference distribution or permutation policy;
- significance/error-rate policy;
- effect size;
- confidence interval;
- power or precision;
- sample-size justification;
- multiple-testing family;
- stopping rule;
- practical significance.

A statistically significant result without meaningful effect or valid design cannot satisfy a decision or causal claim by itself.

---

## 11. Bayesian inference

Bayesian results MUST preserve:

- prior identity and justification;
- likelihood identity;
- posterior computation method;
- diagnostics and convergence;
- posterior predictive checks;
- sensitivity to prior and model choices;
- credible intervals and decision policy;
- model comparison policy;
- data and information cutoff.

The prior is part of the model, not hidden context.

---

## 12. Experiment design

```yaml
schema: focusa.experiment_design.v1
experiment_id:
question_ref:
hypothesis_refs: []
treatment_refs: []
control_refs: []
experimental_unit_ref:
randomization_policy_ref:
blocking_or_stratification_refs: []
sample_size_plan_ref:
power_or_precision_target_ref:
measurement_schedule_ref:
primary_outcome_ref:
secondary_outcome_refs: []
interference_policy_ref:
blinding_policy_ref:
stopping_rule_ref:
safety_boundary_refs: []
data_quality_policy_ref:
analysis_protocol_ref:
registration_stamp_ref:
status:
receipt_ref:
```

`focusa_stats_design` SHOULD be able to propose the smallest experiment that materially discriminates competing hypotheses, subject to authority and safety.

---

## 13. Sequential and online analysis

The system MUST distinguish fixed-horizon analysis from sequential monitoring.

Required concepts:

```text
interim_look
alpha_spending
posterior_stopping
confidence_sequence
sequential_probability_ratio
online_calibration
drift_monitor
change_point
false_discovery_control
```

Repeatedly checking an ordinary fixed-horizon p-value without correction is a protocol violation.

---

## 14. Missingness, censoring, and selection

Missingness classes:

```text
MCAR candidate
MAR candidate
MNAR candidate
structural_missing
sensor_failure
access_restricted
not_yet_observed
right_censored
left_censored
interval_censored
truncated
```

The system MUST NOT treat unresolved predictions, unavailable measurements, dropped failures, or absent users as ordinary negative observations without a governed policy.

---

## 15. Dependence and source structure

Required structures:

```text
cluster
repeated_measure
hierarchy
panel
spatial_dependence
temporal_dependence
shared_upstream_source
common_cause
network_dependence
```

Naive independent-observation methods are rejected when material dependence is known or detected.

---

## 16. Time series and temporal inference

Time-series analysis MUST bind Spec 137/137B temporal authority and preserve:

- event time;
- first-available time;
- ingestion time;
- revision and vintage;
- sampling cadence;
- irregular intervals;
- market/session calendar;
- seasonality;
- autocorrelation;
- nonstationarity;
- regime transitions;
- look-ahead and target leakage;
- walk-forward validation;
- horizon-specific scoring.

Random train/test splitting is prohibited when it permits future regime information to enter past evaluation.

---

## 17. Calibration and prediction evaluation

Spec 152A supplies statistical primitives to Spec 138/138B for:

```text
reliability
resolution
sharpness
bias
Brier decomposition
log score
coverage
proper-score uncertainty
calibration intervals
small-sample shrinkage
cohort comparison
drift detection
skill against baseline
decision value
```

Calibration MUST be cohortable by:

- domain;
- target type;
- horizon;
- model and provider;
- strategy;
- regime;
- source versus transfer;
- assurance tier;
- decision class.

---

## 18. Causal analysis

```yaml
schema: focusa.causal_analysis_contract.v1
causal_question_id:
exposure_or_intervention_ref:
outcome_ref:
target_population_ref:
causal_graph_ref:
identification_strategy_ref:
confounder_refs: []
mediator_refs: []
moderator_refs: []
instrument_refs: []
negative_control_refs: []
positivity_assessment_ref:
consistency_assessment_ref:
interference_policy_ref:
measurement_validity_ref:
estimand_ref:
estimator_ref:
sensitivity_refs: []
falsification_refs: []
validity_envelope_ref:
conclusion_class:
receipt_ref:
```

A causal claim requires an identified estimand and justified assumptions. Predictive performance does not establish causation.

---

## 19. Causal claim classes

```text
association
conditional_association
predictive_relationship
causal_hypothesis
causal_effect_candidate
causal_effect_supported
mechanism_candidate
mechanism_supported
refuted
not_identified
```

Physical mechanism claims additionally require Spec 153/153A applicability.

---

## 20. Bias and failure taxonomy

```text
selection_bias
survivorship_bias
collider_bias
confounding
measurement_bias
recall_bias
publication_bias
multiple_testing
p_hacking
optional_stopping
subgroup_fishing
data_leakage
target_leakage
temporal_leakage
train_test_contamination
dependent_source_double_counting
regression_to_mean
base_rate_neglect
nonstationarity
concept_drift
covariate_shift
label_shift
outlier_sensitivity
model_misspecification
heteroskedasticity
autocorrelation
small_sample_overconfidence
```

Each failure produces a typed finding, affected claim references, severity, Evidence, and recovery path.

---

## 21. Statistical reflexes

Deterministic and advisory reflexes include:

```text
detect_small_sample
detect_selection_bias
detect_survivorship_bias
detect_multiple_testing
detect_unregistered_stopping
detect_data_leakage
detect_temporal_leakage
detect_distribution_shift
detect_nonstationarity
detect_autocorrelation
detect_dependence
detect_confounding
detect_heteroskedasticity
detect_outlier_sensitivity
detect_unidentified_parameters
detect_calibration_drift
detect_false_precision
request_power_analysis
request_sensitivity_analysis
request_replication
request_negative_control
```

Execution and authority integration are governed by Spec 152B.

---

## 22. High-value tools

```text
focusa_stats_infer
focusa_stats_design
focusa_stats_verify
focusa_stats_power
focusa_stats_calibrate
focusa_stats_drift
focusa_stats_bias_audit
focusa_stats_causal
focusa_stats_experiment_discriminator
focusa_stats_replay
focusa_stats_explain
```

The top-level tool family remains small; narrow methods are discovered through the Operation Registry.

---

## 23. Statistical computation providers

Provider profiles may include:

```text
DescriptiveStatisticsProvider
ClassicalInferenceProvider
BayesianInferenceProvider
BootstrapProvider
PermutationTestProvider
SurvivalAnalysisProvider
TimeSeriesProvider
CausalInferenceProvider
ExperimentDesignProvider
CalibrationProvider
DriftDetectionProvider
```

Every provider declares supported methods, assumptions, diagnostics, numeric representation, deterministic/reproducibility posture, resource bounds, fixtures, and known limitations.

---

## 24. Verification obligations

Spec 144 MUST compile obligations for:

```text
statistics.protocol_frozen
statistics.population_defined
statistics.sample_validity
statistics.missingness_handled
statistics.dependence_modeled
statistics.estimand_identified
statistics.estimator_applicable
statistics.assumptions_checked
statistics.multiple_testing_controlled
statistics.stopping_rule_valid
statistics.power_or_precision_sufficient
statistics.effect_size_reported
statistics.practical_significance_reported
statistics.temporal_integrity
statistics.calibration_valid
statistics.causal_identification
statistics.sensitivity_complete
statistics.reproducible
```

---

## 25. Human-facing output

Statistical explanation SHOULD expose:

- question and population;
- sample and collection method;
- effect and uncertainty;
- assumptions;
- practical meaning;
- alternative interpretations;
- power and limitations;
- whether the result is descriptive, predictive, or causal;
- what evidence would change the conclusion.

It MUST NOT reduce results to “significant” or “not significant” without context.

---

## 26. Persistence and replay

Canonical state preserves:

- protocol revisions;
- population and sample identity;
- observation and vintage references;
- models and estimands;
- analysis runs;
- diagnostics;
- inference results;
- corrections;
- drift and bias findings;
- conclusion classification;
- Evidence and Receipts.

Large datasets and posterior samples remain behind handles.

---

## 27. Required tests

1. probability outside `[0,1]` rejected;
2. categorical mass must normalize or declare residual mass;
3. undefined population blocks generalization;
4. biased sampling cannot claim representativeness;
5. repeated measures cannot be treated as independent;
6. missingness and censoring are not flattened;
7. post-outcome primary metric changes are classified exploratory;
8. multiple testing without policy is detected;
9. optional stopping is detected;
10. small sample produces appropriate uncertainty or abstention;
11. time-series random split leakage is blocked;
12. revised data cannot masquerade as point-in-time available data;
13. calibration comparison uses matched cohorts and horizons;
14. high correlation cannot mint a causal claim;
15. unidentified causal effect remains `not_identified`;
16. effect size and practical significance are preserved;
17. Bayesian prior sensitivity is reported;
18. sequential analysis uses an authorized stopping method;
19. drift invalidates stale calibration where policy requires;
20. replay reconstructs the frozen protocol and result.

---

## 28. Acceptance criteria

Spec 152A is accepted only when:

1. probability, confidence, reliability, and uncertainty remain distinct;
2. populations, samples, observations, missingness, censoring, and dependence are first-class;
3. confirmatory protocols freeze before outcome inspection;
4. inference reports effect, uncertainty, power/precision, assumptions, and practical consequence;
5. fixed-horizon and sequential analysis remain distinct;
6. multiple testing and stopping are governed;
7. temporal leakage and point-in-time data integrity are enforced;
8. calibration and prediction evaluation compose with Spec 138B;
9. association, prediction, causation, and mechanism remain separate claim classes;
10. causal analysis requires identification and sensitivity;
11. reflexes, providers, verifier obligations, tools, replay, Evidence, Receipts, and client parity are complete;
12. no Vertical duplicates or weakens the statistical substrate.

---

## 29. Canonical summary

```text
Statistics in Focusa is not an after-the-fact score table.
It governs how evidence is sampled, how uncertainty is represented,
how experiments are designed, how predictions are calibrated,
and how causal claims are admitted or refused.
```
