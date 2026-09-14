# Spec 137 — Focusa Temporal Authority, Deadlines, Urgency, and Grounded Forecasting

## Status

Normative draft — standalone core Focusa temporal-authority specification. This is not an addendum and does not renumber, replace, or broaden the identity of Spec 131. The specification is implementation-ready only after its typed contracts, cross-spec ownership map, and conformance ledger are approved. Existing partial timing fields are not evidence that this specification is implemented.

Canonical label: **Spec 137 — Temporal Authority, Deadlines, Urgency, and Grounded Forecasting**

Depends on: Specs 55, 56, 66, 67, 78, 79, 88, 96, 98, 100, 101, 104, 106, 107, 108, 109, 110, 111, 113, 116, 119, 120, 125, 130, 130A, 131, 133, and 136.

Cross-spec boundary: Spec 130 owns compaction continuity; true addendum Spec 130A owns compaction performance, cache preservation, and zero-waste gates; Spec 131 owns Workpoint Item timing, velocity, and closure authority; proposed Spec 136 owns proposal-to-settlement lifecycle and outcome truth subject to its stated activation conditions. Spec 137 owns the cross-system temporal primitives defined below and consumes—rather than duplicates—those neighboring authorities. Where an implementation requirement touches another owner's domain, the owning spec remains normative and Spec 137 records only the required temporal integration.

Research and inferred-decision basis: `docs/evidence/spec137-temporal-authority-research-audit-2026-07-20.md` and `docs/contracts/spec137-inferred-decision-register.v1.yaml`. External standards support the technical choices, but Focusa's approved local specification and operator authority remain normative.

Primary implementation surfaces: Focusa core, reducer, daemon, SQLite/CRDT persistence, API, Operation Registry, generated contracts, CLI, Pi extension, Awareness/Preload/Context Cognition, Workpoint, Work Loop, Silent Sessions, Trajectory, CompactionMissionPacket, Bloatgaurd, Evidence/ECS, Receipts, Closure Authority, predictions, metacognition, project cards, benchmarks, TUI, Mission Deck/Canvas, menubar, notifications, tests, conformance, and future timeline UI.

## Constitutional temporal directive

> **THE CALENDAR AND THE CLOCK NEVER WAIT.**
>
> **TO BE EARLY IS TO BE ON TIME. TO BE ON TIME IS TO BE LATE.**

Human wall-clock time is nonrenewable. Focusa SHALL minimize time-to-verified-and-settled outcome without weakening scope, safety, authority, evidence, reconciliation, accessibility, or operator control.

This directive is not motivational prose. It is a runtime invariant with typed records, reducer events, daemon enforcement, client projections, negative tests, Receipts, and post-settlement learning.

The doctrine has exact default semantics when the contract uses an exclusive deadline boundary and the trusted-time uncertainty interval does not straddle either boundary:

```text
settled definitely before readiness_target_at
    = on_time

settled definitely at/after readiness_target_at but definitely before operator_deadline_at
    = late / contingency consumed / at risk

settled definitely at/after an exclusive operator_deadline_at
    = deadline breached
```

An inclusive boundary applies the contract's explicit inclusion rule. `possibly_crossed` or `indeterminate` remains visibly uncertain and cannot be reported as definitely on time or definitely breached until policy-authorized reconciliation resolves it.

`complete` for deadline evaluation means every completion/settlement predicate in the approved local target-state contract has passed, external reality is reconciled where applicable, settlement is recorded, and the required Receipt is committed. Proposed Spec 136 is present in this repository but remains subject to its stated activation conditions; once active, its applicable predicates become the settlement contract without replacing Spec 137 temporal primitives or Spec 131 timing and closure authority. Code written, process exit, provider `200`, passing one test, an agent final message, or provider task status is not deadline completion.

## Problem

Focusa has partial elapsed-time and token accounting, but it is not accurate enough to measure real implementation speed, protect human deadlines, or support conversational duration claims. Current timing can reset on operator turns and is not reliably bound to Workpoints, Workpoint Items, beads/tasks, closure targets, proof, pauses, long-running tools, compactions, handoffs, scope changes, daemon restart, host sleep, or external outcome reconciliation.

Focusa currently budgets autonomous execution, but it does not govern the irreversible consumption of human calendar time. Activity may be mistaken for progress. Completed-only velocity can omit failed, abandoned, blocked, reopened, and censored attempts, creating survivorship bias. Agents and clients can emit apparently precise estimates without a trusted current-time source, comparable history, target-state definition, scope revision, uncertainty, or calibration.

Focusa needs one first-class temporal authority so:

- observed time is distinguished from forecasts and budgets;
- deadlines continue through pause, compaction, restart, and operator absence;
- material progress is distinguished from tool activity;
- wasted time produces visible, durable, operational consequences;
- estimates are grounded in measured comparable outcomes or refused;
- urgency changes routing and execution without weakening safety;
- completion, settlement, Receipts, prediction evaluation, and learning preserve temporal truth.

## Goals

1. Consume Spec 131 Workpoint timing records and extend them with trusted cross-system clock, deadline, urgency, and settlement-time semantics.
2. Separate wall-clock, monotonic elapsed, active agent, model, tool, queue, blocked, pause, operator-wait, proof, compaction, handoff, reconciliation, settlement, and offline time without redefining Spec 131 attribution categories.
3. Establish a daemon-owned, typed, scope-safe `TemporalAuthority` and bounded `TimeAwarenessPacket`.
4. Model external deadlines, earlier readiness targets, protected safety margins, calendar constraints, urgency, critical-path slack, and deadline breach.
5. Forbid unsupported numeric and qualitative time estimates across API, CLI, Pi, generated UI, TUI, menubar, Work Loop, and background reports.
6. Represent every permitted estimate as a durable, expiring, scope-revision-bound `EstimateClaim` with target state, comparable history, uncertainty, assumptions, and evaluation.
7. Detect and govern no-progress intervals, repeated unchanged reads, equivalent tool churn, unbounded research, diminishing returns, silent long-running commands, and avoidable rework.
8. Use Spec 131 Workpoint Items as the smallest independently measurable execution unit while preventing nested and concurrent double-counting.
9. Consume Spec 131 token, tool, proof, evidence, and closure attribution while adding operator-attention, resource-contention, and temporal-incident evidence.
10. Extend Spec 131 rollups with forecasts, urgency, deadline posture, and activated Spec 136 settlement stages without creating a second velocity model.
11. Require Spec 131 closure authority and activated Spec 136 outcome truth to preserve temporal posture; Spec 137 cannot independently mark work done.
12. Use all relevant attempt history—not successful completions alone—to improve forecasts, velocity reports, project cards, predictions, metacognition, routing, and critical-path planning.
13. Preserve Spec 131 timing/closure references plus Spec 137 deadlines, forecast provenance, progress, and incidents across compaction, model switches, forks, handoffs, provider overflow, daemon restart, host sleep, and CRDT reconciliation.
14. Make temporal breaches and missed opportunities durable, visible, non-resettable facts when evidence proves them.
15. Prepare the data model for calm, truthful timeline, Mission Canvas, TUI, menubar, notification, and future SaaS projections.
16. Remain the primitive-owning temporal specification consumed by the proposed Spec 136 lifecycle once its activation conditions are satisfied.

## Non-goals

- No surveillance, keystroke tracking, screen spying, or emotional manipulation.
- No claim that an agent literally experiences pain. Operational consequences, authority changes, routing changes, receipts, and learning make temporal failure consequential.
- No attempt to bill customers by raw token count in this spec.
- No silent rewriting of historical timing, deadline, estimate, progress, or breach records.
- No closure automation that overrides operator authority.
- No deadline pressure that bypasses scope, security, evidence, reconciliation, accessibility, or destructive-action controls.
- No assumption that more activity, more tokens, more agents, or longer execution means more progress.
- No point estimate when evidence supports only a range or no forecast at all.
- No conversion of an execution budget into a forecast or external deadline.
- No agent authority to create, extend, clear, or reinterpret an operator deadline without an authorized reducer path.
- No duplicate temporal state machine in Pi, CLI, generated UI, provider adapters, or activated Spec 136 adapters.
- No duplicate Workpoint Item timing, velocity, or closure authority owned by Spec 131.
- No duplicate compaction-performance or prompt-cache authority owned by Spec 130A.
- No visual SaaS implementation requirement; this spec defines the canonical substrate and required projections.
- No replacement of Specs 130/130A compaction authority, Spec 131 timing/closure authority, or Spec 136 proposal-to-settlement authority; Spec 137 provides temporal primitives consumed by those domains.

## Core temporal laws

The following laws are normative:

1. **Time facts are not predictions.** Trusted current time and measured elapsed intervals are observations. Estimated remaining duration is a probabilistic claim.
2. **Budgets are not forecasts.** Authorized runtime does not predict completion.
3. **Budgets are not deadlines.** A renewable Work Loop budget cannot modify an absolute calendar commitment.
4. **Activity is not progress.** Tool calls, tokens, file reads, compilation, process liveness, and narration do not establish material advancement.
5. **Process exit is not completion.** Completion and deadline posture use the exact required target state.
6. **Human time is not renewable.** Pause, compaction, restart, model change, scope confusion, verifier outage, and agent absence do not move an external deadline.
7. **Urgency cannot mint authority.** Deadline pressure cannot grant scope, permission, approval, evidence, closure, or destructive-action authority.
8. **Unknown means unknown.** Missing timing history, unclear completion target, stale scope, or unavailable temporal authority requires refusal of a duration estimate.
9. **Failure history counts.** Forecasting must include failed, blocked, reopened, rolled-back, abandoned, and censored attempts where relevant.
10. **Temporal state is scoped.** Every record binds to its applicable operator/host/project/continuity/Workpoint/item/task and cannot merge across incompatible scope.
11. **Corrections append.** Clock, attribution, estimate, deadline, and incident corrections supersede; they never rewrite history in place.
12. **Deadlines are revisioned authority.** Deadline changes require authenticated, authorized, audited reducer operations.
13. **Reconciliation survives expiry.** Deadline expiry may block new optional work but must not prevent truth recovery, compensation, process cleanup, evidence preservation, or settlement of possible effects.
14. **Only settled completion satisfies a commitment** when the approved target-state contract requires settlement and a Receipt; a locally adopted Spec 136 may supply those predicates, but its remote draft cannot.
15. **One temporal authority.** Clients render and request; the daemon/reducer/persistence path owns canonical temporal state.
16. **Guardrails are runtime-native.** Prompt reminders explain temporal policy but cannot be its enforcement boundary.
17. **No silent deferral or omission.** Every normative statement in this specification must be represented in the machine-readable requirement ledger. Every applicable `MUST`/`SHALL`, activated conditional requirement, and accepted `SHOULD` must be assigned to an implementation slice and closed by durable Evidence and a Receipt.
18. **Absence is not degradation.** Missing implementation, missing wiring, missing tests, an empty projection, a hard-coded placeholder, a mock, a hidden feature flag, or an unavailable route cannot be reported as degraded-but-complete.
19. **Later is a governed decision.** An applicable mandatory requirement may move to a later tranche only through an explicit operator-approved specification amendment that records the exact requirement IDs, reason, impact, replacement tranche, dependencies, acceptance consequences, and superseding Receipt. It remains visibly open and blocks any conformance level that requires it.
20. **Normative classes retain their meaning.** `MUST`/`SHALL` requirements are mandatory when applicable. An unimplemented `SHOULD` requires a recorded, evidence-backed variance and operator acceptance. A `MAY` is optional unless an operation, capability profile, conformance claim, or approved scope activates it; optionality never permits a client to claim unsupported behavior.
21. **Temporal primacy is continuous at decision boundaries.** Immediately below the verified current operator ask, fresh time awareness constrains every plan, action decision, tool decision, response, checkpoint, and continuation. Lower-priority context cannot suppress it, but critical deterministic executors use a pre-authorized local TemporalExecutionGuard rather than adding a remote/model refresh to their dispatch path.
22. **Every interaction and consequential action is human-calendar grounded.** Agent turns, schedule interpretations, relative-time phrases, autonomous continuation, and consequential action decisions require a fresh trusted local calendar/time context or explicit unavailable posture. Irrelevant hot-path records may carry a signed minimal context hash/guard ref instead of private calendar detail.
23. **Urgency is continuously computed.** A daemon-owned temporal pulse repeatedly evaluates trusted current time, elapsed time, deadlines, safety margin, slack, no-progress age, and opportunity risk; urgency cannot depend on the model remembering to check.
24. **Temporal learning is outcome-governed.** Predictions, reflections, and strategy changes about time are continually evaluated, but only evidence-backed settled outcomes may promote durable procedural change.
25. **Pressure produces calm protected focus, not panic.** As time pressure rises, the agent becomes progressively narrower, clearer, more execution-oriented, and more evidence-disciplined while preserving mandatory safety, security, authority, reconciliation, disconfirming-evidence, accessibility, and closure checks.
26. **Past-due delivery becomes the highest eligible execution priority.** After breach, Focusa first verifies what valid delivery opportunity remains, then freezes unrelated optional work and calmly focuses on the smallest safe path to deliver and settle the overdue item. Simultaneous non-preemptible or more-severe obligations create an explicit conflict/infeasibility state rather than silent selection.

## Completeness, non-deferral, and omission firewall

Every normative statement MUST map to at least one stable requirement ID in `docs/contracts/spec137-complete-feature-ledger.v1.yaml` before its implementation begins. The normalized IDs are `S137-REQ-001..S137-REQ-086`, corresponding to the numbered acceptance criteria. Detailed clauses map through the ledger's inherited `normative_source_coverage`; equivalent clauses may share a row only when the row preserves the strongest normative class and all behavior, applicability, failure, and proof obligations. A clause with no mapping, a weaker many-to-one mapping, or an unmapped new heading fails the completeness gate. The append-only/versioned ledger includes:

```yaml
requirement_id:
spec_section:
requirement_text:
requirement_class: must | shall | should | may
applicability: required | conditional | optional | not_applicable
applicability_condition_ref:
applicable_scope_refs: []
platform_refs: []
domain_refs: []
activation_ref:
applicability_decided_by:
applicability_evidence_refs: []
variance_ref:
primitive_owner:
implementation_slice:
blocking_dependencies: []
core_types: []
reducer_events: []
persistence: []
api_operations: []
cli_commands: []
pi_tools: []
ui_surfaces: []
operation_registry_changes: []
generated_contracts: []
migrations: []
positive_tests: []
negative_tests: []
restart_recovery_tests: []
security_tests: []
accessibility_tests: []
evidence_refs: []
receipt_refs: []
status: not_started | active | blocked | optional_unimplemented | variance_approved | not_applicable_verified | implemented_unverified | verified | explicitly_removed_by_amendment
amendment_ref:
```

The ledger's source-spec hash identifies the exact draft used to normalize the rows. Any normative edit invalidates that hash and requires ledger regeneration/review; acceptance-number reuse or semantic reassignment is prohibited.

Release and closure rules:

- A requirement cannot be omitted because its implementation is difficult, cross-platform, expensive, inconvenient, or discovered late.
- A requirement cannot be hidden in prose-only follow-up, TODO, issue comment, backlog, disabled test, ignored test, mock, compatibility fallback, or client-specific implementation.
- A backend-only implementation is incomplete when API, CLI, Pi, generated contracts, documentation, or required operator surfaces are applicable.
- A UI-only projection is incomplete without canonical daemon state and mutation authority.
- Happy-path proof is incomplete without required negative, stale, restart, scope, clock, and adversarial tests.
- Existing partial code does not automatically satisfy a requirement; it must pass the new contract and Evidence gate.
- Unsupported platforms or capabilities must be truthfully declared and remain open where this specification requires support.
- `not_applicable_verified` requires evidence that the declared platform/domain/conformance scope does not activate the requirement; it cannot hide an unsupported required capability.
- `SHOULD` deviations require a versioned variance with rationale, risk, scope, evidence, operator acceptance, and Receipt.
- `MAY` rows may remain `optional_unimplemented` only while no operation, capability profile, conformance claim, or approved scope activates them.
- A requirement marked blocked remains open; blocker visibility is not completion.
- Degraded mode cannot waive estimate grounding, deadline truth, scope, safety, evidence, reconciliation, or omission reporting.
- Generated clients, tool registries, current API/CLI references, migration notes, and conformance manifests are part of implementation, not optional documentation polish.
- Spec 137 cannot close while any mandatory ledger row is missing, open, silently deferred, unsupported without approved scope amendment, or evidenced only by assertion.

Every implementation tranche MUST publish:

1. included requirement IDs;
2. explicitly excluded applicable mandatory requirement IDs, which must be empty unless an approved amendment exists, plus separately evidenced optional/not-applicable/variance rows;
3. code and schema changes;
4. cross-surface parity results;
5. positive and adversarial proof;
6. restart/replay/migration results;
7. Evidence references;
8. a tranche Receipt;
9. remaining open requirement IDs;
10. confirmation that no requirement was silently deferred or omitted.

## Authority and ownership model

### Primitive ownership

Spec 137 owns the semantics and schemas for trusted clocks, cross-system elapsed intervals, civil-time intent, deadlines, readiness targets, estimate and forecast claims, temporal uncertainty, urgency, temporal breaches, opportunity posture, lost-time incidents, and temporal projections.

Spec 131 remains the sole owner of Workpoint Item timing, timing-ledger attribution, velocity, and closure authority. Spec 137 consumes those records as temporal evidence and defines no replacement Workpoint timing ledger or closure state machine. Sections below that discuss Workpoint Items, timing categories, velocity, or closure are integration requirements against Spec 131, not ownership transfers.

Spec 130A remains the sole owner of compaction performance, prompt-cache preservation, dynamic-current-turn-tail placement, cache telemetry, cache-miss classification, and cache-safe degraded mode. Spec 137 may consume their timestamps and incidents but cannot redefine their gates.

Proposed Spec 136 owns the proposal-to-settlement lifecycle and outcome-truth protocol under its stated activation conditions. It MUST consume Spec 137 temporal primitives and Spec 131 timing/closure records rather than create duplicate clock, deadline, estimate, timing-ledger, velocity, or closure authority.

### Authority table

| Component | May observe time | May propose estimate | May set deadline | May classify progress | May enforce | May settle temporal outcome |
| --- | --- | --- | --- | --- | --- | --- |
| Operator | Yes | Yes | Yes through authorized operation | May attest | Yes through policy | Yes where policy assigns |
| Trusted clock service | Yes | No | No | No | No | No |
| Agent/model | Reads projection | Yes, advisory | No direct mutation | May propose | No | No |
| Temporal estimator | Reads ledgers | Produces typed claim | No | No | No | No |
| Reducer | Applies clock-derived events | Commits accepted record | Commits revision | Commits verified event | Commits policy transition | Commits facts |
| Daemon/Work Loop | Reads | May request | No unilateral change | Runs deterministic checks | Yes | Coordinates evaluation |
| Pi/CLI/UI | Displays | Requests | Invokes operation | Displays | No independent authority | No |
| Receipt service | Reads | No | No | Verifies lineage | No | Records temporal lineage |
| Proposed Spec 136 settlement protocol, once activated | Reads | Evaluates actual | No | Evaluates evidence | Applies policy result | Yes through reducer path |

### Clock domains

Focusa MUST distinguish:

- `human_wall_clock` — calendar commitments and local human-readable time;
- `monotonic_elapsed` — duration measurement immune to wall-clock rollback;
- `execution_budget` — authorized resource consumption;
- `authority_expiry` — authorization freshness;
- `lease_expiry` — concurrency safety;
- `security_ttl` — pairing, token, nonce, and credential validity;
- `evidence_freshness` — age policy for proof;
- `provider_time` — external timestamps, advisory until reconciled.

These domains may share an underlying operating-system source but MUST NOT be treated as interchangeable semantics.

```yaml
schema: focusa.temporal_domain_clock_policy.v1
policy_id:
domain: human_wall_clock | monotonic_elapsed | execution_budget | authority_expiry | lease_expiry | security_ttl | evidence_freshness | provider_time
clock_source_class:
counts_suspend:
survives_reboot:
reboot_bridge_policy:
wall_correction_behavior:
uncertainty_limit_ns:
on_clock_unavailable:
on_uncertainty_exceeded:
platform_capability_ref:
policy_version:
```

Every expiry/duration domain declares whether suspend and reboot consume its interval. Leases, security TTLs, authority expiry, evidence freshness, and external deadlines normally continue through process sleep according to their own policy; active execution duration normally does not. No component may substitute a convenient clock whose suspend/reboot behavior contradicts the domain policy.

## Trusted clock and temporal authority

```yaml
schema: focusa.temporal_authority.v1
authority_id:
host_id:
operator_timezone:
tzdb_version:
wall_clock_source:
monotonic_clock_source:
suspend_aware_clock_source:
tai_clock_source:
clock_capability_profile_ref:
clock_trust_profile_ref:
active_precision_profile_ref:
boot_id:
monotonic_epoch_ref:
last_sample_pair_ref:
last_wall_sample_at:
last_monotonic_sample:
last_suspend_aware_sample:
synchronization_status: synchronized | holdover | acquiring | disagreeing | unauthenticated | unavailable
observed_offset_ns:
observed_delay_ns:
observed_jitter_ns:
observed_dispersion_ns:
observed_root_distance_ns:
frequency_error_ppb:
source_count:
source_diversity_status:
source_authentication_status:
leap_handling: step | smear | tai_projection | unknown
holdover_started_at:
measurement_uncertainty_ns:
coverage_probability:
clock_confidence: trusted | corrected | skewed | holdover | disagreeing | unauthenticated | unavailable
correction_event_ref:
schema_version:
```

```yaml
schema: focusa.clock_trust_profile.v1
profile_id:
required_source_count:
required_independent_source_count:
allowed_source_classes: []
required_authentication: nts | authenticated_provider | private_disciplined_source | not_required_by_profile
source_identity_refs: []
diversity_policy_ref:
disagreement_threshold_ns:
max_sync_age_ns:
max_holdover_ns:
max_offset_ns:
max_root_distance_ns:
leap_policy_ref:
on_disagreement: block | degrade | quarantine_source | operator_review
monitoring_policy_ref:
review_cadence_ref:
```

```yaml
schema: focusa.clock_sample_pair.v1
sample_pair_id:
authority_id:
boot_id:
capture_started_monotonic_ns:
wall_time_ns:
capture_finished_monotonic_ns:
suspend_aware_time_ns:
tai_time_ns:
capture_latency_ns:
source_measurement_ref:
uncertainty_components: []
combined_uncertainty_ns:
coverage_factor:
coverage_probability:
clock_capability_profile_ref:
clock_trust_profile_ref:
precision_profile_ref:
recorded_at:
```

```yaml
schema: focusa.clock_capability_profile.v1
profile_id:
platform:
platform_version:
wall_clock_semantics:
monotonic_clock_semantics:
monotonic_counts_suspend:
suspend_aware_clock_semantics:
tai_clock_semantics:
tai_requires_sync_support:
resolution_by_clock: {}
clock_set_behavior:
absolute_timer_behavior:
capability_test_evidence_refs: []
status: verified | degraded | unsupported | unknown
```

Requirements:

- Active-duration arithmetic uses monotonic time only within one boot/process clock epoch.
- Per-boot monotonic segments may be summed as active time, but inter-boot gaps remain separately wall-derived, bounded, or `unknown`; linked boot epochs never become one synthetic monotonic clock.
- Human deadlines preserve fixed-instant or civil-time intent according to the typed deadline semantics below; a timezone-aware absolute instant alone is not sufficient for every future commitment.
- Host sleep and daemon downtime continue wall-clock elapsed while active execution stops. Suspend-sensitive expiry domains use a verified suspend-aware clock where the platform provides one.
- Reboot creates a new monotonic epoch linked through persisted ClockSamplePairs with explicit uncertainty. An untrusted wall bridge cannot produce exact inter-boot duration.
- NTP correction/slew, manual clock change, DST, timezone/tzdb change, source disagreement, leap/smear policy, suspend/wake, and device/daemon skew produce explicit posture and append-only correction events; they cannot create negative or falsely exact elapsed time.
- Accuracy-sensitive deployments use monitored independent/diverse sources and authenticated synchronization such as NTS or an evidence-backed equivalent. Source count alone is not proof of independence or correctness.
- Holdover age, source disagreement, synchronization age, authentication loss, and uncertainty growth are continuously evaluated against the active profile.
- Client clocks are observations, not canonical authority.
- Transcript timestamps are never authority fallback.
- When temporal authority is unavailable or its uncertainty exceeds operation policy, clients fail closed for estimates and affected deadline/dispatch arithmetic while preserving safe status, reconciliation, cleanup, and recovery guidance.

## Precision, accuracy, resolution, and uncertainty

Focusa MUST NOT conflate:

- **resolution** — smallest representable/measurable interval;
- **precision** — repeatability or number of represented digits;
- **accuracy** — closeness to an authoritative time source;
- **uncertainty** — bounded error around a reported timestamp/duration;
- **latency** — delay between event occurrence, observation, decision, dispatch, acknowledgement, and reconciliation;
- **ordering** — causal/sequence relation, which timestamps alone may not prove.

A timestamp with microsecond digits is not evidence of microsecond accuracy. Every high-consequence timestamp and latency measurement MUST include source, capture point, unit, clock domain, synchronization posture, resolution, uncertainty/error bound, and provenance.

```yaml
schema: focusa.temporal_precision_profile.v1
profile_id:
profile_kind: human_calendar | operational_millisecond | high_consequence_microsecond | custom
clock_domain:
clock_source:
synchronization: none | ntp | chrony | ptp | gps | provider | venue | custom
resolution_ns:
target_accuracy_ns:
observed_error_bound_ns:
uncertainty_method_ref:
uncertainty_component_refs: []
coverage_factor:
coverage_probability:
max_permitted_error_ns:
max_sync_age_ns:
max_holdover_ns:
max_data_age_ns:
max_decision_age_ns:
max_dispatch_age_ns:
latency_quantiles_required: [p50, p95, p99, p99_9, max]
platform_capability_ref:
measurement_method_ref:
calibration_evidence_refs: []
fail_mode: warn | block | operator_review | kill_switch
status: supported | degraded | unsupported | unverified
```

Rules:

- Durations use integer nanoseconds/microseconds/milliseconds as appropriate; floating-point time is prohibited for authority, ordering, money-affecting deadlines, and settlement.
- Human projections may format RFC 3339 with IANA timezone, but canonical records preserve UTC instant, offset, timezone identifier, source, and uncertainty.
- High-precision distributed systems SHOULD use leap-safe monotonic/TAI-aware measurement and disciplined UTC projection; leap-second handling is explicit and tested.
- Cross-host/event ordering uses provider/venue sequence IDs, correlation/causation IDs, and protocol state; timestamp ordering alone is insufficient.
- Threshold comparisons are uncertainty-aware. A clock interval relative to a deadline/expiry is classified as `definitely_before`, `possibly_crossed`, `definitely_crossed`, or `indeterminate`; an uncertainty interval that straddles a consequential boundary cannot be treated as definitely on time.
- If observed uncertainty exceeds policy, high-consequence dispatch fails closed. The system may reconcile or cancel safely but may not pretend the requested accuracy remains available.
- Accuracy claims require runtime calibration Evidence for the actual host, network, adapter, provider, and capture path. Platform documentation or timestamp formatting is insufficient.
- Uncertainty reports record method, components, combined/expanded uncertainty, coverage factor/probability, sample age, and calibration lineage; a single unqualified error number is insufficient.
- High-resolution time is capability- and audience-scoped. Untrusted clients, general models, exports, and public telemetry receive policy-coarsened values where detailed timing would create privacy, fingerprinting, strategy, or security risk.
- Every operator surface displays the effective profile and degraded/unsupported posture without false precision.

### Human time and machine event time

Focusa SHALL carry both layers coherently:

1. `human_commitment_time` — readiness targets, external deadlines, working windows, review/delivery margins, and operator-visible calendar reality.
2. `machine_event_time` — market/data/provider events, ingestion, decision, authorization, dispatch, acknowledgement, fill/effect, reconciliation, and settlement.

A human deadline may be measured in days while a consequential execution window is measured in milliseconds or microseconds. Both bind to the same lineage but use explicitly different precision profiles and policies.

## High-consequence and financial-market temporal profile

Focusa is domain-general, but financial-market use makes temporal error potentially catastrophic. No Focusa component may claim readiness for live market operation merely because the general timing substrate exists.

The Markets domain pack MUST define and prove, for each operation class:

- authoritative market calendar, venue session, holiday, early close, auction, halt, and timezone rules;
- market-data source, event timestamp, receive timestamp, sequence, age, gap, and stale-data policy;
- decision timestamp and policy/model revision;
- AuthorityDecision and ExecutionIntent timestamps/expiry;
- pre-trade risk-check start/result timestamps;
- order creation, local dispatch, broker receipt, venue acknowledgement, rejection, partial fill, fill, cancel request, cancel acknowledgement, correction, and reconciliation timestamps;
- end-to-end and stage latency distributions with uncertainty;
- maximum tolerated clock error, data age, decision age, dispatch age, acknowledgement latency, and reconciliation age;
- idempotency, duplicate-order prevention, sequence recovery, cancellation race, partial-fill, disconnect, timeout-after-possible-effect, and unknown-outcome policy;
- exposure, position, price, quantity, loss, concentration, and rate limits;
- circuit breaker and independently reachable kill switch;
- credential, account, venue, instrument, strategy, operator, and jurisdiction scope;
- immutable audit/Receipt requirements and retention;
- simulation, paper-trading, shadow, canary, and live capability levels.

### Deterministic execution boundary

An LLM turn, conversational agent, general Work Loop, networked model call, or UI rendering path MUST NOT sit in a microsecond- or millisecond-critical live order-routing loop.

Models may observe, propose, explain, critique, and produce governed strategy/policy candidates. A specialized deterministic, pre-authorized, risk-bounded adapter/execution engine performs latency-critical actions using immutable inputs and exact constraints. The approved local settlement protocol governs proposal, authority, durable intent, dispatch lineage, reconciliation, completion, settlement, and learning around that engine; this is proposed Spec 136 only after its activation conditions are satisfied.

The critical path uses a signed, immutable, locally enforceable `TemporalExecutionGuard`; it MUST NOT synchronously fetch a model turn, UI state, remote calendar, or daemon projection immediately before dispatch. Full HumanCalendarContext and TemporalPriorityFrame lineage is resolved before guard issuance and linked asynchronously to resulting audit events. Stale, unverifiable, revoked, or uncertainty-violating guards fail closed.

### Time-sensitive action law

For some market operations, a late action is more dangerous than no action. Every consequential intent includes:

```yaml
schema: focusa.temporal_execution_guard.v1
guard_id:
guard_version:
executor_host_id:
boot_id:
monotonic_clock_source:
human_calendar_context_hash:
deadline_ref:
execution_intent_ref:
event_time_ref:
data_observed_at:
decision_at:
authority_checked_at:
issued_wall_at:
issued_monotonic_ns:
valid_until_monotonic_ns:
dispatch_not_before_monotonic_ns:
dispatch_deadline_monotonic_ns:
cancel_deadline_monotonic_ns:
max_data_age_ns:
max_decision_age_ns:
max_clock_error_ns:
precision_profile_ref:
clock_sample_pair_ref:
authority_revision:
policy_hash:
kill_switch_epoch:
sequence_fence:
nonce:
signature_ref:
on_expiry: block | cancel | reconcile_only | kill_switch | operator_review
```

Immediately before dispatch, the deterministic executor locally rechecks guard validity, monotonic time, data freshness, authority revision, risk, account/instrument scope, sequence fence, and kill-switch epoch. Expired, revoked, stale, signature-invalid, sequence-invalid, or uncertainty-violating intents MUST NOT dispatch. After a possible effect, timeout/disconnect enters `outcome_unknown` and reconciliation-before-retry under the approved local settlement protocol; blind retry is prohibited.

### Live-market activation firewall

Live market mutation remains blocked until all required Markets domain-pack ledger rows are verified, including:

1. authoritative clocks and measured accuracy on deployed hardware;
2. deterministic execution outside the model path;
3. pre-trade and continuous risk limits;
4. paper/shadow/canary evidence;
5. broker/venue reconciliation and unknown-outcome recovery;
6. partial-fill/cancel-race/duplicate prevention;
7. market calendar and stale-data tests;
8. overload, disconnect, clock-drift, leap, halt, and restart tests;
9. operator-visible independently reachable kill switch and incident runbook;
10. independent security/risk review, Evidence, Receipt, and explicit operator activation;
11. exact timestamp-application points, stable capture-point identity, UTC traceability design, and periodic compliance review;
12. direct/exclusive control ownership, documented delegation/due diligence where allowed, written supervisory review, issue remediation, certification, and retention;
13. capacity, integrity, resiliency, availability, security, business-continuity/disaster-recovery, RTO/RPO, geographically diverse recovery, and wide-scale disruption proof;
14. jurisdiction, venue, account, activity-class, rule/version/effective-date applicability and required accuracy/granularity thresholds;
15. regulatory/venue incident notification, books-and-records, audit export, and supersession/migration evidence.

Backtest success, simulated timestamps, provider `200`, model confidence, low average latency, or one successful order cannot activate live trading. Unsupported timing accuracy, missing uncertainty, stale data, unavailable reconciliation, incomplete risk controls, missing review/certification, or unproven regulatory applicability fail closed and cannot be silently deferred.

### Cross-domain carryover

Every other high-consequence domain—legal filing, healthcare workflow, infrastructure incident response, security credentials, release windows, industrial control, emergency response, and custom packs—MUST declare its own `TemporalPrecisionProfile`, calendar/deadline semantics, freshness limits, deterministic boundaries, failure mode, evidence, and conformance. The Markets profile is the strict exemplar, not a special alternate temporal runtime.

## Deadline and calendar contract

```yaml
schema: focusa.deadline_contract.v1
deadline_id:
subject_ref:
subject_kind: workpoint_item | workpoint | work_item | task | spec | mission | project | release | external_commitment
project_root:
continuity_id:
workpoint_id:
item_id:
operator_id:
deadline_semantics: fixed_instant | zoned_civil_time | floating_local_time | date_only | business_calendar_date | recurring_calendar_event | external_session_event
timezone:
tzdb_version:
civil_time_intent_ref:
readiness_target_semantics:
readiness_target_intent_ref:
calendar_source_ref:
calendar_version:
fixed_instant_at:
operator_deadline_at:
readiness_target_at:
boundary_semantics: inclusive | exclusive
completion_effect: submitted | accepted | acknowledged | externally_effective | reconciled | settled
required_safety_margin_ms:
safety_margin_basis: operator_supplied | policy | measured_history
completion_target_state:
completion_policy_ref:
settlement_required:
receipt_required:
priority:
hardness: advisory | soft | hard | external_window
deadline_authority_kind: operator | contract | legal | regulatory | venue | provider | policy
deadline_source_ref:
calendar_constraint_refs: []
parent_deadline_ref:
created_by:
created_at:
resolved_at:
resolution_revision:
revision:
supersedes:
status: active | revised | projected_change | cleared | satisfied_early | late_window | breached | cancelled
authority_ref:
reason:
```

Rules:

- `operator_deadline_at` and `readiness_target_at` are the current effective instant projections. For fixed-instant semantics the fixed instant is authoritative; for civil/floating/recurring/session semantics the preserved intent and resolution policy are authoritative and projection revisions remain append-only.
- `readiness_target_at` MUST be earlier than `operator_deadline_at` when a protected margin is required.
- The safety margin preserves verification, review, integration, recovery, delivery, and final confirmation time.
- The agent may not invent a margin; it is operator-supplied or policy/measurement-grounded. Unknown margin remains visible and may require operator review under consequence/proximity policy.
- Satisfying a child deadline does not satisfy its parent unless the parent's target state is independently proven.
- Revising Focusa's record of an external/legal/regulatory/venue deadline does not grant authority to change the real external boundary. An operator may cancel the objective or correct the record with evidence, but cannot clear an immutable external fact.
- Clearing, extending, or weakening an operator/policy deadline requires authenticated scope, CAS revision, reason, applicable authority, and audit Receipt.
- Crossing the readiness target creates `late_window` even if the external deadline has not yet passed.
- Boundary status uses the active clock uncertainty interval; `possibly_crossed` or `indeterminate` cannot be reported as definitely on time.
- A pause or renewable budget never changes either boundary.
- Within non-waivable safety, legal, scope, identity, permission, evidence, and authority constraints, multiple deadlines are ordered by immediate operator steering, consequence, reversibility, critical-path impact, hardness, priority, and uncertainty-aware slack. Clients must not silently choose one.
- Operator working hours, approval availability, release windows, maintenance windows, provider cutoffs, and scheduled interruptions use typed versioned `CalendarConstraint` records with minimum-data privacy.

### Civil-time intent and resolution

```yaml
schema: focusa.civil_time_intent.v1
intent_id:
original_expression:
local_date:
local_time:
iana_timezone:
tzdb_version:
jurisdiction:
calendar_ref:
calendar_version:
recurrence_rule:
fold_policy: earlier | later | reject | operator_review
gap_policy: shift_forward | shift_backward | reject | operator_review
resolution_policy: preserve_instant | preserve_civil_intent | operator_review
resolved_instant:
resolved_offset:
resolved_at:
resolver_version:
prior_resolution_ref:
resolution_evidence_refs: []
status: resolved | ambiguous | nonexistent | projected_change | disputed
```

```yaml
schema: focusa.calendar_constraint.v1
constraint_id:
constraint_kind: working_window | approval_window | release_window | maintenance_window | provider_cutoff | market_session | legal_calendar | scheduled_interruption
subject_ref:
source_ref:
source_version:
authority_ref:
civil_time_intent_ref:
start_projection_at:
end_projection_at:
boundary_semantics:
recurrence_rule:
freshness_expires_at:
signature_ref:
priority:
status: active | revised | stale | conflicted | cancelled
supersedes:
```

A future civil commitment preserves the original civil expression and resolution policy. A tzdb, jurisdiction, holiday, market-calendar, or source revision re-resolves the projection, appends old/new instants and evidence, and creates `projected_change` when material. Offset/zone inconsistency, zero/multiple local-time mappings, unknown recipient timezone, and floating time cannot be silently guessed.

Calendar policy declares authoritative source precedence, source/version/effective date, fetch and freshness time, signature/authentication where available, jurisdiction, conflict handling, and fail mode. Disagreement between operator, provider, venue, regulatory, holiday, or tzdb sources becomes explicit `calendar_source_conflict`; no client silently chooses the most convenient boundary.

### Deadline inheritance, probabilistic slack, and conflict

```text
grounded schedule slack at policy quantile q
  = time until readiness target
  - grounded correlated critical-path remaining duration at q
```

The quantile and loss/risk policy are explicit. Independent task ranges are not naively summed when dependencies or durations are correlated. If critical-path duration is not grounded, slack is `unknown`, not a fabricated number. Unknown slack near a hard/high-consequence deadline escalates according to proximity, consequence, reversibility, and evidence confidence; it never defaults to positive slack.

```yaml
schema: focusa.deadline_conflict.v1
conflict_id:
status: none | feasible | infeasible | unknown
conflicting_deadline_refs: []
minimum_required_action_refs: []
non_preemptible_action_refs: []
preemption_costs: []
selected_primary_objective_ref:
preserved_background_obligation_refs: []
displaced_commitment_refs: []
priority_decision_ref:
operator_review_required:
evidence_refs: []
```

Critical mode has one primary execution objective while retaining non-preemptible safety, containment, kill-switch, monitoring, reconciliation, evidence-preservation, and legal obligations. When all commitments cannot be met, Focusa records `infeasible` and discloses the conflict; prioritization cannot imply that every deadline remains achievable.

### Distributed deadline, cancellation, and retry propagation

At RPC/process/agent boundaries, Focusa converts an absolute deadline into a remaining monotonic timeout, deducts elapsed transit/processing time, and caps every child deadline at the minimum of parent remaining time, child policy, and execution budget. The original deadline and conversion sample remain attached for audit; receivers do not compare unsynchronized host wall clocks as if they were identical.

```yaml
schema: focusa.deadline_propagation.v1
propagation_id:
original_deadline_ref:
parent_operation_ref:
child_operation_ref:
sender_sample_pair_ref:
remaining_timeout_ns_at_send:
receiver_observed_timeout_ns:
elapsed_deducted_ns:
child_timeout_ns:
conversion_uncertainty_ns:
status: active | expired_in_transit | rejected | completed
```

```yaml
schema: focusa.cancellation_contract.v1
cancellation_id:
parent_operation_ref:
child_operation_refs: []
child_ack_refs: []
requested_at:
observed_at:
effective_at:
grace_deadline_monotonic_ns:
force_termination_policy_ref:
cleanup_required:
reconciliation_required:
possible_external_effect:
status: requested | observed | effective | forced | outcome_unknown | reconciled
```

Spawned work periodically or eventfully observes cancellation, acknowledges it, stops within policy, and preserves cleanup/reconciliation. Retries share the original deadline and bounded retry budget. Timeout after a possible external effect is never retry authority; reconciliation/idempotency evidence is required first.

## Past-due opportunity assessment and delivery priority

A canonical TemporalBreach requires a definitely crossed boundary under the active uncertainty/boundary policy or independent authoritative evidence of breach. `possibly_crossed` and `indeterminate` remain visible OpportunityRisk states requiring bounded reconciliation; they do not fabricate either on-time success or confirmed lateness.

A missed deadline does not authorize resignation, distraction, concealment, or immediate movement to easier work. It creates a `TemporalBreach` and activates a mandatory overdue-delivery protocol.

### Opportunity remaining assessment

Before any post-deadline dispatch, Focusa records:

```yaml
schema: focusa.overdue_opportunity_assessment.v1
assessment_id:
subject_ref:
deadline_ref:
temporal_breach_ref:
assessed_at:
operator_ask_ref:
original_target_state:
current_verified_state:
remaining_delivery_window:
opportunity_status: fully_deliverable | partially_deliverable | alternate_delivery_possible | window_closed | delivery_harmful | unknown
valid_delivery_options:
  - option_id:
    target_state:
    value_preserved:
    required_actions: []
    safety_authority_posture:
    evidence_required: []
    latest_valid_dispatch_at:
    latest_valid_settlement_at:
invalid_or_expired_actions: []
recommended_delivery_option:
operator_review_required:
evidence_refs: []
reason_codes: []
```

The assessment determines whether the original delivery remains useful, a partial/alternate delivery preserves value, the opportunity window is closed, or late action would create harm. The agent must not infer remaining opportunity from desire or sunk cost; it uses current external state, calendar/market/session state, recipient/provider capability, authority, and evidence.

### Overdue delivery mode

When `opportunity_status` is `fully_deliverable`, `partially_deliverable`, or `alternate_delivery_possible`, the overdue item becomes the highest execution priority immediately below valid current operator steering and non-waivable safety/truth obligations.

```yaml
schema: focusa.overdue_delivery_mode.v1
mode_id:
subject_ref:
assessment_ref:
activated_at:
selected_delivery_option:
critical_delivery_path: []
frozen_unrelated_work_refs: []
allowed_supporting_actions: []
prohibited_optional_actions: []
progress_update_interval_ms:
operator_notification_ref:
exit_condition: settled | operator_redirected | opportunity_closed | blocked_by_nonwaivable_constraint
```

Required behavior:

1. acknowledge the breach calmly and factually;
2. stop unrelated research, cosmetic work, broad refactoring, optional tests, speculative optimization, and postmortem analysis;
3. preserve scope, authority, safety, evidence, and reconciliation requirements;
4. select the smallest valid path to useful delivery;
5. execute only delivery-critical and required supporting actions;
6. provide bounded progress updates and surface blockers immediately;
7. preserve and deliver the best verified partial result if full delivery becomes impossible;
8. reconcile, verify, settle, and Receipt the actual late outcome;
9. perform lost-time analysis and learning only after delivery/settlement unless analysis is required to unblock delivery.

The Work Loop pins the overdue item and does not select unrelated ready work while a valid delivery path remains, except for:

- explicit operator redirection after the breach is disclosed;
- immediate safety/security containment;
- reconciliation, compensation, or cleanup of a possible external effect;
- a non-waivable authority/legal constraint;
- another commitment whose verified consequence is more severe, requiring visible operator conflict resolution.

### Closed or harmful opportunity

When the original window is closed or late dispatch would be harmful—such as an expired market order intent—the agent MUST NOT blindly execute the original action. Highest-priority delivery becomes the truthful residual outcome: cancel/contain, reconcile external state, preserve evidence, provide the best valid partial or alternate deliverable, notify the operator, and settle the breach with open obligations.

`unknown` opportunity status requires bounded reconciliation or operator review; it cannot justify either blind retry or quiet abandonment.

The temporal breach remains in the final Receipt even when late delivery succeeds. Successful late delivery does not rewrite the deadline outcome as on time.

## Time Awareness Packet

```yaml
schema: focusa.time_awareness.v1
packet_id:
generated_at:
expires_at:
source_state_revision:
temporal_authority_ref:
clock_confidence:
clock_uncertainty_ns:
clock_sample_pair_ref:
clock_policy_version:
trusted_now:
timezone:
tzdb_version:
scope:
  project_root:
  continuity_id:
  workpoint_id:
  item_id:
  task_id:
work_started_at:
wall_clock_elapsed_ms:
active_agent_elapsed_ms:
tool_elapsed_ms:
blocked_ms:
paused_ms:
operator_wait_ms:
proof_ms:
compaction_ms:
handoff_ms:
queue_ms:
reconciliation_ms:
settlement_ms:
nearest_deadline_ref:
top_deadlines:
  - deadline_ref:
    subject_ref:
    readiness_target_at:
    operator_deadline_at:
    deadline_semantics:
    boundary_posture: definitely_before | possibly_crossed | definitely_crossed | indeterminate
    pressure:
    slack_status:
    slack_quantile:
    evidence_confidence:
    critical_path_rank:
    priority_reason:
readiness_target_at:
operator_deadline_at:
deadline_remaining_ms:
deadline_remaining_lower_bound_ms:
deadline_remaining_upper_bound_ms:
safety_margin_remaining_ms:
safety_margin_lower_bound_ms:
safety_margin_upper_bound_ms:
deadline_boundary_posture: definitely_before | possibly_crossed | definitely_crossed | indeterminate
schedule_slack_ms:
schedule_slack_lower_bound_ms:
schedule_slack_upper_bound_ms:
schedule_slack_quantile:
schedule_slack_status: positive | exhausted | breached | unknown
deadline_conflict_ref:
last_material_progress_ref:
last_material_progress_at:
no_progress_elapsed_ms:
unchanged_document_rereads:
equivalent_tool_attempts:
same_subproblem_attempts:
temporal_pressure: normal | watch | at_risk | critical | expired
active_temporal_claim_refs: []
active_estimate_claim_refs: []
active_temporal_execution_guard_refs: []
lost_time_incident_refs: []
critical_next_action:
rehydrate_refs: []
```

Packet laws:

- It is a bounded projection, not a second store.
- Every field is derived from canonical state, registered versioned policy, or append-only temporal records, and the projection preserves the schema/policy/clock/calendar/estimator versions needed to reproduce authority-bearing decisions.
- It expires and is invalidated by scope, Workpoint, deadline, progress, steering, or relevant policy revision.
- Pi receives it before every model turn, after long-running tools, at continuation boundaries, and after compaction/resume.
- During model/tool execution, the daemon watchdog—not prompt memory—enforces deadlines and cancellation.
- Context Cognition, Awareness, Preload, WorkpointResumePacket, CompactionMissionPacket, Work Loop status, Silent Session status, TUI, Mission Canvas, and menubar consume the same projection.

## Agent temporal primacy and attention ordering

Time awareness MUST remain at the forefront of every agent operation. It is not a background dashboard metric, optional reminder, end-of-task statistic, or prompt appendix.

Within non-waivable safety, legal, scope, identity, permission, evidence, and authority boundaries, the agent's decision order is:

```text
1. verified immediate operator ask and current steering
2. fresh TemporalPriorityFrame
3. exact Workpoint/Workpoint Item and critical next action
4. acceptance, evidence, reconciliation, and settlement requirements
5. all other project, historical, research, optimization, and stylistic context
```

Time is therefore the first universal constraint immediately below the operator's current ask. It does not outrank safety or authorize prohibited work, but no lower-priority context may suppress, displace, hide, or cause the agent to forget it.

### Human calendar grounding

Every interaction and consequential action decision binds to a bounded projection; privacy-irrelevant hot-path records may retain only its signed context hash or TemporalExecutionGuard ref:

```yaml
schema: focusa.human_calendar_context.v1
context_id:
context_hash:
generated_at:
expires_at:
operator_id:
trusted_local_datetime:
utc_instant:
iana_timezone:
utc_offset:
tzdb_version:
local_calendar_date:
local_day_of_week:
working_window_status: inside | outside | unknown
operator_availability_status: available | constrained | unavailable | unknown
known_commitment_refs: []
top_deadline_refs: []
market_or_domain_calendar_refs: []
calendar_source_versions: []
relative_time_resolution_policy_ref:
calendar_visibility: connected | partial | not_connected
clock_confidence:
privacy_profile_ref:
data_classification:
retention_policy_ref:
```

Rules:

- `today`, `tonight`, `tomorrow`, `this week`, `EOD`, `next`, `soon`, and similar language resolve against this context and the operator's stated convention.
- Ambiguous relative time that affects action, scheduling, expiry, authorization, or prioritization requires clarification and a durable typed resolution. Fixed-instant semantics record an absolute instant; civil/floating/recurring semantics preserve CivilTimeIntent plus the current uncertainty-bearing projection.
- When calendar integration is absent, Focusa uses trusted local date/time plus explicitly known deadlines and visibly reports `calendar_visibility=not_connected`; it never invents appointments or availability.
- Operator location/timezone change invalidates dependent relative/floating-time resolutions but never silently moves an already fixed-instant deadline or an external civil commitment.
- Human calendar context is privacy-minimized; ordinary agents receive availability/constraint projections rather than unrelated event details.
- Every interaction, consequential tool/action preflight, execution intent, checkpoint, and Receipt carries `human_calendar_context_ref`, `temporal_execution_guard_ref`, or an explicit reason-coded unavailable posture. Reducer commands and low-latency records that do not semantically need private calendar detail carry the signed bounded context hash/guard ref and policy version, not an unnecessary calendar payload.
- Crossing midnight, a working-window boundary, market session boundary, or calendar revision during long work triggers refresh before the next action.

```yaml
schema: focusa.temporal_priority_frame.v1
frame_id:
generated_at:
expires_at:
current_ask_id:
current_ask_revision:
human_calendar_context_ref:
time_awareness_ref:
temporal_execution_guard_ref:
trusted_now:
clock_uncertainty_ns:
elapsed_summary:
readiness_target_at:
operator_deadline_at:
deadline_boundary_posture:
safety_margin_remaining_ms:
schedule_slack_status:
schedule_slack_quantile:
deadline_conflict_ref:
evidence_confidence:
temporal_pressure:
last_material_progress_at:
no_progress_elapsed_ms:
critical_path_ref:
critical_next_action:
active_time_risks: []
known_opportunity_costs: []
top_approaching_deadlines: []
deadline_inquiry_required:
prohibited_time_waste_patterns: []
required_replan_condition:
```

### Mandatory use

Before every plan, model turn, tool/retry/mutation decision, expensive read, research branch, test/build command, browser action, consequential dispatch, checkpoint, compaction, handoff, continuation, status report, forecast, and final response, the active actor MUST either:

- possess a fresh frame matching current ask, scope, Workpoint, deadline, and state revision;
- possess a valid pre-authorized local TemporalExecutionGuard for the exact deterministic operation; or
- perform only the bounded recovery required to refresh temporal authority.

A cached frame can satisfy multiple bounded operations until operation-class expiry or invalidation. The policy declares maximum staleness per operation class, local validation requirements, and no-recursion bootstrap behavior for temporal repair itself. Use is mandatory and auditable, but the actor cannot satisfy this rule by merely receiving a frame; planning/action records preserve the `time_awareness_ref` or guard ref and temporal decision outcome.

### Decision requirements

Every candidate action is evaluated for:

- alignment with the immediate operator ask;
- expected contribution to material progress;
- critical-path relevance;
- readiness/deadline effect;
- protected-margin consumption;
- known opportunity cost;
- expected operator attention/review burden;
- narrower/faster evidence-equivalent alternatives;
- repetition of an already unsuccessful or completed route;
- cancellation and recovery behavior;
- safety, authority, evidence, and settlement obligations.

An action with no defensible operator-ask, progress, risk-reduction, reconciliation, or settlement value must not consume wall-clock time merely because a tool is available.

### Freshness and invalidation

The frame invalidates on:

- operator steering or ask revision;
- project/continuity/Workpoint/item/task change;
- deadline or calendar revision;
- material-progress event;
- pressure-level transition;
- newly discovered blocker or dependency;
- authority, permission, lease, risk, model, provider, or capability change;
- long-running tool completion/failure/timeout;
- compaction, handoff, fork, resume, restart, sleep/wake, or clock correction;
- expiry.

Missing, stale, mismatched, or unverifiable temporal priority blocks new durable work, consequential dispatch, estimate display, and autonomous continuation. Safe read-only temporal repair, reconciliation, cleanup, and operator notification remain allowed.

### Prompt and context placement

Pi/agent bootstrap and every turn packet MUST render a bounded, visible priority header in this order:

```text
CURRENT OPERATOR ASK
TIME NOW / ELAPSED / READINESS TARGET / DEADLINE / PRESSURE
LAST MATERIAL PROGRESS / NO-PROGRESS AGE
CRITICAL NEXT ACTION / KNOWN TIME RISKS
```

Bloatgaurd, token pressure, output truncation, compaction, tool-output flood, awareness ranking, or model-provider adaptation MUST NOT omit this header. Details may move behind handles; the core time facts, freshness, and pressure cannot.

### Conversational temporal behavior

The agent MUST mention time in every response where time materially affects interpretation or action, including:

- task start, progress, pause, resume, handoff, completion, settlement, or final report;
- plans, prioritization, sequencing, critical-path choices, and tradeoffs;
- estimates, delivery questions, `when`/`how long` questions, and schedule-risk discussions;
- long-running tool/process status;
- deadline creation/change, readiness-margin consumption, warning, breach, or missed opportunity;
- blocked work whose wait consumes calendar time;
- any response issued while temporal pressure is `at_risk`, `critical`, or `expired`.

A relevant response uses trusted facts and includes only the needed subset of:

```text
current local time and timezone
elapsed wall-clock and active/proof/blocked posture
readiness target and external deadline
time/safety margin remaining
last material progress and no-progress age
top approaching deadlines and priority order
estimate grounding/uncertainty
time-critical next action
```

The agent MUST NOT fabricate an ETA merely because time must be mentioned. When duration is ungrounded, it reports observed time/deadline facts and explicitly refuses a forecast.

Absolute disclosure governs temporal communication: material uncertainty, assumptions, missing/conflicting evidence, stale/degraded state, failed/unavailable tools, unverified proof, possible external effects, and inference boundaries are stated in direct language. Clear language includes a coherent description of the conclusion, evidence, assumptions, plausible alternatives, consequences, and what would change the conclusion; it is not a terse certainty label.

Confidence percentages are permitted and encouraged when useful, but each percentage identifies what it measures and its basis. `Inference confidence`, `calibrated future-event probability`, `forecast interval coverage`, and `verified work-completion percentage` are distinct and cannot substitute for one another. A judgmental confidence percentage is labeled judgmental; it does not become a calibrated probability or authorize an unsupported duration estimate.

### Deadline inquiry

When an operator asks the agent to begin, plan, prioritize, sequence, estimate, schedule, or perform consequential work and no applicable deadline/readiness target is known, the agent SHALL ask a concise deadline question when the answer can materially change execution:

> What is the external deadline, timezone, and required readiness/review margin for this work?

The agent may continue safe reversible discovery while awaiting the answer only when policy permits and the missing deadline cannot cause harmful prioritization. It must not repeatedly ask once the operator has answered, explicitly stated there is no deadline, or supplied a governing calendar policy.

A deadline inquiry is not necessary for trivial read-only answers where timing cannot affect the action, but the agent remains internally time-aware. If an unknown deadline creates material risk, durable/consequential work pauses or proceeds only under an explicit bounded policy.

### Top approaching deadlines and prioritization

Temporal awareness MUST include a bounded ranked set of the most important approaching readiness targets and external deadlines across the active authorized scope, not only the current item.

Ranking considers:

1. immediate operator ask and explicit priority;
2. hard/external deadline and readiness target proximity;
3. negative/unknown slack and critical-path dependency impact;
4. consequence severity and reversibility;
5. blocked downstream work and opportunity-window risk;
6. required review/verification/delivery margin;
7. task aging and prior commitment;
8. confidence and freshness of the temporal data.

The Work Loop, Trajectory, Mission Canvas, Pi, TUI, and menubar consume the same ranking. If the operator asks for lower-ranked work while a critical deadline is endangered, the agent obeys valid steering but MUST surface the conflict, consequence, and safer sequencing option. It never silently changes the operator's ask.

### Runtime continuity

A model cannot literally receive new prompt tokens continuously during one inference or blocking tool call. Therefore continuous awareness is jointly implemented by:

- fresh prompt-bound TemporalPriorityFrame at every model/decision boundary;
- daemon monotonic watchdog during inference/tool/process execution;
- progress heartbeat and cancellation for long operations;
- SSE/event updates to operator surfaces;
- forced refresh immediately after the operation returns or state changes.

This combined mechanism—not model memory—satisfies continuous temporal awareness.

## Temporal claim types, Estimate Claim, and conversational response gate

A duration or completion-time statement is a probabilistic proposal, never a fact merely because a model phrases it fluently. Forecasts are not operator expectations, deadlines, readiness targets, commitments, budgets, or observed progress.

```yaml
schema: focusa.temporal_claim_envelope.v1
claim_id:
claim_kind: measured_forecast | operator_expectation | external_deadline | readiness_target | commitment | execution_budget | observed_progress
source_principal_ref:
source_record_ref:
created_at:
epistemic_status: measured | operator_supplied | externally_imposed | verified_fact | insufficient_evidence
authority_ref:
may_drive_enforcement:
may_drive_forecast_metrics:
display_label:
provenance_refs: []
```

An operator expectation may be displayed with explicit provenance but does not become a Focusa forecast or deadline. A deadline constrains action but does not predict success. A budget caps resources but does not predict duration. A phrase such as `nearly done` or `most of the work` routes to a verified ProgressClaim when it describes evidence-backed scope state; it routes to forecast refusal when it implies completion time without grounding.

```yaml
schema: focusa.estimate_claim.v1
estimate_id:
claim_envelope_ref:
proposal_ref:
subject_ref:
target_state:
completion_policy_ref:
scope_revision:
deadline_revision:
created_at:
expires_at:
requested_by:
estimator_id:
estimator_version:
history_dataset_ref:
comparable_task_refs: []
comparison_features: []
sample_count:
censored_sample_count:
failed_sample_count:
reopened_sample_count:
quantiles_ms: {}
p50_ms:
p80_ms:
p95_ms:
p99_ms:
range_low_ms:
range_high_ms:
interval_coverage_probability:
cohort_definition_ref:
cohort_sample_count:
baseline_forecast_ref:
correlation_model_ref:
censoring_method_ref:
observation_error_ref:
calibration_profile_ref:
confidence: insufficient | low | medium | high
confidence_policy_ref:
grounding_status: measured_history | deterministic_deadline_arithmetic | mixed_with_operator_prior | insufficient_evidence
assumptions: []
dependencies: []
excluded_time_categories: []
uncertainty_reasons: []
invalidated_by_ref:
status: proposed | verified | displayable | invalidated | expired | evaluated | refused
verification_bundle_ref:
actual_target_event_ref:
actual_elapsed_ms:
calibration_evaluation_ref:
```

```yaml
schema: focusa.forecast_calibration_profile.v1
profile_id:
target_state_class:
risk_policy_ref:
required_quantiles: []
proper_scoring_rule_refs: []
coverage_targets: []
reliability_buckets: []
bias_metric_ref:
sharpness_metric_ref:
skill_baseline_ref:
decision_value_policy_ref:
minimum_sample_count:
rare_event_policy_ref:
cohort_drift_policy_ref:
error_bound_method_ref:
```

Forecast evaluation reports calibration/reliability, bias, interval coverage, sharpness, skill against a declared baseline, decision value under asymmetric early/late costs, sample size, cohort drift, observation/target uncertainty, and error bounds. A single untyped `calibration_score` or average absolute error is insufficient for promotion or operator trust.

### Required estimate target

Every estimate MUST identify what it predicts, such as:

- first material result;
- implementation complete;
- focused tests passing;
- required proof complete;
- provider closure reconciled;
- required settlement recorded under the approved local target-state protocol;
- Receipt committed;
- deployed and externally verified.

“How long?” without target-state resolution requires clarification or explicitly bounded alternatives.

### Grounding policy

- Point estimates are prohibited when evidence supports only a range.
- Completed-only history may support closure velocity but MUST NOT be the sole forecast dataset.
- Forecast history includes relevant failed, abandoned, blocked, reopened, rolled-back, and right-censored attempts to prevent survivorship bias.
- Comparable-task selection records stack, task family, novelty, proof class, dependency depth, environment, model/agent, platform, scope size, and outcome target.
- Scope, acceptance, dependency, target-state, deadline, or material environment change invalidates the estimate.
- A budget, task count, model intuition, prose complexity, or current clock alone cannot ground a duration forecast.
- Operator-provided expectations are separate TemporalClaimEnvelopes, labeled as operator-provided, and cannot masquerade as measured Focusa forecasts. An operator prior may contribute to a mixed model only when its effect is disclosed and measured evidence remains separately visible.
- Correlated dependency paths use a declared correlation/simulation model; independent ranges are not naively summed.
- Estimates expire and must be evaluated against the exact target event.

### Response enforcement

All response surfaces MUST route forecast-shaped output through one daemon-owned validator and progress-shaped output through the canonical progress validator. Numeric and qualitative forecast claims include `soon`, `quick`, `a little while`, `a few hours`, `should finish today`, and equivalent phrasing. `Nearly done` and `most of the work` require verified ProgressClaim evidence or are refused; they are not automatically treated as duration forecasts.

When grounding is insufficient, the canonical replacement is:

> Focusa lacks sufficient measured wall-clock evidence for a grounded duration estimate.

The response MAY still provide verified remaining scope, dependencies, completed/remaining item counts, deadline facts, and the exact next action. It MUST NOT translate those into invented time.

The validator returns a typed reason such as `estimate.insufficient_history`, `estimate.target_ambiguous`, `estimate.scope_stale`, `estimate.packet_expired`, or `clock.authority_unavailable`.

## Material progress contract

```yaml
schema: focusa.material_progress.v1
progress_id:
subject_ref:
workpoint_item_ref:
phase:
recorded_at:
progress_kind: verified_delta | uncertainty_resolved | blocker_removed | proof_advanced | canonical_decision | reconciliation_advanced | settlement_advanced
result_summary:
evidence_refs: []
state_revision_before:
state_revision_after:
target_state_advancement:
verified_by:
verification_policy_ref:
correlation_id:
```

The following do not establish material progress alone:

- tokens consumed;
- time elapsed;
- model narration;
- reading or searching;
- process liveness;
- compilation started;
- a command returned zero without applicable proof;
- file modification without a verified relevant delta;
- repeated test execution without changed evidence;
- a provider status or agent completion message.

## No-progress, waste, and lost-time governance

### Detection

The daemon MUST detect bounded patterns including:

- unchanged content reread without a stated decision purpose;
- semantically equivalent tool attempts with the same result class;
- planning loops without execution or uncertainty reduction;
- research without a bounded question, decision target, evidence target, budget, and exit condition;
- repeated full test matrices when narrower proof is sufficient;
- avoidable clean rebuilds or cache destruction;
- silent subprocesses without progress heartbeat;
- repeated compaction/reorientation;
- duplicated handoff work;
- optional polish after acceptance is already proven;
- resource-lock or provider-wait time that could permit unrelated ready work.

Detection uses content hashes, normalized action intent, target refs, result classes, material-progress events, and policy—not naive raw call counts.

### Incident records

```yaml
schema: focusa.lost_time_incident.v1
incident_id:
subject_ref:
detected_at:
interval_start:
interval_end:
wall_clock_lost_ms:
classification: avoidable | external | operator_wait | contention | uncertainty | recovery | unknown
cause_code:
action_refs: []
progress_refs: []
deadline_refs: []
opportunity_risk_refs: []
material_impact:
detection_delay_ms:
recovery_action_ref:
prevention_candidate_ref:
verification_status: proposed | verified | disputed | unknown
settlement_ref:
```

Focusa MUST distinguish:

- `OpportunityRisk` — a known window may be lost;
- `MissedOpportunity` — evidence proves a real window or commitment was missed;
- `CounterfactualUnknown` — claimed impact cannot be proven.

Models may propose these classifications but cannot canonically accuse, quantify impact, or invent counterfactual damage.

### Operational consequences

Verified repeated waste or deadline failure MAY, according to registered policy:

- force a concise replan;
- suppress optional research;
- narrow permitted tools/context;
- require a different route, model, or adapter;
- lower autonomous turn/research budgets;
- pause the current item and select unrelated ready work;
- require operator review;
- reduce autonomy for the affected task family;
- block procedural-learning promotion;
- create a remediation Workpoint Item;
- remain visible in completion and settlement Receipts.

These consequences are the system's operational analogue of pain. They are evidence-based, auditable, reversible through governed policy, and never emotional punishment.

## Temporal pressure and urgency policy

```text
normal   — readiness margin is evidenced healthy, or no applicable deadline/risk is known after required inquiry; unknown slack is never silently treated as positive
watch    — protected margin is being consumed or no-progress threshold approached
at_risk  — readiness target likely endangered, possibly crossed, or definitely crossed
critical — external deadline is near or indeterminate with unresolved critical-path obligations
expired  — operator deadline is definitely crossed under its boundary/uncertainty policy or authoritative external evidence; a merely straddling interval remains critical/indeterminate
```

Policies may configure `max_no_progress_ms`, `max_research_without_delta_ms`, `max_document_rereads`, `max_equivalent_tool_attempts`, `max_same_subproblem_ms`, `max_operator_silence_ms`, deadline warning thresholds, and required replanning behavior.

Urgency SHOULD:

- prioritize acceptance-critical files, failing checks, current diffs, and exact next actions;
- favor bounded surgical reads and tests;
- parallelize independent proof where safe;
- preserve the best verified partial outcome;
- increase checkpoint frequency near deadlines;
- defer optional polish and unrelated research;
- provide bounded operator-visible updates.

Urgency MUST NOT:

- weaken safety, scope, approval, security, evidence, reconciliation, accessibility, or closure gates;
- hide uncertainty or lateness;
- skip required stages of the approved local settlement protocol, including proposed Spec 136 stages only after activation;
- encourage blind retry after possible external effect;
- prevent cleanup, reconciliation, compensation, or settlement after expiry.

## Temporal pulse, urgency control, prediction, and metacognitive improvement

The agent must remain under real temporal pressure without relying on simulated emotion. Focusa represents the practical function of anxiety—awareness that an irreversible window is narrowing—as a typed `TemporalUrgencySignal` that has runtime consequences.

### Temporal pulse policy

```yaml
schema: focusa.temporal_pulse_policy.v1
policy_id:
normal_interval_ms:
watch_interval_ms:
at_risk_interval_ms:
critical_interval_ms:
max_clock_sample_age_ms:
max_progress_sample_age_ms:
operator_update_interval_ms:
long_tool_heartbeat_interval_ms:
pressure_transition_thresholds:
pressure_exit_thresholds:
minimum_dwell_ms:
minimum_pulse_interval_ms:
debounce_ms:
deduplication_window_ms:
notification_budget_ref:
replan_thresholds:
notification_policy_ref:
```

At each pulse the daemon evaluates:

- trusted current wall and monotonic time;
- HumanCalendarContext and top deadlines;
- readiness margin and external-deadline distance;
- grounded/unknown critical-path slack;
- elapsed time in current action/subproblem;
- time since last material progress;
- active tool/process heartbeat and cancellation posture;
- repeated reads/actions/retries and research budget;
- unresolved blockers, operator wait, and resource contention;
- estimate invalidation and forecast deviation;
- opportunity-window risk;
- evidence, reconciliation, settlement, safety, and authority obligations.

Pulse intervals tighten as pressure rises, within resource and precision policy. Sampling must not itself become waste: the daemon uses efficient timers/events and pushes bounded changes; the agent must not repeatedly invoke shell clock commands to imitate awareness.

### Urgency signal

```yaml
schema: focusa.temporal_urgency_signal.v1
signal_id:
generated_at:
time_awareness_ref:
pressure: normal | watch | at_risk | critical | expired
primary_reason_codes: []
readiness_margin_status:
critical_path_status:
no_progress_status:
opportunity_risk_refs: []
required_behavior_changes: []
prohibited_behavior: []
required_next_decision_at:
operator_notification_required:
```

Required behavior changes may include narrower context, shorter progress intervals, cancellation of low-value work, forced replan, critical-path selection, evidence-first execution, best-verified-partial preservation, or operator escalation. The signal is calm and factual in UX but impossible for the execution policy to ignore.

### Calm focus gradient

The agent MUST remain calm under pressure. Urgency is expressed through disciplined execution, not emotional language, panic, rushed reasoning, repeated status noise, paralysis, or reckless action.

| Pressure | Required agent posture |
| --- | --- |
| `normal` | concise planning, bounded discovery, explicit acceptance target, efficient execution |
| `watch` | reduce optional breadth, stop redundant reading, verify critical path, increase progress checks |
| `at_risk` | execution-first mode, freeze nonessential scope, prefer smallest evidence-producing action, surface conflict |
| `critical` | one primary critical objective plus explicit non-preemptible obligations; no speculative research or cosmetic work; immediate bounded proof/checkpoint cadence; preserve best verified partial |
| `expired` | immediately assess remaining delivery opportunity, freeze unrelated optional work, execute the smallest valid overdue-delivery path calmly, or reconcile/contain/settle if the window is closed/harmful |

At every level the agent:

- communicates clearly and factually;
- preserves scope, security, authority, evidence, reconciliation, and accessibility;
- avoids panic-driven tool switching and repeated replanning;
- does not conceal uncertainty, bad news, lateness, or missing proof;
- does not flood the operator with repetitive warnings;
- continues decisive safe execution rather than narrating urgency;
- asks only questions that materially affect authority, deadline, critical path, or outcome;
- returns to the exact critical action after interruptions.

As pressure rises, context breadth and narration generally decrease while action specificity, verification cadence, and critical-path adherence increase. A protected checklist preserves safety, security, authority, required acceptance/proof, reconciliation, disconfirming evidence, accessibility, and stop-work/escalation authority at every pressure level. High-consequence profiles may require independent or two-person review. The system tracks sustained critical duration, handoff freshness, operator/agent workload posture, and fatigue/attention limits; it escalates to a fresh reviewer when policy requires.

Pressure transitions use hysteresis, minimum dwell, debounce, deduplication, backpressure, and notification budgets so the pulse cannot flap, overload the daemon, or harass an unavailable operator. `max_operator_silence_ms` is subordinate to availability, quiet-hours, severity, consent, and escalation policy.

This focus gradient is deterministic policy, observable in traces, and covered by evals; it is not left to personality prompting.

### Temporal prediction loop

Before a nontrivial route/tool/action, Focusa records a bounded prediction when sufficient evidence exists:

- expected target-state advancement;
- expected time category/range, if grounded;
- expected proof/result;
- probability of material progress;
- deadline/slack effect;
- likely blocker/failure class;
- recommended stop/replan condition;
- alternative route considered.

After the action or checkpoint, Focusa evaluates:

- actual elapsed and category breakdown;
- actual material progress;
- prediction error/calibration;
- deadline/margin impact;
- avoidable waste;
- whether the stop/replan condition fired at the correct time;
- whether a different route would have been safer or faster, without fabricating counterfactual certainty.

No-history cases still permit a non-duration hypothesis about result/risk, explicitly marked low-confidence; they do not permit invented numeric time.

### Temporal metacognition loop

```text
TemporalPriorityFrame
→ bounded action/route prediction
→ execution with temporal pulse
→ material-progress and elapsed observation
→ prediction evaluation
→ lost-time/route-quality reflection
→ scoped LearningCandidate
→ fixed eval/holdout comparison
→ governed promotion or rejection
→ versioned policy/routing update
→ future outcome evaluation and rollback if worse
```

Metacognitive reflection is required after significant overrun, deadline breach, repeated no-progress incident, avoidable rebuild, duplicate research, cancellation failure, gross estimate error, or unusually effective route. It identifies reusable causes and interventions—not raw transcript blame.

Learning rules:

- one event cannot rewrite global policy;
- only settled outcomes with causal Evidence can support promotion;
- reflections remain advisory until the approved local settlement/learning governance accepts a LearningCandidate;
- eval definitions and holdouts cannot be modified by the candidate being evaluated;
- changes are scoped by project/task family/tool/model/environment/precision profile;
- promotion requires improved time-to-verified-outcome without quality, safety, accuracy, or operator-attention regression;
- every promoted change is versioned, observable, reversible, and re-evaluated;
- failed strategies and negative outcomes remain in forecast/training history;
- the agent cannot suppress incidents or select only favorable examples.

### Progressive temporal performance

Focusa reports improvement using measured settled outcomes:

- reduced time to first material progress;
- reduced no-progress and duplicated-work time;
- improved estimate calibration;
- earlier readiness-target settlement;
- fewer deadline breaches and missed windows;
- lower tail latency and uncertainty in high-consequence paths;
- lower operator review/interruption burden;
- equal or better evidence, safety, reconciliation, and settlement quality.

Raw speed, token reduction, shorter narration, or more completed micro-items alone cannot establish improvement.

## Expensive action and long-running tool contract

Before an expensive operation, the executor records:

```yaml
schema: focusa.temporal_preflight.v1
operation_ref:
subject_ref:
time_awareness_ref:
critical_path_relevance:
necessity:
narrower_alternative_refs: []
historical_runtime_claim_ref:
remaining_margin_ms:
cancellation_supported:
progress_heartbeat_supported:
timeout_policy_ref:
decision: allow | warn | block | operator_review
reason_codes: []
```

Every long-running tool/process exposes start time, elapsed time, progress heartbeat or explicit silence posture, timeout, cancellation, process-tree cleanup, partial-result capture, and reason to continue. If Focusa lacks grounded duration history, it reports elapsed status without inventing an ETA.

## Integration with Specs 130, 130A, and 131

Spec 130 owns the HLT-aware Compaction Mission Packet and Bloatgaurd Context Firewall. Spec 130A is its true addendum and exclusively owns zero-waste compaction performance, cache-prefix preservation, compaction eligibility and ROI gates, cache telemetry, and cache-safe degradation. Spec 137 consumes their canonical packet timestamps, omission/rehydration evidence, and performance incidents; it does not redefine their packet authority or performance gates.

Spec 131 exclusively owns the Workpoint Item model, Work Timing Ledger, timing categories, token/tool attribution, velocity rollups, and closure authority. Spec 137 consumes those records as evidence for forecasts, urgency, deadline posture, temporal incidents, and post-outcome calibration. It MUST NOT create a second Workpoint timing ledger, velocity definition, or closure state machine.

The integration invariant is:

```text
Spec 130/130A continuity and performance evidence
+ Spec 131 Workpoint timing, velocity, and closure records
+ Spec 137 trusted clocks, deadlines, urgency, and forecasts
+ Spec 136 proposal-to-settlement outcomes when activated
= one linked evidence chain, never duplicate authority
```

Across compaction and resume, Spec 137 temporal records MUST preserve references to the authoritative Spec 130 packet, Spec 130A performance evidence, Spec 131 Workpoint/closure record, required Evidence/ECS handles, HLT posture, and Receipt posture. Transcript tails, generic HLT, cache telemetry, or temporal projections cannot become closure authority.

## Spec 131 closure velocity and Spec 137 forecast history


Focusa MUST maintain separate projections:

- `ClosureVelocitySummary` includes only authorized, evidence-backed, settled outcomes and answers what was actually completed.
- `ForecastHistorySummary` includes successful, failed, abandoned, reopened, blocked, rolled-back, and censored attempts and answers what future work may consume.

Using completed-only velocity as forecast history is prohibited because it creates survivorship bias. Forecast samples preserve target-state identity, scope revision, task similarity features, temporal category breakdown, model/agent/tool identity, environment, interruptions, deadline posture, and outcome class.

Additional temporal metrics include:

- time to first material progress;
- time between material-progress events;
- no-progress time and detection delay;
- readiness-target and deadline miss rates;
- protected margin consumed;
- forecast P50/P80 calibration;
- stale/invalidated estimate rate;
- false-precision rejection count;
- unchanged-reread and equivalent-tool-attempt rate;
- research-to-decision conversion;
- operator attention and review burden;
- time lost by cause;
- missed-opportunity and opportunity-risk counts;
- critical-path throughput;
- queue and resource-contention time;
- compaction/handoff duplication;
- time from temporal breach to recovery;
- quality-adjusted time to settled outcome.

Velocity reports include cohort definition, sample/censored counts, distributions and tails, uncertainty/error bounds, exclusions, policy/schema versions, and comparison baselines. Raw item throughput cannot establish improvement because item split/merge policy is gameable; decomposition-policy revisions are versioned and outcome/target-state-normalized measures govern comparisons. Averages and one scalar `estimate_accuracy` are never sufficient.

## Spec 136 proposal-to-settlement integration

Proposed Spec 136 describes the intended cross-system lifecycle and MUST consume Spec 137 temporal primitives plus Spec 131 timing and closure records when its stated activation conditions are satisfied. Repository presence alone does not satisfy those conditions or any implementation requirement. Integration remains blocked until an approved immutable dependency manifest pins the accepted schema, source commit, document hash, ownership map, migration behavior, and conformance evidence.

### Lifecycle mapping

| Spec 137 object | Spec 136 treatment |
| --- | --- |
| trusted clock sample | deterministic observation; never model proposal |
| deadline set/revision | operator/policy constraint committed through reducer |
| duration estimate | `CognitiveProposal` with `proposal_kind=prediction` and EstimateClaim payload |
| estimate verification | `VerificationBundle` over history, target, scope, assumptions, and calibration |
| estimate display | advisory projection; never canonical completion fact |
| material progress | evidence-backed observation/reducer event |
| no-progress incident | deterministic policy observation; may propose replan |
| temporal preflight | Governance Context/AuthorityDecision input |
| deadline breach | canonical reducer event |
| overdue opportunity assessment | verified post-breach observation feeding Work Loop/AuthorityDecision |
| overdue delivery mode | canonical execution-priority constraint until settlement/redirection/window closure |
| missed opportunity | outcome fact only after evidence verification |
| lost-time incident | completion/settlement evidence and Receipt lineage |
| forecast evaluation | outcome verification against exact target event |
| process/routing improvement | post-settlement `LearningCandidate` |

### Required Spec 136 integration points

If Spec 136 is locally adopted through the immutable dependency gate, its integrations SHALL include:

- normative basis reference to Spec 137;
- core laws that time is nonrenewable, budgets are not forecasts/deadlines, activity is not progress, and urgency does not grant authority;
- Governance Context `temporal` references rather than an opaque `budgets.wall_clock` field;
- time/deadline/estimate/progress/opportunity reason-code domains;
- deadline revision and TimeAwareness references on AuthorityDecision and ExecutionIntent;
- immediate pre-dispatch deadline/freshness validation;
- attempt, reconciliation, completion, settlement, and Receipt timing refs;
- temporal reducer event families;
- Operation Registry temporal descriptors;
- Work Loop/Silent Session binding to deadline revision;
- temporal metrics, perturbations, conformance scenarios, task decomposition fields, and release gates.

Deadline expiry may block new optional dispatch but MUST allow reconciliation of possible effects, compensation, security cleanup, process termination, evidence capture, completion evaluation, and settlement. The protocol must preserve the difference between functional success and temporal failure: an outcome can succeed and still settle late with a temporal breach.

### Spec 136 reason codes

At minimum:

```text
clock.authority_unavailable
clock.skew_detected
clock.uncertainty_exceeded
precision.profile_unsupported
precision.calibration_missing
time.packet_stale
time.pulse_stale
time.attribution_degraded
urgency.required_behavior_ignored
urgency.focus_thrash_detected
urgency.excessive_operator_alerting
deadline.at_risk
deadline.critical
deadline.expired
deadline.revision_conflict
estimate.ungrounded
estimate.insufficient_history
estimate.target_ambiguous
estimate.scope_invalidated
estimate.expired
prediction.temporal_uncalibrated
metacog.temporal_source_not_settled
progress.not_material
progress.stalled
research.budget_exhausted
tool.runtime_exceeded
opportunity.at_risk
opportunity.window_missed
opportunity.delivery_still_valid
opportunity.partial_delivery_only
opportunity.late_delivery_harmful
opportunity.status_unknown
overdue.delivery_mode_required
overdue.unrelated_work_blocked
lost_time.avoidable
lost_time.external
data.stale
data.sequence_gap
intent.temporally_expired
market.session_closed
market.trading_halted
risk.limit_exceeded
execution.kill_switch_active
```

After approved local Spec 136 adoption, these map to its accepted `focusa.protocol_block.v1` envelope with temporal refs, safe next operation, reconciliation posture, and operator review route. Until then, that remote envelope name is informative only and cannot authorize a local mutation or conformance claim.

## Cross-system coherence requirements

Spec 137 is incomplete unless every applicable integration below is implemented and proven.

| System/spec | Required integration |
| --- | --- |
| Spec 55 tool contracts | propagated remaining deadline, cancellation token/acknowledgement, heartbeat, elapsed, bounded retry budget, cleanup, cost provenance, progress result |
| Spec 56 trace/recovery | temporal refs in checkpoint, replay, recovery, and corrections |
| Spec 66 ontology | canonical temporal object/action/relation types |
| Spec 67 relevance | deadline/critical-path/information-gain/reread-aware relevance |
| Spec 78 autonomy | bounded cognition/research and temporal stop-loss |
| Spec 79 Work Loop | deadline, watchdog, critical path, no-progress, replan, expiry behavior |
| Spec 88 Workpoint | items, timing, deadline, estimate, progress, incident refs in resume packet |
| Specs 96/102 Trajectory | milestone constraints, aging, risk, critical path without replacing goal authority |
| Spec 97 reflexes | stall, deadline, stale estimate, repeated action, silent tool reflexes |
| Spec 98 CRDT | scoped convergence, idempotency, interval reconciliation, no double-counting for replicated/portable projections; canonical mutation remains reducer/CAS/fencing-owned and CRDT cannot invent authority |
| Spec 100 Context Cognition | temporal optimization frame and critical-next context |
| Spec 101 Bloatgaurd | reread/context/review/compaction human-time budgets |
| Spec 103 architecture | one temporal call stack and ownership map |
| Spec 104 scoped runtime | no global timer/deadline/timezone/estimate singleton |
| Spec 105 DX/UX | typed temporal failures and exact recovery |
| Spec 106 vision | constitutional temporal doctrine |
| Spec 107 lifecycle | instrumentation, target, deadline, incident, estimate evaluation |
| Specs 108/110/111 | awareness, reminder, preload delivery and freshness |
| Specs 109/124 | agent-first API and CLI parity |
| Specs 113/114 | quality-adjusted benchmarks and public-safe provenance |
| Specs 116/119 | exact completion target, closure, Receipt temporal lineage |
| Spec 120 workbench | adversarial clock/deadline/estimate/anti-gaming review |
| Spec 122 self-heal | verified lost-time patterns drive systemic remediation |
| Spec 125 interlock | time pressure cannot bypass HLT/receipt/ontology authority |
| Spec 130 compaction | preserve deadline, elapsed, progress, reread hashes, estimate and incident refs |
| Spec 133 Silent Sessions | run timing, deadline, process progress, timeout, recovery, settlement |
| Spec 135A/135K | Mission Canvas/work rail temporal projection and human-friction learning |
| proposed Spec 136 | governed temporal lifecycle through settlement and learning only after an approved immutable local dependency contract pins schema/commit/hash; remote prose alone is blocked and non-conformant |
| Project Card | canonical ledger timing; remove turn-local timing as estimate authority |
| Prediction/metacognition | estimate calibration and settled lost-time learning |
| API/CLI/Pi/TUI/menubar | common canonical routes, gate, display, errors, and parity |
| updates/install/release | truthful progress, windows, cancellation, no fabricated ETA |
| pairing/license/security TTL | shared clock substrate but distinct security semantics |

The complete feature ledger must assign every row to owned code, generated contracts, clients, tests, Evidence, and Receipt. Cross-reference text without implementation is not conformance.

## Mandatory timeline projection and future SaaS renderer

The canonical timeline-ready API projection is mandatory in this specification and cannot be deferred. A separately shipped SaaS renderer may consume it later only as an explicitly separate product surface. The projection hierarchy is:

```text
Project
 └─ Spec
    └─ Bead / Task
       └─ Workpoint
          └─ Workpoint Items
             ├─ audit
             ├─ design
             ├─ implementation
             ├─ tests
             ├─ proof
             └─ closure
```

Timeline cards should show:

- elapsed active vs blocked time;
- token burn;
- tool calls;
- files changed;
- commits;
- tests/proofs;
- closure authority;
- agent handoffs;
- compactions/session resumes;
- predictions vs actual outcomes;
- Workpoint lineage;
- CompactionMissionPacket boundaries;
- HLT warning intervals;
- Bloatgaurd omitted-context/rehydration events;
- receipt/closure-gate transitions;
- readiness target and external deadline;
- safety margin consumption;
- material-progress events and no-progress intervals;
- estimate claims, invalidations, and actual outcomes;
- temporal breaches, opportunity posture, and lost-time incidents;
- precision profile, clock confidence, and uncertainty for high-consequence operations.

The canonical timeline projection must answer now:

- Where did this task stall?
- Which item burned most tokens?
- What proof closed this work?
- Was this estimate accurate?
- Which agent/session did what?
- What changed between Workpoints?
- Which specs consume the most implementation time?

## CLI surface

```bash
# Trusted time and awareness
focusa time now --json
focusa time status --workpoint <id> --json
focusa time status --task <bead> --json
focusa time trust inspect --json
focusa time samples list --host <id> --json
focusa time capabilities --host <id> --json
focusa time watch --workpoint <id>
focusa time doctor --json

# Deadline authority
focusa deadline set --subject <ref> --at <rfc3339> --timezone <iana> \
  --readiness-target <rfc3339> --completion-target <policy-ref> --confirm
focusa deadline set-civil --subject <ref> --local <yyyy-mm-ddThh:mm:ss> \
  --timezone <iana> --fold-policy <policy> --gap-policy <policy> \
  --calendar <ref> --completion-target <policy-ref> --confirm
focusa deadline inspect <deadline-id> --json
focusa deadline resolve-civil <deadline-id> --tzdb-version <version> --json
focusa deadline conflicts --project <root> --json
focusa deadline revise <deadline-id> --expected-revision <n> --reason "..." --confirm
focusa deadline clear <deadline-id> --expected-revision <n> --reason "..." --confirm
focusa deadline list --project <root> --json
focusa temporal guard inspect <guard-id> --json
focusa cancellation inspect <cancellation-id> --json

# Estimates and calibration
focusa estimate request --subject <ref> --target-state <state> --json
focusa estimate inspect <estimate-id> --json
focusa estimate validate <estimate-id> --json
focusa estimate evaluate <estimate-id> --actual-event <ref> --json
focusa estimate history --task-family <family> --include-censored --json

# Progress and temporal incidents
focusa progress record --item <id> --kind <kind> --evidence <ref>
focusa progress status --item <id> --json
focusa no-progress inspect --item <id> --json
focusa lost-time list --subject <ref> --json
focusa lost-time inspect <incident-id> --json
focusa opportunity inspect <ref> --json

# Spec 131-owned Workpoint timing and closure integration surfaces (not redefined by Spec 137)
focusa workpoint item create --workpoint <id> --task <bead> --title "..."
focusa workpoint item list --workpoint <id> --json
focusa workpoint item start <item-id>
focusa workpoint item pause <item-id> --reason blocked
focusa workpoint item resume <item-id>
focusa workpoint item complete <item-id> --evidence <ref>
focusa workpoint item close-check <item-id> --json
focusa work timing status --workpoint <id> --json
focusa work velocity --project <root> --json
focusa task closure check <task-id> --json
```

CLI human and JSON modes MUST be semantically identical. Mutation commands require exact scope, authenticated principal, expected revision where applicable, preview, explicit confirmation, stable reason codes, and canonical API use. CLI cannot compute or store independent deadlines or estimates.

## API surface

Canonical routes:

- `GET /v1/time/now`
- `GET /v1/time/awareness`
- `GET /v1/time/status`
- `GET /v1/time/trust`
- `GET /v1/time/samples`
- `GET /v1/time/capabilities`
- `GET /v1/time/stream` (SSE)
- `POST /v1/deadline/set`
- `POST /v1/deadline/revise`
- `POST /v1/deadline/clear`
- `GET /v1/deadlines`
- `GET /v1/deadline/:id`
- `POST /v1/deadline/resolve-civil`
- `GET /v1/deadline/conflicts`
- `POST /v1/deadline/propagate`
- `POST /v1/temporal/guard/issue`
- `POST /v1/temporal/guard/validate`
- `POST /v1/temporal/guard/revoke`
- `POST /v1/cancellation/request`
- `GET /v1/cancellation/:id`
- `POST /v1/estimate/request`
- `POST /v1/estimate/validate`
- `POST /v1/estimate/evaluate`
- `GET /v1/estimate/:id`
- `GET /v1/estimate/history`
- `POST /v1/response/temporal-claims/validate`
- `POST /v1/progress/record`
- `GET /v1/progress/status`
- `GET /v1/no-progress/incidents`
- `GET /v1/lost-time/incidents`
- `GET /v1/opportunities`
- `POST /v1/temporal/preflight`
- `POST /v1/workpoint/item/create`
- `GET /v1/workpoint/items`
- `POST /v1/workpoint/item/start`
- `POST /v1/workpoint/item/pause`
- `POST /v1/workpoint/item/resume`
- `POST /v1/workpoint/item/complete`
- `POST /v1/workpoint/item/close-check`
- `GET /v1/work/timing/status`
- `GET /v1/work/velocity`
- `POST /v1/task/closure/check`

The `/v1/workpoint/item/*`, `/v1/work/timing/status`, `/v1/work/velocity`, and `/v1/task/closure/check` routes are Spec 131-owned integration surfaces listed here only because Spec 137 consumes their canonical records. Spec 137 does not redefine their schemas, operations, or authority.

All routes use generated shared schemas and a common result envelope. Time, deadline, estimate, progress, and incident events are exposed through the native durable event stream. API routes do not create a second scheduler, estimator, clock, timing ledger, closure state machine, or policy engine.

## Pi and agent surface

Pi tools SHALL include bounded equivalents for time awareness/status/trust, civil-deadline resolution and conflict inspection, deadline inspection/mutation, estimate request/validation, typed claim inspection, progress status/recording, temporal preflight, guard inspection, cancellation inspection, and incident inspection. Guard issuance/revocation and deadline mutation retain exact authority/confirmation boundaries and are never inferred from conversational urgency. Mutation tools preserve the same approval and CAS requirements as API/CLI.

Pi turn behavior:

1. resolve the verified immediate operator ask;
2. fetch/build and inject a matching fresh TemporalPriorityFrame immediately after the ask and before all other project context;
3. bind every tool/action/continuation record to the frame used for its temporal decision;
4. route explicit or inferred forecast questions to estimate authority and separately preserve operator expectation/deadline/commitment/budget/progress claim kinds;
5. validate forecast-shaped and progress-shaped final output through their respective canonical validators before display;
6. refresh awareness after long tools, steering, deadline change, Workpoint change, material progress, pressure transition, compaction, resume, and scope change;
7. block durable work/autonomous continuation when the frame is stale or unavailable while permitting bounded temporal repair/reconciliation/cleanup;
8. display temporal authority failure rather than guessing;
9. keep packets compact and place detailed timelines behind rehydrate refs without omitting the priority header.

## Operator UI and notification surface

Mission Deck/Canvas, TUI, menubar, generated UI, and notifications show:

- trusted local current time and timezone source;
- readiness target and external deadline;
- safety margin remaining;
- elapsed category summary;
- last material progress and no-progress age;
- current pressure level;
- critical next action;
- temporal claim kind/provenance plus forecast grounding/confidence/target state where applicable;
- lateness, breach, and opportunity posture;
- cancellation/recovery controls appropriate to authority.

Alerts are deduplicated and escalate through `watch`, `at_risk`, `critical`, and `expired`. Alert fatigue, inaccessible color-only encoding, raw millisecond-only output, decorative success, and hidden breach state are prohibited.

## Storage

Required logical append-only ledgers (SQLite-backed canonical persistence may materialize equivalent tables/outbox/read models; JSONL names describe portable evidence projections):

- `temporal-authority/{host_hash}/clock-events.jsonl`
- `clock-samples/{host_hash}/sample-pairs.jsonl`
- `clock-trust/{host_hash}/profiles-and-reviews.jsonl`
- `clock-capabilities/{host_hash}/capability-evidence.jsonl`
- `civil-time-intent/{scope_hash}/intent-and-resolution-events.jsonl`
- `deadlines/{scope_hash}/deadline-events.jsonl`
- `deadline-propagation/{scope_hash}/propagation-events.jsonl`
- `deadline-conflicts/{scope_hash}/conflict-events.jsonl`
- `cancellations/{scope_hash}/cancellation-events.jsonl`
- `temporal-execution-guards/{scope_hash}/guard-events.jsonl`
- `calendar-constraints/{operator_hash}/constraints.jsonl`
- `human-calendar-context/{operator_hash}/context-receipts.jsonl`
- `time-awareness/{scope_hash}/packet-receipts.jsonl`
- `temporal-priority/{scope_hash}/frame-receipts.jsonl`
- `temporal-pulses/{scope_hash}/pulse-events.jsonl`
- `temporal-urgency/{scope_hash}/signals.jsonl`
- `temporal-learning/{project_hash}/prediction-evaluation-links.jsonl`
- `material-progress/{project_hash}/progress-events.jsonl`
- `temporal-claims/{project_hash}/claim-envelopes.jsonl`
- `estimate-claims/{project_hash}/claims.jsonl`
- `forecast-calibration/{project_hash}/profiles.jsonl`
- `estimate-evaluations/{project_hash}/evaluations.jsonl`
- `no-progress/{project_hash}/incidents.jsonl`
- `lost-time/{project_hash}/incidents.jsonl`
- `opportunity-posture/{project_hash}/events.jsonl`
- `overdue-opportunity/{project_hash}/assessments.jsonl`
- `overdue-delivery/{project_hash}/mode-events.jsonl`
- `temporal-breaches/{project_hash}/breaches.jsonl`
- `forecast-history/{project_hash}/attempts.jsonl`
- `temporal-control-reviews/{scope_hash}/reviews-and-certifications.jsonl`
- `temporal-policy-versions/{scope_hash}/policy-events.jsonl`
- `temporal-outbox/{project_hash}/events.jsonl`

Spec 137 records link to, but do not create or own, the canonical Spec 131 `workpoint-items`, `work-timing`, `work-token-usage`, `closure-authority`, and `closure-velocity` stores or Spec 130/130A compaction records.

All records include a typed `scope_kind`/`scope_id` and only the identifiers applicable to that scope. Host clock/trust records are host-scoped and referenced by project projections; operator calendar records are operator-scoped; project/work records are project-scoped. A missing inapplicable identifier is not an authority fallback, and no host/operator record may be merged across incompatible identities.

Applicable identifiers include:

- `host_id` for host clock/trust/capability records;
- `operator_id` for private calendar/availability records;
- `project_root` and `continuity_id` for project/workstream records;
- `session_id` when available and semantically applicable;
- `workpoint_id` and `item_id` where applicable;
- source route/tool;
- created timestamp plus applicable monotonic epoch/sample;
- clock source, confidence, precision profile, and uncertainty where applicable;
- deadline, estimate, progress, correlation, and causation refs where applicable;
- integer canonical duration units;
- schema version;
- `hlt_status` when closure or durable work is involved;
- `compaction_packet_ref` when record was created before/after compaction;
- Bloatgaurd/ECS rehydrate refs for omitted proof/context.

### Temporal security, privacy, integrity, and retention

Clock/calendar/activity/availability/deadline/market-strategy data is security- and privacy-sensitive. Every ledger and projection declares data classification, least-privilege readers/writers, encryption at rest/in transit, redaction/coarsening policy, export policy, retention/deletion/legal-hold behavior, audit access, and aggregation minimums. Detailed operator calendars are never copied merely to prove temporal grounding; signed bounded context hashes/refs are preferred.

Authoritative ClockSamplePairs, correction events, deadline revisions, TemporalExecutionGuards, cancellation events, closure dispositions, and Receipts are signed or hash-chained under versioned key/provenance policy. Time-source authentication and ledger tamper evidence are separate controls: NTS/authenticated synchronization cannot prove later database integrity, and a signed ledger cannot prove the clock source was accurate.

High-resolution timestamps are coarsened or withheld from untrusted clients/models/public exports when exposure would enable fingerprinting, activity surveillance, venue strategy inference, or side-channel attack. Retention is purpose-limited; deletion/tombstone behavior preserves required audit lineage without retaining unrelated private calendar content.

## Acceptance criteria

### Temporal authority and scope

1. Trusted wall, per-boot monotonic, suspend-aware, and TAI-capability clocks are distinct, typed, health-checked, capability-tested, and linked across restart only through uncertainty-bearing ClockSamplePairs.
2. DST/tzdb/timezone change, NTP step/slew, source disagreement/authentication loss, manual clock correction, sleep/wake, reboot, daemon outage, leap/smear policy, and client/daemon skew cannot create negative, cross-epoch-monotonic, falsely exact, or silently missing elapsed time.
3. Every temporal record is bound to its exact applicable host/operator/project/continuity/Workpoint/item/task scope.
4. Spec 98 reconciliation converges duplicate/concurrent replicated or portable projection records without cross-scope merge, authority invention, or elapsed double-counting; canonical mutation remains reducer/CAS/lease/fencing-owned.
5. Corrections append superseding records; no history is rewritten in place.
6. Temporal authority outage fails closed for estimates and deadline arithmetic with explicit recovery.

### Deadline doctrine

7. Deadline contracts distinguish readiness target, external deadline, safety margin, completion target, and settlement requirement.
8. Under the contract's inclusive/exclusive boundary policy and trusted-time uncertainty, definitely settled before readiness is `on_time`, definitely between readiness and deadline is `late_window`, definitely beyond the deadline boundary is `breached`, and boundary-straddling cases remain `possibly_crossed` or `indeterminate` until reconciled.
9. Pause, compaction, model switch, restart, operator wait, and budget renewal do not move deadlines.
10. Agents cannot set, extend, clear, weaken, or reinterpret deadlines outside an authorized CAS reducer operation; no operation may imply that Focusa can change an immutable external/legal/regulatory/venue boundary.
11. Deadline inheritance, civil-time resolution, boundary semantics, conflicts/infeasibility, multiple commitments, working windows, uncertainty intervals, and unknown slack are visible.
12. A definitely crossed deadline or authoritative breach evidence creates a TemporalBreach and evidence-backed OverdueOpportunityAssessment; possibly-crossed/indeterminate posture creates visible OpportunityRisk and reconciliation. Confirmed breach pins the smallest valid delivery/partial/alternate path above unrelated optional work and blocks expired/harmful dispatch while preserving reconciliation, compensation, cleanup, evidence, and settlement.

### Estimate truth

13. Every forecast identifies claim kind, target state, scope revision, expiry, estimator version, cohort, evidence basis, comparable/all-attempt samples, censoring/correlation method, uncertainty/coverage, calibration profile, and grounding status.
14. Unsupported numeric and qualitative agent forecasts are blocked across API, CLI, Pi, UI, Work Loop, Silent Session, project card, and generated reports; attributed operator expectations/deadlines/commitments/budgets and verified progress remain displayable as their own claim kinds.
15. Insufficient history produces refusal, never a heuristic task-count conversion.
16. Forecast history includes relevant failed, abandoned, reopened, blocked, rolled-back, and censored attempts.
17. Point estimates are rejected when only ranges are justified.
18. Scope/target/dependency/deadline/material environment changes invalidate affected estimates.
19. Every displayable forecast is evaluated against its exact actual target event for reliability/calibration, bias, coverage, sharpness, skill baseline, decision value, sample uncertainty, and cohort drift.
20. Operator expectations, deadlines, readiness targets, commitments, budgets, forecasts, and observed progress remain separately typed/labeled and cannot masquerade as one another.

### Progress and waste

21. Material progress requires evidence-backed target advancement.
22. Activity-only signals cannot reset no-progress clocks.
23. Unchanged rereads, equivalent actions, unbounded research, repeated full proof, silent tools, compaction churn, and duplicated handoff work are detected with false-positive controls.
24. Long-running processes expose elapsed status, heartbeat/silence posture, timeout, cancellation, cleanup, and partial results.
25. Daemon temporal pulses continuously recompute urgency, apply bounded hysteresis/debounce/dwell/backpressure/notification budgets, trigger the calm protected-focus gradient, and never rely on model memory, flap, overload, harass, create panic/thrashing, or weaken safety/authority.
26. Opportunity risk is distinguished from evidence-proven missed opportunity and unknown counterfactual impact.
27. Lost-time incidents are append-only, reviewable, disputable, settleable, and linked through prediction evaluation, metacognitive reflection, fixed evals, governed LearningCandidates, promotion/rejection, and rollback.
28. Agent/model change, compaction, retry, or process restart cannot erase accumulated incident/prediction history; temporal policy improves only from measured settled outcomes without safety, quality, accuracy, or operator-attention regression.

### Workpoint, closure, and settlement

29. Integration consumes append-only Spec 131 Workpoint Item records without creating or mutating a parallel item lifecycle.
30. Cross-system temporal calculations preserve Spec 131 timing categories and overlap rules while adding trusted clock-epoch and bounded/unknown offline semantics.
31. Forecast and incident calculations consume Spec 131 token/tool attribution and add attention evidence without treating usage as progress.
32. Resume packets preserve canonical Spec 131 active/blocked/next and closure posture while adding Spec 137 deadlines, progress, incidents, and estimates.
33. Task/spec closure remains Spec 131 authority; Spec 137 contributes temporal posture but cannot convert cancellation, accepted risk, variance, abandonment, or scope amendment into factual completion.
34. HLT, Receipt, Bloatgaurd, Evidence, Spec 130/130A compaction posture, Spec 131 closure posture, and Spec 137 temporal posture survive closure and resume.
35. Proposed Spec 136, once activated, can represent functional success with temporal failure; Receipts preserve both without replacing Spec 131 closure authority.
36. Project cards consume canonical Spec 131 and Spec 137 ledgers rather than turn-local timers as estimate authority.

### Cross-system, agent primacy, and UX parity

37. A fresh HumanCalendarContext and TemporalPriorityFrame appear immediately below the verified current operator ask and above all lower-priority context at every interaction/model/decision boundary; deterministic critical paths use only a valid pre-authorized local TemporalExecutionGuard.
38. Every plan, consequential tool/action, retry, mutation, research branch, checkpoint, continuation, forecast, and final response preserves the TimeAwareness or TemporalExecutionGuard reference used for its temporal decision without copying irrelevant private calendar data.
39. Missing/stale/mismatched temporal priority or guard blocks new durable work, consequential dispatch, forecast display, and autonomous continuation while preserving bounded repair/reconciliation/cleanup.
40. Daemon watchdog enforcement continues while the model/tool cannot receive a new prompt.
41. Relevant responses mention trusted time and deadline/progress posture without fabricating an ETA; relative human-calendar phrases resolve to explicit timezone-aware boundaries or trigger clarification.
42. The agent asks for external deadline, timezone, and readiness/review margin when missing information can materially alter consequential planning or priority, and does not repeat resolved inquiries.
43. A bounded ranked set of top approaching deadlines is visible and shared by Work Loop, Trajectory, Pi, TUI, Mission Canvas, and menubar.
44. Priority follows operator steering while surfacing critical deadline conflicts, consequences, and safer sequencing; the agent never silently changes the ask.
45. API, CLI, Pi, generated clients, TUI, Mission Canvas, menubar, notifications, Work Loop, and Silent Sessions expose semantically equivalent state and reason codes.
46. Bloatgaurd keeps hot temporal context bounded but cannot omit the temporal priority header; all detail remains reachable through rehydrate refs.
47. UI clearly distinguishes evidence confidence, slack status, definitely/possibly/indeterminately crossed boundaries, and normal/watch/at-risk/critical/expired; it uses accessible non-color-only presentation, never shows decorative success over a breach, measures operator review burden, and deduplicates/rate-limits alerts.

### High-consequence precision and Markets profile

48. Precision, resolution, accuracy, uncertainty, latency, and ordering are distinct in schemas, UI, policy, and tests.
49. High-consequence timestamps include exact stable capture point, paired clock sample, source/authentication/diversity posture, clock domain, synchronization/holdover posture, integer unit, uncertainty method/coverage, policy version, and provenance.
50. Runtime calibration proves effective accuracy on actual deployed host/network/adapter/provider paths; formatted precision cannot substitute.
51. Policy blocks dispatch when clock uncertainty, market-data age, decision age, or dispatch age exceeds bounds.
52. Cross-host ordering uses sequence/causation evidence rather than timestamp order alone.
53. LLM/model paths are absent from latency-critical deterministic execution loops.
54. Markets operations preserve event/ingestion/decision/authority/risk-check/dispatch/acknowledgement/fill/cancel/reconciliation timestamps, exact capture points, causal/sequence lineage, and uncertainty-bearing latency distributions.
55. Market calendars, timezone/DST, holidays, early closes, auctions, halts, stale data, gaps, overload, disconnect, leap, restart, and clock drift pass deterministic tests.
56. Duplicate prevention, idempotency, unknown-outcome reconciliation, partial-fill, cancellation-race, and kill-switch behavior pass adversarial runtime proof.
57. No expired, stale, uncertainty-violating, out-of-scope, or risk-limit-violating market intent dispatches.
58. Live-market capability remains blocked through simulation/paper/shadow/canary levels until every activation-firewall requirement is evidenced and explicitly approved.
59. Every high-consequence domain pack declares and proves an applicable TemporalPrecisionProfile, control owner/reviewer, jurisdiction/rule version, resilience/BCDR posture, retention, and deterministic boundary without creating a parallel temporal runtime.

### Completeness and release

60. Every normative statement has a stable feature-ledger row; every applicable mandatory/activated conditional row has implementation ownership, tests, Evidence, and Receipt, while optional/not-applicable/variance posture is separately evidenced.
61. No applicable mandatory row is silently deferred, omitted, mocked, disabled, hidden behind a flag, or marked complete while blocked.
62. Every approved deferral of applicable mandatory scope exists only as an explicit specification amendment and remains visibly open for affected conformance; verified non-applicability and optional unimplemented capability use their typed ledger states instead of fake amendments.
63. Static, unit, contract, runtime, restart/replay, scope/CAS/fencing, applicable CRDT replication, security/privacy, accessibility, precision, high-consequence, and adversarial fault-injection tests cover all mandatory behavior.
64. Current API/CLI docs, tool registry, generated schemas/clients, migrations, architecture docs, operator docs, and conformance manifests match implementation.
65. Final closure includes a requirement-by-requirement proof matrix and explicit zero-unapproved-omission attestation.

### Research-audit integrity additions

66. Clock trust proves monitored independent/diverse sources, authentication/replay/request-response posture, disagreement handling, synchronization/holdover age, and source quarantine/recovery.
67. Clock uncertainty records components, method, combined/expanded uncertainty, coverage factor/probability, offset/delay/jitter/dispersion/root distance/frequency, sample age, and calibration lineage.
68. Platform capability evidence distinguishes realtime, suspend-excluding monotonic, suspend-aware, CPU, and TAI behavior; unsupported/fallback behavior is explicit and tested.
69. Civil-time intent preserves original expression, tzdb/calendar/jurisdiction versions, fold/gap policy, recurrence/floating semantics, resolution history, and material rule-change re-resolution.
70. Deadline/expiry comparisons expose definitely-before, possibly-crossed, definitely-crossed, and indeterminate states; uncertainty crossing a consequential boundary cannot be reported as on time.
71. RPC/process/agent children receive remaining monotonic timeout with elapsed deducted and parent cap; cancellation is observed/acknowledged/effective, and retries share the original deadline/budget with reconciliation-before-retry after possible effect.
72. Forecast, operator expectation, external deadline, readiness target, commitment, execution budget, and observed progress use distinct claim types, authority, display, and metric behavior.
73. Forecast calibration uses policy-selected quantiles, proper scoring/coverage/reliability/bias/sharpness/skill/value measures, sample/error bounds, censoring, correlation, baseline, and drift handling.
74. Simultaneous deadlines produce explicit feasible/infeasible/unknown conflict state, one primary objective, preserved non-preemptible obligations, disclosed displacement, and operator escalation when needed.
75. Urgency transitions use hysteresis, dwell, debounce, deduplication, backpressure, quiet-hours/availability policy, and notification budgets.
76. Narrowing under pressure preserves protected safety/security/authority/proof/reconciliation/disconfirming-evidence checklists, independent review where required, workload/fatigue posture, and fresh-reviewer handoff.
77. Markets proof includes direct/exclusive control ownership, delegated-control due diligence, written/annual review and certification, capacity/integrity/resilience/availability/security, BCDR/RTO/RPO, exact timestamp point/UTC traceability review, and jurisdiction/activity-specific thresholds.
78. Temporal data has classification, least privilege, encryption, coarsening, redaction, retention/deletion/legal hold, export, aggregation, audit-access, and side-channel policy.
79. Clock samples, correction/deadline/guard/cancellation/closure events, and Receipts are signed/hash-chained; source authentication and ledger integrity are tested as separate controls.
80. Velocity and temporal-performance reports include cohorts, sample/censored counts, distributions/tails, uncertainty, baselines, policy versions, and split/merge anti-gaming; raw item count or averages cannot prove improvement.
81. Every authority-bearing temporal decision records exact schema, policy, adapter, calendar/tzdb, estimator, and clock-profile versions needed for deterministic replay and settlement.
82. Fault injection covers wall step/slew, source disagreement/spoof/replay, leap/smear, DST fold/gap, tzdb/calendar revision, suspend, reboot, daemon outage, uncertainty-boundary crossing, propagation delay, queueing, cancellation races, stale data, and alert/pulse overload.
83. Spec 136 integration remains blocked until its stated activation conditions and an approved immutable dependency contract pin source commit/document hash/schema/ownership/migration/conformance; repository presence, prose, or aliases are not activation authority.
84. Requirement applicability preserves RFC normative class meaning and cannot hide unsupported required platforms/domains or activate optional claims without proof.
85. Closure factual status, operator disposition, amendment, degraded posture, and rollup eligibility remain separate through Workpoint/task/spec closure, settlement, Receipts, velocity, and release conformance.
86. Temporal communication explicitly discloses material uncertainty, assumptions, degraded/stale state, failed tools, missing proof, alternatives, and inference boundaries in coherent plain language; confidence percentages identify their object/basis and cannot masquerade as calibrated probability, work completion, or duration evidence.

## Implementation order

Implementation is progressive but no requirement may be silently deferred. Each slice closes only with its assigned feature-ledger rows, code, migrations, parity proof, negative tests, Evidence, Receipt, and zero-unapproved-omission statement.

### Slice 0 — Reality lock and complete requirement ledger

- Inventory every current timer, timestamp, deadline-like field, budget, lease, security TTL, turn-local timer, projection, and estimate-producing surface.
- Inventory Workpoint, Work Loop, Silent Session, Project Card, prediction, metacognition, benchmark, compaction, Awareness, Preload, Context Cognition, Bloatgaurd, Receipt, UI, installer, update, and release consumers.
- Inspect and hash-pin proposed Spec 136 ownership and activation constraints; mark integration blocked until its stated activation conditions and an approved immutable dependency contract are satisfied.
- Create and validate `spec137-inferred-decision-register.v1.yaml`, `spec137-complete-feature-ledger.v1.yaml`, delivery DAG, schema inventory, reason-code catalog, cross-spec ownership matrix, dependency manifest, applicability/variance policy, and migration inventory.
- Record baseline behavior including unsupported forecast examples, claim-type conflation, timing reset/cross-boot defects, override-as-completion, and hot-path context coupling.
- Gate: every normative statement has a ledger row; every applicable mandatory/activated conditional requirement has an owner; optional/not-applicable/variance states have evidence; excluded applicable mandatory requirements list is empty.

### Slice 1 — Trusted temporal substrate

- Implement wall/per-boot monotonic/suspend-aware/TAI-capability abstractions, ClockCapabilityProfile, ClockTrustProfile, ClockSamplePair, source authentication/diversity/disagreement, holdover, uncertainty propagation, correction events, timezone/tzdb authority, and deterministic fake/fault-injection clocks.
- Add scoped temporal event envelope, append-only signed/hash-chained persistence, SQLite migrations, portable evidence projections, applicable CRDT reconciliation, interval-union logic, and separate bounded/unknown inter-epoch gaps.
- Prove DST/tzdb/timezone changes, NTP step/slew, source disagreement/spoof/replay/authentication loss, leap/smear, sleep/wake, suspend-clock differences, reboot, daemon restart, duplicate/out-of-order events, and unavailable/uncertain clock behavior.
- Keep security TTL, lease, authority expiry, deadline, duration, CPU time, suspend-aware expiry, and evidence freshness semantics distinct.

### Slice 2 — Spec 131 timing and closure integration

- Pin the approved Spec 131 schemas, reason codes, and ownership version; do not fork its Workpoint Item lifecycle, Work Timing Ledger, velocity, or closure state machine.
- Build adapters that consume Spec 131 item, timing-category, token/tool, evidence, and closure records as temporal evidence.
- Add Spec 137 clock-epoch, deadline, urgency, forecast, and incident refs without changing Spec 131 attribution semantics.
- Prove item → Workpoint → task/bead → spec → project → trajectory integration does not double-count or create competing canonical records.
- Extend API/CLI and resume projections with explicit Spec 131/137 provenance.

### Slice 3 — Deadline, readiness target, and calendar authority

- Implement HumanCalendarContext, CivilTimeIntent, DeadlineContract, CalendarConstraint, fixed/civil/floating/date/business/recurring/session semantics, fold/gap policy, tzdb/calendar versioning and re-resolution, boundary/completion-effect semantics, hierarchy, DeadlineConflict/infeasibility, readiness target, safety margin, exact completion target, external-authority distinction, revision/CAS, approval, clear/revise semantics, OverdueOpportunityAssessment, OverdueDeliveryMode, and audit.
- Implement uncertainty-aware definitely/possibly/indeterminately crossed posture, probabilistic/correlated critical-path slack, and consequence-sensitive unknown-slack policy.
- Implement remaining-time deadline propagation, child capping, cancellation observation/acknowledgement/effectiveness, cleanup, retry-budget sharing, and reconciliation-before-retry.
- Integrate operator timezone/working windows with privacy, quiet-hours/availability policy, revocation, and minimal signed context hashes.
- Prove agents cannot mutate deadlines, Focusa cannot purport to clear external boundaries, budget renewal cannot move them, and deadlines/civil intent survive every lifecycle boundary.

### Slice 4 — Material progress and anti-waste governance

- Implement material-progress verification, TemporalPulsePolicy, TemporalUrgencySignal, adaptive polling/event refresh, hysteresis/dwell/debounce/deduplication/backpressure/notification budgets, protected checklists, workload/fatigue/handoff posture, and derived last-progress projections.
- Implement content-hash reread detection, normalized equivalent-action detection, bounded research contract, diminishing-return checks, duplicate handoff detection, and no-progress watchdog.
- Implement temporal preflight, long-process heartbeat/silence, timeout, cancellation, process-tree cleanup, partial-result capture, and user-visible update cadence.
- Add LostTimeIncident, OpportunityRisk, MissedOpportunity, CounterfactualUnknown, TemporalBreach, dispute/review, settlement, and remediation.
- Prove legitimate long compilation/reconciliation is not falsely classified when bounded progress evidence exists.

### Slice 5 — Estimate engine and conversational gate

- Implement TemporalClaimEnvelope, EstimateClaim, ForecastCalibrationProfile, comparable-task/cohort selection, all-attempt forecast history, censoring, dependency correlation, policy-selected quantiles, interval coverage, confidence mapping, expiry, invalidation, estimator versioning, observation uncertainty, calibration/reliability/bias/sharpness/skill/value evaluation, and drift/error bounds.
- Separate forecasts, operator expectations, deadlines, readiness targets, commitments, budgets, observed progress, closure velocity, and forecast history.
- Add common daemon response validators for numeric/qualitative forecast claims and evidence-backed progress claims.
- Route estimate questions automatically from Pi/API/UI.
- Refuse cold-start/ambiguous/stale claims with typed recovery.
- Remove turn-local Project Card timing as estimate authority and migrate to canonical ledgers.
- Prove no model/client can bypass the gate with wording variants or direct client output.

### Slice 6 — Work Loop and Silent Session enforcement

- Integrate TimeAwareness, local TemporalExecutionGuard issuance/validation/revocation, deadline revision/propagation, cancellation, correlated critical path, conflict/infeasibility, pressure, no-progress, preflight, overdue opportunity assessment, past-due item pinning, protected delivery focus, and consequence policy into Work Loop selection/continuation.
- Integrate absolute deadlines separately from renewable execution budgets.
- Integrate Silent Session run/attempt/process timing, heartbeat, adoption, recovery, model changes, operator steering, writer leases, and settlement.
- Ensure deadline expiry preserves reconciliation/compensation/cleanup truth paths.
- Prove restart, orphan adoption, controller loss, root-contained scheduling, unrelated ready-work selection, and no silent idle state.

### Slice 7 — Context, compaction, awareness, and agent delivery

- Extend ContextCognitionPacket optimization frame, AwarenessPacket, Preload Packet, WorkpointResumePacket, Trajectory packet, CompactionMissionPacket, and Bloatgaurd with bounded HumanCalendarContext, TemporalPriorityFrame, and temporal refs.
- Inject fresh TimeAwareness before every Pi turn and after every invalidating event.
- Preserve deadlines, elapsed, progress, reread hashes, estimates, incidents, and do-not-repeat work through compaction/handoff.
- Add daemon-side enforcement during model/tool execution and common communication projection for absolute uncertainty disclosure, coherent evidence/assumption/alternative explanation, and typed confidence-percentage semantics.
- Prove no transcript/cached/client clock fallback can become authority and no confidence label can masquerade as calibrated probability, work completion, or duration grounding.

### Slice 8 — Closure, Receipts, Spec 136, and learning

- Integrate factual completion status, closure-check status, operator disposition, amendment, degraded posture, and rollup eligibility into Spec 116 closure and Spec 119 Receipts; no override may manufacture verified completion.
- Implement proposal/verification/resolution mapping for EstimateClaim; reducer mapping for deadlines/progress/breaches; AuthorityDecision/ExecutionIntent temporal preflight; completion/settlement temporal outcomes; and post-settlement learning. Implement the Spec 136-specific mapping only after its stated activation conditions and immutable dependency contract are satisfied; otherwise keep that adapter blocked and unclaimed.
- Allow functional success with temporal failure and preserve both.
- Implement the continuous temporal prediction/evaluation/reflection/LearningCandidate/promotion/rollback loop and feed settled estimate accuracy, route quality, lost-time patterns, urgency responses, and remediation outcomes to prediction/metacognition/self-heal without self-sovereign learning.
- Prove urgency cannot skip proposal-to-settlement stages.

### Slice 9 — API, Operation Registry, generated clients, CLI, and tools

- Implement all canonical routes, result/block envelopes, SSE events, Operation Registry descriptors, OpenAPI schemas, generated Rust/TypeScript clients and portable OpenAPI/JSON Schema contracts where supported, tool contracts, tool choreography, CLI, Pi tools, doctor, and docs.
- Enforce exact scope, principal, CAS, confirmation, idempotency, and common temporal reason codes.
- Prove human/JSON/generated-client parity and no direct adapter/client authority path.

### Slice 10 — Operator surfaces and notifications

- Implement Mission Deck/Canvas, Work Rail, TUI, menubar, generated UI, accessible warnings, deadline/readiness presentation, progress/stall posture, grounded-estimate display, cancellation/recovery, incident inspection, and notification deduplication.
- Track operator attention/review burden and notification friction.
- Render clear conclusions with coherent evidence, assumptions, alternatives, uncertainty, and what changes confidence; label judgmental confidence percentages separately from calibrated probabilities and verified completion.
- Prove responsive layouts, non-color encoding, calm default presentation, critical escalation, restart/rehydration, and no decorative success.

### Slice 11 — Benchmarks, high-consequence domain proof, security, and anti-gaming

- Extend Golden Tasks, Spec 113/114 benchmarks, replay fixtures, fake clocks, and holdouts with temporal metrics.
- Test unsupported numeric/qualitative estimates, survivorship bias, scope invalidation, clock skew/rollback, DST, restart, deadline mutation, budget-renewal bypass, fake progress, repeated actions, stuck tools, temporal-pulse loss/overpolling, pressure transitions, prediction calibration, metacognitive promotion/rollback, concurrency overlap, alert fatigue, metric gaming, and urgency safety.
- Test temporal-data classification, calendar privacy, least privilege, encryption, coarsening/redaction, retention/deletion/legal hold, exports, aggregation, side channels, authority, signature/hash-chain integrity, malicious prompt/tool/provider time claims, and agents attempting to improve scores by category or item-split manipulation.
- Implement and prove precision profiles, integer high-resolution timing, NIST-style uncertainty method/coverage, P50/P80/P95/P99/P99.9/max latency, sequence/causation ordering, exact stable capture points, and deployed-path calibration/traceability review.
- Implement the Markets domain temporal contract and simulation/paper/shadow/canary capability gates; prove direct/exclusive control ownership, delegation due diligence, written/annual review/certification, rule applicability/versioning, capacity/integrity/resilience/availability/security, BCDR/RTO/RPO, records/notifications, timestamp-point traceability, and jurisdiction/activity thresholds; test stale/gapped data, market calendars, clock drift, overload, timeout-after-effect, duplicate order, partial fill, cancel race, halt, disconnect, restart, reconciliation, and independently reachable kill switch.
- Require equivalent declared temporal profiles and negative proof for every other high-consequence domain pack.
- Report quality-adjusted time to settled outcome and tail latency/uncertainty, not raw speed or average latency alone.

### Slice 12 — Migration, documentation, conformance, and final closure

- Migrate legacy turn-local timing, Work Loop epochs, Silent Session run fields, Project Card outcomes, benchmark ledgers, and compatible historical timestamps with explicit confidence/degradation.
- Update architecture, all affected specs' integration clauses, current API/CLI references, tool docs/registry, operator guides, troubleshooting, doctor, privacy, accessibility, migration, and developer guides.
- Execute the complete applicability-aware feature ledger, cross-system parity matrix, restart/replay/CAS/fencing/applicable-CRDT matrix, cross-platform clock capability matrix, temporal fault-injection matrix, security/privacy/integrity matrix, immutable Spec 136 dependency/integration matrix, and 293-MUST mapping.
- Produce final Evidence bundle, proof matrix, Completion Receipt, temporal calibration report, and explicit zero-unapproved-deferral/zero-omission attestation.
- Final gate fails if any required ledger row is open, blocked, prose-only, mocked, disabled, client-incomplete, undocumented, untested, unreceipted, or silently deferred.

## Machine-readable delivery artifacts

Required before implementation decomposition closes:

```text
docs/contracts/spec137-inferred-decision-register.v1.yaml
docs/contracts/spec137-complete-feature-ledger.v1.yaml
docs/contracts/spec137-delivery-dag.v1.yaml
docs/contracts/spec137-temporal-state-machine.v1.yaml
docs/contracts/spec137-reason-codes.v1.yaml
docs/contracts/spec137-cross-spec-ownership.v1.yaml
docs/contracts/spec137-cross-surface-parity.v1.yaml
docs/contracts/spec137-conformance-matrix.v1.yaml
docs/contracts/spec137-clock-capability-and-trust.v1.yaml
docs/contracts/spec137-civil-time-and-deadline-semantics.v1.yaml
docs/contracts/spec137-temporal-claim-and-calibration.v1.yaml
docs/contracts/spec137-temporal-security-privacy-integrity.v1.yaml
docs/contracts/spec137-spec136-dependency-manifest.v1.yaml
```

Every task generated from this specification includes:

```yaml
requirement_refs: []
primitive_owner:
implementation_slice:
blocking_refs: []
scope_model:
clock_domains: []
clock_capability_and_trust:
clock_sample_and_uncertainty:
civil_time_semantics:
deadline_propagation_and_cancellation:
human_calendar_grounding:
temporal_priority_behavior:
temporal_pulse_behavior:
calm_focus_gradient:
deadline_behavior:
overdue_opportunity_behavior:
overdue_delivery_behavior:
temporal_claim_behavior:
estimate_behavior:
forecast_calibration_behavior:
temporal_prediction_behavior:
temporal_metacognition_behavior:
progress_behavior:
no_progress_behavior:
lost_time_behavior:
core_types: []
reducer_events: []
persistence: []
api_operations: []
operation_registry_changes: []
generated_contracts: []
cli_commands: []
pi_tools: []
ui_surfaces: []
compaction_resume:
work_loop:
silent_sessions:
closure_completion_and_disposition:
spec136_mapping:
spec136_dependency_lock:
security:
privacy:
integrity_and_retention:
accessibility:
migration:
positive_tests: []
negative_tests: []
clock_tests: []
restart_recovery_tests: []
crdt_tests: [] # required only for declared replicated/portable merge surfaces under Spec 98; canonical authority remains reducer/CAS/fencing
fault_injection_tests: []
adversarial_tests: []
evidence: []
receipts: []
definition_of_done: []
not_done_if: []
excluded_requirement_refs: []
optional_unimplemented_refs: []
not_applicable_refs: []
variance_refs: []
```

`excluded_requirement_refs` applies only to applicable mandatory scope and MUST be empty unless each entry points to an explicit operator-approved specification amendment. `optional_unimplemented_refs`, `not_applicable_refs`, and `variance_refs` MUST be separate, evidence-backed collections; they cannot conceal unsupported mandatory behavior or claimed capability.

## Final closure law

Spec 137 is complete only when Focusa can prove that:

- every interaction and consequential action decision is grounded in fresh trusted human calendar/time context or a valid locally enforceable TemporalExecutionGuard, and the agent/operator receive truthful scoped temporal priority without unnecessary private-calendar disclosure or critical-path round trips;
- fixed-instant and civil-time deadlines, external authority, boundary semantics, uncertainty, conflicts, and protected readiness targets cannot be silently reset, reinterpreted, or consumed;
- unsupported agent forecasts cannot reach any user-facing surface, while operator expectations, deadlines, commitments, budgets, and verified progress remain clearly typed and attributed;
- material progress and activity are structurally distinct;
- avoidable waste is detected, recorded, consequential, and learnable only after governed settlement;
- every affected system consumes one canonical temporal authority;
- functional success cannot hide temporal failure;
- urgency makes the agent progressively calmer, narrower, more execution-focused, and more evidence-disciplined without weakening any safety or truth boundary;
- temporal pulses, predictions, outcome evaluation, metacognition, governed learning, and rollback measurably improve time execution over settled outcomes;
- top approaching deadlines are continuously ranked, missing material deadline context triggers a concise inquiry, and breached items enter evidence-backed overdue-delivery mode as highest execution priority while valid delivery opportunity remains;
- every applicable mandatory and activated conditional requirement is implemented and evidenced, every `SHOULD` variance and optional/not-applicable row is truthful and proven, and no requirement is silently deferred or omitted;
- factual completion, operator disposition, degraded posture, and release rollup eligibility remain separate, so no override or Receipt state can manufacture verified completion;
- the accepted Spec 136 dependency, clock/calendar/policy versions, and temporal security/privacy/integrity controls are immutably pinned and replayable.

The governing watchword remains:

> **THE CALENDAR AND THE CLOCK NEVER WAIT. TO BE EARLY IS TO BE ON TIME; TO BE ON TIME IS TO BE LATE.**

<!-- SPEC137A_138A_144_ARCHITECTURE_CLOSURE:mandatory-spec137a-companion -->
## Mandatory companion: Spec 137A

Spec 137A is a mandatory companion to Spec 137. Combined Spec 137 + Spec 137A
conformance remains open until the zero-deferral applicability, omission, runtime,
and receipt requirements are verified together; implemented temporal slices alone
do not establish combined full conformance.
