# 138 — Focusa Prediction, Outcome, Calibration, Metacognitive Learning, Transfer, and Epistemic Governance Spec

**Status:** Normative draft; primitive-owning; implementation not implied
**Owner:** Focusa core
**Source baseline:** `b08acf4f6a1f73c61a07cf6845b37efbad316ddb`
**Research audit:** `docs/evidence/spec138-prediction-metacognition-maximal-primitives-audit-2026-07-21.md`
**Amends and deepens:** current Prediction, Metacognition, reflection, project-card algorithm, and prediction/metacognition flywheel surfaces
**Depends on:** Specs 45–50/135F ontology; Spec 76 retention/decay; Spec 80 metacognition tooling; Spec 96 trajectory projection; Spec 104 scoped CRDT; Spec 119 receipts; Spec 131 Workpoint Item timing; Spec 133 governed sessions; combined Spec 137 + Spec 137A temporal authority
**Does not activate:** market feeds, financial trading, live mutation, external source access, or domain-specific models

---

## 0. Executive requirement

Focusa MUST provide a maximal, domain-general substrate for:

- asking forecastable questions;
- committing immutable forecasts;
- representing distributions and uncertainty;
- binding forecasts to exact information sets;
- resolving outcomes through typed authority;
- scoring forecasts with versioned policies;
- measuring calibration, skill, sharpness, drift, and decision value;
- fusing indicators without double-counting dependent evidence;
- representing scenarios and counterfactuals;
- capturing surprise, failure, success, bias, and uncertainty;
- generating reflection claims and adjustment proposals;
- evaluating whether an intervention actually improved outcomes;
- promoting, inhibiting, expiring, superseding, revoking, and rolling back learning;
- measuring transfer and negative transfer;
- consolidating long-term learning without erasing conflict or uncertainty;
- maintaining a self-model of competence and calibration;
- supporting specialized predictive engines such as Focusa Market Lab without making Focusa itself a market model.

This specification is intentionally maximal in primitive coverage.

Maximal coverage MUST NOT be interpreted as requiring every field in every hot-path record. Implementations MUST use composable records, bounded projections, capability profiles, and staged activation.

---

## 1. Problem statement

Current Focusa Prediction is a useful scoped record but is too shallow for full quantitative and long-horizon predictive authority:

- forecast questions and answers are not separated;
- outcome resolution and scoring are not separate authorities;
- confidence semantics are overloaded;
- forecast distributions are represented mostly as strings;
- caller-supplied scores can become canonical;
- calibration is not a first-class core family;
- information-set identity and point-in-time integrity are incomplete;
- source trust, dependence, weighted fusion, scenarios, and counterfactuals are not first-class.

Current Focusa Metacognition is a useful workflow but is too shallow for governed long-term learning:

- lifecycle types are primarily API-local;
- metrics and outcomes are largely prose strings;
- reflection outputs are often templated;
- adjustment expectations are placeholders;
- promotion is heuristic rather than evidence-governed;
- high-confidence misses are not consistently prioritized;
- applicability, expiry, transfer, conflict, supersession, revocation, and rollback are not first-class;
- reflection and metacognition authority are not fully unified.

Focusa therefore needs a primitive-owning epistemic substrate rather than another domain-specific algorithm.

---

## 2. Scope

### 2.1 In scope

- Generic forecast-question identity.
- Generic source, observation, indicator, evidence, and information-set semantics.
- Leading, coincident, lagging, confirmatory, contextual, and structural indicator roles.
- Generic weighted fusion, triangulation, independence, redundancy, and contradiction semantics.
- Typed forecast values and distributions.
- Uncertainty decomposition.
- Scenario, conditional, causal, and counterfactual forecasts.
- Immutable prediction commitments and typed revisions.
- Outcome claims, resolution, disputes, revision, voiding, and censoring.
- Scoring policies and calibration.
- Experiment validity and multiple-testing awareness.
- Metacognitive capture, reflection, adjustment, evaluation, and learning authority.
- Self-model, competence, bias, and error-mode tracking.
- Learning applicability, expiry, transfer, negative transfer, conflict, supersession, revocation, rollback, and consolidation.
- Storage, events, APIs, CLI, Pi tools, projections, and migration requirements.

### 2.2 Out of scope

- Raw market feeds.
- Broker integration.
- Market-specific indicators or weights.
- Stock, options, or crypto models.
- Domain-specific alert thresholds.
- Illegal or direct access to underground systems.
- A general-purpose data warehouse.
- Automatic live financial authority.
- Unbounded model self-modification.

---

## 3. Ownership and precedence

### 3.1 Spec 137 remains temporal authority

Spec 137 owns:

- trusted clocks;
- clock domains;
- temporal uncertainty;
- deadlines;
- urgency;
- civil-time intent;
- estimate claims;
- authority and lease expiry;
- temporal incidents.

Spec 138 consumes Spec 137 references for:

- forecast creation time;
- information-set as-of time;
- first-available time;
- forecast horizon;
- resolution window;
- evidence freshness;
- learning expiry and review times;
- transfer and drift windows.

Spec 138 MUST NOT create a competing clock or deadline authority.

### 3.2 Ontology remains semantic identity authority

Ontology owns canonical object, action, tool, relation, evidence, domain-pack, and semantic-graph identity.

Spec 138 records ontology references; it MUST NOT invent parallel object identity.

### 3.3 Evidence and Receipts remain proof authority

Evidence and Receipt primitives own proof handles and auditable mutation lineage.

Spec 138 requires evidence and receipts but does not replace their primitive ownership.

### 3.4 Domain applications own domain algorithms

Applications own:

- data-source adapters;
- raw observations;
- feature calculations;
- domain models;
- domain weights and thresholds;
- domain utility and risk;
- high-volume analytics.

Focusa owns the canonical bounded forecast and learning authority records those applications submit.

---

## 4. Core laws

1. **Prediction commitment is immutable.** Later evidence creates new outcome, evaluation, revision, or supersession records.
2. **Question and answer are distinct.** A forecast cannot be interpreted without a target and resolution contract.
3. **Probability is not evidence confidence.** Forecast probability, source reliability, model confidence, and resolution confidence are separate.
4. **Information sets are frozen.** Later information never retroactively enters the forecast's original information set.
5. **Outcome resolution precedes scoring.** A score cannot define the outcome.
6. **Resolution authority is explicit.** Outcome claims are not canonical merely because a caller submits them.
7. **Scoring policy freezes before resolution.** Post-outcome scorer selection is prohibited except through an explicit correction record.
8. **Canonical scores identify scorer and version.** Anonymous floating-point scores are advisory only.
9. **Proper scoring is required for probabilistic forecasts.** Accuracy alone is insufficient.
10. **Abstention is a valid forecast action.** The system must not force unsupported precision.
11. **Unknown remains unknown.** Missingness or conflict cannot be silently converted into neutral or positive evidence.
12. **Dependent evidence is not independent confirmation.** Triangulation must account for shared upstream sources.
13. **Weights are versioned claims.** Every weight has origin, scope, evidence, and applicability.
14. **Composite scores expose contributions.** A single opaque total cannot be canonical without decomposition.
15. **Uncertainty is decomposed.** One confidence scalar cannot represent all uncertainty.
16. **Scenario probabilities must be coherent.** Branches, assumptions, and residual uncertainty are explicit.
17. **Counterfactuals are labeled.** Simulated outcomes cannot masquerade as observations.
18. **Reflection proposes; it does not promote.** LLM or heuristic reflection output is a candidate claim.
19. **Failure is learning evidence.** High-confidence misses and unexpected failures are first-class.
20. **Success is not automatically reusable.** A correct result may be luck, leakage, or regime-specific.
21. **Learning requires structured evaluation.** Metric names without values and evidence cannot prove improvement.
22. **Promotion requires applicability and expiry.** No learning is universally valid by default.
23. **Transfer is predicted and evaluated.** Reuse in a new context creates a transfer forecast and outcome.
24. **Negative transfer is retained.** Failed reuse cannot be hidden by aggregate success.
25. **Learning can conflict.** Competing lessons remain visible until resolved or scoped.
26. **Learning can be superseded, revoked, or rolled back.** Promotion is not permanent truth.
27. **Long-term memory preserves provenance.** Consolidation does not erase source records.
28. **Self-evolution uses champion/challenger governance.** Live or canonical policy never mutates itself in place.
29. **Agent authority remains bounded.** Agents may propose forecasts and learning but cannot grant themselves authority.
30. **Domain specialization remains outside core.** Focusa provides primitives, not hard-coded domain beliefs.

---

## 5. Primitive family catalog

The following primitive families are canonical vocabulary. Implementations MAY activate them in stages, but MUST NOT replace them with incompatible domain-local meanings.

### 5.1 Identity and scope

```text
PredictionProgram
PredictionQuestion
PredictionTarget
TargetDefinition
TargetEntityScope
TargetPopulationScope
TargetCohort
TargetMetric
TargetEventDefinition
TargetStateDefinition
TargetResolutionContract
ForecastHorizon
ForecastSeries
ForecastBatch
ForecastPortfolio
ForecastDependencyGraph
ForecastConsistencyConstraint
PredictionLineage
PredictionFork
PredictionSupersession
```

### 5.2 Source, acquisition, and evidence

```text
SourceIdentity
SourceVersion
SourceClass
SourceAccessClass
AcquisitionAuthority
AcquisitionMethod
SourceLicenseReference
SourceTermsReference
SourceRetentionPolicy
SourceRedistributionPolicy
SourceTrustProfile
SourceReliabilityProfile
SourceIndependenceProfile
SourceLatencyProfile
SourceFreshnessProfile
SourceRevisionPolicy
SourceManipulationRisk
SourceDeceptionRisk
SourcePromptInjectionRisk
SourcePoisoningRisk
SourceAttributionConfidence
SourceSanitizationReceipt
SourceQuarantineDisposition
EvidenceBundle
EvidenceClaim
EvidenceConflict
EvidenceRevision
EvidenceQualityProfile
```

### 5.3 Observation and indicator

```text
ObservationDefinition
ObservationValue
ObservationSeries
ObservationRevision
ObservationAvailability
ObservationUncertainty
IndicatorDefinition
IndicatorObservation
IndicatorRole
IndicatorDirection
IndicatorPolarity
IndicatorStrengthScale
IndicatorNormalizationPolicy
IndicatorThresholdPolicy
IndicatorLatencyProfile
IndicatorLeadLagProfile
IndicatorFreshnessPolicy
IndicatorDecayPolicy
IndicatorReliabilityProfile
IndicatorRegimeProfile
IndicatorCausalStatus
IndicatorTargetRelationship
IndicatorValidationRelationship
IndicatorMissingnessPolicy
IndicatorOutlierPolicy
IndicatorQualityProfile
IndicatorContribution
```

### 5.4 Indicator roles

`IndicatorRole` MUST support at least:

```text
leading
coincident
lagging
confirmatory
contextual
structural
trigger
inhibitor
moderator
mediator
proxy
control
negative_control
sentinel
anomaly
regime_marker
resolution_evidence
```

### 5.5 Feature and derived-state primitives

```text
FeatureDefinition
FeatureValue
FeatureSeries
FeatureWindow
FeatureTransformation
FeatureNormalization
FeatureImputation
FeatureMissingness
FeatureRevision
FeatureDependencyGraph
FeatureLeakageAssessment
FeatureStabilityProfile
FeatureDriftProfile
FeatureContribution
FeatureInteraction
FeatureSelectionRecord
FeatureAblationResult
```

### 5.6 Fusion, triangulation, and weighting

```text
FusionPolicy
CompositeSignal
CompositeScore
CompositeContribution
SignalWeight
WeightPolicy
WeightOrigin
WeightPrior
WeightPosterior
WeightContext
WeightRegimeOverride
WeightDecay
WeightUncertainty
WeightLearningRecord
WeightVersion
NormalizationPolicy
MissingnessPolicy
ImputationPolicy
CorrelationGraph
CorrelationPenalty
RedundancyGraph
RedundancyPenalty
IndependenceGraph
TriangulationPolicy
TriangulationResult
AgreementMeasure
ContradictionMeasure
DiversityRequirement
MinimumIndependentSupport
SourceAgreementProfile
SourceConflictProfile
SensitivityAnalysis
ThresholdPolicy
HysteresisPolicy
AlertBand
```

### 5.7 Information-set primitives

```text
InformationSetSnapshot
InformationSetItem
AvailabilityTimestamp
FirstAvailableTimestamp
AsOfTimestamp
SourceSnapshotReference
DatasetSnapshotReference
FeatureSnapshotReference
ObservationSnapshotReference
SignalSnapshotReference
ExcludedInformationReference
UnavailableInformationReference
StaleInformationReference
LateInformationReference
LeakageCertificate
ContaminationRecord
InformationSetHash
InformationSetRevision
```

### 5.8 Forecaster and model provenance

```text
ForecasterIdentity
ForecasterClass
ModelIdentity
ModelVersion
ModelArtifactReference
PromptIdentity
PromptVersion
PolicyIdentity
PolicyVersion
CodeCommitReference
ContainerDigestReference
DependencyManifestReference
TrainingDatasetReference
TrainingWindow
ValidationWindow
HyperparameterSet
RandomSeedReference
FeatureSetReference
CalibrationVersion
ScorerVersion
BaselineManifestReference
ReproducibilityManifest
```

### 5.9 Forecast commitment primitives

```text
PredictionCommitment
ForecastValue
ForecastDistribution
ForecastCondition
ForecastAssumption
ForecastConstraint
ForecastRationale
ForecastEvidenceMap
ForecastActionRecommendation
ForecastDecisionRelevance
ForecastAbstention
ForecastInvalidationCondition
ForecastExpiry
ForecastRevisionProposal
ForecastStatus
```

### 5.10 Forecast shapes

`ForecastValue` MUST support at least:

```text
binary_probability
categorical_distribution
ordinal_distribution
numeric_point
numeric_interval
quantile_set
distribution_reference
ranking
pairwise_preference
expected_value
expected_utility
return_distribution
volatility_distribution
drawdown_distribution
time_to_event
hazard_curve
survival_curve
count_distribution
rate_distribution
joint_distribution
conditional_distribution
hierarchical_distribution
scenario_distribution
causal_effect_distribution
counterfactual_distribution
ensemble_distribution
custom_registered
abstain
```

### 5.11 Uncertainty primitives

```text
UncertaintyEnvelope
UncertaintyDecomposition
AleatoricUncertainty
EpistemicUncertainty
DataUncertainty
MeasurementUncertainty
SourceUncertainty
ModelUncertainty
ParameterUncertainty
StructuralUncertainty
RegimeUncertainty
TemporalUncertaintyReference
ResolutionUncertainty
ScenarioUncertainty
ExecutionUncertainty
MissingnessUncertainty
AttributionUncertainty
UnknownUncertainty
UncertaintyCorrelation
UncertaintyBudget
UncertaintyPropagationPolicy
ConfidenceInterval
CredibleInterval
PredictionInterval
```

### 5.12 Scenario and counterfactual primitives

```text
ScenarioDefinition
ScenarioAssumptionSet
ScenarioBranch
ScenarioBranchProbability
ScenarioTrigger
ScenarioPath
ScenarioState
ScenarioOutcome
ScenarioResidualBranch
ScenarioStressTest
ScenarioSensitivity
ScenarioComparison
ScenarioRevision
ScenarioInvalidation
CounterfactualQuestion
CounterfactualAssumption
CounterfactualWorld
CounterfactualOutcome
CounterfactualComparison
CausalGraphReference
CausalEffectClaim
InterventionVariable
ConfounderReference
```

### 5.13 Meta-prediction primitives

```text
ForecastCorrectnessPrediction
ForecasterCompetencePrediction
SourceFailurePrediction
SourceRevisionPrediction
CalibrationDriftPrediction
RegimeShiftPrediction
LearningTransferPrediction
InterventionEffectPrediction
ResolutionRiskPrediction
DataAvailabilityPrediction
ValueOfInformationPrediction
ExpectedRegretPrediction
ModelSelectionPrediction
ToolRoutePrediction
ActionSuccessPrediction
```

### 5.14 Outcome and resolution primitives

```text
OutcomeClaim
OutcomeClaimant
OutcomeEvidence
OutcomeResolver
OutcomeResolverVersion
ResolutionRule
ResolutionPolicy
ResolutionAuthority
ResolutionWindow
ResolutionAttempt
OutcomeResolution
OutcomeComponent
PartialOutcome
CompositeOutcome
OutcomeUncertainty
OutcomeDispute
OutcomeConflict
OutcomeRevision
OutcomeCorrection
OutcomeVoid
OutcomeCensoring
CensoringReason
OutcomeFinality
ResolutionReceipt
```

### 5.15 Scoring and calibration primitives

```text
ScoringPolicy
ScoringRule
ScorerIdentity
ScorerVersion
ScoreComponent
PredictionEvaluation
EvaluationCohort
EvaluationWindow
EvaluationBaseline
BaselineForecast
SkillScore
CalibrationCohort
CalibrationBucket
CalibrationReport
ReliabilityCurve
CalibrationError
SharpnessMeasure
CoverageMeasure
BiasMeasure
DiscriminationMeasure
RankingMeasure
DecisionValueMeasure
AsymmetricLossPolicy
CostMatrix
RegretMeasure
ValueOfInformationMeasure
EvaluationUncertainty
EvaluationCorrection
EvaluationReceipt
```

### 5.16 Required scoring registry

The core scoring registry SHOULD support at least:

```text
binary_accuracy
multiclass_accuracy
brier_score
multiclass_brier_score
log_loss
multiclass_log_loss
spherical_score
continuous_ranked_probability_score
mean_absolute_error
mean_squared_error
root_mean_squared_error
mean_absolute_percentage_error
symmetric_mape
quantile_pinball_loss
interval_coverage
interval_width
winkler_interval_score
rank_correlation
information_coefficient
top_k_precision
top_k_recall
ndcg
concordance_index
survival_brier_score
expected_calibration_error
maximum_calibration_error
adaptive_calibration_error
skill_score
expected_utility
realized_regret
custom_registered
```

Domain applications MAY register additional scorers but MUST provide identity, version, direction, range, assumptions, and test fixtures.

### 5.17 Decision and action primitives

```text
DecisionCandidate
DecisionContext
DecisionPolicy
DecisionThreshold
DecisionBand
DecisionHysteresis
ActionCandidate
ActionRecommendation
ActionAbstention
ActionCost
ActionBenefit
ActionRisk
ActionConstraint
ExpectedUtilityModel
ExpectedRegretModel
ValueOfInformationPolicy
InformationAcquisitionAction
DecisionOutcomeLink
DecisionReceipt
```

### 5.18 Experiment and validation primitives

```text
PredictionExperiment
PredictionHypothesis
ExperimentProtocol
ControlDefinition
TreatmentDefinition
RandomizationPolicy
HoldoutPolicy
WalkForwardPolicy
CrossValidationPolicy
NegativeControl
PositiveControl
AblationDefinition
AblationResult
SensitivityResult
RobustnessResult
MultipleTestingFamily
MultipleTestingCorrection
FalseDiscoveryAssessment
BacktestOverfittingAssessment
LeakageAssessment
ContaminationAssessment
SurvivorshipAssessment
SelectionBiasAssessment
PublicationBiasAssessment
ExperimentOutcome
ExperimentConclusion
ExperimentPromotionGate
```

### 5.19 Metacognitive signal primitives

```text
MetacognitiveSignal
LearningSignal
SurpriseSignal
PredictionMissSignal
HighConfidenceMiss
LowConfidenceMiss
UnexpectedSuccess
UnexpectedFailure
CalibrationFailure
SourceFailure
DataQualityFailure
ToolFailure
ExecutionFailure
AuthorityFailure
ScopeFailure
TemporalFailure
ResolutionFailure
ReasoningFailure
CausalFailure
RegimeFailure
TransferFailure
NegativeTransferSignal
ContradictionSignal
AnomalySignal
BiasSignal
DriftSignal
OperatorOverrideSignal
```

### 5.20 Reflection primitives

```text
ReflectionRequest
ReflectionWindow
ReflectionInputSet
ReflectionClaim
ReflectionObservation
ReflectionHypothesis
AlternativeHypothesis
NullHypothesis
RootCauseHypothesis
CausalAttribution
ContributingFactor
Confounder
Counterevidence
DisconfirmingEvidence
UnknownCause
ErrorClassification
BiasClassification
CapabilityGap
CompetenceAssessment
ReflectionConfidence
ReflectionUncertainty
ReflectionRecommendation
ReflectionReceipt
```

### 5.21 Adjustment and intervention primitives

```text
AdjustmentProposal
AdjustmentTarget
PolicyAdjustment
StrategyAdjustment
WeightAdjustment
ThresholdAdjustment
DataAdjustment
ToolRouteAdjustment
PromptAdjustment
ModelAdjustment
ProcessAdjustment
SafetyAdjustment
InterventionDefinition
InterventionMechanism
ExpectedEffect
ExpectedEffectDistribution
AffectedMetric
AffectedContext
AdjustmentConstraint
AdjustmentRisk
AdjustmentRollback
AdjustmentEvaluationPlan
AdjustmentStatus
AdjustmentReceipt
```

### 5.22 Learning evaluation primitives

```text
MetricDefinition
MetricObservation
MetricBaseline
MetricExpectation
MetricDelta
MetricDirection
MetricUnit
MetricSampleSize
MetricUncertainty
MetricEvidence
EffectSize
StatisticalSignificance
PracticalSignificance
RobustnessAssessment
RegressionAssessment
ConfounderAssessment
NegativeControlResult
PositiveControlResult
InterventionAdherence
EvaluationCompleteness
LearningEvaluation
LearningEvaluationConclusion
```

### 5.23 Learning candidate and promotion primitives

```text
LearningCandidate
LearningStatement
LearningType
LearningEvidenceMap
LearningRationale
LearningConfidence
LearningUncertainty
ApplicabilityEnvelope
ApplicabilityCondition
ApplicabilityExclusion
ApplicabilityCohort
ApplicabilityRegime
ApplicabilityHorizon
LearningRisk
LearningExpiryPolicy
LearningReviewPolicy
LearningRollbackPolicy
PromotionPolicy
PromotionGate
PromotionDecision
PromotionInhibition
PromotionOverride
PromotionReceipt
LearningRecord
LearningVersion
LearningStatus
```

Promotion dispositions MUST include:

```text
promote
inhibit
continue_experiment
reject
expire
supersede
revoke
rollback
quarantine
operator_review_required
```

### 5.24 Transfer primitives

```text
LearningApplication
ApplicationContext
ContextSimilarity
ContextDifference
TransferPrediction
TransferExpectation
TransferOutcome
TransferScore
TransferSuccess
TransferFailure
PartialTransfer
NegativeTransfer
TransferUncertainty
TransferDecay
TransferCohort
TransferHistory
TransferCalibration
TransferReview
TransferReceipt
```

### 5.25 Learning conflict and change primitives

```text
LearningConflict
ConflictType
ConflictEvidence
ConflictScope
ConflictResolution
LearningSupersession
LearningRevocation
LearningRollback
LearningCorrection
LearningDeprecation
LearningReinstatement
LearningLineage
LearningFork
```

### 5.26 Memory lifecycle primitives

```text
LearningMemory
MemoryCluster
MemoryDuplicate
MemoryAbstraction
MemoryGeneralization
MemorySpecialization
MemoryConsolidation
MemoryCompression
MemoryCanonicalization
MemoryConflictSet
MemoryRetentionPolicy
MemoryDecayProfile
MemoryForgettingDecision
MemoryArchive
MemoryReactivation
MemoryLegalHold
MemoryDeletionAuthority
MemoryMigration
MemoryIntegrityReceipt
```

### 5.27 Self-model and competence primitives

```text
SelfModel
CapabilityProfile
CompetenceDomain
CompetenceEstimate
CompetenceUncertainty
CompetenceBoundary
KnownKnown
KnownUnknown
UnknownKnown
UnknownUnknown
CalibrationProfile
OverconfidenceProfile
UnderconfidenceProfile
AbstentionProfile
ErrorModeProfile
RecoveryProfile
LearningRateProfile
TransferProfile
SelfModelRevision
```

### 5.28 Bias and reasoning-risk primitives

```text
AnchoringBias
ConfirmationBias
RecencyBias
AvailabilityBias
SurvivorshipBias
HindsightBias
NarrativeBias
BaseRateNeglect
CorrelationCausationError
SelectionBias
PublicationBias
ActionBias
OmissionBias
OverfittingRisk
UnderfittingRisk
LeakageRisk
DoubleCountingRisk
SourceDependencyBlindness
MotivatedReasoningRisk
PrematureClosureRisk
ScopeDriftRisk
AuthorityConfusionRisk
BiasMitigation
BiasOutcome
```

### 5.29 Drift and regime primitives

```text
RegimeDefinition
RegimeObservation
RegimeAssignment
RegimeProbability
RegimeTransition
RegimeDuration
RegimeSimilarity
RegimeUncertainty
RegimeForecast
RegimeInvalidation
ConceptDrift
DataDrift
FeatureDrift
SourceDrift
CalibrationDrift
OutcomeDrift
PolicyDrift
TransferDrift
DriftDetectionPolicy
DriftAlert
DriftDisposition
DriftRecoveryPlan
```

### 5.30 Governance and safety primitives

```text
EpistemicAuthority
PredictionAuthority
ResolutionAuthority
ScoringAuthority
PromotionAuthority
RevocationAuthority
OverrideAuthority
HumanReviewRequirement
IndependentReviewRequirement
HighConsequenceProfile
SensitiveSourcePolicy
AcquisitionComplianceRecord
PrivacyClass
RetentionRestriction
RedistributionRestriction
AdversarialInputPolicy
PromptInjectionDisposition
PoisoningDisposition
ManipulationDisposition
QuarantineRecord
SafetyInvariant
PolicyViolation
GovernanceReceipt
```

---

## 6. Canonical type contracts

The following contracts are representative minimum fields for the maximal families. Exact Rust organization may split these into smaller records.

### 6.1 Prediction question

```rust
pub struct PredictionQuestion {
    pub question_id: String,
    pub target_ref: String,
    pub question_text: String,
    pub target_type: String,
    pub entity_refs: Vec<String>,
    pub cohort_refs: Vec<String>,
    pub conditions: Vec<ForecastCondition>,
    pub horizon_ref: String,
    pub resolution_contract_ref: String,
    pub decision_relevance_ref: Option<String>,
    pub ontology_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub temporal_authority_ref: String,
}
```

### 6.2 Indicator observation

```rust
pub struct IndicatorObservation {
    pub indicator_observation_id: String,
    pub indicator_definition_ref: String,
    pub role: IndicatorRole,
    pub entity_refs: Vec<String>,
    pub target_refs: Vec<String>,
    pub direction: Option<String>,
    pub raw_value_ref: Option<String>,
    pub normalized_strength: Option<f64>,
    pub strength_scale_ref: Option<String>,
    pub event_time: Option<DateTime<Utc>>,
    pub first_available_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub source_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub reliability: Option<f64>,
    pub independence_refs: Vec<String>,
    pub uncertainty_ref: Option<String>,
    pub regime_refs: Vec<String>,
    pub provenance_ref: String,
}
```

### 6.3 Information-set snapshot

```rust
pub struct InformationSetSnapshot {
    pub information_set_id: String,
    pub as_of: DateTime<Utc>,
    pub first_committed_at: DateTime<Utc>,
    pub source_snapshot_refs: Vec<String>,
    pub observation_refs: Vec<String>,
    pub indicator_refs: Vec<String>,
    pub feature_refs: Vec<String>,
    pub scenario_refs: Vec<String>,
    pub excluded_refs: Vec<String>,
    pub unavailable_refs: Vec<String>,
    pub stale_refs: Vec<String>,
    pub late_refs: Vec<String>,
    pub leakage_certificate_ref: Option<String>,
    pub temporal_authority_ref: String,
    pub baseline_manifest_ref: String,
    pub snapshot_hash: String,
}
```

### 6.4 Forecast value

```rust
pub enum ForecastValue {
    BinaryProbability { probability: f64 },
    CategoricalDistribution { probabilities: BTreeMap<String, f64> },
    NumericPoint { value: Decimal, units: Option<String> },
    NumericInterval { lower: Decimal, upper: Decimal, coverage: f64 },
    Quantiles { values: BTreeMap<String, Decimal> },
    Ranking { ordered_refs: Vec<String>, score_refs: BTreeMap<String, f64> },
    ExpectedValue { value: Decimal, units: Option<String> },
    TimeToEvent { distribution_ref: String },
    HazardCurve { curve_ref: String },
    ScenarioDistribution { branch_probabilities: BTreeMap<String, f64> },
    CausalEffect { distribution_ref: String },
    CustomRegistered { schema_ref: String, payload_ref: String },
    Abstain { reason: String },
}
```

### 6.5 Prediction commitment

```rust
pub struct PredictionCommitment {
    pub prediction_id: String,
    pub schema_version: u16,
    pub scope: WorkstreamKey,
    pub question_ref: String,
    pub forecast: ForecastValue,
    pub information_set_ref: String,
    pub uncertainty_ref: String,
    pub assumptions: Vec<String>,
    pub constraints: Vec<String>,
    pub rationale_ref: Option<String>,
    pub recommended_action_ref: Option<String>,
    pub evaluation_policy_ref: String,
    pub provenance_ref: String,
    pub trajectory_ref: Option<String>,
    pub ontology_refs: Vec<String>,
    pub committed_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub temporal_authority_ref: String,
    pub commitment_hash: String,
}
```

### 6.6 Uncertainty decomposition

```rust
pub struct UncertaintyDecomposition {
    pub uncertainty_id: String,
    pub aleatoric: Option<f64>,
    pub epistemic: Option<f64>,
    pub data: Option<f64>,
    pub source: Option<f64>,
    pub model: Option<f64>,
    pub parameter: Option<f64>,
    pub structural: Option<f64>,
    pub regime: Option<f64>,
    pub temporal_ref: Option<String>,
    pub resolution: Option<f64>,
    pub missingness: Option<f64>,
    pub other: BTreeMap<String, f64>,
    pub correlation_refs: Vec<String>,
    pub propagation_policy_ref: String,
    pub summary: Option<String>,
}
```

### 6.7 Weight policy and contribution

```rust
pub struct SignalWeight {
    pub weight_id: String,
    pub signal_definition_ref: String,
    pub target_ref: String,
    pub raw_weight: f64,
    pub normalized_weight: f64,
    pub origin: WeightOrigin,
    pub context_refs: Vec<String>,
    pub regime_refs: Vec<String>,
    pub reliability_adjustment: Option<f64>,
    pub freshness_adjustment: Option<f64>,
    pub uncertainty_adjustment: Option<f64>,
    pub redundancy_adjustment: Option<f64>,
    pub correlation_adjustment: Option<f64>,
    pub effective_weight: f64,
    pub evidence_refs: Vec<String>,
    pub version: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

pub struct CompositeContribution {
    pub signal_ref: String,
    pub normalized_strength: f64,
    pub effective_weight: f64,
    pub signed_contribution: f64,
    pub uncertainty_discount: f64,
    pub final_contribution: f64,
    pub explanations: Vec<String>,
}
```

### 6.8 Outcome resolution

```rust
pub struct OutcomeResolution {
    pub resolution_id: String,
    pub prediction_ref: String,
    pub question_ref: String,
    pub resolver_ref: String,
    pub resolver_version: String,
    pub authority_ref: String,
    pub resolution_status: String,
    pub resolved_value_ref: String,
    pub component_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub uncertainty_ref: Option<String>,
    pub dispute_refs: Vec<String>,
    pub censoring_ref: Option<String>,
    pub resolved_at: DateTime<Utc>,
    pub temporal_authority_ref: String,
    pub resolution_hash: String,
}
```

### 6.9 Prediction evaluation

```rust
pub struct PredictionEvaluation {
    pub evaluation_id: String,
    pub prediction_ref: String,
    pub resolution_ref: String,
    pub scoring_policy_ref: String,
    pub scorer_refs: Vec<String>,
    pub score_components: Vec<ScoreComponent>,
    pub baseline_refs: Vec<String>,
    pub skill_scores: Vec<ScoreComponent>,
    pub decision_value_ref: Option<String>,
    pub cohort_refs: Vec<String>,
    pub uncertainty_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
    pub evaluation_hash: String,
}
```

### 6.10 Metric observation

```rust
pub struct MetricObservation {
    pub metric_observation_id: String,
    pub metric_definition_ref: String,
    pub baseline_value: Option<f64>,
    pub expected_value: Option<f64>,
    pub observed_value: f64,
    pub delta: Option<f64>,
    pub units: Option<String>,
    pub preferred_direction: MetricDirection,
    pub sample_size: Option<u64>,
    pub effect_size: Option<f64>,
    pub uncertainty_ref: Option<String>,
    pub evaluation_window_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub provenance_ref: String,
}
```

### 6.11 Learning candidate

```rust
pub struct LearningCandidate {
    pub candidate_id: String,
    pub learning_type: String,
    pub statement: String,
    pub source_signal_refs: Vec<String>,
    pub reflection_claim_refs: Vec<String>,
    pub adjustment_refs: Vec<String>,
    pub evaluation_refs: Vec<String>,
    pub applicability_ref: String,
    pub exclusions: Vec<String>,
    pub uncertainty_ref: String,
    pub expiry_policy_ref: String,
    pub review_policy_ref: String,
    pub rollback_policy_ref: String,
    pub conflict_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub status: String,
}
```

### 6.12 Learning record

```rust
pub struct LearningRecord {
    pub learning_id: String,
    pub candidate_ref: String,
    pub version: u32,
    pub statement: String,
    pub applicability_ref: String,
    pub promotion_decision_ref: String,
    pub confidence_ref: String,
    pub evidence_refs: Vec<String>,
    pub transfer_history_refs: Vec<String>,
    pub conflict_refs: Vec<String>,
    pub supersedes_refs: Vec<String>,
    pub status: String,
    pub promoted_at: DateTime<Utc>,
    pub review_after: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### 6.13 Transfer outcome

```rust
pub struct TransferOutcome {
    pub transfer_outcome_id: String,
    pub learning_ref: String,
    pub application_ref: String,
    pub transfer_prediction_ref: String,
    pub source_context_ref: String,
    pub target_context_ref: String,
    pub similarity_ref: String,
    pub metric_observation_refs: Vec<String>,
    pub result: String,
    pub transfer_score: Option<f64>,
    pub negative_transfer: bool,
    pub evidence_refs: Vec<String>,
    pub resolved_at: DateTime<Utc>,
}
```

---

## 7. Weighted fusion model requirements

Focusa MUST support weighted models without hard-coding any domain weights.

### 7.1 Signal-strength scales

A signal-strength scale MUST define:

- scale identity and version;
- minimum and maximum;
- whether the scale is ordinal, interval, ratio, probability, or score;
- direction and polarity;
- normalization method;
- missing and unknown behavior;
- outlier handling;
- evidence and calibration history.

### 7.2 Weight origins

`WeightOrigin` MUST distinguish:

```text
operator_defined
expert_defined
policy_defined
historical_backtest
walk_forward_learned
bayesian_prior
bayesian_posterior
model_learned
regime_specific
transfer_inherited
emergency_override
experimental
```

### 7.3 Effective contribution

A generic effective contribution MAY be calculated from:

```text
normalized signal strength
× base weight
× source reliability adjustment
× freshness/decay adjustment
× uncertainty adjustment
× regime adjustment
× applicability adjustment
× independence adjustment
− redundancy/correlation penalties
```

The implementation MUST preserve every intermediate contribution.

### 7.4 Missingness

Missing indicators MUST be handled through an explicit policy:

```text
exclude_and_renormalize
retain_zero_contribution
use_prior
use_imputation
block_composite
mark_indeterminate
```

Unknown MUST NOT silently become zero unless the selected policy says zero is semantically valid.

### 7.5 Correlation and dependence

The system MUST support:

- shared-source detection;
- upstream-source lineage;
- same-event duplication;
- correlation estimates;
- causal dependence claims;
- redundancy penalties;
- effective independent evidence count.

### 7.6 Thresholds and alert bands

Thresholds MUST be versioned policy, not embedded in the score definition.

A threshold policy SHOULD support:

- bands;
- entry and exit thresholds;
- hysteresis;
- dwell time;
- cooldown;
- minimum evidence;
- minimum independent support;
- uncertainty guard;
- operator review requirements.

---

## 8. Triangulation and contradiction

### 8.1 Triangulation policy

A `TriangulationPolicy` MUST define:

- minimum total support;
- minimum independent support;
- minimum source diversity;
- required source classes;
- prohibited shared dependencies;
- conflict tolerance;
- freshness requirements;
- evidence-quality requirements;
- outcome-specific exceptions.

### 8.2 Contradictory evidence

Contradiction MUST remain visible.

A composite forecast MUST expose:

- supporting evidence;
- opposing evidence;
- unresolved conflict;
- source dependence;
- confidence impact;
- whether conflict caused abstention.

### 8.3 Leading and lagging relationships

Lagging confirmation MAY update:

- indicator reliability;
- weight posterior;
- calibration history;
- learning candidates.

It MUST NOT alter the original forecast's information set or create hindsight credit.

---

## 9. Forecast commitment lifecycle

```text
draft
→ committed
→ active
→ awaiting_resolution
→ resolvable
→ resolved
→ evaluated
→ archived
```

Exceptional states:

```text
expired
void
censored
disputed
corrected
superseded
quarantined
```

Rules:

- `draft` MAY change.
- `committed` forecast content is immutable.
- a revised forecast creates a new commitment linked by lineage;
- expired forecasts remain evaluable when policy permits;
- void and censored outcomes are not scored as ordinary failures;
- disputed resolutions remain non-final until authority resolves them.

---

## 10. Outcome resolution authority

### 10.1 Resolver contract

An outcome resolver MUST declare:

- target types supported;
- required evidence;
- source precedence;
- first-valid resolution time;
- revision handling;
- ambiguity handling;
- partial outcome handling;
- void and censoring rules;
- dispute process;
- authority identity;
- version.

### 10.2 Outcome revisions

When an external source revises data:

- preserve original resolution;
- append revised claim and resolution;
- identify affected evaluations;
- compute corrected scores as new evaluation records;
- preserve both historical and corrected calibration views.

### 10.3 Canonical score authority

A caller-supplied score MAY be stored as `advisory_score`.

It MUST NOT become canonical unless:

- the caller has scoring authority;
- scorer identity and version are supplied;
- the scoring policy permits external scoring;
- required evidence is linked;
- the score passes schema and range validation.

---

## 11. Calibration requirements

Calibration MUST be first-class rather than a label on average accuracy.

### 11.1 Required dimensions

Calibration reports SHOULD support grouping by:

- prediction type;
- target;
- horizon;
- entity and cohort;
- source set;
- indicator family;
- feature set;
- model version;
- prompt version;
- policy version;
- forecaster;
- confidence/probability bucket;
- regime;
- scenario;
- trajectory;
- environment;
- time period;
- transfer versus original context.

### 11.2 Required measures

A calibration report SHOULD include:

- sample size;
- evaluated and unresolved counts;
- censoring and void counts;
- reliability;
- bias;
- sharpness;
- coverage;
- discrimination;
- proper score;
- baseline score;
- skill score;
- uncertainty bounds;
- missingness;
- cohort drift;
- decision value;
- tail performance;
- high-confidence miss rate;
- abstention rate and quality.

### 11.3 Small samples

Small cohorts MUST back off to supported parent cohorts.

Reports MUST expose:

- cohort used;
- parent cohort;
- backoff depth;
- sample size;
- effective sample size;
- uncertainty;
- whether the result is descriptive or authority-bearing.

---

## 12. Metacognitive priority model

Metacognition MUST prioritize learning signals by expected learning value rather than success alone.

A priority policy SHOULD consider:

```text
forecast confidence
× outcome surprise
× consequence
× recurrence probability
× transfer opportunity
× evidence quality
× unresolved uncertainty
− analysis cost
```

High-priority cases include:

- high-confidence miss;
- unexpected success with weak rationale;
- repeated low-confidence correctness;
- severe consequence despite ordinary score;
- source or resolver failure;
- negative transfer;
- regime break;
- repeated contradiction;
- calibration drift;
- operator override revealing authority mismatch.

---

## 13. Reflection authority

### 13.1 Reflection outputs are claims

Reflection engines, including LLMs, MAY generate:

- observations;
- hypotheses;
- causal attributions;
- alternative explanations;
- bias classifications;
- recommendations;
- adjustment proposals.

They MUST NOT directly create canonical promoted learning.

### 13.2 Causal discipline

A causal attribution MUST identify:

- claimed cause;
- claimed mechanism;
- supporting evidence;
- opposing evidence;
- confounders;
- alternative hypotheses;
- intervention or counterfactual basis;
- uncertainty;
- causal-status label.

Causal-status labels SHOULD include:

```text
descriptive_association
predictive_association
plausible_mechanism
quasi_experimental_support
experimental_support
operator_judgment
unknown
```

### 13.3 Reflection scheduler boundary

Scheduled reflection MAY produce candidate signals and hypotheses.

The metacognitive learning authority MUST evaluate them through the same candidate, evidence, applicability, and promotion lifecycle as manually initiated reflection.

---

## 14. Adjustment and intervention evaluation

An adjustment proposal MUST specify:

- target policy, model, process, tool, prompt, weight, threshold, or strategy;
- expected mechanism;
- expected metric changes;
- expected effect distribution;
- applicability;
- exclusions;
- risks;
- rollback;
- evaluation window;
- baseline;
- positive and negative controls when applicable;
- stop conditions.

Hard-coded generic expected deltas are prohibited unless the policy explicitly defines them as priors rather than observed evidence.

---

## 15. Promotion policy

A learning candidate MUST NOT be promoted solely because fields are populated.

### 15.1 Minimum promotion gates

A promotion policy SHOULD evaluate:

1. schema completeness;
2. evidence availability;
3. evidence quality;
4. outcome authority;
5. sample size or justified single-event exception;
6. baseline comparison;
7. expected versus observed metric deltas;
8. practical significance;
9. uncertainty;
10. regression checks;
11. confounders;
12. negative controls;
13. applicability and exclusions;
14. expiry and review policy;
15. rollback path;
16. conflict with existing learning;
17. transfer evidence when claiming generality;
18. security and authority constraints.

### 15.2 Single-event promotion

A single event MAY justify promotion only when:

- consequence is high;
- evidence is strong;
- the lesson is narrow;
- applicability is tightly bounded;
- review is scheduled;
- rollback is available;
- policy explicitly permits single-event promotion.

### 15.3 Promotion does not equal truth

Promoted learning is a governed reusable policy claim with status, applicability, evidence, uncertainty, and review—not timeless truth.

---

## 16. Transfer and long-term outcome tracking

### 16.1 Transfer prediction

Before applying a learning record in a materially new context, Focusa SHOULD record:

- expected applicability;
- context similarity;
- important differences;
- expected benefit;
- expected risk;
- transfer confidence;
- evaluation plan.

### 16.2 Transfer evaluation

After use, Focusa SHOULD record:

- adherence;
- metric outcomes;
- comparison to expectation;
- negative effects;
- context differences;
- whether the lesson should remain, narrow, expand, expire, supersede, or revoke.

### 16.3 Longitudinal learning value

A learning record SHOULD expose:

- application count;
- successful transfers;
- failed transfers;
- negative transfers;
- partial transfers;
- last applied;
- last success;
- last failure;
- cumulative benefit;
- cumulative harm;
- calibration of transfer predictions;
- applicability drift;
- review status.

---

## 17. Self-model and competence calibration

Focusa SHOULD maintain a scoped self-model that answers:

- Which forecast classes are well calibrated?
- Which contexts produce overconfidence?
- Where does the system abstain appropriately?
- Which sources or tools frequently fail?
- Which strategies transfer well?
- Which error modes recur?
- Which domains lack enough evidence?
- Which learning methods produce durable gains?

The self-model MUST be evidence-backed and versioned.

It MUST NOT be represented as a single global competence score.

---

## 18. Bias and error taxonomy

The core taxonomy SHOULD include at least:

```text
data_missingness
data_revision
data_leakage
data_contamination
source_failure
source_dependency
source_manipulation
source_deception
measurement_error
label_error
resolution_error
scoring_error
model_underfit
model_overfit
parameter_instability
concept_drift
regime_shift
selection_bias
survivorship_bias
publication_bias
anchoring
confirmation_bias
recency_bias
availability_bias
hindsight_bias
narrative_bias
base_rate_neglect
correlation_causation_error
double_counting
overconfidence
underconfidence
premature_closure
action_bias
omission_bias
tool_route_error
scope_error
authority_error
temporal_error
execution_error
negative_transfer
unknown
```

Applications MAY extend the taxonomy through ontology/domain packs.

---

## 19. Memory consolidation and forgetting

### 19.1 Consolidation

Consolidation MAY:

- cluster similar learning records;
- deduplicate exact equivalents;
- abstract shared applicability;
- preserve exceptions;
- compress summaries;
- create parent/child learning relationships;
- propose canonical learning records.

It MUST preserve source lineage and conflicts.

### 19.2 Forgetting

Forgetting MUST be governed by Spec 76-compatible policy.

A learning record MAY decay because of:

- age;
- disuse;
- transfer failure;
- regime change;
- supersession;
- evidence invalidation;
- source revision;
- policy change.

High-consequence or legally retained records MUST NOT be deleted through ordinary decay.

### 19.3 Reactivation

Archived or decayed learning MAY be reactivated when:

- a similar context returns;
- new evidence supports it;
- a superseding record is revoked;
- the operator explicitly restores it.

Reactivation creates a receipt and review requirement.

---

## 20. Sensitive and adversarial source handling

### 20.1 Public-summary-only underground intelligence

Focusa MAY represent claims derived from lawful public OSINT summaries about underground or threat activity.

Focusa MUST NOT provide primitives that authorize direct illegal access.

Adapters MUST declare:

```text
source_access_class
acquisition_authority
legal_basis_ref
terms_ref
public_summary_only=true|false
sanitization_receipt
attribution_confidence
manipulation_risk
prompt_injection_risk
retention_policy
redistribution_policy
```

### 20.2 Adversarial content

External content MUST be treated as data, never instruction authority.

A source-derived claim may be:

```text
accepted
degraded
quarantined
rejected
blocked
human_review_required
```

### 20.3 Source reliability is target-specific

A source may be reliable for one claim family and unreliable for another.

Reliability MUST be scoped by:

- target;
- horizon;
- entity class;
- context;
- regime;
- acquisition method;
- source version;
- historical evaluation window.

---

## 21. Temporal integration

Every high-consequence Spec 138 record MUST include an applicable Spec 137 temporal reference.

At minimum:

- prediction commitment;
- information-set snapshot;
- source observation;
- outcome resolution;
- evaluation;
- learning expiry;
- transfer application;
- drift alert.

Time-based semantics MUST distinguish:

- event time;
- publication time;
- first-available time;
- ingestion time;
- commitment time;
- effective time;
- resolution time;
- evaluation time;
- review time;
- expiry time.

---

## 22. Storage and event model

### 22.1 Append-only semantic events

The canonical history SHOULD use separate append-only typed events:

```text
prediction_question_created
information_set_committed
prediction_committed
prediction_revision_proposed
prediction_superseded
outcome_claimed
outcome_disputed
outcome_resolved
outcome_corrected
prediction_evaluated
calibration_report_created
metacognitive_signal_captured
reflection_claim_created
adjustment_proposed
adjustment_applied
learning_evaluated
learning_promoted
learning_inhibited
learning_expired
learning_superseded
learning_revoked
learning_rolled_back
learning_applied
transfer_resolved
memory_consolidated
memory_archived
memory_reactivated
```

### 22.2 CRDT projections

Scoped CRDT records MAY project current state.

Whole-record last-writer selection MUST NOT be the only semantic history for prediction, outcome, evaluation, or learning authority.

### 22.3 Persistence

Canonical writes MUST:

- return explicit success or failure;
- use atomic or transactional boundaries appropriate to the backend;
- survive restart;
- preserve checksums;
- support migration;
- support backup and restore;
- expose corruption and partial-write conditions;
- never report durable success when persistence failed.

---

## 23. API surface

The final API shape may evolve, but canonical operations SHOULD include:

```text
POST /v1/prediction-questions
POST /v1/information-sets
POST /v1/predictions/commit
POST /v1/predictions/{id}/supersede
GET  /v1/predictions/{id}
GET  /v1/predictions/recent
POST /v1/outcomes/claim
POST /v1/outcomes/{id}/dispute
POST /v1/outcomes/resolve
POST /v1/evaluations/predictions
GET  /v1/calibration/reports
POST /v1/metacognition/signals
POST /v1/metacognition/reflections
POST /v1/metacognition/adjustments
POST /v1/metacognition/evaluations
POST /v1/learning/candidates/{id}/decide
POST /v1/learning/{id}/apply
POST /v1/learning/transfers/resolve
GET  /v1/learning/retrieve
GET  /v1/learning/conflicts
POST /v1/learning/{id}/expire
POST /v1/learning/{id}/revoke
POST /v1/learning/{id}/rollback
POST /v1/learning/consolidate
GET  /v1/self-model
```

Legacy `/v1/predictions` and `/v1/metacognition/*` routes MUST remain available through compatibility adapters until migration is complete.

---

## 24. CLI and Pi tools

Suggested CLI families:

```text
focusa predict question create
focusa predict commit
focusa predict show
focusa predict recent
focusa predict supersede
focusa outcome claim
focusa outcome resolve
focusa outcome dispute
focusa predict evaluate
focusa predict calibration
focusa metacog signal capture
focusa metacog reflect
focusa metacog adjust
focusa metacog evaluate
focusa learning candidate decide
focusa learning retrieve
focusa learning apply
focusa learning transfer resolve
focusa learning expire
focusa learning revoke
focusa learning rollback
focusa learning consolidate
focusa self-model show
```

Pi tools SHOULD expose compact bounded contracts and retrieve full artifacts by reference rather than injecting entire histories.

---

## 25. Focus Slice and UI projections

Focus Slice SHOULD eventually support compact cards:

### `PREDICTIVE_CONTEXT`

- active forecast questions;
- recent commitments;
- pending resolutions;
- calibration summary;
- high-confidence misses;
- current uncertainty and abstention posture;
- recommended next evaluation.

### `METACOG_CONTEXT`

- highest-priority learning signals;
- relevant promoted learning;
- conflicts and expiry warnings;
- pending adjustment evaluations;
- transfer success and failure;
- self-model competence warnings.

### `EPISTEMIC_HEALTH`

- unresolved outcomes;
- stale information sets;
- calibration drift;
- source reliability drift;
- promotion backlog;
- negative transfer;
- persistence health.

UI is projection only and does not own promotion or scoring authority.

---

## 26. Legacy migration

### 26.1 Prediction v1 migration

Existing `PredictionValue` records remain readable.

A migration adapter SHOULD map:

```text
prediction_type → target/question family hint
predicted_outcome → legacy text forecast
confidence → legacy_ambiguous_confidence
recommended_action → action recommendation text
why → rationale text
actual_outcome → legacy outcome claim
score → advisory legacy score unless scorer authority is known
learning_signal_ref → learning linkage
ontology_context → ontology references
trajectory → trajectory reference
```

Legacy predictions MUST be labeled as insufficient for strong calibration unless their forecast and scoring semantics can be reconstructed.

### 26.2 Metacognition v1 migration

Existing captures, reflections, adjustments, and evaluations remain readable.

Migration SHOULD create:

- legacy learning signals;
- legacy reflection claims;
- legacy adjustment proposals;
- advisory evaluations;
- unverified promotion status where evidence and metric semantics are insufficient.

No legacy heuristic promotion should automatically become a high-authority LearningRecord.

---

## 27. Capability profiles

Maximal vocabulary MUST be activated through profiles.

### Profile A — Core recording

- questions;
- information sets;
- prediction commitments;
- outcomes;
- evaluations;
- receipts.

### Profile B — Proper scoring and calibration

- scorer registry;
- calibration cohorts;
- reliability and skill reports;
- small-sample backoff.

### Profile C — Source and indicator fusion

- indicator roles;
- weights;
- triangulation;
- independence;
- contradiction;
- sensitivity.

### Profile D — Scenario and causal analysis

- scenarios;
- counterfactuals;
- causal claims;
- stress tests.

### Profile E — Metacognitive learning

- learning signals;
- reflection claims;
- adjustment proposals;
- structured metric evaluation;
- promotion.

### Profile F — Transfer and self-model

- applicability;
- transfer prediction;
- transfer outcome;
- competence profiles;
- bias and error profiles.

### Profile G — Consolidation and long-horizon governance

- consolidation;
- conflict;
- expiry;
- supersession;
- revocation;
- rollback;
- retention and forgetting.

### Profile H — High-consequence governance

- independent review;
- strict resolution/scoring authority;
- sensitive-source handling;
- stronger evidence, retention, and receipt requirements.

---

## 28. Implementation order

### Order 0 — Reconciliation and contracts

- Pin baseline.
- Produce complete feature ledger.
- Reconcile Specs 76, 80, 96, 104, 119, 131, 133, 135F, and 137.
- Define compatibility and migration policy.

### Order 1 — Core type extraction

- Move canonical prediction and metacognition types into `focusa-core`.
- Add question, information set, commitment, outcome, evaluation, learning candidate, and learning record types.
- Add versioned schemas.

### Order 2 — Append-only event storage

- Add typed scoped events.
- Build CRDT/current-state projections.
- Add durable explicit-failure persistence.
- Add import and migration.

### Order 3 — Scoring registry and calibration

- Expose existing Brier/log-loss math through generic scorers.
- Add forecast-shape-specific scoring.
- Add calibration cohorts and reports.
- Add scorer authority.

### Order 4 — Metacognitive authority

- Replace string-count promotion with structured evaluation.
- Prioritize high-confidence misses.
- Add applicability, expiry, review, rollback, conflict, and revocation.

### Order 5 — Transfer and self-model

- Add transfer forecasts and outcomes.
- Add competence and bias profiles.
- Add negative-transfer reporting.

### Order 6 — Fusion and scenarios

- Add indicator roles, weights, triangulation, contradiction, independence, scenarios, and sensitivity.

### Order 7 — Consolidation

- Add clustering, abstraction, conflict preservation, retention, decay, archive, and reactivation.

### Order 8 — Surfacing and automation

- Focus Slice cards.
- End-of-task automatic evaluation proposals.
- Governed consolidation cadence.
- Drift alerts.

---

## 29. Acceptance gates

### Gate A — Primitive completeness

- Canonical types exist in core.
- Schemas are versioned.
- Ownership and precedence are explicit.
- No duplicate temporal or ontology authority.

### Gate B — Forecast integrity

- Original commitments are immutable.
- Information sets are frozen.
- Outcome resolution is separate.
- Scoring policy is frozen.
- Arbitrary caller scores cannot silently become canonical.

### Gate C — Calibration

- Probabilistic scorers are proper.
- Cohort reports expose sample size and uncertainty.
- Reliability, sharpness, skill, and drift are available.
- Small-sample backoff works.

### Gate D — Learning authority

- High-confidence misses create candidates.
- Metrics are typed.
- Promotion evaluates evidence and effect, not field count.
- Applicability, expiry, conflict, and rollback are enforced.

### Gate E — Transfer

- Learning application creates transfer prediction.
- Transfer outcomes are measured.
- Negative transfer is visible.
- Self-model updates are evidence-backed.

### Gate F — Persistence and migration

- Writes fail explicitly.
- Restart and restore pass.
- Legacy records remain readable.
- Legacy heuristic scores/promotions are clearly labeled.

### Gate G — Governance and security

- Sensitive-source policy is enforced.
- External content lacks instruction authority.
- Receipts cover every consequential mutation.
- Agent self-promotion and authority escalation are blocked.

---

## 30. Market Lab specialization boundary

Focusa Market Lab should consume this spec through a specialized engine, provisionally:

> **Market Predictive and Metacognitive Intelligence (MPMI)**

MPMI owns:

- market-source registry;
- leading and lagging market indicators;
- market-specific feature engineering;
- weights and thresholds;
- source admission and ablation;
- market regimes;
- stock, option, crypto, macro, policy, geopolitical, environmental, demographic, logistics, social, and threat-intelligence interpretations;
- portfolio and tracked-actor signals;
- financial utility and asymmetric loss;
- champion/challenger models;
- market calibration and long-horizon learning.

Spec 138 owns the generic authority records MPMI writes.

The operator-supplied example weights and `low/moderate/high` thresholds belong to MPMI policy and experiments, not Focusa core.

---

## 31. Final invariants

1. Focusa core is domain-general.
2. Spec 137 remains temporal authority.
3. Ontology remains semantic identity authority.
4. Evidence and Receipts remain proof authority.
5. Prediction questions are explicit.
6. Prediction commitments are immutable.
7. Information sets are point-in-time and frozen.
8. Forecast probability is distinct from evidence and model confidence.
9. Unknown and abstention remain valid states.
10. Outcome resolution is separate from scoring.
11. Canonical scores are versioned and authorized.
12. Calibration includes reliability, sharpness, skill, uncertainty, and decision value.
13. Dependent evidence is not independent confirmation.
14. Weighted scores expose every contribution and penalty.
15. Scenarios expose assumptions and residual uncertainty.
16. Counterfactuals never masquerade as observations.
17. Reflection produces claims, not promoted truth.
18. High-confidence misses are priority learning events.
19. Success alone does not prove reusable learning.
20. Promotion requires structured evidence and metric evaluation.
21. Learning has applicability, exclusions, expiry, review, and rollback.
22. Transfer is predicted and evaluated.
23. Negative transfer remains visible.
24. Learning conflicts remain visible.
25. Learning may be superseded, revoked, or rolled back.
26. Consolidation preserves provenance and exceptions.
27. Forgetting is governed and reversible where policy permits.
28. Self-model claims are scoped and evidence-backed.
29. Live or canonical champions never mutate themselves in place.
30. Agents cannot grant themselves prediction, scoring, promotion, or financial authority.
31. External content is data, not instruction authority.
32. Direct illegal source access is never authorized by this spec.
33. Legacy records remain readable but cannot manufacture strong epistemic claims.
34. Domain applications own domain models and thresholds.
35. Every consequential epistemic mutation is receipted.
36. The system may always report `unknown`, `insufficient evidence`, or `abstain`.

<!-- SPEC137A_138A_144_ARCHITECTURE_CLOSURE:mandatory-spec138a-companion -->
## Mandatory companion: Spec 138A

Spec 138A is a mandatory companion to Spec 138. Full-profile runtime conformance
remains bound to the current evidence-gated Spec 138 activation receipt and is not
inferred from documentation closure or partial prediction primitives.
