# Spec 144 — Focusa Semantic Integrity, RDF/OWL/SHACL Build↔Verify Routing, and Vertical Intelligence

**Status:** Draft / proposed / documentation-first; implementation not implied  
**Owner:** Focusa Core / Ontology / Secondary Cognition / Verification / Vertical Intelligence  
**Created:** 2026-07-26  
**Implementation start condition:** Spec 143’s locked release implementation and required proof must close, followed by an explicit operator activation decision for Spec 144 implementation. Documentation, research, schema design, non-mutating prototypes, and compatibility analysis may proceed earlier.  
**Primary relationship:** Extends and composes Specs 45–50, 61, 66, 70, 72, 74–79, 88, 90, 95, 97, 100, 107, 109, 113, 116, 119, 120, 125, 130, 131, 133, 135F, 136, 137, 138, 140, 141, 142, and 143.  
**Does not create:** a second reducer, ontology registry, event store, Workpoint authority, deadline authority, prediction authority, learning-promotion authority, permission system, receipt ledger, workflow engine, or agent framework.

---

## 0. One-line definition

Focusa SHALL compile its core-owned semantic registry into RDF, OWL, and SHACL contracts and SHALL govern consequential work through separate Builder cognition and ontology-routed domain-specific verification portfolios, using immutable evidence snapshots, typed findings, calibrated verifier capabilities, vertical-specific temporal and epistemic profiles, reflex overlays, and reducer-controlled settlement so that neither a fluent Builder, an agreeable Verifier, a passing process, nor an attractive projection can mint operational truth.

---

## 1. Executive directive

Focusa’s semantic and verification architecture SHALL preserve these distinctions:

```text
well-formed            ≠ semantically valid
semantically valid     ≠ empirically true
asserted               ≠ inferred
inferred               ≠ verified
verified               ≠ canonical
canonical              ≠ authorized
built                   ≠ verified
verified               ≠ settled
settled                 ≠ universally reusable learning
one verifier agreed    ≠ required verification coverage exists
no defect was detected ≠ completeness was proven
```

The governing architecture is:

```text
Operator direction + governing specification + ProjectIdentity + Workpoint revision
                                      │
                                      ▼
                              Semantic Work Contract
                                      │
                   ┌──────────────────┴──────────────────┐
                   ▼                                     ▼
          Builder cognition lineage             Verification obligation compiler
          separate context + writer             RDF + OWL + SHACL + policy
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

RDF represents semantic assertions and provenance.  
OWL defines bounded semantic meaning and entailment.  
SHACL validates closed operational graph requirements.  
Deterministic validators inspect mechanically decidable properties.  
Domain-specific verifier agents attempt to falsify claims.  
The daemon schedules and supervises work.  
The reducer remains the sole canonical state-transition authority.

---

## 2. Why this specification exists

Focusa already has strong but distributed primitives:

- typed ontology objects, relations, actions, status, provenance, evidence, and domain-pack concepts;
- candidate-versus-canonical separation;
- reducer-only canonical writes;
- bounded Secondary Cognition;
- adversarial closure-veracity verification;
- Workpoint continuity and immediate action authority;
- daemon-governed Work Loop and Silent Sessions;
- isolated workspaces and writer leases;
- Evidence, Receipts, replay, and settlement direction;
- Temporal Authority and grounded estimates under Spec 137;
- prediction, resolution, scoring, calibration, metacognitive learning, transfer, and epistemic governance under Spec 138;
- Reflex Primitives under Spec 97;
- Workspace View Profiles and domain packs under the Spec 135 series.

The remaining systemic gaps are:

1. Ontology rules are often expressed as prose, enums, strings, or generic JSON rather than one executable graph-level semantic conformance system.
2. Reducer acceptance proves governed state transition but does not alone prove semantic completeness, logical coherence, evidence sufficiency, or domain correctness.
3. One Builder or one general-purpose Verifier can carry correlated blind spots across implementation and review.
4. Separate context windows alone do not prove independence when models, tools, evidence, prompts, and hidden assumptions remain correlated.
5. A single Verifier is expected to understand security, migrations, UI, contracts, temporal semantics, evidence, prediction, calibration, causal reasoning, and vertical-specific authority equally well.
6. New Verticals can define workspace presentation and domain packs, but Focusa lacks one complete contract for adding domain verification obligations, verifier capabilities, temporal applicability, prediction applicability, and reflex overlays.
7. Verifier findings can themselves be unsupported, stale, outside scope, or based on invalid tests unless the Verifier is also governed and verified.
8. Spec 137 and Spec 138 can define maximal primitives yet still depend on generic model judgment unless domain-specific verification is executable.

This specification closes those gaps without creating a competing authority system.

---

## 3. Normative foundations and ownership

### 3.1 Existing primitive owners remain authoritative

| Concern | Primitive owner | Spec 144 responsibility |
|---|---|---|
| Canonical state transitions | Core reducer | Validate and route proposals; never replace reducer authority |
| Ontology object/link/action primitives | Specs 45–50 and 135F | Compile formal semantic artifacts and verification obligations |
| Domain-general cognition | Spec 61 | Reuse Mission, Task, Constraint, Risk, Blocker, Verification, and Evidence semantics |
| Affordance and execution reality | Spec 66 | Validate capability, permission, precondition, cost, reliability, and reversibility |
| Shared status/lifecycle/provenance | Spec 70 | Generate consistent semantic constraints |
| Agent identity/role/permissions | Spec 72 | Bind Builder, Verifier, Router, and settlement roles |
| Identity and reference resolution | Spec 74 | Prevent unsafe equivalence and identity merging |
| Projection and bounded context | Specs 75, 100 | Generate separate role-specific packets and preserve projection boundaries |
| Retention and decay | Spec 76 | Retain findings, attempts, calibration, and supersession correctly |
| Ontology governance/versioning | Spec 77 | Govern semantic artifacts, packs, migrations, and compatibility |
| Secondary Cognition | Spec 78 | Supply subordinate proposal, critique, reflection, and verification roles |
| Continuous execution | Spec 79 | Schedule Build↔Verify phases and continuation |
| Workpoint | Specs 88, 125, 143 | Remain immediate action authority and exact revision anchor |
| Reflexes | Spec 97 | Host universal and Vertical reflex overlays |
| Work-item closure | Spec 116 | Consume verification coverage and settlement evidence |
| Evidence and Receipts | Spec 119 | Preserve full execution, finding, validation, and settlement lineage |
| Adversarial spec work | Spec 120 | Reuse verification portfolios while retaining operator approval |
| Silent Sessions | Spec 133 | Execute isolated Builder and Verifier sessions |
| Proposal-to-settlement | Spec 136 | Consume Build↔Verify and semantic validation as lifecycle phases |
| Temporal Authority | Spec 137 | Retain clock, deadline, urgency, and temporal semantics ownership |
| Prediction and learning | Spec 138 | Retain forecast, resolution, scoring, calibration, learning, transfer, and self-model ownership |
| Runtime constitution | Spec 140 | Compile role-specific instructions under one authority graph |
| Release gates and trace matrices | Specs 141–143 | Require contracts, proof, parity, and no silent deferral |

### 3.2 No authority by fluency

Model quality, role naming, confidence, repeated success, long runtime, access to broad context, or domain vocabulary SHALL NOT implicitly grant:

- canonical mutation authority;
- permission escalation;
- deadline authority;
- outcome-resolution authority;
- scoring authority;
- settlement authority;
- learning-promotion authority;
- ontology-authoring authority;
- routing-policy self-modification authority.

---

## 4. Standards profile and semantic compilation

### 4.1 Initial mandatory profile

```text
RDF 1.1 abstract data model
JSON-LD 1.1 interchange
OWL 2 RL bounded reasoning profile
SHACL Core validation
SPARQL 1.1 for bounded internal queries where required
PROV-O interoperability mapping
JSON Schema 2020-12 structural contracts
OpenAPI 3.0.3 HTTP operation contracts
```

Draft or experimental standards may be evaluated behind explicit compatibility flags but SHALL NOT become required canonical persistence or cross-client dependencies without a governed amendment.

### 4.2 Single-source compilation law

The core-owned Focusa registry remains authoritative.

```text
Core semantic registry
       │
       ├── Rust types and registries
       ├── JSON Schema 2020-12
       ├── OpenAPI 3.0.3
       ├── generated TypeScript contracts
       ├── RDF vocabulary
       ├── OWL modules
       ├── SHACL shape bundles
       ├── operation and tool bindings
       └── conformance fixtures
```

Hand-maintained duplicate definitions are forbidden where deterministic generation can represent the contract.

Build or release gates SHALL fail when generated artifacts disagree about:

- semantic identifiers;
- required properties;
- datatypes;
- cardinality;
- source and target classes;
- lifecycle transitions;
- status vocabulary;
- action targets and preconditions;
- evidence requirements;
- domain-pack ownership;
- verification obligation triggers;
- deprecation or replacement;
- compatibility version.

### 4.3 OWL reasoning and validation are not equivalent

OWL uses open-world semantics. Missing information does not automatically mean false, and logical consistency does not prove operational completeness.

SHACL supplies closed validation for questions such as:

- Is every required property present?
- Did every required verification obligation receive coverage?
- Are the correct evidence classes attached?
- Did the Verifier inspect the exact final snapshot?
- Is an open critical finding unresolved?
- Is the assigned Verifier eligible and sufficiently independent?

Therefore:

```text
OWL may derive candidate implications.
SHACL may establish conformance to a declared shape bundle.
Neither may independently create canonical truth or settlement.
```

---

## 5. Semantic identity and graph partitioning

### 5.1 Stable semantic identifiers

Every V2 semantic definition, obligation, capability, finding, validation receipt, snapshot, and settlement record SHALL have a stable semantic IRI or deterministic internal ID mapping one-to-one to an IRI.

Illustrative form:

```text
urn:focusa:type:core:task:1
urn:focusa:link:core:depends_on:1
urn:focusa:action:core:complete_task:1
urn:focusa:pack:software:1
urn:focusa:verification:security:authentication_regression:1
urn:focusa:verifier:security_code_review:1
urn:focusa:project:{project_fingerprint}:object:{semantic_id}
urn:focusa:pair:{semantic_pair_id}
urn:focusa:snapshot:{snapshot_id}
urn:focusa:validation:{validation_id}
```

Identifiers SHALL NOT expose raw credentials, secrets, private corpus contents, or unsafe local paths.

### 5.2 Required named graph partitions

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

#### Registry graph

Contains classes, properties, actions, status, pack metadata, policy references, compatibility, and deprecation declarations. It contains definitions, not project facts.

#### Contract graph

Contains the exact operator direction, governing specification, Workpoint revision, acceptance criteria, policies, role assignments, active domain packs, and verification requirements.

#### Builder assertion graph

Contains explicit Builder claims. Builder claims SHALL NOT be silently promoted to runtime observations.

#### Observation graph

Contains deterministic and runtime-observed facts such as file hashes, diffs, tests, browser proof, provider reconciliation, timestamps, and tool results.

#### Inference graph

Contains bounded OWL-derived assertions with rule/axiom, reasoner version, input graph hash, timestamp, and invalidation metadata.

#### Verifier graph

Contains findings, objections, evidence, reproduction, confidence, uncertainty, and requested dispositions.

#### Settlement graph

Contains the final protocol decision and references supporting it. It does not replace canonical event history.

#### Quarantine graph

Contains malformed, inconsistent, malicious, unsupported, migration-blocked, or policy-prohibited material. Quarantined material remains auditable but SHALL NOT enter ordinary action selection or context.

### 5.3 Epistemic classes remain distinct

Every semantic projection SHALL distinguish at least:

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

Canonical asserted facts, canonical inferred facts, verified candidates, unverified candidates, contradicted assertions, stale assertions, and legacy assumptions SHALL NOT be flattened into one generic `fact` category.

---

## 6. Formal Semantic Integrity validation

### 6.1 Validation profiles

Focusa SHALL provide purpose-specific SHACL bundles.

#### Intake profile

Validates whether material may safely enter candidate state:

- known definition IDs;
- payload and datatype correctness;
- namespace ownership;
- scope;
- resource bounds;
- prohibited properties;
- provenance;
- version compatibility.

Failure produces an invalid or quarantined candidate with diagnostics; it does not silently erase input.

#### Promotion profile

Validates whether a candidate may proceed to promotion consideration:

- required properties and links;
- source/target class compatibility;
- cardinality;
- status and lifecycle legality;
- evidence and freshness;
- identity resolution;
- unresolved contradiction;
- operator and policy references;
- pack compatibility;
- candidate/canonical separation.

Conformance is necessary but not sufficient for promotion.

#### Action preflight profile

Validates actor, role, permission, scope, target, preconditions, constraints, blockers, inputs, reversibility, idempotency, side effects, evidence expectations, timeout, retry, and tool mapping.

#### Verification-plan profile

Validates obligations, capability eligibility, independence, assignment dependencies, snapshot binding, coverage, assurance tier, required tools, and data-access posture.

#### Settlement profile

Validates complete mandatory coverage, final snapshot identity, evidence sufficiency, no unresolved critical findings, required operator approval, receipt readiness, and reducer-only canonical transition.

#### Domain-pack profile

Validates manifest integrity, namespace, shared-interface conformance, OWL profile, shape validity, lifecycle mapping, migration, signatures/trusted origin, and resource limits.

#### Migration and replay profile

Validates historical version compatibility, graph identity, evidence preservation, unknown-event handling, V1 projection equivalence, and post-migration conformance.

### 6.2 Semantic validation receipt

```yaml
schema: focusa.semantic_validation_receipt.v1

validation_id:
validation_purpose:
target_ref:
semantic_pair_id:
candidate_id:
project_root:
continuity_id:
workpoint_ref:

registry_version:
domain_pack_versions: []
shape_bundle_id:
shape_bundle_hash:
data_graph_hash:
inference_graph_hash:

inference_profile: none | rdfs | owl2_rl
reasoner:
  implementation:
  version:
validator:
  implementation:
  version:

conforms:
severity_counts:
  info:
  warning:
  violation:
  fatal:

results:
  - result_id:
    shape_id:
    focus_node:
    result_path:
    constraint_component:
    severity:
    code:
    message:
    expected:
    actual:
    related_semantic_refs: []
    evidence_refs: []
    suggested_repairs: []

policy_refs: []
evidence_refs: []
created_at:
expires_at:
receipt_hash:
```

A string such as `passed`, `accepted`, `verified`, `approved`, or `success` SHALL NOT substitute for this receipt where formal validation is required.

---

## 7. Semantic Work Contract and execution pair

### 7.1 `SemanticWorkContract`

```yaml
schema: focusa.semantic_work_contract.v1

work_contract_id:
project_identity_ref:
project_root:
continuity_id:
trajectory_ref:
workpoint_ref:
operator_direction_ref:
governing_spec_refs: []
acceptance_criterion_refs: []

active_domain_pack_refs: []
registry_version:
shape_bundle_refs: []

allowed_action_type_ids: []
allowed_scope_refs: []
constraint_refs: []
risk_refs: []
required_evidence_kinds: []

builder_policy_ref:
verification_policy_ref:
independence_policy_ref:
disclosure_policy_ref:
settlement_policy_ref:
resource_policy_ref:

created_at:
contract_hash:
status: draft | validating | frozen | superseded | cancelled
```

The Evaluation Contract and acceptance criteria freeze before building. Builder and Verifier cognition MAY propose amendments only through the governing specification or operator path.

### 7.2 `SemanticExecutionPair`

The historical singular `VerifierAssignment` model is replaced by one Builder lineage and a Verification Portfolio.

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
receipt_refs: []
state:
created_at:
updated_at:
```

The pair coordinates Silent Sessions but does not replace their canonical session, run, stream, lease, checkpoint, or receipt identities.

---

## 8. Builder cognition lineage

### 8.1 Builder authority

The Builder MAY:

- inspect authorized context;
- mutate only its assigned workspace under a valid writer lease;
- invoke permitted implementation tools;
- run tests and diagnostics;
- create Evidence;
- submit typed Build Claims;
- respond to findings;
- request clarification, rerouting, or policy escalation.

The Builder SHALL NOT:

- alter the frozen Work Contract, shapes, scoring policy, or acceptance criteria;
- modify Verifier findings;
- inspect hidden verifier material unless disclosure permits;
- settle completion;
- grant itself authority;
- issue its own completion Receipt.

### 8.2 `BuilderContextPacket`

Contains:

- exact current operator direction;
- ProjectIdentity, Trajectory, Workpoint and revision;
- authoritative specifications and criteria;
- active constraints, decisions, blockers, and risks;
- implementation-relevant ontology slice;
- accepted prior findings;
- authorized actions/tools;
- workspace and writer-lease bindings;
- do-not-drift boundaries;
- temporal and resource posture.

It excludes hidden verifier tests, irrelevant verifier history, and settlement internals not required for execution.

### 8.3 `BuildAttempt`

```yaml
schema: focusa.build_attempt.v1

build_attempt_id:
semantic_pair_id:
builder_session_id:
builder_run_id:
context_packet_ref:
work_contract_ref:
source_snapshot_ref:
result_snapshot_ref:
claim_refs: []
changed_semantic_refs: []
changed_artifact_refs: []
evidence_refs: []
test_refs: []
known_blocker_refs: []
started_at:
claimed_at:
status: active | claimed | challenged | repairing | superseded | failed | cancelled
```

---

## 9. Verification obligation compilation

### 9.1 Core law

```text
Do not route a whole task to one Verifier.

Compile the Build Attempt into verification obligations,
then route each obligation to an eligible verification capability.
```

### 9.2 Compilation inputs

```text
operator direction
+ Workpoint revision
+ governing specifications and acceptance criteria
+ Builder action types
+ changed RDF objects and links
+ affected domain packs
+ artifact classes
+ permission and authority boundaries
+ risk and reversibility classes
+ evidence requirements
+ Spec 137 temporal applicability
+ Spec 138 epistemic applicability
+ OWL inferences
+ SHACL obligation-trigger shapes
→ Verification Obligation Graph
```

Registered deterministic requirements SHALL be emitted without model interpretation. Model cognition MAY propose additional obligations but SHALL NOT remove registered obligations.

### 9.3 `VerificationObligation`

```yaml
schema: focusa.verification_obligation.v1

obligation_id:
obligation_type_id:
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
status: proposed | required | assigned | active | satisfied | blocked | waived_by_authorized_variance | superseded
```

### 9.4 Example obligation decomposition

A change touching authentication middleware, credential schema, generated UI, and deployment configuration may generate:

```text
security.authentication_regression
security.authorization_boundary
schema.migration_reversibility
api.backward_compatibility
ui.generated_action_binding
runtime.deployment_configuration_safety
temporal.credential_expiry_semantics
evidence.completion_sufficiency
scope.authority_conformance
```

These obligations SHALL NOT be collapsed into one generic `software_review` verdict.

---

## 10. Domain-Specific Verification Router

### 10.1 One-line definition

The Domain-Specific Verification Router compiles and resolves each Verification Obligation Graph into the minimum policy-compliant portfolio of deterministic validators, formal semantic validators, specialist agents, proof environments, external authorities, and operator review required to support settlement.

### 10.2 Verification provider classes

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

A provider may satisfy multiple compatible obligations but gains no settlement authority from doing so.

### 10.3 `VerifierCapabilityProfile`

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
```

A domain-specific verifier is therefore not merely a model name. It is:

```text
role profile
+ supported domains and obligations
+ tools and proof environments
+ context and disclosure policy
+ model-selection policy
+ workspace permissions
+ independence posture
+ calibration history
+ cost and latency profile
```

### 10.4 Candidate discovery and eligibility

OWL MAY classify verifier capabilities and obligation subclasses. Policy and SHACL SHALL determine assignment eligibility.

Candidate filtering SHALL consider:

- domain-pack compatibility;
- obligation and artifact support;
- required tool availability;
- data-access permission;
- assurance tier;
- model-family independence;
- calibration and reliability;
- runtime availability;
- cost, latency, token, and concurrency budgets;
- workspace and sandbox capability;
- source and evidence independence.

### 10.5 Verification portfolio construction

The router SHALL produce a portfolio DAG, not a flat unordered list.

```text
snapshot integrity
        │
        ▼
structural and semantic validation
        │
        ├───────────────┬────────────────┐
        ▼               ▼                ▼
contract review    security review   temporal review
        │               │                │
        ▼               ▼                ▼
integration tests adversarial tests deadline/freshness checks
        └───────────────┼────────────────┘
                        ▼
                evidence sufficiency
                        ▼
                settlement validation
```

Dependencies SHALL prevent downstream verifiers from relying on an untrusted snapshot, unresolved identity, or invalid test environment.

### 10.6 `VerificationPlan`

```yaml
schema: focusa.verification_plan.v1

verification_plan_id:
semantic_pair_id:
work_contract_ref:
workpoint_ref:
build_attempt_ref:
verification_snapshot_ref:
obligation_graph_ref:

assignments:
  - assignment_id:
    obligation_ids: []
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

### 10.7 Router authority

The router MAY:

- compile and propose obligations;
- discover eligible capabilities;
- construct and revise a plan;
- request subordinate sessions;
- add coverage when new findings reveal impact;
- suppress optional redundant checks;
- escalate assurance;
- stop routing after policy-complete coverage.

The router SHALL NOT:

- remove mandatory obligations;
- weaken assurance;
- alter criteria;
- grant permissions;
- waive independence;
- treat unavailable verification as passed;
- resolve critical disagreement;
- settle completion;
- self-promote routing-policy learning.

---

## 11. Verifier cognition lineages

### 11.1 Separate contexts

Each verifier assignment receives an independently generated `VerifierContextPacket` containing only the material needed for its obligations:

- exact operator direction hash/text where policy permits;
- Workpoint and revision;
- frozen Evaluation Contract;
- assigned obligations and criteria;
- immutable source, diff, semantic, evidence, test, and temporal snapshots;
- applicable ontology modules and shapes;
- threat model, risk classes, and verification strategy;
- approved tools and workspace bindings;
- disclosure policy.

By default it excludes:

- Builder chain-of-thought;
- full Builder transcript;
- Builder confidence rhetoric;
- irrelevant attempts;
- mutable source state;
- material outside verification scope.

### 11.2 Workspace asymmetry

A verifier SHALL NOT acquire the Builder project writer lease.

Allowed verifier posture:

```text
source workspace: read_only_shared or immutable snapshot
verification sandbox: writable when tests/probes require it
project source mutation: forbidden
repair output: proposal or artifact only
```

### 11.3 Verifier continuation modes

```text
fresh
carry_open_findings_only
carry_verifier_summary
full_verifier_lineage
```

Default:

```text
carry_open_findings_only
```

### 11.4 Disclosure modes

```text
blind_behavioral
gray_box
white_box
```

Hidden-test disclosure policies:

```text
full
minimal_reproduction
finding_summary_only
withheld_until_settlement
```

The policy SHALL preserve repair fairness without encouraging overfitting.

---

## 12. Verification snapshot integrity

### 12.1 Frozen snapshot law

Every required verifier SHALL inspect the same immutable snapshot or a declared derivative whose lineage and content hash are explicit.

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

Any source, diff, evidence, criterion, domain-pack, or Workpoint revision change SHALL invalidate or supersede affected verification and settlement readiness.

### 12.2 Final snapshot identity

The snapshot settled SHALL equal the snapshot verified. A repaired attempt requires a new snapshot and re-execution of every invalidated obligation.

---

## 13. Findings, verdicts, and verifying the Verifier

### 13.1 `VerificationFinding`

```yaml
schema: focusa.verification_finding.v1

finding_id:
semantic_pair_id:
verification_round_id:
verifier_assignment_ref:
obligation_ref:

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

created_at:
status: open | responded | confirmed | partially_confirmed | reproduced | not_reproduced | unsupported | withdrawn | superseded | accepted_risk | operator_resolved | inconclusive
```

A critical finding requires stronger evidence and reproduction posture than a warning.

### 13.2 Findings are claims

The Verifier is not presumed correct. Its output SHALL pass structural, semantic, scope, evidence, and freshness validation.

A finding may be rejected or narrowed when:

- it targets the wrong criterion;
- it is outside scope;
- it lacks evidence;
- reproduction fails;
- it inspects stale state;
- it mistakes preference for requirement;
- it contradicts the governing specification;
- its test is invalid;
- its graph fails conformance;
- counterevidence disproves it.

### 13.3 Builder responses

The Builder MAY submit a `FindingResponse` with repair mapping, dispute, counterevidence, or request for clarification. The Builder SHALL NOT unilaterally dismiss a finding.

### 13.4 Verdict requirements

A positive verdict SHALL NOT be valid when:

- required criteria were not inspected;
- evidence is unknown or insufficient;
- a required provider failed or was unavailable;
- an open critical finding exists;
- snapshot identity is stale;
- verifier independence is below policy;
- the verdict graph or finding graph fails SHACL.

---

## 14. Independence and assurance

### 14.1 `VerifierIndependenceProfile`

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

Separate session IDs or prompts alone SHALL NOT prove independence.

### 14.2 Assurance tiers

#### Tier 0 — deterministic conformance

For harmless and mechanically decidable operations. No LLM verifier required.

#### Tier 1 — separate-context verification

For low-risk reversible work. Same model permitted, separate context and no writer lease required.

#### Tier 2 — cross-model verification

Default for meaningful code, architecture, data, integration, specification, prediction, and temporal work. Cross-family verification is preferred or required by policy.

#### Tier 3 — multi-aspect verification

For security, migration, compliance, release, destructive operations, high-consequence predictions, temporal commitments, and public proof. Multiple obligation-specific verifiers and deterministic proof are required.

#### Tier 4 — operator or external authority settlement

For inherently judgmental, disputed, regulated, legal, safety-critical, or unresolved cases.

### 14.3 No majority-vote settlement

A valid critical security, legal, temporal, evidence, or authority finding SHALL NOT be outvoted by unrelated pass verdicts.

Settlement uses:

```text
obligation coverage
+ severity
+ evidence sufficiency
+ reproduction
+ verifier eligibility
+ independence
+ calibration
+ contradiction state
+ veto and escalation policy
+ SHACL settlement conformance
```

---

## 15. Build↔Verify lifecycle

### 15.1 Main path

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

### 15.2 Challenge and repair path

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

### 15.3 Exceptional states

```text
inconclusive
verification_blocked
operator_required
budget_exhausted
oscillation_detected
scope_invalidated
snapshot_invalidated
routing_conflicted
superseded
cancelled
failed
```

`verification_passed` SHALL NOT equal `settled`.

### 15.4 Semantic checkpoints

Deep verification SHOULD run at meaningful boundaries:

- implementation-plan formation for consequential work;
- before risky mutation;
- after major patch/artifact revision;
- unexpected deterministic failure;
- material Evidence change;
- Workpoint satisfaction claim;
- WorkItem closure;
- release, migration, deployment, or irreversible action;
- material operator steering;
- completion claim.

Lightweight Secondary Cognition sentinels may inspect turn and tool outcomes continuously but emit observations and candidate findings only.

---

## 16. Dynamic rerouting

### 16.1 Rerouting triggers

```text
new semantic impact inferred
new critical finding
verification environment failure
verifier unavailability
tool capability mismatch
unexpected artifact class
scope or authority mismatch
finding disagreement
evidence insufficiency
repeated unsupported findings
operator steering
snapshot supersession
pack or policy revision
```

### 16.2 `VerificationRerouteRecord`

Every reroute SHALL preserve:

- prior plan;
- triggering Evidence;
- added/removed assignments;
- unresolved obligations;
- policy and reason;
- cost/latency impact;
- validation result;
- supersession lineage.

Mandatory obligations SHALL NOT be silently removed.

### 16.3 Oscillation control

The loop SHALL stop or escalate when:

- the same dispute repeats without material evidence delta;
- repeated repairs fail the same criterion;
- Verifiers produce repeated unsupported findings;
- the Builder optimizes narrowly for hidden checks while broader criteria regress;
- budget or deadline policy requires operator choice;
- governing specifications conflict.

`inconclusive` is a valid truthful outcome.

---

## 17. Spec 137 Temporal Authority integration

### 17.1 Material improvement boundary

Spec 144 materially improves the effective accuracy of Spec 137 by validating temporal interpretation, applicability, evidence, and domain policy. It does not improve the physical accuracy of clocks, synchronization infrastructure, provider timestamps, or networks.

It improves the probability that Focusa:

- selects the correct clock domain and precision profile;
- detects unsupported precision;
- rejects stale calibration;
- applies the correct calendar and timezone semantics;
- validates deadline completion effect;
- preserves uncertainty across boundaries;
- refuses ungrounded estimates;
- identifies breach and remaining opportunity correctly;
- routes high-consequence timing claims to qualified temporal specialists.

### 17.2 Temporal verification obligations

```text
temporal.trusted_clock_integrity
temporal.clock_source_independence
temporal.calendar_source_authority
temporal.civil_time_resolution
temporal.deadline_contract_completeness
temporal.readiness_margin_grounding
temporal.evidence_freshness
temporal.critical_path_duration_grounding
temporal.boundary_uncertainty_classification
temporal.deadline_breach_classification
temporal.overdue_opportunity_assessment
temporal.settlement_time_integrity
```

### 17.3 Temporal verifier capabilities

```text
ClockIntegrityValidator
CalendarAuthorityVerifier
CivilTimeVerifier
DeadlineContractVerifier
EstimateGroundingVerifier
CriticalPathVerifier
EvidenceFreshnessValidator
TemporalSettlementVerifier
DomainTemporalVerifier
```

### 17.4 `DomainTemporalApplicabilityProfile`

Every high-consequence Vertical SHALL declare its applicable temporal semantics rather than fork Spec 137.

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

Markets remains a strict exemplar, not a separate temporal runtime. Legal filings, healthcare workflows, infrastructure incidents, credentials, release windows, industrial control, emergency response, and custom high-consequence packs SHALL declare their own applicability.

### 17.5 Temporal reflexes

Universal or Vertical overlays may register:

```text
verify_temporal_authority_freshness
resolve_civil_time_before_commit
reject_unsupported_precision
check_domain_calendar_version
recompute_deadline_posture_on_clock_revision
block_unverified_deadline_claim
validate_remaining_opportunity_after_breach
freeze_unrelated_optional_work_in_overdue_mode
route_temporal_disagreement_to_specialist
```

---

## 18. Spec 138 Prediction and Epistemic Governance integration

### 18.1 Material improvement boundary

Spec 144 materially improves Spec 138 by distributing epistemic checks across obligation-specific capabilities rather than concentrating them in one model.

The Router SHALL support obligations for:

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

### 18.2 Epistemic verifier capabilities

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

### 18.3 Calibration extensions

Spec 138 calibration SHALL be extendable by:

- verifier role;
- capability profile;
- obligation class;
- verification strategy;
- model family;
- artifact class;
- Vertical and domain pack;
- risk class;
- assurance tier;
- original versus transfer context.

This enables scoped evidence-backed answers to:

- Which verifier catches schema incompatibility reliably?
- Which verifier over-reports security findings?
- Which verifier misses temporal ambiguity?
- Which causal verifier mistakes association for causation?
- Which verifier transfers well from Software to Legal or Markets?
- When does deterministic proof outperform model critique?

### 18.4 Learning promotion portfolios

A learning candidate may activate separate obligations for Evidence, outcome authority, experiment validity, confounders, controls, transfer, regression, and governance.

A fluent causal narrative SHALL NOT become promoted learning without the required portfolio.

### 18.5 Unknown and abstention

When ground truth, samples, target definitions, source independence, resolution authority, or applicability are inadequate, the correct result may be:

```text
unknown
insufficient_evidence
not_resolvable_yet
abstain
operator_judgment_required
experimental_only
```

More verification SHALL NOT manufacture certainty.

### 18.6 Champion/challenger routing policy

Routing and verifier-selection policies SHALL use Spec 138 champion/challenger governance:

```text
immutable current routing policy
→ champion

candidate policy
→ challenger

fixed replay and live shadow evaluation
→ evidence-backed comparison

operator/governance promotion
→ new champion
```

The router SHALL NOT mutate its live canonical policy from its own outcomes.

---

## 19. Vertical Intelligence Bundles

### 19.1 One-line definition

A new Vertical is added through a versioned `VerticalIntelligenceBundle`, not by forking Focusa, hard-coding a router branch, or treating a visual theme as semantic capability.

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
status:
```

### 19.2 Bundle layers

#### Workspace View Profile

Controls layout, terminology, visual grammar, artifact renderers, and emphasis. It SHALL NOT redefine semantic truth.

#### Domain Pack

Defines entities, relations, actions, status, lifecycle, evidence, identity, authority, and slice policies.

#### Verification Pack

Defines obligation classes, verification dimensions, eligible verifier profiles, tools, evidence, assurance, independence, veto, escalation, and finding shapes.

#### Reflex Overlay

Defines small typed routines activated by the domain.

#### Temporal applicability overlay

Extends Spec 137 only where domain-specific calendar, deadline, precision, freshness, latency, or completion effects exist.

#### Prediction and learning overlay

Extends Spec 138 only where domain-specific targets, indicators, outcomes, scorers, regimes, resolution authorities, or transfer boundaries exist.

### 19.3 When a new ontology is not required

No new domain ontology is required when the Vertical is primarily:

- presentation, terminology, or layout;
- a combination of existing domain packs;
- a new artifact renderer;
- a verification policy over existing semantic classes;
- a connector producing existing Evidence classes;
- a user or project workspace profile.

### 19.4 When a new domain ontology is required

A new ontology module is required when omitting the distinction could cause Focusa to:

- take the wrong action;
- trust the wrong Evidence;
- misunderstand identity or completion;
- miss a deadline or calendar boundary;
- resolve an outcome incorrectly;
- violate domain authority;
- generalize learning beyond its valid scope.

Indicators include unique:

1. entities;
2. relationships;
3. actions/preconditions;
4. Evidence standards;
5. lifecycle states;
6. identity semantics;
7. temporal semantics;
8. prediction targets/outcomes/scorers/regimes;
9. permission or authority boundaries.

### 19.5 Shared versus Vertical ontology modules

One shared Verification Fabric Ontology SHALL be added once:

```text
VerificationDomain
VerificationDimension
VerificationObligation
VerificationRequirement
VerifierCapability
VerifierCapabilityProfile
VerifierAssignment
VerificationPlan
VerificationPlanDAG
VerificationSnapshot
VerificationFinding
FindingDisposition
VerificationCoverage
IndependenceProfile
VerificationReroute
SettlementEvaluation
```

Verticals may add:

```text
Domain World Ontology
Domain Evidence Ontology
Domain Temporal Applicability Profile
Domain Prediction Profile
Domain Verification Ontology
Domain Affordance and Authority Overlay
```

They SHALL extend rather than duplicate shared primitives.

### 19.6 Cross-Vertical composition

Projects MAY activate multiple Vertical bundles.

Example:

```text
focusa.software
+ focusa.legal
+ focusa.security
+ focusa.research
```

A licensing-related source change may generate software compatibility, legal compliance, supply-chain integrity, source-authority, and Evidence-sufficiency obligations.

No selected workspace may silently suppress another active pack’s mandatory obligations.

Pack conflict SHALL create explicit `verification_plan_conflicted`, migration, governance, or operator review rather than convenient winner selection.

---

## 20. Vertical activation flow

```text
1. Register Vertical Intelligence Bundle
2. Validate namespace, identity, version, signature/trusted origin
3. Resolve required shared and domain packs
4. Compile JSON Schema, OpenAPI, RDF, OWL, and SHACL artifacts
5. Validate OWL profile and consistency
6. Validate SHACL shape integrity
7. Register verification obligations and capabilities
8. Register reflex overlays
9. Declare Spec 137 applicability
10. Declare Spec 138 prediction/learning applicability
11. Preview migration and compatibility
12. Run conformance and golden scenarios
13. Record operator/project activation approval
14. Resolve project semantic registry
15. Generate bounded workspace and agent projections
```

The runtime SHALL route from semantic impact:

```text
active packs
+ changed objects/relations/actions
+ risk and authority
+ acceptance criteria
→ obligations
→ eligible capabilities
→ verification portfolio
```

Hard-coded branches such as `if vertical == legal` are forbidden when registry and policy composition can represent the behavior.

---

## 21. Reflex architecture

### 21.1 Reflex contract

Reflexes preserve Spec 97’s shape:

```text
Trigger
→ Context inputs
→ Reflex action
→ Evidence output
→ Escalation boundary
```

They remain small, typed, inspectable, operator-governed, context-fed, authority-bounded, degradable, and composable.

### 21.2 Shared verification reflexes

#### Scope and obligation

```text
detect_verification_domain_impact
compile_verification_obligations
detect_uncovered_mandatory_obligation
detect_cross_domain_verification_conflict
```

#### Snapshot and continuity

```text
freeze_verification_snapshot
invalidate_verification_after_snapshot_change
resume_open_verification_obligations
supersede_stale_verifier_context
```

#### Routing

```text
route_verification_portfolio
enforce_verifier_capability_eligibility
enforce_verifier_independence
escalate_assurance_tier
reroute_on_new_finding
reroute_on_verifier_failure
```

#### Evidence and settlement

```text
require_evidence_for_verifier_finding
reject_unsupported_critical_finding
block_settlement_on_open_critical_finding
block_settlement_on_uncovered_obligation
verify_final_snapshot_matches_verified_snapshot
```

#### Recovery

```text
retry_verifier_with_bounded_fallback
replace_unavailable_verifier
route_disagreement_to_arbiter
escalate_inconclusive_verification
detect_build_verify_oscillation
```

#### Learning

```text
record_verifier_prediction
evaluate_verifier_after_settlement
detect_verifier_false_positive_pattern
detect_verifier_false_negative_pattern
detect_negative_verifier_transfer
propose_verifier_policy_adjustment
```

These map to existing Spec 97 families rather than requiring one opaque mega-reflex.

### 21.3 Vertical reflex overlay template

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

### 21.4 Example Legal reflexes

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

### 21.5 Example Markets reflexes

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

### 21.6 Example Research reflexes

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

## 22. Settlement protocol

### 22.1 Settlement requirements

Settlement SHALL evaluate:

- Work Contract and exact Workpoint revision;
- final immutable snapshot;
- structural and semantic validation receipts;
- obligation coverage;
- deterministic check results;
- findings and dispositions;
- Evidence sufficiency;
- verifier eligibility, independence, and calibration posture;
- temporal and epistemic applicability;
- unresolved blockers and contradictions;
- operator or external approval where required;
- Receipt readiness.

### 22.2 Settlement outcomes

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

### 22.3 Fail-closed conditions

Settlement SHALL block when:

- a mandatory obligation is uncovered;
- a required verifier is unavailable;
- a critical finding is open;
- Evidence is insufficient or stale;
- snapshot identity differs;
- independence is below policy;
- domain-pack conflict is unresolved;
- temporal or outcome authority is unavailable;
- formal validation cannot complete;
- the operation requires operator approval.

An explicit governed variance may waive only requirements whose primitive owner and policy allow waiver. The variance remains visible and Receipt-backed.

---

## 23. API, CLI, Operation Registry, and client contracts

### 23.1 Required operation families

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

Exact routes and commands SHALL follow the Operation Registry and generated-contract architecture.

### 23.2 Client behavior

Pi, PWA, Mission Canvas, TUI, menubar, UIAI Engine Cockpit, CLI, and generated clients SHALL render shared bounded read models. They SHALL NOT recompute routing, validation, settlement, or ontology authority locally.

### 23.3 Required status truth

Clients SHALL distinguish:

```text
operational
read_only
schema_only
pack_missing
migration_required
verification_required
verification_blocked
operator_required
unsupported_future_definition
writer_blocked
degraded
stale
conflicted
quarantined
```

---

## 24. Events and durable records

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
verification.reasoning_completed

verifier.session_bound
verifier.context_bound
verifier.round_started
verifier.finding_created
verifier.finding_revised
verifier.finding_withdrawn
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

Every event SHALL carry applicable pair, session, run, Workpoint, contract, snapshot, correlation, semantic registry, and schema-version references.

---

## 25. Persistence, replay, and migration

### 25.1 Persistence

Use existing Focusa SQLite canonical event and snapshot infrastructure. Raw graph documents, transcripts, large findings, and proof artifacts remain behind Reference Store handles where appropriate.

### 25.2 Replay

Replay SHALL reconstruct:

- Work Contract;
- every Build Attempt;
- snapshot lineage;
- obligation graph and plans;
- assignments and contexts;
- findings and responses;
- dispositions;
- validation receipts;
- reroutes;
- settlement decision;
- Receipt linkage.

### 25.3 Legacy migration

Current completion representations such as:

```text
adversarial_verifier_verdict: optional string
acceptance_verified: bool
adversarial_verified: bool
```

remain readable as `legacy_advisory_verdict` or `legacy_assumed_verification`. They SHALL NOT silently become strict independent verification.

### 25.4 Unknown future semantics

Unknown non-authoritative graph material may be preserved and skipped with diagnostics. Unknown authoritative definitions/events SHALL block canonical replay or mutation until a compatible runtime is available.

---

## 26. Security and privacy

Required controls:

- trusted namespaces and pack origins;
- signed or governed distributed packs;
- no remote ontology imports during hot execution;
- no arbitrary SHACL-SPARQL from untrusted packs in the initial profile;
- query, node, edge, depth, time, memory, and result bounds;
- malicious recursive-shape detection;
- reasoner denial-of-service tests;
- ontology poisoning tests;
- prompt-injection and source-poisoning handling;
- no Verifier project writer lease;
- project/continuity isolation;
- cross-project IRI collision prevention;
- Evidence access checks;
- data-class-aware verifier eligibility;
- redaction-aware JSON-LD and public export;
- validation-receipt tamper detection;
- identity-merge escalation;
- no agent-authored canonical `owl:sameAs`;
- secrets excluded from graph and context projections.

Identity similarity SHALL create a resolution candidate, not strict OWL equivalence.

---

## 27. Performance and resource laws

1. Whole-world reasoning SHALL NOT run synchronously for every turn.
2. Registry, OWL, and shape bundles SHALL be cached by content hash.
3. Candidate validation SHALL operate on the candidate delta and affected semantic neighborhood.
4. Canonical inference SHOULD be incrementally materialized where practical.
5. Every traversal and validation has explicit bounds.
6. Low-memory mode may reduce explanation detail but not authority, hashes, violation counts, or critical findings.
7. Validation timeout is not conformance.
8. A timed-out required verification blocks or invokes an explicit governed fallback.
9. Verifier portfolio construction SHALL prefer deterministic checks where they provide stronger, cheaper proof.
10. Verification cost optimization SHALL NOT trade away mandatory obligation coverage.
11. Background reasoning and revalidation SHALL NOT hold canonical reducer locks for long-running work.
12. Role-specific context remains bounded; full graphs and transcripts stay behind handles.

---

## 28. Evaluation, calibration, and proof

### 28.1 Required comparisons

```text
Builder-only baseline
vs same-model self-review
vs same-model separate-context verification
vs cross-family verification
vs deterministic + model verification
vs multi-aspect verification where applicable
```

### 28.2 Required metrics

- task completion;
- acceptance-criterion coverage;
- defect escape rate;
- verifier false-positive and false-negative rates;
- critical-finding precision;
- finding reproduction rate;
- repair success;
- rounds to settlement;
- oscillation and repeated-finding rates;
- scope contamination;
- Builder and Verifier anchoring;
- Evidence-linked finding rate;
- unsupported-objection rate;
- hidden-test overfitting;
- obligation coverage;
- router eligibility errors;
- unnecessary verification rate;
- temporal interpretation errors;
- unsupported estimate rejection;
- information-set leakage detection;
- source-dependence detection;
- outcome-resolution and scoring errors;
- learning-promotion reversal rate;
- negative transfer detection;
- token, latency, cost, RSS, and storage impact;
- operator interventions;
- post-settlement regressions.

### 28.3 Golden scenarios

At minimum prove:

1. a defect prevented by domain-specific verification;
2. a false Verifier objection rejected;
3. a successful repair and re-verification;
4. fail-closed verifier outage;
5. operator-resolved inconclusive dispute;
6. cross-domain obligation compilation;
7. temporal calendar or deadline error caught;
8. unsupported time estimate refused;
9. information-set leakage caught;
10. correlated sources not counted as independent;
11. invalid causal learning prevented from promotion;
12. negative transfer narrows a learning record;
13. a presentation-only Vertical activates no unnecessary ontology;
14. a semantic Vertical activates required packs and reflexes;
15. pack conflict blocks activation;
16. snapshot change invalidates prior verification;
17. router reroutes after a newly discovered risk;
18. replay reconstructs the full pair and settlement.

---

## 29. Implementation order

### Phase 0 — Reality and compatibility

- current ontology and Secondary Cognition reality pack;
- existing completion and verifier-field inventory;
- fixed V1/V2 snapshots, events, Workpoints, receipts, temporal and prediction fixtures;
- threat model;
- ownership and migration map;
- machine-readable requirement ledger for this specification.

### Phase 1 — Formal semantic compilation

- stable IDs;
- RDF/OWL/SHACL generation from the registry;
- parity checks against JSON Schema/OpenAPI/Rust/TypeScript;
- shared Verification Fabric Ontology;
- build-time pack conformance.

### Phase 2 — Shadow semantic validation

- candidate graph construction beside current behavior;
- intake/promotion/action validation in non-blocking shadow mode;
- false-positive/negative and performance measurement.

### Phase 3 — Execution pair substrate

- Semantic Work Contract;
- Semantic Execution Pair;
- Builder context and attempts;
- immutable snapshots;
- separate Verifier context and read-only workspace posture.

### Phase 4 — Obligation compiler and router

- obligation graph;
- capability registry;
- plan DAG;
- eligibility/coverage/independence SHACL;
- deterministic validators first;
- specialist verifier sessions.

### Phase 5 — Findings, repair, and settlement

- typed findings and responses;
- verifying-the-Verifier;
- rerouting;
- settlement shapes;
- reducer-backed decision and Receipt.

### Phase 6 — Spec 137 and 138 integration

- temporal obligation profiles;
- epistemic obligation profiles;
- calibration cohorts;
- champion/challenger routing policy;
- prediction and learning reflexes.

### Phase 7 — Vertical Intelligence Bundles

- bundle schema;
- activation flow;
- shared and Vertical packs;
- reflex overlays;
- composite Vertical proof;
- migration and governance.

### Phase 8 — Product and client projections

- Work Rail verifying states;
- Mission Canvas portfolio, findings, and proof surfaces;
- Pi/TUI/CLI/menubar parity;
- generated contract and docs release gates.

---

## 30. Required machine-readable decomposition

Before implementation, create:

```text
docs/contracts/spec144-complete-feature-ledger.v1.yaml
docs/contracts/spec144-primitive-ownership-matrix.v1.yaml
docs/contracts/spec144-obligation-verifier-matrix.v1.yaml
docs/contracts/spec144-cross-spec-amendment-matrix.v1.yaml
docs/contracts/spec144-migration-matrix.v1.yaml
docs/contracts/spec144-proof-matrix.v1.yaml
```

Every normative requirement SHALL map to implementation, positive/negative/adversarial/replay proof, Evidence, and Receipt. Unmapped or silently deferred mandatory requirements block conformance.

---

## 31. Acceptance criteria

Spec 144 is accepted only when:

1. Core registry definitions generate RDF, OWL 2 RL, and SHACL artifacts in parity with existing structural contracts.
2. Candidate, observation, inference, verifier, and settlement graphs remain distinct.
3. Every required semantic validation produces a durable receipt.
4. Unknown or invalid authoritative material fails closed and remains auditable.
5. Builder and Verifier lineages have distinct session/run/context identities.
6. Verifiers cannot acquire the Builder project writer lease.
7. Required Verifiers inspect an immutable content-addressed snapshot.
8. The settled snapshot equals the verified snapshot.
9. Build Claims and Verifier Findings are typed and evidence-linked.
10. Unsupported Verifier criticism cannot block settlement indefinitely.
11. An open valid critical finding prevents settlement.
12. Absence of findings cannot substitute for required coverage.
13. Obligations compile from semantic impact and policy rather than one task-to-one-Verifier routing.
14. Verification plans are DAGs with complete mandatory coverage.
15. SHACL blocks ineligible, incomplete, under-assured, or non-independent plans.
16. Same-model and cross-family verification receive honest independence classifications.
17. Deterministic validators, model agents, proof environments, external authorities, and operator review compose through one provider-neutral portfolio.
18. Dynamic rerouting responds to new semantic impact, findings, failures, steering, and snapshot changes.
19. Simple majority vote cannot override obligation-specific veto or severity policy.
20. Spec 137 temporal obligations improve domain calendar, deadline, estimate, freshness, and breach correctness without creating a second clock authority.
21. Spec 138 obligations improve information-set, source, resolution, scoring, calibration, causal, transfer, and learning-promotion correctness without creating a second epistemic authority.
22. Verifier and router calibration is scoped, evidence-backed, and governed through champion/challenger policy.
23. A new Vertical can activate through a versioned Vertical Intelligence Bundle.
24. Presentation-only Verticals do not require unnecessary ontology modules.
25. Semantic Verticals can add domain world, Evidence, temporal, prediction, verification, affordance, and authority extensions without duplicating shared primitives.
26. Composite Verticals preserve all active mandatory obligations.
27. Pack conflicts block activation and produce migration/governance guidance.
28. Universal verification reflexes and Vertical reflex overlays operate through existing Spec 97 families and authority boundaries.
29. Workpoint remains immediate action authority, Silent Sessions remain execution substrate, and reducer remains settlement authority.
30. Every settled pair produces Evidence and a Spec 119-compatible Receipt.
31. Replay reconstructs every contract, attempt, snapshot, obligation, plan, assignment, finding, response, disposition, validation, reroute, and settlement.
32. Fixed evaluations prove measurable improvement over Builder-only and flat self-review baselines.
33. Security, privacy, resource, migration, compatibility, generated-contract, and client-parity gates pass.
34. No feature is claimed complete through prose, booleans, verdict strings, process exit, model confidence, or mock surfaces alone.

---

## 32. Closure blockers

This specification cannot close while:

- ontology objects and links remain generic unvalidated JSON on the strict path;
- a favorable string substitutes for semantic validation;
- inferred facts are indistinguishable from asserted facts;
- OWL, SHACL, Router, Verifier, or client can mint canonical authority;
- Builder and Verifier share one flat context lineage;
- verification is merely asking the Builder to review itself;
- the Verifier can mutate the Builder source workspace;
- verification inspects a moving target;
- findings lack exact targets, evidence, severity, and method;
- Verifier findings bypass validation;
- a passing test automatically means complete;
- one agreeable model verdict automatically means settled;
- the Builder can self-dismiss findings;
- unsupported Verifier preferences can block work;
- independence is inferred from prompt wording alone;
- same-model verification is presented as fully independent;
- verifier timeout or unavailable tools silently pass;
- mandatory obligations can be removed by routing optimization;
- unrelated pass votes override a valid critical finding;
- repeated rounds continue without evidence delta;
- Workpoint, operator steering, Temporal Authority, or governing specifications can be outranked by loop momentum;
- Spec 137 is forked by a Vertical instead of extended through applicability profiles;
- Spec 138 is forked by a Vertical instead of extended through registered targets, outcomes, scoring, and learning applicability;
- a workspace profile substitutes visually for a missing semantic pack;
- a new Vertical requires hard-coded runtime branches where registry composition suffices;
- pack conflicts are silently resolved;
- completion remains reducible to booleans or unstructured verdict strings;
- implementation begins before the Spec 143 activation gate and explicit operator authorization permit it;
- required implementation, proof, migration, or parity work is silently deferred outside the machine-readable ledger.

---

## 33. Final architectural law

```text
A Vertical contributes domain meaning, evidence rules, temporal and epistemic applicability,
verifier capabilities, and reflexes.

RDF records what is asserted and observed.
OWL determines bounded semantic implications.
SHACL determines which declared operational constraints conform.
The Verification Router assigns each obligation to the right independent specialist or machine.
Secondary Cognition proposes, challenges, repairs, and learns.
Focusa’s daemon governs execution.
Focusa’s reducer alone records canonical settlement.
```
