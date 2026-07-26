# Spec 137A — Focusa Temporal Zero-Deferral, Applicability, and Omission Firewall Addendum

**Status:** NORMATIVE ADDENDUM — MANDATORY COMPANION — ZERO-DEFERRAL — IMPLEMENTATION AND CLOSURE GOVERNING  
**Parent:** [Spec 137 — Focusa Temporal Authority, Deadlines, Urgency, and Grounded Forecasting](137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md)  
**Parent baseline:** commit `7b7b63b4950b12a4d2c3243b72b54400f6eb37e7`, blob `53ea528377399cd30a70ea1adf42e16650d6ea76`  
**Created:** 2026-07-26  
**Owner:** Focusa Core / Temporal Authority / Release and Conformance Governance  
**Closure relationship:** Spec 137 cannot claim full conformance or closure without Spec 137A.  
**Precedence:** This addendum governs interpretation of deferral, sequencing, applicability, optionality, variance, degraded posture, surface parity, migration, proof, and closure wherever the parent wording could permit a weaker reading. All stronger temporal, safety, security, privacy, authority, evidence, reconciliation, accessibility, and retention requirements in Spec 137 remain intact.

---

## 0. Constitutional directive

```text
THE CALENDAR DOES NOT PERMIT REQUIREMENTS TO DISAPPEAR.

Sequence is not deferral.
A later execution slice is not a backlog.
Blocked is not complete.
Partial is not complete.
Degraded is not complete.
Unsupported is not not-applicable.
A missing capability is not an inactive capability.
A user-selectable feature is not optional implementation work after it is accepted.
A schema, enum, prompt, route declaration, mock, static card, successful process, or passing subset is not implementation proof.
Every accepted temporal requirement remains in the root closure graph until verified or removed by an explicit operator-approved specification amendment.
```

No model, agent, implementation tranche, issue tracker, capability profile, platform limitation, deadline, resource condition, release train, or client may weaken this directive.

---

## 1. Purpose

Spec 137 already contains a substantial completeness and omission firewall. This addendum closes its remaining interpretive escape hatches:

1. the phrase that an applicable requirement may move to a later tranche;
2. `SHOULD` variance ambiguity;
3. `MAY` and `optional_unimplemented` applicability ambiguity;
4. capability-profile avoidance, where an implementer could avoid declaring a profile to avoid its requirements;
5. broad phrases such as `where applicable`, `where supported`, `when available`, or equivalent wording;
6. incomplete platform, domain, client, migration, and proof scope;
7. a partial tranche or degraded path being presented as Spec 137 completion;
8. parent ledger coverage being treated as sufficient after a normative edit without addendum-aware source-hash regeneration.

This addendum does not create new temporal authority. It hardens implementation and conformance truth for the temporal primitives already owned by Spec 137.

---

## 2. Normative interpretation

### 2.1 Normative classes

The words `MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, and `MAY` are normative.

1. `MUST` and `SHALL` are mandatory whenever the requirement's recorded applicability condition is true.
2. `MUST NOT` and `SHALL NOT` are prohibitions. They cannot be waived by convenience, cost, deadline, unsupported tooling, platform weakness, or release pressure.
3. `SHOULD` is mandatory unless a versioned, evidence-backed, operator-approved variance records the exact clause, reason, risk, scope, expiry, replacement behavior, tests, Evidence, Receipt, closure consequence, and rollback.
4. A `SHOULD` variance does **not** satisfy any conformance class whose target state includes the original behavior.
5. `MAY` grants permission. It does not grant permission to omit an implementation that an operation, capability profile, product claim, platform claim, domain pack, Vertical, connector, conformance class, or approved scope activates.
6. An illustrative example is non-normative only when explicitly labeled `Illustrative`.
7. Unlabeled schema fields, state transitions, table rows, required lists, failure behavior, acceptance criteria, closure blockers, implementation slices, and machine-readable artifact requirements are normative.

### 2.2 Applicability cannot remain implicit

Phrases such as the following cannot act as unrecorded implementation discretion:

```text
where applicable
where required
where supported
when available
when practical
where possible
when appropriate
as needed
if relevant
for applicable platforms
for applicable domains
```

Every conditional requirement MUST have a durable applicability record:

```yaml
schema: focusa.temporal_applicability_decision.v1
applicability_decision_id:
requirement_ref:
source_clause_ref:
applicability_condition_ref:
applicability_decision_authority:
applicable_scope_refs: []
platform_refs: []
domain_refs: []
operation_refs: []
capability_profile_refs: []
activation_evidence_refs: []
non_activation_evidence_refs: []
applicability_status: active | conditional_inactive_verified | not_applicable_verified | disputed
review_trigger_refs: []
review_after:
supersedes:
receipt_ref:
```

Rules:

- `conditional_inactive_verified` and `not_applicable_verified` require affirmative Evidence.
- Missing implementation, missing credentials, an unavailable provider, absent tests, an empty UI, an undeclared platform, lack of time, lack of budget, or unsupported hardware is not Evidence of non-applicability.
- `disputed` remains open and blocks affected conformance.
- A scope, operation, profile, platform, domain, Vertical, connector, or product-claim change MUST trigger applicability reassessment.
- An implementer cannot avoid a requirement by declining to register the profile, capability, surface, or adapter whose need is implied by accepted scope.

### 2.3 User choice is not implementation choice

A user may choose not to:

- set a deadline;
- enable a high-consequence domain profile;
- expose a particular client projection;
- connect an external calendar or provider;
- request high-precision timing;
- activate a particular notification channel.

Once an approved product scope promises the capability, or an operation/profile activates it, Focusa MUST implement the capability's canonical state, API/operation path, permission behavior, degraded posture, migration, client projection, tests, Evidence, Receipts, and recovery rules.

---

## 3. Surgical overrides to Spec 137

### 3.1 Parent core law 19 is replaced

The parent wording titled **Later is a governed decision** is replaced for conformance purposes by:

> **Execution order is governed; accepted scope is not deferred.** A requirement may be assigned to a later dependency tranche only when it already exists in the root delivery DAG with stable identity, ownership, dependencies, implementation destination, proof obligations, Evidence, Receipt, and parent closure impact. Changing its execution order does not remove it from the active delivery contract. It remains open and blocks every parent conformance class that requires it. Moving a requirement outside the accepted Spec 137 target state requires an operator-approved specification amendment, not a scheduling decision.

### 3.2 Parent core law 20 is strengthened

The parent normative-class law is supplemented by:

- no mandatory clause may be waived by a runtime variance;
- an accepted `SHOULD` remains required for full conformance unless the governing conformance class explicitly excludes it through an operator-approved specification amendment;
- an unactivated `MAY` row must still retain explicit applicability, review triggers, and capability truth;
- `optional_unimplemented` can never describe a capability advertised, exposed, activated, depended upon, or required by accepted product scope.

### 3.3 Parent `optional_unimplemented` status is constrained

`optional_unimplemented` is permitted only when all of the following are true:

1. the parent clause is genuinely permissive rather than a required capability;
2. no approved operation, profile, platform, domain, product claim, connector, or conformance class activates it;
3. affirmative non-activation Evidence exists;
4. review triggers are recorded;
5. no client advertises or exposes it as operational;
6. no dependency assumes it;
7. its absence does not weaken safety, authority, Evidence, reconciliation, compatibility, accessibility, or temporal truth.

Otherwise the row is `missing`, `blocked`, `partial`, or `implemented_unverified` and remains closure-blocking.

### 3.4 Parent `SHOULD` variance is constrained

A `SHOULD` variance:

- remains visible in the root ledger;
- is scoped and expiring;
- identifies the conformance classes it prevents;
- cannot be inherited silently by a different platform, domain, or release;
- cannot become permanent through repeated renewal without a specification amendment;
- cannot weaken high-consequence clock, calendar, deadline, freshness, uncertainty, reconciliation, security, privacy, or Evidence behavior;
- cannot be described as equivalent implementation unless equivalent-or-stronger proof exists.

### 3.5 Broad platform and domain qualifiers are fail-closed

When Spec 137 uses platform, domain, capability, clock-source, provider, or high-consequence qualifiers:

- unknown applicability blocks the affected operation or conformance claim;
- unavailable implementation is reported as unsupported or blocked, never not-applicable by default;
- a high-consequence domain cannot downgrade a requirement because its required source, verifier, calibration path, or tool is unavailable;
- a platform claim activates every requirement necessary to make that claim truthful;
- a domain pack must declare its temporal profile or remain blocked from high-consequence activation.

---

## 4. Complete source coverage

### 4.1 Combined normative source

For implementation and conformance, the normative source is:

```text
Spec 137 parent
+ Spec 137A addendum
+ activated inherited requirements from primitive-owning specifications
```

The current parent-only source hash is insufficient after adoption of this addendum.

### 4.2 Required source-coverage artifact

Before further Spec 137 decomposition, implementation promotion, or full-conformance claims, create and validate:

```text
docs/contracts/spec137a-normative-source-coverage.v1.yaml
```

Minimum shape:

```yaml
schema: focusa.spec137a_normative_source_coverage.v1
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

Every parent and addendum `MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, accepted `SHOULD`, activated `MAY`, required schema field, table obligation, state transition, acceptance criterion, closure blocker, implementation-slice item, and inherited activated clause MUST map without semantic weakening.

Any normative edit invalidates source coverage, decomposition admission, and closure until regeneration and review complete.

---

## 5. Ledger and delivery-DAG requirements

### 5.1 Existing ledger must be extended, not replaced by prose

`docs/contracts/spec137-complete-feature-ledger.v1.yaml` MUST include:

- every Spec 137A requirement;
- exact parent/addendum source references;
- combined source hash;
- applicability records;
- conformance-class impact;
- parent and tranche closure impact;
- all surfaces, migrations, tests, Evidence, and Receipts.

Addendum rows use stable IDs such as:

```text
S137A-REQ-001 ... S137A-REQ-NNN
```

IDs are append-only and cannot be reused for different semantics.

### 5.2 Required truth statuses

The ledger MUST distinguish:

```text
missing
active
blocked
partial
contract_only
schema_only
shadow_only
implemented_unverified
verified
conditional_inactive_verified
not_applicable_verified
variance_approved_nonconforming
operator_removed_by_spec_amendment
unknown_impact
```

Only `verified` satisfies an active mandatory requirement.

### 5.3 Required root-DAG presence

Every accepted requirement MUST appear in the root delivery DAG before implementation decomposition or merge with:

```yaml
requirement_ref:
primitive_owner:
implementation_owner:
dependency_refs: []
blocking_refs: []
implementation_tasks: []
affected_repositories: []
affected_files_or_packages: []
core_types: []
reducer_events: []
persistence: []
api_operations: []
operation_registry_changes: []
generated_contracts: []
cli_commands: []
pi_tools: []
ui_surfaces: []
platforms: []
domains: []
migrations: []
positive_tests: []
negative_tests: []
clock_tests: []
restart_recovery_tests: []
fault_injection_tests: []
security_tests: []
privacy_tests: []
accessibility_tests: []
performance_tests: []
evidence_requirements: []
receipt_requirements: []
parent_closure_impact:
```

A later slice is execution order only. It is not a post-release backlog and cannot disappear from the root graph.

---

## 6. No hidden deferral or omission

Mandatory work cannot be hidden in:

- prose-only follow-ups;
- TODO or FIXME comments;
- issue or PR comments;
- an unlinked backlog;
- disabled, ignored, quarantined, or non-blocking tests;
- feature flags disabled during acceptance;
- mocks, fixtures, placeholders, static cards, or hard-coded success responses;
- a schema without runtime consumption;
- a route without implementation;
- a capability enum without a provider;
- a client-local implementation absent from shared contracts;
- an external repository without verified project authority, owner, dependency, and proof;
- unsupported platform notes without an open blocking requirement;
- a `known issue` section without requirement identity and closure impact;
- a future timeline UI label without canonical state and operation support.

### 6.1 Forbidden disposition language

The following phrases and equivalents cannot remove or close accepted work:

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
follow-up after launch
not needed for MVP
known limitation accepted by default
works in principle
close enough
mostly complete
backend complete
UI complete
schema complete
docs complete
```

Permitted planning requires requirement ID, truth status, execution order, dependency, blocker, owner, implementation, proof, and closure impact.

### 6.2 Newly discovered closure work

A newly discovered defect, migration, client-parity gap, clock-capability issue, calendar conflict, privacy control, accessibility need, fault-injection case, recovery behavior, proof obligation, or cross-spec integration necessary to satisfy accepted Spec 137 behavior MUST join the ledger and DAG. It is closure work, not scope creep.

Unrelated new product behavior requires a separate specification or operator-approved scope amendment.

---

## 7. Surface and platform completeness

### 7.1 No backend-only completion

A temporal requirement is incomplete when its approved scope activates any of the following and that surface is absent or inconsistent:

- reducer and canonical state;
- daemon enforcement;
- SQLite/CRDT persistence where declared;
- Operation Registry;
- API;
- generated OpenAPI/JSON Schema/Rust/TypeScript contracts;
- CLI;
- Pi tools and context delivery;
- Awareness/Preload/Context Cognition;
- Workpoint and Work Loop;
- Silent Sessions;
- Trajectory and compaction packets;
- closure and Receipts;
- Mission Canvas/Deck, Work Rail, TUI, menubar, or notification surfaces;
- documentation, doctor, migration, and recovery guidance.

### 7.2 No UI-only completion

A visual deadline, urgency, estimate, progress, or breach projection is incomplete without canonical authority, reducer events, persistence, permissions, freshness, Evidence, migration, replay, and recovery behavior.

### 7.3 Degraded and unsupported modes

Low-memory, offline, clock-unavailable, provider-unavailable, credential-missing, transport-degraded, or unsupported-platform modes may alter posture only through explicit policy. They cannot:

- report missing functionality as complete;
- waive estimate grounding;
- reinterpret deadlines;
- erase uncertainty;
- bypass reconciliation;
- hide unsupported capabilities;
- fabricate non-applicability;
- convert absent proof into a pass;
- remove requirement IDs or recovery guidance.

---

## 8. Tranche and merge contract

Every implementation tranche MUST publish:

1. included requirement IDs;
2. all remaining open requirement IDs;
3. an empty list of excluded applicable mandatory requirements unless each has an operator-approved specification amendment;
4. separate evidence-backed optional, non-applicable, conditional-inactive, and variance rows;
5. code, schemas, reducer, persistence, API, generated-contract, client, migration, and documentation changes;
6. positive, negative, stale, scope, clock, restart, replay, fault-injection, security, privacy, accessibility, and adversarial proof as activated;
7. Evidence references;
8. tranche Receipt;
9. parent closure impact;
10. zero-hidden-deferral attestation.

A tranche may close as a verified slice only. It cannot be described as Spec 137 complete unless the final closure law is satisfied.

---

## 9. Settlement, release, and conformance truth

Truthful implementation statuses are:

```text
documentation_only
contract_only
schema_only
shadow_only
partial_runtime
implemented_unverified
verified_slice
full_spec137_conformance
```

Only `full_spec137_conformance` may be described as Spec 137 complete.

A partial or degraded settlement MUST identify every unsatisfied requirement ID and cannot close any WorkItem, parent specification, release, or conformance class whose target state includes those requirements.

No override, deadline pressure, Receipt state, operator disposition, or successful functional outcome may manufacture verified temporal conformance.

---

## 10. Required machine-readable amendments

Before the next full Spec 137 implementation or conformance claim, the following MUST exist and validate in addition to the parent artifacts:

```text
docs/contracts/spec137a-normative-source-coverage.v1.yaml
docs/contracts/spec137a-applicability-matrix.v1.yaml
docs/contracts/spec137a-conformance-class-matrix.v1.yaml
docs/contracts/spec137a-forbidden-placeholder-audit.v1.yaml
docs/contracts/spec137a-parent-override-map.v1.yaml
```

The parent ledger, delivery DAG, parity, proof, migration, and conformance artifacts MUST reference these addendum artifacts and the combined normative source hash.

Empty files, schemas without rows, placeholder statuses, or generated shells are `contract_only` or `schema_only`, not completion.

---

## 11. Acceptance criteria

Spec 137A is accepted only when:

1. every parent and addendum normative clause maps without weakening;
2. the combined source hash is current;
3. no ambiguous applicability remains;
4. every `not_applicable_verified` and `conditional_inactive_verified` row has affirmative Evidence and review triggers;
5. no capability is avoided by failing to declare its required profile or adapter;
6. parent law 19 is implemented as sequencing without scope removal;
7. no mandatory clause is waived by runtime variance;
8. every `SHOULD` variance identifies prevented conformance classes;
9. every accepted requirement exists in the root DAG from the beginning;
10. scheduled-later requirements remain open and parent-blocking;
11. blocked, partial, degraded, schema-only, shadow-only, and implemented-unverified rows cannot satisfy closure;
12. optional user choice is distinguished from implementation completeness;
13. unsupported platforms and capabilities remain truthful and open when required;
14. backend, client, generated-contract, migration, documentation, Evidence, and Receipt parity is proven;
15. degraded modes preserve temporal truth and requirement visibility;
16. hidden-deferral and forbidden-placeholder audits pass;
17. every tranche discloses remaining open requirements;
18. newly discovered closure work joins the ledger and DAG;
19. no partial settlement closes a target state requiring omitted rows;
20. only full conformance is called Spec 137 complete.

---

## 12. Closure blockers

Spec 137 and Spec 137A MUST NOT close while:

- either normative source contains an unmapped or weakened clause;
- the source hash changed without regenerated coverage and review;
- any active mandatory row is missing, blocked, partial, contract-only, schema-only, shadow-only, degraded, disputed, implemented-unverified, or unknown-impact;
- any later-tranche requirement is absent from the root DAG;
- any applicability decision is implicit, unsupported, stale, or disputed;
- missing credentials, tooling, providers, platforms, or implementations are used as non-applicability Evidence;
- a mandatory clause is waived without a specification amendment;
- a `SHOULD` variance is unscoped, permanent, unapproved, unreceipted, or presented as full conformance;
- an accepted `MAY` capability is advertised while unimplemented;
- required client, platform, migration, recovery, security, privacy, accessibility, fault-injection, replay, Evidence, or Receipt work is absent;
- any required behavior exists only in prose, schemas, enums, mocks, static UI, disabled tests, disabled flags, unpublished branches, or client-local code;
- a functional success hides temporal failure;
- a degraded result is described as complete;
- a partial tranche or verified slice is described as full Spec 137 completion;
- any accepted work is hidden outside the machine-readable closure system;
- the final zero-unapproved-deferral and zero-omission Receipt is absent.

---

## 13. Final law

```text
Spec 137 owns temporal truth.
Spec 137A owns the rule that accepted temporal truth work cannot disappear.

Execution may be sequenced.
Applicability may be proven.
User activation may be selective.
Implementation truth may not be weakened.

Nothing closes while anything required is omitted, deferred out of the root graph,
hidden, partial, unsupported, unverified, or unknown.
```
