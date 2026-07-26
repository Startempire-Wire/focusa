# Spec 144 — Focusa Semantic Integrity, Zero-Omission Build↔Verify Routing, and Vertical Intelligence

**Status:** NORMATIVE DRAFT — ZERO-DEFERRAL — IMPLEMENTATION-GATED — NO IMPLEMENTATION OR CONFORMANCE CLAIM IMPLIED  
**Owner:** Focusa Core / Ontology / Secondary Cognition / Verification / Vertical Intelligence  
**Created:** 2026-07-26  
**Revised:** 2026-07-26 — complete zero-deferral and omission-firewall replacement  
**Implementation activation:** Spec 143’s locked release implementation and required proof MUST close, and the operator MUST explicitly activate Spec 144 implementation. This is dependency sequencing only. It does not make any accepted Spec 144 requirement optional, removable, post-MVP, or eligible for silent deferral.  
**Primary relationship:** Extends and composes Specs 45–50, 61, 66, 70, 72, 74–79, 88, 90, 95, 97, 100, 107, 109, 113, 116, 119, 120, 125, 130, 131, 133, 135F, 136, 137, 138, 140, 141, 142, and 143.  
**Precedence:** Primitive-owning specifications retain ownership of their primitives. Spec 144 owns the cross-cutting formal-semantic, Build↔Verify, verification-routing, Vertical Intelligence Bundle, and zero-omission conformance contract defined here. Stronger safety, privacy, authority, evidence, compatibility, and no-deferral requirements always survive composition.  
**Does not create:** a second reducer, ontology registry, event store, Workpoint authority, deadline authority, prediction authority, learning-promotion authority, permission system, Receipt ledger, workflow engine, browser engine, or agent framework.

---

## 0. Constitutional directive

```text
NOTHING ACCEPTED MAY DISAPPEAR.

Sequence is not deferral.
Blocked is not complete.
Partial is not complete.
Degraded is not complete.
Schema-only is not complete.
Shadow mode is not complete.
A mock, static card, enum, prompt, verdict string, successful process, or passing subset is not complete.
A user-selectable capability is not optional implementation work after that capability is accepted.
An activated Vertical cannot omit the semantic, temporal, epistemic, verification, reflex, migration, client, Evidence, or Receipt obligations it activates.
Unknown impact blocks implementation promotion and closure.
Every normative clause remains in the closure graph until verified or removed by an explicit operator-approved specification amendment.
```

No agent, router, model, implementation phase, product surface, issue tracker, capability profile, resource constraint, deadline, or release train may weaken this directive.

---

## 1. One-line definition

Focusa MUST compile its core-owned semantic registry into parity-checked RDF, OWL, and SHACL contracts and MUST govern consequential work through isolated Builder cognition and ontology-routed domain-specific verification portfolios using immutable snapshots, typed findings, calibrated verifier capabilities, Vertical-specific temporal and epistemic applicability, bounded reflex overlays, complete machine-readable requirement coverage, and reducer-controlled settlement so that neither fluent generation, agreeable review, partial implementation, missing applicability analysis, nor attractive presentation can mint operational truth or erase accepted work.

---

## 2. Normative language, applicability, and interpretation

### 2.1 Normative classes

The key words `MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, and `MAY` are normative.

1. `MUST` and `SHALL` are mandatory whenever the requirement’s recorded applicability condition is true.
2. `MUST NOT` and `SHALL NOT` are prohibitions and cannot be waived by implementation convenience.
3. `SHOULD` is mandatory unless a versioned, evidence-backed, operator-approved variance records the exact clause, reason, risk, scope, expiry, replacement behavior, tests, closure consequence, and rollback. A variance does not satisfy conformance levels that require the original behavior.
4. `MAY` grants permission. It does not grant permission to omit a capability that an approved operation, Vertical, profile, product claim, contract, or conformance scope activates.
5. Illustrative examples are non-normative only when explicitly labeled `Illustrative`. An unlabeled schema field, table row, state transition, list item under a normative heading, acceptance criterion, closure blocker, or required operation is normative.

### 2.2 Applicability cannot remain implicit

Phrases such as `where applicable`, `where required`, `as needed`, `when practical`, `where possible`, `when appropriate`, and equivalent wording MUST NOT be used as unrecorded implementation discretion.

Every conditional requirement MUST have:

```yaml
applicability_condition_ref:
applicability_decision_authority:
applicable_scope_refs: []
activation_evidence_refs: []
non_activation_evidence_refs: []
applicability_status: active | conditional_inactive_verified | not_applicable_verified | disputed
review_trigger_refs: []
```

`conditional_inactive_verified` and `not_applicable_verified` require affirmative evidence. Absence of implementation, missing credentials, unsupported tooling, lack of time, lack of budget, or an empty product surface is not non-applicability.

### 2.3 Optional user choice is not optional implementation

A user may choose not to activate a Vertical, connector, verifier, profile, export, or assurance tier. Once the approved scope promises that capability or an operation/profile activates it, the implementation, degradation, migration, testing, Evidence, Receipt, and recovery obligations remain mandatory.

### 2.4 Conflict interpretation

When two clauses appear to conflict:

1. operator and safety authority remain highest;
2. the primitive-owning specification controls primitive semantics;
3. the stronger authority, safety, privacy, evidence, compatibility, or no-deferral requirement controls;
4. the conflict MUST be recorded in the cross-spec amendment matrix;
5. unresolved conflict blocks the affected implementation and closure;
6. no agent may silently choose the easier interpretation.

---

## 3. Zero-deferral and omission firewall

### 3.1 Accepted requirements never leave the closure graph silently

Every normative clause in this document and every inherited requirement activated by Spec 144 MUST receive a stable requirement ID and remain in the complete feature ledger until one of these terminal dispositions occurs:

```text
verified
not_applicable_verified
conditional_inactive_verified
operator_removed_by_spec_amendment
```

Only `verified` satisfies an active mandatory requirement.

A requirement marked `blocked`, `missing`, `partial`, `implemented_unverified`, `schema_only`, `shadow_only`, `degraded`, `disputed`, or `unknown_impact` remains open and blocks every parent closure and conformance claim that depends on it.

### 3.2 Removal requires a specification amendment

A requirement may be removed only through a versioned operator-approved amendment containing:

```yaml
amendment_id:
original_requirement_id:
original_normative_text:
source_spec_hash:
reason:
affected_objects_actions_and_surfaces: []
affected_acceptance_criteria: []
affected_dependencies: []
affected_verticals: []
safety_privacy_authority_impact:
compatibility_and_migration_impact:
replacement_requirement_refs: []
proof_consequences: []
operator_approval_ref:
receipt_ref:
effective_at:
```

The original row remains in the append-only ledger with status `operator_removed_by_spec_amendment`. Renaming, moving, merging, superseding, or deleting a task does not remove the requirement.

### 3.3 Sequencing is not deferral

Implementation phases and dependency ordering determine execution order only. Every accepted requirement MUST be present from the beginning in the delivery DAG with:

- stable requirement ID;
- primitive owner;
- implementation owner;
- dependency edges;
- implementation tasks;
- affected repositories and files;
- applicable clients and surfaces;
- migration obligations;
- positive, negative, adversarial, restart, replay, security, privacy, accessibility, and performance proof obligations;
- Evidence requirements;
- Receipt requirements;
- closure impact.

A later phase is not a backlog. A requirement assigned to a later phase remains mandatory and blocks Spec 144 closure.

### 3.4 Blocked work remains visible and blocking

A blocked requirement MUST retain:

```yaml
blocker_id:
blocked_requirement_refs: []
blocker_class:
owner:
detected_at:
evidence_refs: []
recovery_path:
next_review_at:
parent_closure_impact:
```

A blocker is not a disposition, variance, completion state, or permission to omit downstream work.

### 3.5 Newly discovered closure work is admitted automatically

A newly discovered defect, missing acceptance requirement, migration, parity gap, security control, performance bound, proof obligation, semantic conflict, Vertical composition requirement, or recovery behavior necessary to satisfy an accepted requirement MUST be added to the ledger and DAG. It MUST NOT be dismissed as scope creep.

Unrelated new product ideas require a separate specification or approved scope amendment and do not silently join Spec 144.

### 3.6 Forbidden planning and closure language

The following phrases and equivalents MUST NOT be used to remove, hide, or close accepted work:

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
known limitation accepted by default
TBD without owner and closure impact
TODO without requirement ID
not needed for MVP
close enough
works in principle
docs-only complete
schema complete
backend complete
UI complete
mostly done
```

Permitted planning form:

```text
requirement ID
current truth status
execution order
dependency and blocker
owner
required implementation
required proof
parent closure impact
```

### 3.7 No hidden deferral surfaces

Mandatory work MUST NOT be hidden in:

- prose-only follow-ups;
- TODO/FIXME comments;
- issue comments;
- an unlinked backlog;
- disabled, ignored, quarantined, or non-blocking tests;
- feature flags disabled in acceptance;
- mocks, fixtures, static cards, placeholders, or compatibility shims presented as final behavior;
- client-specific code not represented in the shared contracts;
- unpublished branches;
- external repositories without a verified cross-repository dependency and owner;
- a `known issue` section without an open blocking requirement;
- a capability enum without an operational adapter;
- an ontology declaration without runtime consumers and proof;
- a Verifier profile without an executable provider;
- a Vertical profile without its required semantic packs.

### 3.8 No surface omission

A backend-only implementation is incomplete whenever the approved requirement activates API, CLI, Pi, MCP/OpenAI/REST projection, generated client, Mission Canvas, Work Rail, TUI, menubar, UIAI Engine integration, documentation, migration, or operator recovery behavior.

A UI-only implementation is incomplete without canonical daemon state, reducer events, persistence, authority, generated contracts, Evidence, and Receipts.

A declaration-only ontology, reflex, verifier, Vertical, or policy is incomplete without the registered runtime path, tests, proof, and recovery behavior required by its contract.

### 3.9 No silent capability shrinkage under degraded modes

Low-memory, offline, verifier-unavailable, missing-credential, unsupported-platform, transport-degraded, or resource-pressure modes may change execution posture only according to explicit policy. They MUST NOT:

- report missing functionality as complete;
- waive mandatory verification;
- erase obligations;
- fabricate non-applicability;
- reduce authority or evidence requirements;
- silently switch to a weaker model or verifier;
- convert unavailable proof into a pass.

### 3.10 No complete claim from partial conformance

Truthful implementation statuses are:

```text
documentation_only
contract_only
schema_only
shadow_only
partial_runtime
implemented_unverified
verified_slice
full_spec_conformance
```

Only `full_spec_conformance` may be described as `Spec 144 complete`, and only after every active mandatory ledger row is verified and every closure blocker is cleared.

---

## 4. Mandatory machine-readable closure system

### 4.1 Required artifacts before decomposition or implementation

Before any Spec 144 production implementation task is decomposed, claimed active, or merged, all of these files MUST exist and validate:

```text
docs/contracts/spec144-normative-source-coverage.v1.yaml
docs/contracts/spec144-complete-feature-ledger.v1.yaml
docs/contracts/spec144-delivery-dag.v1.yaml
docs/contracts/spec144-primitive-ownership-matrix.v1.yaml
docs/contracts/spec144-obligation-verifier-matrix.v1.yaml
docs/contracts/spec144-cross-spec-amendment-matrix.v1.yaml
docs/contracts/spec144-client-parity-matrix.v1.yaml
docs/contracts/spec144-vertical-pack-matrix.v1.yaml
docs/contracts/spec144-migration-matrix.v1.yaml
docs/contracts/spec144-proof-matrix.v1.yaml
docs/contracts/spec144-forbidden-placeholder-audit.v1.yaml
```

Agents MUST NOT infer the delivery graph from prose alone.

### 4.2 Normative source coverage

The source-coverage artifact MUST record:

```yaml
schema: focusa.spec144_normative_source_coverage.v1
spec_path:
spec_content_sha256:
spec_commit_sha:
extraction_tool_version:
extracted_at:
clause_count:
requirement_ids: []
unmapped_clause_refs: []
duplicate_or_weakened_mapping_refs: []
inherited_activated_requirement_refs: []
coverage_status: complete | incomplete | disputed
reviewed_by:
review_receipt_ref:
```

Every `MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, accepted `SHOULD`, activated `MAY`, required schema field, state transition, table obligation, acceptance criterion, closure blocker, and activated inherited clause MUST map to at least one ledger row without semantic weakening.

Any normative edit changes the source hash and invalidates prior source coverage, decomposition admission, and closure until regeneration and review complete.

### 4.3 Complete feature ledger

Minimum row:

```yaml
requirement_id:
source_spec:
source_spec_hash:
source_section_anchor:
exact_normative_text:
normative_class: must | must_not | shall | shall_not | should | may
applicability_class: required | conditional | optional_permission | prohibition
applicability_condition_ref:
applicability_status: active | conditional_inactive_verified | not_applicable_verified | disputed
applicability_decision_authority:
applicability_evidence_refs: []
primitive_owner:
implementation_owner:
repository_owners: []
implementation_phase:
dependency_requirement_refs: []
blocking_requirement_refs: []
implementation_task_refs: []
core_types: []
reducer_events: []
persistence_changes: []
api_operations: []
cli_commands: []
pi_tools: []
generated_contracts: []
client_surfaces: []
vertical_refs: []
reflex_refs: []
migration_refs: []
positive_tests: []
negative_tests: []
adversarial_tests: []
restart_recovery_tests: []
replay_tests: []
security_tests: []
privacy_tests: []
accessibility_tests: []
performance_tests: []
evidence_refs: []
receipt_refs: []
status: not_started | active | blocked | missing | partial | schema_only | shadow_only | implemented_unverified | verified | conditional_inactive_verified | not_applicable_verified | variance_approved_should_only | operator_removed_by_spec_amendment
variance_ref:
amendment_ref:
closure_impact_refs: []
```

### 4.4 Variance boundary

`variance_approved_should_only` is permitted only for a `SHOULD` clause. It MUST NOT be used for `MUST`, `MUST NOT`, `SHALL`, `SHALL NOT`, safety, security, privacy, identity, authority, evidence, settlement, compatibility, migration, replay, omission-firewall, or closure requirements.

A mandatory clause can change only through an operator-approved specification amendment.

### 4.5 Delivery DAG

The DAG MUST prove:

- every active mandatory requirement is reachable from the Spec 144 root closure node;
- every implementation task maps back to requirement IDs;
- every requirement has proof and closure destinations;
- no orphan task, orphan requirement, hidden parallel tree, or unowned cross-repository dependency exists;
- dependency cycles are absent or explicitly governed;
- all phase merge gates are represented;
- all Vertical and client feeder paths terminate in integrated acceptance.

### 4.6 Proof matrix

No ledger row may become `verified` without exact proof references appropriate to the requirement. Assertions, screenshots without lineage, local-only success where live behavior is required, one happy-path test, or a model summary are insufficient.

### 4.7 Every tranche publishes a no-omission statement

Every implementation tranche MUST publish:

1. included requirement IDs;
2. active mandatory requirement IDs excluded from the tranche, with an empty list unless they are explicitly scheduled by DAG dependency;
3. proof that scheduled-later requirements remain open in the parent graph;
4. newly discovered closure requirements;
5. exact code, schema, migration, and client changes;
6. positive and negative proof;
7. restart, replay, security, privacy, performance, and accessibility results activated by the tranche;
8. Evidence references;
9. tranche Receipt;
10. remaining open requirement IDs;
11. confirmation that no accepted requirement was removed, hidden, weakened, or marked complete by status substitution.

---

## 5. Implementation admission and activation gate

### 5.1 Documentation work before activation

Research, documentation, schema design, compatibility analysis, threat modeling, and non-mutating prototypes may proceed before the Spec 143 dependency closes. They MUST be labeled `documentation_only`, `contract_only`, or `shadow_only` and MUST NOT be reported as runtime implementation.

### 5.2 Production implementation gate

Production implementation MUST NOT begin until:

- Spec 143 closure Evidence and Receipt exist;
- the operator explicitly activates Spec 144 implementation;
- every artifact in §4.1 exists and validates;
- normative source coverage is complete;
- every active requirement has an owner, phase, dependencies, surfaces, migration, proof, Evidence, and Receipt destination;
- primitive ownership conflicts are resolved;
- all affected Spec 135–135K, 137, 138, 140, 141, 142, and 143 impacts are classified;
- no affected impact remains `unknown`;
- the delivery DAG has no ungoverned cycle or orphan;
- the exact source hash is frozen in an activation record.

### 5.3 Activation record

```yaml
schema: focusa.spec144_activation.v1
activation_id:
spec_path:
spec_content_sha256:
spec_commit_sha:
spec143_completion_receipt_ref:
operator_activation_ref:
normative_source_coverage_ref:
feature_ledger_ref:
delivery_dag_ref:
ownership_matrix_ref:
client_parity_matrix_ref:
vertical_pack_matrix_ref:
migration_matrix_ref:
proof_matrix_ref:
unknown_impact_refs: []
blocking_conflict_refs: []
status: eligible | blocked
activated_at:
activation_receipt_ref:
```

No agent may infer activation from the existence of this document, a version number, a branch, an issue, or partial code.

---

## 6. Primitive ownership and authority

| Concern | Primitive owner | Spec 144 responsibility |
|---|---|---|
| Canonical state transitions | Core reducer | Validate and route; never replace reducer authority |
| Ontology object/link/action identity | Specs 45–50 and 135F | Compile formal artifacts and obligation triggers |
| Domain-general cognition | Spec 61 | Reuse Mission, Task, Constraint, Risk, Blocker, Verification, and Evidence |
| Affordance/execution reality | Spec 66 | Validate capability, permission, precondition, cost, reliability, and reversibility |
| Status/lifecycle/provenance | Spec 70 | Generate consistent constraints and projections |
| Agent identity/role/permissions | Spec 72 | Bind Builder, Verifier, Router, and settlement roles |
| Reference resolution | Spec 74 | Prevent unsafe equivalence and identity merging |
| Projection/context | Specs 75 and 100 | Build bounded role-specific packets |
| Retention/decay | Spec 76 | Retain attempts, findings, calibration, conflicts, and supersession |
| Ontology governance | Spec 77 | Govern versions, packs, migration, compatibility, and deprecation |
| Secondary Cognition | Spec 78 | Supply subordinate proposal, critique, reflection, and verification roles |
| Continuous execution | Spec 79 | Schedule Build↔Verify and continuation |
| Workpoint | Specs 88, 125, and 143 | Remain immediate action authority and revision anchor |
| Reflexes | Spec 97 | Host universal and Vertical reflexes |
| Closure | Spec 116 | Consume verification coverage and settlement Evidence |
| Evidence/Receipts | Spec 119 | Preserve complete causal lineage |
| Adversarial specs | Spec 120 | Reuse portfolios while preserving operator approval |
| Silent Sessions | Spec 133 | Execute isolated Builder and Verifier sessions |
| Proposal-to-settlement | Spec 136 | Consume semantic validation and Build↔Verify phases |
| Temporal Authority | Spec 137 | Retain all clock, deadline, urgency, and estimate authority |
| Prediction/learning | Spec 138 | Retain forecast, resolution, scoring, calibration, learning, and transfer authority |
| Runtime constitution | Spec 140 | Compile role-specific instructions under one authority graph |
| Release/trace gates | Specs 141–143 | Enforce contracts, parity, Evidence, and no-pass truth |

Model fluency, confidence, role naming, broad context, repeated success, or long runtime MUST NOT grant canonical mutation, permission, deadline, resolution, scoring, settlement, learning-promotion, ontology-authoring, or routing-policy self-modification authority.

---

## 7. Governing architecture

```text
Operator direction + governing specification + ProjectIdentity + Workpoint revision
                                      │
                                      ▼
                              Semantic Work Contract
                                      │
                   ┌──────────────────┴──────────────────┐
                   ▼                                     ▼
          Builder cognition lineage             Obligation compiler
          isolated context + writer             RDF + OWL + SHACL + policy
                   │                                     │
                   ▼                                     ▼
          Frozen Build Attempt                  Verification Obligation Graph
                   │                                     │
                   └──────────────────┬──────────────────┘
                                      ▼
                         Domain-Specific Verification Router
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
          deterministic checks  specialist agents  proof environments
                    │                 │                 │
                    └─────────────────┼─────────────────┘
                                      ▼
                         Findings + Evidence + Coverage
                                      │
                         repair / escalate / abstain
                                      │
                                      ▼
                              Settlement evaluation
                                      │
                                      ▼
                                   Reducer
                                      │
                                      ▼
                     canonical settlement + Evidence + Receipt
```

RDF represents assertions, observations, and provenance. OWL derives bounded semantic implications. SHACL validates declared closed operational requirements. Deterministic validators inspect mechanically decidable properties. Domain-specific Verifiers attempt to falsify claims. The daemon supervises execution. The reducer alone records canonical settlement.

---

## 8. Standards and single-source semantic compilation

### 8.1 Mandatory initial profile

```text
RDF 1.1 abstract data model
JSON-LD 1.1 interchange
OWL 2 RL bounded reasoning
SHACL Core validation
SPARQL 1.1 bounded internal queries
PROV-O interoperability mapping
JSON Schema 2020-12 structural contracts
OpenAPI 3.0.3 HTTP contracts
```

Experimental standards MUST remain behind explicit compatibility flags and MUST NOT become canonical dependencies without a specification amendment, migration, compatibility proof, generated-contract parity, and operator approval.

### 8.2 Single-source law

The core Focusa semantic registry remains authoritative and MUST generate or deterministically bind:

```text
Rust types and registries
JSON Schema
OpenAPI
generated TypeScript contracts
RDF vocabulary
OWL modules
SHACL shape bundles
operation/tool bindings
Vertical pack contracts
conformance fixtures
```

Hand-maintained duplicates are prohibited when deterministic generation is possible. Build and release gates MUST fail on disagreement in identifiers, properties, datatypes, cardinality, class ranges, lifecycle transitions, actions, preconditions, Evidence, obligation triggers, pack ownership, compatibility, deprecation, or replacements.

### 8.3 OWL and SHACL boundary

OWL open-world consistency is not operational completeness. SHACL conformance is not empirical truth, authorization, or settlement.

```text
OWL may derive bounded candidate implications.
SHACL may prove conformance to a declared shape bundle.
Neither may mint canonical state, action authority, Evidence sufficiency, or settlement.
```

---

## 9. Semantic identity and named graphs

Every V2 definition, obligation, capability, finding, snapshot, validation, and settlement record MUST have a stable semantic IRI or deterministic one-to-one IRI mapping. IDs MUST NOT expose credentials, secrets, private corpus contents, or unsafe local paths.

Required named graphs:

```text
graph:registry
graph:shapes
graph:contract/{pair_id}
graph:builder/{pair_id}/{attempt_id}
graph:observations/{snapshot_id}
graph:inference/{snapshot_id}/{reasoner_version}
graph:verifier/{pair_id}/{round_id}
graph:response/{pair_id}/{response_id}
graph:settlement/{pair_id}
graph:quarantine/{scope_id}
```

Builder assertions MUST NOT become observations. Reasoner inferences MUST remain distinguishable from explicit assertions. Verifier findings MUST remain claims. Quarantined material remains auditable but cannot enter ordinary context, routing, action, promotion, or settlement.

Every projection MUST preserve epistemic class, including:

```text
operator_asserted
user_asserted
deterministic_asserted
tool_observed
runtime_observed
reducer_asserted
model_proposed
model_inferred
reasoner_inferred
verification_confirmed
legacy_assumed
contradicted
invalid
quarantined
unsupported_opaque
```

---

## 10. Mandatory SHACL profile families

Focusa MUST implement purpose-specific profiles when their requirements are activated:

1. **Intake:** identity, namespace, datatype, bounds, prohibited properties, provenance, scope, compatibility.
2. **Promotion:** required properties/links, cardinality, lifecycle, Evidence, freshness, identity resolution, contradictions, pack compatibility, candidate/canonical separation.
3. **Action preflight:** actor, role, permission, scope, target, preconditions, blockers, reversibility, idempotency, side effects, timeout, retry, Evidence, tool mapping.
4. **Verification plan:** obligations, eligibility, independence, DAG dependencies, snapshot binding, coverage, assurance, tools, data access.
5. **Finding/verdict:** exact targets, Evidence, method, severity, freshness, scope, reproduction, valid dispositions.
6. **Settlement:** mandatory coverage, snapshot identity, Evidence sufficiency, no open critical findings, approval, Receipt readiness, reducer-only transition.
7. **Domain pack:** manifest, namespace, interfaces, OWL profile, shape integrity, lifecycle, migration, trusted origin, bounds.
8. **Migration/replay:** historical compatibility, graph identity, Evidence preservation, unknown semantics, V1 projection equivalence, post-migration conformance.
9. **Vertical bundle:** required pack presence, overlay compatibility, reflex validity, temporal/epistemic applicability, client projection completeness.
10. **Omission firewall:** requirement IDs, active applicability, task/proof ownership, no orphaned clauses, no forbidden placeholder completion.

A required validation MUST emit a durable `SemanticValidationReceipt`. Strings such as `passed`, `accepted`, `verified`, `approved`, or `success` are invalid substitutes.

Minimum receipt:

```yaml
schema: focusa.semantic_validation_receipt.v1
validation_id:
validation_purpose:
target_ref:
semantic_pair_id:
project_root:
continuity_id:
workpoint_ref:
registry_version:
domain_pack_versions: []
shape_bundle_id:
shape_bundle_hash:
data_graph_hash:
inference_graph_hash:
inference_profile:
reasoner_implementation:
reasoner_version:
validator_implementation:
validator_version:
conforms:
severity_counts: {}
results: []
policy_refs: []
evidence_refs: []
created_at:
expires_at:
receipt_hash:
```

---

## 11. Semantic Work Contract and execution pair

### 11.1 Work Contract

The Work Contract MUST bind:

- ProjectIdentity, root, and continuity;
- Trajectory and exact Workpoint revision;
- exact current operator direction;
- governing specification hashes;
- acceptance criteria and complete requirement IDs;
- active domain packs and versions;
- ontology, OWL, and SHACL bundle versions;
- allowed actions and scope;
- constraints, blockers, and risk;
- required Evidence;
- Builder, verification, independence, disclosure, settlement, temporal, epistemic, and resource policies;
- contract hash and immutable frozen revision.

Acceptance and evaluation contracts MUST freeze before building. Amendments require the governing specification/operator path, a new contract revision, invalidation analysis, and re-verification.

### 11.2 Execution Pair

A `SemanticExecutionPair` MUST coordinate one Builder lineage and one Verification Portfolio while preserving separate Silent Session, run, stream, checkpoint, lease, Evidence, and Receipt identities.

```yaml
schema: focusa.semantic_execution_pair.v1
semantic_pair_id:
work_contract_ref:
workpoint_ref:
workpoint_revision:
builder_assignment_ref:
verification_plan_ref:
active_build_attempt_ref:
active_snapshot_ref:
settlement_evaluation_ref:
requirement_coverage_ref:
receipt_refs: []
state:
created_at:
updated_at:
```

---

## 12. Builder cognition lineage

The Builder may inspect authorized context, mutate only its leased workspace, use permitted tools, run tests, produce Evidence, submit typed claims, respond to findings, and request escalation.

The Builder MUST NOT alter frozen criteria, shapes, scoring, or policy; modify findings; inspect hidden verifier material outside disclosure policy; self-settle; self-authorize; or issue its own completion Receipt.

The `BuilderContextPacket` MUST contain exact direction, identity, Workpoint revision, specifications, requirement IDs, constraints, blockers, risks, relevant ontology slice, accepted findings, authorized actions, workspace/lease, drift boundaries, and temporal/resource posture. It MUST exclude hidden tests, irrelevant verifier history, and settlement internals not needed for execution.

Every `BuildAttempt` MUST bind its source snapshot, result snapshot, changed semantic/artifact refs, claims, Evidence, tests, known blockers, session/run identities, and status.

---

## 13. Verification obligation compiler

### 13.1 Core law

```text
Do not route the whole task to one Verifier.
Compile semantic impact into complete verification obligations.
Route each obligation to eligible independent capabilities.
```

Compilation MUST consume:

```text
operator direction
+ Workpoint revision
+ every active requirement ID
+ governing specs and criteria
+ Builder actions
+ changed objects, links, and artifacts
+ active Vertical/domain packs
+ permissions, authority, risk, reversibility
+ Evidence requirements
+ Spec 137 temporal applicability
+ Spec 138 epistemic applicability
+ OWL inferences
+ SHACL obligation triggers
→ Verification Obligation Graph
```

Registered deterministic requirements MUST be emitted without model interpretation. Model cognition may add obligations but MUST NOT remove, merge away, weaken, or mark mandatory obligations optional.

Minimum obligation:

```yaml
schema: focusa.verification_obligation.v1
obligation_id:
obligation_type_id:
requirement_refs: []
domain_pack_id:
verification_dimension_ids: []
target_refs: []
acceptance_criterion_refs: []
action_type_refs: []
artifact_refs: []
risk_refs: []
derived_from_assertion_refs: []
derived_from_inference_refs: []
derivation_policy_ref:
required_provider_classes: []
required_capability_ids: []
required_tool_capabilities: []
required_evidence_kinds: []
minimum_verifier_count:
minimum_independent_verifier_count:
minimum_assurance_tier:
criticality:
settlement_blocking:
operator_approval_required:
dependency_obligation_ids: []
completion_shape_ref:
freshness_policy_ref:
status: proposed | required | assigned | active | satisfied | blocked | superseded | operator_removed_by_spec_amendment
```

No generic verdict may collapse distinct security, schema, API, UI, runtime, temporal, Evidence, scope, prediction, learning, or authority obligations.

---

## 14. Domain-Specific Verification Router

The Router MUST resolve the obligation graph into the minimum **complete** policy-compliant portfolio. `Minimum` means no unnecessary duplicate work after mandatory coverage, independence, assurance, and Evidence are satisfied. It MUST NOT mean reduced coverage.

Provider classes:

```text
DeterministicValidator
FormalSemanticValidator
StaticAnalyzer
TestExecutor
RuntimeProbe
BrowserEvaluator
DomainVerifierAgent
CrossModelVerifier
EvidenceAuditor
ExternalAuthority
OperatorReviewer
```

A verifier capability is not merely a model name. It MUST bind domain/obligation support, artifact types, tools, Evidence generation, workspace capabilities, harness adapters, model policy, context policy, finding/verdict shapes, assurance range, calibration, reliability, latency, cost, security classification, data access, and permission profile.

```yaml
schema: focusa.verifier_capability_profile.v1
verifier_id:
actor_identity_ref:
role_profile_ref:
supported_domain_pack_ids: []
supported_obligation_type_ids: []
supported_verification_dimension_ids: []
supported_artifact_kinds: []
supported_action_type_ids: []
supported_risk_classes: []
tool_capability_ids: []
evidence_generation_capabilities: []
workspace_capabilities: []
harness_adapter_ids: []
model_policy:
  provider_allowlist: []
  model_family_allowlist: []
  model_allowlist: []
  thinking_profiles: []
  cross_family_eligible:
  local_only_supported:
context_policy_ref:
finding_shape_ref:
verdict_shape_ref:
minimum_assurance_tier:
maximum_assurance_tier:
calibration_profile_ref:
reliability_profile_ref:
latency_profile_ref:
cost_profile_ref:
security_classification:
data_access_classes: []
permission_profile_ref:
deprecated:
replacement_verifier_id:
version:
```

A capability profile without a live provider, valid tool path, current calibration posture, and executable conformance proof is `schema_only` and is ineligible for settlement coverage.

The Router MUST filter by eligibility and MUST output a dependency-aware `VerificationPlan` with:

- exact snapshot;
- requirement and obligation coverage;
- assignments and dependencies;
- assurance tier;
- independence requirements;
- tools and environments;
- cost/latency budgets;
- routing reasons and policy;
- validation receipt;
- uncovered obligations, which MUST be empty before authorization.

```yaml
schema: focusa.verification_plan.v1
verification_plan_id:
semantic_pair_id:
work_contract_ref:
workpoint_ref:
build_attempt_ref:
verification_snapshot_ref:
obligation_graph_ref:
requirement_coverage_ref:
assignments:
  - assignment_id:
    obligation_ids: []
    requirement_refs: []
    verifier_id:
    provider_class:
    session_policy_ref:
    context_policy_ref:
    workspace_policy_ref:
    tool_policy_ref:
    model_policy_ref:
    independence_requirement_ref:
    execution_order:
    dependency_assignment_ids: []
coverage:
  required_obligation_ids: []
  covered_obligation_ids: []
  uncovered_obligation_ids: []
  redundantly_covered_obligation_ids: []
assurance_tier:
estimated_cost:
estimated_latency:
resource_policy_ref:
routing_policy_ref:
routing_reason_refs: []
validation_receipt_ref:
status: proposed | valid | invalid | authorized | executing | completed | superseded
```

The Router may suppress only optional redundant checks after mandatory coverage is proven. It MUST NOT remove mandatory obligations, weaken assurance, alter criteria, grant permissions, waive independence, treat unavailability as pass, resolve critical disagreement, settle completion, or self-promote policy learning.

---

## 15. Verifier cognition lineages

Each assignment MUST receive an independently generated context containing only its obligations, criteria, immutable snapshot, applicable semantic modules/shapes, threat/risk model, approved tools, workspace, and disclosure policy.

By default, Verifiers MUST NOT receive Builder chain-of-thought, full Builder transcript, Builder confidence rhetoric, irrelevant attempts, mutable source state, or material outside scope.

A Verifier MUST NOT acquire the Builder project writer lease. Source access is read-only or immutable. A writable verification sandbox is permitted. Repairs are proposals or artifacts, never direct project mutation.

Continuation defaults to `carry_open_findings_only`. Any broader lineage reuse MUST be recorded and reflected in the independence profile.

Disclosure modes are `blind_behavioral`, `gray_box`, and `white_box`. Hidden-test policy MUST preserve enough reproduction information for fair repair while preventing overfitting.

---

## 16. Immutable Verification Snapshot

Every required verifier MUST inspect the same immutable content-addressed snapshot or a declared derivative with explicit lineage and hash.

The snapshot MUST bind source, diff, semantic graph, Evidence, tests, runtime observations, temporal state, Work Contract, Workpoint revision, registry version, domain-pack versions, shape versions, exclusions, unavailable material, hashes, and immutable status.

```yaml
schema: focusa.verification_snapshot.v1
snapshot_id:
semantic_pair_id:
build_attempt_ref:
work_contract_ref:
workpoint_ref:
workpoint_revision:
source_snapshot_ref:
diff_snapshot_ref:
semantic_graph_snapshot_ref:
evidence_snapshot_ref:
test_snapshot_ref:
runtime_observation_snapshot_ref:
temporal_snapshot_ref:
registry_version:
domain_pack_versions: []
shape_bundle_versions: []
excluded_or_unavailable_refs: []
content_hashes: {}
created_at:
immutable:
status: freezing | frozen | invalid | superseded
```

Any change to source, diff, criteria, Workpoint, Evidence, tests, temporal state material to the obligation, domain pack, registry, shape, or policy MUST invalidate affected verification and settlement readiness.

The settled snapshot MUST equal the verified snapshot. Repair creates a new snapshot and MUST rerun every invalidated obligation. No stale pass may be carried forward by convenience.

---

## 17. Findings, verdicts, and verification of the Verifier

A `VerificationFinding` MUST identify exact obligation, requirement, claim/criterion/semantic targets, type, severity, confidence, uncertainty, summary, reasoning summary, Evidence, reproduction or inspection method, impact, requested repair, settlement-blocking posture, provenance, freshness, and status.

```yaml
schema: focusa.verification_finding.v1
finding_id:
semantic_pair_id:
verification_round_id:
verifier_assignment_ref:
obligation_ref:
requirement_refs: []
target_claim_refs: []
target_criterion_refs: []
target_semantic_refs: []
finding_type:
severity:
confidence:
uncertainty_ref:
summary:
reasoning_summary:
evidence_refs: []
reproduction_refs: []
inspection_method_refs: []
impact_refs: []
requested_repair:
settlement_blocking:
provenance_ref:
fresh_until:
created_at:
status: open | responded | confirmed | partially_confirmed | reproduced | not_reproduced | unsupported | withdrawn | superseded | accepted_risk | operator_resolved | inconclusive
```

A Verifier is not presumed correct. Every finding MUST pass structural, semantic, scope, Evidence, freshness, and eligibility validation.

A finding may be rejected or narrowed only through a recorded disposition supported by Evidence, including wrong target, outside scope, missing Evidence, failed reproduction, stale state, preference mistaken for requirement, spec contradiction, invalid test, graph nonconformance, or disproving counterevidence.

The Builder may respond with repair mapping, dispute, counterevidence, or clarification request but MUST NOT self-dismiss a finding.

A positive verdict is invalid if any required criterion was uninspected, Evidence is unknown/insufficient, a required provider failed, an open critical finding exists, snapshot identity is stale, independence is insufficient, or finding/verdict graphs fail conformance.

Unsupported criticism MUST NOT block indefinitely. It must reach a governed `unsupported`, `not_reproduced`, `withdrawn`, `operator_resolved`, or other evidence-backed disposition.

---

## 18. Independence and assurance

Every pair MUST record Builder/Verifier actor, session, run, context hash, model provider/family/version, workspace separation, writer posture, shared transcript/reasoning, independent Evidence acquisition, independent test generation, frozen snapshot, tier, score, and degradation reasons.

```yaml
schema: focusa.verifier_independence_profile.v1
builder_actor_ref:
verifier_actor_ref:
builder_session_id:
builder_run_id:
verifier_session_id:
verifier_run_id:
builder_context_packet_hash:
verifier_context_packet_hash:
builder_model:
  provider:
  family:
  model:
  version:
verifier_model:
  provider:
  family:
  model:
  version:
same_model:
same_model_family:
same_provider:
workspace_separation:
verifier_read_only:
verifier_has_writer_lease:
shared_hidden_reasoning:
shared_transcript:
independent_evidence_acquisition:
independent_test_generation:
frozen_snapshot_verified:
independence_tier:
independence_score:
degraded_reasons: []
```

Separate session IDs, role prompts, or context labels alone do not prove independence.

Assurance tiers:

- **Tier 0:** deterministic conformance for harmless mechanically decidable work.
- **Tier 1:** separate-context, no-writer verification for low-risk reversible work.
- **Tier 2:** cross-model verification for meaningful code, architecture, data, integration, specification, prediction, and temporal work.
- **Tier 3:** multi-aspect deterministic and specialist verification for security, migration, compliance, release, destructive operations, high-consequence predictions, temporal commitments, and public proof.
- **Tier 4:** operator or external authority for judgmental, disputed, regulated, legal, safety-critical, or unresolved cases.

An applicability policy MUST choose the tier. An implementation may not downgrade the tier because of cost, delay, availability, or preferred model.

No majority vote may override an obligation-specific veto or a valid critical finding. Settlement uses coverage, severity, Evidence, reproduction, eligibility, independence, calibration, contradiction, veto/escalation policy, and SHACL conformance.

---

## 19. Build↔Verify lifecycle and rerouting

Main path:

```text
draft
→ contract_validating
→ contract_frozen
→ builder_initializing
→ building
→ build_claimed
→ snapshot_freezing
→ deterministic_validating
→ semantic_reasoning
→ verification_routing
→ verifying
→ verification_passed
→ settlement_evaluating
→ settlement_ready
→ settled
```

Challenge path:

```text
verifying
→ challenged
→ repair_requested
→ repairing
→ build_claimed
→ snapshot_freezing
→ obligation_recompilation
→ verifying
```

Exceptional states include `inconclusive`, `verification_blocked`, `operator_required`, `budget_exhausted`, `oscillation_detected`, `scope_invalidated`, `snapshot_invalidated`, `routing_conflicted`, `superseded`, `cancelled`, and `failed`.

`verification_passed` is not `settled`.

Deep verification MUST run at every policy-activated semantic checkpoint, including consequential plan formation, risky mutation, major revision, unexpected deterministic failure, material Evidence change, Workpoint satisfaction, closure, release/migration/deployment/irreversible action, material operator steering, and completion claim.

Rerouting MUST occur when new semantic impact, critical findings, environment failure, verifier unavailability, tool mismatch, unexpected artifact, scope/authority mismatch, disagreement, insufficient Evidence, repeated unsupported findings, steering, snapshot supersession, or pack/policy revision changes the obligation graph.

Every reroute MUST preserve prior plan, trigger Evidence, assignment changes, unresolved obligations, reason/policy, cost/latency impact, validation, and supersession. Mandatory obligations cannot disappear.

The loop MUST stop or escalate on repeated dispute without Evidence delta, repeated repair failure, repeated unsupported findings, hidden-test overfitting, budget/deadline conflict requiring operator choice, or governing-spec conflict. `Inconclusive` is a valid result and cannot be converted to pass.

---

## 20. Spec 137 Temporal Authority integration

Spec 144 improves effective temporal correctness by validating interpretation, applicability, Evidence, and domain policy. It does not improve physical clock, synchronization, provider timestamp, or network accuracy.

The Router MUST support temporal obligations for trusted-clock integrity, source independence, calendar authority, civil-time resolution, deadline completeness, readiness-margin grounding, Evidence freshness, critical-path grounding, boundary uncertainty, breach classification, remaining opportunity, and settlement time.

Every high-consequence Vertical MUST declare a `DomainTemporalApplicabilityProfile` binding object/action classes, clock domains, precision, calendar sources, deadline semantics, completion effects, freshness, latency, uncertainty, breach policy, required obligations, verifier capabilities, Evidence, fail mode, and version. Verticals extend Spec 137 and MUST NOT fork temporal authority.

```yaml
schema: focusa.domain_temporal_applicability_profile.v1
profile_id:
vertical_id:
domain_pack_ref:
applicable_object_type_ids: []
applicable_action_type_ids: []
clock_domain_policy_refs: []
precision_profile_refs: []
calendar_source_policy_refs: []
deadline_semantics: []
completion_effects: []
freshness_policy_refs: []
latency_policy_refs: []
uncertainty_policy_refs: []
breach_policy_refs: []
required_verification_obligation_ids: []
required_verifier_capability_ids: []
required_evidence_kinds: []
fail_mode:
version:
```

Temporal reflexes MUST be registered when their activation conditions exist, including authority freshness, civil-time resolution, unsupported precision rejection, calendar-version checks, deadline recomputation, unverified deadline blocking, overdue opportunity validation, overdue focus, and specialist escalation.

A missing required temporal profile, stale calendar, unsupported precision, unresolved clock uncertainty, or unavailable temporal authority blocks affected verification and settlement. It cannot be marked `not applicable` merely because the Vertical omitted the profile.

---

## 21. Spec 138 Prediction and Epistemic Governance integration

The Router MUST support obligation-specific verification for information-set integrity, source authority/independence, leakage/contamination, forecast shape, uncertainty, scenario coherence, outcome resolution, scoring, calibration, decision value, causal claims, transfer applicability, and learning promotion.

Required obligation vocabulary:

```text
epistemic.information_set_integrity
epistemic.source_authority
epistemic.source_independence
epistemic.leakage_and_contamination
epistemic.forecast_shape_validity
epistemic.uncertainty_decomposition
epistemic.scenario_coherence
epistemic.outcome_resolution
epistemic.scoring_policy
epistemic.calibration
epistemic.decision_value
epistemic.causal_claim
epistemic.transfer_applicability
epistemic.learning_promotion
```

Required capability vocabulary:

```text
InformationSetIntegrityVerifier
SourceAuthorityVerifier
SourceIndependenceVerifier
LeakageAndContaminationVerifier
ForecastShapeValidator
UncertaintyVerifier
ScenarioCoherenceVerifier
OutcomeResolutionVerifier
ScoringPolicyVerifier
CalibrationVerifier
DecisionValueVerifier
CausalClaimVerifier
TransferApplicabilityVerifier
LearningPromotionVerifier
```

Calibration MUST be cohortable by verifier role/capability, obligation, strategy, model family, artifact, Vertical/domain pack, risk, assurance, and source-versus-transfer context.

Learning-promotion portfolios MUST separately cover Evidence, outcome authority, experimental validity, confounders, controls, transfer, regression, applicability, expiry, rollback, conflict, security, and governance when activated. A fluent causal narrative cannot satisfy these obligations.

When ground truth, samples, target definitions, source independence, resolution authority, or applicability are inadequate, the result MUST remain `unknown`, `insufficient_evidence`, `not_resolvable_yet`, `abstain`, `operator_judgment_required`, or `experimental_only`. More verification cannot manufacture certainty.

Router and verifier-selection learning MUST use immutable champion/challenger governance, fixed replay/live-shadow evaluation, operator/governance promotion, and rollback. The Router cannot mutate its live canonical policy from its own outcomes.

---

## 22. Vertical Intelligence Bundles

A Vertical MUST be added through a versioned `VerticalIntelligenceBundle`, never by forking Focusa, hard-coding routing, or treating a visual theme as semantic capability.

Minimum bundle:

```yaml
schema: focusa.vertical_intelligence_bundle.v1
vertical_id:
version:
workspace_view_profile_ref:
domain_pack_refs: []
ontology_module_refs: []
owl_module_refs: []
shacl_shape_bundle_refs: []
verification_obligation_refs: []
verifier_capability_refs: []
routing_policy_ref:
reflex_overlay_refs: []
temporal_applicability_profile_ref:
prediction_applicability_profile_ref:
learning_applicability_profile_ref:
artifact_interpretation_refs: []
evidence_policy_refs: []
connector_requirement_refs: []
migration_refs: []
conformance_suite_refs: []
golden_scenario_refs: []
requirement_refs: []
status:
```

Layers:

1. Workspace View Profile — presentation only; cannot redefine truth.
2. Domain Pack — entities, relations, actions, status, lifecycle, Evidence, identity, authority, slice policy.
3. Verification Pack — obligations, dimensions, verifier profiles, tools, Evidence, assurance, independence, veto, escalation, shapes.
4. Reflex Overlay — small typed domain routines.
5. Temporal Applicability — Spec 137 extension.
6. Prediction/Learning Applicability — Spec 138 extension.

A new ontology is not required for presentation, terminology, layout, existing-pack composition, artifact rendering, policy over existing classes, connector output using existing Evidence, or workspace profile only.

A new ontology module is mandatory when omitting a unique entity, relation, action/precondition, Evidence standard, lifecycle, identity rule, temporal semantic, prediction target/outcome/scorer/regime, or authority boundary could cause wrong action, wrong Evidence trust, identity/completion error, missed deadline, incorrect resolution, authority violation, or invalid learning transfer.

One shared Verification Fabric Ontology MUST define verification domains, dimensions, obligations, requirements, capabilities, assignments, plans/DAGs, snapshots, findings, dispositions, coverage, independence, reroutes, and settlement evaluation. Verticals extend it and MUST NOT duplicate shared primitives.

Projects may activate multiple bundles. Every active pack’s mandatory obligations survive composition. No workspace selection may suppress them. Pack conflicts MUST block activation or affected execution and produce explicit migration/governance/operator review.

Vertical activation MUST validate identity/version/trusted origin, resolve dependencies, compile all contracts, validate OWL and SHACL, register obligations/capabilities/reflexes, declare Spec 137/138 applicability, preview migration, run conformance/golden scenarios, receive operator/project approval, resolve the project registry, and generate bounded projections.

A missing required pack produces a truthful blocked or degraded workspace, but the Vertical remains incomplete and cannot claim operational support.

---

## 23. Reflex architecture

Reflexes preserve:

```text
Trigger → Context inputs → Reflex action → Evidence output → Escalation boundary
```

They MUST remain small, typed, inspectable, operator-governed, context-fed, authority-bounded, degradable, and composable.

Shared required reflex catalog when activated:

```text
detect_verification_domain_impact
compile_verification_obligations
detect_uncovered_mandatory_obligation
detect_cross_domain_verification_conflict
freeze_verification_snapshot
invalidate_verification_after_snapshot_change
resume_open_verification_obligations
supersede_stale_verifier_context
route_verification_portfolio
enforce_verifier_capability_eligibility
enforce_verifier_independence
escalate_assurance_tier
reroute_on_new_finding
reroute_on_verifier_failure
require_evidence_for_verifier_finding
reject_unsupported_critical_finding
block_settlement_on_open_critical_finding
block_settlement_on_uncovered_obligation
verify_final_snapshot_matches_verified_snapshot
retry_verifier_with_bounded_fallback
replace_unavailable_verifier
route_disagreement_to_arbiter
escalate_inconclusive_verification
detect_build_verify_oscillation
record_verifier_prediction
evaluate_verifier_after_settlement
detect_verifier_false_positive_pattern
detect_verifier_false_negative_pattern
detect_negative_verifier_transfer
propose_verifier_policy_adjustment
```

A Vertical reflex overlay MUST declare exact triggers, context, actions, Evidence, escalation, authority, budget, failure envelope, tests, and requirement IDs. A reflex name in a registry without executable routing and proof is `schema_only`, not implemented.

Vertical overlay template:

```text
verify_<vertical>_source_authority
enforce_<vertical>_freshness
resolve_<vertical>_identity
detect_<vertical>_scope_conflict
require_<vertical>_evidence_bundle
enforce_<vertical>_action_preconditions
check_<vertical>_deadline_or_calendar
detect_<vertical>_policy_or_regime_change
resolve_<vertical>_outcome
evaluate_<vertical>_prediction
detect_<vertical>_negative_transfer
revalidate_on_<vertical>_pack_revision
```

Illustrative Legal reflexes:

```text
verify_jurisdiction_before_filing
resolve_controlling_authority
check_citation_currency
check_court_calendar_version
require_filing_acceptance_evidence
detect_privilege_boundary
block_unverified_legal_deadline_claim
route_conflicting_authority_to_legal_verifier
```

Illustrative Markets reflexes:

```text
verify_market_session_state
reject_stale_market_data
verify_sequence_continuity
freeze_information_set_before_forecast
require_pretrade_risk_verification
block_expired_execution_intent
reconcile_order_before_retry
evaluate_forecast_after_resolution
detect_regime_shift
```

Illustrative Research reflexes:

```text
detect_shared_upstream_source
require_independent_source_support
surface_contradictory_evidence
freeze_claim_information_set
route_causal_claim_to_methodology_verifier
abstain_on_unresolvable_claim
evaluate_claim_after_outcome
decay_superseded_source_learning
```

---

## 24. Settlement protocol

Settlement MUST evaluate:

- exact Work Contract and Workpoint revision;
- complete active requirement coverage;
- final immutable snapshot;
- structural and semantic validation receipts;
- obligation coverage;
- deterministic results;
- findings and dispositions;
- Evidence sufficiency and freshness;
- verifier eligibility, independence, and calibration;
- temporal and epistemic applicability;
- unresolved blockers, contradictions, and pack conflicts;
- operator/external approval;
- migration and compatibility posture;
- client parity obligations;
- Receipt readiness.

Outcomes:

```text
settled_complete
settled_partial
blocked
failed
inconclusive
operator_required
external_authority_required
cancelled
superseded
```

`settled_partial` MUST identify unsatisfied requirement IDs and cannot close the WorkItem, parent specification, release, or conformance class whose target state requires them.

Settlement MUST fail closed on uncovered mandatory obligations, unavailable required verifiers, open critical findings, insufficient/stale Evidence, snapshot mismatch, inadequate independence, unresolved pack conflict, unavailable temporal/outcome authority, incomplete validation, missing approval, unknown impact, missing migration, or incomplete client parity.

No mandatory requirement may be waived by a runtime variance. Mandatory change requires a specification amendment. `SHOULD` variances remain visible, scoped, expiring, Receipt-backed, and nonconforming to classes requiring original behavior.

---

## 25. API, events, persistence, and clients

Required operation families are:

```text
semantic.integrity.registry
semantic.integrity.validate
semantic.integrity.reason.preview
semantic.integrity.reason.explain
semantic.integrity.receipt.get
semantic_pair.create
semantic_pair.get
semantic_pair.pause
semantic_pair.resume
semantic_pair.cancel
semantic_pair.contract.preview
semantic_pair.contract.commit
semantic_pair.builder.start
semantic_pair.builder.claim
semantic_pair.builder.respond
semantic_pair.builder.repair
semantic_pair.snapshot.freeze
semantic_pair.snapshot.get
semantic_pair.obligations.compile
semantic_pair.verification.plan.preview
semantic_pair.verification.plan.commit
semantic_pair.verify.start
semantic_pair.verify.findings
semantic_pair.verify.verdict
semantic_pair.finding.respond
semantic_pair.finding.resolve
semantic_pair.settlement.preview
semantic_pair.settlement.commit
semantic_pair.receipt.get
semantic_pair.replay
semantic_pair.eval
vertical.bundle.validate
vertical.bundle.preview
vertical.bundle.activate
vertical.bundle.conformance
```

Exact API, CLI, Pi, MCP/OpenAI/REST, generated-client, and UI bindings MUST derive from the Operation Registry. Clients MUST NOT reproduce routing, validation, settlement, or ontology authority locally.

Required event families include:

```text
semantic_pair.created
semantic_pair.contract_frozen
builder.session_bound
builder.context_bound
builder.attempt_started
builder.attempt_claimed
builder.repair_started
builder.response_submitted
verification.obligations_compiled
verification.plan_proposed
verification.plan_validated
verification.plan_authorized
verification.plan_rerouted
verification.snapshot_started
verification.snapshot_frozen
verification.deterministic_started
verification.deterministic_completed
verification.reasoning_started
verification.reasoning.completed
verifier.session_bound
verifier.context_bound
verifier.round_started
verifier.finding_created
verifier.finding_revised
verifier.finding.withdrawn
verifier.verdict_submitted
finding.response_submitted
finding.disposition_changed
settlement.evaluation_started
settlement.blocked
settlement.operator_required
settlement.ready
settlement.committed
vertical.bundle_validated
vertical.bundle_activated
vertical.bundle_deactivated
vertical.pack_conflict_detected
semantic_pair.completed
semantic_pair.failed
semantic_pair.cancelled
```

Every event MUST carry applicable scope, identity, contract, snapshot, registry, schema, requirement, and correlation references.

Use existing SQLite event/snapshot and Reference Store architecture. Replay MUST reconstruct every contract, attempt, snapshot, requirement/obligation, plan, assignment/context, finding/response/disposition, validation, reroute, settlement, and Receipt.

Legacy bools and verdict strings remain readable only as `legacy_advisory_verdict` or `legacy_assumed_verification`; they cannot satisfy strict independent verification.

Unknown authoritative semantics block replay or mutation. Unknown non-authoritative material may be preserved with diagnostics but cannot influence action or settlement.

Clients MUST display truthful states, including `schema_only`, `pack_missing`, `migration_required`, `verification_required`, `verification_blocked`, `operator_required`, `unsupported_future_definition`, `writer_blocked`, `degraded`, `stale`, `conflicted`, and `quarantined`.

---

## 26. Security, privacy, and identity

Mandatory controls include trusted namespaces/origins, governed signatures, no hot remote imports, no arbitrary untrusted SHACL-SPARQL in the initial profile, query/node/edge/depth/time/memory/result bounds, recursive-shape and reasoner-DoS defense, ontology/source/prompt poisoning tests, no Verifier writer lease, project/continuity isolation, IRI collision prevention, Evidence access control, data-class-aware eligibility, redaction-aware export, tamper-evident receipts, identity-merge escalation, and secret exclusion.

An agent MUST NOT author canonical `owl:sameAs`. Similarity creates an identity-resolution candidate. Strict equivalence requires governed Evidence and identity authority.

Security, privacy, privilege, regulated, legal, and high-consequence obligations cannot be downgraded or marked not applicable because the required verifier/tool is unavailable.

---

## 27. Performance and resource laws

Whole-world reasoning MUST NOT run synchronously per turn. Bundles MUST be content-hash cached. Candidate validation MUST use bounded deltas and affected neighborhoods. Traversals and validators MUST have explicit bounds. Large graphs/transcripts remain behind handles.

Low-memory mode may reduce explanation detail but cannot remove authority, hashes, violations, critical findings, requirement IDs, or recovery guidance. Validation timeout is not conformance. Required timeout blocks or invokes an explicitly authorized equivalent-or-stronger fallback. A weaker fallback cannot satisfy the obligation.

Optimization MUST prefer deterministic proof when stronger and cheaper but MUST NOT trade away mandatory coverage, Evidence, independence, client parity, migration, replay, or closure proof.

Resource or deadline pressure cannot make accepted work disappear. It may trigger operator prioritization, a blocker, or an amendment request; until resolved, requirements remain open.

---

## 28. Evaluation, calibration, and proof

Required comparisons:

```text
Builder-only
same-model self-review
same-model separate-context
cross-family verification
deterministic + model verification
multi-aspect portfolio
```

Required metrics include task success, criterion and requirement coverage, defect escape, verifier false positives/negatives, critical precision, reproduction, repair, rounds, oscillation, anchoring, Evidence linkage, unsupported objections, overfitting, routing eligibility, unnecessary verification, temporal errors, unsupported estimate rejection, information leakage, source dependence, resolution/scoring errors, learning-promotion reversal, negative transfer, resource cost, operator interventions, and post-settlement regressions.

Golden scenarios MUST prove at least:

1. domain-specific defect prevention;
2. false finding rejection;
3. repair and re-verification;
4. fail-closed outage;
5. operator-resolved inconclusive case;
6. cross-domain obligation compilation;
7. temporal/calendar error detection;
8. unsupported estimate refusal;
9. information-set leakage detection;
10. dependent-source detection;
11. invalid causal learning blocked;
12. negative transfer narrows learning;
13. presentation-only Vertical avoids unnecessary ontology;
14. semantic Vertical activates all packs/reflexes;
15. pack conflict blocks;
16. snapshot change invalidates passes;
17. rerouting after new risk;
18. full replay;
19. unmapped normative clause blocks implementation;
20. forbidden placeholder audit detects hidden deferral;
21. a blocked or scheduled-later row remains open and blocks parent closure;
22. a `not_applicable` claim without evidence is rejected;
23. a partial client implementation cannot satisfy parity;
24. an unknown Spec 135/137/138 impact blocks promotion;
25. a runtime variance cannot waive a mandatory clause.

Every benchmark MUST preserve exact code, model, prompt, policy, registry, pack, shape, data, environment, and source hashes.

---

## 29. Mandatory implementation sequence

All phases are mandatory dependency order, not feature tiers or deferral buckets.

### Phase 0 — Reality, source coverage, and closure graph

Complete all §4 artifacts, current-runtime inventory, fixed fixtures, threat model, ownership map, migration map, compatibility map, forbidden-placeholder audit, and activation record.

### Phase 1 — Formal semantic compilation

Implement stable IDs, registry generation, RDF/OWL/SHACL, parity gates, Verification Fabric Ontology, pack conformance, and omission-firewall shapes.

### Phase 2 — Shadow validation

Run candidate graph and validation in non-blocking shadow mode to measure correctness and performance. Shadow proof cannot satisfy runtime enforcement or full conformance.

### Phase 3 — Execution pair substrate

Implement Work Contract, pair, Builder context/attempt, immutable snapshot, Verifier context, read-only posture, persistence, replay, and contracts.

### Phase 4 — Obligation compiler and Router

Implement requirement-aware obligation graph, capability registry, plan DAG, eligibility/coverage/independence validation, deterministic providers, specialist sessions, and fail-closed availability.

### Phase 5 — Findings, repair, and settlement

Implement typed findings/responses/dispositions, Verifier validation, rerouting, settlement shapes, reducer decision, closure integration, Evidence, and Receipts.

### Phase 6 — Spec 137 and 138 integration

Implement temporal and epistemic obligation profiles, calibration cohorts, champion/challenger governance, prediction/learning reflexes, and negative tests.

### Phase 7 — Vertical Intelligence Bundles

Implement bundle contracts, activation, pack composition, ontology decision rules, reflex overlays, composite Verticals, conflict/migration governance, conformance, and golden scenarios.

### Phase 8 — Complete product/client integration

Implement Work Rail/Mission Canvas portfolio and findings, Pi, CLI, TUI, menubar, generated clients, docs/runbooks, UIAI Engine Eval for browser behavior, release gates, accessibility, recovery, and full client parity.

Each phase is a merge gate. A phase cannot close while any of its feeder requirements are missing, partial, blocked, schema-only, shadow-only, unverified, or absent from integrated proof. Spec 144 cannot close until every phase and every active mandatory requirement is verified.

---

## 30. Acceptance criteria

Spec 144 is accepted only when all criteria below are verified and linked to ledger rows, tests, Evidence, and Receipts:

1. Normative source coverage reports zero unmapped and zero weakened clauses.
2. Every active requirement is reachable in the delivery DAG and has owner, implementation, proof, and closure destinations.
3. No forbidden placeholder or hidden deferral surface remains.
4. Every phase preserves scheduled-later requirements as open blocking rows.
5. Core definitions generate RDF, OWL 2 RL, SHACL, JSON Schema, OpenAPI, Rust, and TypeScript in parity.
6. Candidate, observation, inference, verifier, response, settlement, and quarantine graphs remain distinct.
7. Every required validation produces a durable receipt.
8. Unknown or invalid authoritative material fails closed and remains auditable.
9. Builder and Verifier lineages have distinct session/run/context identities.
10. Verifiers cannot acquire the Builder project writer lease.
11. Required Verifiers inspect immutable content-addressed snapshots.
12. The settled snapshot equals the verified snapshot.
13. Build claims and findings are typed, scoped, fresh, and Evidence-linked.
14. Unsupported criticism reaches governed disposition and cannot block indefinitely.
15. An open valid critical finding prevents settlement.
16. Absence of findings cannot substitute for complete mandatory coverage.
17. Obligations compile from every active requirement and semantic impact.
18. Plans are validated DAGs with zero uncovered mandatory obligations.
19. SHACL blocks ineligible, incomplete, under-assured, stale, or non-independent plans.
20. Same-model and cross-family verification receive truthful independence classifications.
21. Deterministic validators, agents, proof environments, external authorities, and operator review compose through one provider-neutral portfolio.
22. Dynamic rerouting responds to new impact, findings, failures, steering, and snapshot changes without losing obligations.
23. Majority vote cannot override obligation-specific veto or critical Evidence.
24. Spec 137 integration improves temporal interpretation without creating another clock authority.
25. Every high-consequence Vertical declares and proves temporal applicability.
26. Spec 138 integration improves information-set, source, resolution, scoring, calibration, causal, transfer, and promotion correctness without creating another epistemic authority.
27. Router/verifier calibration is scoped, evidence-backed, and champion/challenger governed.
28. A new Vertical activates through a versioned Vertical Intelligence Bundle.
29. Presentation-only Verticals do not add unnecessary ontology.
30. Semantic Verticals add required world, Evidence, temporal, prediction, verification, affordance, and authority extensions without duplication.
31. Composite Verticals preserve every active pack’s mandatory obligations.
32. Pack conflict blocks and provides migration/governance guidance.
33. Universal and Vertical reflexes operate through Spec 97 authority boundaries and executable runtime paths.
34. Workpoint remains immediate action authority, Silent Sessions remain execution substrate, and reducer remains settlement authority.
35. Every settled pair produces complete Evidence and a Spec 119-compatible Receipt.
36. Replay reconstructs every requirement, contract, attempt, snapshot, obligation, plan, assignment, context, finding, response, disposition, validation, reroute, settlement, and Receipt.
37. Fixed evaluations prove improvement over Builder-only and flat self-review baselines.
38. Security, privacy, performance, accessibility, restart, replay, migration, compatibility, generated-contract, and client-parity gates pass.
39. No requirement is marked verified by prose, enum, schema, mock, static UI, verdict string, process exit, model confidence, or subset proof.
40. `not_applicable_verified` and `conditional_inactive_verified` are supported by affirmative evidence and review triggers.
41. No mandatory clause is waived by variance.
42. Unknown impacts are zero before implementation promotion, release, and closure.
43. Every discovered in-scope closure requirement is added to the ledger and DAG.
44. Full conformance is the only status described as Spec 144 complete.

---

## 31. Closure blockers

Spec 144 MUST NOT close while any of the following is true:

- any normative clause is unmapped, duplicated with weaker meaning, or missing from the source-coverage artifact;
- any active mandatory ledger row is missing, partial, blocked, schema-only, shadow-only, degraded, disputed, implemented-unverified, or unknown-impact;
- any scheduled-later requirement is absent from the root DAG;
- any required client, Vertical, migration, proof, recovery, security, privacy, accessibility, performance, replay, or Receipt obligation is missing;
- any required operation exists only in prose, registry data, enum, schema, mock, static card, disabled flag, disabled test, unpublished branch, or backend/client silo;
- forbidden deferral language is used as a disposition;
- `not_applicable` or `conditional inactive` lacks affirmative evidence;
- a `SHOULD` variance lacks required operator approval, scope, expiry, Evidence, Receipt, and closure consequence;
- a mandatory requirement is waived without a specification amendment;
- an operator-removed row lacks preserved original text and amendment lineage;
- the spec hash changed without regenerated source coverage and ledger review;
- ontology objects/links remain generic unvalidated JSON on the strict path;
- a favorable string substitutes for semantic validation;
- inferred and asserted material is flattened;
- OWL, SHACL, Router, Verifier, or client can mint canonical authority;
- Builder and Verifier share one flat lineage;
- verification is self-review under another label;
- a Verifier can mutate Builder source;
- verification inspects a moving or stale target;
- findings lack exact target, Evidence, severity, method, or freshness;
- findings or verdicts bypass validation;
- a passing test, successful process, provider response, screenshot, or final message automatically means complete;
- one model verdict automatically means settled;
- Builder self-dismissal is possible;
- unsupported preference can block indefinitely;
- independence is inferred from prompt wording or session IDs alone;
- same-model verification is presented as fully independent;
- verifier/tool timeout or unavailability silently passes;
- mandatory obligations can be removed by Router optimization;
- unrelated pass votes override a valid critical finding;
- repeated rounds continue without Evidence delta;
- Workpoint, operator steering, Temporal Authority, primitive owners, or governing specs can be outranked by loop momentum;
- Spec 137 or 138 is forked by a Vertical;
- a workspace profile substitutes for a missing semantic pack;
- a Vertical requires hard-coded branching where registry composition suffices;
- active pack obligations can be suppressed by another pack or workspace;
- pack conflict is silently resolved;
- missing temporal/epistemic applicability is treated as non-applicable by default;
- completion is reducible to booleans or unstructured verdict strings;
- production implementation began before the activation gate;
- any accepted work is hidden outside the machine-readable closure system;
- any claim of full conformance lacks exact-SHA integrated live proof and a final closure Receipt.

---

## 32. Final architectural law

```text
A Vertical contributes domain meaning, Evidence rules, temporal and epistemic applicability,
verifier capabilities, reflexes, migrations, client projections, and proof obligations.

RDF records what is asserted and observed.
OWL determines bounded semantic implications.
SHACL determines whether declared operational constraints conform.
The Verification Router assigns every active obligation to eligible independent specialists or machines.
Secondary Cognition proposes, challenges, repairs, and learns.
Focusa’s daemon governs execution.
Focusa’s reducer alone records canonical settlement.
The complete feature ledger prevents accepted work from disappearing.
Nothing closes while anything required is omitted, deferred, hidden, partial, unverified, or unknown.
```