# Spec 137B — High-Resolution Temporal Event Lineage, Precision Transport, and Deterministic Market Boundary Addendum

**Status:** NORMATIVE ADDENDUM — MANDATORY FOR HIGH-CONSEQUENCE TEMPORAL PROFILES — IMPLEMENTATION NOT IMPLIED  
**Parent:** Spec 137 + Spec 137A  
**Owner:** Focusa Core / Temporal Authority / High-Consequence Execution  
**Created:** 2026-08-01  
**Source baseline:** `77966bc82cd4229cf23985d0bff1a6bf14264363`  
**Primary consumers:** Spec 138B, Spec 144, Spec 152, Spec 152A, Spec 153, Spec 153A, Markets Vertical, engineering and robotics Verticals  
**Closure relationship:** A profile claiming microsecond, nanosecond, market-grade, exchange-grade, deterministic-latency, or high-resolution temporal conformance cannot close without this addendum.

---

## 0. Constitutional directive

```text
PRECISION IS A REPRESENTATION.
ACCURACY IS A MEASURED PROPERTY.
ORDER IS A CAUSAL CLAIM.

A timestamp is not authoritative merely because it contains many digits.
A high-resolution event is admissible only when its capture point, clock domain,
source lineage, synchronization posture, measured resolution, calibrated
uncertainty, causal order, transport representation, and deployed-path evidence
are explicit and verified.

LLMs may reason about market and scientific time.
They do not participate in deterministic microsecond execution loops.
```

---

## 1. Purpose

Spec 137 correctly distinguishes wall time, monotonic elapsed time, civil time, deadlines, urgency, uncertainty, and grounded forecasts. This addendum closes the additional requirements exposed by high-frequency markets, robotics, distributed measurement, and quantitative-scientific cognition:

1. one canonical high-resolution temporal stamp;
2. exact integer transport across Rust and JavaScript surfaces;
3. stable capture-point semantics;
4. paired wall/monotonic/TAI lineage;
5. measured resolution versus calibrated accuracy;
6. explicit clock-source authentication and diversity;
7. uncertainty-aware comparisons at microsecond and nanosecond boundaries;
8. causal and provider sequence ordering;
9. locally enforceable monotonic execution guards;
10. complete market intent-to-reconciliation traces;
11. cross-process single-writer and fencing requirements;
12. performance and calibration proof on the deployed path;
13. no-LLM deterministic execution boundary.

---

## 2. Scope and ownership

### 2.1 This addendum owns

- high-resolution timestamp representation;
- nanosecond integer storage and transport rules;
- precision, resolution, accuracy, uncertainty, and latency profiles;
- stable capture-point identity;
- wall/monotonic/suspend-aware/TAI sample pairing;
- cross-host causal and provider sequence lineage;
- microsecond/nanosecond freshness and age policies;
- deterministic execution lease and guard semantics;
- high-consequence temporal traces;
- deployed-path calibration and conformance;
- privacy-aware timestamp coarsening outside authorized profiles.

### 2.2 This addendum does not own

- deadline meaning, urgency, civil-time intent, or temporal closure owned by Spec 137;
- prediction questions, information sets, scoring, or learning owned by Spec 138;
- mathematical quantities or statistical inference owned by Specs 152/152A;
- market strategy, broker policy, order type, portfolio policy, or risk appetite;
- exchange or venue truth not supplied by an authorized adapter;
- hardware timestamping implementation inside a NIC, kernel, clock appliance, or venue;
- permission, approval, or settlement authority.

---

## 3. Required temporal domains

```text
wall_utc
monotonic_active
suspend_aware_elapsed
tai
provider_event_time
provider_publication_time
provider_sequence
venue_time
civil_time_intent
logical_sequence
causal_partial_order
```

No domain is silently substituted for another.

- Wall time is for correspondence with UTC and external events.
- Monotonic time is for local elapsed intervals and execution deadlines.
- Suspend-aware time is used when suspension consumes the governed interval.
- TAI is a projection only when supported and verified.
- Provider time is evidence supplied by a named provider, not Focusa clock authority.
- Logical sequence establishes reducer/ledger order.
- Causal references establish partial order where wall clocks cannot.

---

## 4. Canonical high-resolution temporal stamp

```yaml
schema: focusa.high_resolution_temporal_stamp.v1
stamp_id:
scope_ref:

capture:
  stable_capture_point:
  capture_adapter_ref:
  capture_adapter_version:
  host_id:
  process_id:
  thread_or_executor_ref:
  boot_id:

values:
  utc_epoch_ns: ""
  tai_epoch_ns:
  monotonic_ns: ""
  suspend_aware_ns:

ordering:
  reducer_sequence: ""
  ledger_sequence: ""
  provider_sequence:
  protocol_sequence:
  causal_predecessor_refs: []
  causal_successor_refs: []

clock:
  clock_domain:
  source_ids: []
  source_diversity_classes: []
  sources_authenticated:
  replay_protected:
  synchronization_status:
  synchronization_age_ns: ""
  holdover_age_ns:

quality:
  displayed_precision_digits:
  measured_resolution_ns: ""
  calibrated_accuracy_ns: ""
  standard_uncertainty_ns: ""
  expanded_uncertainty_ns: ""
  coverage_factor:
  coverage_probability_ppm:
  offset_ns:
  delay_ns:
  jitter_ns:
  dispersion_ns:
  root_distance_ns:
  frequency_error_ppb:

lineage:
  temporal_authority_ref:
  precision_profile_ref:
  uncertainty_method_ref:
  sample_pair_ref:
  calibration_receipt_ref:
  deployed_path_evidence_refs: []
  version_lineage_ref:

privacy:
  classification:
  coarsening_policy_ref:
  exported_precision_ns:

receipt_ref:
```

All integer fields capable of exceeding JavaScript's exact integer range MUST serialize as canonical decimal strings. Clients may provide ergonomic wrappers but cannot change canonical transport.

---

## 5. Stable capture points

Every high-consequence timestamp MUST identify where it was applied. Examples include:

```text
feed_socket_receive
message_decode_complete
feature_snapshot_sealed
forecast_commit_sealed
risk_check_start
risk_check_complete
order_intent_created
order_dispatch_kernel_boundary
broker_ack_receive
venue_ack_event
fill_event
cancel_request_dispatch
cancel_ack_receive
reconciliation_complete
sensor_sample_capture
actuator_command_dispatch
```

A timestamp applied at one point cannot be presented as though it described another. Capture-point changes are versioned compatibility changes and require recalibration.

---

## 6. Paired sampling and conversion

An authoritative wall/monotonic association MUST use a bracketed or otherwise measured sample pair:

```text
monotonic_before
→ wall/TAI/provider capture
→ monotonic_after
```

The record MUST preserve:

- capture order;
- lower and upper elapsed bounds;
- adapter and OS clock identity;
- boot ID;
- suspension posture;
- measurement latency;
- uncertainty propagation;
- correction lineage.

A process-relative monotonic origin is not a cross-process or cross-reboot epoch. Cross-boot elapsed time is bounded wall-derived duration with explicit uncertainty, never pure monotonic time.

---

## 7. Precision profiles

```yaml
schema: focusa.temporal_precision_profile.v2
profile_id:
domain_refs: []
integer_unit: nanosecond
required_capture_points: []
required_clock_sources:
required_independent_source_count:
required_authentication:
maximum_offset_ns: ""
maximum_root_distance_ns: ""
maximum_uncertainty_ns: ""
maximum_sample_age_ns: ""
maximum_holdover_ns: ""
maximum_capture_latency_ns: ""
maximum_transport_loss_ns: "0"
ordering_method:
coarsening_policy_ref:
deployed_path_fixture_refs: []
calibration_evidence_refs: []
status:
```

Profiles may include:

```text
general_millisecond
high_resolution_local
scientific_measurement
robotics_control
market_research
market_pretrade
market_execution
regulated_audit
```

A profile's numeric thresholds are domain policy. This addendum defines the required semantics and proof.

---

## 8. Accuracy and uncertainty laws

1. `measured_resolution_ns` describes the smallest distinguishable clock increment.
2. `calibrated_accuracy_ns` describes demonstrated agreement with the governing reference under the tested path.
3. `maximum_uncertainty_ns` is an admissibility threshold, not a display preference.
4. Resolution smaller than accuracy MUST NOT be described as accuracy.
5. Hard-coded uncertainty based only on clock availability is insufficient for high-consequence profiles.
6. Uncertainty MUST include relevant source, capture, transport, frequency, holdover, conversion, and calibration components.
7. Comparisons near a boundary MUST operate on uncertainty intervals.

Required boundary results:

```text
definitely_before
possibly_crossed
definitely_crossed
indeterminate
```

---

## 9. Deterministic execution guard

The general human-calendar and priority guard is insufficient for a microsecond execution path. High-consequence execution MUST use a locally checkable monotonic guard:

```yaml
schema: focusa.deterministic_temporal_execution_guard.v1
guard_id:
scope_ref:
host_id:
boot_id:
fencing_token: ""
issued_wall_stamp_ref:
issued_monotonic_ns: ""
expires_monotonic_ns: ""
maximum_clock_uncertainty_ns: ""
maximum_data_age_ns: ""
maximum_decision_age_ns: ""
maximum_dispatch_age_ns: ""
maximum_ack_age_ns: ""
authorized_action_refs: []
authorized_venue_refs: []
risk_limit_policy_ref:
kill_switch_epoch: ""
preauthorized:
reconciliation_policy_ref:
policy_version:
receipt_ref:
```

Admission MUST be local and deterministic. It MUST NOT require:

- an LLM call;
- an HTTP round trip to a conversational service;
- a remote prompt refresh;
- natural-language interpretation;
- mutable UI state.

The guard expires on:

- monotonic deadline;
- boot change;
- fencing-token supersession;
- kill-switch change;
- material risk-policy revision;
- venue or scope mismatch;
- clock uncertainty breach;
- data or decision staleness;
- unresolved prior external effect requiring reconciliation.

---

## 10. Market temporal trace

```yaml
schema: focusa.market_temporal_trace.v2
trace_id:
strategy_ref:
intent_id:
idempotency_key:

market_event_ref:
market_event_stamp_ref:
feed_receive_stamp_ref:
decode_complete_stamp_ref:
feature_snapshot_ref:
feature_snapshot_stamp_ref:
forecast_commit_ref:
forecast_commit_stamp_ref:
decision_ref:
decision_stamp_ref:
risk_check_ref:
risk_check_start_stamp_ref:
risk_check_complete_stamp_ref:
dispatch_ref:
dispatch_stamp_ref:
acknowledgement_ref:
acknowledgement_stamp_ref:
fill_refs: []
fill_stamp_refs: []
cancellation_ref:
cancellation_stamp_ref:
cancellation_ack_ref:
cancellation_ack_stamp_ref:
reconciliation_ref:
reconciliation_stamp_ref:

provider_sequence_refs: []
causal_sequence_refs: []
latency_distribution_ref:
clock_uncertainty_budget_ref:
partial_fill:
cancellation_race:
unknown_outcome:
kill_switch_checked:
receipt_refs: []
```

A retry is prohibited while prior effects are unknown unless the idempotency and reconciliation policy proves safety.

---

## 11. Prediction and information-set interlock

Spec 138B consumes this addendum. Every high-consequence prediction MUST bind:

- information cutoff stamp;
- temporal snapshot hash;
- evidence first-available stamps;
- evidence received and ingested stamps;
- forecast request and completion stamps;
- immutable commitment stamp;
- counterforecast stamp;
- adversarial reveal and close stamps;
- action and resolution stamps.

A prediction may not cite evidence whose authorized-known interval begins after its information cutoff.

---

## 12. Persistence and writer authority

Canonical high-resolution events MUST be written through a single-writer or equivalently fenced authority.

Required controls:

- writer lease or reducer ownership;
- monotonically increasing fencing token;
- compare-and-swap generation;
- authoritative sequence assignment by the writer;
- idempotency;
- signed or tamper-evident event chain according to profile;
- durable flush policy;
- crash recovery;
- duplicate detection;
- stale-writer rejection;
- cross-process concurrency tests.

Caller-supplied sequence numbers are advisory until admitted and reassigned by the canonical writer.

---

## 13. Exact transport and client parity

Generated schemas MUST represent large integers losslessly.

Permitted canonical forms:

```json
{"utc_epoch_ns":"1785600000123456789"}
```

or:

```json
{"seconds":"1785600000","nanoseconds":123456789}
```

Ordinary JSON floating-point or JavaScript `number` values MUST NOT carry authoritative epoch nanoseconds, monotonic nanoseconds, sequence values, fencing tokens, or exact duration limits when they may exceed exact range.

Parity tests MUST cover:

```text
Rust → JSON → TypeScript/Pi → MCP → browser/Tauri → Rust
```

with zero value loss.

---

## 14. LLM boundary

LLMs MAY:

- interpret slower market and scientific context;
- propose strategies and models;
- identify assumptions;
- produce counterforecasts;
- recommend experiments;
- explain traces;
- analyze resolved outcomes.

LLMs MUST NOT perform live:

- pretrade limit arithmetic;
- price or quantity rounding;
- deterministic order sequencing;
- dispatch age checks;
- kill-switch checks;
- cancellation-race resolution;
- fill reconciliation;
- actuator hard-real-time control.

---

## 15. API, CLI, Pi, and operation families

```text
temporal.precision.profile.validate
temporal.clock.capture.paired
temporal.clock.sources.evaluate
temporal.stamp.issue
temporal.stamp.verify
temporal.stamp.explain
temporal.guard.issue
temporal.guard.verify
temporal.guard.revoke
temporal.trace.market.create
temporal.trace.market.append
temporal.trace.market.reconcile
temporal.trace.audit
temporal.transport.exactness.verify
temporal.calibration.record
temporal.calibration.status
temporal.high_consequence.preflight
```

Generated Pi tools should be discoverable under a bounded `focusa_temporal_authority` and high-consequence workflow bundle rather than all being hot-loaded.

---

## 16. Required tests

1. wall-clock step backward and forward;
2. slew and drift;
3. suspend and resume;
4. reboot and boot-ID change;
5. holdover expiry;
6. source disagreement and quarantine;
7. authenticated-source loss;
8. leap-second and leap-smear incompatibility;
9. TAI unavailable or degraded;
10. paired-sample capture latency;
11. uncertainty interval straddling a deadline;
12. exact nanosecond serialization through every client;
13. cross-process concurrent writers;
14. stale fencing token;
15. crash during append and recovery;
16. duplicate idempotency key;
17. provider sequence gaps and duplicates;
18. out-of-order ingestion;
19. equal wall timestamps with distinct causal order;
20. cancellation race and partial fill;
21. unknown external effect before retry;
22. kill-switch epoch change;
23. LLM path rejected from deterministic loop;
24. deployed-path latency and accuracy calibration;
25. privacy coarsening for unauthorized clients.

---

## 17. Migration

1. introduce string-encoded nanosecond types beside legacy millisecond fields;
2. mark millisecond fields as profile-limited, not globally authoritative;
3. add capture-point and uncertainty lineage;
4. migrate high-consequence guards to monotonic v1 guard;
5. route canonical writes through fenced authority;
6. update Operation Registry and generated contracts;
7. add exactness and deployed-path conformance tests;
8. prohibit high-consequence activation while legacy bypasses remain.

Legacy records remain readable with an explicit `legacy_precision_unknown` classification and cannot satisfy strict high-resolution proof.

---

## 18. Acceptance criteria

Spec 137B is accepted only when:

1. canonical high-resolution stamps preserve exact integer values across all surfaces;
2. precision, resolution, accuracy, and uncertainty are distinct and proven;
3. every consequential timestamp identifies a stable capture point;
4. paired sample lineage and boot/suspend posture are preserved;
5. microsecond/nanosecond comparisons avoid millisecond truncation;
6. high-consequence guards are local, monotonic, fenced, and deterministic;
7. market traces cover event through reconciliation;
8. causal and provider sequences survive out-of-order arrival;
9. canonical writers assign sequence and reject stale writers;
10. prediction information sets can prove no future evidence;
11. LLMs are excluded from deterministic execution loops;
12. deployed-path calibration demonstrates the claimed profile;
13. tests, Evidence, Receipts, replay, migration, and client parity are complete.

---

## 19. Canonical summary

```text
Spec 137 defines temporal authority.
Spec 137A prevents temporal omission.
Spec 137B makes high-resolution temporal claims measurable, exact,
causally ordered, transport-safe, and admissible for deterministic execution.
```
