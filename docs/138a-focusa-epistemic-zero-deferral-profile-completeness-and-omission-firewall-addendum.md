# Spec 138A — Focusa Epistemic Zero-Deferral, Profile Completeness, and Omission Firewall Addendum

**Status:** NORMATIVE ADDENDUM — MANDATORY COMPANION — ZERO-DEFERRAL — FULL-PROFILE CONFORMANCE GOVERNING  
**Parent:** [Spec 138 — Focusa Prediction, Outcome, Calibration, Metacognitive Learning, Transfer, and Epistemic Governance](138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md)  
**Parent baseline:** commit `7b7b63b4950b12a4d2c3243b72b54400f6eb37e7`, blob `f680115eb0b79aa456f75e8bf20eaa27661ad8e0`  
**Created:** 2026-07-26  
**Owner:** Focusa Core / Prediction / Outcome Resolution / Calibration / Metacognitive Learning / Epistemic Governance  
**Closure relationship:** Spec 138 cannot claim full conformance or closure without Spec 138A.  
**Precedence:** This addendum governs staged activation, capability profiles, normative optionality, APIs, clients, migration, projections, proof, applicability, implementation truth, and closure wherever the parent wording permits a weaker reading. Every parent primitive, contract, law, taxonomy, security boundary, and epistemic safeguard remains included unless an explicit operator-approved specification amendment removes it.

---

## 0. Constitutional directive

```text
MAXIMAL PRIMITIVE COVERAGE REQUIRES MAXIMAL DELIVERY ACCOUNTABILITY.

Staged activation is execution order, not permission to omit.
A capability profile is not a backlog bucket.
Runtime selection is not implementation discretion.
A project may decline to use a capability; Focusa may not claim the capability while failing to implement it.
A partial prediction recorder is not the Spec 138 epistemic substrate.
A score without resolution, calibration, provenance, and authority is not epistemic completion.
A reflection without evaluation, applicability, transfer, conflict, expiry, rollback, and promotion governance is not learning completion.
A schema, vocabulary list, enum, route sketch, suggested CLI, compact card, mock, heuristic, or successful demonstration is not implementation proof.
Every accepted parent primitive and requirement remains in the root closure graph until verified or removed by an explicit operator-approved specification amendment.
```

No implementation order, capability profile, model, agent, product tier, release train, resource limit, domain application, or client may weaken this directive.

---

## 1. Purpose and preserved scope

Spec 138 defines a maximal domain-general substrate for prediction, outcome resolution, scoring, calibration, metacognitive learning, transfer, consolidation, and self-model governance. Its technical content remains authoritative and included.

This addendum closes these delivery loopholes:

1. `MUST use ... staged activation` being interpreted as permission for permanent partial implementation;
2. primitive families that `MAY activate ... in stages` disappearing from the closure target;
3. major generic capabilities expressed as `SHOULD` without a conformance consequence;
4. canonical operation families described as suggestions;
5. Focus Slice support described as `eventually`;
6. prediction and metacognition migration described as `SHOULD`;
7. Profiles A–H being treated as optional product tiers rather than full-substrate implementation slices;
8. acceptance gates lacking stable requirement IDs, complete proof, and a final closure law;
9. capability or domain non-declaration being used to avoid implementation;
10. schemas and API-local records being presented as complete epistemic authority;
11. partial implementation being described as Spec 138 completion.

The addendum does not make raw market feeds, broker integration, market-specific models, or automatic financial authority part of Focusa core. Parent non-goals remain intact. It makes the accepted **generic** substrate non-omissible.

---

## 2. Critical distinction: selective runtime use versus complete implementation

### 2.1 Runtime use may be selective

A project, operator, domain pack, or product edition may choose not to activate a particular capability for a particular scope, for example:

- causal analysis;
- scenario forecasting;
- external source fusion;
- transfer evaluation;
- memory consolidation;
- high-consequence independent review.

That decision must be explicit, scoped, evidence-backed, reviewable, and truthful.

### 2.2 Full Spec 138 implementation is not selective

To claim `full_spec138_conformance`, Focusa MUST implement, integrate, test, migrate, document, expose, and prove every generic capability profile A–H defined by the parent:

```text
Profile A — Core recording
Profile B — Proper scoring and calibration
Profile C — Source and indicator fusion
Profile D — Scenario and causal analysis
Profile E — Metacognitive learning
Profile F — Transfer and self-model
Profile G — Consolidation and long-horizon governance
Profile H — High-consequence governance
```

Profiles are composable runtime capability bundles and mandatory full-conformance implementation slices. They are not optional backlog categories.

A narrower product or release may claim only the exact verified profile subset, for example:

```text
spec138_profile_a_verified
spec138_profiles_a_b_verified
spec138_profiles_a_through_e_verified
```

It MUST NOT call that subset `Spec 138 complete`, `full prediction and learning authority`, `maximal epistemic substrate`, or equivalent language.

### 2.3 Record sparsity is not capability omission

The parent correctly states that not every hot-path record needs every maximal field. Bounded records and projections remain required.

This means:

- a record contains fields activated by its exact operation and profile;
- inactive fields may be absent from that record;
- the underlying generic type, operation, authority, migration, proof, and recovery capability must still exist for full conformance;
- bounded projection is permitted;
- absent implementation is not bounded projection.

---

## 3. Normative language

### 3.1 Normative classes

`MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, and `MAY` are normative.

1. `MUST` and `SHALL` are mandatory whenever recorded applicability is active.
2. `MUST NOT` and `SHALL NOT` are prohibitions and cannot be waived for convenience, cost, model weakness, provider limitations, release pressure, or incomplete data.
3. `SHOULD` is mandatory unless a versioned, evidence-backed, operator-approved variance records exact clause, reason, risk, scope, expiry, replacement behavior, tests, Evidence, Receipt, closure consequence, and rollback.
4. A `SHOULD` variance cannot satisfy a conformance class whose target state includes the original behavior.
5. `MAY` grants permission; it does not permit omission of an implementation activated by an operation, capability profile, domain pack, Vertical, platform, connector, product claim, or conformance class.
6. Parent primitive lists, schema fields, lifecycle states, scorer lists, API operation lists, profile contents, implementation-order items, gates, and final invariants are normative unless explicitly labeled `Illustrative`.
7. The lowercase words `should`, `may`, `suggested`, and `eventually` cannot silently downgrade a capability that the parent scope, executive requirement, profile, gate, or final invariant accepts.

### 3.2 Applicability cannot remain implicit

Every conditional requirement MUST have:

```yaml
schema: focusa.epistemic_applicability_decision.v1
applicability_decision_id:
requirement_ref:
source_clause_ref:
applicability_condition_ref:
applicability_decision_authority:
project_scope_refs: []
domain_refs: []
vertical_refs: []
operation_refs: []
capability_profile_refs: []
conformance_class_refs: []
activation_evidence_refs: []
non_activation_evidence_refs: []
applicability_status: active | conditional_inactive_verified | not_applicable_verified | disputed
review_trigger_refs: []
review_after:
supersedes:
receipt_ref:
```

Rules:

- non-activation and non-applicability require affirmative Evidence;
- absent implementation, missing data, unavailable tools, missing credentials, model incapability, lack of samples, an empty UI, or lack of budget is not non-applicability;
- inadequate evidence may require `unknown`, `abstain`, `blocked`, or `experimental_only`, but it does not erase the capability requirement;
- scope, operation, profile, domain, Vertical, source, scorer, model, or product-claim changes trigger reassessment;
- `disputed` remains open and closure-blocking;
- a system cannot avoid a requirement by declining to register the capability profile that accepted scope implies.

---

## 4. Surgical overrides to parent wording

### 4.1 Parent staged-activation language

The parent sentence requiring composable records, bounded projections, capability profiles, and staged activation is interpreted as:

> Implementations MUST use composable records, bounded projections, and capability profiles. Staged activation controls dependency order and per-scope runtime use only. Every accepted generic primitive and every Profile A–H requirement MUST exist in the root delivery DAG from the beginning and remains required for full Spec 138 conformance.

The parent sentence that implementations `MAY activate` primitive families in stages is replaced for conformance purposes by:

> Implementations MAY enable primitive families for individual scopes in stages, but MUST implement and prove every parent primitive family for full Spec 138 conformance. No family may leave the root closure graph because it is scheduled later or inactive for one project.

### 4.2 Required scoring registry

The parent scoring registry's generic minimum set is REQUIRED for full conformance, including applicable support for:

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

A scorer may be inactive for forecast shapes to which it does not apply. The registry implementation, identity, version, assumptions, direction, valid range, fixtures, error behavior, and operation path remain mandatory.

### 4.3 Calibration requirements

Parent calibration grouping and measurement dimensions are mandatory capabilities for full conformance, not optional report polish.

Calibration MUST support applicable grouping by:

- prediction type and target;
- horizon;
- entity and cohort;
- source and indicator set;
- feature set;
- model, prompt, policy, scorer, and calibration version;
- forecaster;
- confidence/probability bucket;
- regime and scenario;
- trajectory and environment;
- time period;
- original versus transfer context;
- verifier role/capability when Spec 144 verification applies.

Reports MUST expose applicable sample size, evaluated/unresolved/censored/void counts, reliability, bias, sharpness, coverage, discrimination, proper score, baseline, skill, uncertainty, missingness, cohort drift, decision value, tail behavior, high-confidence miss rate, and abstention quality.

Small-sample backoff, cohort identity, parent cohort, backoff depth, effective sample size, uncertainty, and authority posture are mandatory.

### 4.4 Append-only event history

The parent `SHOULD use separate append-only typed events` is strengthened:

> Canonical prediction, outcome, evaluation, metacognition, learning, transfer, consolidation, revocation, rollback, and correction history MUST use separate append-only typed events or an explicitly proven equivalent-or-stronger event model preserving identical semantic distinctions, immutable lineage, correction behavior, replay, and auditability.

Whole-record last-writer state cannot substitute for semantic history.

### 4.5 Canonical operations

The parent wording that the final API shape may evolve remains valid for route spelling and transport design. The **operation capabilities** are mandatory.

Required operation families include:

```text
prediction.question.create
prediction.information_set.commit
prediction.commit
prediction.supersede
prediction.get
prediction.list
outcome.claim
outcome.dispute
outcome.resolve
outcome.correct
prediction.evaluate
calibration.report
metacognition.signal.capture
metacognition.reflect
metacognition.adjustment.propose
metacognition.adjustment.evaluate
learning.candidate.decide
learning.apply
learning.transfer.resolve
learning.retrieve
learning.conflicts
learning.expire
learning.supersede
learning.revoke
learning.rollback
learning.consolidate
self_model.get
```

Exact HTTP routes, CLI commands, Pi tools, and generated-client method names derive from the Operation Registry. Route evolution cannot remove the operation.

### 4.6 CLI, Pi, and client surfaces

The parent `Suggested CLI families` become required capability families when CLI is in the approved Focusa product/conformance scope, which the parent explicitly includes.

API, CLI, Pi, generated contracts/clients, Focus Slice, Mission Canvas/Deck, TUI, menubar, documentation, and migration surfaces MUST either:

- expose the activated operation through shared contracts; or
- record affirmative evidence that the surface is outside the exact approved conformance class.

A client cannot advertise a capability that lacks canonical operation support.

Pi contracts MUST remain compact and bounded, with large artifacts retrieved by reference. Boundedness cannot be used to omit capability truth, authority, uncertainty, conflicts, or recovery.

### 4.7 Focus Slice and UI projections

The parent phrase `Focus Slice SHOULD eventually support` is replaced by:

> Focus Slice and required operator projections MUST support bounded `PREDICTIVE_CONTEXT`, `METACOG_CONTEXT`, and `EPISTEMIC_HEALTH` read models as part of the mandatory surfacing and automation slice. Execution may be sequenced, but the projections remain in the root DAG and block conformance classes that include them.

`Eventually` is not a disposition.

### 4.8 Legacy migration

Parent Prediction v1 and Metacognition v1 migration behavior is mandatory.

Migration MUST:

- preserve readability;
- map reconstructable fields;
- label ambiguous confidence;
- preserve advisory scores without manufacturing scoring authority;
- produce legacy learning/reflection/adjustment/evaluation records where reconstructable;
- mark insufficient evidence and unverified promotion status truthfully;
- preserve lineage and source data;
- support restart, replay, rollback, and migration receipts;
- prevent legacy heuristic promotion from becoming high-authority learning automatically.

Migration omission blocks full conformance.

### 4.9 Transfer, self-model, and longitudinal learning

Parent `SHOULD` language for transfer prediction, transfer evaluation, longitudinal value, and self-model becomes mandatory capability behavior for Profiles F and G.

Before materially new-context application, Focusa MUST record transfer expectation, similarity, differences, benefit, risk, confidence, and evaluation plan.

After application, it MUST record adherence, outcomes, deltas, negative effects, context differences, and the resulting keep/narrow/expand/expire/supersede/revoke decision posture.

The self-model MUST remain scoped, evidence-backed, versioned, uncertainty-aware, and non-global.

### 4.10 Market Lab specialization boundary

Focusa core remains domain-general and does not implement market-specific models merely to satisfy Spec 138.

If Focusa Market Lab or another specialized predictive engine claims Spec 138 integration, it MUST consume the generic Spec 138 authority records and MUST NOT create parallel prediction, outcome, scoring, calibration, learning, transfer, or promotion authority.

The parent `Market Lab should consume` is therefore mandatory when that specialization is activated.

---

## 5. Complete normative source coverage

### 5.1 Combined source

The normative implementation source is:

```text
Spec 138 parent
+ Spec 138A addendum
+ activated inherited requirements from Specs 76, 80, 96, 104, 119, 131, 133, 135F, 137/137A, 144, and other primitive owners
```

### 5.2 Required coverage artifact

Before further production decomposition, implementation promotion, or conformance claims, create and validate:

```text
docs/contracts/spec138a-normative-source-coverage.v1.yaml
```

Minimum shape:

```yaml
schema: focusa.spec138a_normative_source_coverage.v1
parent_spec_path:
parent_spec_blob_sha:
parent_spec_sha256:
addendum_spec_path:
addendum_spec_blob_sha:
addendum_spec_sha256:
combined_normative_source_hash:
extraction_tool_version:
extracted_at:
parent_clause_count:
addendum_clause_count:
inherited_activated_clause_count:
requirement_ids: []
unmapped_clause_refs: []
duplicate_or_weakened_mapping_refs: []
ambiguous_applicability_refs: []
forbidden_deferral_refs: []
coverage_status: complete | incomplete | disputed
reviewed_by:
review_receipt_ref:
```

Coverage MUST include every parent/addendum:

- `MUST`, `MUST NOT`, `SHALL`, and `SHALL NOT`;
- accepted `SHOULD`;
- activated `MAY`;
- primitive family and required vocabulary item;
- required schema field;
- lifecycle state and transition;
- scorer and calibration capability;
- operation and client capability;
- profile requirement;
- implementation-order item;
- acceptance gate;
- final invariant;
- closure blocker;
- activated inherited requirement.

Any normative edit invalidates coverage, decomposition admission, and closure until regeneration and review.

---

## 6. Mandatory machine-readable closure system

Before Spec 138 implementation decomposition can be called complete, all of the following MUST exist and validate:

```text
docs/contracts/spec138a-normative-source-coverage.v1.yaml
docs/contracts/spec138-complete-feature-ledger.v1.yaml
docs/contracts/spec138-delivery-dag.v1.yaml
docs/contracts/spec138-profile-activation-and-conformance-matrix.v1.yaml
docs/contracts/spec138-primitive-ownership-matrix.v1.yaml
docs/contracts/spec138-operation-client-parity-matrix.v1.yaml
docs/contracts/spec138-scorer-and-calibration-matrix.v1.yaml
docs/contracts/spec138-source-independence-and-triangulation-matrix.v1.yaml
docs/contracts/spec138-outcome-resolution-authority-matrix.v1.yaml
docs/contracts/spec138-learning-promotion-and-rollback-matrix.v1.yaml
docs/contracts/spec138-transfer-self-model-and-consolidation-matrix.v1.yaml
docs/contracts/spec138-migration-matrix.v1.yaml
docs/contracts/spec138-security-privacy-retention-matrix.v1.yaml
docs/contracts/spec138-proof-matrix.v1.yaml
docs/contracts/spec138-forbidden-placeholder-audit.v1.yaml
docs/contracts/spec138a-parent-override-map.v1.yaml
```

Empty artifacts, schemas without rows, placeholder status, and generated shells are `contract_only` or `schema_only`, not implementation.

### 6.1 Ledger row

Minimum row:

```yaml
requirement_id:
source_spec:
source_spec_hash:
source_section_anchor:
exact_normative_text:
normative_class: must | must_not | shall | shall_not | should | may
applicability_condition_ref:
applicability_status:
applicability_decision_ref:
profile_refs: []
conformance_class_refs: []
primitive_owner:
implementation_owner:
implementation_order:
dependency_refs: []
blocking_refs: []
core_types: []
events: []
persistence: []
api_operations: []
operation_registry_changes: []
generated_contracts: []
cli_commands: []
pi_tools: []
ui_surfaces: []
migrations: []
positive_tests: []
negative_tests: []
restart_recovery_tests: []
replay_tests: []
security_tests: []
privacy_tests: []
accessibility_tests: []
performance_tests: []
adversarial_tests: []
evidence_refs: []
receipt_refs: []
parent_closure_impact:
status: missing | active | blocked | partial | contract_only | schema_only | shadow_only | implemented_unverified | verified | conditional_inactive_verified | not_applicable_verified | variance_approved_nonconforming | operator_removed_by_spec_amendment | unknown_impact
amendment_ref:
```

Only `verified` satisfies an active mandatory row.

### 6.2 Removal

A requirement leaves the active target state only through an operator-approved specification amendment preserving:

- original ID and normative text;
- source hash;
- reason;
- affected primitives, profiles, operations, clients, domains, and Verticals;
- safety, privacy, retention, authority, calibration, and learning consequences;
- migration and compatibility impact;
- replacement requirements;
- proof consequences;
- operator approval;
- Receipt.

Renaming, merging, superseding, deleting, or closing a task does not remove the requirement.

---

## 7. Mandatory delivery DAG and implementation order

The parent Orders 0–8 are mandatory dependency sequence, not optional feature tiers:

```text
Order 0 — Reconciliation and contracts
Order 1 — Core type extraction
Order 2 — Append-only event storage
Order 3 — Scoring registry and calibration
Order 4 — Metacognitive authority
Order 5 — Transfer and self-model
Order 6 — Fusion and scenarios
Order 7 — Consolidation
Order 8 — Surfacing and automation
```

Every accepted requirement and every Profile A–H component MUST appear in the root DAG before implementation begins.

Each order is a merge gate. It cannot close while a feeder row is missing, blocked, partial, contract-only, schema-only, shadow-only, implemented-unverified, unknown-impact, or absent from integrated proof.

A later order remains mandatory and blocks full Spec 138 closure.

Newly discovered migration, authority, calibration, scorer, source-independence, replay, security, privacy, retention, projection, recovery, or proof work necessary to satisfy an accepted requirement automatically joins the root DAG. It is closure work, not scope creep.

---

## 8. No hidden omission

Accepted work cannot be hidden in:

- prose-only follow-ups;
- TODO or FIXME comments;
- issues or PR comments;
- an unlinked backlog;
- disabled, ignored, quarantined, or non-blocking tests;
- feature flags disabled in acceptance;
- API-local types without core ownership;
- string fields presented as typed semantics;
- placeholder metrics or hard-coded expected deltas;
- schema-only primitive definitions;
- route sketches without handlers;
- suggested CLI commands without implementation;
- static cards without canonical read models;
- mock scorers, resolvers, source profiles, or promotion gates;
- an undeclared profile used to avoid its requirements;
- a domain-local authority that bypasses Focusa core;
- an external repository without verified authority, ownership, dependency, and proof;
- a `known limitation` without requirement ID and closure impact;
- a successful experiment or demo presented as durable authority.

### 8.1 Forbidden language

These phrases and equivalents cannot remove or close accepted work:

```text
later
eventually
future enhancement
post-MVP
nice to have
when time permits
out of scope for now
can be added afterward
phase two someday
optional implementation
suggested only
profile not implemented yet
not needed for MVP
known limitation accepted by default
works in principle
schema complete
API complete
UI complete
mostly done
core recording is enough
```

Permitted planning requires requirement ID, truth status, profile/conformance impact, execution order, dependencies, blockers, owner, implementation, proof, and parent closure consequence.

---

## 9. Profile completeness laws

### Profile A — Core recording

MUST include questions, frozen information sets, immutable commitments, typed outcomes, evaluations, Evidence, Receipts, lifecycle, persistence, replay, migration, and client operations.

### Profile B — Proper scoring and calibration

MUST include scorer registry, scorer authority/version, forecast-shape-specific scoring, calibration cohorts, small-sample backoff, reliability, sharpness, bias, coverage, discrimination, skill, uncertainty, decision value, and correction lineage.

### Profile C — Source and indicator fusion

MUST include source identity/version/access/authority, first-availability, freshness, reliability, independence, shared-dependency detection, contradiction, weights, contribution decomposition, redundancy/correlation penalties, triangulation, sensitivity, missingness, and source revision.

### Profile D — Scenario and causal analysis

MUST include scenario definitions, branches, assumptions, residual probability, stress and sensitivity, counterfactual labels, causal status, mechanisms, confounders, alternatives, disconfirming evidence, interventions, and experiment validity.

### Profile E — Metacognitive learning

MUST include signals, high-confidence misses, reflection claims, alternatives, adjustments, expected effects, metrics, baselines, controls, evaluation, promotion/inhibition, applicability, expiry, review, conflict, and rollback.

### Profile F — Transfer and self-model

MUST include transfer prediction, similarity/difference, expected benefit/risk, outcome, negative transfer, transfer calibration, competence domains, uncertainty, abstention, error modes, over/underconfidence, and versioned self-model revision.

### Profile G — Consolidation and long-horizon governance

MUST include clustering, deduplication, abstraction, specialization, conflict preservation, retention, decay, forgetting decision, archive, reactivation, supersession, revocation, legal hold, migration, and integrity Receipts.

### Profile H — High-consequence governance

MUST include explicit authority, independent review, stronger Evidence, strict source and resolution controls, sensitive-source policy, privacy, retention, audit export, adversarial handling, quarantine, operator/external review, and fail-closed behavior.

A profile name without operational implementation and proof is `schema_only`.

---

## 10. Surface parity

Spec 138's accepted storage, events, API, CLI, Pi, projection, and migration scope requires shared-contract parity.

A backend-only implementation is incomplete when approved scope activates generated contracts, CLI, Pi, Focus Slice, Mission Canvas/Deck, TUI, menubar, docs, migration, or recovery.

A UI-only implementation is incomplete without canonical core types, event history, persistence, authority, operations, Evidence, Receipts, replay, and migration.

A domain engine integration is incomplete when it writes opaque scores, claims, or learning outside Spec 138 authority records.

All clients MUST distinguish at least:

```text
operational
profile_inactive
profile_missing
schema_only
migration_required
resolution_pending
resolution_disputed
calibration_insufficient
source_conflicted
insufficient_evidence
abstained
experimental_only
promotion_pending
transfer_unverified
negative_transfer
rollback_required
operator_required
external_authority_required
degraded
blocked
quarantined
```

---

## 11. Degraded, unknown, and insufficient-evidence posture

Unknown and abstention remain valid epistemic results.

They do not permit implementation omission.

When data, samples, sources, independence, resolution authority, tools, or models are inadequate:

- the operation may return `unknown`, `insufficient_evidence`, `not_resolvable_yet`, `abstain`, `experimental_only`, or `operator_required`;
- the capability's canonical path, contracts, status, recovery, Evidence, and Receipt behavior must still exist;
- no probability, score, calibration, causal claim, or promoted learning may be fabricated;
- no missing provider or tool may become a pass;
- no unavailable capability may be reported as non-applicable by default.

Resource pressure may bound projection and computation but cannot erase authority, provenance, uncertainty, conflict, requirement identity, or recovery.

---

## 12. Tranche, merge, and release contract

Every Spec 138 implementation tranche MUST publish:

1. included requirement IDs;
2. included Profiles A–H components;
3. remaining open requirement IDs and profiles;
4. an empty list of excluded applicable mandatory rows unless each has an operator-approved specification amendment;
5. separate evidence-backed non-applicable, conditional-inactive, optional, and variance rows;
6. core types, events, persistence, operations, generated contracts, clients, migrations, documentation, and recovery changes;
7. positive, negative, restart, replay, security, privacy, retention, authority, calibration, leakage, source-dependence, causal, transfer, rollback, and adversarial proof as activated;
8. Evidence references;
9. tranche Receipt;
10. exact conformance subset claim;
11. parent closure impact;
12. zero-hidden-deferral attestation.

A verified profile subset is not full Spec 138 conformance.

---

## 13. Acceptance gates are mandatory

Parent Gates A–G are mandatory merge and closure gates:

```text
Gate A — Primitive completeness
Gate B — Forecast integrity
Gate C — Calibration
Gate D — Learning authority
Gate E — Transfer
Gate F — Persistence and migration
Gate G — Governance and security
```

Each gate MUST map to stable requirement IDs, tests, Evidence, Receipts, and exact profile/conformance consequences.

A gate cannot pass from:

- fields existing;
- schema generation;
- API route registration;
- one happy-path test;
- one correct forecast;
- one successful promotion;
- a model assertion;
- a static UI;
- a migration plan without execution;
- a mock resolver or scorer;
- an unevaluated reflection.

---

## 14. Truthful conformance statuses

```text
documentation_only
contract_only
schema_only
shadow_only
partial_runtime
implemented_unverified
profile_subset_verified
full_spec138_conformance
```

Only `full_spec138_conformance` may be described as:

- Spec 138 complete;
- maximal epistemic substrate implemented;
- complete prediction and metacognitive authority;
- complete calibration and learning system;
- equivalent language.

A profile subset claim MUST name the exact profiles, operations, clients, platforms, domains, migrations, and limitations proven.

---

## 15. Spec 144 integration

When Spec 144 is activated, Spec 138/138A MUST expose its information-set, source, leakage, forecast, uncertainty, scenario, resolution, scoring, calibration, causal, transfer, and promotion requirements to the Verification Obligation Graph.

Spec 144 verification strengthens proof but does not replace Spec 138A's delivery graph or permit omitted implementation.

A favorable Verifier verdict cannot satisfy an absent Spec 138 requirement.

A required epistemic verifier or deterministic validator that is unavailable blocks the affected high-consequence operation; it does not waive the requirement.

---

## 16. Acceptance criteria

Spec 138A is accepted only when:

1. every parent/addendum normative clause maps without weakening;
2. combined source coverage has zero unmapped, ambiguous, and weakened clauses;
3. every primitive family remains in the root closure graph;
4. Profiles A–H are mandatory components of full conformance;
5. runtime profile inactivity is distinguished from implementation absence;
6. every applicability decision has affirmative Evidence and review triggers;
7. no capability is avoided through profile non-declaration;
8. staged activation is implemented as dependency order only;
9. every later order remains open and parent-blocking until verified;
10. the required scorer registry is operational, versioned, tested, and authority-governed;
11. calibration dimensions, measures, and small-sample backoff are implemented;
12. canonical append-only semantic history or proven equivalent exists;
13. required operation families exist independent of route spelling;
14. API, CLI, Pi, generated contracts, and activated UI projections preserve shared semantics;
15. `PREDICTIVE_CONTEXT`, `METACOG_CONTEXT`, and `EPISTEMIC_HEALTH` are implemented where the parent product scope requires them;
16. legacy prediction and metacognition migration executes and is receipted;
17. transfer prediction/evaluation and negative-transfer retention operate;
18. self-model claims are scoped, versioned, and evidence-backed;
19. consolidation preserves provenance, exceptions, conflicts, retention, and rollback;
20. high-consequence profiles fail closed;
21. parent Gates A–G are stable requirement-backed merge gates;
22. no mandatory clause is waived by runtime variance;
23. no `SHOULD` variance is presented as full conformance;
24. unknown and abstention remain truthful without manufacturing certainty;
25. backend, client, migration, documentation, Evidence, Receipt, and replay parity is proven;
26. hidden-deferral and forbidden-placeholder audits pass;
27. every tranche discloses remaining open profiles and requirements;
28. newly discovered closure work joins the DAG;
29. profile-subset completion is labeled exactly;
30. only full conformance is called Spec 138 complete.

---

## 17. Closure blockers

Spec 138 and Spec 138A MUST NOT close while:

- any parent/addendum normative clause is unmapped, weakened, duplicated ambiguously, or absent from source coverage;
- the source hash changed without regenerated coverage and review;
- any accepted primitive family or Profile A–H component is absent from the root DAG;
- any active mandatory row is missing, blocked, partial, contract-only, schema-only, shadow-only, degraded, disputed, implemented-unverified, or unknown-impact;
- staged activation is used to remove a requirement from the full-conformance target;
- a profile is avoided by failing to declare or expose it;
- non-applicability lacks affirmative Evidence;
- a mandatory clause is waived without a specification amendment;
- a `SHOULD` variance lacks scope, expiry, operator approval, Evidence, Receipt, rollback, or conformance consequence;
- the scorer registry, calibration, outcome authority, source independence, transfer, self-model, consolidation, or high-consequence governance exists only as vocabulary or schema;
- canonical history collapses prediction, outcome, evaluation, and learning into mutable whole-record state;
- required operation capabilities exist only as route sketches or suggestions;
- CLI, Pi, generated contracts, or required projections are silently omitted;
- Focus Slice support remains assigned to `eventually` or an unlinked backlog;
- legacy migration remains prose-only, optional, or untested;
- a domain application creates parallel authority;
- unknown, weak evidence, or missing tools are converted into confidence or a pass;
- reflection output promotes itself;
- success alone becomes reusable learning;
- dependent evidence counts as independent confirmation;
- a partial profile subset is described as maximal or complete;
- any accepted work is hidden in TODOs, disabled tests, mocks, flags, static UI, unpublished branches, or external repositories without tracked proof;
- a full-conformance claim lacks exact-SHA integrated proof and a final zero-unapproved-deferral, zero-omission Receipt.

---

## 18. Final law

```text
Spec 138 defines the complete generic epistemic organism.
Spec 138A prevents its organs from becoming optional implementation fragments.

Projects may activate only what they need.
Records may remain bounded.
Unknown may remain unknown.
Agents may abstain.

But Focusa may not call the maximal substrate complete while scoring, calibration,
source independence, causal discipline, learning authority, transfer, self-model,
consolidation, high-consequence governance, migration, clients, proof, or rollback is missing.

Nothing accepted disappears.
Nothing partial becomes maximal through language.
Nothing closes while anything required is omitted, staged out of the root graph,
hidden, unverified, unsupported, or unknown.
```
