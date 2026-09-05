# Spec 153A — Physical Verification, Experimental Settlement, and High-Consequence Scientific Control Addendum

**Status:** NORMATIVE ADDENDUM — MANDATORY FOR CONSEQUENTIAL PHYSICAL AND EXPERIMENTAL CLAIMS — IMPLEMENTATION NOT IMPLIED  
**Parent:** Spec 153  
**Owner:** Focusa Core / Physical Verification / Experimental Settlement  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Depends on:** Spec 137B, Spec 138B, Specs 152/152A/152B, Spec 140/140A, Spec 144, Evidence, Receipts, authority and permission primitives

---

## 0. Constitutional directive

```text
A SIMULATION PASS IS NOT AN EXPERIMENTAL PASS.
A SENSOR VALUE IS NOT AUTHORITATIVE WITHOUT CALIBRATION AND LINEAGE.
A PHYSICAL CLAIM IS NOT SETTLED WHILE REQUIRED MEASUREMENT, SAFETY,
REPLICATION, OR INDEPENDENT VERIFICATION REMAINS OPEN.

High-consequence scientific and engineering work fails closed on unknown
physical state, stale calibration, uncovered verification, or unresolved effect.
```

---

## 1. Purpose

This addendum governs how Focusa verifies and settles physical and experimental claims. It prevents:

- numerical convergence from masquerading as physical correctness;
- simulated results from being treated as measurements;
- stale or uncalibrated sensors from authorizing action;
- one successful run from becoming a universal law;
- physical feasibility claims without safety margins;
- experiments without frozen hypotheses, methods, or measurement plans;
- selective reporting and post-outcome rule changes;
- robotics or engineering actions from bypassing deterministic safety controls;
- market-adjacent latency and hardware claims from relying on nominal specifications rather than deployed-path measurement.

---

## 2. Ownership

### 2.1 This addendum owns

- physical verification portfolios;
- measurement authority and calibration admission;
- experimental outcome authority;
- simulation validation and verification distinction;
- replication and external-authority requirements;
- high-consequence physical assurance tiers;
- actuation and experiment safety interlocks;
- physical claim settlement and correction;
- simulation-to-reality settlement;
- scientific learning promotion prerequisites.

### 2.2 This addendum does not own

- model definition;
- statistical methods;
- time synchronization;
- generic verifier routing;
- permission or capability assignment;
- domain-specific regulatory certification;
- actual operation of external laboratories, sensors, robots, brokers, or facilities.

---

## 3. Claim classes and proof requirements

```text
physical_description
physical_measurement_claim
physical_state_estimate
physical_feasibility_claim
simulation_result_claim
simulation_validity_claim
experimental_outcome_claim
physical_mechanism_claim
engineering_safety_claim
scientific_law_candidate
```

Each class activates a distinct verification portfolio. A weaker claim class cannot be relabeled as a stronger one without additional proof.

---

## 4. Measurement authority

```yaml
schema: focusa.measurement_authority_profile.v1
authority_profile_id:
measurement_domain_ref:
authorized_sensor_classes: []
authorized_calibration_authorities: []
required_traceability_refs: []
required_capture_point_refs: []
required_temporal_precision_profile_ref:
maximum_calibration_age_ref:
maximum_measurement_uncertainty_ref:
required_environment_condition_refs: []
required_redundancy:
required_independent_measurement_count:
required_quality_flags: []
on_disagreement:
on_calibration_expired:
on_temporal_authority_unavailable:
version:
status:
```

A measurement is authoritative only if its sensor, calibration, capture, time, frame, uncertainty, and quality satisfy the active profile.

---

## 5. Calibration record

```yaml
schema: focusa.sensor_calibration.v1
calibration_id:
sensor_ref:
measurement_model_ref:
calibration_authority_ref:
reference_standard_refs: []
reference_standard_traceability_refs: []
method_ref:
condition_refs: []
calibration_points: []
bias_model_ref:
linearity_assessment_ref:
hysteresis_assessment_ref:
repeatability_ref:
reproducibility_ref:
resolution_ref:
accuracy_ref:
uncertainty_budget_ref:
performed_at_stamp_ref:
valid_from_stamp_ref:
expires_at_stamp_ref:
drift_review_policy_ref:
evidence_refs: []
receipt_ref:
```

A calibration is condition- and range-specific unless proven otherwise.

---

## 6. Physical verification portfolio

```yaml
schema: focusa.physical_verification_portfolio.v1
portfolio_id:
claim_ref:
physical_system_ref:
model_ref:
verification_snapshot_ref:
obligations:
  - obligation_ref:
    provider_class:
    verifier_ref:
    independence_requirement_ref:
    status:
required_checks:
  - dimensional
  - law_applicability
  - initial_and_boundary_conditions
  - conservation
  - physical_feasibility
  - numerical_convergence
  - numerical_stability
  - measurement_traceability
  - sensor_calibration
  - uncertainty
  - safety_margin
  - simulation_validation
  - experiment_or_replication
uncovered_obligation_refs: []
assurance_tier:
status:
receipt_ref:
```

Mandatory obligations cannot be removed because a verifier or instrument is unavailable.

---

## 7. Simulation verification versus validation

Focusa MUST distinguish:

```text
simulation verification
  Did the equations and numerical method execute correctly?

simulation validation
  Does the model adequately represent the relevant physical system?
```

Verification may include:

- code and solver checks;
- analytical fixtures;
- mesh or time-step convergence;
- conservation residuals;
- independent implementation comparison;
- precision and stability checks.

Validation may include:

- comparison to measurements;
- comparison to controlled experiments;
- parameter calibration on separated data;
- validation against held-out conditions;
- uncertainty-aware residual analysis;
- validity-envelope establishment.

A verified simulation can remain physically invalid.

---

## 8. Verification snapshot

```yaml
schema: focusa.physical_verification_snapshot.v1
snapshot_id:
physical_system_ref:
model_ref:
model_hash:
quantity_definition_refs: []
constant_and_property_refs: []
initial_condition_refs: []
boundary_condition_refs: []
parameter_snapshot_ref:
measurement_snapshot_ref:
calibration_snapshot_ref:
temporal_snapshot_ref:
solver_and_simulation_refs: []
statistical_protocol_ref:
safety_policy_refs: []
excluded_or_unavailable_refs: []
content_hashes: {}
frozen_at_stamp_ref:
immutable:
status:
```

Any material model, parameter, condition, measurement, calibration, solver, temporal, or safety change invalidates affected verification.

---

## 9. Experiment preregistration and execution

Consequential experiments MUST freeze before execution:

- question and hypotheses;
- predicted outcomes by hypothesis;
- variables and controls;
- apparatus and sensors;
- calibration requirements;
- procedure;
- safety boundaries;
- sample size or precision target;
- stopping rule;
- primary and secondary outcomes;
- statistical analysis;
- exclusion and failure policy;
- temporal schedule;
- settlement authority.

Execution records preserve every deviation. Unapproved material deviation changes confirmatory results to exploratory unless a governing correction path applies.

---

## 10. Experimental observation

```yaml
schema: focusa.experimental_observation.v1
observation_id:
experiment_ref:
run_ref:
physical_system_ref:
condition_snapshot_ref:
measurement_refs: []
procedure_deviation_refs: []
safety_event_refs: []
event_window_ref:
operator_or_automation_ref:
quality_status:
evidence_refs: []
receipt_ref:
```

An observation cannot be silently excluded. Exclusions follow the frozen policy or are separately disclosed.

---

## 11. Experimental outcome authority

```yaml
schema: focusa.experimental_outcome_authority_event.v1
event_id:
experiment_ref:
action: claim | dispute | escalate | resolve | correct | void | censor
authority_class:
authority_ref:
settlement_policy_ref:
settlement_policy_version:
policy_locked_at_stamp_ref:
experiment_started_at_stamp_ref:
outcome_occurred_at_stamp_ref:
outcome_observed_at_stamp_ref:
resolved_at_stamp_ref:
resolved_outcome_ref:
supersedes_event_ref:
statistical_inference_ref:
physical_verification_ref:
evidence_refs: []
receipt_ref:
```

Corrections append and supersede. Settled experimental outcomes feed prediction scoring and learning only through explicit references.

---

## 12. Replication

Replication classes:

```text
technical_repeat
independent_repeat
parameter_replication
condition_replication
cross-site_replication
cross-instrument_replication
cross-model_replication
conceptual_replication
```

A replication record preserves differences from the original. Exact duplication is not required for conceptual replication, but claims must state what was and was not replicated.

---

## 13. Physical mechanism verification

A mechanism claim requires more than predictive fit. The verification portfolio SHOULD consider:

- temporally ordered cause and effect;
- intervention or natural experiment;
- competing mechanism predictions;
- discriminating measurements;
- mediation or state-path evidence;
- physical-law compatibility;
- conservation and feasibility;
- alternate explanations;
- parameter and regime sensitivity;
- replication.

A mechanism remains candidate or unsupported when these requirements are inadequate.

---

## 14. Safety margin

```yaml
schema: focusa.physical_safety_margin.v1
margin_id:
requirement_ref:
capacity_quantity_ref:
demand_quantity_ref:
nominal_margin_ref:
uncertainty_adjusted_margin_ref:
worst_case_margin_ref:
load_combination_policy_ref:
factor_of_safety_policy_ref:
validity_envelope_ref:
result: sufficient | insufficient | indeterminate
```

Nominal feasibility with insufficient uncertainty-adjusted margin cannot satisfy an engineering safety claim.

---

## 15. High-consequence assurance tiers

```text
P0 — deterministic arithmetic and dimensional checks
P1 — verified simulation for harmless reversible analysis
P2 — independent model and measurement review for meaningful engineering claims
P3 — multi-method verification, calibrated measurements, safety margin, and replication for high-consequence systems
P4 — operator, licensed specialist, laboratory, regulator, or external authority where required
```

Cost, delay, or provider unavailability cannot silently downgrade the required tier.

---

## 16. Robotics and actuation admission

Before consequential actuation, the runtime MUST verify:

- exact physical-system and actuator identity;
- current sensor and calibration state;
- current reference frames and transforms;
- authorized target and control limits;
- deterministic control guard;
- energy, thermal, structural, and motion limits;
- watchdog and kill-switch state;
- communication and command freshness;
- operator and permission authority;
- reconciliation policy.

Unknown state, stale sensing, expired calibration, or unresolved prior effect blocks actuation.

---

## 17. High-frequency market and hardware timing claims

Claims about microsecond or nanosecond market execution, hardware clocks, feed latency, or decision latency require:

- Spec 137B precision profile;
- stable capture points;
- deployed-path calibration;
- exact transport;
- distributional latency rather than one average;
- hardware, OS, kernel, NIC, provider, venue, and version lineage;
- clock uncertainty;
- causal sequence and reconciliation;
- independent verification.

Nominal hardware resolution or synthetic benchmark alone cannot satisfy a deployed-path claim.

---

## 18. Simulation-reality settlement

A simulation-reality comparison MUST match:

- physical quantity and unit;
- reference frame;
- spatial and temporal scope;
- initial and boundary conditions;
- environment and regime;
- measurement model;
- calibration state;
- uncertainty;
- model version;
- solver version.

Unmatched comparisons are descriptive and cannot validate the model.

---

## 19. Physical finding

```yaml
schema: focusa.physical_verification_finding.v1
finding_id:
claim_ref:
obligation_ref:
finding_type:
severity:
confidence:
uncertainty_ref:
physical_target_refs: []
measurement_refs: []
simulation_refs: []
summary:
reasoning_summary:
reproduction_or_inspection_refs: []
impact_refs: []
requested_action_ref:
settlement_blocking:
fresh_until_ref:
evidence_refs: []
status:
```

A verifier is not presumed correct. Findings require scope, Evidence, eligibility, and reproduction validation.

---

## 20. Scientific-learning firewall

Physical or experimental learning cannot be promoted unless the packet includes:

```text
model and validity identity
measurement authority
calibration and temporal integrity
experiment protocol
outcome settlement
statistical validity
physical verification
simulation validation where used
countermodel and alternate mechanism review
replication or explicit limited-evidence status
applicability and exclusions
failure and rollback conditions
independent promotion verifier
```

One successful prototype run cannot become a universal design law.

---

## 21. Operation families

```text
measurement.authority.evaluate
measurement.calibration.record
measurement.capture
measurement.correct
measurement.disagreement.resolve
physics.verification.plan
physics.verification.snapshot.freeze
physics.verification.run
physics.verification.findings
physics.feasibility.settle
physics.safety_margin.evaluate
simulation.verify
simulation.validate
experiment.preregister
experiment.start
experiment.observe
experiment.deviation.record
experiment.resolve
experiment.correct
experiment.replicate
simulation_reality.reconcile
physical_learning.packet.build
physical_learning.promotion.verify
```

---

## 22. Spec 144 interlock

Spec 144 routes the physical verification portfolio across:

```text
DeterministicValidator
FormalSemanticValidator
NumericalVerifier
SimulationVerifier
MeasurementAuditor
CalibrationAuthority
PhysicalDomainVerifier
SafetyVerifier
ExperimentMethodologyVerifier
StatisticalVerifier
ExternalLaboratoryOrAuthority
OperatorReviewer
```

No majority vote overrides a valid safety veto, calibration failure, conservation failure, or uncovered mandatory obligation.

---

## 23. Degraded and unavailable behavior

If an instrument, calibration, verifier, clock source, simulation provider, laboratory, or specialist is unavailable:

- the missing obligation remains visible;
- result becomes blocked, indeterminate, degraded, or experimental;
- no physical safety or mechanism claim is upgraded;
- nonconsequential drafting or simulation may continue;
- actuation and settlement fail closed where required;
- recovery and alternate authorized providers are explicit.

---

## 24. Security and privacy

- experiment and sensor data use data-class-aware access;
- biometric, location, household, and environmental data receive applicable privacy controls;
- physical-system control surfaces require least privilege;
- calibration certificates and external authority records are tamper-evident;
- untrusted simulation/model content is quarantined;
- generated experiment procedures cannot bypass safety policies;
- hazardous or regulated operations require appropriate external authority.

---

## 25. Required tests

1. stale calibration blocks authoritative measurement;
2. untraceable reference standard blocks calibration claim;
3. simulation verification and validation remain separate;
4. converged but physically invalid simulation fails;
5. experiment cannot start without required frozen protocol;
6. material procedure deviation is recorded and changes claim class;
7. selective observation exclusion is detected;
8. unresolved experiment cannot score prediction;
9. correction supersedes without erasing history;
10. replication differences remain explicit;
11. mechanism fit without discriminating evidence stays candidate;
12. insufficient safety margin blocks safety claim;
13. unknown robot state blocks actuation;
14. kill-switch or watchdog failure blocks actuation;
15. nominal timestamp resolution cannot satisfy deployed-path accuracy;
16. unmatched simulation/measurement comparison cannot validate model;
17. physical verifier unsupported finding reaches governed disposition;
18. required external authority cannot be replaced by an LLM;
19. physical learning cannot promote without complete packet;
20. full replay reconstructs measurement, calibration, experiment, verification, and settlement.

---

## 26. Acceptance criteria

Spec 153A is accepted only when:

1. measurement authority and calibration are explicit and enforceable;
2. physical verification portfolios cover every activated obligation;
3. simulation verification and physical validation remain distinct;
4. verification snapshots are immutable and invalidate on material change;
5. experiments preregister hypotheses, procedure, measurements, safety, statistics, and settlement;
6. observations, deviations, exclusions, and corrections are append-only;
7. experimental outcomes resolve through separate authority;
8. replication and mechanism claims have explicit proof classes;
9. safety margins and high-consequence assurance tiers are enforced;
10. robotics and actuation use deterministic safety admission;
11. market/hardware timing claims use Spec 137B deployed-path proof;
12. simulation-reality comparisons are matched and uncertainty-aware;
13. scientific learning uses a complete promotion firewall;
14. Spec 144 routing, tools, reducer, persistence, replay, clients, Evidence, and Receipts have parity.

---

## 27. Canonical summary

```text
Spec 153 defines physical models and experiments.
Spec 153A decides when their measurements, simulations, feasibility claims,
mechanisms, safety conclusions, and scientific learning are actually proven.
```
