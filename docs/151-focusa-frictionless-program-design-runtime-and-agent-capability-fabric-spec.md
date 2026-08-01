# Spec 151 — Frictionless Program Design Runtime, Vertical Composition, Universal Agent Capability Fabric, Workset Binding, and Human-Agent Experience Integrity

**Status:** Normative draft — primitive-owning — implementation and conformance not implied  
**Canonical label:** Focusa Program Design Runtime  
**Abbreviation:** PDR  
**Owner:** Focusa Core / Program Design / Agent Capability Fabric / Design-to-Reality Conformance  
**Created:** 2026-08-01  
**Source baseline reviewed:** `38a68112f57bddc765bec68629faef90e391c1ae`  
**Target release:** post-`0.9.137-dev`, subject to approved Workset admission  
**Supersedes:** all earlier conversational Program Design proposals and proposed Spec 151 amendments  
**Operationally supersedes:** Spec 103 as the canonical Program Design authority  
**Preserves:** existing Spec 103 `CallStackDesign` IDs and records as noncanonical compatibility projections  
**Primary dependencies:** Specs 66, 88, 100, 103, 104, 107, 109, 111, 113, 116, 119, 120, 125, 130/130A, 131, 133, 135B, 135F, 135I, 136, 137/137A, 138/138A, 139, 140/140A, 141, 144, 145–148, the Workset Flow Ledger specification currently numbered 149, and Spec 150.

---

## 0. Canonical consolidation rule

This document is the complete normative Spec 151 source.

No prior draft, amendment, conversation fragment, superseded rule, or alternative wording remains independently normative.

Implementation agents MUST NOT reconstruct precedence from conversation order.

Before implementation begins, Focusa MUST produce:

```text
docs/contracts/spec151-normative-source-coverage.v1.yaml
docs/contracts/spec151-complete-feature-ledger.v1.yaml
docs/contracts/spec151-delivery-dag.v1.yaml
```

Those artifacts MUST reference only this consolidated source hash.

---

## 1. One-line definition

Focusa MUST maintain a canonical, versioned, domain-general Program Design Runtime that converts approved user intent and semantic obligations into an executable design; equips agents with the exact tools, surfaces, context, permissions, recovery routes, and verification paths needed to realize it; binds that design to Worksets and Workpoints; continuously compares intended design with observed reality; and removes avoidable friction without transferring infrastructure maintenance to the human operator.

---

## 2. Executive decision

Program Design is not an architecture document.

It is the machine-operational bridge between:

```text
user intent
→ approved requirements
→ semantic obligations
→ intended system design
→ executable Workset
→ current Workpoint
→ available agent capabilities
→ implementation
→ observed reality
→ verification
→ settlement
→ forecasting and learning
```

The Program Design Runtime MUST answer:

1. What is the user trying to accomplish?
2. What obligations and outcomes must be satisfied?
3. How should the program or workflow function?
4. Which vertical semantics apply?
5. Which components, actors, operations, states, contracts, and relationships are required?
6. Which Workset members implement each part?
7. What is the immediate Workpoint action?
8. Which API, CLI, Pi, UIAI, shell, Git, connector, or other capability can perform it?
9. Which tool route is currently available and authorized?
10. How will success be observed and verified?
11. How will failure be contained and recovered?
12. What has actually been implemented?
13. Where has reality drifted from design?
14. What does the user need to decide?
15. What can the system handle automatically?
16. How long is the remaining work likely to take?
17. Did the process measurably reduce friction and improve outcomes?

---

## 3. Current repository grounding

Focusa already exposes a broad architecture of core reducers, API routes, CLI operations, Workpoints, Worksets, Pi tools, generated machine contracts, skills, runbooks, TUI, menubar, Mission Canvas, and UIAI browser integration.

Specs 137/137A supply typed temporal authority, exact scope, bounded timing arithmetic, Workpoint temporal projections, deadlines, urgency, and no-omission enforcement.

Specs 138/138A supply prediction commitment, information-set identity, resolution, scoring, calibration, learning, transfer, and epistemic governance.

Specs 140/140A supply Runtime Constitution compilation, instruction integrity, temporal adaptation boundaries, canonical amendment authority, and headless enforcement.

Spec 144 defines RDF/OWL/SHACL semantic integrity, obligation compilation, Builder↔Verifier separation, vertical composition, and zero-omission verification routing.

The Workset Flow Ledger defines durable membership, dependency edges, bounded ready frontiers, checkpoint flows, and formal completion transitions.

The remaining legacy `CallStackDesign` implementation is advisory and creates generic handlers, services, and adapters rather than a complete feature-specific Program Design. Spec 151 closes that gap.

---

## 4. Constitutional philosophy

```text
THE USER PROVIDES PURPOSE, JUDGMENT, DOMAIN TRUTH, AND AUTHORITY.

THE AGENT PROVIDES STRUCTURE, DESIGN, IMPLEMENTATION DISCIPLINE,
TOOL AWARENESS, CONTINUITY, VERIFICATION, DOCUMENTATION,
RECOVERY, AND PREDICTABILITY.

PROGRAM DESIGN MUST REDUCE HUMAN WORK.

THE USER MUST NOT BECOME THE CLERK OF THE AGENT’S INFRASTRUCTURE.

A MISSING CAPABILITY MUST NOT BECOME A DEAD END.

A BLOCKED ACTION MUST NOT BLOCK UNRELATED PRODUCTIVE WORK.

A GENERIC SCAFFOLD IS NOT PROGRAM DESIGN.

A TOOL CALL IS NOT AN OUTCOME.

A SOURCE-STRING HIT IS NOT CONFORMANCE.

A CLOSED TASK IS NOT PROOF THAT THE INTENDED SYSTEM WAS BUILT.

ACCEPTED MEANING MUST HAVE A DESIGN.
DESIGN MUST HAVE AN EXECUTABLE ROUTE.
EXECUTION MUST HAVE OBSERVABLE REALITY.
REALITY MUST HAVE VERIFICATION.
```

---

## 5. Foundational laws

1. **One canonical Program Design Runtime.**
2. **One canonical operation identity, many surface projections.**
3. **One canonical Focusa agent browser: UIAI Engine.**
4. **The user’s scoped outcome is primary.**
5. **Program Design is agent infrastructure, not user homework.**
6. **The minimum sufficient design is generated first.**
7. **Design depth increases only when evidence, risk, or complexity requires it.**
8. **Every material design element has stable identity.**
9. **Every canonical design is immutable by revision.**
10. **Corrections append and supersede; history is not rewritten.**
11. **Every requirement maps to a design obligation or explicit nonstructural disposition.**
12. **Every executable design element maps to a canonical operation.**
13. **Every operation maps to one or more governed execution routes.**
14. **Every consequential route identifies authority, confirmation, evidence, and recovery.**
15. **Every mutable state object identifies one canonical owner.**
16. **Every external interaction identifies an adapter boundary.**
17. **Every failure path identifies containment and recovery.**
18. **Every consequential behavior identifies verification.**
19. **Observed code does not silently amend canonical design.**
20. **Approved design without observed implementation remains incomplete.**
21. **Unexpected implementation without a design or governed deviation is drift.**
22. **Worksets do not become Program Design authority.**
23. **Program Design does not become Workset scheduling authority.**
24. **Workpoints remain immediate-action authority.**
25. **UIAI Engine remains browser perception and actuation authority.**
26. **Focusa retains mission, scope, Workset, Workpoint, evidence, and closure authority.**
27. **No alternate browser silently substitutes for UIAI Engine.**
28. **No missing integration globally blocks unrelated work.**
29. **No silent installation.**
30. **No dead-end failure message.**
31. **No repeated question when the answer is already available.**
32. **No unnecessary user approval for safe reversible metadata work.**
33. **Protective friction is compressed, not removed.**
34. **Avoidable friction is a closure-blocking defect.**
35. **Automation remains inspectable, reversible where possible, and truthfully reported.**
36. **The complete capability graph stays machine-readable; only bounded relevant context reaches the model.**
37. **Program Design must improve runtime stability and predictability.**
38. **Experience quality must be evaluated, not assumed.**
39. **No active mandatory obligation may be deferred or disappear from the closure graph.**
40. **Runtime adaptation may change execution order and route only inside approved semantic and authority boundaries.**

---

## 6. Normal user contract

The normal user experience is:

```text
User:
  describes the desired outcome
  supplies domain truth unavailable elsewhere
  corrects misunderstandings
  approves consequential choices
  changes priorities
  reviews material tradeoffs

Focusa and the agent:
  inspect the project
  discover applicable tools
  infer verticals
  create and maintain the design
  decompose and bind Worksets
  create Workpoints
  select execution routes
  acquire missing capabilities
  continue independent work
  implement and verify
  maintain documentation
  detect and repair drift
  preserve continuity
  estimate completion
```

The user is not expected to manually maintain:

- design graphs;
- RDF statements;
- SHACL mappings;
- tool mappings;
- Workset design bindings;
- Workpoint slices;
- state-ownership maps;
- source bindings;
- runtime conformance records;
- machine documentation;
- evidence references;
- routine design revisions;
- restart instructions.

---

## 7. Minimum necessary interaction

The agent may interrupt the user only for:

1. unresolved intent;
2. unavailable domain truth;
3. material tradeoffs;
4. irreversible consequences;
5. public-contract changes;
6. destructive operations;
7. external authentication;
8. material cost;
9. authority or permission;
10. unresolved contradictions of similar authority;
11. canonical amendment;
12. high-consequence vertical activation.

The agent MUST NOT ask the user merely because:

- an internal schema field is empty;
- an internal identifier is needed;
- a node name must be generated;
- an edge type must be selected;
- an ontology IRI is required;
- a Workset binding must be created;
- a tool route must be selected;
- machine documentation needs regeneration;
- a design profile can be inferred;
- an answer already exists in project context.

Questions MUST be compressed, recommendations MUST be supplied, and related decisions SHOULD be batched.

---

## 8. Six-graph alignment model

Focusa MUST distinguish:

```text
1. Obligation Graph
   What must be true.

2. Program Design Graph
   How the intended system should satisfy those obligations.

3. Workset Flow Graph
   How implementation work is grouped, ordered, checked, and completed.

4. Capability and Execution Graph
   Which operations, surfaces, tools, adapters, environments, and permissions
   can realize the design.

5. Observed Reality and Verification Graph
   What actually exists and how it is proven.

6. Temporal and Epistemic Graph
   What was predicted, how long it took, what happened, and what was learned.
```

These graphs MUST be linked but MUST NOT be collapsed into one overloaded object.

---

## 9. Authority boundaries

| Concern | Canonical authority |
|---|---|
| User intent and final steering | Operator |
| Requirement meaning | Primitive-owning specifications |
| Semantic composition and formal verification | Spec 144 |
| Program architecture and behavior design | Spec 151 |
| Canonical operation semantics | Operation Registry |
| Tool and surface availability | Capability Registry |
| Stable agent instruction | Specs 140/140A |
| Environment and placement | Spec 139 |
| Work membership and flow | Workset Flow Ledger |
| Immediate action | Workpoint |
| Workpoint Item timing and closure | Spec 131 |
| Browser perception and actuation | UIAI Engine |
| Evidence and Receipts | Spec 119 |
| Settlement | Spec 136 |
| Time and estimates | Specs 137/137A |
| Prediction and learning | Specs 138/138A |
| Release execution | Specs 145–148 |

Program Design may constrain these authorities but MUST NOT duplicate them.

---

## 10. Runtime aggregate family

Spec 151 MUST implement these distinct runtime objects:

```text
ProgramDesignDefinition
ProgramDesignRevision
ProgramDesignRuntimeState
ProgramDesignSlice
ProgramDesignExecutionBinding
ProgramDesignAdmissionDecision
ProgramDesignExecutionEvent
ObservedProgramGraph
ProgramConformanceReport
ProgramDesignDeviation
ProgramDesignSettlement
CapabilityReadinessPlan
AgentCapabilityPacket
FrictionFinding
FrictionResolution
DecisionCapsule
```

---

## 11. `ProgramDesignGraph`

```yaml
schema: focusa.program_design_graph.v1

design_id:
revision:
parent_revision_ref:

scope:
  project_ref:
  project_root:
  continuity_id:
  workstream_ref:

status:
  draft |
  grounded |
  challenged |
  pending_operator |
  approved |
  active |
  superseded |
  revoked

profile:
  micro |
  standard |
  critical |
  systemic

mission:
title:
design_digest:
git_baseline_ref:

requirement_refs: []
acceptance_refs: []
semantic_obligation_refs: []
non_structural_obligation_refs: []

node_refs: []
edge_refs: []
contract_refs: []
state_ownership_refs: []
invariant_refs: []
failure_path_refs: []
recovery_path_refs: []
verification_binding_refs: []
authority_boundary_refs: []
migration_refs: []
compatibility_refs: []

vertical_composition_ref:
runtime_constitution_ref:
capability_plan_ref:
workset_binding_refs: []
workpoint_binding_refs: []

prediction_refs: []
estimate_claim_refs: []

created_by:
created_at:
approved_by:
approved_at:
evidence_refs: []
receipt_refs: []
```

---

## 12. Domain-general metamodel

The universal core MUST use stable high-level kinds:

```text
Actor
Capability
Operation
Decision
Object
Artifact
State
Contract
Event
Observation
Dependency
AuthorityBoundary
Invariant
FailurePath
RecoveryPath
VerificationPath
Outcome
```

Vertical-specific meaning is supplied through ontology-addressed kinds:

```yaml
core_kind: operation
kind_ref: focusa://vertical/software/http-route
```

```yaml
core_kind: artifact
kind_ref: focusa://vertical/legal/court-filing
```

```yaml
core_kind: operation
kind_ref: focusa://vertical/robotics/motor-command
```

Focusa Core MUST NOT require a new hardcoded enum for every vertical concept.

---

## 13. Vertical composition

```yaml
schema: focusa.vertical_program_composition.v1

composition_id:
primary_vertical_ref:
supporting_vertical_refs: []

vertical_bundle_refs: []
domain_pack_refs: []
ontology_pack_refs: []
profile_refs: []

cross_vertical_bridge_refs: []
combined_shape_set_ref:
combined_verifier_portfolio_ref:
combined_temporal_profile_ref:
combined_epistemic_profile_ref:

conflict_refs: []
unknown_impact_refs: []

status:
  proposed |
  approved |
  active |
  conflicted |
  blocked
```

Reference vertical profiles include:

### Software

```text
route, handler, service, reducer, schema, event, store,
adapter, migration, test, deployment surface
```

### Professional operations

```text
role, activity, decision, approval, case, document,
handoff, deliverable, service obligation
```

### Research

```text
question, hypothesis, source, dataset, observation,
transformation, experiment, result, review, replication
```

### Compliance

```text
obligation, control, policy, approval, segregation of duties,
audit artifact, incident, remediation
```

### Physical or robotic systems

```text
sensor, perception, world-state estimate, controller,
actuator, physical action, safety envelope, emergency stop
```

Unknown compatibility between activated profiles blocks only affected consequential work.

---

## 14. Program Design profiles and design-tax budget

### Micro

For tiny, local, reversible work.

Required:

- exact intent;
- affected element;
- invariant;
- expected change;
- proof;
- revert posture.

### Standard

For bounded features.

Required:

- nodes and edges;
- contracts;
- state ownership;
- failure paths;
- verification;
- Workset binding.

### Critical

For security, authentication, persistence, billing, migration, destructive operations, distributed state, public APIs, or closure authority.

Requires full authority, failure, recovery, migration, and independent verification modeling.

### Systemic

For cross-repository, multi-daemon, platform, or ecosystem architecture.

Requires full topology, cross-system contracts, vertical composition, placement, multi-agent territories, and operator approval.

### Design-tax budget

```yaml
schema: focusa.program_design_tax_budget.v1

maximum_time_before_first_useful_action_ms:
maximum_tokens_before_first_useful_action:
maximum_initial_questions:
maximum_initial_graph_depth:

automatic_profile_promotion: true
promotion_triggers:
  - risk_increased
  - scope_expanded
  - state_ownership_changed
  - public_contract_changed
  - cross_repository_detected
  - destructive_behavior_detected
  - unknown_impact_detected
```

Design may automatically promote in depth. It MUST NOT silently demote.

---

## 15. Program nodes, edges, and contracts

### Program node

```yaml
node_id:
core_kind:
kind_ref:
canonical_name:
purpose:

requirement_refs: []
acceptance_refs: []
ontology_refs: []

input_contract_refs: []
output_contract_refs: []
state_owner_ref:

required_operation_refs: []
capability_requirement_refs: []
verification_binding_refs: []

authority_requirement_refs: []
failure_policy_ref:
source_location_expectations: []
```

### Program edge

```yaml
edge_id:
kind:
from_node_ref:
to_node_ref:
purpose:

requirement_refs: []
input_contract_ref:
output_contract_ref:

precondition_refs: []
postcondition_refs: []
failure_path_refs: []

operation_ref:
capability_requirement_ref:
verification_binding_refs: []
```

### Program contract

Every boundary contract MUST identify:

- schema and version;
- producer;
- consumers;
- compatibility;
- privacy;
- retention;
- serialization;
- migration;
- validation;
- error behavior.

---

## 16. State ownership

```yaml
schema: focusa.program_state_ownership.v1

state_ref:
canonical_owner_node_ref:

readers: []
writers: []
mutation_operation_refs: []

transaction_policy_ref:
concurrency_policy_ref:
replication_policy_ref:
conflict_policy_ref:
recovery_policy_ref:

evidence_refs: []
```

Multiple undeclared canonical writers are prohibited.

---

## 17. Runtime state

```yaml
schema: focusa.program_design_runtime_state.v1

project_ref:
continuity_id:

active_design_ref:
active_design_revision:
active_vertical_bundle_refs: []

status:
  inactive |
  grounding |
  pending_approval |
  active |
  drifted |
  blocked |
  degraded |
  superseded

workset_bindings: []
workpoint_bindings: []
active_execution_bindings: []

semantic_validation_ref:
observed_program_graph_ref:
latest_conformance_report_ref:
git_anchor_ref:

unresolved_design_findings: []
unsettled_deviation_refs: []
unknown_impact_refs: []

freshness:
  design_revision_current:
  requirement_revision_current:
  vertical_bundle_current:
  runtime_constitution_current:
  capability_snapshot_current:
  observed_graph_current:
  git_relation:

updated_at:
receipt_refs: []
```

---

## 18. Program Design lifecycle

```text
scope_verified
→ obligations_loaded
→ current_reality_observed
→ verticals_inferred
→ minimum_design_drafted
→ alternatives_compared
→ adversarial_challenge
→ semantic_validation
→ pending_operator_when_required
→ approved
→ active
→ Workset_bound
→ Workpoint_slices_issued
→ implementation_observed
→ incremental_conformance
→ verification_complete
→ settlement_input_ready
→ superseded
```

Agents MUST inspect existing evidence before questioning the user.

Material alternatives SHOULD receive predictions through Spec 138 before approval. Prediction advises; it does not approve.

---

## 19. Workset integration

```yaml
schema: focusa.workset_design_binding.v1

binding_id:
workset_ref:
workset_revision:
member_ref:

program_design_ref:
program_design_revision:
program_slice_ref:

vertical_profile_refs: []
requirement_refs: []
architecture_obligation_refs: []

allowed_node_refs: []
required_edge_refs: []
allowed_mutation_targets: []

required_operation_refs: []
required_capability_refs: []
verification_portfolio_ref:

design_freshness:
  current |
  stale |
  superseded |
  conflicted |
  unknown

created_event_ref:
receipt_ref:
```

### Ready-frontier rule

A member is ready only when:

```text
provider readiness
+ dependency readiness
+ approved design revision
+ valid Program Design Slice
+ Runtime Constitution validity
+ capability readiness
+ temporal preflight
+ environment and placement validity
+ Workpoint eligibility
+ verification readiness
```

A blocked member MUST NOT globally block independent members.

A material design revision MUST mark affected Workset bindings stale, preserve unaffected evidence, recalculate verification and ETA impact, and require governed rebind. Members MUST NOT silently inherit a new design revision.

---

## 20. Workpoint integration

Every implementation Workpoint SHOULD carry:

```text
program_design_ref
program_design_revision
active_program_slice_ref
current_program_node_ref
expected_transition_ref
allowed_mutation_targets
applicable_invariants
required_operations
preferred_tools
verification_bindings
recovery_tools
design_freshness
```

Compaction, handoff, restart, and model changes MUST preserve these references.

A Workpoint remains immediate-action authority. A Program Design Slice constrains what counts as aligned action.

---

## 21. Universal Agent Capability Fabric

Program Design MUST represent:

```text
Capability
  what the agent needs to accomplish

Operation
  the canonical governed action

Tool
  one callable projection

Surface
  where the operation is exposed

Adapter
  connection to an external execution system

Environment
  where the route can execute
```

These concepts MUST remain distinct.

---

## 22. Required surfaces

Applicable Program Design operations MUST account for:

- Focusa daemon API;
- Focusa CLI;
- Pi `focusa_*` tools;
- MCP tools;
- OpenAI-compatible tools;
- generated SDKs and clients;
- UIAI Engine browser;
- UIAI Engine OS actuation;
- UIAI-governed WebMCP;
- shell and process execution;
- Git and repository operations;
- Beads and task providers;
- Workset operations;
- Workpoint operations;
- Evidence and Receipt tools;
- Mission Canvas;
- Work Rail;
- A2UI;
- TUI;
- menubar;
- Silent Sessions;
- daemon workers;
- external connectors.

Parity does not require identical presentation. It requires equivalent authority, scope, mutation behavior, confirmation, idempotency, result semantics, failure behavior, evidence, and recovery.

---

## 23. `CapabilityRequirement`

```yaml
schema: focusa.capability_requirement.v1

capability_requirement_id:
program_design_ref:
program_element_ref:

capability_class:
operation_ref:
purpose:

mode:
  inspect |
  plan |
  mutate |
  execute |
  observe |
  verify |
  recover |
  communicate |
  settle

side_effect_class:
  none |
  local_reversible |
  local_consequential |
  external_reversible |
  external_consequential |
  destructive

required_input_contract_refs: []
required_output_contract_refs: []
required_evidence_kinds: []
required_receipt_types: []

scope_requirements:
  project_required:
  continuity_required:
  workset_required:
  workpoint_required:
  item_required:
  session_required:
  origin_required:

authority_requirement_refs: []
permission_requirement_refs: []
confirmation_policy_ref:
idempotency_policy_ref:

environment_requirements: []
vertical_profile_refs: []

fallback_policy_ref:
recovery_policy_ref:
```

---

## 24. Execution surfaces and routes

```yaml
schema: focusa.execution_surface_descriptor.v1

surface_id:
surface_kind:
  focusa_api |
  focusa_cli |
  pi_tool |
  mcp_tool |
  openai_tool |
  generated_client |
  uiai_browser |
  uiai_os |
  uiai_webmcp_capability |
  shell |
  git |
  task_provider |
  mission_canvas |
  work_rail |
  a2ui |
  tui |
  menubar |
  silent_session |
  daemon_worker |
  external_connector

provider_ref:
adapter_ref:
version:
platform:
environment_ref:

health:
  available |
  degraded |
  unavailable |
  unknown

operation_refs: []
capability_refs: []
trust_class:
fresh_until:
evidence_refs: []
```

```yaml
schema: focusa.execution_route.v1

route_id:
operation_ref:
capability_requirement_ref:

surface_ref:
tool_ref:
adapter_ref:
environment_ref:

priority:
selection_reason:
applicability_conditions: []

preflight_operation_refs: []
invocation_contract_ref:
expected_result_contract_ref:
verification_operation_refs: []
evidence_requirements: []
receipt_requirements: []
recovery_operation_refs: []

fallback_route_refs: []

current_status:
  preferred |
  eligible |
  degraded |
  unavailable |
  blocked |
  incompatible
```

Runtime route selection considers:

1. operation compatibility;
2. authority;
3. exact scope;
4. safety;
5. evidence quality;
6. semantic directness;
7. reliability;
8. health;
9. latency;
10. cost;
11. user disruption.

Program Design refers to canonical operations. Runtime selects an eligible projection.

---

## 25. Program Design admission guard

Every consequential Workset or Workpoint action MUST pass a daemon-owned `ProgramDesignAdmissionGuard`.

```yaml
schema: focusa.program_design_admission.v1

decision:
  allowed |
  blocked |
  operator_required |
  degraded_read_only

design_ref:
design_revision:
vertical_bundle_refs: []
workset_ref:
member_ref:
workpoint_ref:
item_ref:

current_node_ref:
proposed_operation_ref:
proposed_target_refs: []

satisfied_invariants: []
violated_invariants: []
warnings: []
blockers: []

exact_next_action:
recovery_operations: []
evidence_refs: []
receipt_ref:
```

The guard MUST use local canonical state and bounded design slices. It MUST NOT require a remote model call before every action.

---

## 26. UIAI Engine exclusive browser authority

```text
UIAI ENGINE IS THE FOCUSA AGENT BROWSER.

NOT ONE OPTION.
NOT ONE PROVIDER AMONG MANY.
```

UIAI Engine exclusively owns Focusa-governed agent:

- browser sessions;
- navigation;
- page perception;
- accessibility snapshots;
- DOM and source inspection;
- screenshots and visual perception;
- console and network diagnostics;
- authenticated browser workflows;
- browser mutation;
- WebMCP execution;
- session and origin isolation;
- browser evidence;
- browser recovery and cleanup.

No independently governed Playwright, Selenium, Puppeteer, Browserless, browser MCP, harness-native browser, or separate Chromium agent may substitute for UIAI Engine.

Those technologies may exist internally under UIAI ownership.

Focusa owns why, whether, under what scope, and with what evidence a browser operation occurs. UIAI Engine owns browser perception and actuation.

---

## 27. WebMCP boundary

WebMCP is a capability inside UIAI Engine, not a separate browser.

```text
Program Design browser operation
→ Focusa authority
→ UIAI session
→ session/origin-bound capability intake
→ optional validated WebMCP operation
→ UIAI execution
→ UIAI observation
→ Focusa Evidence and Receipt
```

Page-provided capabilities MUST be schema-validated, trust-classified, session-bound, origin-bound, mutation-classified, and prevented from cross-origin reuse.

Page annotations never grant authority.

---

## 28. UIAI browser execution plan

```yaml
schema: focusa.browser_execution_plan.v1

plan_id:
program_design_ref:
program_slice_ref:
workset_member_ref:
workpoint_ref:

operation_intent:
target_origin:
session_requirement:
mutation:
destructive:

canonical_browser: uiai_engine
alternate_browser_routes: forbidden

preferred_mode:
  validated_webmcp |
  uiai_accessibility |
  uiai_dom_source |
  uiai_visual |
  uiai_direct_action

required_steps:
  - uiai_health
  - browser_session_open_or_bind
  - browser_read_or_source
  - capability_intake_when_present
  - diagnostics_if_required
  - stable_snapshot
  - mutation_preflight
  - execute_in_bound_session
  - post_action_read
  - diagnostics_intake
  - evidence_capture
  - unused_session_close

confirmation_policy_ref:
evidence_policy_ref:
failure_policy_ref:
recovery_routes: []
```

Browser completion requires post-action observation and evidence, not merely successful invocation.

---

## 29. UIAI perpetual evaluation mode

UIAI Engine MUST provide a **perpetual free evaluation mode**.

Evaluation mode:

- has no expiration date;
- has no countdown;
- must not terminate unexpectedly;
- provides a stable, documented limited capability profile;
- may omit or limit premium features;
- must preserve user work, evidence, pairing, and configuration;
- must not repeatedly interrupt the operator with sales prompts;
- must not become a dark-pattern trial;
- must clearly identify unavailable premium capabilities.

```yaml
schema: focusa.uiai_evaluation_profile.v1

evaluation_mode: perpetual_feature_limited

included_capabilities: []
limited_capabilities: []
unavailable_capabilities: []

usage_limits: {}
privacy_profile_ref:
support_profile_ref:
upgrade_operation_ref:

expires_at: null
```

When a task requires a capability unavailable in evaluation mode:

1. explain the exact missing capability;
2. offer the applicable upgrade;
3. preserve the suspended operation;
4. continue independent work;
5. never select another browser;
6. automatically resume after later capability activation.

---

## 30. UIAI acquisition and installation

When UIAI is required but unavailable:

```text
inspect
→ identify exact missing state
→ offer perpetual evaluation install
→ continue independent work
→ obtain approval or standing policy
→ resolve trusted artifact
→ verify provenance
→ install atomically
→ activate evaluation
→ pair with Focusa
→ verify health and capabilities
→ run read-only smoke proof
→ resume suspended work automatically
```

No silent installation is permitted unless an explicit standing policy authorizes it.

```yaml
uiai_companion_policy:
  ask_when_required |
  auto_install_evaluation |
  auto_repair_and_update |
  never_install
```

Installation MUST be idempotent, user-safe, previewable, verifiable, repairable, updateable, and rollback-capable.

---

## 31. Productive failure

The rule is:

> Fail safely at the affected boundary and productively everywhere else.

When UIAI or another capability is unavailable:

```text
affected operation:
  suspended safely

independent Workset work:
  continues

recovery:
  begins or is offered immediately

resume:
  automatic after verified recovery
```

Dead-end messages such as “cannot continue” or “install manually” are prohibited unless no supported automated or guided route exists.

The scheduler MUST NOT start irrelevant work merely to appear productive. Parallel work must remain inside the approved Workset and contribute to the scoped outcome.

---

## 32. Capability suspension and automatic resume

```yaml
schema: focusa.capability_suspension.v1

suspension_id:
reason:

scope:
  project_ref:
  continuity_id:
  workset_ref:
  member_ref:
  workpoint_ref:
  item_ref:

program_design_ref:
program_design_revision:
program_slice_ref:
operation_ref:

last_completed_step:
next_required_step:
input_binding_ref:
idempotency_key:

required_capability_refs: []
independent_work_refs: []

recovery_plan_ref:
resume_policy: automatic_after_verified_recovery

created_at:
evidence_refs: []
receipt_refs: []
```

Resume must verify that:

- the binding remains current;
- the operation is not already complete;
- no external mutation would be duplicated;
- leases and permissions remain valid;
- the design and Workset revision remain compatible.

---

## 33. Proactive capability readiness

Focusa MUST prepare capabilities before they block the critical path.

```yaml
schema: focusa.capability_readiness_plan.v1

plan_id:
workset_ref:
program_design_ref:

required_capabilities:
  - capability_ref:
    predicted_first_use_ref:
    acquisition_lead_time_ms:
    current_readiness:
    prepare_before_ref:
    critical_path_impact_ms:
    can_prepare_automatically:
    operator_action_refs: []

readiness_digest:
updated_at:
```

Preparation must be neither unnecessarily early nor discovered too late.

Readiness planning includes UIAI, authentication, provider access, model endpoints, test environments, datasets, dependencies, disk, memory, network, verifier availability, and foreseeable approvals.

---

## 34. Agent Capability Packet

```yaml
schema: focusa.agent_capability_packet.v1

packet_id:
scope:
  project_ref:
  continuity_id:
  workset_ref:
  member_ref:
  workpoint_ref:
  item_ref:

program_design_ref:
program_design_revision:
program_slice_ref:

current_objective:
current_program_node_ref:
expected_transition_ref:

required_operations: []
preferred_tools: []
fallback_tools: []
prohibited_tools: []

preflight_tools: []
verification_tools: []
evidence_tools: []
recovery_tools: []

skill_refs: []
runbook_refs: []

surface_health_summary: []
capability_gaps: []
operator_decisions_required: []

packet_digest:
fresh_until:
rehydrate_refs: []
```

The packet remains bounded. Full schemas are loaded only when selected.

---

## 35. Progressive disclosure

```text
Level 0 — Agent Card
  Surface families and registry digest.

Level 1 — Capability search
  Narrow matching operations.

Level 2 — Tool graph or bundle
  Required workflow.

Level 3 — Exact contract
  Selected tool schema.

Level 4 — Runbook
  Complex or failed workflow.

Level 5 — Cold artifacts
  Full specs and evidence.
```

Program Design maintains the exhaustive machine graph. The model sees only the relevant slice.

---

## 36. Runtime Constitution integration

Spec 140 MUST compile from Program Design:

- relevant capabilities;
- visible tools;
- auto-selectable tools;
- operator-only tools;
- prohibited tools;
- progressive skill-loading rules;
- permitted fallback routes;
- routes that must never be substituted;
- stable architecture invariants;
- dynamic slice-retrieval obligations.

The Runtime Constitution MUST mention only tools available to the exact harness and environment.

A generated instruction artifact that contradicts the active Program Design MUST fail instruction-integrity validation.

---

## 37. Multi-agent coordination

```yaml
schema: focusa.agent_design_execution_binding.v1

binding_id:
agent_instance_ref:

program_design_ref:
program_design_revision:

workset_member_refs: []
claimed_program_element_refs: []
allowed_mutation_targets: []
prohibited_overlap_refs: []

lease_ref:
generation:
idempotency_namespace:

builder_or_verifier_role:
verifier_separation_refs: []

handoff_policy_ref:
recovery_policy_ref:
```

Required behavior:

- stale-agent fencing;
- overlapping-edit detection;
- duplicate-operation prevention;
- exactly-once external mutations;
- explicit Builder↔Verifier separation;
- automatic reassignment after agent failure;
- bounded handoff packets;
- reconciliation responsibility.

---

## 38. Cross-surface action continuity

Every material action or approval receives one global interaction identity.

```yaml
schema: focusa.cross_surface_interaction.v1

interaction_id:
operation_ref:
current_state:

presented_surface_refs: []
acknowledged_surface_ref:
approved_surface_ref:

decision_ref:
approval_ref:
execution_ref:
result_ref:

deduplication_key:
resume_ref:
```

A UIAI install offered in Pi may be approved in the menubar, observed in Mission Canvas, inspected through CLI, and resumed in another harness without duplicate prompts or divergent state.

---

## 39. Approval leases

```yaml
schema: focusa.scoped_approval_lease.v1

lease_id:
scope_refs: []

permitted_operation_classes: []
excluded_operation_refs: []

maximum_risk_class:
maximum_cost:
maximum_uses:

valid_from:
expires_at:

revocable:
revocation_ref:
audit_refs: []
```

Approval leases reduce repeated prompts without granting unlimited authority.

---

## 40. Operator Attention Router

```yaml
schema: focusa.operator_attention_decision.v1

attention_id:
decision_required:
decision_owner_ref:

urgency:
interruption_cost:
batch_group_ref:

preferred_surface_ref:
quiet_window_policy_ref:
acknowledgement_required:
escalation_at:

why_now:
recommended_action:
alternatives: []

evidence_refs: []
```

The router MUST:

- batch related decisions;
- avoid duplicate notifications;
- respect quiet windows;
- select the appropriate operator;
- escalate only when necessary;
- explain why attention is required now.

---

## 41. Decision compression and Decision Capsules

Every material automatic decision produces a bounded capsule:

```yaml
schema: focusa.decision_capsule.v1

decision_id:
what_happened:
why:
evidence_basis: []

automatic_authority_ref:
alternatives_considered: []

user_impact:
work_continuing:

reversible:
undo_operation_ref:

attention_required:
```

The default user projection should answer:

```text
What happened?
Why?
What is happening now?
What decision needs me?
What will continue automatically?
Can it be undone?
How did the estimate change?
```

Routine successful internal maintenance remains quiet. Material changes remain visible.

---

## 42. Simulation and rehearsal

Before consequential operations, Focusa SHOULD support:

```text
program_design.simulate
workset.rebind.simulate
capability.acquire.simulate
uiai.browser_workflow.simulate
migration.simulate
external_mutation.simulate
release_effect.simulate
```

Simulation output MUST distinguish:

- observed facts;
- planned actions;
- predicted effects;
- assumptions;
- actual side effects, which MUST be zero.

---

## 43. Universal undo and compensation

The user SHOULD be able to request:

```text
Undo the last reversible Program Design execution.
```

Focusa MUST explain:

- what is reversible;
- what is not;
- external consequences;
- compensation steps;
- verification;
- changed Workset members;
- forecast impact.

Rollback success does not convert the original failed operation into success.

---

## 44. Protective-friction classification

```yaml
schema: focusa.friction_classification.v1

class:
  avoidable |
  reducible |
  protective |
  externally_imposed |
  currently_irreducible |
  unknown

reason:
required_authority_refs: []
reduction_options: []
prohibited_removals: []
```

The optimization goal is:

```text
eliminate avoidable friction
compress reducible friction
preserve necessary protective friction
explain externally imposed friction
reassess currently irreducible friction
```

Safety and authority controls cannot be deleted merely because they consume time.

---

## 45. Friction Eradication Runtime

```text
observe
→ detect
→ classify
→ identify root cause
→ preserve productive flow
→ remediate
→ verify
→ measure
→ prevent recurrence
```

```yaml
schema: focusa.friction_finding.v1

finding_id:
scope_refs: []

friction_class:
source_surface_ref:
affected_operation_refs: []
affected_capability_refs: []

symptom:
root_cause:
avoidability:

impact:
  user_interruptions:
  manual_steps:
  user_time_ms:
  agent_wait_ms:
  critical_path_delay_ms:
  extra_tokens:
  retries:
  rework_risk:

automatic_remediation_available:
remediation_ref:
productive_parallel_work_refs: []

operator_action_required:
recurrence_count:

status:
  detected |
  remediating |
  eliminated |
  reduced |
  protective_verified |
  recurring
```

```yaml
schema: focusa.friction_resolution.v1

resolution_id:
finding_ref:
remediation_operation_refs: []

before_metrics: {}
after_metrics: {}

user_action_count:
automatic_action_count:
productive_work_preserved:
resume_successful:
recurrence_prevention_refs: []

verification_refs: []
prediction_outcome_refs: []
learning_refs: []
receipt_ref:

status:
  eliminated |
  reduced |
  failed |
  regressed
```

A workaround alone does not close a friction finding. Closure requires verified reduction or automatic recoverability.

---

## 46. Required friction classes

Focusa MUST hunt for:

- capability discovery friction;
- installation friction;
- license/evaluation friction;
- authentication friction;
- tool-routing friction;
- project-scope friction;
- Program Design friction;
- Workset friction;
- Workpoint friction;
- context and compaction friction;
- verification friction;
- cross-surface inconsistency;
- version incompatibility;
- environment friction;
- user-interruption friction;
- recovery friction;
- documentation drift;
- performance and latency friction;
- multi-agent collision;
- accessibility friction;
- team-decision duplication.

Repeated avoidable friction MUST create a governed improvement candidate through Specs 138 and the approved recursive-improvement substrate.

---

## 47. Friction budgets

```yaml
schema: focusa.friction_budget.v1

maximum_user_questions:
maximum_separate_approvals:
maximum_manual_fields:
maximum_copy_paste_steps:
maximum_repeated_questions:
maximum_unexplained_failures:
maximum_avoidable_full_restarts:
maximum_avoidable_wait_ms:

automatic_grounding_target:
automatic_binding_target:
automatic_resume_target:
```

Default targets:

```text
manual design graph entry: 0
manual tool mapping: 0
manual Workset design binding: 0
manual continuation reconstruction: 0
repeated project questions: 0
silent failures: 0
alternate browsers: 0
avoidable full-flow restarts: 0
```

---

## 48. Brownfield adoption

Existing projects may not have a complete Program Design.

Required adoption path:

```text
observe current project
→ build provisional design
→ assign confidence
→ identify high-risk unknowns
→ operate in advisory mode
→ validate progressively
→ activate guards only where sufficient evidence exists
```

The absence of a complete design MUST NOT globally block safe existing work.

Consequential operations may require higher design confidence.

---

## 49. Observed Program Graph

Sources MAY include:

- compiler and language-server metadata;
- AST and import analysis;
- route registries;
- Operation Registry;
- schemas;
- migrations;
- dependency manifests;
- reducer events;
- database tables;
- runtime traces;
- tests;
- generated contracts;
- provider registrations;
- verified Git state.

Observation confidence:

```text
compiler_verified
ast_verified
registry_verified
runtime_verified
schema_verified
source_presence_only
inferred
unknown
```

`source_presence_only` cannot establish behavioral or authority conformance.

Observation adapters MUST report uncertainty rather than inventing relationships.

---

## 50. Design-to-reality conformance

```text
Level 0 — scaffold
Level 1 — surface existence
Level 2 — structural alignment
Level 3 — control/data/event flow alignment
Level 4 — behavioral and failure-path alignment
Level 5 — authority alignment
Level 6 — executable runtime proof
Level 7 — settled requirement/design/code/evidence/outcome alignment
```

Only the profile-required level satisfies closure.

```yaml
schema: focusa.program_conformance_report.v1

report_id:
design_ref:
design_revision:
observed_graph_ref:
git_anchor_ref:
profile:
required_level:
achieved_level:

requirement_coverage:
design_node_coverage:
design_edge_coverage:
contract_coverage:
state_ownership_coverage:
failure_path_coverage:
verification_coverage:
capability_route_coverage:

missing_design_elements: []
missing_implementation_elements: []
unexpected_implementation_elements: []
authority_violations: []
contract_mismatches: []
runtime_failures: []
approved_deviation_refs: []
unknown_impact_refs: []

status:
  aligned |
  partial |
  drifted |
  contradicted |
  indeterminate

evidence_refs: []
receipt_ref:
```

---

## 51. Drift classes

Focusa MUST detect:

- requirement-to-design gap;
- design-to-code gap;
- code-to-design gap;
- design revision drift;
- contract drift;
- state ownership drift;
- authority drift;
- verification drift;
- Workset binding drift;
- capability drift;
- cross-surface drift;
- vertical profile drift;
- documentation drift.

Drift MUST update Workset readiness, Workpoint packets, forecasts, and operator projections where material.

---

## 52. Program Design deviations

```yaml
schema: focusa.program_design_deviation.v1

deviation_id:
design_ref:
design_revision:

scope_refs: []
observed_element_refs: []
affected_requirement_refs: []

classification:
  implementation_detail |
  compatible_extension |
  semantic_change |
  authority_change |
  scope_expansion |
  defect |
  unknown

reason:
proposed_disposition:

operator_approval_required:
status:
  proposed |
  approved |
  rejected |
  incorporated |
  reverted |
  unsettled

evidence_refs: []
receipt_ref:
```

Observed code never automatically updates canonical design.

Semantic, authority, public-contract, state-ownership, and scope changes require governed revision and any higher-level amendment required by existing authorities.

---

## 53. Legacy CallStackDesign migration

Existing `CallStackDesign` records MUST be retained as:

```text
legacy_call_stack_projection
profile = scaffold
canonical_program_design = false
closure_eligible = false
```

Existing IDs referenced by releases, Workpoints, Evidence, or specifications MUST remain resolvable.

A call stack remains one useful projection:

```text
entry point
→ handlers
→ services
→ adapters
→ persistence
→ output
```

It is not the full Program Design.

A generic generated scaffold MUST return a truthful status such as:

```text
status = scaffold_generated
canonical = false
closure_eligible = false
required_next = program_design_ground
```

---

## 54. Machine documentation

Focusa MUST automatically maintain:

- Program Design Graph;
- Observed Program Graph;
- state-ownership map;
- operation catalog;
- capability matrix;
- operation-to-tool map;
- operation-to-surface map;
- Workset bindings;
- Workpoint slices;
- authority map;
- failure/recovery catalog;
- verification map;
- vertical composition manifest;
- conformance projection;
- decision history;
- friction findings;
- capability readiness;
- cross-surface parity.

Human-readable projections derive from those canonical records.

Every projection identifies:

- source revision;
- observed Git revision;
- generation time;
- freshness;
- scope;
- conformance;
- known gaps.

The user MUST NOT maintain duplicate human and machine architecture documentation.

---

## 55. Privacy and capability supply chain

Spec 151 MUST govern:

- source sensitivity;
- private URLs;
- browser cookies and storage;
- credentials;
- customer data;
- local-only design details;
- sanitized exports;
- signed capability descriptors;
- signed vertical packs;
- adapter provenance;
- artifact verification;
- version pinning;
- revocation;
- quarantine;
- rollback.

Dynamic capabilities are untrusted until validated.

Secrets never enter prompts, design documents, logs, receipts, or generated documentation.

Capability installation and update MUST preserve project, user, pairing, evidence, and configuration state by default.

---

## 56. Accessibility and modality parity

All important actions MUST be available through applicable:

- keyboard;
- screen reader;
- CLI;
- TUI;
- nonvisual structured output;
- high contrast;
- reduced motion;
- low-bandwidth operation;
- future voice and mobile surfaces.

Color, animation, visual graphs, and drag interactions cannot be the only means of understanding or controlling state.

Generated UI MUST bind the same canonical operations as headless surfaces.

---

## 57. Team and multi-operator coordination

Program Design MUST support:

- decision ownership;
- role-based detail;
- delegated approvals;
- approval quorum where required;
- timezone-aware temporal context;
- handoff of unresolved decisions;
- deduplicated notifications;
- one canonical decision state;
- explicit blocker owner.

The same question MUST NOT be asked independently of multiple operators unless separate approval is required.

---

## 58. Temporal and predictive integration

Program Design supplies structural features:

- node count;
- edge count;
- critical-path depth;
- fan-in/fan-out;
- external dependencies;
- state stores;
- migrations;
- authority boundaries;
- failure paths;
- verifier count;
- cross-repository count;
- parallelizable subgraphs;
- high-consequence nodes;
- capability acquisition risk;
- friction risk;
- unknown-impact count.

Specs 137/138 own estimates, uncertainty, outcomes, calibration, and learning.

Forecasts MUST account for:

- capability acquisition;
- UIAI availability;
- evaluation feature limits;
- authentication;
- model/provider/network regime;
- multi-agent coordination;
- friction;
- rework;
- verification;
- human attention;
- productive parallel work.

Only evidence-backed settled outcomes may train promoted estimation or routing behavior.

---

## 59. Experience outcome measurement

Required metrics include:

```text
time to first useful action
time to verified outcome
avoidable interruption rate
operator correction rate
unexpected-action rate
undo rate
abandonment rate
repeated-friction rate
automatic-resume success
cross-surface duplication rate
user-visible surprise rate
ETA calibration
rework
quality-adjusted time saved
```

Fewer questions alone do not prove a better experience.

Experience forecasts and outcomes MUST use the Spec 138 epistemic substrate rather than a separate learning authority.

---

## 60. Quiet success

Program Design SHOULD remain silent when:

- design is aligned;
- capabilities are healthy;
- Workset bindings are current;
- documentation regenerated normally;
- recovery completed automatically;
- no material decision is required.

Surface only:

- meaningful changes;
- risks;
- required decisions;
- failed recovery;
- consequential automatic actions;
- material ETA changes;
- permission or privacy changes.

Quiet success MUST NOT conceal material automation or changed canonical state.

---

## 61. Canonical operation families

### Program Design

```text
program_design.create
program_design.ground
program_design.observe
program_design.challenge
program_design.approve
program_design.revise
program_design.supersede
program_design.slice.create
program_design.conformance.verify
program_design.drift.explain
program_design.deviation.propose
program_design.deviation.decide
program_design.settle
```

### Capability fabric

```text
capability.inspect
capability.resolve
capability.readiness.plan
capability.acquire.preview
capability.acquire.authorize
capability.acquire.execute
capability.acquire.verify
capability.repair
capability.update
capability.rollback
capability.resume_dependents
```

### UIAI

```text
uiai.companion.inspect
uiai.companion.offer
uiai.companion.install.preview
uiai.companion.install
uiai.evaluation.activate
uiai.pair
uiai.health.verify
uiai.capabilities.verify
uiai.repair
uiai.update
uiai.rollback
uiai.resume_browser_work
```

### Experience

```text
friction.detect
friction.classify
friction.remediate
friction.verify
friction.report
attention.route
decision.capsule.get
execution.simulate
execution.undo
```

Exact routes, CLI commands, Pi tools, MCP tools, OpenAI functions, generated-client methods, and UI actions derive from the Operation Registry.

---

## 62. Invocation receipts

Every consequential invocation SHOULD bind:

```yaml
schema: focusa.capability_invocation_receipt.v1

invocation_id:
operation_ref:
tool_ref:
surface_ref:
adapter_ref:
environment_ref:

project_ref:
continuity_id:
workset_ref:
member_ref:
workpoint_ref:
program_design_ref:
program_design_revision:
program_element_ref:

input_digest:
authority_ref:
confirmation_ref:
idempotency_key:

started_at:
completed_at:
status:
  succeeded |
  failed |
  indeterminate |
  rolled_back |
  cancelled

result_ref:
postcondition_refs: []
evidence_refs: []
recovery_ref:
receipt_hash:
```

A successful tool invocation does not establish successful operation completion. Postconditions and evidence do.

---

## 63. Machine-readable artifacts

Required before production implementation:

```text
docs/contracts/spec151-normative-source-coverage.v1.yaml
docs/contracts/spec151-complete-feature-ledger.v1.yaml
docs/contracts/spec151-delivery-dag.v1.yaml
docs/contracts/spec151-primitive-ownership-matrix.v1.yaml

docs/contracts/spec151-program-design.schema.v1.json
docs/contracts/spec151-event-payloads.schema.v1.json
docs/contracts/spec151-operation-contracts.v1.yaml
docs/contracts/spec151-openapi.v1.yaml

docs/contracts/spec151-rdf-mapping.v1.yaml
docs/contracts/spec151-shacl-shapes.v1.ttl
docs/contracts/spec151-vertical-profile.schema.v1.json

docs/contracts/spec151-workset-binding.schema.v1.json
docs/contracts/spec151-capability-fabric.schema.v1.json
docs/contracts/spec151-uiai-profile.schema.v1.json
docs/contracts/spec151-friction-runtime.schema.v1.json

docs/contracts/spec151-observation-adapter-registry.v1.yaml
docs/contracts/spec151-conformance-level-matrix.v1.yaml
docs/contracts/spec151-client-parity-matrix.v1.yaml
docs/contracts/spec151-migration-matrix.v1.yaml
docs/contracts/spec151-proof-matrix.v1.yaml
docs/contracts/spec151-reference-vertical-suite.v1.yaml
docs/contracts/spec151-forbidden-placeholder-audit.v1.yaml
```

Every normative clause, schema field, lifecycle state, operation family, acceptance criterion, and closure blocker MUST receive stable requirement coverage.

---

## 64. Implementation phases

### P0 — Canonicalization

- persist this consolidated spec;
- resolve the duplicate Spec 149 numbering separately;
- generate source coverage;
- generate the complete requirement ledger;
- inventory every CallStackDesign reference;
- prohibit superseded Spec 151 wording.

### P1 — Core runtime

- Program Design types;
- revision lifecycle;
- reducer events;
- persistence;
- runtime state;
- scope and digest validation.

### P2 — Semantic and vertical integration

- obligation bindings;
- RDF projection;
- SHACL shapes;
- vertical profile registry;
- reference vertical suites.

### P3 — Capability fabric

- operation bindings;
- execution routes;
- surface descriptors;
- agent capability packets;
- progressive disclosure.

### P4 — UIAI exclusive browser

- companion discovery;
- perpetual evaluation profile;
- install/repair/update/pair;
- Workset suspension;
- productive parallelism;
- automatic resume.

### P5 — Observed reality and conformance

- Rust adapter;
- TypeScript/JavaScript adapter;
- schema and operation adapters;
- Levels 0–7;
- bidirectional drift.

### P6 — Workset and Workpoint

- WorksetDesignBinding;
- readiness gates;
- design slices;
- capability readiness;
- compaction and handoff.

### P7 — Multi-agent and cross-surface continuity

- execution leases;
- overlap detection;
- global interaction IDs;
- portable approval;
- deduplicated attention.

### P8 — Experience integrity

- friction runtime;
- budgets;
- simulations;
- undo;
- Decision Capsules;
- accessibility.

### P9 — Forecasting and learning

- structural features;
- experience forecasts;
- calibration;
- friction improvement candidates;
- champion/challenger remediation.

### P10 — Full conformance

- all vertical reference suites;
- all surfaces;
- restart and recovery;
- experience regression proof;
- closure Receipt.

---

## 65. Mandatory reference suites

Full cross-vertical conformance requires:

1. software implementation;
2. professional workflow;
3. research pipeline;
4. regulated workflow;
5. physical or robotic actuation;
6. cross-vertical Workset with at least three profiles.

Each suite MUST prove:

- design grounding;
- vertical activation;
- semantic validation;
- Workset binding;
- Workpoint slicing;
- capability resolution;
- runtime admission;
- execution;
- observed reality;
- drift detection;
- verification;
- settlement;
- compaction;
- restart;
- friction measurement.

---

## 66. Mandatory test families

Tests MUST include at least:

1. scaffold cannot satisfy canonical design;
2. requirement without design binding;
3. design without implementation;
4. implementation without design;
5. multiple state writers;
6. missing recovery;
7. stale Workset binding;
8. stale Workpoint slice;
9. Program Design profile promotion;
10. brownfield advisory mode;
11. UIAI absent;
12. UIAI perpetual evaluation install;
13. evaluation feature limitation;
14. UIAI repair and update;
15. no alternate browser;
16. WebMCP cross-origin rejection;
17. productive parallel work;
18. automatic resume;
19. duplicate mutation prevention;
20. capability discovered too late;
21. proactive readiness;
22. approval lease;
23. attention batching;
24. cross-surface approval;
25. multi-agent collision;
26. stale-agent fencing;
27. simulation;
28. undo;
29. screen-reader operation;
30. low-bandwidth mode;
31. repeated friction detection;
32. friction remediation learning;
33. experience regression detection;
34. restart restores design, Workset, capability, and suspension state;
35. a browser action succeeds but its postcondition fails;
36. a dynamic capability is available but untrusted;
37. a material design revision occurs during capability acquisition;
38. another agent completes the suspended operation before resume;
39. a Workset completes only after required conformance and settlement;
40. no accepted obligation silently leaves the delivery graph.

---

## 67. Acceptance criteria

Spec 151 is accepted only when:

1. One consolidated canonical source exists.
2. Program Design is a typed runtime primitive.
3. Legacy CallStackDesign IDs remain resolvable.
4. Generic scaffolds cannot satisfy design conformance.
5. Requirements map to design.
6. Design maps to operations.
7. Operations map to governed capability routes.
8. Worksets bind exact design revisions.
9. Workpoints receive bounded slices.
10. Agents receive bounded capability packets.
11. UIAI Engine is the sole Focusa agent browser.
12. UIAI evaluation is perpetual and feature-limited.
13. UIAI absence triggers assisted acquisition, not a dead end.
14. No silent installation occurs.
15. Independent work continues during recovery.
16. Suspended work resumes automatically.
17. Dynamic capabilities are validated.
18. Multi-agent collisions are prevented.
19. Cross-surface decisions are deduplicated.
20. Protective friction is distinguished from avoidable friction.
21. Program Design tax stays within profile budgets.
22. Machine documentation maintains itself.
23. Brownfield projects can adopt progressively.
24. Accessibility parity is verified.
25. Team decision ownership is explicit.
26. Experience metrics are collected.
27. Friction regressions block conformance.
28. No user manually maintains design or capability infrastructure.
29. No silent material automation occurs.
30. No accepted obligation disappears.
31. Every consequential invocation has an evidence and recovery path.
32. Every required surface preserves canonical authority semantics.
33. Every active vertical composition has zero unresolved unknown impact for affected consequential work.
34. Every capability suspension has an exact resume path.
35. Every material automatic decision is explainable and reversible where the underlying operation is reversible.

---

## 68. Closure blockers

Spec 151 cannot close while:

- superseded wording remains active;
- any mandatory requirement is unmapped;
- generic scaffolds can claim completion;
- any required Workset member lacks a current binding;
- any active Workpoint uses a stale design;
- any consequential operation lacks an executable route;
- any UIAI-dependent flow permits an alternate browser;
- UIAI evaluation expires;
- a missing capability produces a dead end;
- independent work is unnecessarily blocked;
- any external mutation can be duplicated on resume;
- any material automation lacks a Decision Capsule;
- any multi-agent overlap is unresolved;
- any required surface has weaker authority semantics;
- any vertical reference suite fails;
- any accessibility boundary lacks equivalent operation;
- any recurring avoidable friction remains untracked;
- experience metrics regress beyond the approved budget;
- unknown semantic impact remains;
- any active design deviation remains unsettled;
- any source-string-only finding is presented as full conformance;
- any capability advertised to the agent is unavailable without truthful degraded posture;
- any capability installation bypasses required approval or standing policy.

---

## 69. Final closure report

```yaml
spec151_closure:
  source_hash:
  total_requirements:
  verified_requirements:
  unmapped_requirements: 0

  program_design_runtime: pass
  semantic_projection: pass
  vertical_reference_suites: pass
  workset_binding: pass
  workpoint_slicing: pass
  capability_fabric: pass

  uiai_exclusive_browser: pass
  uiai_perpetual_evaluation: pass
  uiai_install_and_recovery: pass
  alternate_browser_routes: 0

  multi_agent_coordination: pass
  cross_surface_continuity: pass
  accessibility_parity: pass
  brownfield_adoption: pass

  design_node_coverage: 1.0
  design_edge_coverage: 1.0
  state_ownership_coverage: 1.0
  failure_path_coverage: 1.0
  verification_binding_coverage: 1.0
  capability_route_coverage: 1.0

  stale_workset_bindings: 0
  stale_workpoints: 0
  unresolved_design_deviations: 0
  unknown_semantic_impacts: 0
  duplicate_external_mutations: 0

  silent_failures: 0
  dead_end_recovery_states: 0
  repeated_project_questions: 0
  manual_tool_mappings: 0
  manual_workset_design_bindings: 0
  manual_continuation_reconstruction: 0

  automatic_resume_success_rate:
  avoidable_interruption_rate:
  operator_correction_rate:
  unexpected_action_rate:
  experience_regression_gate: pass

  evidence_refs: []
  receipt_refs: []
```

---

## 70. Final invariant

```text
PROGRAM DESIGN IS THE INFRASTRUCTURE THAT ALLOWS THE AGENT
TO UNDERSTAND, BUILD, OPERATE, VERIFY, RECOVER, AND IMPROVE
THE USER’S SCOPED WORK.

THE USER PROVIDES PURPOSE, JUDGMENT, DOMAIN TRUTH, AND AUTHORITY.

FOCUSA CARRIES THE STRUCTURE.
THE AGENT CARRIES THE IMPLEMENTATION BURDEN.
THE WORKSET CARRIES THE GOVERNED FLOW.
THE WORKPOINT CARRIES THE NEXT ACTION.
THE CAPABILITY FABRIC CARRIES THE EXECUTION ROUTE.
UIAI ENGINE CARRIES ALL FOCUSA AGENT BROWSER PERCEPTION AND ACTUATION.
THE TEMPORAL AND EPISTEMIC SUBSTRATES CARRY FORECASTS AND LEARNING.
EVIDENCE AND SETTLEMENT CARRY COMPLETION TRUTH.

NO OBLIGATION DISAPPEARS.
NO DESIGN REMAINS A GENERIC SCAFFOLD.
NO AGENT GUESSES WHICH TOOL TO USE.
NO USER MAINTAINS THE AGENT’S INFRASTRUCTURE.
NO MISSING CAPABILITY BECOMES A DEAD END.
NO BLOCKED STEP STOPS UNRELATED PRODUCTIVE WORK.
NO MATERIAL AUTOMATION REMAINS INVISIBLE.
NO AVOIDABLE FRICTION IS NORMALIZED.
```
