# Spec 153 — Physical World Modeling, Measurement, Simulation, and Scientific Reasoning

**Status:** NORMATIVE DRAFT — PRIMITIVE-OWNING — IMPLEMENTATION NOT IMPLIED  
**Owner:** Focusa Core / Physical and Scientific Cognition  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Depends on:** Spec 137 family, Spec 152/152A/152B, Spec 140 family, Spec 144  
**Primary consumers:** robotics, embedded systems, energy, sailing, structures, fluids, circuits, physical simulations, scientific research, high-consequence engineering

---

## 0. Constitutional directive

```text
A PHYSICAL MODEL IS NOT THE PHYSICAL WORLD.
A SENSOR READING IS NOT A MEASUREMENT UNTIL ITS AUTHORITY IS KNOWN.
A SIMULATION IS NOT AN OBSERVATION.
A NUMERICALLY CONVERGED SOLUTION MAY STILL BE PHYSICALLY WRONG.

Declare the system, frame, state, laws, conditions, approximations,
measurements, uncertainty, validity envelope, and verification before
claiming physical feasibility or mechanism.
```

---

## 1. Purpose

This specification gives Focusa a domain-general physical and scientific cognition substrate so agents can:

- formulate physical systems from real missions;
- reason with reference frames, state variables, fields, forces, energy, and constraints;
- distinguish measurements from estimates and simulations;
- select applicable physical laws and approximations;
- create bounded simulations and digital twins;
- design experiments;
- test feasibility and conservation;
- reconcile simulation with reality;
- generate physical predictions through Spec 138B;
- learn physics through real systems.

---

## 2. Ownership

### 2.1 This specification owns

- physical-system identity;
- physical objects, components, media, fields, and environments;
- coordinate systems and reference frames;
- state vectors, state variables, parameters, and regimes;
- initial and boundary conditions;
- physical laws and conservation constraints;
- constitutive relationships;
- physical constants and authority;
- measurements, sensors, calibration references, and observation models;
- physical-model applicability and approximation levels;
- simulation and digital-twin contracts;
- physical feasibility and limiting constraints;
- experiment-model relationships;
- simulation-to-reality gap semantics.

### 2.2 Domain modules own

- specialized variables and equations;
- validated constants and material properties;
- solver and simulation adapters;
- domain-specific safety, regulation, and design criteria;
- specialist interpretation.

### 2.3 This specification does not own

- mathematical or statistical primitives;
- time authority;
- prediction commitment;
- actuation permission;
- physical-world truth without measurement or experimental authority;
- domain-specific engineering certification.

---

## 3. Physical ontology

```text
PhysicalSystem
PhysicalObject
Component
Particle
RigidBody
Continuum
Fluid
Solid
Material
Field
Environment
Interface
Contact
ReferenceFrame
CoordinateSystem
StateVector
StateVariable
Parameter
PhysicalConstant
InitialCondition
BoundaryCondition
PhysicalLaw
ConservationLaw
ConstitutiveRelation
Force
Torque
Momentum
AngularMomentum
Energy
Power
Heat
Temperature
Pressure
Flow
Charge
Voltage
Current
Wave
Signal
Sensor
Measurement
Actuator
Controller
EnergySource
Simulation
Experiment
DigitalTwin
PhysicalRegime
PhysicalFailureMode
FeasibilityEnvelope
```

---

## 4. Physical system contract

```yaml
schema: focusa.physical_system_model.v1
physical_system_id:
scope_ref:
subject_refs: []
component_refs: []
environment_ref:
interface_refs: []

frames:
  reference_frame_refs: []
  coordinate_system_refs: []

state:
  state_variable_refs: []
  state_vector_ref:
  parameter_refs: []
  constant_refs: []

model:
  governing_law_refs: []
  conservation_constraint_refs: []
  constitutive_relation_refs: []
  approximation_refs: []
  regime_ref:
  validity_envelope_ref:

conditions:
  initial_condition_refs: []
  boundary_condition_refs: []
  forcing_function_refs: []

measurement:
  sensor_refs: []
  measurement_model_refs: []
  calibration_refs: []
  uncertainty_budget_ref:

execution:
  solver_policy_ref:
  simulation_policy_ref:
  experiment_plan_ref:

runtime_constitution_ref:
temporal_snapshot_ref:
semantic_snapshot_ref:
model_hash:
status:
receipt_ref:
```

---

## 5. Reference frames and coordinates

Every vector, velocity, acceleration, force, momentum, orientation, and field observation MUST identify its frame where ambiguity could affect meaning.

Required distinctions:

```text
inertial frame
non-inertial frame
body-fixed frame
world frame
sensor frame
actuator frame
navigation frame
venue or market-clock frame as analogy only, not physical frame
```

Frame transformations are explicit mathematical operations with versioned conventions.

The system MUST detect:

- vectors added across untransformed frames;
- angular conventions mixed;
- left- and right-handed coordinate confusion;
- body/world frame ambiguity;
- stale frame transforms;
- missing orientation timestamps.

---

## 6. State and parameters

A physical model distinguishes:

- **state variables:** evolve during the modeled process;
- **parameters:** fixed or slowly varying within the active model;
- **controls:** externally selected inputs;
- **disturbances:** exogenous inputs;
- **observations:** measured functions of state;
- **latent states:** unobserved but modeled quantities.

```yaml
schema: focusa.physical_state_variable.v1
state_variable_id:
physical_system_ref:
quantity_definition_ref:
frame_ref:
spatial_domain_ref:
temporal_domain_ref:
observation_model_ref:
initial_condition_ref:
valid_range_ref:
```

---

## 7. Initial and boundary conditions

```yaml
schema: focusa.physical_condition.v1
condition_id:
condition_class: initial | boundary | interface | forcing | constraint
physical_system_ref:
target_state_or_field_ref:
spatial_scope_ref:
temporal_scope_ref:
quantity_value_ref:
functional_form_ref:
source_ref:
uncertainty_ref:
status:
```

A solver result without sufficient conditions is not a complete physical solution.

---

## 8. Physical laws and constitutive relations

```yaml
schema: focusa.physical_law_binding.v1
binding_id:
physical_system_ref:
law_ref:
applicable_regime_ref:
assumption_refs: []
state_variable_refs: []
parameter_refs: []
equation_refs: []
conservation_effect_refs: []
known_limitations: []
verification_refs: []
status:
```

The system distinguishes:

```text
fundamental law
conservation law
constitutive relation
empirical correlation
engineering approximation
phenomenological model
control law
numerical closure model
```

An empirical correlation cannot be presented as a fundamental mechanism.

---

## 9. Conservation constraints

Potentially applicable conservation checks include:

```text
mass
energy
linear momentum
angular momentum
electric charge
species amount
probability mass in coupled stochastic models
```

```yaml
schema: focusa.conservation_check.v1
check_id:
physical_system_ref:
conserved_quantity_ref:
control_volume_or_system_ref:
input_terms: []
output_terms: []
storage_change_ref:
source_sink_terms: []
residual_ref:
tolerance_ref:
result: satisfied | violated | indeterminate
violation_explanation_refs: []
evidence_refs: []
receipt_ref:
```

Conservation failure blocks physical verification unless an omitted source, sink, open-system boundary, or measurement explanation resolves it.

---

## 10. Measurement model

A sensor output is not automatically the target physical quantity.

```yaml
schema: focusa.measurement_model.v1
measurement_model_id:
sensor_ref:
target_quantity_ref:
observation_function_ref:
frame_ref:
sampling_policy_ref:
latency_profile_ref:
noise_model_ref:
bias_model_ref:
calibration_ref:
resolution_ref:
accuracy_ref:
uncertainty_ref:
saturation_ref:
quantization_ref:
drift_policy_ref:
validity_envelope_ref:
```

Required distinctions:

```text
raw sensor output
calibrated reading
corrected measurement
estimated latent state
fused measurement
simulated sensor output
```

---

## 11. Physical measurement record

```yaml
schema: focusa.physical_measurement.v1
measurement_id:
physical_system_ref:
sensor_ref:
measurement_model_ref:
raw_value_ref:
calibrated_value_ref:
target_quantity_ref:
frame_ref:
event_stamp_ref:
receive_stamp_ref:
ingestion_stamp_ref:
calibration_ref:
uncertainty_budget_ref:
quality_flags: []
environment_condition_refs: []
evidence_refs: []
receipt_ref:
```

Corrections append and supersede; they do not rewrite the original measurement.

---

## 12. Physical constants and properties

Constants and material properties MUST identify:

- value and units;
- source authority;
- version or edition;
- applicable conditions;
- uncertainty;
- temperature, pressure, frequency, composition, or regime dependence where relevant;
- temporal validity if the value is revised;
- license or redistribution conditions for external tables.

A model cannot silently treat a condition-dependent material property as universal.

---

## 13. Physical feasibility

`focusa_physics_feasibility` evaluates whether a proposed system or action can satisfy required physical constraints.

```yaml
schema: focusa.physical_feasibility_result.v1
result_id:
physical_system_ref:
target_behavior_ref:
constraint_refs: []
limiting_constraint_refs: []
required_capacity_refs: []
available_capacity_refs: []
safety_margin_refs: []
uncertainty_ref:
validity_envelope_ref:
result: feasible | infeasible | conditionally_feasible | indeterminate
condition_refs: []
recommended_measurement_refs: []
recommended_design_change_refs: []
verification_ref:
receipt_ref:
```

Examples:

- actuator torque and power;
- battery energy and discharge capability;
- thermal dissipation;
- structural load and fatigue;
- buoyancy and payload;
- flow and pressure;
- circuit voltage/current limits;
- communication energy and bandwidth when modeled physically;
- braking, stopping, and control authority.

---

## 14. Approximation and scale

Approximation records MUST state:

```text
ignored effects
small parameter or asymptotic basis
scale range
expected error
regime limits
failure conditions
comparison model
```

`focusa_scale_bridge` evaluates whether component-level results can be extrapolated to system, fleet, time, or spatial scale.

Linear scaling cannot be assumed across saturation, turbulence, thermal limits, queueing, nonlinear materials, feedback, or interaction effects.

---

## 15. Order-of-magnitude and limiting-case analysis

Every high-consequence physical model SHOULD support:

- order-of-magnitude estimate;
- dimensional analysis;
- zero/infinite or boundary parameter limits;
- frictionless/idealized limit where meaningful;
- conservation check;
- comparison with known scale or benchmark.

A detailed computation that contradicts a reliable scale estimate requires audit.

---

## 16. Simulation contract

```yaml
schema: focusa.physical_simulation.v1
simulation_id:
physical_system_ref:
model_ref:
initial_condition_refs: []
boundary_condition_refs: []
parameter_snapshot_ref:
forcing_refs: []
solver_ref:
discretization_ref:
step_policy_ref:
tolerance_ref:
stability_policy_ref:
random_seed_ref:
mesh_or_geometry_ref:
resource_budget_ref:
started_at_stamp_ref:
completed_at_stamp_ref:
convergence_ref:
validation_ref:
result_artifact_refs: []
warning_refs: []
evidence_refs: []
receipt_ref:
```

Simulation output remains `simulated` even after numerical verification. It becomes evidence about reality only through validation against authorized observations or experiments.

---

## 17. Digital twin

```yaml
schema: focusa.digital_twin.v1
twin_id:
physical_system_ref:
model_refs: []
measurement_stream_refs: []
state_estimator_ref:
parameter_update_policy_ref:
calibration_policy_ref:
validity_envelope_ref:
last_assimilated_measurement_ref:
last_update_stamp_ref:
drift_status:
verification_ref:
status:
```

A digital twin does not own physical truth. It is a model maintained against measurement evidence.

---

## 18. Experiment contract

```yaml
schema: focusa.physical_experiment.v1
experiment_id:
physical_system_ref:
question_ref:
hypothesis_refs: []
controlled_variable_refs: []
independent_variable_refs: []
dependent_variable_refs: []
confounder_refs: []
control_condition_refs: []
apparatus_refs: []
sensor_refs: []
calibration_refs: []
procedure_ref:
measurement_schedule_ref:
safety_boundary_refs: []
statistical_protocol_ref:
preregistration_stamp_ref:
execution_window_ref:
expected_outcome_by_hypothesis_refs: []
status:
receipt_ref:
```

Experiment design consumes Spec 152A and settlement consumes Spec 153A.

---

## 19. Simulation-to-reality gap

```yaml
schema: focusa.simulation_reality_gap.v1
gap_id:
simulation_ref:
experiment_or_observation_ref:
matched_quantity_refs: []
matched_temporal_window_ref:
matched_condition_refs: []
predicted_value_refs: []
observed_value_refs: []
residual_refs: []
attribution_refs: []
unexplained_residual_refs: []
candidate_missing_physics_refs: []
candidate_measurement_error_refs: []
candidate_parameter_error_refs: []
recommended_experiment_refs: []
model_revision_refs: []
```

Reality-gap analysis feeds Spec 138 learning only after verification.

---

## 20. Physics modules

```text
classical_mechanics
rigid_body_dynamics
continuum_mechanics
fluid_dynamics
thermodynamics
heat_transfer
electromagnetism
circuits
waves_and_oscillations
optics
materials_and_structures
control_systems
robotics
orbital_mechanics
acoustics
geophysics
relativity
quantum_mechanics
statistical_mechanics
biophysics
custom_scientific_pack
```

Modules activate only when applicable and supported by operational providers and verification.

---

## 21. High-value tools

```text
focusa_physics_model
focusa_physics_feasibility
focusa_physics_simulate
focusa_physics_verify
focusa_physics_conservation_audit
focusa_physics_measurement_plan
focusa_physics_experiment
focusa_physics_simulation_reality_gap
focusa_physics_scale_bridge
focusa_physics_order_of_magnitude
```

Vertical composites may include robotics, energy, structures, fluids, circuits, sailing, and sensor-system operations.

---

## 22. Robotics and actuation boundary

Physics cognition may model and advise actuation. Actual actuation requires:

- capability and permission;
- current physical state;
- fresh sensor authority;
- safety limits;
- deterministic control path;
- watchdog and kill switch;
- reconciliation after uncertain effect;
- applicable operator approval.

An LLM MUST NOT directly own a hard-real-time control loop.

---

## 23. Prediction integration

Physical forecasts bind:

- physical system and model version;
- state and parameter snapshot;
- measurement and calibration state;
- temporal snapshot;
- simulation or computation run;
- uncertainty and validity envelope;
- physical feasibility result;
- resolution measurement plan.

Spec 138B challenges both outcome probability and physical mechanism.

---

## 24. Verification obligations

Spec 144 vocabulary includes:

```text
physics.system_identity
physics.reference_frame
physics.coordinate_transform
physics.quantity_and_unit
physics.initial_conditions
physics.boundary_conditions
physics.law_applicability
physics.constitutive_relation
physics.conservation
physics.material_property_authority
physics.sensor_calibration
physics.measurement_traceability
physics.uncertainty
physics.numerical_stability
physics.simulation_validation
physics.physical_feasibility
physics.safety_margin
physics.experiment_design
physics.simulation_reality_gap
```

---

## 25. Security and safety

- physical models cannot grant actuation permission;
- unsafe requested targets are blocked or escalated;
- simulation and solver providers are sandboxed and bounded;
- sensor data follows privacy and access policy;
- hazardous experiment plans require applicable safety authority;
- model uncertainty and unknown physical state cannot be converted to permission;
- degraded sensing fails closed for consequential actuation;
- generated experiments do not authorize dangerous materials or procedures.

---

## 26. Persistence and replay

Canonical bounded state preserves:

- system definitions;
- frame and coordinate bindings;
- law and condition versions;
- measurement and calibration references;
- simulation plans and summaries;
- experiment contracts and settlement;
- feasibility and conservation findings;
- reality-gap evaluations;
- corrections and model invalidation;
- Evidence and Receipts.

Large meshes, trajectories, sensor streams, images, and simulation fields remain behind handles.

---

## 27. Required tests

1. incompatible reference frames rejected;
2. missing frame transform detected;
3. insufficient initial/boundary conditions block complete solution;
4. empirical correlation is not labeled fundamental law;
5. conservation violation blocks verification;
6. open-system source/sink resolves a valid apparent imbalance;
7. raw sensor output remains distinct from calibrated measurement;
8. expired calibration blocks high-consequence use;
9. measurement timestamp and frame are preserved;
10. condition-dependent property cannot be used universally;
11. physically impossible efficiency rejected;
12. limiting-case contradiction invalidates model;
13. converged simulation can still fail physical validation;
14. simulation cannot enter observation state;
15. experiment preregistration and measurement plan are preserved;
16. reality-gap analysis identifies residuals without inventing cause;
17. LLM actuation path rejected from hard-real-time control;
18. physical prediction binds complete model and measurement lineage;
19. module activation is applicability- and provider-gated;
20. replay reconstructs model, measurement, simulation, experiment, and settlement state.

---

## 28. Acceptance criteria

Spec 153 is accepted only when:

1. physical systems, frames, states, laws, conditions, measurements, simulations, and experiments are first-class;
2. quantities and units reuse Spec 152;
3. uncertainty and inference reuse Spec 152A;
4. time and measurement chronology reuse Spec 137B;
5. simulation and observation remain separate;
6. conservation, feasibility, order-of-magnitude, limiting-case, and scale checks are operational;
7. sensors and calibrations have explicit authority;
8. simulations declare method, discretization, stability, convergence, validity, and Evidence;
9. experiments bind safety and statistical protocols;
10. simulation-reality gaps feed verified model revision and learning;
11. hard-real-time actuation remains deterministic and authority-gated;
12. tools, ontology, reducer, persistence, replay, verification, Evidence, Receipts, and client parity are complete.

---

## 29. Canonical summary

```text
Spec 153 lets Focusa model the physical world without confusing the model,
the simulation, the sensor, the measurement, or the prediction with reality.
Physical truth remains earned through traceable measurement and experiment.
```
