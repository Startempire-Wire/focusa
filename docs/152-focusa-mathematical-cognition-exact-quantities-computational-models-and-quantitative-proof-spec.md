# Spec 152 — Focusa Mathematical Cognition, Exact Quantities, Computational Models, and Quantitative Proof

**Status:** NORMATIVE DRAFT — PRIMITIVE-OWNING — IMPLEMENTATION NOT IMPLIED  
**Owner:** Focusa Core / Quantitative Cognition  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Depends on:** Project identity and scope, Evidence, Receipts, Reference Store, Spec 137 family, Spec 140 family, Spec 141 Operation Registry, Spec 144 semantic integrity  
**Primary consumers:** Spec 138B, Spec 152A/B/C, Spec 153/153A, all quantitative Verticals

---

## 0. Constitutional directive

```text
FORMALIZE BEFORE SOLVING.
ATTACH MEANING TO EVERY NUMBER.
PRESERVE EXACTNESS WHERE IT MATTERS.
DECLARE APPROXIMATION WHERE IT DOES NOT.
VERIFY DIMENSIONS, DOMAINS, CONSTRAINTS, AND NUMERICS.
RETURN A COMPUTATIONAL PROOF, NOT AN UNTRACEABLE ANSWER.
```

A calculator returns a result. Focusa MUST govern the entire quantitative claim: problem formulation, quantities, units, dimensions, assumptions, equations, constraints, method, solver, tolerance, computation, verification, interpretation, decision threshold, Evidence, and replay.

---

## 1. Purpose

This specification establishes a domain-general mathematical cognition substrate for agents and humans. It enables Focusa to:

- recognize quantitative structure in ordinary missions;
- represent exact and approximate quantities safely;
- formulate mathematical problems;
- select and govern mathematical methods;
- invoke deterministic computation providers;
- verify dimensional, symbolic, numerical, and constraint validity;
- expose assumptions, sensitivity, and decision boundaries;
- support discovery without converting conjectures into truth;
- project useful mathematics through every Focusa surface;
- feed prediction, statistics, physics, metacognition, and human learning.

---

## 2. Ownership

### 2.1 Focusa Core owns

- numeric representation contracts;
- exactness and approximation semantics;
- quantities, units, dimensions, scales, and coordinate systems;
- variables, parameters, constants, expressions, equations, inequalities, and functions;
- vectors, matrices, tensors, sets, graphs, and symbolic objects;
- mathematical problem contracts;
- assumptions, constraints, objectives, and feasibility;
- solver and computation-run contracts;
- tolerance, convergence, conditioning, stability, and error contracts;
- symbolic derivation and quantitative proof artifacts;
- sensitivity, threshold, frontier, and failure-envelope semantics;
- countermodels, counterexamples, conjectures, and validity envelopes;
- provider-neutral deterministic computation authority.

### 2.2 Verticals own

- domain variables and specialized equations;
- domain constants and their authority;
- domain-specific utility and risk policies;
- domain model applicability;
- interpretation of results;
- external data sources;
- specialized solver adapters and simulations.

### 2.3 External computation providers own

- implementation of symbolic algebra;
- high-performance linear algebra;
- numerical integration;
- optimization algorithms;
- constraint and satisfiability solving;
- formal proof execution;
- arbitrary-precision arithmetic;
- large simulation workloads.

Focusa owns admission, contracts, provenance, bounds, verification, Evidence, Receipts, and learning authority.

---

## 3. Numeric representation

```rust
pub enum NumericRepresentation {
    SignedInteger,
    UnsignedInteger,
    Rational,
    FixedDecimal,
    ArbitraryPrecisionDecimal,
    BinaryFloatingPoint,
    Complex,
    Interval,
    Symbolic,
    DistributionRef,
    VectorRef,
    MatrixRef,
    TensorRef,
}
```

Canonical values MUST declare representation and exactness.

```yaml
schema: focusa.quantitative_value.v1
quantity_value_id:
representation:
canonical_value:
unit_ref:
dimension_ref:
scale_ref:
exactness: exact | measured | estimated | approximate | simulated | inferred | predicted
uncertainty_ref:
significant_digits:
precision_policy_ref:
rounding_policy_ref:
valid_domain_ref:
source_ref:
temporal_stamp_ref:
evidence_refs: []
receipt_ref:
```

### 3.1 Required representation laws

- money and accounting values use fixed decimal or an authorized exact representation;
- authoritative high-resolution time uses integer nanoseconds according to Spec 137B;
- probabilities use validated bounded representations;
- exact algebra preserves rationals and symbolic forms where required;
- floating-point results retain implementation, precision, rounding, and error posture;
- values crossing JavaScript surfaces preserve exact integers as strings or structured seconds/nanoseconds;
- NaN and infinities cannot silently enter canonical state;
- overflow, underflow, catastrophic cancellation, and precision loss are detected or bounded.

---

## 4. Quantity, unit, and dimension system

```yaml
schema: focusa.quantity_definition.v1
quantity_id:
name:
symbol:
dimension_ref:
canonical_unit_ref:
allowed_unit_refs: []
scale_type: ratio | interval | ordinal | nominal | logarithmic | count
value_domain_ref:
conversion_policy_ref:
semantic_subject_refs: []
registry_version:
status:
```

Base dimensions include SI dimensions plus governed nonphysical dimensions:

```text
length
mass
time
temperature
electric_current
amount_of_substance
luminous_intensity
angle
count
currency
information
probability
custom_registered
```

Derived dimensions are symbolic compositions.

### 4.1 Unit conversion

Every conversion MUST preserve:

- source and destination units;
- conversion formula and version;
- exactness and rounding;
- timestamp and rate source for temporally varying conversions;
- affine versus multiplicative semantics;
- uncertainty propagation;
- Evidence and Receipt.

Currency conversion is a temporally sourced quantitative operation, not a timeless unit conversion.

### 4.2 Dimensional guard

The deterministic dimensional guard MUST detect:

```text
addition of incompatible dimensions
comparison of noncomparable scales
incorrect exponent dimensions
missing conversion
ratio/interval temperature confusion
rate versus total confusion
percentage versus percentage-point confusion
milliseconds versus microseconds
energy versus power
nominal currency across dates without authority
```

A dimensional failure blocks canonical computation.

---

## 5. Mathematical objects

Core ontology and types MUST support:

```text
Scalar
Variable
Parameter
Constant
Expression
Equation
Inequality
Function
Sequence
Series
Set
Relation
Vector
Matrix
Tensor
Polynomial
DifferentialEquation
IntegralEquation
Graph
Network
ObjectiveFunction
Constraint
FeasibleRegion
OptimizationProblem
RootFindingProblem
InverseProblem
BoundaryValueProblem
InitialValueProblem
StochasticProcessReference
```

Every object binds identity, version, scope, provenance, domain, assumptions, and Evidence.

---

## 6. Quantitative problem contract

```yaml
schema: focusa.quantitative_problem.v1
problem_id:
problem_class:
scope_ref:
operator_question_ref:
subject_refs: []

targets:
  target_quantity_refs: []
  target_function_refs: []
  target_decision_refs: []

inputs:
  known_quantity_refs: []
  unknown_quantity_refs: []
  parameter_refs: []
  constant_refs: []

model:
  expression_refs: []
  equation_refs: []
  constraint_refs: []
  objective_refs: []
  assumption_refs: []
  validity_envelope_ref:

solution:
  permitted_method_classes: []
  solver_policy_ref:
  tolerance_policy_ref:
  resource_policy_ref:
  reproducibility_policy_ref:

interpretation:
  decision_threshold_refs: []
  utility_policy_ref:
  explanation_level_policy_ref:

runtime_constitution_ref:
temporal_snapshot_ref:
semantic_snapshot_ref:
problem_hash:
status:
receipt_ref:
```

Problem status includes:

```text
draft
well_posed
underdetermined
overdetermined
inconsistent
unbounded
infeasible
ill_conditioned
outside_validity
ready_for_computation
superseded
```

---

## 7. Problem framing

`focusa_quantitative_compile` MUST be able to transform a natural-language mission into a candidate problem contract while clearly distinguishing:

- operator-provided facts;
- retrieved observations;
- inferred quantities;
- assumptions;
- unknowns;
- decision variables;
- objectives;
- constraints;
- required measurements;
- required statistical or physical models;
- unresolved ambiguity.

The compiled problem remains a proposal until reducer admission.

---

## 8. Assumptions and validity

```yaml
schema: focusa.mathematical_assumption.v1
assumption_id:
statement:
assumption_class:
source_ref:
applicable_model_refs: []
activation_condition_ref:
violation_condition_ref:
test_refs: []
sensitivity_ref:
status: proposed | active | violated | unsupported | superseded
```

Common classes:

```text
linearity
independence
stationarity
continuity
differentiability
convexity
normality
homogeneity
isotropy
constant_parameter
closed_system
perfect_information
zero_transaction_cost
unlimited_capacity
```

Every model MUST declare a `ValidityEnvelope`:

```yaml
schema: focusa.model_validity_envelope.v1
envelope_id:
model_ref:
valid_parameter_regions: []
valid_state_regions: []
valid_scale_regions: []
valid_temporal_regions: []
required_assumption_refs: []
known_failure_regions: []
extrapolation_policy_ref:
review_trigger_refs: []
evidence_refs: []
receipt_ref:
```

---

## 9. Deterministic computation provider registry

```yaml
schema: focusa.computation_provider_profile.v1
provider_id:
provider_class:
implementation_ref:
version:
supported_problem_classes: []
supported_numeric_representations: []
supported_precision_profiles: []
resource_limits_ref:
security_profile_ref:
determinism_profile_ref:
reproducibility_profile_ref:
conformance_fixture_refs: []
known_limitations: []
status:
```

Provider classes:

```text
SymbolicAlgebraProvider
ArithmeticProvider
ArbitraryPrecisionProvider
IntervalArithmeticProvider
LinearAlgebraProvider
NumericalSolverProvider
DifferentialEquationProvider
OptimizationProvider
ConstraintSolverProvider
SMTSolverProvider
FormalProofProvider
GraphAlgorithmProvider
SimulationProvider
VisualizationProvider
```

A provider declaration without executable conformance proof is `schema_only` and ineligible for canonical computation.

---

## 10. Computation plan and run

```yaml
schema: focusa.computation_plan.v1
plan_id:
problem_ref:
method_ref:
provider_ref:
input_snapshot_ref:
precision_profile_ref:
tolerance_policy_ref:
convergence_policy_ref:
random_seed_policy_ref:
resource_budget_ref:
expected_artifact_refs: []
verification_obligation_refs: []
status:
```

```yaml
schema: focusa.computation_run.v1
run_id:
plan_ref:
provider_ref:
provider_version:
started_at_stamp_ref:
completed_at_stamp_ref:
input_hash:
output_hash:
random_seed_ref:
iteration_count:
convergence_status:
residual_ref:
error_budget_ref:
condition_assessment_ref:
result_refs: []
intermediate_artifact_refs: []
warning_refs: []
failure_ref:
evidence_refs: []
receipt_ref:
```

Canonical results require immutable input snapshots and reproducible configuration, or an explicit justified non-reproducibility classification.

---

## 11. Numerical validity

Required concepts:

```text
absolute_error
relative_error
truncation_error
rounding_error
measurement_error
residual
condition_number
stability
convergence
convergence_rate
tolerance
step_size
discretization
precision
significant_digits
```

Required failure classes:

```text
nonconvergence
divergence
oscillation
ill_conditioned
unstable_method
precision_exhausted
overflow
underflow
catastrophic_cancellation
constraint_violation
invalid_domain
singular_system
multiple_solutions_unresolved
local_optimum_only
resource_exhausted
```

A process exit code or returned number is not proof of valid computation.

---

## 12. Quantitative proof portfolio

A quantitative result may require several proof modes:

```text
dimensional proof
symbolic equivalence
independent arithmetic
interval enclosure
limiting-case test
order-of-magnitude test
numerical residual
constraint satisfaction
alternative solver comparison
simulation comparison
formal proof
external reference fixture
operator or specialist review
```

```yaml
schema: focusa.quantitative_proof_packet.v1
proof_packet_id:
problem_ref:
run_ref:
result_refs: []
proof_assignments: []
verified_assumption_refs: []
violated_assumption_refs: []
validity_envelope_ref:
uncertainty_ref:
countermodel_refs: []
counterexample_refs: []
verdict:
evidence_refs: []
receipt_ref:
```

---

## 13. High-value mathematical cognition operations

### 13.1 Decision frontier

`focusa_math_decision_frontier` computes the exact or bounded region where the recommended action changes.

### 13.2 Unknowns ranking

`focusa_math_unknowns_rank` combines sensitivity, uncertainty, value of information, acquisition cost, time, reversibility, and action deadline to rank the next measurement or research action.

### 13.3 Assumption stress

`focusa_math_assumption_stress` generates alternative authorized model variants and identifies which assumptions dominate the result.

### 13.4 Failure envelope

`focusa_math_failure_envelope` computes the parameter/state region where a plan becomes unsafe, infeasible, insolvent, unstable, or negative utility.

### 13.5 Model tournament

`focusa_math_model_tournament` compares eligible models against one frozen information set without erasing disagreement.

### 13.6 Countermodel

`focusa_math_countermodel` constructs the strongest quantitatively plausible model reaching an alternative conclusion.

### 13.7 Reality gap

`focusa_math_reality_gap` decomposes expected-versus-observed error into input, assumption, model, execution, environment, temporal, and residual contributions.

### 13.8 Ablation

`focusa_math_strategy_ablate` removes components and measures their marginal contribution under frozen evaluation policy.

### 13.9 Reversibility and option value

`focusa_math_decision_reversibility` compares act, wait, test, hedge, and preserve-optionality strategies.

---

## 14. Discovery primitives

```text
StructureDiscovery
EquationCandidate
InvariantCandidate
CounterexampleSearch
ResidualExplanation
ConjecturePortfolio
SymbolicRegressionCandidate
PhaseTransitionCandidate
ThresholdCandidate
InteractionCandidate
LagStructureCandidate
```

Discovery laws:

1. candidate equations must pass dimensional consistency;
2. discovery and validation cohorts remain separate;
3. time-ordered domains use temporal holdouts;
4. multiple-search correction and complexity penalties are recorded;
5. counterexamples and failure regions are actively searched;
6. out-of-sample performance and replication are required before promotion;
7. a discovered relation remains a conjecture until governed settlement;
8. causal and physical interpretations require separate evidence.

---

## 15. Vertical composites

Core operations compose into vertical tools rather than duplicating mathematics.

```text
focusa_finance_runway_intelligence
focusa_finance_goal_path
focusa_finance_debt_liquidity_optimizer
focusa_market_edge_decompose
focusa_market_execution_frontier
focusa_market_risk_of_ruin
focusa_engineering_change_risk
focusa_engineering_bottleneck
focusa_engineering_performance_frontier
focusa_operations_completion_probability
focusa_operations_capacity_breakpoint
focusa_property_total_position
focusa_compliance_control_value
```

Vertical results MUST preserve core quantitative contracts and state their domain policy and applicability.

---

## 16. Human-visible output

Every result supports progressive explanation:

```text
L0 decision/result
L1 formula and variable map
L2 assumptions, uncertainty, and sensitivity
L3 derivation and computation proof
L4 visualization or simulation
L5 transfer challenge and mastery exercise
```

Human explanation artifacts are inspectable computational summaries, not hidden chain-of-thought.

---

## 17. Shared result envelope

```yaml
schema: focusa.quantitative_result.v1
problem_ref:
run_ref:
method_ref:
method_version:
result_refs: []
unit_refs: []
dimension_refs: []
uncertainty_ref:
validity_envelope_ref:
assumption_refs: []
constraint_status_ref:
sensitivity_ref:
decision_frontier_ref:
verification_ref:
runtime_constitution_ref:
temporal_snapshot_ref:
semantic_snapshot_ref:
reproducibility_ref:
human_explanation_ref:
recommended_next_action_ref:
evidence_refs: []
receipt_ref:
```

---

## 18. Operation families

```text
quantitative.problem.compile
quantitative.problem.validate
quantitative.quantity.register
quantitative.quantity.observe
quantitative.unit.convert
quantitative.dimension.check
quantitative.model.register
quantitative.assumption.register
quantitative.validity.evaluate
quantitative.computation.plan
quantitative.computation.run
quantitative.computation.verify
quantitative.sensitivity.evaluate
quantitative.decision_frontier.compute
quantitative.unknowns.rank
quantitative.assumption.stress
quantitative.failure_envelope.compute
quantitative.countermodel.generate
quantitative.ablation.run
quantitative.reality_gap.evaluate
quantitative.discovery.run
quantitative.counterexample.search
quantitative.replay
quantitative.explain
```

---

## 19. Security and safety

- expression inputs are parsed through bounded grammars;
- no arbitrary shell or code execution from untrusted formulas;
- solver time, memory, recursion, graph, and iteration limits are explicit;
- external providers are allowlisted and sandboxed according to policy;
- secrets and private data remain behind access-controlled handles;
- denial-of-service through symbolic expansion or pathological solving is bounded;
- high-consequence results require higher assurance;
- unsafe physical or financial actions remain subject to domain authority and approval.

---

## 20. Persistence and replay

Canonical state stores bounded definitions, contracts, decisions, and results. Large arrays, matrices, plots, solver traces, notebooks, and simulations use the Reference Store.

Replay MUST reconstruct:

- problem versions;
- quantity and unit definitions;
- assumptions and constraints;
- plans and provider identities;
- input snapshots;
- computation runs;
- results and proof packets;
- corrections and supersession;
- decision thresholds;
- learning and discovery disposition.

---

## 21. Required tests

1. exact rational preserved through serialization;
2. fixed decimal money has no binary-float drift;
3. large integer transport survives JavaScript surfaces;
4. incompatible dimensions block;
5. affine temperature conversion is correct and versioned;
6. percent and percentage-point confusion is detected;
7. unknown values are not fabricated;
8. underdetermined and inconsistent systems are classified;
9. solver nonconvergence cannot produce a pass;
10. ill-conditioning is surfaced;
11. limiting-case failure invalidates a model;
12. order-of-magnitude contradiction blocks result promotion;
13. alternative solver disagreement remains visible;
14. constraint violations block feasibility;
15. countermodel preserves alternative conclusion and assumptions;
16. decision frontier identifies recommendation-changing boundaries;
17. discovery candidate fails when holdout performance disappears;
18. conjecture cannot become canonical law automatically;
19. every client preserves identical contracts and values;
20. replay reproduces the admitted computation state.

---

## 22. Acceptance criteria

Spec 152 is accepted only when:

1. exact and approximate numeric representations are explicit;
2. quantities bind units, dimensions, provenance, time, and uncertainty where applicable;
3. mathematical problems can be formalized before solving;
4. assumptions, constraints, objectives, and validity envelopes are first-class;
5. deterministic providers are registered and conformance-tested;
6. computations bind immutable inputs, method, provider, tolerance, and reproducibility;
7. numerical errors, convergence, conditioning, and residuals are governed;
8. dimensional, symbolic, numeric, constraint, and alternative-method proof compose;
9. decision frontiers, unknown ranking, assumption stress, failure envelopes, countermodels, and reality-gap analysis are operational;
10. discovery remains conjectural until independent validation;
11. API, CLI, Pi, MCP, generated clients, Mission Canvas, Evidence, Receipts, migration, and replay have parity;
12. no vertical duplicates or weakens the shared mathematical substrate.

---

## 23. Canonical summary

```text
Focusa does not merely calculate.
It formalizes the problem, preserves quantity meaning, selects a governed method,
executes deterministic computation, verifies the result, exposes the decision
boundary, and learns only from resolved evidence.
```
