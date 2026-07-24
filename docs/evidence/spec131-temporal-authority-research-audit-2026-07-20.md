# Spec 131 Temporal Authority Research and Contradiction Audit — 2026-07-20

## Status and scope

- Audit time: 2026-07-20 15:28:01 PDT (UTC-0700)
- Project: `$PWD` (isolated Focusa worktree)
- Branch: `local/work-loop-completion`
- Audited HEAD: `e1d71ee4f2a859b3b633888190eee52e5f40f083`
- Audited file: `docs/131-focusa-workpoint-item-timing-velocity-and-closure-authority-spec.md`
- Audited SHA-256: `8814fbe63c31c79d6dd3920887932c2081d9e7cf3a1fd5e11e82768c8af80b79`
- Remote proposed Spec 136 inspected but not merged: commit `19898df8d0c3bac632e3e4b44ca1ab9367b595c7`, document SHA-256 `7309d1b1bf39c30d56e061f1331723883a65758e4bdca58f85802ffadf0ee77e`
- Method: sequential UIAI Engine Source-to-Markdown through the current-capacity-gated workaround documented in `WPUIAI/uiai-engine#2`; no automatic retries or concurrent browser fan-out.

## Executive verdict

Spec 131 has a strong authority model and correctly separates wall time, monotonic duration, budgets, leases, TTLs, evidence freshness, and provider time. It also correctly rejects timestamp precision as proof of accuracy, distinguishes event time from human commitment time, prohibits model-driven latency-critical trading, and requires reconciliation after uncertain external effects.

At the audited SHA, it was **not internally airtight or implementation-ready**. This audit found:

- 8 contradiction or authority conflicts requiring normative correction;
- 9 overconstraint/ambiguity risks capable of producing deadlock, unsafe narrowing, or unusable clients;
- 37 material missing contracts spanning clock trust, civil time, deadline propagation, uncertainty, forecasting, human factors, markets, privacy, metrics, and dependency/version authority.

The amended working-tree draft subsequently incorporated a documented disposition for every `C-01..C-08`, `A-01..A-09`, and `G-01..G-37` finding. The post-amendment ledger below distinguishes resolved normative structure from operator/domain policy values that remain intentionally unknown, the blocked Spec 136 dependency, and all unimplemented proof obligations.

No runtime, implementation, release, 293/293, regulatory-conformance, or live-market readiness conclusion follows from either the audit or the amendments.

## Authoritative research evidence

| Source | Material requirement | UIAI evidence |
| --- | --- | --- |
| [W3C High Resolution Time](https://w3c.github.io/hr-time/) | Wall clocks can move backward; monotonic clocks are for measurement, are execution-local, and timer resolution may be coarsened for security/privacy. | `uiai-source-markdown:sha256:89460701e35aff68` |
| [POSIX.1-2024 `clock_gettime`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_gettime.html) | `CLOCK_REALTIME` is settable; `CLOCK_MONOTONIC` is non-settable and has an unspecified epoch; clock resolution is implementation-defined. | `uiai-source-markdown:sha256:be07088ab87ffdbd` |
| [Linux `clock_gettime(2)`](https://man7.org/linux/man-pages/man2/clock_gettime.2.html) | `CLOCK_MONOTONIC` excludes suspend, `CLOCK_BOOTTIME` includes suspend, and `CLOCK_TAI` can degrade to realtime behavior without NTP support. | `uiai-source-markdown:sha256:60d60985649b1be5` |
| [RFC 5905, NTPv4](https://www.rfc-editor.org/rfc/rfc5905.html) | Offset, delay, dispersion, jitter, root delay, root dispersion, and synchronization distance represent distinct error components; displayed precision is not an error bound. | `uiai-source-markdown:sha256:c1acd0bf322031e7` |
| [RFC 8633, NTP BCP](https://www.rfc-editor.org/rfc/rfc8633.html) | Accurate operators should use at least four independent, diverse sources, monitor disagreement, and explicitly handle leap-smear disagreement. | `uiai-source-markdown:sha256:916b8ffec58eb465` |
| [RFC 8915, Network Time Security](https://www.rfc-editor.org/rfc/rfc8915.html) | Time synchronization needs authenticated key establishment, authenticated encryption, replay detection, and request-response consistency. | `uiai-source-markdown:sha256:5867a8e178f90e8b` |
| [NIST TN 1297](https://www.nist.gov/pml/nist-technical-note-1297) | Measurement uncertainty requires a defined method, components, combined/expanded uncertainty, coverage factor, and reporting discipline. | `uiai-source-markdown:sha256:3e1565ccd8615d45`, `uiai-source-markdown:sha256:908ae952d37999bc` |
| [IANA tzdb theory](https://data.iana.org/time-zones/tzdb/theory.html) | Civil-time rules and future predictions change; offsets are not stable timezone identities; tzdb is not itself authoritative for every jurisdiction. | `uiai-source-markdown:sha256:1e896f2282b26e6a` |
| [RFC 9557](https://www.rfc-editor.org/rfc/rfc9557.html) | A local civil time may map to zero or multiple instants; future local-time intent is not equivalent to an already-fixed UTC timestamp; offset/zone inconsistencies require detection and policy. | `uiai-source-markdown:sha256:8d8c504509e72e4c` |
| [gRPC deadlines](https://grpc.io/docs/guides/deadlines/) | Calls have no deadline by default; propagated deadlines should become remaining timeouts with elapsed time deducted to avoid cross-host clock-skew errors; cancellation must stop spawned work. | `uiai-source-markdown:sha256:a5574d5e8cc1de9b` |
| [Lamport, 1978](https://www.microsoft.com/en-us/research/publication/time-clocks-ordering-events-distributed-system/) | Distributed events have a causal partial order; wall-clock timestamps cannot establish an invariant total order. | `uiai-source-markdown:sha256:1917e8c361d9b518` |
| [SEC Rule 15c3-5](https://www.ecfr.gov/current/title-17/chapter-II/part-240/section-240.15c3-5) | Pre-order controls must reject limit breaches, erroneous/duplicate orders, and unauthorized/restricted activity; controls require direct ownership, documented review, annual review, and certification. | `uiai-source-markdown:sha256:62ab9051b6489ffe` |
| [SEC Regulation SCI, 17 CFR 242.1001](https://www.ecfr.gov/current/title-17/chapter-II/part-242/section-242.1001) | SCI systems require capacity, integrity, resiliency, availability, security, testing methodology, and business-continuity/disaster-recovery capabilities. | `uiai-source-markdown:sha256:9054efd244fd7e8c` |
| [SEC CAT Rule 613](https://www.ecfr.gov/current/title-17/chapter-II/part-242/section-242.613) | Audit records must be accurate and time-sequenced across order receipt/origination, routing, modification, cancellation, and execution; business clocks require synchronization. | `uiai-source-markdown:sha256:ce8bf46255ecdcbd` |
| [EU RTS 25, Regulation 2017/574](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32017R0574) | Clocks must be traceable to UTC; exact timestamp application points must be identified and stable; compliance is reviewed annually; accuracy/granularity thresholds depend on activity/system class. | `uiai-source-markdown:sha256:47136e754533ef82` |
| [WMO forecast-verification guidance](https://www.cawcr.gov.au/projects/verification/) | Forecast evaluation distinguishes quality, value, bias, skill, reliability, resolution, sharpness, discrimination, uncertainty, observational error, sample size, and error bounds. | `uiai-source-markdown:sha256:79387a620647394f` |
| [Maule, Hockey, and Bdzola, indexed by PubMed](https://pubmed.ncbi.nlm.nih.gov/10900697/) | Time pressure changes affective state and coping/information-processing strategy; effects and risk-taking vary with task structure rather than following one universal response. | `uiai-source-markdown:sha256:b771fabf8a793679` |

### Transparently unusable captures

- The canonical W3C `/TR/` page returned a challenge page; the official W3C GitHub publication was used instead.
- FINRA Rules 4590/6820 were blocked by a challenge page. No claim in this audit relies on an unseen FINRA rule body.
- One PubMed meta-analysis page returned a reCAPTCHA challenge and was not cited.
- HSE candidate pages were empty/404 and were not cited.

## Contradiction and authority-conflict matrix

| ID | Spec location | Conflict | Consequence | Required correction |
| --- | --- | --- | --- | --- |
| C-01 | Lines 1427-1434 versus lines 122-181 and 2144-2160 | Closure permits `operator_override` without evidence and `degraded_allowed`, while the omission firewall says evidence cannot be waived and degraded mode cannot become completion. | An operator action could rewrite an unproven implementation as done. | Separate **verified completion** from **operator disposition**. Override may cancel, accept risk, or waive a policy obligation, but cannot manufacture evidence or set `verified_complete`. Remove `degraded_allowed` as a completion state. |
| C-02 | Requirement ledger lines 122-181 | The ledger includes `should` and `may`, but has no applicability/not-applicable state; excluded refs require an amendment and every normative row is implied to block closure. | `MAY` is effectively converted into `MUST`, while inapplicable platform/domain requirements cannot be represented truthfully. | Add `applicability`, `applicability_basis`, and `not_applicable_verified`; define which RFC 2119 classes block release. |
| C-03 | Lines 1315-1370 and trusted-clock rules | `monotonic_elapsed_ms` is described as spanning “linked boot epochs,” but a monotonic clock has no comparable epoch across reboot. Wall samples can bridge calendar gaps only with their own uncertainty; they do not create cross-boot monotonic time. | False exact duration can be manufactured across reboot or clock correction. | Store per-boot monotonic intervals; represent inter-epoch gap separately as wall-derived bounded/unknown duration. Never label a cross-boot sum as pure monotonic elapsed. |
| C-04 | Lines 219-303 and 373-429 | Human deadlines are required to be timezone-aware absolute instants, but future civil-time commitments can change under tzdb/jurisdiction revisions and ambiguous/nonexistent local times can map to multiple/zero instants. | Converting too early can silently alter the operator’s intended deadline. | Distinguish fixed instant, civil-time intent, floating time, recurrence, and external-calendar event. Preserve original civil expression and resolution policy alongside any current instant projection. |
| C-05 | Lines 972-1155 | `critical` requires a single critical objective, while top-deadline ranking and overdue exceptions admit multiple simultaneous higher-consequence commitments. No deterministic conflict resolution or infeasibility state is defined. | The system can oscillate, starve a severe commitment, or choose arbitrarily. | Add `deadline_conflict`/`schedule_infeasible`, deterministic tie-breaks, preemption rules, and explicit operator escalation. |
| C-06 | Lines 972-1001 versus 429 and 692-776 | `normal` includes unknown margin “without immediate risk,” while unknown critical-path duration makes slack unknown and missing deadline context can block consequential work. The spec does not define how immediate risk is proven when duration is unknown. | Unknown can be treated optimistically and suppress warning. | Require consequence- and proximity-sensitive uncertainty posture; unknown slack near a hard deadline cannot default to normal. |
| C-07 | Lines 303-368 versus 600-709 | Every action/dispatch must carry fresh HumanCalendarContext/TemporalPriorityFrame, while millisecond/microsecond-critical market execution must be deterministic and outside conversational/LLM paths. The required local precomputed enforcement contract is absent. | A critical executor may either violate the “every action” law or add unsafe network/daemon latency before dispatch. | Define a pre-authorized, locally cached, monotonic `TemporalExecutionLease/Guard` with bounded validity and asynchronous audit linkage; forbid remote refresh on the critical path. |
| C-08 | Lines 788-868 versus final closure law | Operator-supplied expectations are permitted and labeled, but final closure says unsupported estimates cannot reach any user-facing surface. | Clients cannot know whether a labeled operator expectation may be shown. | Distinguish measured forecast, operator target/expectation, commitment, budget, and refused forecast; prohibit only unlabeled/invalid forecast claims. |

## Overconstraint and ambiguity risks

| ID | Risk | Correction |
| --- | --- | --- |
| A-01 | Invalidating the priority frame after every tool completion can require a daemon round trip between all operations and recursively involve temporal-repair actions. | Define local cache validity, maximum staleness by operation class, generation bootstrap, and no-recursion recovery semantics. |
| A-02 | Requiring HumanCalendarContext refs on every reducer command creates privacy, retention, storage, and hot-path coupling even when calendar facts cannot affect the command. | Require a minimal temporal-context hash/projection; include detailed calendar refs only when semantically relevant. |
| A-03 | The estimate validator treats `most of the work` and `nearly done` as duration forecasts even when they are evidence-backed scope/completion claims. | Split `ForecastClaim`, `ProgressClaim`, `CommitmentClaim`, and `DeadlineFact` validators. |
| A-04 | Overdue mode says to stop “optional tests,” but optionality can be misclassified under pressure. | Define non-skippable acceptance, safety, security, reconciliation, and regression checks in the closure contract before urgency begins. |
| A-05 | Increasing pulse/checkpoint frequency can itself create load, flapping, and operator noise. | Add hysteresis, debounce, minimum intervals, deduplication, backpressure, and notification budgets. |
| A-06 | Automatic narrowing under pressure can remove disconfirming evidence or alternate safe routes; human-factors evidence does not support one universal coping strategy. | Maintain a protected safety/evidence checklist and permit bounded challenge/review paths at every pressure level. |
| A-07 | `max_operator_silence_ms` can turn normal unavailability into pressure or repeated prompts. | Bind to operator availability, quiet hours, severity, escalation policy, and notification consent. |
| A-08 | Average-based velocity fields imply precision and are unstable under heterogeneous or small cohorts. | Require cohort size, distribution/quantiles, censoring posture, uncertainty, and task-normalized comparisons. |
| A-09 | Workpoint Item throughput is gameable by item splitting/merging. | Version decomposition rules and report outcome/target-state-normalized throughput, not raw item count alone. |

## Missing-gap matrix

### Clock trust, measurement, and platform semantics

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-01 | Authenticated and diverse time-source policy | RFC 8633; RFC 8915 | Add source count/diversity, source identity, NTS/auth posture, replay/request-response verification, disagreement/quorum policy, and source-removal incidents. |
| G-02 | Complete clock-error model | RFC 5905; NIST TN 1297 | Add measured offset, delay, jitter, dispersion, root dispersion/distance, frequency error, holdover age, synchronization age, uncertainty components, method, coverage factor, and confidence/coverage level. |
| G-03 | OS capability/suspend matrix | POSIX; Linux man-pages; W3C | Map each clock domain to tested OS clocks. Distinguish suspend-excluding monotonic, suspend-including boottime/continuous clocks, CPU time, realtime, and TAI capability/fallback. |
| G-04 | Paired sample lineage | W3C; POSIX | Every authoritative conversion needs paired wall/monotonic samples, capture order/latency, boot ID, source/profile refs, uncertainty, and correction lineage. |
| G-05 | Uncertainty-aware threshold comparison | NIST TN 1297 | Define `definitely_before`, `possibly_crossed`, `definitely_crossed`, and `indeterminate` when the time uncertainty interval approaches/straddles a deadline. |
| G-06 | Precision exposure policy | W3C | High-resolution timestamps must be capability- and audience-scoped, coarsened for untrusted clients/models, and protected against timing/privacy side channels. |

### Civil time, calendars, and deadline semantics

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-07 | Civil-time intent schema | RFC 9557; IANA tzdb | Store original local date/time, IANA zone, tzdb version, jurisdiction/calendar source, fold/gap disambiguation, floating/fixed/recurring kind, and resolution history. |
| G-08 | Rule-change re-resolution | IANA tzdb; RFC 9557 | On tzdb/calendar change, recompute projected instants, detect differences, preserve old/new projections, and require policy/operator resolution where material. |
| G-09 | Boundary semantics | RFC 9557 | Deadlines/windows need inclusive/exclusive endpoints, grace rules, start/end authority, submission-versus-receipt target, and acknowledgment/settlement target. |
| G-10 | External deadline authority | Legal/market deadline model | Separate observation/revision of Focusa’s record from authority to change the external commitment. Operators cannot “clear” an immutable legal/venue cutoff; they can cancel the associated objective or correct the record with evidence. |
| G-11 | Calendar source conflict/versioning | IANA tzdb; RTS 25 | Define source precedence, version, fetch time, freshness, signatures where available, disagreement posture, and fail-closed policy for market/legal calendars. |

### Distributed deadlines, cancellation, and ordering

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-12 | Deadline propagation | gRPC | At process/RPC boundaries propagate remaining duration, subtract elapsed time, cap child deadline at parent remaining budget, and preserve original absolute deadline for audit. |
| G-13 | Cancellation propagation and acknowledgment | gRPC | Define cancellation tokens, child registration, periodic checks, grace period, force termination, cleanup/reconciliation, and `cancel_requested/observed/effective` timestamps. |
| G-14 | Timeout/retry budget | gRPC; existing unknown-outcome law | Allocate retries within the original deadline, apply backoff/jitter where applicable, and prohibit retry after possible external effect until reconciliation. |
| G-15 | Explicit causal-order representation | Lamport | Add event sequence, parent/causal refs, per-stream monotonic sequence/fencing, and partial-order conflict states; never infer causality from wall timestamp order. |
| G-16 | Suspend/reboot expiry behavior | POSIX/Linux | Map authority expiry, leases, security TTLs, and evidence freshness to clocks that continue through suspend/reboot as required; define recovery when the trusted bridge is unavailable. |

### Estimates and calibration

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-17 | Calibration metric definitions | WMO guidance | Define reliability, bias, coverage, sharpness, skill against a baseline, and decision value; replace the untyped single `calibration_score`. |
| G-18 | Tail-risk quantiles and risk appetite | WMO; deadline doctrine | Add policy-selected quantiles (including high tails for hard/high-consequence deadlines), interval coverage, and asymmetric early/late loss. |
| G-19 | Verification uncertainty | WMO; NIST | Report sample size, error bounds, observation/closure-target uncertainty, missingness, and cohort drift. |
| G-20 | Censoring/dependency methodology | Forecasting practice | Specify survival/censoring treatment and correlated dependency-DAG simulation; do not add independent task ranges naively. |
| G-21 | Typed confidence semantics | NIST/WMO | Map `low/medium/high` to explicit evidence/calibration thresholds or remove qualitative confidence from authority-bearing output. |
| G-22 | Forecast value versus quality | WMO | Evaluate whether a forecast improves the operator decision under stated costs; accuracy alone cannot justify display or promotion. |

### Human factors and urgency safety

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-23 | Pressure-transition hysteresis and anti-flap controls | Operational safety | Add enter/exit thresholds, dwell time, deduplication, notification rate limits, and pulse resource budgets. |
| G-24 | Protected cognitive forcing/checklists | PubMed time-pressure evidence; high-consequence safety model | Preserve mandatory challenge checks, disconfirming evidence, independent review/two-person control where required, and stop-work authority. |
| G-25 | Human handoff/fatigue posture | Human-factors model | Add operator/agent workload, shift/handoff, sustained critical duration, fatigue/attention limits, and escalation to a fresh reviewer for high-consequence work. |

### Financial-market and high-consequence profiles

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-26 | Control ownership, review, and certification | SEC 15c3-5 | Add direct/exclusive control owner, delegated-control contract/due diligence, written review procedure, review cadence, issue remediation, certification, and records retention. |
| G-27 | SCI resilience/BCDR profile | SEC Regulation SCI | Add capacity thresholds, integrity/availability objectives, geographically diverse recovery, RTO/RPO, wide-scale disruption tests, and incident notification/records. |
| G-28 | Timestamp application-point consistency | EU RTS 25 | Identify exact hardware/software capture point, prove it remains consistent, preserve system design/specification, and require periodic/annual traceability review. |
| G-29 | Jurisdiction/activity-specific thresholds | EU RTS 25 | Domain packs must bind precision/accuracy/granularity to operation class and jurisdiction; generic microsecond formatting cannot imply compliance. |
| G-30 | Regulatory lifecycle and applicability | SEC/EU rules | Record rule/venue version, applicability decision, effective dates, retention, required reviewer/certifier, and supersession/migration evidence. |

### Security, privacy, metrics, and dependency authority

| ID | Missing contract | Source basis | Proposed amendment |
| --- | --- | --- | --- |
| G-31 | Temporal-data security and privacy lifecycle | W3C timer privacy; calendar sensitivity | Classify clock/calendar/activity/strategy data; define least privilege, encryption, redaction, retention/deletion, audit access, aggregation minimums, and export controls. |
| G-32 | Tamper-evident temporal evidence | NTS/NIST trust model | Sign/hash-chain authoritative samples, correction events, deadline revisions, and Receipts; distinguish secure time-source authentication from later ledger integrity. |
| G-33 | Metric distributions and anti-gaming | WMO/forecast guidance | Require quantiles, cohort/sample counts, uncertainty, exclusions, revision lineage, split/merge detection, and quality/safety guardrails for velocity comparisons. |
| G-34 | Remote Spec 136 dependency lock | Current branch state | A local canonical spec cannot normatively depend on an unversioned “remote proposed” document. Vendor/merge an approved contract or pin immutable schema/hash with explicit blocked status. |
| G-35 | Complete temporal fault-injection matrix | All clock sources | Require deterministic tests for wall step forward/back, slew, source disagreement/spoof/replay, leap/smear, DST fold/gap, tzdb revision, suspend, reboot, daemon downtime, uncertainty crossing, queue delay, cancellation races, and stale calendar/data. |
| G-36 | Policy/version lineage in every decision | Cross-system authority | Clock, calendar, estimate, urgency, deadline, and market decisions must record the exact policy/schema/adapter versions used so replay and settlement can reproduce the result. |
| G-37 | `crdt_tests` ownership and semantics | Lines 2076-2142 | Define the actual multi-writer/merge model requiring CRDT proof, or remove this unexplained mandatory task field in favor of reducer/CAS/fencing/replay tests. |

## Line-targeted amendment plan

1. **Lines 182-303 — trusted clocks:** introduce `ClockTrustProfile`, `ClockSamplePair`, source-security/diversity, uncertainty method/coverage, holdover, and OS capability mapping.
2. **Lines 249-303 — precision/privacy:** add precision-exposure and uncertainty-aware comparison laws.
3. **Lines 373-429 — deadlines:** split fixed instant from civil intent; add external authority, interval boundaries, calendar/tzdb versions, conflict/infeasibility, and re-resolution.
4. **After line 429 — distributed enforcement:** add remaining-time propagation, cancellation acknowledgment, child deadline capping, and retry-budget semantics.
5. **Lines 515-788 — awareness:** define cached local guard semantics by operation class and remove detailed calendar coupling from irrelevant hot-path commands.
6. **Lines 788-868 — estimate gate:** split forecast/progress/commitment/deadline claims and replace scalar calibration with a versioned evaluation profile.
7. **Lines 972-1155 — urgency:** add hysteresis, protected checklists, multi-deadline conflict handling, fatigue/handoff, and notification budgets.
8. **Lines 1302-1370 — timing ledger:** make intervals per-boot, add paired sample/profile refs and elapsed uncertainty, and represent cross-boot gaps separately.
9. **Lines 1400-1437 — closure:** prohibit override from producing verified completion; create explicit cancelled/waived/accepted-risk dispositions.
10. **Lines 1459-1522 — velocity:** replace averages/single accuracy with distributions, cohorts, error bounds, baselines, and anti-gaming controls.
11. **Lines 1523-1616 — Spec 136:** replace the remote-proposed dependency with immutable local authority or mark integration blocked.
12. **Lines 303-368 and Markets acceptance — market controls:** add SEC control ownership/review/certification, Regulation SCI BCDR/capacity, and RTS 25 timestamp-point/traceability proof.
13. **Lines 122-181 and 2076-2142 — feature ledger:** add applicability semantics, policy-version refs, complete fault-injection requirements, and resolve undefined CRDT ownership.

## Post-amendment finding disposition

Disposition time: 2026-07-20 17:38:04 PDT (UTC-0700).

Canonical decision record: `docs/contracts/spec131-inferred-decision-register.v1.yaml`.

Interpretation:

- `resolved in amended normative draft` means the contradiction/gap now has explicit normative semantics in Spec 131; it does **not** mean code or conformance exists.
- `structure resolved; policy value explicitly reserved` means the schema, authority, fail-closed behavior, and activation gate are specified, while an operator/domain owner must supply a genuinely policy-owned value before that capability can activate.
- `resolved by explicit blocked-dependency rule` means the draft is internally coherent because it prohibits pretending that the missing dependency is satisfied; the dependency itself remains unavailable.
- Every row retains an open implementation/conformance obligation unless its residual boundary says otherwise.

| ID | Post-amendment disposition | Normative location | Residual boundary |
| --- | --- | --- | --- |
| C-01 | resolved in amended normative draft | Core temporal laws; Closure Authority; Workpoint Item; Final closure law | implementation and conformance proof remain open |
| C-02 | resolved in amended normative draft | Completeness non-deferral and omission firewall; Machine-readable delivery artifacts | implementation and conformance proof remain open |
| C-03 | resolved in amended normative draft | Trusted clock and temporal authority; Work Timing Ledger | implementation and conformance proof remain open |
| C-04 | resolved in amended normative draft | Deadline and calendar contract; Civil-time intent and resolution | implementation and conformance proof remain open |
| C-05 | resolved in amended normative draft | Deadline inheritance probabilistic slack and conflict; Calm focus gradient | implementation and conformance proof remain open |
| C-06 | resolved in amended normative draft | Deadline inheritance probabilistic slack and conflict; Temporal pressure and urgency policy | implementation and conformance proof remain open |
| C-07 | resolved in amended normative draft | Deterministic execution boundary; Time-sensitive action law; Mandatory use | implementation and conformance proof remain open |
| C-08 | resolved in amended normative draft | Temporal claim types Estimate Claim and conversational response gate | implementation and conformance proof remain open |
| A-01 | resolved in amended normative draft | Mandatory use | implementation and conformance proof remain open |
| A-02 | resolved in amended normative draft | Human calendar grounding; Temporal security privacy integrity and retention | implementation and conformance proof remain open |
| A-03 | resolved in amended normative draft | Temporal claim types Estimate Claim and conversational response gate | implementation and conformance proof remain open |
| A-04 | resolved in amended normative draft | Calm focus gradient; Closure Authority | implementation and conformance proof remain open |
| A-05 | resolved in amended normative draft | Temporal pulse policy; Calm focus gradient | implementation and conformance proof remain open |
| A-06 | resolved in amended normative draft | Calm focus gradient | implementation and conformance proof remain open |
| A-07 | structure resolved; policy value explicitly reserved | Calm focus gradient | operator/domain owner must supply applicable value before activation |
| A-08 | resolved in amended normative draft | Velocity metrics | implementation and conformance proof remain open |
| A-09 | resolved in amended normative draft | Velocity metrics | implementation and conformance proof remain open |
| G-01 | resolved in amended normative draft | Trusted clock and temporal authority | implementation and conformance proof remain open |
| G-02 | resolved in amended normative draft | Trusted clock and temporal authority; Precision accuracy resolution and uncertainty | implementation and conformance proof remain open |
| G-03 | resolved in amended normative draft | Trusted clock and temporal authority | implementation and conformance proof remain open |
| G-04 | resolved in amended normative draft | Trusted clock and temporal authority | implementation and conformance proof remain open |
| G-05 | resolved in amended normative draft | Precision accuracy resolution and uncertainty; Deadline and calendar contract | implementation and conformance proof remain open |
| G-06 | resolved in amended normative draft | Precision accuracy resolution and uncertainty; Temporal security privacy integrity and retention | implementation and conformance proof remain open |
| G-07 | resolved in amended normative draft | Civil-time intent and resolution | implementation and conformance proof remain open |
| G-08 | resolved in amended normative draft | Civil-time intent and resolution | implementation and conformance proof remain open |
| G-09 | resolved in amended normative draft | Deadline and calendar contract | implementation and conformance proof remain open |
| G-10 | resolved in amended normative draft | Deadline and calendar contract | implementation and conformance proof remain open |
| G-11 | resolved in amended normative draft | Civil-time intent and resolution | implementation and conformance proof remain open |
| G-12 | resolved in amended normative draft | Distributed deadline cancellation and retry propagation | implementation and conformance proof remain open |
| G-13 | resolved in amended normative draft | Distributed deadline cancellation and retry propagation | implementation and conformance proof remain open |
| G-14 | resolved in amended normative draft | Distributed deadline cancellation and retry propagation | implementation and conformance proof remain open |
| G-15 | resolved in amended normative draft | Precision accuracy resolution and uncertainty; Work Timing Ledger | implementation and conformance proof remain open |
| G-16 | resolved in amended normative draft | Clock domains; Trusted clock and temporal authority | implementation and conformance proof remain open |
| G-17 | resolved in amended normative draft | ForecastCalibrationProfile; Velocity metrics | implementation and conformance proof remain open |
| G-18 | structure resolved; policy value explicitly reserved | ForecastCalibrationProfile; Deadline inheritance probabilistic slack and conflict | operator/domain owner must supply applicable value before activation |
| G-19 | resolved in amended normative draft | ForecastCalibrationProfile | implementation and conformance proof remain open |
| G-20 | resolved in amended normative draft | EstimateClaim; Deadline inheritance probabilistic slack and conflict | implementation and conformance proof remain open |
| G-21 | resolved in amended normative draft | EstimateClaim | implementation and conformance proof remain open |
| G-22 | resolved in amended normative draft | ForecastCalibrationProfile | implementation and conformance proof remain open |
| G-23 | resolved in amended normative draft | Temporal pulse policy | implementation and conformance proof remain open |
| G-24 | resolved in amended normative draft | Calm focus gradient | implementation and conformance proof remain open |
| G-25 | structure resolved; policy value explicitly reserved | Calm focus gradient | operator/domain owner must supply applicable value before activation |
| G-26 | resolved in amended normative draft | Live-market activation firewall | implementation and conformance proof remain open |
| G-27 | structure resolved; policy value explicitly reserved | Live-market activation firewall | operator/domain owner must supply applicable value before activation |
| G-28 | resolved in amended normative draft | Live-market activation firewall | implementation and conformance proof remain open |
| G-29 | structure resolved; policy value explicitly reserved | Live-market activation firewall | operator/domain owner must supply applicable value before activation |
| G-30 | resolved in amended normative draft | Live-market activation firewall | implementation and conformance proof remain open |
| G-31 | structure resolved; policy value explicitly reserved | Temporal security privacy integrity and retention | operator/domain owner must supply applicable value before activation |
| G-32 | resolved in amended normative draft | Temporal security privacy integrity and retention | implementation and conformance proof remain open |
| G-33 | resolved in amended normative draft | Velocity metrics | implementation and conformance proof remain open |
| G-34 | resolved by explicit blocked-dependency rule | Status; Spec 136 proposal-to-settlement integration | local adoption remains blocked pending operator approval and immutable contract |
| G-35 | resolved in amended normative draft | Acceptance criteria; Slice 11 | implementation and conformance proof remain open |
| G-36 | resolved in amended normative draft | Acceptance criteria; Storage | implementation and conformance proof remain open |
| G-37 | resolved in amended normative draft | Cross-system coherence requirements; Machine-readable delivery artifacts | implementation and conformance proof remain open |

All 54 finding IDs occur exactly once in the decision register and once in this disposition table. Six findings (`A-07`, `G-18`, `G-25`, `G-27`, `G-29`, `G-31`) intentionally reserve deployment/operator/domain values while specifying fail-closed structure. `G-34` intentionally leaves Spec 136 adoption blocked. No finding was silently rejected, deferred, or represented as implemented.

## What is already correct and should be preserved

- Separate semantic domains for wall time, monotonic elapsed, budgets, leases, authority expiry, security TTL, evidence freshness, and provider time.
- Runtime calibration rather than precision-by-format claims.
- Timestamp ordering is insufficient for distributed causality.
- Human commitments and machine event time remain related but distinct.
- Readiness targets do not rewrite external deadlines.
- Late functional success retains the temporal breach.
- Possible external effect requires reconciliation before retry.
- LLMs remain outside latency-critical market execution.
- Failed, abandoned, blocked, reopened, rolled-back, and censored attempts remain in forecast history.
- Urgency cannot weaken safety, scope, evidence, authority, reconciliation, or settlement.
- No silent omission, prose-only deferral, mock proof, or degraded-but-complete release claim.

## Post-amendment validation record

Validation time: 2026-07-20 17:56:31 PDT (UTC-0700).

Validated artifact hashes:

- amended Spec 131: `sha256:ee0e72ef66468109733ad98c76c068d2d089d7af84d90ff367cc9cd393d2dd00`;
- complete feature ledger: `sha256:7594e587f8e9f859062fbfa4e32764690b2afdd942716797a250a0e6b8103f9b`;
- inferred-decision register: `sha256:2728dc34eb315fdfdd8fa946cfdafd2ec5f258d946c9570bd4572d74a4035aa1`.

Static validation passed:

- exact acceptance sequence `1..86` and exact `S131-REQ-001..S131-REQ-086` text/hash mapping;
- exact finding sequence `C-01..C-08`, `A-01..A-09`, and `G-01..G-37` in both the decision register and post-amendment disposition table;
- 42/42 top-level Spec 131 headings covered by inherited normative source mappings;
- all ledger-required fields present; 74 rows are required and 12 conditional;
- truthful ledger status: 84 `not_started`, one `active` completeness-ledger row, one `blocked` Spec 136 row, zero `verified` rows;
- 96 Markdown fence markers balanced and all 33 fenced YAML/JSON examples parsed;
- 17 unique authoritative source links and 18 unique UIAI evidence refs present;
- all required local document refs resolve except the explicitly declared absent proposed Spec 136 path;
- Spec 131 consumption/authority alignment present in 10 directly affected normative, implementation-note, compaction, Work Loop, Workpoint, CRDT, closure, Receipt, Silent Session, and closure-design documents;
- stale `degraded_allowed`/`closure_override` completion states absent from the aligned closure documents;
- `git diff --check` passed for the complete temporal documentation set.

Validation limitations and disclosed degraded state:

- this was structural/semantic documentation validation, not code generation, compilation, runtime, migration, restart, fault-injection, regulatory, release, or live-market proof;
- external sources were not re-fetched during this post-amendment pass; the audit relies on the 18 already captured UIAI evidence refs, and the transparently unusable captures remain unusable;
- Focusa ProjectIdentity verified the exact repository root with high confidence, but two post-compaction Workpoint checkpoint writes timed out and returned only degraded cached fallback; no canonical Workpoint checkpoint is claimed;
- operator/domain policy values enumerated in the decision register remain intentionally unknown, and no external deadline or protected review margin is recorded.

## Acceptance posture

Spec 131 remains a normative documentation draft. Post-amendment documentation disposition is:

1. `C-01..C-08`, `A-01..A-09`, and `G-01..G-37` each have an explicit decision and normative destination;
2. the inferred-decision register makes confidence semantics and operator-reserved values machine-readable;
3. directly affected Spec 55 normative/implementation notes, Specs 79, 88, 98, 116, 119, 130, and 133, plus the closure-authority design note are aligned to consume rather than duplicate Spec 131 authority;
4. Spec 136 remains hash-pinned as inspected evidence but blocked as a local dependency; this is a valid integrity disposition, not dependency completion;
5. `spec131-complete-feature-ledger.v1.yaml` now defines 86 normalized requirement rows and maps all top-level normative sections plus all 54 findings, but 84 rows remain `not_started`, one is `active`, and the Spec 136 integration row is `blocked`; state-machine/reason-code artifacts, implementation fields, migrations, runtime tests, fault injection, Receipts, and conformance evidence remain open deliverables governed by the amended specification;
6. no external deadline, review margin, risk threshold, activated jurisdiction/venue/account, retention period, quiet-hours/escalation policy, Spec 136 adoption, or live-market authorization was inferred.

Therefore the bounded conclusion is: **the identified specification contradictions, ambiguities, and missing normative contracts have documented dispositions in the amended draft, but the draft is not implementation proof, dependency approval, regulatory certification, release proof, 293/293 proof, or live-market readiness.**
