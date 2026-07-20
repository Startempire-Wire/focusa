# Spec 131 — Focusa Temporal Authority, Wall-Clock Urgency, Grounded Estimates, Workpoint Timing, Velocity, and Closure

## Status

Normative draft — operator-directed core Focusa temporal-authority specification. The specification is implementation-ready only after its typed contracts, cross-spec ownership map, and conformance ledger are approved. Existing partial timing fields are not evidence that this specification is implemented.

Canonical label: **Spec 131 — Temporal Authority, Workpoint Item Timing, Deadline Urgency, Estimate Grounding, Velocity, and Closure Authority**

Depends on: Specs 55, 56, 66, 67, 78, 79, 88, 96, 98, 100, 101, 104, 106, 107, 108, 109, 110, 111, 113, 116, 119, 120, 125, 130, and 133.

Successor integration: proposed remote Spec 136, `docs/136-governed-proposal-to-settlement-protocol-and-outcome-truth-infrastructure-spec.md`, observed at `origin/main` commit `19898df8d0c3bac632e3e4b44ca1ab9367b595c7`.

Primary implementation surfaces: Focusa core, reducer, daemon, SQLite/CRDT persistence, API, Operation Registry, generated contracts, CLI, Pi extension, Awareness/Preload/Context Cognition, Workpoint, Work Loop, Silent Sessions, Trajectory, CompactionMissionPacket, Bloatgaurd, Evidence/ECS, Receipts, Closure Authority, predictions, metacognition, project cards, benchmarks, TUI, Mission Deck/Canvas, menubar, notifications, tests, conformance, and future timeline UI.

## Constitutional temporal directive

> **THE CALENDAR AND THE CLOCK NEVER WAIT.**
>
> **TO BE EARLY IS TO BE ON TIME. TO BE ON TIME IS TO BE LATE.**

Human wall-clock time is nonrenewable. Focusa SHALL minimize time-to-verified-and-settled outcome without weakening scope, safety, authority, evidence, reconciliation, accessibility, or operator control.

This directive is not motivational prose. It is a runtime invariant with typed records, reducer events, daemon enforcement, client projections, negative tests, Receipts, and post-settlement learning.

The doctrine has exact semantics:

```text
settled before readiness_target_at
    = on_time

settled at or after readiness_target_at but before operator_deadline_at
    = late / contingency consumed / at risk

settled at or after operator_deadline_at
    = deadline breached
```

`complete` for deadline evaluation means the required Spec 136 completion predicates have passed, external reality is reconciled where applicable, settlement is recorded, and the required Receipt is committed. Code written, process exit, provider `200`, passing one test, an agent final message, or provider task status is not deadline completion.

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

1. Accurately measure Workpoint Item, Workpoint, task, spec, project, protocol-stage, tool, proof, and settlement time.
2. Separate wall-clock, monotonic elapsed, active agent, model, tool, queue, blocked, pause, operator-wait, proof, compaction, handoff, reconciliation, settlement, and offline time.
3. Establish a daemon-owned, typed, scope-safe `TemporalAuthority` and bounded `TimeAwarenessPacket`.
4. Model external deadlines, earlier readiness targets, protected safety margins, calendar constraints, urgency, critical-path slack, and deadline breach.
5. Forbid unsupported numeric and qualitative time estimates across API, CLI, Pi, generated UI, TUI, menubar, Work Loop, and background reports.
6. Represent every permitted estimate as a durable, expiring, scope-revision-bound `EstimateClaim` with target state, comparable history, uncertainty, assumptions, and evaluation.
7. Detect and govern no-progress intervals, repeated unchanged reads, equivalent tool churn, unbounded research, diminishing returns, silent long-running commands, and avoidable rework.
8. Make Workpoint Items the smallest independently measurable execution unit while preventing nested and concurrent double-counting.
9. Track tokens, tool calls, proof runs, commits, changed files, evidence, operator attention, resource contention, and temporal incidents per unit of work.
10. Roll metrics up from Workpoint Item to Workpoint, bead/task, spec, project, trajectory, and Spec 136 protocol stages.
11. Add closure authority so work cannot be marked done without required evidence, checks, authorization, outcome truth, and temporal posture.
12. Use all relevant attempt history—not successful completions alone—to improve forecasts, velocity reports, project cards, predictions, metacognition, routing, and critical-path planning.
13. Preserve timing, deadlines, estimate provenance, progress, lost-time incidents, and closure state across compaction, model switches, forks, handoffs, provider overflow, daemon restart, host sleep, and CRDT reconciliation.
14. Make temporal breaches and missed opportunities durable, visible, non-resettable facts when evidence proves them.
15. Prepare the data model for calm, truthful timeline, Mission Canvas, TUI, menubar, notification, and future SaaS projections.
16. Integrate as the primitive-owning temporal specification consumed by Spec 136's proposal-to-settlement lifecycle.

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
- No duplicate temporal state machine in Pi, CLI, generated UI, Spec 136, or provider adapters.
- No visual SaaS implementation requirement; this spec defines the canonical substrate and required projections.
- No replacement of Spec 130 compaction authority or Spec 136 proposal-to-settlement authority; this spec provides temporal primitives consumed by both.

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
14. **Only settled completion satisfies a commitment** when the commitment's target state requires Spec 136 settlement and Receipt.
15. **One temporal authority.** Clients render and request; the daemon/reducer/persistence path owns canonical temporal state.
16. **Guardrails are runtime-native.** Prompt reminders explain temporal policy but cannot be its enforcement boundary.
17. **No silent deferral or omission.** Every normative requirement in this specification must be represented in the machine-readable requirement ledger, assigned to an implementation slice, implemented on every applicable surface, and closed by durable Evidence and a Receipt.
18. **Absence is not degradation.** Missing implementation, missing wiring, missing tests, an empty projection, a hard-coded placeholder, a mock, a hidden feature flag, or an unavailable route cannot be reported as degraded-but-complete.
19. **Later is a governed decision.** A requirement may move to a later tranche only through an explicit operator-approved specification amendment that records the exact requirement IDs, reason, impact, replacement tranche, dependencies, acceptance consequences, and superseding Receipt. It remains visibly open and blocks any conformance level that requires it.
20. **No implicit optionality.** `MUST`/`SHALL` requirements are mandatory. An unimplemented `SHOULD` requires a recorded variance with evidence and operator acceptance; silence never counts as a variance.

## Completeness, non-deferral, and omission firewall

Before implementation decomposition, every normative statement MUST receive a stable requirement ID in `docs/contracts/spec131-complete-feature-ledger.v1.yaml`. The ledger is append-only/versioned and includes:

```yaml
requirement_id:
spec_section:
requirement_text:
requirement_class: must | shall | should | may
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
status: not_started | active | implemented_unverified | verified | explicitly_removed_by_amendment
amendment_ref:
```

Release and closure rules:

- A requirement cannot be omitted because its implementation is difficult, cross-platform, expensive, inconvenient, or discovered late.
- A requirement cannot be hidden in prose-only follow-up, TODO, issue comment, backlog, disabled test, ignored test, mock, compatibility fallback, or client-specific implementation.
- A backend-only implementation is incomplete when API, CLI, Pi, generated contracts, documentation, or required operator surfaces are applicable.
- A UI-only projection is incomplete without canonical daemon state and mutation authority.
- Happy-path proof is incomplete without required negative, stale, restart, scope, clock, and adversarial tests.
- Existing partial code does not automatically satisfy a requirement; it must pass the new contract and Evidence gate.
- Unsupported platforms or capabilities must be truthfully declared and remain open where this specification requires support.
- A requirement marked blocked remains open; blocker visibility is not completion.
- Degraded mode cannot waive estimate grounding, deadline truth, scope, safety, evidence, reconciliation, or omission reporting.
- Generated clients, tool registries, current API/CLI references, migration notes, and conformance manifests are part of implementation, not optional documentation polish.
- Spec 131 cannot close while any mandatory ledger row is missing, open, silently deferred, unsupported without approved scope amendment, or evidenced only by assertion.

Every implementation tranche MUST publish:

1. included requirement IDs;
2. explicitly excluded requirement IDs, which must be an empty list unless an approved amendment exists;
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

Spec 131 owns the semantics and schemas for clocks, elapsed intervals, deadlines, readiness targets, estimates, progress, no-progress detection, temporal breaches, opportunity posture, lost-time incidents, velocity, and temporal projections.

Spec 136 owns the cross-system proposal-to-settlement lifecycle in which those primitives participate. Under Spec 136's primitive-owner rule, Spec 136 MUST reference Spec 131 rather than create a second timing ledger, deadline authority, estimate type, progress classifier, or reason taxonomy.

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
| Spec 136 settlement | Reads | Evaluates actual | No | Evaluates evidence | Applies policy result | Yes through reducer path |

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

## Trusted clock and temporal authority

```yaml
schema: focusa.temporal_authority.v1
authority_id:
host_id:
operator_timezone:
wall_clock_source:
monotonic_clock_source:
boot_id:
monotonic_epoch_ref:
last_wall_sample_at:
last_monotonic_sample:
observed_clock_skew_ms:
clock_confidence: trusted | corrected | skewed | unavailable
correction_event_ref:
schema_version:
```

Requirements:

- Duration arithmetic uses monotonic time within a boot epoch.
- Human deadlines use timezone-aware absolute instants.
- Host sleep and daemon downtime continue wall-clock elapsed while active execution stops.
- Reboot creates a new monotonic epoch linked through persisted wall samples.
- NTP correction, manual clock change, DST, timezone change, and device/daemon skew produce explicit posture; they cannot create negative elapsed time.
- Client clocks are observations, not canonical authority.
- Transcript timestamps are never authority fallback.
- When temporal authority is unavailable, clients fail closed for estimates and deadline arithmetic while preserving safe status and recovery guidance.

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
max_permitted_error_ns:
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
- If observed uncertainty exceeds policy, high-consequence dispatch fails closed. The system may reconcile or cancel safely but may not pretend the requested accuracy remains available.
- Accuracy claims require runtime calibration Evidence for the actual host, network, adapter, provider, and capture path. Platform documentation or timestamp formatting is insufficient.
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

Models may observe, propose, explain, critique, and produce governed strategy/policy candidates. A specialized deterministic, pre-authorized, risk-bounded adapter/execution engine performs latency-critical actions using immutable inputs and exact constraints. Spec 136 governs proposal, authority, durable intent, dispatch lineage, reconciliation, completion, settlement, and learning around that engine.

### Time-sensitive action law

For some market operations, a late action is more dangerous than no action. Every consequential intent includes:

```yaml
temporal_execution_guard:
  event_time_ref:
  data_observed_at:
  decision_at:
  authority_checked_at:
  dispatch_not_before:
  dispatch_deadline:
  cancel_deadline:
  max_data_age_ns:
  max_decision_age_ns:
  max_clock_error_ns:
  precision_profile_ref:
  on_expiry: block | cancel | reconcile_only | kill_switch | operator_review
```

Immediately before dispatch, the deterministic executor rechecks time, data freshness, authority, risk, account/instrument scope, and kill-switch state. Expired or uncertainty-violating intents MUST NOT dispatch. After a possible effect, timeout/disconnect enters `outcome_unknown` and reconciliation-before-retry under Spec 136; blind retry is prohibited.

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
9. operator-visible kill switch and incident runbook;
10. independent security/risk review, Evidence, Receipt, and explicit operator activation.

Backtest success, simulated timestamps, provider `200`, model confidence, low average latency, or one successful order cannot activate live trading. Unsupported timing accuracy, missing uncertainty, stale data, unavailable reconciliation, or incomplete risk controls fail closed and cannot be silently deferred.

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
timezone:
operator_deadline_at:
readiness_target_at:
required_safety_margin_ms:
safety_margin_basis: operator_supplied | policy | measured_history
completion_target_state:
completion_policy_ref:
settlement_required:
receipt_required:
priority:
hardness: advisory | soft | hard | external_window
calendar_constraint_refs: []
parent_deadline_ref:
created_by:
created_at:
revision:
supersedes:
status: active | revised | cleared | satisfied_early | late_window | breached | cancelled
authority_ref:
reason:
```

Rules:

- `readiness_target_at` MUST be earlier than `operator_deadline_at` when a protected margin is required.
- The safety margin preserves verification, review, integration, recovery, delivery, and final confirmation time.
- The agent may not invent a margin; it is operator-supplied or policy/measurement-grounded.
- Satisfying a child deadline does not satisfy its parent unless the parent's target state is independently proven.
- Clearing, extending, or weakening a deadline requires authenticated scope, CAS revision, reason, and audit Receipt.
- Crossing the readiness target creates `late_window` even if the external deadline has not yet passed.
- A pause or renewable budget never changes either boundary.
- Multiple deadlines are ordered by critical-path impact, hardness, priority, and slack; clients must not silently choose one.
- Operator working hours, approval availability, release windows, maintenance windows, provider cutoffs, and scheduled interruptions use typed `CalendarConstraint` records with minimum-data privacy.

### Deadline inheritance and slack

```text
grounded schedule slack
  = time until readiness target
  - grounded critical-path remaining range
```

If critical-path duration is not grounded, slack is `unknown`, not a fabricated number. Parent/child deadline inheritance, conflicts, and overrides must be visible and reason-coded.

## Time Awareness Packet

```yaml
schema: focusa.time_awareness.v1
packet_id:
generated_at:
expires_at:
source_state_revision:
temporal_authority_ref:
clock_confidence:
trusted_now:
timezone:
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
readiness_target_at:
operator_deadline_at:
deadline_remaining_ms:
safety_margin_remaining_ms:
schedule_slack_ms:
schedule_slack_status: positive | exhausted | breached | unknown
last_material_progress_ref:
last_material_progress_at:
no_progress_elapsed_ms:
unchanged_document_rereads:
equivalent_tool_attempts:
same_subproblem_attempts:
temporal_pressure: normal | watch | at_risk | critical | expired
active_estimate_claim_refs: []
lost_time_incident_refs: []
critical_next_action:
rehydrate_refs: []
```

Packet laws:

- It is a bounded projection, not a second store.
- Every field is derived from canonical state, registered policy, or append-only temporal records.
- It expires and is invalidated by scope, Workpoint, deadline, progress, steering, or relevant policy revision.
- Pi receives it before every model turn, after long-running tools, at continuation boundaries, and after compaction/resume.
- During model/tool execution, the daemon watchdog—not prompt memory—enforces deadlines and cancellation.
- Context Cognition, Awareness, Preload, WorkpointResumePacket, CompactionMissionPacket, Work Loop status, Silent Session status, TUI, Mission Canvas, and menubar consume the same projection.

## Estimate Claim and conversational response gate

A duration or completion-time statement is a probabilistic proposal, never a fact merely because a model phrases it fluently.

```yaml
schema: focusa.estimate_claim.v1
estimate_id:
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
p50_ms:
p80_ms:
range_low_ms:
range_high_ms:
confidence: insufficient | low | medium | high
grounding_status: measured_history | deterministic_deadline_arithmetic | operator_supplied | mixed | insufficient_evidence
assumptions: []
dependencies: []
excluded_time_categories: []
uncertainty_reasons: []
invalidated_by_ref:
status: proposed | verified | displayable | invalidated | expired | evaluated | refused
verification_bundle_ref:
actual_target_event_ref:
actual_elapsed_ms:
calibration_score:
```

### Required estimate target

Every estimate MUST identify what it predicts, such as:

- first material result;
- implementation complete;
- focused tests passing;
- required proof complete;
- provider closure reconciled;
- Spec 136 settlement recorded;
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
- Operator-provided expectations are labeled as operator-provided, not measured Focusa forecasts.
- Estimates expire and must be evaluated against the exact target event.

### Response enforcement

All response surfaces MUST route estimate-shaped output through one daemon-owned validator. This includes numeric and qualitative claims such as `soon`, `quick`, `a little while`, `nearly done`, `a few hours`, `most of the work`, `should finish today`, and equivalent phrasing.

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
normal   — readiness margin healthy or unknown without immediate risk
watch    — protected margin is being consumed or no-progress threshold approached
at_risk  — readiness target likely endangered or already crossed
critical — external deadline is near with unresolved critical-path obligations
expired  — operator deadline reached or passed
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
- skip required Spec 136 stages;
- encourage blind retry after possible external effect;
- prevent cleanup, reconciliation, compensation, or settlement after expiry.

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

## Existing partial surfaces

Current code already has pieces that can be reused or extended:

- Pi task timing auto-populates `task_timing.elapsed_ms`, `elapsed_seconds`, and `elapsed_hms` during `focusa_project_card_outcome`.
- Project card outcome records store task timing and token usage.
- Project card summaries compute average elapsed time and token usage from recorded outcomes.
- CLI turn history exposes turn durations.

Current gaps:

- Timing is not consistently bound to a bead/task/Workpoint Item.
- Timing resets on operator input and can include unrelated scope changes.
- Pauses, blocked time, compaction, handoff, and operator-wait time are not separated.
- Closure does not consistently require item-level proof.
- Velocity is not computed from granular completed implementation units.

## Relationship to Spec 130

Spec 130 defines the HLT-aware Compaction Mission Packet and Bloatgaurd Context Firewall. Spec 131 extends that architecture by making Workpoint Items measurable and closeable across compaction boundaries.

Spec 131 imports these Spec 130 rules:

```text
Compaction packets are not authority.
Transcript tails are not authority.
Generic HLT is not authority.
HLT posture must be visible before durable work.
Evidence and receipt expectations must survive compaction.
Raw bulky context belongs behind ECS/Evidence handles.
Closure claims require receipt/evidence posture.
```

Spec 131 adds this item-level invariant:

```text
A Workpoint Item cannot be measured, completed, closed, or used for velocity unless its timing, token usage, evidence refs, HLT posture, receipt posture, and closure authority survive compaction or are explicitly marked degraded.
```

### Spec 130 data consumed by Spec 131

Workpoint Item and closure records must be able to reference:

- `CompactionMissionPacket.packet_id`;
- `TrajectoryResumePacketV3.packet_id`;
- `WorkpointResumePacketV2.workpoint_id`;
- `HLT_STATUS`;
- `GENERIC_BOOTSTRAP`;
- `FALLBACK_SOURCE`;
- Bloatgaurd omitted-context receipt;
- ECS/Evidence rehydrate refs;
- active blocker excerpt/rehydrate handle;
- receipt expectation;
- closure authority result.

### Durable-work gate

Workpoint Item closure is durable work. It must obey Spec 130 durable-work rules:

```text
HLT_STATUS=canonical_explicit
OR HLT_STATUS=previous_valid_fallback with refreshed session-specific state
OR explicit degraded-mode receipt posture
OR operator override with recorded reason where allowed.
```

Generic HLT can never become canonical closure authority through override alone.

## Core model

### Rollup hierarchy

```text
WorkpointItem → Workpoint → Bead/Task → Spec → Project → Trajectory
```

A Workpoint Item is the smallest measurable unit of execution. Workpoints contain one or more items. Beads/tasks link to Workpoint Items. Specs and projects roll up completed item metrics.

## Workpoint Item

A Workpoint Item is an actionable, measurable slice of work such as audit, design, implementation, test, proof, docs, or closure.

```json
{
  "schema": "focusa.workpoint_item.v1",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "focusa-wefzg.2",
  "parent_item_id": null,
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "...",
  "session_id": "...",
  "spec_ref": "docs/128-...md#release-manifest",
  "phase": "audit|design|implement|test|proof|docs|closure",
  "title": "Implement release manifest validator",
  "target_objects": [],
  "acceptance_refs": [],
  "required_evidence_refs": [],
  "status": "queued|active|paused|blocked|done|closed",
  "closure_authority": "spec_acceptance|bead_done_condition|operator_override",
  "hlt_status": "canonical_explicit|previous_valid_fallback|supersession_pending|missing_required|generic_degraded|conflicted",
  "trajectory_packet_ref": null,
  "compaction_packet_ref": null,
  "receipt_refs": [],
  "bloatgaurd": {
    "omitted_context_refs": [],
    "rehydrate_refs": [],
    "raw_context_externalized": false
  },
  "started_at": null,
  "last_active_at": null,
  "completed_at": null,
  "closed_at": null,
  "timing": {},
  "token_usage": {},
  "tool_usage": {},
  "evidence_refs": [],
  "blockers": [],
  "next_item_ids": []
}
```

## Work Timing Ledger

Timing records are append-only. Corrections append superseding records instead of rewriting history.

```json
{
  "schema": "focusa.work_timing.v1",
  "event_id": "...",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "focusa-wefzg.2",
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "...",
  "session_id": "...",
  "phase": "audit|design|implementation|test|proof|docs|review|closure",
  "event_type": "start|pause|resume|block|unblock|complete|close|correction",
  "started_at": "...",
  "ended_at": "...",
  "wall_clock_elapsed_ms": 0,
  "monotonic_elapsed_ms": 0,
  "active_agent_elapsed_ms": 0,
  "model_elapsed_ms": 0,
  "tool_elapsed_ms": 0,
  "queue_elapsed_ms": 0,
  "paused_ms": 0,
  "operator_wait_ms": 0,
  "blocked_ms": 0,
  "proof_elapsed_ms": 0,
  "compaction_elapsed_ms": 0,
  "handoff_elapsed_ms": 0,
  "reconciliation_elapsed_ms": 0,
  "settlement_elapsed_ms": 0,
  "offline_elapsed_ms": 0,
  "resource_contention_ms": 0,
  "compaction_count": 0,
  "scope_switch_count": 0,
  "deadline_ref": null,
  "estimate_claim_refs": [],
  "clock_confidence": "trusted",
  "attribution_confidence": "measured",
  "overlap_group_id": null,
  "reason": "...",
  "compaction_packet_ref": null,
  "trajectory_hlt_status": null
}
```

### Timing categories

| Category | Meaning |
| --- | --- |
| `wall_clock_elapsed_ms` | calendar elapsed time from exact start target to exact end target |
| `monotonic_elapsed_ms` | monotonic duration within linked boot epochs |
| `active_agent_elapsed_ms` | model/agent execution classified as active under registered policy |
| `model_elapsed_ms` | provider/model inference and response processing |
| `tool_elapsed_ms` | tool and subprocess execution, with per-tool child records |
| `queue_elapsed_ms` | ready but not dispatched time |
| `paused_ms` | intentional policy/operator pause time; deadline still advances |
| `operator_wait_ms` | time waiting for operator input, review, or approval |
| `blocked_ms` | time blocked by an identified dependency/failure |
| `proof_elapsed_ms` | validation and proof execution/review time |
| `compaction_elapsed_ms` | checkpoint, compaction, rehydration, and reorientation time |
| `handoff_elapsed_ms` | agent/session transfer and duplicated-orientation time |
| `reconciliation_elapsed_ms` | time resolving possible external effects or provider divergence |
| `settlement_elapsed_ms` | completion evaluation, Receipt, and settlement time |
| `offline_elapsed_ms` | wall time while daemon/host/session unavailable |
| `resource_contention_ms` | lock, CPU, memory, rate-limit, or shared-resource wait |

Intervals may overlap. Human wall-clock rollups use interval union, not naive summation. Aggregate agent/compute time may sum concurrent execution only when explicitly labeled. Every category has normative start/end events, attribution policy, overlap handling, and confidence. Queue, blocked, paused, offline, and operator-wait are distinct; unknown time remains `unclassified`, not silently assigned.

## Token and tool accounting

Tokens and tool calls are tracked per Workpoint Item and rolled up.

```json
{
  "schema": "focusa.work_token_usage.v1",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "...",
  "provider_input_tokens": 0,
  "provider_output_tokens": 0,
  "estimated_input_tokens": 0,
  "estimated_output_tokens": 0,
  "total_tokens": 0,
  "tool_call_count": 0,
  "tool_calls_by_family": {
    "bash": 0,
    "read": 0,
    "edit": 0,
    "write": 0,
    "focusa": 0,
    "uiai": 0,
    "web": 0
  }
}
```

## Closure Authority

Closure authority determines whether a Workpoint Item, Workpoint, bead/task, or spec can be marked done.

```json
{
  "schema": "focusa.closure_authority.v1",
  "item_id": "wpi_...",
  "workpoint_id": "...",
  "task_id": "...",
  "closure_requested_by": "agent|operator|daemon|work_loop",
  "closure_authority": "operator|bead_done_condition|spec_acceptance|workpoint_contract",
  "required_evidence_refs": [],
  "provided_evidence_refs": [],
  "required_checks": [],
  "passed_checks": [],
  "closure_status": "authorized|blocked|premature|operator_override|degraded_allowed",
  "hlt_status": "canonical_explicit|previous_valid_fallback|supersession_pending|missing_required|generic_degraded|conflicted",
  "receipt_posture": "canonical|advisory|degraded|blocked|stale",
  "compaction_packet_ref": null,
  "trajectory_packet_ref": null,
  "bloatgaurd_rehydrate_refs": [],
  "reason": "...",
  "checked_at": "..."
}
```

Rules:

- Workpoint Items cannot close without required evidence or explicit operator override.
- Workpoints cannot close until required Workpoint Items close.
- Beads/tasks cannot close until linked Workpoint Items satisfy done conditions.
- Specs cannot close until required beads/tasks and Workpoint Items have proof.
- Operator override must be explicit, visible, and auditable.
- Closure checks must distinguish `blocked`, `premature`, `authorized`, `degraded_allowed`, and `operator_override`.
- Closure checks must include HLT posture from Spec 125/130.
- Closure checks must include receipt posture from Spec 119/130.
- Closure checks must preserve Bloatgaurd rehydrate refs when proof/evidence context was omitted from the hot prompt.

## Compaction and resume requirements

Workpoint Item state must survive Spec 130 compaction. Compaction may elide raw context, but it must preserve item ids, status, timing rollups, HLT posture, closure posture, active blockers, and rehydrate refs.

When `CompactionMissionPacket.status=degraded|blocked`, Workpoint Items may remain active, but closure is blocked unless degraded receipt posture or explicit operator override is recorded.

Workpoint resume packets must expose item state:

- active item;
- blocked items;
- completed but unclosed items;
- next queued items;
- elapsed active time;
- wall-clock time;
- token usage;
- closure authority status;
- missing evidence/checks;
- associated `CompactionMissionPacket` refs;
- HLT status and receipt posture affecting closure.

## Velocity metrics

Velocity must be computed from completed Workpoint Items first, then rolled up.

```json
{
  "schema": "focusa.velocity_summary.v1",
  "project_root": "/home/wirebot/focusa",
  "task_family": "spec128-update-system",
  "completed_items": 0,
  "completed_workpoints": 0,
  "completed_tasks": 0,
  "average_active_elapsed_ms": 0,
  "average_wall_clock_elapsed_ms": 0,
  "average_total_tokens": 0,
  "average_tool_calls": 0,
  "proof_failure_rate": 0.0,
  "rollback_rate": 0.0,
  "reopen_rate": 0.0,
  "estimate_accuracy": 0.0
}
```

Useful reports:

- average time per Workpoint Item phase;
- average time per task family;
- average time per spec section;
- proof/test failure rate;
- token burn per successful closure;
- implementation throughput per day/session/week;
- estimate accuracy by task type;
- common blockers and pause reasons.

### Closure velocity versus forecast history

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

## Spec 136 proposal-to-settlement integration

Remote proposed Spec 136 owns the cross-system lifecycle and MUST consume Spec 131 temporal primitives.

### Lifecycle mapping

| Spec 131 object | Spec 136 treatment |
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
| missed opportunity | outcome fact only after evidence verification |
| lost-time incident | completion/settlement evidence and Receipt lineage |
| forecast evaluation | outcome verification against exact target event |
| process/routing improvement | post-settlement `LearningCandidate` |

### Required Spec 136 integration points

Spec 136 integrations SHALL include:

- normative basis reference to Spec 131;
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
time.attribution_degraded
deadline.at_risk
deadline.critical
deadline.expired
deadline.revision_conflict
estimate.ungrounded
estimate.insufficient_history
estimate.target_ambiguous
estimate.scope_invalidated
estimate.expired
progress.not_material
progress.stalled
research.budget_exhausted
tool.runtime_exceeded
opportunity.at_risk
opportunity.window_missed
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

These map to Spec 136's shared `focusa.protocol_block.v1` envelope with temporal refs, safe next operation, reconciliation posture, and operator review route.

## Cross-system coherence requirements

Spec 131 is incomplete unless every applicable integration below is implemented and proven.

| System/spec | Required integration |
| --- | --- |
| Spec 55 tool contracts | timeout, cancellation, heartbeat, elapsed, cost provenance, progress result |
| Spec 56 trace/recovery | temporal refs in checkpoint, replay, recovery, and corrections |
| Spec 66 ontology | canonical temporal object/action/relation types |
| Spec 67 relevance | deadline/critical-path/information-gain/reread-aware relevance |
| Spec 78 autonomy | bounded cognition/research and temporal stop-loss |
| Spec 79 Work Loop | deadline, watchdog, critical path, no-progress, replan, expiry behavior |
| Spec 88 Workpoint | items, timing, deadline, estimate, progress, incident refs in resume packet |
| Specs 96/102 Trajectory | milestone constraints, aging, risk, critical path without replacing goal authority |
| Spec 97 reflexes | stall, deadline, stale estimate, repeated action, silent tool reflexes |
| Spec 98 CRDT | scoped convergence, idempotency, interval reconciliation, no double-counting |
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
| proposed Spec 136 | governed temporal lifecycle through settlement and learning |
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
focusa time watch --workpoint <id>
focusa time doctor --json

# Deadline authority
focusa deadline set --subject <ref> --at <rfc3339> --timezone <iana> \
  --readiness-target <rfc3339> --completion-target <policy-ref> --confirm
focusa deadline inspect <deadline-id> --json
focusa deadline revise <deadline-id> --expected-revision <n> --reason "..." --confirm
focusa deadline clear <deadline-id> --expected-revision <n> --reason "..." --confirm
focusa deadline list --project <root> --json

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

# Existing Workpoint item and closure surfaces
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
- `GET /v1/time/stream` (SSE)
- `POST /v1/deadline/set`
- `POST /v1/deadline/revise`
- `POST /v1/deadline/clear`
- `GET /v1/deadlines`
- `GET /v1/deadline/:id`
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

All routes use generated shared schemas and a common result envelope. Time, deadline, estimate, progress, and incident events are exposed through the native durable event stream. API routes do not create a second scheduler, estimator, clock, or policy engine.

## Pi and agent surface

Pi tools SHALL include bounded equivalents for time awareness/status, deadline inspection/mutation, estimate request/validation, progress status/recording, temporal preflight, and incident inspection. Mutation tools preserve the same approval and CAS requirements as API/CLI.

Pi turn behavior:

1. fetch/inject fresh TimeAwareness before the model turn;
2. route explicit or inferred time-estimate questions to estimate authority;
3. validate estimate-shaped final output before display;
4. refresh awareness after long tools, steering, deadline change, Workpoint change, compaction, resume, and scope change;
5. display temporal authority failure rather than guessing;
6. keep packets compact and place detailed timelines behind rehydrate refs.

## Operator UI and notification surface

Mission Deck/Canvas, TUI, menubar, generated UI, and notifications show:

- trusted local current time and timezone source;
- readiness target and external deadline;
- safety margin remaining;
- elapsed category summary;
- last material progress and no-progress age;
- current pressure level;
- critical next action;
- estimate grounding/confidence/target state;
- lateness, breach, and opportunity posture;
- cancellation/recovery controls appropriate to authority.

Alerts are deduplicated and escalate through `watch`, `at_risk`, `critical`, and `expired`. Alert fatigue, inaccessible color-only encoding, raw millisecond-only output, decorative success, and hidden breach state are prohibited.

## Storage

Required logical append-only ledgers (SQLite-backed canonical persistence may materialize equivalent tables/outbox/read models; JSONL names describe portable evidence projections):

- `temporal-authority/{host_hash}/clock-events.jsonl`
- `deadlines/{scope_hash}/deadline-events.jsonl`
- `calendar-constraints/{operator_hash}/constraints.jsonl`
- `time-awareness/{scope_hash}/packet-receipts.jsonl`
- `workpoint-items/{project_hash}/items.jsonl`
- `work-timing/{project_hash}/timing-events.jsonl`
- `material-progress/{project_hash}/progress-events.jsonl`
- `estimate-claims/{project_hash}/claims.jsonl`
- `estimate-evaluations/{project_hash}/evaluations.jsonl`
- `no-progress/{project_hash}/incidents.jsonl`
- `lost-time/{project_hash}/incidents.jsonl`
- `opportunity-posture/{project_hash}/events.jsonl`
- `temporal-breaches/{project_hash}/breaches.jsonl`
- `work-token-usage/{project_hash}/token-events.jsonl`
- `closure-authority/{project_hash}/closure-checks.jsonl`
- `closure-velocity/{project_hash}/summaries.jsonl`
- `forecast-history/{project_hash}/attempts.jsonl`
- `work-compaction-links/{project_hash}/item-compaction-links.jsonl`
- `temporal-outbox/{project_hash}/events.jsonl`

All records must include:

- `project_root`;
- `continuity_id`;
- `session_id` when available;
- `workpoint_id`;
- `item_id` where applicable;
- source route/tool;
- created timestamp plus applicable monotonic epoch/sample;
- clock source, confidence, precision profile, and uncertainty where applicable;
- deadline, estimate, progress, correlation, and causation refs where applicable;
- integer canonical duration units;
- schema version;
- `hlt_status` when closure or durable work is involved;
- `compaction_packet_ref` when record was created before/after compaction;
- Bloatgaurd/ECS rehydrate refs for omitted proof/context.

## Acceptance criteria

### Temporal authority and scope

1. Trusted wall and monotonic clocks are distinct, typed, health-checked, and persisted across restart epochs.
2. DST, timezone change, NTP/manual clock correction, sleep/wake, reboot, daemon outage, and client/daemon skew cannot create negative or silently missing elapsed time.
3. Every temporal record is bound to its exact applicable host/operator/project/continuity/Workpoint/item/task scope.
4. Spec 98 reconciliation converges duplicate/concurrent records without cross-scope merge or elapsed double-counting.
5. Corrections append superseding records; no history is rewritten in place.
6. Temporal authority outage fails closed for estimates and deadline arithmetic with explicit recovery.

### Deadline doctrine

7. Deadline contracts distinguish readiness target, external deadline, safety margin, completion target, and settlement requirement.
8. Settled-before-readiness is `on_time`; readiness-to-deadline is `late_window`; at/after deadline is `breached`.
9. Pause, compaction, model switch, restart, operator wait, and budget renewal do not move deadlines.
10. Agents cannot set, extend, clear, weaken, or reinterpret deadlines outside an authorized CAS reducer operation.
11. Deadline inheritance, conflicts, multiple commitments, working windows, and unknown slack are visible.
12. Deadline expiry blocks prohibited new work but permits reconciliation, compensation, cleanup, evidence preservation, and settlement.

### Estimate truth

13. Every duration claim identifies target state, scope revision, expiry, estimator version, evidence basis, comparable samples, uncertainty, and grounding status.
14. Unsupported numeric and qualitative estimates are blocked across API, CLI, Pi, UI, Work Loop, Silent Session, project card, and generated reports.
15. Insufficient history produces refusal, never a heuristic task-count conversion.
16. Forecast history includes relevant failed, abandoned, reopened, blocked, rolled-back, and censored attempts.
17. Point estimates are rejected when only ranges are justified.
18. Scope/target/dependency/deadline/material environment changes invalidate affected estimates.
19. Every displayable estimate is evaluated against its exact actual target event and calibration is recorded.
20. Operator expectations remain labeled and cannot masquerade as measured Focusa estimates.

### Progress and waste

21. Material progress requires evidence-backed target advancement.
22. Activity-only signals cannot reset no-progress clocks.
23. Unchanged rereads, equivalent actions, unbounded research, repeated full proof, silent tools, compaction churn, and duplicated handoff work are detected with false-positive controls.
24. Long-running processes expose elapsed status, heartbeat/silence posture, timeout, cancellation, cleanup, and partial results.
25. Temporal pressure triggers registered operational consequences and never weakens safety or authority.
26. Opportunity risk is distinguished from evidence-proven missed opportunity and unknown counterfactual impact.
27. Lost-time incidents are append-only, reviewable, disputable, settleable, and linked to remediation/learning.
28. Agent/model change, compaction, retry, or process restart cannot erase accumulated incident history.

### Workpoint, closure, and settlement

29. Workpoint Item records are append-only and can be created, listed, started, paused, resumed, completed, and close-checked.
30. Timing separates all normative categories and handles interval overlap correctly.
31. Token/tool/attention usage aggregates across turns without treating usage as progress.
32. Resume packets show active/blocked/next items, deadline posture, elapsed, progress, incidents, estimates, and missing closure proof.
33. Task/spec closure inspects linked Workpoint Items and blocks unsupported completion unless authorized policy explicitly permits a recorded override.
34. HLT, Receipt, Bloatgaurd, Evidence, compaction, and temporal posture survive closure and resume.
35. Spec 136 completion and settlement can represent functional success with temporal failure; Receipts preserve both.
36. Project cards consume canonical ledgers rather than turn-local timers as estimate authority.

### Cross-system and UX parity

37. Time Awareness is injected before every agent turn and refreshed after long tools, steering, deadline/scope/Workpoint change, compaction, and resume.
38. Daemon watchdog enforcement continues while the model/tool cannot receive a new prompt.
39. API, CLI, Pi, generated clients, TUI, Mission Canvas, menubar, notifications, Work Loop, and Silent Sessions expose semantically equivalent state and reason codes.
40. Bloatgaurd keeps hot temporal context bounded while all omitted detail remains reachable through rehydrate refs.
41. UI clearly distinguishes normal/watch/at-risk/critical/expired, uses accessible non-color-only presentation, and never shows decorative success over a breach.
42. Operator review time and notification burden are measured and alert deduplication works.

### High-consequence precision and Markets profile

43. Precision, resolution, accuracy, uncertainty, latency, and ordering are distinct in schemas, UI, policy, and tests.
44. High-consequence timestamps include source, capture point, clock domain, synchronization posture, integer unit, uncertainty, and provenance.
45. Runtime calibration proves effective accuracy on actual deployed host/network/adapter/provider paths; formatted precision cannot substitute.
46. Policy blocks dispatch when clock uncertainty, market-data age, decision age, or dispatch age exceeds bounds.
47. Cross-host ordering uses sequence/causation evidence rather than timestamp order alone.
48. LLM/model paths are absent from latency-critical deterministic execution loops.
49. Markets operations preserve event/ingestion/decision/authority/dispatch/acknowledgement/fill/cancel/reconciliation timestamps and latency distributions.
50. Market calendars, timezone/DST, holidays, early closes, auctions, halts, stale data, gaps, overload, disconnect, leap, restart, and clock drift pass deterministic tests.
51. Duplicate prevention, idempotency, unknown-outcome reconciliation, partial-fill, cancellation-race, and kill-switch behavior pass adversarial runtime proof.
52. No expired, stale, uncertainty-violating, out-of-scope, or risk-limit-violating market intent dispatches.
53. Live-market capability remains blocked through simulation/paper/shadow/canary levels until every activation-firewall requirement is evidenced and explicitly approved.
54. Every high-consequence domain pack declares and proves an applicable TemporalPrecisionProfile without creating a parallel temporal runtime.

### Completeness and release

55. Every normative requirement has a stable feature-ledger row, implementation owner, tests, Evidence, and Receipt.
56. No mandatory row is silently deferred, omitted, mocked, disabled, hidden behind a flag, or marked complete while blocked.
57. Every approved deferral exists only as an explicit specification amendment and remains visibly open for affected conformance.
58. Static, unit, contract, runtime, restart, CRDT, security, accessibility, precision, high-consequence, and adversarial tests cover all mandatory behavior.
59. Current API/CLI docs, tool registry, generated schemas/clients, migrations, architecture docs, operator docs, and conformance manifests match implementation.
60. Final closure includes a requirement-by-requirement proof matrix and explicit zero-unapproved-omission attestation.

## Implementation order

Implementation is progressive but no requirement may be silently deferred. Each slice closes only with its assigned feature-ledger rows, code, migrations, parity proof, negative tests, Evidence, Receipt, and zero-unapproved-omission statement.

### Slice 0 — Reality lock and complete requirement ledger

- Inventory every current timer, timestamp, deadline-like field, budget, lease, security TTL, turn-local timer, projection, and estimate-producing surface.
- Inventory Workpoint, Work Loop, Silent Session, Project Card, prediction, metacognition, benchmark, compaction, Awareness, Preload, Context Cognition, Bloatgaurd, Receipt, UI, installer, update, and release consumers.
- Import Spec 136 ownership constraints without merging remote implementation changes.
- Create `spec131-complete-feature-ledger.v1.yaml`, delivery DAG, schema inventory, reason-code catalog, cross-spec ownership matrix, and migration inventory.
- Record baseline behavior including unsupported estimate examples and timing reset defects.
- Gate: every normative requirement has a ledger row and owner; excluded requirements list is empty.

### Slice 1 — Trusted temporal substrate

- Implement wall/monotonic clock abstraction, boot epochs, clock health, skew/correction events, timezone authority, and deterministic fake clock.
- Add scoped temporal event envelope, append-only persistence, SQLite migrations, portable evidence projections, CRDT reconciliation, and interval-union logic.
- Prove DST, timezone changes, NTP/manual correction, sleep/wake, reboot, daemon restart, duplicate/out-of-order events, and unavailable clock behavior.
- Keep security TTL, lease, authority expiry, deadline, and duration semantics distinct.

### Slice 2 — Workpoint Items and timing taxonomy

- Add/complete Workpoint Item schemas, reducer events, lifecycle, task/bead links, phase taxonomy, overlap groups, confidence, and correction events.
- Track every normative elapsed category and exact start/end boundary.
- Aggregate item → Workpoint → task/bead → spec → project → trajectory without double-counting.
- Integrate token/tool/attention/resource usage while preserving activity/progress separation.
- Add API/CLI basics and resume projections.

### Slice 3 — Deadline, readiness target, and calendar authority

- Implement DeadlineContract, CalendarConstraint, hierarchy, conflict resolution, readiness target, safety margin, exact completion target, revision/CAS, approval, clear/revise semantics, and audit.
- Add deterministic pressure/slack evaluation and unknown-slack posture.
- Integrate operator timezone/working windows with privacy and revocation.
- Prove agents cannot mutate deadlines, budget renewal cannot move them, and deadlines survive every lifecycle boundary.

### Slice 4 — Material progress and anti-waste governance

- Implement material-progress verification and derived last-progress projections.
- Implement content-hash reread detection, normalized equivalent-action detection, bounded research contract, diminishing-return checks, duplicate handoff detection, and no-progress watchdog.
- Implement temporal preflight, long-process heartbeat/silence, timeout, cancellation, process-tree cleanup, partial-result capture, and user-visible update cadence.
- Add LostTimeIncident, OpportunityRisk, MissedOpportunity, CounterfactualUnknown, TemporalBreach, dispute/review, settlement, and remediation.
- Prove legitimate long compilation/reconciliation is not falsely classified when bounded progress evidence exists.

### Slice 5 — Estimate engine and conversational gate

- Implement EstimateClaim, comparable-task selection, all-attempt forecast history, censoring, ranges, confidence, expiry, invalidation, estimator versioning, and evaluation.
- Separate closure velocity from forecast history.
- Add common daemon response validator for numeric and qualitative estimate claims.
- Route estimate questions automatically from Pi/API/UI.
- Refuse cold-start/ambiguous/stale claims with typed recovery.
- Remove turn-local Project Card timing as estimate authority and migrate to canonical ledgers.
- Prove no model/client can bypass the gate with wording variants or direct client output.

### Slice 6 — Work Loop and Silent Session enforcement

- Integrate TimeAwareness, deadline revision, critical path, pressure, no-progress, preflight, and consequence policy into Work Loop selection/continuation.
- Integrate absolute deadlines separately from renewable execution budgets.
- Integrate Silent Session run/attempt/process timing, heartbeat, adoption, recovery, model changes, operator steering, writer leases, and settlement.
- Ensure deadline expiry preserves reconciliation/compensation/cleanup truth paths.
- Prove restart, orphan adoption, controller loss, root-contained scheduling, unrelated ready-work selection, and no silent idle state.

### Slice 7 — Context, compaction, awareness, and agent delivery

- Extend ContextCognitionPacket optimization frame, AwarenessPacket, Preload Packet, WorkpointResumePacket, Trajectory packet, CompactionMissionPacket, and Bloatgaurd with bounded temporal refs.
- Inject fresh TimeAwareness before every Pi turn and after every invalidating event.
- Preserve deadlines, elapsed, progress, reread hashes, estimates, incidents, and do-not-repeat work through compaction/handoff.
- Add daemon-side enforcement during model/tool execution.
- Prove no transcript/cached/client clock fallback can become authority.

### Slice 8 — Closure, Receipts, Spec 136, and learning

- Integrate temporal posture into Spec 116 closure and Spec 119 Receipts.
- Implement Spec 136 proposal/verification/resolution mapping for EstimateClaim; reducer mapping for deadlines/progress/breaches; AuthorityDecision/ExecutionIntent temporal preflight; completion/settlement temporal outcomes; and post-settlement learning.
- Allow functional success with temporal failure and preserve both.
- Feed settled estimate accuracy, lost-time patterns, and remediation outcomes to prediction/metacognition/self-heal without self-sovereign learning.
- Prove urgency cannot skip proposal-to-settlement stages.

### Slice 9 — API, Operation Registry, generated clients, CLI, and tools

- Implement all canonical routes, result/block envelopes, SSE events, Operation Registry descriptors, OpenAPI schemas, generated Rust/TypeScript/Go clients where supported, tool contracts, tool choreography, CLI, Pi tools, doctor, and docs.
- Enforce exact scope, principal, CAS, confirmation, idempotency, and common temporal reason codes.
- Prove human/JSON/generated-client parity and no direct adapter/client authority path.

### Slice 10 — Operator surfaces and notifications

- Implement Mission Deck/Canvas, Work Rail, TUI, menubar, generated UI, accessible warnings, deadline/readiness presentation, progress/stall posture, grounded-estimate display, cancellation/recovery, incident inspection, and notification deduplication.
- Track operator attention/review burden and notification friction.
- Prove responsive layouts, non-color encoding, calm default presentation, critical escalation, restart/rehydration, and no decorative success.

### Slice 11 — Benchmarks, high-consequence domain proof, security, and anti-gaming

- Extend Golden Tasks, Spec 113/114 benchmarks, replay fixtures, fake clocks, and holdouts with temporal metrics.
- Test unsupported numeric/qualitative estimates, survivorship bias, scope invalidation, clock skew/rollback, DST, restart, deadline mutation, budget-renewal bypass, fake progress, repeated actions, stuck tools, concurrency overlap, alert fatigue, metric gaming, and urgency safety.
- Test calendar privacy, redaction, authority, malicious prompt/tool/provider time claims, and agents attempting to improve scores by category manipulation.
- Implement and prove precision profiles, integer high-resolution timing, uncertainty propagation, P50/P95/P99/P99.9/max latency, sequence/causation ordering, and deployed-path calibration.
- Implement the Markets domain temporal contract and simulation/paper/shadow/canary capability gates; test stale/gapped data, market calendars, clock drift, overload, timeout-after-effect, duplicate order, partial fill, cancel race, halt, disconnect, restart, reconciliation, and kill switch.
- Require equivalent declared temporal profiles and negative proof for every other high-consequence domain pack.
- Report quality-adjusted time to settled outcome and tail latency/uncertainty, not raw speed or average latency alone.

### Slice 12 — Migration, documentation, conformance, and final closure

- Migrate legacy turn-local timing, Work Loop epochs, Silent Session run fields, Project Card outcomes, benchmark ledgers, and compatible historical timestamps with explicit confidence/degradation.
- Update architecture, all affected specs' integration clauses, current API/CLI references, tool docs/registry, operator guides, troubleshooting, doctor, privacy, accessibility, migration, and developer guides.
- Execute the complete feature ledger, cross-system parity matrix, restart/replay/CRDT matrix, cross-platform capability matrix, Spec 136 integration matrix, and 293-MUST mapping.
- Produce final Evidence bundle, proof matrix, Completion Receipt, temporal calibration report, and explicit zero-unapproved-deferral/zero-omission attestation.
- Final gate fails if any required ledger row is open, blocked, prose-only, mocked, disabled, client-incomplete, undocumented, untested, unreceipted, or silently deferred.

## Machine-readable delivery artifacts

Required before implementation decomposition closes:

```text
docs/contracts/spec131-complete-feature-ledger.v1.yaml
docs/contracts/spec131-delivery-dag.v1.yaml
docs/contracts/spec131-temporal-state-machine.v1.yaml
docs/contracts/spec131-reason-codes.v1.yaml
docs/contracts/spec131-cross-spec-ownership.v1.yaml
docs/contracts/spec131-cross-surface-parity.v1.yaml
docs/contracts/spec131-conformance-matrix.v1.yaml
```

Every task generated from this specification includes:

```yaml
requirement_refs: []
primitive_owner:
implementation_slice:
blocking_refs: []
scope_model:
clock_domains: []
deadline_behavior:
estimate_behavior:
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
spec136_mapping:
security:
privacy:
accessibility:
migration:
positive_tests: []
negative_tests: []
clock_tests: []
restart_recovery_tests: []
crdt_tests: []
adversarial_tests: []
evidence: []
receipts: []
definition_of_done: []
not_done_if: []
excluded_requirement_refs: []
```

`excluded_requirement_refs` MUST be empty unless each entry points to an explicit operator-approved specification amendment.

## Final closure law

Spec 131 is complete only when Focusa can prove that:

- the agent and operator continuously receive fresh, truthful, scoped time awareness;
- human deadlines and protected readiness targets cannot be silently reset or consumed;
- unsupported estimates cannot reach any user-facing surface;
- material progress and activity are structurally distinct;
- avoidable waste is detected, recorded, consequential, and learnable only after governed settlement;
- every affected system consumes one canonical temporal authority;
- functional success cannot hide temporal failure;
- urgency improves time-to-verified-outcome without weakening any safety or truth boundary;
- every mandatory requirement is implemented and evidenced with no silent deferral or omission.

The governing watchword remains:

> **THE CALENDAR AND THE CLOCK NEVER WAIT. TO BE EARLY IS TO BE ON TIME; TO BE ON TIME IS TO BE LATE.**
