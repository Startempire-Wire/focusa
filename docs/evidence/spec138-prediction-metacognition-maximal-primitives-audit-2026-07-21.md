# Spec 138 Prediction and Metacognition Maximal-Primitives Audit

**Date:** 2026-07-21  
**Source baseline:** `b08acf4f6a1f73c61a07cf6845b37efbad316ddb`  
**Status:** research audit supporting a normative draft  
**Target specification:** `docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md`

## 1. Purpose

This audit determines the deepest generic prediction and metacognition primitives that Focusa should own before a domain application such as Focusa Market Lab builds a specialized predictive and metacognitive algorithm.

The design objective is intentionally maximal rather than minimal:

> Define the broadest coherent, reusable, domain-general substrate for forecasts, outcomes, calibration, epistemic uncertainty, learning, self-evaluation, transfer, consolidation, and governed self-improvement while keeping raw domain feeds, domain models, broker logic, and domain policy outside Focusa core.

The table supplied by the operator contains leading indicators, lagging indicators, contextual layers, weighted scoring, triangulation, cross-validation, scenarios, and public underground-threat-intelligence summaries. The individual feeds and market interpretations are not Focusa-core primitives, but they expose generic semantics that Focusa must represent.

## 2. Current implementation reality

### 2.1 Prediction strengths

Current prediction support includes:

- typed `WorkstreamKey` scoping;
- scope-partitioned CRDT storage;
- trajectory context;
- ontology context;
- a bounded predicted-outcome string;
- one floating-point confidence value;
- recommended action and rationale;
- outcome capture;
- a score field;
- recent and stats routes;
- CLI and Pi bindings;
- a prediction-to-metacognition and metacognition-to-prediction flywheel.

These are real and useful foundations.

### 2.2 Prediction shallowness

The current core object does not yet distinguish:

- a forecast question from its answer;
- binary probability from confidence in evidence;
- categorical distributions from prose;
- point estimates from intervals or quantiles;
- event forecasts from numeric, ranking, return, volatility, or time-to-event targets;
- the original immutable forecast from later outcome and evaluation records;
- outcome resolution from scoring;
- resolution authority, dispute, revision, void, or censoring;
- scoring policy identity and version;
- calibration cohort, baseline, uncertainty, sharpness, skill, or decision value;
- information-set identity and point-in-time availability;
- model, prompt, dataset, feature, source, code, container, and policy provenance;
- forecast dependencies, contradictions, scenarios, or ensembles;
- source quality, independence, redundancy, or manipulation risk;
- forecast revision policy and lineage;
- meta-predictions about model competence or forecast reliability.

The fallback prediction scorer is based on text containment unless the caller supplies a score. That is not sufficient for canonical statistical evaluation.

### 2.3 Metacognition strengths

Current metacognition support includes:

- capture;
- retrieval;
- reflection;
- adjustment;
- evaluation;
- scoped storage;
- recent readbacks;
- a hot index;
- promotion into retrievable learning;
- follow-up transfer predictions;
- a separate scheduled reflection overlay.

These provide a functioning workflow.

### 2.4 Metacognition shallowness

The current lifecycle is still mostly API-local and prose-oriented:

- metacognition record types are defined in the API route rather than `focusa-core`;
- observations and metrics are mostly strings;
- reflection hypotheses and strategy updates are templated;
- expected adjustment deltas are hard-coded placeholders;
- promotion is based primarily on the number of metric strings plus the presence of selected updates;
- poor predictions do not reliably produce learning captures;
- applicability, exclusions, expiry, review, transfer, conflict, supersession, revocation, rollback, and negative transfer are not first-class;
- retrieval is keyword-centered rather than applicability-, ontology-, evidence-, and transfer-centered;
- persistence helpers may ignore write failures;
- reflection and metacognition have overlapping but incompletely unified authority.

## 3. Maximal ownership boundary

### Focusa core should own

- forecast and question identity;
- information-set references;
- generic observations, indicators, evidence, and source trust semantics;
- forecast shapes and distributions;
- uncertainty decomposition;
- outcome claims and resolution authority;
- scoring interfaces and calibration;
- ensemble and weighted-fusion semantics;
- scenario and counterfactual semantics;
- metacognitive capture and reflection claims;
- adjustment and intervention proposals;
- learning evaluation;
- promotion, inhibition, expiry, conflict, supersession, revocation, and rollback;
- transfer and negative-transfer tracking;
- long-horizon consolidation, abstraction, retention, decay, forgetting, archive, and reactivation;
- self-model, competence, bias, and error-mode tracking;
- provenance, receipts, policy versions, and authority boundaries.

### Focusa Market Lab should own

- the actual market data sources;
- market indicator definitions and feature calculations;
- stock, option, crypto, macro, geopolitical, policy, environment, demographic, logistics, social, and threat-intelligence interpretations;
- market-specific source reliability estimates;
- market-specific signal weights;
- market regimes and forecast targets;
- strategy and portfolio policies;
- alert thresholds such as `400` or `650`;
- broker and execution behavior;
- financial utility, slippage, fees, capital, and risk.

### External research workers should own

- raw ingestion;
- large numeric tables;
- feature computation;
- model fitting;
- backtesting;
- walk-forward validation;
- ablation;
- large-scale scenario simulation;
- high-volume calibration computation.

Focusa stores bounded, typed, evidence-linked records and canonical learning authority, not every raw row.

## 4. Operator table mapped to generic primitives

### 4.1 Leading and lagging indicators

The table requires a generic `IndicatorRole` supporting at least:

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
```

Each indicator must expose:

- target or target family;
- expected lead/lag distribution;
- horizon applicability;
- direction and polarity;
- strength scale;
- normalization method;
- source latency;
- first-available time;
- freshness and decay;
- reliability;
- independence and redundancy;
- causal status;
- regime sensitivity;
- historical performance;
- uncertainty;
- evidence and source lineage.

### 4.2 Triangulation

The table's requirement for agreement across indicators implies:

- `TriangulationPolicy`;
- `EvidenceCluster`;
- `SourceIndependenceGraph`;
- `AgreementMeasure`;
- `ContradictionMeasure`;
- `RedundancyPenalty`;
- `CorrelationPenalty`;
- `MinimumIndependentSupport`;
- `DiversityRequirement`;
- `TriangulationResult`.

Agreement among copies of the same upstream report cannot count as independent confirmation.

### 4.3 Leading-plus-lagging cross-validation

The table requires relationships such as:

```text
leading_indicator_predicts_target
lagging_indicator_confirms_prior_target
lagging_indicator_refutes_prior_signal
indicator_validates_indicator
indicator_updates_reliability
```

The confirmation record must not rewrite the original information set or award hindsight credit to a forecast that lacked the later data.

### 4.4 Weighted scoring

The example weighted model implies generic primitives for:

- signal-strength scales;
- weight policies;
- normalization;
- context-specific weights;
- learned weights;
- prior and posterior weights;
- contribution decomposition;
- source-quality adjustment;
- freshness decay;
- uncertainty discount;
- correlation and redundancy penalties;
- missingness policy;
- imputation policy;
- threshold policy;
- hysteresis;
- sensitivity analysis;
- versioned weight lineage;
- champion/challenger comparison.

The table's specific percentages and alert thresholds are domain policy, not core defaults.

### 4.5 Contextual layers

Geopolitical, policy, environmental, demographic, cultural, logistical, social, and economic layers imply:

- `ContextEnvelope`;
- `ContextDimension`;
- `ContextObservation`;
- `RegimeDefinition`;
- `RegimeAssignment`;
- `RegimeTransition`;
- `ContextInteraction`;
- `EffectModifier`;
- `ContextualWeightOverride`;
- `ContextMissingness`.

### 4.6 Scenario analysis

The scenario combinations require:

- `ScenarioDefinition`;
- `ScenarioAssumptionSet`;
- `ScenarioBranch`;
- `BranchProbability`;
- `ScenarioTrigger`;
- `ScenarioPath`;
- `ScenarioOutcome`;
- `ScenarioStressTest`;
- `ScenarioSensitivity`;
- `CounterfactualScenario`;
- `ScenarioComparison`;
- `ScenarioRevision`.

### 4.7 Public underground-threat-intelligence summaries

The supplied table explicitly limits this category to public OSINT summaries and aggregated reports. Focusa therefore needs generic sensitive-source controls rather than direct underground access functionality:

- `SourceAccessClass`;
- `AcquisitionAuthority`;
- `PublicSummaryOnly` policy;
- `LegalBasisReference`;
- `TermsAndLicenseReference`;
- `SensitiveSourceClass`;
- `AdversarialContentRisk`;
- `PromptInjectionRisk`;
- `ManipulationRisk`;
- `DeceptionRisk`;
- `AttributionConfidence`;
- `SourceSanitizationReceipt`;
- `QuarantineDisposition`;
- `HumanReviewRequirement`;
- `RetentionRestriction`;
- `RedistributionRestriction`.

Focusa core must never create a primitive that authorizes illegal access. It represents claims and provenance supplied through lawful adapters.

## 5. Additional maximal primitives not explicit in the table

The table is broad, but a durable general substrate also requires the following.

### 5.1 Forecast-question semantics

- target ontology;
- event definition;
- population and entity scope;
- conditional assumptions;
- time horizon;
- resolution rule;
- decision relevance;
- abstention option;
- invalidation conditions.

### 5.2 Distributional forecasts

- binary probability;
- categorical distribution;
- point estimate;
- interval;
- quantiles;
- rank distribution;
- expected value;
- time-to-event distribution;
- joint and conditional distributions;
- scenario distribution;
- causal-effect distribution.

### 5.3 Uncertainty decomposition

- aleatoric;
- epistemic;
- data;
- source;
- model;
- parameter;
- structural;
- regime;
- temporal;
- outcome-resolution;
- measurement;
- execution;
- missingness;
- unknown/other.

### 5.4 Meta-prediction

- forecast of forecast correctness;
- forecast of model competence;
- forecast of source failure;
- forecast of calibration drift;
- forecast of learning transfer;
- forecast of intervention effect;
- value-of-information forecast;
- expected-regret forecast.

### 5.5 Self-model and bias

- competence profile;
- known-unknown boundary;
- overconfidence and underconfidence;
- anchoring;
- confirmation bias;
- recency bias;
- survivorship bias;
- hindsight bias;
- narrative bias;
- base-rate neglect;
- double-counting;
- source-dependency blindness;
- correlation-causation confusion;
- selection and publication bias;
- leakage and contamination;
- action bias and omission bias.

### 5.6 Learning transfer

- context similarity;
- applicability;
- exclusions;
- transfer prediction;
- transfer outcome;
- transfer score;
- negative transfer;
- partial transfer;
- transfer decay;
- domain shift;
- supersession;
- revocation;
- rollback.

### 5.7 Memory lifecycle

- capture;
- deduplication;
- clustering;
- abstraction;
- consolidation;
- compression;
- canonicalization;
- conflict preservation;
- retention;
- decay;
- forgetting;
- archive;
- reactivation;
- legal hold;
- deletion authority.

## 6. Maximum breadth without runtime bloat

A maximal specification does not require every primitive in every hot-path record.

Spec 138 should define:

```text
maximal canonical vocabulary
+
small composable records
+
capability and activation profiles
+
bounded projections
+
feature-gated implementation stages
```

Recommended profiles:

```text
core_recording
proper_scoring
calibration
source_fusion
scenario_analysis
metacognitive_learning
transfer_tracking
memory_consolidation
high_consequence_governance
```

Applications activate only the profiles their evidence and cost justify.

## 7. Core findings requiring a new primitive-owning spec

1. Prediction records are too prose-centric for general quantitative forecasts.
2. Confidence semantics are overloaded.
3. Outcome resolution and scoring are not separate authorities.
4. Caller-supplied arbitrary scores can become canonical.
5. Calibration is not a first-class core service.
6. Information-set identity and point-in-time integrity are missing.
7. Source trust, independence, and weighted fusion are not modeled.
8. Scenario and counterfactual forecasts are not modeled.
9. Metacognition is an API workflow rather than a complete core type family.
10. Reflection claims, learning candidates, and promoted learning are not separated.
11. Promotion policy is far shallower than existing Spec 80 design requirements.
12. High-confidence misses are not prioritized correctly.
13. Transfer and negative transfer are not measured.
14. Learning applicability, expiry, conflict, supersession, and rollback are not first-class.
15. Persistence failures can be hidden.
16. Reflection and metacognition authority need unification.
17. Long-term consolidation and forgetting are not integrated with learning validity.
18. Existing prediction math is not a general scorer/calibration registry.

## 8. Required specification decision

Create a new primitive-owning standalone specification rather than another narrow implementation note:

> **Spec 138 — Focusa Prediction, Outcome, Calibration, Metacognitive Learning, Transfer, and Epistemic Governance**

It should be maximal in vocabulary and authority while allowing staged implementation.

## 9. Required Market Lab follow-up

After Spec 138 is approved, Market Lab should add a specialized layer provisionally named:

> **Market Predictive and Metacognitive Intelligence (MPMI)**

MPMI will consume Spec 138 and implement:

- market indicators and source adapters;
- the operator's leading/lagging indicator catalog;
- market-specific weighted fusion;
- regime models;
- scenarios;
- stock, option, and crypto targets;
- actor/portfolio signals;
- source and feature ablation;
- financial utility;
- champion/challenger forecasting;
- long-horizon market learning.

Spec 138 must not hard-code the table's feeds, percentages, or alert thresholds.
