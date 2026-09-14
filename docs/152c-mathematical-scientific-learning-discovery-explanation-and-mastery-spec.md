# Spec 152C — Mathematical-Scientific Learning, Discovery, Explanation, and Mastery

**Status:** NORMATIVE DRAFT — LEARNING AND DISCOVERY GOVERNING — IMPLEMENTATION NOT IMPLIED  
**Owner:** Focusa Core / Quantitative Learning / Scientific Discovery  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Depends on:** Spec 138 learning and transfer, Spec 144 verification, Specs 152/152A/152B, Specs 153/153A, UXP/UFI, Workpoints, Evidence, Receipts, Mission Canvas

---

## 0. Constitutional directive

```text
USE REAL MISSIONS AS THE CURRICULUM.
SHOW THE MATHEMATICS WITHOUT PRETENDING TO SHOW PRIVATE REASONING.
DISCOVER BOLDLY, LABEL CONJECTURES HONESTLY, AND SEARCH FOR COUNTEREXAMPLES.
MASTERY REQUIRES EXPLANATION, APPLICATION, TRANSFER, AND RETENTION.
```

Focusa MUST help agents and humans become better quantitative and scientific reasoners while preventing attractive equations, correlations, simulations, or visualizations from being promoted as verified knowledge without proof.

---

## 1. Purpose

This specification turns quantitative work into:

- inspectable explanations;
- progressive mathematical learning;
- concept mastery;
- misconception detection;
- derivation and equation maps;
- simulation and visualization experiences;
- conjecture and hypothesis portfolios;
- counterexample and replication workflows;
- transferable scientific learning.

It supports the operator's desire to learn mathematics, statistics, and physics through actual Focusa, Wirebot, robotics, markets, software, finance, sailing, and scientific projects.

---

## 2. Ownership

### 2.1 This specification owns

- quantitative concept and prerequisite graphs;
- mastery and misconception records;
- explanation levels and educational artifacts;
- derivation, equation-map, and limiting-case artifacts;
- practice, transfer, and retention evidence;
- conjecture, counterexample, invariant, and residual-discovery portfolios;
- discovery experiment and replication plans;
- learning-path personalization;
- educational surface projection;
- promotion of human-learning records.

### 2.2 This specification does not own

- mathematical truth or computation;
- statistical inference;
- physical measurement or experiment settlement;
- prediction scoring;
- canonical scientific-law promotion;
- educational credentialing or external certification.

---

## 3. Concept graph

```yaml
schema: focusa.quantitative_concept.v1
concept_id:
name:
domain: mathematics | statistics | physics | scientific_reasoning | domain_specific
canonical_definition_ref:
intuition_ref:
formalism_refs: []
prerequisite_refs: []
successor_refs: []
misconception_refs: []
example_refs: []
counterexample_refs: []
visualization_refs: []
practice_template_refs: []
transfer_domain_refs: []
version:
status:
```

Concept graphs MAY include:

```text
arithmetic
algebra
functions
geometry
trigonometry
calculus
linear algebra
differential equations
probability
statistics
causality
optimization
numerical analysis
classical mechanics
energy
thermodynamics
fluids
circuits
waves
control systems
robotics
scientific measurement
experiment design
```

---

## 4. Mastery record

```yaml
schema: focusa.concept_mastery_record.v1
mastery_id:
learner_ref:
concept_ref:
mastery_dimensions:
  recognition:
  intuition:
  symbolic_manipulation:
  derivation:
  computation:
  application:
  explanation:
  transfer:
  error_detection:
  retention:
mastery_estimate_ref:
uncertainty_ref:
evidence_refs: []
misconception_refs: []
last_practice_at_ref:
next_review_at_ref:
recommended_challenge_ref:
status:
receipt_ref:
```

Mastery cannot be inferred from exposure or reading alone.

---

## 5. Explanation ladder

Every quantitative result MAY produce the following layers:

```text
L0 — operational result or decision
L1 — intuition and physical meaning
L2 — variable, unit, and equation map
L3 — assumptions, uncertainty, sensitivity, and validity
L4 — inspectable derivation or computational proof
L5 — interactive graph, simulation, or experiment
L6 — transfer problem in a new context
L7 — counterexample, edge case, or model critique
```

The user or Runtime Constitution may choose default depth. Higher layers remain available through progressive disclosure.

---

## 6. Derivation artifact

```yaml
schema: focusa.derivation_artifact.v1
derivation_id:
problem_ref:
result_ref:
starting_definition_refs: []
assumption_refs: []
steps:
  - step_id:
    expression_before_ref:
    transformation_ref:
    expression_after_ref:
    justification_ref:
    verification_ref:
limiting_case_refs: []
dimension_check_ref:
independent_derivation_refs: []
human_summary_ref:
evidence_refs: []
receipt_ref:
```

A derivation artifact is a public computational explanation, not private chain-of-thought.

---

## 7. Equation map

```yaml
schema: focusa.equation_map.v1
map_id:
problem_ref:
nodes:
  - object_ref:
    role: quantity | variable | parameter | constant | equation | constraint | result
edges:
  - from_ref:
    to_ref:
    relationship:
solution_path_refs: []
sensitivity_path_refs: []
uncertainty_path_refs: []
visualization_ref:
```

Equation maps help learners see how quantities flow into a result.

---

## 8. Misconception model

```yaml
schema: focusa.quantitative_misconception.v1
misconception_id:
concept_ref:
pattern:
likely_cause_refs: []
diagnostic_prompt_refs: []
counterexample_refs: []
corrective_explanation_refs: []
practice_refs: []
severity:
status:
```

Common examples include:

```text
correlation implies causation
more decimal places imply more accuracy
force is required to maintain constant velocity
heavier objects fall faster in vacuum
probability and confidence are the same
average describes every member
statistical significance implies importance
negative result means no effect
simulation equals observation
energy and power are interchangeable
percent and percentage points are interchangeable
```

---

## 9. Learning from real missions

The system SHOULD derive learning opportunities from active Workpoints without hijacking the mission.

Example mappings:

```text
robot actuator sizing
→ torque, energy, power, efficiency, uncertainty

server throughput
→ rates, queueing, nonlinear saturation, distributions

market signal
→ probability, calibration, time series, causality, latency decay

business runway
→ sequences, cash flow, scenario analysis, uncertainty

sailing route
→ vectors, reference frames, fluid forces, navigation uncertainty
```

Learning prompts are optional operator-facing projections and do not block mission completion unless learning itself is the Workpoint target.

---

## 10. Discovery portfolios

### 10.1 Conjecture portfolio

```yaml
schema: focusa.conjecture_portfolio.v1
portfolio_id:
question_ref:
candidate_refs: []
prior_support_refs: []
disconfirming_evidence_refs: []
discriminating_experiment_refs: []
counterexample_search_refs: []
replication_refs: []
current_disposition:
```

### 10.2 Candidate classes

```text
EquationCandidate
InvariantCandidate
ThresholdCandidate
PhaseTransitionCandidate
LagStructureCandidate
InteractionCandidate
MechanismCandidate
ScalingLawCandidate
ResidualExplanationCandidate
```

### 10.3 Discovery status

```text
speculative
candidate
temporarily_supported
out_of_sample_supported
replicated
bounded_applicability
refuted
superseded
```

No status independently creates a scientific law.

---

## 11. Counterexample search

`focusa_math_counterexample_search` MUST attempt to find cases where a proposed rule fails.

It may search:

- boundary and limiting cases;
- alternate regimes;
- excluded populations;
- adversarial parameter values;
- different temporal windows;
- different model providers;
- physical feasibility constraints;
- measurement error scenarios;
- negative-transfer contexts.

A valid counterexample narrows or refutes an overbroad claim and remains visible even when average performance is favorable.

---

## 12. Invariant mining

Candidate invariants MUST pass:

- dimensional consistency;
- scale and coordinate analysis;
- temporal holdout;
- independent cohort validation;
- counterexample search;
- complexity penalty;
- robustness to measurement uncertainty;
- physical plausibility where applicable;
- Spec 144 independent verification.

A stable relationship may still be correlational or context-bound.

---

## 13. Residual intelligence

Residual analysis SHOULD ask what the active model repeatedly fails to explain.

```yaml
schema: focusa.residual_intelligence_result.v1
result_id:
model_ref:
residual_set_ref:
cluster_refs: []
explained_component_refs: []
unexplained_component_refs: []
candidate_missing_variable_refs: []
candidate_regime_refs: []
candidate_mechanism_refs: []
recommended_measurement_refs: []
recommended_experiment_refs: []
```

Residual clusters can propose new ontology primitives, measurements, or model dimensions, but cannot add them canonically without governance.

---

## 14. Experiment discriminator

Given competing hypotheses, `focusa_experiment_discriminator` SHOULD propose the smallest safe experiment whose expected outcomes differ materially across hypotheses.

The result preserves:

- hypotheses;
- predicted outcome under each;
- discriminating variable;
- measurement plan;
- power/precision;
- cost, time, risk, and reversibility;
- action deadline;
- authority and safety requirements.

---

## 15. Practice and transfer

Practice classes:

```text
recognition
calculation
derivation
model_selection
assumption_detection
unit_and_dimension_check
error_diagnosis
experiment_design
simulation_interpretation
transfer_problem
teach_back
```

A transfer challenge changes context while preserving the underlying concept. Successful rote repetition does not prove transfer.

---

## 16. Review and retention

Concept review may use evidence-backed spacing and retrieval practice. The system MUST preserve:

- prior mastery estimate;
- time since practice;
- performance and error type;
- context similarity;
- confidence and uncertainty;
- recommended review;
- operator preferences.

The system MUST NOT turn learning into compulsive notifications or claim neuroscientific precision unsupported by evidence.

---

## 17. Learning and discovery reflexes

```text
detect_relevant_learning_opportunity
detect_misconception_pattern
request_intuitive_explanation
request_equation_map
request_derivation
request_limiting_case
request_transfer_problem
request_counterexample
request_experiment_discriminator
surface_unexplained_residual
quarantine_unverified_discovery
schedule_mastery_review
narrow_overbroad_rule
```

Reflex authority and runtime integration are governed by Spec 152B.

---

## 18. Operation families

```text
learning.concept.inspect
learning.prerequisites.resolve
learning.explain
learning.derivation.generate
learning.equation_map.generate
learning.practice.generate
learning.practice.evaluate
learning.transfer.evaluate
learning.mastery.view
learning.mastery.update
learning.misconception.inspect
learning.review.plan

discovery.conjecture.create
discovery.portfolio.view
discovery.counterexample.search
discovery.invariant.mine
discovery.residual.analyze
discovery.experiment.discriminate
discovery.replication.plan
discovery.claim.verify
```

Pi tools remain bounded under:

```text
focusa_quantitative_explain
focusa_math_discover
focusa_science_experiment
focusa_quantitative_replay
```

---

## 19. Mission Canvas surfaces

```text
Concept Map
Equation Map
Derivation
Interactive Function or Simulation
Assumption and Validity Panel
Counterexample Gallery
Conjecture Portfolio
Experiment Designer
Residual Explorer
Mastery and Review Map
```

Terminal and TUI surfaces provide bounded summaries and links to rich artifacts.

---

## 20. Prediction and metacognitive integration

A resolved prediction can update mastery in:

- calibration understanding;
- base-rate use;
- uncertainty decomposition;
- evidence independence;
- causal reasoning;
- decision thresholds.

A quantitative failure can update mastery in:

- units and dimensions;
- numerical stability;
- assumption detection;
- physical feasibility;
- statistical protocol selection.

Metacognitive learning and concept mastery remain separate records. System strategy improvement is not automatically human mastery, and human mastery is not automatically system policy.

---

## 21. Verification

Spec 144 obligations include:

```text
learning.explanation_fidelity
learning.derivation_validity
learning.example_correctness
learning.counterexample_validity
learning.mastery_evidence
learning.transfer_evidence
discovery.holdout_integrity
discovery.multiple_search_control
discovery.counterexample_search
discovery.replication
discovery.applicability
discovery.causal_and_physical_claim_boundary
```

---

## 22. Privacy and user control

Mastery, misconception, and learning-preference records are private user data.

Required controls:

- explicit learner identity scope;
- export and deletion;
- no advertiser access;
- no public exposure by default;
- no employment or educational scoring use without explicit authorization;
- explanation and learning mode controls;
- quiet hours and notification consent;
- bounded retention.

---

## 23. Required tests

1. explanation formula matches canonical computation;
2. derivation steps are independently verified;
3. explanation does not expose hidden reasoning;
4. concept prerequisites are stable and versioned;
5. reading alone does not increase mastery to verified;
6. misconception requires evidence;
7. transfer test differs from rote problem;
8. learning overlay does not block an unrelated mission;
9. counterexample narrows an overbroad rule;
10. discovery cohort and validation cohort remain separate;
11. temporal leakage invalidates a discovery;
12. physical interpretation requires physical verification;
13. correlation cannot become mechanism;
14. conjecture cannot auto-promote to law;
15. mastery state survives compaction and replay;
16. deletion removes private mastery projections according to policy.

---

## 24. Acceptance criteria

Spec 152C is accepted only when:

1. quantitative concepts and prerequisites are first-class;
2. mastery is multidimensional, uncertain, and Evidence-linked;
3. explanations support progressive depth;
4. derivations and equation maps are inspectable and verified;
5. real missions can generate optional relevant learning;
6. misconceptions are diagnosed and corrected through evidence;
7. transfer and retention are measured separately from exposure;
8. conjectures, counterexamples, invariants, residuals, experiments, and replication are governed;
9. discovery cannot bypass Spec 138 learning or Spec 144 verification;
10. privacy and learner control are complete;
11. Pi, API, CLI, MCP, Mission Canvas, TUI, replay, Evidence, and Receipt parity is proven.

---

## 25. Canonical summary

```text
Focusa should help solve the mission and teach the mathematics inside it.
It should also help discover new structure—but every discovery remains a
traceable, challengeable conjecture until independent evidence earns more.
```
