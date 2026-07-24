# Spec 136 — Governed Proposal-to-Settlement Protocol and Outcome Truth Infrastructure

**Status:** Proposed / post-Spec-135 implementation-ready specification  
**Owner:** Focusa / Verious Smith  
**Proposed path:** `docs/136-governed-proposal-to-settlement-protocol-and-outcome-truth-infrastructure-spec.md`  
**Implementation start condition:** Full Spec 135 series implementation and closure, including the frozen 135–135K Delivery Contract, permanent dogfood traversal, required Evidence, Receipts, client parity, generated UI, UIAI Engine Eval proof, restart/resume proof, and closure-ledger completion  
**Scope:** Focusa core, reducer, daemon, persistence, API, Operation Registry, generated contracts, C.R.I.S.T., Project Genesis, ontology registry, domain packs, Proposal Resolution Engine, Secondary Cognition, Context Cognition, Context Authority, Workpoint, Trajectory, continuous Work Loop, Silent Sessions, UIAI Engine integration, provider adapters, work-item closure, Evidence, Receipts, Eval Ledger, public-safe proof projections, generated UI, recovery, migration, conformance, and post-settlement learning  
**Relationship to Spec 135:** Independent successor specification. It MUST NOT create Spec 135L, reopen frozen Spec 135 framework decisions, delay completion of Spec 135, or substitute for any unmet Spec 135 requirement.

---

## 0. One-line definition

Focusa SHALL provide one domain-neutral, reducer-governed protocol that turns probabilistic observations and proposals into canonical decisions, authorized execution, reconciled external outcomes, verified completion, settled Receipts, and safely promotable learning without allowing models, clients, connectors, executors, or projections to mint operational truth.

---

## 1. Executive directive

Focusa MUST preserve these distinctions throughout every consequential workflow:

```text
proposed     ≠ verified
verified     ≠ resolved
resolved     ≠ canonical
canonical    ≠ authorized
authorized   ≠ dispatched
dispatched   ≠ executed
executed     ≠ reconciled
reconciled   ≠ outcome-verified
outcome-verified ≠ complete
complete     ≠ settled
settled      ≠ permanently learned
```

No component MAY collapse these stages merely because:

- an LLM expressed confidence;
- a function returned success;
- a process exited with code `0`;
- a browser action appeared to work;
- an external provider returned `200`;
- a task manager shows `done`;
- a screenshot exists;
- a generated UI displays a success state;
- an executor wrote a final message;
- a single verification model agreed;
- a previous similar action succeeded;
- a projection, summary, or transcript states that the work is complete.

The canonical Focusa protocol is:

```text
Observe
→ Propose
→ Verify
→ Resolve
→ Commit canonical intent/state
→ Evaluate authority
→ Record durable execution intent
→ Dispatch
→ Observe execution
→ Reconcile external reality
→ Verify outcome
→ Evaluate completion
→ Settle
→ Commit Receipt
→ Consider bounded learning
```

---

## 2. Why this specification exists

Focusa already defines strong but distributed components:

- bounded Secondary Cognition and persistent autonomy;
- candidate and canonical ontology state;
- deterministic proposal resolution;
- a single-writer reducer;
- Workpoint continuity;
- Context Cognition and Context Authority;
- typed operations and generated action bindings;
- daemon-governed continuous work;
- durable Silent Sessions;
- UIAI Engine execution and proof;
- provider-neutral closure authority;
- Evidence and Receipts;
- Eval Ledger and benchmark infrastructure;
- generated C.R.I.S.T. UI;
- domain packs and verification/promotion policies.

The remaining systemic gap is not another subsystem. It is the absence of one enforceable cross-system lifecycle joining those components.

Without this protocol, individually correct components can still produce an incorrect system outcome:

- Secondary Cognition can propose a useful delta but its promotion path can differ by feature.
- PRE can resolve a proposal without one normalized downstream execution identity.
- The reducer can commit state while side-effect dispatch is lost after a crash.
- An executor can mutate an external system and time out before recording success.
- Retry logic can repeat an operation that already succeeded remotely.
- UIAI Engine can produce an artifact without a complete mission-to-action lineage.
- a Workpoint can appear complete before external reality is reconciled.
- a Receipt can summarize proof without preserving every causal protocol reference.
- a failed or successful run can become a learning candidate without a governed promotion boundary.
- C.R.I.S.T., ontology actions, generated UI, and adversarial spec planning can each describe guardrails differently.

Spec 136 defines the shared protocol, state transitions, records, reason codes, transaction boundaries, policy references, progressive implementation order, and conformance requirements that prevent those seams from becoming alternate truth paths.

---

## 3. Normative basis

This specification composes and preserves, rather than replaces, at minimum:

- `docs/core-reducer.md`
- `docs/41-proposal-resolution-engine.md`
- `docs/54a-operator-priority-and-subject-preservation.md`
- `docs/54b-context-injection-and-attention-routing.md`
- `docs/56-trace-checkpoints-recovery.md`
- `docs/57-golden-tasks-and-evals.md`
- `docs/61-domain-general-cognition-core.md`
- `docs/66-affordance-and-execution-environment-ontology.md`
- `docs/67-query-scope-and-relevance-control.md`
- `docs/68-current-ask-and-scope-integration.md`
- `docs/69-scope-failure-and-relevance-tracing.md`
- `docs/70-shared-interfaces-statuses-and-lifecycle.md`
- `docs/71-governing-priors-and-scalar-weights.md`
- `docs/72-agent-identity-role-and-self-model-ontology.md`
- `docs/73-intention-commitment-and-self-regulation.md`
- `docs/74-identity-and-reference-resolution.md`
- `docs/75-projection-and-view-semantics.md`
- `docs/76-retention-forgetting-and-decay-policy.md`
- `docs/77-ontology-governance-versioning-and-migration.md`
- `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`
- `docs/79-focusa-governed-continuous-work-loop.md`
- `docs/88-ontology-backed-workpoint-continuity.md`
- `docs/100-context-cognition-spec.md`
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`
- `docs/109-agent-first-api-redesign-ax-spec.md`
- `docs/113-agent-benchmark-spec.md`
- `docs/116-provider-neutral-work-item-closure-authority-spec.md`
- `docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md`
- `docs/120-adversarial-spec-workbench-and-operator-approval-gates.md`
- `docs/125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md`
- `docs/130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md`
- `docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md`
- Spec 135 master and the frozen 135A–135K series
- `docs/135-series-current-manifest.md`
- `docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md`
- `docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md`

When this specification appears to conflict with a primitive-owning specification, the primitive-owning specification retains ownership of its internal domain semantics. Spec 136 owns the cross-system proposal-to-settlement protocol and conformance boundary.

---

## 4. Post-Spec-135 activation gate

### 4.1 No preemption of Spec 135

Spec 136 implementation MUST NOT begin until the authoritative Spec 135 Delivery Contract is closed with actual evidence.

Documentation, design discussion, schema drafts, implementation estimates, and non-mutating prototypes MAY occur before that point. Production implementation tasks MUST remain blocked.

### 4.2 Required activation evidence

The Spec 136 start gate MUST verify:

- every Spec 135 feature-ledger requirement is closed;
- the permanent C.R.I.S.T. dogfood traversal passes;
- generated UI uses live canonical state rather than hidden mock state;
- every browser-facing proof uses UIAI Engine Eval;
- Operation Registry generation and typed client parity pass;
- the durable native event stream replays after disconnect and restart;
- Workpoint, Evidence, Receipt, C.R.I.S.T., Mission Canvas, and exact-state resume operate end to end;
- all declared domain packs operate through one canonical runtime;
- authority, scope, capability, permission, recovery, and receipt requirements pass;
- no required behavior exists only in prose, static cards, mocks, or CLI-only paths;
- release proof includes a Spec 135 completion Receipt and closure certificate.

### 4.3 Activation record

Starting Spec 136 requires a canonical record:

```yaml
schema: focusa.spec136_activation.v1
activation_id:
spec135_completion_receipt_ref:
spec135_feature_ledger_ref:
spec135_proof_matrix_ref:
spec135_dogfood_evidence_refs: []
focusa_version:
ontology_registry_version:
operation_registry_version:
receipt_schema_version:
activated_by:
activated_at:
status: eligible | blocked
block_reason_codes: []
```

No agent may infer eligibility from a version number or document status alone.

---

## 5. Scope

Spec 136 governs consequential state and outcome transitions involving one or more of:

- candidate cognition or semantic state;
- canonical state promotion;
- operator or delegated decisions;
- Workpoint action intent;
- ontology actions;
- generated C.R.I.S.T. actions;
- provider mutations;
- browser or document actions;
- filesystem or repository mutations;
- continuous or autonomous execution;
- work-item creation, update, or closure;
- deployment, release, migration, configuration, credentials, or live-service effects;
- outcome verification;
- completion and settlement;
- durable learning promotion.

Spec 136 also defines a fast path for read-only and non-consequential operations so system awareness does not impose unnecessary ceremony.

---

## 6. Non-goals

Spec 136 is not:

- a new model or agent framework;
- a second reducer;
- a second event store;
- a second ontology registry;
- a second Operation Registry;
- a second permission system;
- a second receipt ledger;
- a generic workflow engine replacing Focusa’s daemon and Work Loop;
- an attempt to provide universal distributed ACID transactions;
- a replacement for Workpoint, Trajectory, Context Cognition, Context Authority, Evidence, PRE, Spec 78, Spec 116, Spec 119, Spec 120, Spec 133, or UIAI Engine;
- a requirement that every harmless read invoke an LLM verifier;
- permission for Secondary Cognition to authorize or dispatch actions;
- permission for clients or generated UI to reproduce reducer rules;
- permission for external providers to become Focusa’s completion authority;
- permission to call compensation “undo” when full reversal cannot be guaranteed;
- a guarantee of exactly-once external execution;
- a reason to delay Spec 135;
- a reason to rewrite working post-135 infrastructure before a progressive slice proves value.

---

## 7. Core authority model

### 7.1 Authority chain

```text
Hard system and safety law
→ operator’s newest explicit instruction
→ approved governing specs and governing priors
→ verified ProjectIdentity + continuity scope
→ CurrentAsk + QueryScope
→ canonical Workpoint / Trajectory / ontology revision
→ registered policy and authority decisions
→ daemon-governed execution
→ independently inspectable evidence
→ completion and settlement authority
```

### 7.2 Responsibility table

| Component | May observe | May propose | May verify | May resolve | May commit canonical state | May authorize action | May execute | May settle completion |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Operator | Yes | Yes | Yes | Yes through approved paths | Yes through reducer-backed operations | Yes within authority | Through operations | Yes where policy assigns |
| Secondary Cognition | Yes | Yes | Yes, advisory | No | No | No | No | No |
| PRE | Reads | Receives | Uses | Yes for registered proposal classes | No; submits reducer command | No | No | No |
| Reducer | Reads event | No model proposal | Enforces deterministic invariants | Applies accepted resolution | Yes | Records decision facts | No | Records settlement facts |
| Context Authority / policy services | Reads | May recommend | Yes | No canonical cognition resolution | No | Yes, bounded | No | May block |
| Daemon / Work Loop | Reads | May emit operational proposals | Operational checks | No cognitive truth resolution | Applies reducer commands only | Enforces recorded authority | Coordinates | Coordinates evaluation |
| Runner / harness adapter | Observes runtime | May report | Reports observations | No | No | No | Yes within intent | No |
| UIAI Engine | Observes browser/document world | May return findings | Produces bounded proof | No | No | No | Owns browser-facing execution | No |
| Provider adapter | Observes provider | No cognitive proposal | Reconciles provider state | No | No | Uses supplied authorization | Executes adapter mutation | No |
| Completion evaluator | Reads proof | May object | Yes | No | No | No | No | Recommends/blocks per policy |
| Receipt service | Reads lineage | No | Verifies completeness | No | Commits Receipt event through reducer path | No | No | Records settled result |
| Generated UI / C.R.I.S.T. | Displays | Captures operator proposal | Displays verification | No | No | No | Invokes registered operations | No |

### 7.3 No authority by fluency

Model capability, role expertise, repeated success, long execution duration, or access to broad context MUST NOT implicitly grant:

- canonical mutation authority;
- policy authorship;
- permission escalation;
- approval authority;
- completion authority;
- learning-promotion authority.

---

## 8. Core design laws

1. **One canonical state authority.** Every canonical Focusa mutation remains reducer-expressed.
2. **One native event history.** Protocol events use the existing SQLite canonical event history and hash chain.
3. **Proposals are not facts.** Candidate material remains structurally and visibly non-canonical.
4. **Verification is typed.** Verification has policy identity, verifier identity, evidence, freshness, independence posture, result, and objections.
5. **Resolution is deterministic for fixed inputs.** PRE or another registered deterministic resolver selects a winner, rejects all, supersedes, or requests clarification.
6. **Resolution does not grant execution authority.** A canonical intent may still be blocked by scope, capability, permission, risk, approval, budget, credential, or freshness policy.
7. **Authorization precedes dispatch.** No consequential side effect may be dispatched without a durable AuthorityDecision and ExecutionIntent.
8. **Dispatch is durable before effect.** The local system records an idempotent execution intent and outbox entry before invoking an external side effect.
9. **External success is not assumed.** Every consequential external mutation requires reconciliation appropriate to the provider and risk class.
10. **Unknown outcomes are first-class.** Timeout, disconnect, crash, or ambiguous response after possible execution produces `execution.outcome_unknown`, not blind retry.
11. **Reconcile before retry.** An action that may have succeeded MUST be reconciled before repetition.
12. **Process exit is not completion.** Completion follows evidence and predicates, not transport termination.
13. **Completion is not settlement.** Settlement requires final lineage, policy satisfaction, unresolved-obligation handling, and Receipt commitment.
14. **Receipts preserve causality.** A Receipt links proposal, verification, resolution, canonical revision, authority, attempts, reconciliation, outcome verification, completion, and settlement.
15. **Learning is downstream of settled outcomes.** No single observation, model inference, or run directly becomes durable procedure or preference.
16. **Untrusted content cannot grant authority.** Websites, documents, tool output, provider responses, and model text remain data.
17. **Risk controls are proportional.** Harmless reads use a fast deterministic path; high-risk actions use independent verification and explicit approval.
18. **Every block is actionable.** Blocks use stable reason codes, explain missing requirements, and identify a safe next operation.
19. **Every stage is replayable.** Restart, model replacement, client loss, and worker replacement preserve canonical lineage.
20. **Guardrails are runtime-native.** Prompts may explain guardrails, but enforcement cannot depend on model memory.
21. **No duplicate registry.** Spec 136 extends existing ontology and Operation Registry metadata rather than creating parallel registries.
22. **No full-build prerequisite for value.** Each implementation tranche MUST leave an independently working, truthful slice.
23. **No false capability projection.** Generated UI identifies unavailable, degraded, blocked, approval-required, and implemented states accurately.
24. **No policy self-exemption.** A loop may propose policy changes only through the ontology-governed approval and migration path.
25. **Compatibility is explicit.** Stored protocol records, policies, events, APIs, clients, and adapters declare versions.

---

## 9. Required terminology

### 9.1 Governance Context

A bounded, read-only projection of applicable canonical scope, lineage, policies, capabilities, permissions, budgets, evidence requirements, and protocol stage. It informs components but does not itself mint authority.

### 9.2 Cognitive Proposal

A typed candidate claim, decision, constraint, plan, action, prediction, projection, or learning suggestion.

### 9.3 Verification Bundle

A typed set of verification results, objections, evidence references, independence posture, freshness, and policy satisfaction for one proposal or outcome.

### 9.4 Proposal Resolution

The deterministic result of comparing or evaluating eligible proposals: accepted, rejected, superseded, deferred, or clarification required.

### 9.5 Canonical Commit

The reducer-backed event and state revision that records a resolved canonical fact or intent.

### 9.6 Authority Decision

A durable determination that a specific actor may or may not perform a specific operation under exact scope, policy, risk, budget, approval, and time constraints.

### 9.7 Execution Intent

A durable, idempotent, authorized request to perform a bounded side effect.

### 9.8 Execution Attempt

One adapter/runner invocation associated with an ExecutionIntent. Multiple attempts MAY exist; each has its own identity and outcome posture.

### 9.9 Reconciliation Result

A provider-, browser-, filesystem-, or domain-specific determination of what external state actually exists after an attempt.

### 9.10 Outcome Verification

Evidence-backed evaluation that the reconciled effect satisfies the intended operation semantics.

### 9.11 Completion Decision

Evaluation of whether declared mission, Workpoint, task, or operation completion predicates are satisfied.

### 9.12 Settlement

The protocol stage at which the system records the final accepted outcome, unresolved obligations, evidence sufficiency, completion posture, and durable Receipt lineage. “Settlement” is domain-neutral and does not imply financial settlement.

### 9.13 Learning Candidate

A post-outcome proposal to modify procedural memory, preference memory, routing, verification policy, domain-pack policy, eval coverage, or another governed learning target.

---

## 10. Canonical protocol lifecycle

### 10.1 Primary states

```text
observed
proposed
verifying
verified
resolution_pending
resolved
canonical_committed
authority_pending
authorized
dispatch_pending
executing
execution_observed
reconciliation_pending
reconciled
outcome_verifying
outcome_verified
completion_pending
complete
settlement_pending
settled
receipt_committed
```

### 10.2 Alternate and terminal states

```text
rejected
superseded
deferred
clarification_required
verification_blocked
authority_blocked
approval_required
expired
cancelled
failed
outcome_unknown
diverged
partial
compensation_required
compensating
compensated
dead_letter
not_complete
operator_review
settlement_blocked
archived
```

### 10.3 Required transition map

```text
observed → proposed

proposed → verifying
proposed → rejected
proposed → superseded
proposed → deferred

verifying → verified
verifying → verification_blocked
verifying → rejected
verifying → deferred

verified → resolution_pending

resolution_pending → resolved
resolution_pending → rejected
resolution_pending → superseded
resolution_pending → clarification_required
resolution_pending → deferred

resolved → canonical_committed
resolved → expired

canonical_committed → authority_pending
canonical_committed → complete
  only for non-executing canonical cognition transitions whose completion policy permits

authority_pending → authorized
authority_pending → authority_blocked
authority_pending → approval_required
authority_pending → expired

approval_required → authority_pending
approval_required → cancelled
approval_required → expired

authorized → dispatch_pending
authorized → cancelled
authorized → expired

dispatch_pending → executing
dispatch_pending → failed
dispatch_pending → cancelled

executing → execution_observed
executing → outcome_unknown
executing → failed
executing → cancelled
executing → compensation_required

execution_observed → reconciliation_pending

outcome_unknown → reconciliation_pending
outcome_unknown → dead_letter
  only after reconciliation policy is exhausted

reconciliation_pending → reconciled
reconciliation_pending → diverged
reconciliation_pending → partial
reconciliation_pending → outcome_unknown
reconciliation_pending → dead_letter

diverged → compensation_required
diverged → operator_review
diverged → dead_letter

partial → outcome_verifying
partial → compensation_required
partial → operator_review

reconciled → outcome_verifying

outcome_verifying → outcome_verified
outcome_verifying → verification_blocked
outcome_verifying → compensation_required
outcome_verifying → operator_review

outcome_verified → completion_pending

completion_pending → complete
completion_pending → not_complete
completion_pending → operator_review
completion_pending → settlement_blocked

complete → settlement_pending

settlement_pending → settled
settlement_pending → settlement_blocked
settlement_pending → operator_review

settled → receipt_committed

receipt_committed → archived
receipt_committed → proposed
  only for a new LearningCandidate, never by rewriting the settled record
```

### 10.4 Transition law

Every transition MUST identify:

- transition event ID;
- prior state;
- next state;
- project and continuity scope;
- active Workpoint and revision when applicable;
- actor;
- policy references;
- reason code;
- evidence references;
- correlation and causation IDs;
- canonical state revision;
- timestamp;
- schema version.

A component MUST NOT skip a required stage by emitting a later-stage status directly.

---

## 11. Governance Context contract

```yaml
schema: focusa.governance_context.v1
context_id:
generated_at:
expires_at:
source_state_revision:
ontology_registry_version:
operation_registry_version:
policy_bundle_version:

scope:
  project_root:
  project_identity_ref:
  continuity_id:
  workstream_id:
  attachment_id:
  work_surface_id:
  current_ask_id:
  query_scope_id:
  relevant_context_set_ref:
  excluded_context_set_ref:
  workpoint_id:
  workpoint_revision:
  trajectory_ref:

lineage:
  proposal_id:
  verification_bundle_id:
  resolution_id:
  canonical_commit_event_id:
  execution_intent_id:
  correlation_id:
  causation_id:
  trace_id:

actor:
  actor_id:
  actor_instance_id:
  role_profile_id:
  capability_profile_id:
  permission_profile_id:
  delegation_ref:
  trust_posture:

operation:
  operation_id:
  ontology_action_type_id:
  mode: read | preview | commit
  side_effect_class:
  risk_class:
  reversibility_kind:
  protocol_stage:

authority:
  status: not_required | pending | authorized | blocked | approval_required | expired
  authority_decision_id:
  capability_refs: []
  permission_refs: []
  approval_refs: []
  missing_refs: []

budgets:
  financial:
  tokens:
  model_calls:
  browser_minutes:
  worker_runtime:
  network:
  retries:
  human_attention:
  wall_clock:
  remaining_summary:

policy:
  verification_policy_ref:
  promotion_policy_ref:
  authority_policy_ref:
  approval_policy_ref:
  idempotency_policy_ref:
  retry_policy_ref:
  reconciliation_policy_ref:
  compensation_policy_ref:
  completion_policy_ref:
  settlement_policy_ref:
  receipt_policy_ref:
  retention_policy_ref:

evidence:
  required_classes: []
  current_refs: []
  missing_classes: []
  freshness_status:
  contradiction_status:

projection:
  purpose:
  included_fields: []
  excluded_fields: []
  redaction_profile_ref:
```

### 11.1 Context laws

- The envelope is a projection, not a second state store.
- Every field MUST be derivable from canonical state, registered policy, or durable protocol records.
- Components receive only the minimum applicable projection.
- Credentials and secret values MUST NOT appear.
- Expired Governance Context MUST NOT authorize continued execution.
- New operator steering invalidates context when it changes CurrentAsk, scope, authority, or active intent.
- A client MAY cache the envelope only until `expires_at` or source revision invalidation.

---

## 12. Core record contracts

### 12.1 Cognitive Proposal

```yaml
schema: focusa.cognitive_proposal.v1
proposal_id:
proposal_kind: belief | decision | constraint | plan | action | prediction | projection | learning
target_refs: []
project_root:
continuity_id:
current_ask_id:
query_scope_id:
workpoint_ref:
actor_id:
actor_instance_id:
role_profile_id:
created_at:
expires_at:
payload_schema_ref:
payload:
evidence_refs: []
input_refs: []
confidence:
uncertainty:
assumptions: []
contradictions: []
requested_verification_policy_ref:
requested_promotion_policy_ref:
status: proposed | verifying | verified | rejected | superseded | deferred
correlation_id:
causation_id:
trace_id:
```

Rules:

- Payloads MUST validate against a registered schema.
- Evidence-free proposals MAY exist only where policy allows, but MUST be visibly unverified.
- Confidence MUST be finite and within the registered range.
- Proposal identity is immutable.
- Revision creates a new proposal linked by `supersedes`.
- Secondary Cognition, agents, connectors, and UI may propose; none may directly promote.

### 12.2 Verification Bundle

```yaml
schema: focusa.verification_bundle.v1
verification_bundle_id:
subject_ref:
subject_kind: proposal | execution | outcome | completion | settlement | learning
verification_policy_ref:
policy_version:
verifier_runs:
  - verifier_run_id:
    verifier_kind: deterministic | model | human | UIAI | provider | test | composite
    verifier_identity_ref:
    independence_posture:
    input_refs: []
    evidence_refs: []
    started_at:
    completed_at:
    result: pass | fail | inconclusive | unavailable
    confidence:
    objections: []
    reason_codes: []
aggregate_result: pass | fail | inconclusive | unavailable
sufficiency_status: sufficient | insufficient | conditional
freshness_status:
contradiction_status:
required_followups: []
created_at:
expires_at:
correlation_id:
trace_id:
```

Rules:

- Deterministic checks SHOULD precede model verification.
- High-risk completion policies MAY require independent verifier identities.
- A verifier MUST NOT modify the subject under verification.
- Verifier unavailability MUST follow policy; consequential completion defaults fail-closed.
- Model agreement alone MUST NOT prove external state.

### 12.3 Proposal Resolution

```yaml
schema: focusa.proposal_resolution.v1
resolution_id:
resolution_window_ref:
target_ref:
eligible_proposal_ids: []
selected_proposal_id:
outcome: accepted | rejected_all | superseded | clarification_required | deferred
resolver_kind:
resolver_version:
policy_refs: []
verification_bundle_refs: []
deterministic_inputs_hash:
reason_codes: []
explanation:
citations: []
resolved_at:
correlation_id:
trace_id:
```

Rules:

- Fixed inputs and policy version MUST produce the same result.
- All alternatives remain durable.
- An accepted resolution becomes canonical only through a reducer command.
- Clarification is a valid outcome, not an execution failure.

### 12.4 Canonical Commit reference

```yaml
schema: focusa.canonical_commit_ref.v1
commit_ref_id:
resolution_id:
selected_proposal_id:
reducer_event_id:
state_revision_before:
state_revision_after:
canonical_object_refs: []
canonical_delta_ref:
event_hash_chain_ref:
committed_at:
```

### 12.5 Authority Decision

```yaml
schema: focusa.authority_decision.v1
authority_decision_id:
canonical_intent_ref:
operation_id:
ontology_action_type_id:
actor_id:
actor_instance_id:
delegation_ref:
scope_snapshot_ref:
capability_snapshot_ref:
permission_snapshot_ref:
approval_refs: []
budget_snapshot_ref:
risk_class:
side_effect_class:
credential_handle_refs: []
policy_refs: []
decision: authorized | blocked | approval_required | expired
reason_codes: []
missing_requirements: []
max_uses:
uses_consumed:
valid_from:
expires_at:
revocation_ref:
created_at:
correlation_id:
trace_id:
```

Rules:

- Authority is operation-, actor-, resource-, scope-, and time-specific.
- Authorization MUST NOT be inferred from role alone.
- Revocation and expiry are checked again immediately before dispatch.
- Sensitive credentials remain opaque handles and origin-bound where possible.
- Consuming a one-use grant is a canonical event.

### 12.6 Execution Intent

```yaml
schema: focusa.execution_intent.v1
execution_intent_id:
authority_decision_id:
canonical_intent_ref:
operation_id:
adapter_id:
adapter_version:
target_refs: []
input_ref:
input_hash:
idempotency_key:
concurrency_key:
expected_effects: []
forbidden_effects: []
preconditions: []
retry_policy_ref:
reconciliation_policy_ref:
compensation_policy_ref:
timeout_policy_ref:
evidence_policy_ref:
receipt_policy_ref:
status: dispatch_pending | executing | completed | cancelled | failed | outcome_unknown
outbox_event_id:
created_at:
expires_at:
correlation_id:
causation_id:
trace_id:
```

Rules:

- `idempotency_key` is REQUIRED for every consequential operation.
- Input is immutable after authorization.
- Changed input requires a new intent and authority evaluation.
- Intent MUST be durably committed before adapter invocation.
- Dispatch workers consume outbox entries; clients MUST NOT call adapters directly.

### 12.7 Execution Attempt

```yaml
schema: focusa.execution_attempt.v1
attempt_id:
execution_intent_id:
attempt_number:
worker_id:
lease_ref:
runner_ref:
adapter_id:
adapter_version:
started_at:
ended_at:
request_fingerprint:
provider_request_ref:
provider_response_ref:
transport_status:
adapter_status:
observed_side_effects: []
artifact_refs: []
evidence_refs: []
outcome_posture: succeeded | failed_before_effect | failed_after_possible_effect | outcome_unknown | cancelled
reason_codes: []
resource_usage_ref:
correlation_id:
trace_id:
```

### 12.8 Reconciliation Result

```yaml
schema: focusa.reconciliation_result.v1
reconciliation_id:
execution_intent_id:
attempt_ids: []
policy_ref:
reconciler_identity_ref:
observation_refs: []
provider_state_before_ref:
provider_state_after_ref:
expected_effects: []
observed_effects: []
unexpected_effects: []
missing_effects: []
result: reconciled | partial | diverged | outcome_unknown | unavailable
safe_to_retry:
retry_preconditions: []
compensation_required:
compensation_candidate_ref:
reason_codes: []
created_at:
correlation_id:
trace_id:
```

Rules:

- `safe_to_retry=true` requires evidence.
- Search-before-create or confirmation lookup MUST be used where provider idempotency is absent.
- Late provider events MUST attach to the existing intent rather than create a new success lineage.
- Reconciliation may determine success even when the original transport reported failure.
- Reconciliation may determine failure even when the original transport reported success.

### 12.9 Outcome Verification

```yaml
schema: focusa.outcome_verification.v1
outcome_verification_id:
execution_intent_id:
reconciliation_id:
verification_bundle_id:
intended_outcome_ref:
observed_outcome_ref:
acceptance_predicate_refs: []
result: verified | verification_blocked | operator_review
unsatisfied_predicates: []
evidence_refs: []
reason_codes: []
created_at:
correlation_id:
trace_id:
```

### 12.10 Completion Decision

```yaml
schema: focusa.completion_decision.v1
completion_decision_id:
subject_ref:
subject_kind: operation | task | workpoint | workstream | mission | work_item
completion_policy_ref:
predicate_results:
  - predicate_ref:
    result: satisfied | unsatisfied | unavailable | waived
    evidence_refs: []
    waiver_ref:
open_obligations: []
blockers: []
verification_refs: []
decision: complete | not_complete | operator_review | settlement_blocked
reason_codes: []
decided_at:
correlation_id:
trace_id:
```

### 12.11 Settlement Record

```yaml
schema: focusa.settlement_record.v1
settlement_id:
completion_decision_id:
subject_ref:
final_outcome_class: succeeded | partially_succeeded | failed | cancelled | compensated | blocked
canonical_state_revision:
execution_intent_refs: []
attempt_refs: []
reconciliation_refs: []
outcome_verification_refs: []
evidence_refs: []
authority_refs: []
approval_refs: []
open_obligations: []
accepted_residual_risks: []
compensation_status:
closure_refs: []
receipt_policy_ref:
settlement_policy_ref:
settled_by:
settled_at:
status: settled | settlement_blocked | operator_review
reason_codes: []
correlation_id:
trace_id:
```

### 12.12 Receipt integration

Spec 119 remains the canonical Receipt owner.

A Spec 136-aware Receipt MUST add or preserve:

```yaml
protocol_lineage:
  governance_context_ref:
  proposal_refs: []
  verification_bundle_refs: []
  resolution_refs: []
  canonical_commit_refs: []
  authority_decision_refs: []
  execution_intent_refs: []
  execution_attempt_refs: []
  reconciliation_refs: []
  outcome_verification_refs: []
  completion_decision_ref:
  settlement_ref:
```

A Receipt MUST NOT claim `complete`, `actual`, or `settled` when required lineage is absent.

### 12.13 Learning Candidate

```yaml
schema: focusa.learning_candidate.v1
learning_candidate_id:
source_settlement_refs: []
source_receipt_refs: []
source_eval_refs: []
learning_kind: procedural | preference | routing | verification_policy | promotion_policy | retry_policy | domain_pack | eval_case
scope_assignment:
proposed_change_ref:
predicted_benefit:
known_risks: []
evidence_refs: []
verification_policy_ref:
promotion_policy_ref:
status: proposed | quarantined | verifying | promoted | rejected | archived
created_at:
expires_at:
correlation_id:
trace_id:
```

---

## 13. Transaction and durability model

Focusa MUST NOT pretend external systems participate in one universal database transaction. Spec 136 defines three local atomic boundaries and one reconciled external boundary.

### 13.1 Cognitive commitment transaction

The following MUST commit atomically:

```text
ProposalResolution
+ selected proposal status change
+ losing proposal status changes
+ reducer event
+ canonical state revision
+ verification/evidence linkage
+ event hash-chain linkage
```

Failure leaves the proposal eligible for safe retry or resolution replay. It MUST NOT leave evidence claiming a canonical commit that never occurred.

### 13.2 Authorization and dispatch transaction

The following MUST commit atomically:

```text
AuthorityDecision
+ approval/grant consumption where applicable
+ immutable ExecutionIntent
+ idempotency key
+ transactional outbox entry
+ canonical protocol event
```

The adapter is invoked only after this local commit.

### 13.3 External execution boundary

External execution uses:

- idempotency where supported;
- deterministic request fingerprints;
- search-before-create;
- confirmation lookup;
- version checks;
- leases;
- bounded retry;
- reconciliation before repetition;
- provider event correlation;
- compensation where possible.

Exactly-once execution MUST NOT be claimed unless the entire provider path actually guarantees it.

### 13.4 Outcome settlement transaction

After reconciliation and verification, the following MUST commit atomically:

```text
final ReconciliationResult
+ OutcomeVerification
+ CompletionDecision
+ SettlementRecord
+ ReceiptCommitted event
+ Evidence links
+ Workpoint/Trajectory/closure updates
+ event hash-chain linkage
```

Where a Receipt payload is generated after the canonical event, a transactional outbox or deterministic materialization process MUST guarantee eventual generation from committed state.

### 13.5 Crash recovery matrix

| Crash point | Required recovery |
|---|---|
| Before cognitive commit | Proposal remains pending; rerun deterministic resolution |
| After resolution, before reducer commit | No canonical claim; replay same transaction |
| After authorization, before outbox dispatch | Dispatcher resumes from durable outbox |
| During adapter call, before response | Mark possible effect; reconcile before retry |
| After remote success, before local success record | Reconciliation discovers effect and attaches it to original intent |
| After execution record, before verification | Resume outcome verification |
| After completion decision, before settlement | Resume settlement transaction |
| After settlement, before projection generation | Rebuild Receipt/UI projection deterministically |
| After Receipt commit, before learning analysis | Learning is optional; settled outcome remains valid |

---

## 14. Risk-tiered execution lanes

### 14.1 Lane F — Fast deterministic path

Use when all are true:

- read-only or no consequential side effect;
- exact verified scope;
- no sensitive credential use beyond established read access;
- no policy requiring approval;
- bounded response;
- no canonical promotion beyond ordinary read telemetry.

Path:

```text
Governance Context
→ deterministic scope/capability check
→ execute read
→ ToolResult
→ optional Evidence
```

No LLM verifier is required.

### 14.2 Lane G — Governed reversible path

Use for:

- reversible local mutation;
- idempotent provider mutation;
- ordinary canonical promotion;
- bounded generated-UI commit;
- low/moderate-risk work-item operations.

Path:

```text
Propose
→ Verify
→ Resolve
→ Canonical commit
→ Authority
→ Preview where required
→ Dispatch
→ Reconcile
→ Verify outcome
→ Complete
→ Receipt
```

### 14.3 Lane C — Consequential path

Use for:

- destructive or irreversible actions;
- secrets, identity, legal, financial, health, or restricted data;
- production deploy/release;
- database migration;
- broad repository rewrite;
- live service operation;
- high-value provider mutation;
- organizational delegation;
- public claim with material consequence.

Additional requirements MAY include:

- independent verifier;
- explicit operator approval;
- two-person or organizational approval;
- short-lived capability grant;
- simulation or dry run;
- strict budget;
- external backup/checkpoint;
- compensation plan;
- post-action independent verification;
- signed or externally checkpointed Receipt.

### 14.4 Lane assignment

Lane assignment comes from registered ontology action and Operation Registry metadata. A model or client MAY recommend a lane but MUST NOT downgrade it.

---

## 15. Secondary Cognition integration

### 15.1 Permitted roles

Spec 78 Secondary Cognition MAY participate as:

- proposal/extraction worker;
- verification worker;
- adversarial critic;
- prediction worker;
- projection/compression worker;
- reflection worker;
- post-settlement learning proposer.

### 15.2 Required input

Each Secondary Cognition invocation receives a bounded projection containing:

- current objective;
- CurrentAsk and QueryScope;
- relevant and excluded context;
- applicable proposal schema;
- evidence requirements;
- verification policy;
- forbidden authority;
- risk posture;
- stop conditions;
- allowed output types;
- expiry and correlation data.

### 15.3 Required output

Secondary outputs MUST use registered candidate or verification schemas. Freeform prose MAY accompany a structured result but cannot substitute for it.

### 15.4 Forbidden behavior

Secondary Cognition MUST NOT:

- commit canonical state;
- select its own proposal as winner;
- grant authority;
- consume approval;
- access plaintext credentials;
- dispatch external effects;
- mark an attempt reconciled;
- declare settlement;
- promote its own LearningCandidate;
- rewrite the eval harness;
- weaken a policy to make its proposal pass;
- continue after operator steering invalidates its Governance Context.

### 15.5 Adversarial closure role

For consequential completion, Secondary Cognition SHOULD attempt to falsify:

- claimed scope;
- evidence sufficiency;
- missing requirements;
- hidden partial results;
- incorrect provider state;
- stale proof;
- unintended effects;
- false equivalence between process exit and completion;
- missing recovery or restart proof;
- unsupported “actual” claims.

Its output remains advisory to deterministic completion and settlement policy, but policy MAY make a failed or unavailable adversarial verifier blocking.

### 15.6 Post-settlement learning

Learning begins only from settled outcomes or explicitly classified failed/blocked outcomes with sufficient trace.

Path:

```text
Settled outcome
→ comparative analysis
→ LearningCandidate
→ quarantine
→ fixed eval
→ verification
→ scope assignment
→ operator/governance promotion
→ versioned policy/procedure
```

---

## 16. Proposal Resolution Engine integration

PRE remains the decisional concurrency authority.

Spec 136 requires PRE to support or map:

- typed proposal kinds;
- verification bundle references;
- scope and Workpoint revisions;
- proposal expiry;
- deterministic input hash;
- policy bundle version;
- shared reason codes;
- accepted/rejected/superseded/deferred/clarification outcomes;
- canonical commit linkage;
- audit of all alternatives.

PRE MUST NOT:

- directly dispatch execution;
- assume acceptance equals authorization;
- erase losing proposals;
- use unbounded model judgment as the sole resolver for consequential state;
- resolve proposals under a stale CurrentAsk or invalid scope.

---

## 17. Reducer integration

### 17.1 Reducer responsibilities

The reducer SHALL record canonical facts such as:

- proposal resolved;
- canonical candidate promoted;
- authority decision recorded;
- approval consumed;
- execution intent committed;
- execution attempt observed;
- reconciliation recorded;
- outcome verification recorded;
- completion decided;
- settlement committed;
- Receipt committed;
- LearningCandidate promoted or rejected.

### 17.2 Reducer prohibitions

The reducer MUST NOT:

- call models;
- invoke adapters;
- poll providers;
- wait for timeouts;
- reconcile external state;
- generate screenshots;
- execute compensation;
- read credentials;
- perform retry loops.

### 17.3 Proposed event families

```text
GovernanceContextProjected
CognitiveProposalSubmitted
ProposalVerificationRecorded
ProposalResolved
CanonicalIntentCommitted
AuthorityDecisionRecorded
ApprovalConsumed
ExecutionIntentCommitted
ExecutionDispatchRequested
ExecutionAttemptObserved
ExecutionOutcomeMarkedUnknown
ReconciliationRecorded
CompensationRequested
CompensationRecorded
OutcomeVerificationRecorded
CompletionDecisionRecorded
SettlementCommitted
ReceiptCommitted
LearningCandidateSubmitted
LearningCandidatePromoted
LearningCandidateRejected
ProtocolViolationRecorded
```

Every event MUST carry stable scope, correlation, causation, actor, schema, and policy version fields.

---

## 18. Ontology and domain-pack integration

### 18.1 Existing registry extension

Spec 136 MUST extend the existing ontology action definition rather than create a new action registry.

Required policy references:

```yaml
authority_policy_ref:
approval_policy_ref:
dispatch_policy_ref:
reconciliation_policy_ref:
compensation_policy_ref:
completion_policy_ref:
settlement_policy_ref:
receipt_policy_ref:
reason_taxonomy_version:
protocol_lane: F | G | C
```

Existing references remain:

```text
precondition
permission
verification
promotion
rollback
idempotency
timeout
retry
```

### 18.2 Domain-pack obligations

Every domain pack that defines consequential actions MUST declare:

- side-effect class;
- affected resource kinds;
- identity and scope requirements;
- evidence needed for proposal promotion;
- authority and approval policy;
- idempotency strategy;
- reconciliation strategy;
- outcome predicates;
- completion predicates;
- settlement authority;
- compensation posture;
- receipt profile;
- required negative scenarios;
- retention and learning policy.

### 18.3 Candidate/canonical separation

Candidate semantic graph state MAY inform proposals and verification. Only reducer-promoted state satisfying registered promotion policy becomes canonical.

Canonical semantic state MAY establish intent but does not itself prove external execution.

### 18.4 Semantic subscriptions

Semantic delta subscriptions MAY:

- invalidate projections;
- refresh bounded context;
- trigger verification;
- submit proposals;
- pause stale execution.

They MUST NOT silently promote state or dispatch action.

---

## 19. C.R.I.S.T. and Project Genesis integration

Every C.R.I.S.T. stage MUST become protocol-aware through existing read models, ontology definitions, Operation Registry descriptors, and generated action bindings.

### 19.1 Context

Context ingestion and claims MUST expose:

- source provenance;
- capture state;
- freshness;
- contradictions;
- candidate versus canonical status;
- verification policy;
- evidence linkage;
- blocked promotion reasons.

### 19.2 Role

Role candidates MUST distinguish:

- expertise and responsibility;
- operational permissions;
- approval authority;
- delegation;
- scope;
- expiry;
- evidence basis.

Role selection MUST NOT imply permission escalation.

### 19.3 Interview

The Interview SHOULD ask operator-owned questions created by unresolved protocol requirements, including:

- missing evidence;
- ambiguous completion predicates;
- unresolved authority;
- irreversible side effects;
- absent reconciliation path;
- unclear settlement authority;
- incompatible domain policies.

### 19.4 Spec

A generated or approved spec MUST contain protocol declarations for consequential capabilities.

### 19.5 Tasks

Task decomposition MUST include:

```yaml
protocol:
  proposal_kinds: []
  ontology_action_refs: []
  operation_ids: []
  protocol_lane:
  side_effect_classes: []
  verification_policy_refs: []
  promotion_policy_refs: []
  authority_policy_refs: []
  approval_policy_refs: []
  idempotency_policy_refs: []
  retry_policy_refs: []
  reconciliation_policy_refs: []
  compensation_policy_refs: []
  completion_policy_refs: []
  settlement_policy_refs: []
  receipt_policy_refs: []
  reason_codes: []
  negative_scenarios: []
  restart_recovery_scenarios: []
```

### 19.6 Workpoint

Workpoint becomes the immediate protocol posture carrier:

- current proposal/resolution;
- canonical intent;
- current authority posture;
- active execution intent;
- current attempt;
- reconciliation status;
- verification status;
- completion predicates;
- open obligations;
- next safe action;
- recovery instructions.

### 19.7 Generated UI

Generated UI MUST show plain-language protocol posture without requiring raw IDs:

```text
Suggested
Being checked
Ready for decision
Approved as project truth
Permission required
Ready to run
Running
Checking what actually happened
Needs review
Verified
Not complete
Complete
Settled and receipted
```

Advanced Inspector views MAY show full lineage and IDs.

---

## 20. Adversarial Spec Workbench integration

Spec 120 SHALL use Spec 136 as the required execution-governance section for every consequential proposed capability.

### 20.1 Required adversarial questions

The Workbench MUST challenge:

1. What is merely proposed?
2. What verification makes it eligible for canonical promotion?
3. Who resolves conflicting proposals?
4. What reducer event records the canonical result?
5. What authority permits each side effect?
6. What exact operation and ontology action represent it?
7. What idempotency strategy prevents duplicate effects?
8. What happens if the request times out after remote success?
9. How is external state reconciled?
10. What outcome evidence is required?
11. What predicates define completion?
12. Who or what settles completion?
13. What Receipt proves the lineage?
14. What recovery path exists after restart?
15. What post-outcome learning is permitted?
16. Which negative scenarios prove bypass resistance?

### 20.2 Approval requirements

A spec section defining consequential behavior MUST NOT advance to final approval without:

- all required policy references or explicitly tracked policy-definition tasks;
- at least one unknown-outcome scenario;
- at least one duplicate-side-effect scenario;
- at least one wrong-scope or authority-block scenario;
- at least one false-completion scenario;
- an implementation slice that provides value before full capability completion.

### 20.3 Spec outputs

The final spec artifact and decomposition MUST carry stable protocol requirement IDs. Task generation MUST NOT infer this information later from prose.

---

## 21. Operation Registry and generated-contract integration

### 21.1 Descriptor extensions

The existing `focusa.operation_descriptor` SHALL add:

```yaml
protocol:
  protocol_version:
  lane: F | G | C
  proposal_kind_refs: []
  ontology_action_type_id:
  verification_policy_ref:
  promotion_policy_ref:
  authority_policy_ref:
  approval_policy_ref:
  idempotency_policy_ref:
  retry_policy_ref:
  reconciliation_policy_ref:
  compensation_policy_ref:
  completion_policy_ref:
  settlement_policy_ref:
  receipt_policy_ref:
  reason_taxonomy_version:
```

### 21.2 OpenAPI vendor extensions

```text
x-focusa-protocol-version
x-focusa-protocol-lane
x-focusa-proposal-kinds
x-focusa-verification-policy
x-focusa-promotion-policy
x-focusa-authority-policy
x-focusa-approval-policy
x-focusa-reconciliation-policy
x-focusa-compensation-policy
x-focusa-completion-policy
x-focusa-settlement-policy
x-focusa-reason-taxonomy
```

Existing `x-focusa-scope`, capability, permission, preview/commit, confirmation, idempotency, concurrency, receipt, reversibility, and generated-UI metadata remain authoritative.

### 21.3 No generic bypass

Every generated mutation action follows:

```text
trusted generated action binding
→ Operation Descriptor
→ exact scope validation
→ current revision validation
→ policy and lane resolution
→ preview/confirmation where required
→ typed operation
→ canonical protocol event
→ durable execution or canonical commit
→ ToolResult
→ Evidence/Receipt
→ generated UI delta
```

No generic mutation route, raw client call, or UI-local workflow authority may bypass this path.

---

## 22. Daemon, Work Loop, Silent Session, and adapter integration

### 22.1 Daemon ownership

The daemon owns:

- protocol-stage coordination;
- dispatch outbox;
- attempt scheduling;
- leases and concurrency;
- timeout classification;
- retry admission;
- reconciliation scheduling;
- compensation coordination;
- completion-evaluation scheduling;
- settlement orchestration;
- recovery after controller loss.

### 22.2 Work Loop

The continuous Work Loop MUST select the next step from canonical protocol state, not from transcript momentum.

Examples:

- `verification_blocked` → collect evidence or request operator decision;
- `approval_required` → pause;
- `dispatch_pending` → dispatch;
- `outcome_unknown` → reconcile;
- `partial` → verify partial predicates or compensate;
- `not_complete` → produce next Workpoint slice;
- `settled` → advance task graph;
- `dead_letter` → operator review or alternate safe work.

### 22.3 Silent Sessions

A Silent Session MUST bind to:

- canonical execution intent;
- exact Workpoint revision;
- authority decision;
- model/provider configuration;
- writer lease/worktree;
- budgets;
- checkpoint policy;
- evidence and receipt policy.

Session exit enters completion evaluation; it does not directly settle.

### 22.4 Adapter contract

Every consequential adapter MUST implement:

```text
preflight
dispatch
observe
reconcile
classify_retry
compensation_candidate
health
capabilities
```

Where an operation cannot support reconciliation, the Operation Registry MUST disclose that limitation and policy MUST choose fail-closed, operator review, or a lower-risk lane.

---

## 23. UIAI Engine integration

UIAI Engine remains browser-facing execution and artifact authority.

Spec 136 requires each UIAI action result to identify:

- ExecutionIntent;
- attempt;
- browser context and lease;
- action sequence or scenario;
- stable artifact references;
- diagnostics;
- observed effect;
- provider/page state used for reconciliation;
- proof limitations;
- correlation and causation IDs.

UIAI Engine MUST NOT:

- grant itself Focusa authority;
- mark Focusa canonical completion;
- convert a screenshot into a settled claim by itself;
- promote website instructions into policy;
- hide browser recovery or partial-action posture.

UIAI Engine Eval SHOULD provide independent browser proof for relevant outcome predicates.

---

## 24. Evidence, completion, settlement, and Receipt rules

### 24.1 Evidence classes

Evidence retains the shared classes:

```text
actual
partial
surrogate
blocked
missing
```

Spec 136 adds stage relevance:

```text
proposal evidence
authority evidence
execution evidence
reconciliation evidence
outcome evidence
completion evidence
settlement evidence
```

### 24.2 Completion predicates

Completion policy MUST define machine-checkable or explicitly reviewable predicates.

Examples:

- required artifact exists and hashes match;
- provider state equals expected state;
- tests pass;
- UIAI scenario passes;
- no required ledger item remains open;
- Workpoint drift is resolved;
- recovery/restart proof passes;
- operator approval exists;
- evidence is fresh;
- no unresolved high-severity contradiction remains.

### 24.3 Settlement requirements

Settlement MUST verify:

- exact subject identity;
- final canonical revision;
- required authority and approval;
- all execution intents have known final posture;
- unknown outcomes are resolved or explicitly accepted;
- required reconciliations completed;
- completion predicates evaluated;
- open obligations represented;
- residual risks represented;
- compensation state represented;
- required Evidence linked;
- Receipt policy satisfied.

### 24.4 Blocked settlement

`settlement_blocked` MUST include:

- stable reason codes;
- missing requirements;
- unresolved intent or attempt IDs;
- next safe operation;
- operator review route;
- whether unrelated ready work may continue.

---

## 25. Unified reason-code taxonomy

### 25.1 Format

```text
<domain>.<specific_reason>
```

Codes are stable, versioned, machine-readable, and presentation-neutral.

### 25.2 Required domains

```text
scope
identity
context
proposal
verification
resolution
canonical
authority
approval
capability
permission
budget
credential
concurrency
dispatch
execution
retry
reconciliation
compensation
evidence
completion
settlement
receipt
learning
migration
security
protocol
```

### 25.3 Minimum codes

```text
scope.missing
scope.mismatch
scope.stale
scope.contaminated
scope.current_ask_superseded

identity.unverified
identity.actor_mismatch
identity.project_mismatch

proposal.schema_invalid
proposal.evidence_missing
proposal.expired
proposal.superseded

verification.failed
verification.inconclusive
verification.unavailable
verification.evidence_stale
verification.conflict_unresolved
verification.confidence_below_threshold
verification.independence_required

resolution.rejected_all
resolution.clarification_required
resolution.policy_version_mismatch

canonical.revision_conflict
canonical.commit_failed

authority.blocked
authority.expired
authority.revoked
authority.scope_exceeded
authority.delegation_depth_exceeded

approval.required
approval.expired
approval.actor_not_authorized

capability.unavailable
capability.degraded
permission.missing

budget.exhausted
budget.retry_limit
budget.human_attention_required

credential.missing
credential.expired
credential.origin_mismatch

concurrency.lease_conflict
concurrency.revision_conflict

dispatch.outbox_failed
dispatch.adapter_unavailable

execution.failed_before_effect
execution.failed_after_possible_effect
execution.outcome_unknown
execution.cancelled

retry.not_safe
retry.reconciliation_required
retry.exhausted

reconciliation.partial
reconciliation.diverged
reconciliation.unavailable
reconciliation.remote_state_missing
reconciliation.unexpected_effect

compensation.required
compensation.unavailable
compensation.failed
compensation.partial

evidence.missing
evidence.partial
evidence.surrogate_only
evidence.stale
evidence.contradictory

completion.predicate_unsatisfied
completion.verifier_unavailable
completion.false_done_risk
completion.open_obligations

settlement.blocked
settlement.unknown_attempt
settlement.receipt_incomplete
settlement.residual_risk_unaccepted

receipt.lineage_incomplete
receipt.projection_not_canonical

learning.not_settled
learning.eval_failed
learning.scope_too_broad
learning.promotion_not_authorized

migration.schema_incompatible
migration.policy_missing

security.untrusted_authority_attempt
security.secret_exposure_attempt

protocol.stage_skip_attempt
protocol.version_incompatible
protocol.bypass_detected
```

### 25.4 Required block envelope

Every blocked generated UI, API, CLI, Pi, daemon, adapter, and provider-guard surface MUST map to:

```yaml
schema: focusa.protocol_block.v1
status: blocked
stage:
reason_code:
summary:
why_blocked:
missing_requirements: []
affected_refs: []
safe_to_retry:
retry_after_reconciliation:
required_next_operation_id:
alternative_safe_operation_ids: []
operator_review_route:
doctor_operation_id:
correlation_id:
trace_id:
```

---

## 26. Persistence, eventing, and read models

### 26.1 Persistence

Canonical protocol records use existing persistence abstractions and SQLite event history.

Recommended query tables MAY include:

```text
protocol_proposals
protocol_verifications
protocol_resolutions
protocol_authority_decisions
protocol_execution_intents
protocol_execution_attempts
protocol_reconciliations
protocol_completion_decisions
protocol_settlements
```

The canonical integrity source remains events plus reducer state; query tables are rebuildable read models.

### 26.2 Event envelope

Existing native stream events MUST preserve:

- event ID;
- sequence;
- scope;
- source revision;
- payload reference;
- correlation ID;
- causation ID;
- protocol stage;
- subject references;
- invalidation keys.

### 26.3 Large payloads

Model outputs, provider bodies, screenshots, videos, logs, and large evidence artifacts remain externalized behind stable handles.

### 26.4 Retention

- canonical decisions and settled Receipts follow durable domain retention policy;
- failed attempts remain auditable;
- raw output may age to cold storage;
- projections may be regenerated;
- predictions decay aggressively;
- LearningCandidates remain quarantined until promoted or archived;
- deletion/redaction preserves required integrity metadata where legally and technically appropriate.

---

## 27. Security and trust boundaries

1. Untrusted content may inform proposals but cannot define policy or authority.
2. Secrets remain outside model context where practical.
3. Credential handles are origin-, actor-, operation-, and time-bound.
4. Data egress checks destination, project, mission, grant, and classification.
5. Cross-project data is inaccessible by default.
6. Adapter outputs are schema-validated and risk-classified.
7. Policy definitions are versioned and immutable after release.
8. Older incompatible clients are blocked from consequential mutation.
9. Generated UI cannot mint approvals.
10. External provider webhooks are observations until correlated and verified.
11. A compromised verifier cannot alone authorize execution.
12. High-risk policies SHOULD require verifier independence.
13. Every bypass attempt emits `protocol.bypass_detected`.
14. Every stage skip attempt is blocked and audited.
15. Revocation is checked at dispatch, not only at initial planning.

---

## 28. User experience requirements

### 28.1 Calm default presentation

Normal users see:

- what is proposed;
- why it is recommended;
- what will change;
- whether approval is needed;
- current progress;
- what actually happened;
- what remains;
- what proves completion;
- next safe action.

### 28.2 Advanced inspection

Inspector views expose:

- proposal and policy IDs;
- verification objections;
- authority and approvals;
- attempts;
- provider requests/responses through redacted handles;
- reconciliation comparison;
- completion predicates;
- Receipt lineage;
- reason codes;
- event history.

### 28.3 No ceremony for harmless work

Fast-path reads MUST NOT display unnecessary approval dialogs or settlement language.

### 28.4 Consequence preview

Before consequential commit, UI MUST show:

- intended effects;
- possible irreversible effects;
- scope;
- authority source;
- evidence basis;
- expected cost/time;
- reconciliation plan;
- recovery/compensation posture;
- required proof.

### 28.5 Unknown outcome UX

The UI MUST distinguish:

```text
Failed before anything changed
May have changed; checking external state
Changed partially
Changed and verified
Unable to verify; review required
```

It MUST NOT show a generic retry button when reconciliation is required.

---

## 29. Observability and operational metrics

Required metrics:

- proposals by outcome;
- verification pass/fail/inconclusive rate;
- proposal-to-canonical latency;
- authority block rate;
- approval wait time;
- dispatch latency;
- execution attempt count;
- unknown-outcome rate;
- reconciliation success rate;
- duplicate-side-effect rate;
- partial/diverged outcome rate;
- compensation rate and success;
- false-done rate;
- completion-to-settlement latency;
- settlement block rate;
- Receipt lineage completeness;
- recovery success after restart;
- operator intervention rate;
- cost per verified outcome;
- cost per settled outcome;
- learning-candidate promotion rate;
- regression after learning promotion.

Metrics MUST remain scoped and privacy-preserving. Public aggregation requires explicit redaction and publication.

---

## 30. Evaluation and benchmark requirements

Spec 113 remains the benchmark owner.

### 30.1 Required ablation arms

```text
A0 — raw harness, no Focusa
A1 — Focusa scope/context only
A2 — proposal + verification + canonical commit
A3 — A2 + authority
A4 — A3 + durable execution intent
A5 — A4 + reconciliation
A6 — A5 + completion/settlement/Receipt
A7 — A6 + governed post-settlement learning
```

### 30.2 Required perturbations

- stale evidence;
- conflicting evidence;
- low confidence;
- wrong project root;
- wrong continuity ID;
- new operator steering;
- approval expiry;
- capability revocation;
- budget exhaustion;
- duplicate dispatch;
- provider timeout after success;
- provider success response without actual mutation;
- worker crash after side effect;
- daemon restart;
- browser context loss;
- stale generated UI action;
- optimistic concurrency conflict;
- malicious web content attempting to grant authority;
- task marked done without completion evidence;
- single successful run attempting policy promotion;
- incompatible client/schema version.

### 30.3 Required outcome metrics

- unsafe action rate;
- unsupported canonical mutation rate;
- stage-skip/bypass rate;
- duplicate-side-effect rate;
- unknown-outcome recovery;
- false-done rate;
- settled completion rate;
- evidence completeness;
- Receipt lineage completeness;
- recovery success;
- operator rescue rate;
- latency;
- token and financial cost;
- cost per settled outcome.

### 30.4 Immutable eval law

The system under evaluation MUST NOT modify:

- task definitions;
- scoring;
- acceptance predicates;
- stop conditions;
- holdout set;
- promotion threshold.

Policy-change proposals follow governance and are evaluated in a future version.

---

## 31. Compatibility and migration

### 31.1 Existing records

Existing proposals, verification records, Workpoints, authority records, execution events, closures, and Receipts MUST be classified as:

```text
fully_mapped
partially_mapped
legacy_projection
unmappable
```

Migration MUST NOT fabricate missing lineage.

### 31.2 Legacy completion

A legacy “completed” item without sufficient lineage MAY remain historically completed in its source system but MUST be marked:

```text
legacy_completion_unsettled
```

until verified under an applicable policy.

### 31.3 Client compatibility

Consequential writes require protocol-version compatibility.

Read-only clients MAY receive degraded projections for unknown future fields.

### 31.4 Policy migration

Policy changes require:

- compatibility profile;
- migration plan;
- affected record classes;
- replay/conformance proof;
- downgrade posture;
- operator or governance approval.

---

# 32. Critical-path implementation order after Spec 135 closure

## 32.0 Delivery law

Implementation MUST follow the critical path to progressively complete functionality.

The team MUST NOT wait for every Spec 136 subsystem before delivering a working slice.

Each tranche MUST:

- preserve all previous working slices;
- use live canonical state;
- use typed APIs and generated clients;
- expose truthful capability status;
- add negative-path proof;
- produce Evidence and a tranche Receipt;
- pass restart/replay tests;
- leave no parallel temporary authority path;
- update the machine-readable delivery graph and feature ledger.

The sequence below is normative unless a documented dependency correction is approved through the Adversarial Spec Workbench.

---

## 32.1 Tranche P0 — Post-135 activation and reality lock

### Goal

Prove Spec 135 is actually complete and freeze the starting architecture.

### Deliverables

- `Spec136Activation` record;
- current code/reducer/API/event/ontology/Operation Registry/Receipt inventory;
- cross-spec ownership matrix;
- current lifecycle and reason-code collision audit;
- baseline benchmark run;
- migration inventory;
- architecture decision record confirming no second registry/store/reducer.

### Working result

No user-facing protocol change yet, but implementation can begin without reopening architecture questions.

### Gate

P0 closes only with actual Spec 135 closure Evidence and Receipt.

### Not done if

- Spec 135 remains partially implemented;
- any core generated UI still uses hidden mock state;
- the Operation Registry or native event replay is incomplete;
- the ownership matrix leaves an ambiguous authority boundary.

---

## 32.2 Tranche P1 — Contract spine and shared reason taxonomy

### Goal

Create the versioned contracts that every later slice uses.

### Deliverables

- JSON Schemas for all core Spec 136 records;
- Rust types and validation;
- OpenAPI exposure for read surfaces;
- generated TypeScript models over language-neutral OpenAPI/JSON Schema contracts;
- reason taxonomy package;
- shared protocol block envelope;
- event envelope extensions;
- registry extension schemas;
- compatibility tests.

### Working slice

All existing post-135 operations can expose protocol posture and stable block reasons even before execution orchestration changes.

### User value

Better diagnostics and recovery immediately.

### Not done if

- schemas are handwritten independently in multiple languages;
- clients use duplicate DTOs;
- reason codes exist only in prose;
- existing errors cannot map to the common block envelope.

---

## 32.3 Tranche P2 — Candidate-to-canonical cognition slice

### Goal

Implement the full non-executing path:

```text
CognitiveProposal
→ VerificationBundle
→ PRE resolution
→ reducer canonical commit
→ Evidence
→ Receipt
```

### First product slice

Use one real C.R.I.S.T. or ontology decision that does not cause an external side effect, such as:

- Role Profile candidate approval;
- governed glossary/ADR candidate;
- project decision or constraint promotion.

### Deliverables

- proposal APIs;
- deterministic resolution integration;
- reducer events;
- canonical commit linkage;
- generated UI states;
- full trace;
- restart/replay;
- adversarial rejection example;
- Receipt lineage.

### Working result

Focusa gains a real Alethic-like propose/verify/commit path without waiting for side-effect infrastructure.

### Not done if

- a model writes canonical state directly;
- generated UI treats proposal as fact;
- rejected proposals disappear;
- transaction failure can leave false commit evidence.

---

## 32.4 Tranche P3 — Governance Context and operation-policy awareness

### Goal

Make all registered operations and C.R.I.S.T. surfaces aware of applicable protocol policy.

### Deliverables

- Governance Context projection service;
- Operation Registry extensions;
- ontology action policy extensions;
- lane assignment;
- generated action-binding support;
- stale-context invalidation;
- reason-coded capability posture;
- fast-path read behavior.

### Working slice

Every operation can answer:

```text
What stage is this?
What lane applies?
What policy applies?
What is allowed?
What is missing?
What happens next?
```

### User value

C.R.I.S.T., ontology views, generated UI, and agents become consistently guardrail-aware before external execution is changed.

### Not done if

- guardrails live only in prompts;
- clients infer policy from labels;
- harmless reads acquire consequential approval ceremony;
- policy projection can grant authority by itself.

---

## 32.5 Tranche P4 — Authority and durable local execution intent

### Goal

Implement:

```text
canonical intent
→ AuthorityDecision
→ approval/grant consumption
→ ExecutionIntent
→ transactional outbox
```

### First mutation slice

Choose one reversible, local, idempotent operation already implemented after Spec 135, such as a bounded local project metadata or generated artifact update.

### Deliverables

- authority service integration;
- immutable ExecutionIntent;
- outbox dispatcher;
- idempotency;
- attempt identity;
- lease/concurrency check;
- restart after outbox commit;
- cancellation before dispatch;
- Receipt update.

### Working result

Focusa can durably authorize and dispatch one real mutation without losing it across a crash.

### Not done if

- a client invokes the side effect directly;
- authorization exists only in memory;
- a crash between authorization and execution loses the operation;
- changed inputs reuse the old authority decision.

---

## 32.6 Tranche P5 — First reconciled external side-effect slice

### Goal

Prove external outcome truth.

### Recommended first slice

Use the post-135 provider-neutral task adapter to perform one bounded work-item creation or status mutation, then reconcile provider state and issue a Receipt.

Alternative selection requires approval and must have equivalent idempotency and observable state.

### Deliverables

- adapter `preflight/dispatch/observe/reconcile`;
- provider request fingerprint;
- unknown-outcome handling;
- search-before-create or provider idempotency;
- safe retry classification;
- partial/diverged states;
- generated UI recovery;
- restart during possible remote success;
- Receipt settlement.

### Working result

Focusa proves one complete proposal-to-settlement external mutation.

### Not done if

- timeout automatically retries;
- provider `200` is accepted without reconciliation where policy requires it;
- remote success after local timeout creates a duplicate on restart;
- reconciliation cannot connect to the original intent.

---

## 32.7 Tranche P6 — C.R.I.S.T. end-to-end settled workflow

### Goal

Apply the protocol across one complete post-135 C.R.I.S.T. journey.

### Required dogfood slice

```text
Context evidence
→ Role/decision proposal
→ Interview/operator decision
→ approved Spec section
→ task decomposition
→ provider task mutation
→ Workpoint
→ bounded execution
→ Evidence
→ completion
→ settlement
→ Receipt
→ pause/restart/resume proof
```

### Deliverables

- generated UI for every protocol stage;
- one primary next action;
- Inspector lineage;
- capability degradation;
- approval and block recovery;
- exact-state resume;
- UIAI Engine Eval browser proof;
- nontechnical usability proof.

### Working result

The protocol is no longer infrastructure-only; it improves the flagship product workflow.

### Not done if

- any stage uses a separate authority path;
- raw JSON is required for ordinary completion;
- browser proof bypasses UIAI Engine;
- restart loses stage posture;
- success is decorative rather than canonical.

---

## 32.8 Tranche P7 — General completion and settlement engine

### Goal

Generalize completion and settlement beyond the first adapter.

### Deliverables

- registered CompletionPolicy and SettlementPolicy;
- predicate engine;
- open-obligation model;
- residual-risk acceptance;
- settlement transaction;
- Receipt lineage validation;
- work-item closure mapping;
- false-done blocker;
- legacy-completion classification.

### Working slices

- code task completion;
- docs/spec completion;
- UI/product completion;
- investigation/no-code completion;
- provider work-item closure.

### Not done if

- one generic “done” predicate applies to every domain;
- process exit closes a Workpoint;
- provider status becomes Focusa completion truth;
- Receipt can be committed without required lineage.

---

## 32.9 Tranche P8 — UIAI and Silent Session outcome governance

### Goal

Apply protocol semantics to browser execution and durable autonomous sessions.

### Deliverables

- UIAI attempt and reconciliation mapping;
- browser-context/lease lineage;
- proof artifact binding;
- Silent Session ExecutionIntent binding;
- completion after process exit;
- controller-loss recovery;
- operator steering invalidation;
- budget and model-change events;
- worktree writer isolation;
- settlement Receipts.

### Working result

Long-running and browser-facing work becomes recoverable and verifiably complete.

### Not done if

- terminal output is canonical evidence by itself;
- session exit implies completion;
- UIAI artifacts lack intent/attempt lineage;
- operator steering does not invalidate stale execution direction.

---

## 32.10 Tranche P9 — Controlled post-settlement learning

### Goal

Use settled outcomes to improve Focusa without self-sovereign learning.

### Deliverables

- LearningCandidate contract and quarantine;
- fixed eval linkage;
- comparative outcome analysis;
- scope assignment;
- promotion/rejection governance;
- policy and procedure versioning;
- rollback/deprecation plan;
- UI for inspecting learned candidates;
- regression monitoring.

### First learning slice

Promote one narrowly scoped procedural or routing improvement only after:

- multiple settled examples or policy-specified evidence;
- holdout eval improvement;
- no safety regression;
- explicit governance approval.

### Working result

Focusa begins getting sharper from verified outcomes.

### Not done if

- one successful run changes global procedure;
- the learning loop edits its own eval;
- a model promotes its own recommendation;
- learning provenance is missing.

---

## 32.11 Tranche P10 — Domain-pack expansion

### Goal

Apply the same protocol to built-in General, Software, Research, Legal, Markets, and Custom packs.

### Deliverables per pack

- consequential action inventory;
- policy references;
- evidence profiles;
- completion predicates;
- reconciliation adapters;
- reason-code extensions;
- generated UI projections;
- negative tests;
- benchmark tasks;
- migration declaration.

### Working rule

Each pack ships independently after conformance. Packs MUST NOT wait for every other pack.

### Not done if

- a pack creates a separate protocol runtime;
- domain expertise grants permission;
- completion semantics are undocumented;
- generic behavior remains trapped in pack-local code.

---

## 32.12 Tranche P11 — Advanced resilience and distributed operation

### Goal

Complete high-maturity capabilities after local settled workflows are proven.

### Candidate capabilities

- organizational approval chains;
- distributed workers and device handoff;
- signed Receipts;
- external checkpoint publication;
- multi-node outbox/inbox;
- distributed leases;
- provider webhook correlation;
- compensation orchestration;
- advanced cost routing;
- incident response and dead-letter operations;
- cross-application execution.

### Law

These capabilities MUST extend the proven local protocol and MUST NOT delay P2–P10 working slices.

---

## 32.13 Parallel lanes after P3

After the contract spine, cognition slice, and policy-awareness layer stabilize, implementation MAY proceed in parallel:

```text
Lane A — authority, approvals, budgets, credentials
Lane E — execution intent, outbox, attempts, leases
Lane R — reconciliation, retry, compensation, dead letter
Lane C — completion, settlement, Receipts
Lane U — C.R.I.S.T. generated UI and UIAI proof
Lane O — ontology registry and domain-pack policy
Lane S — Secondary Cognition and adversarial verification
Lane B — benchmarks, negative tests, conformance
Lane M — migration, compatibility, documentation
```

Cross-lane merges require stable generated contracts and must preserve the currently working vertical slice.

---

## 33. Machine-readable delivery artifacts

Before P1 decomposition closes, create:

```text
docs/contracts/spec136-complete-feature-ledger.v1.yaml
docs/contracts/spec136-delivery-dag.v1.yaml
docs/contracts/spec136-protocol-state-machine.v1.yaml
docs/contracts/spec136-reason-taxonomy.v1.yaml
docs/contracts/spec136-policy-ownership-matrix.v1.yaml
docs/contracts/spec136-record-schema-index.v1.yaml
docs/contracts/spec136-client-parity-matrix.v1.yaml
docs/contracts/spec136-adapter-conformance-matrix.v1.yaml
docs/contracts/spec136-proof-matrix.v1.yaml
docs/contracts/spec136-migration-matrix.v1.yaml
```

Every normative requirement MUST have:

- stable requirement ID;
- owner;
- dependencies;
- implementation tranche;
- code owner/package;
- reducer event impact;
- API operation;
- generated client impact;
- C.R.I.S.T./UI surface;
- policy reference;
- reason codes;
- tests;
- negative tests;
- restart/recovery scenario;
- Evidence requirement;
- Receipt requirement;
- migration requirement;
- closure status.

Agents MUST NOT infer the implementation graph from prose alone.

---

## 34. Task decomposition contract

Every Spec 136 implementation task MUST include:

```yaml
requirement_refs: []
blocking_refs: []
implementation_tranche:
working_slice_ref:
primitive_owner:
reuse_assessment:
files_and_packages: []
core_types: []
record_schemas: []
reducer_events: []
state_transitions: []
api_operations: []
operation_registry_changes: []
ontology_registry_changes: []
generated_contracts: []
generated_ui: []
uiai_eval: []
policy_refs: []
reason_codes: []
transaction_boundary:
idempotency:
reconciliation:
compensation:
completion:
settlement:
security:
privacy:
migration:
compatibility:
performance:
accessibility:
unit_tests: []
contract_tests: []
integration_tests: []
negative_tests: []
restart_recovery_tests: []
benchmark_tasks: []
evidence: []
receipts: []
definition_of_done: []
not_done_if: []
```

Required `not_done_if` cases include:

```text
proposal becomes canonical without reducer promotion
verification has no policy/version/evidence identity
resolution implies execution authority
action dispatches without durable ExecutionIntent
authorization exists only in memory
timeout retries before reconciliation
unknown outcome is represented as ordinary failure
process exit is treated as completion
provider status is treated as Focusa settlement
Receipt omits required protocol lineage
generated UI recreates authority logic
guardrail exists only in prompt text
reason is a vague string without stable code
restart cannot resume exact protocol stage
domain pack creates a parallel registry or runtime
learning promotes without settled source and fixed eval
feature is backend-only with no truthful generated UI where user-facing
browser proof bypasses UIAI Engine
```

---

## 35. Conformance levels

### S136-L0 — Aware

- Governance Context projection exists.
- Operations expose lane and policy.
- Stable reason codes exist.
- No new bypass path is introduced.

### S136-L1 — Cognition governed

- Proposal, verification, PRE resolution, and reducer commit are linked.
- Candidate/canonical separation is enforced.
- Rejected/superseded history is durable.

### S136-L2 — Execution governed

- AuthorityDecision and durable ExecutionIntent precede side effects.
- Outbox, attempts, idempotency, and leases operate.
- Restart resumes dispatch safely.

### S136-L3 — Outcome settled

- Reconciliation, outcome verification, completion policy, settlement, and Receipt operate.
- Unknown outcomes and false-done are blocked.
- At least one real external adapter is fully conformant.

### S136-L4 — Adaptive and distributed

- Controlled post-settlement learning operates.
- Multiple domain packs conform.
- Distributed workers, organizational authority, signed or external proof, and advanced resilience operate where supported.

A capability MUST NOT be marketed or rendered as a higher level than demonstrated by Evidence.

---

## 36. Required conformance scenarios

At minimum:

1. A valid proposal is verified, resolved, committed, and receipted.
2. A locally plausible proposal is rejected for missing evidence.
3. Two conflicting proposals resolve deterministically.
4. A stale CurrentAsk blocks promotion.
5. A canonical intent is blocked by missing authority.
6. An approval expires before dispatch.
7. A crash after outbox commit resumes without losing the action.
8. A provider succeeds but the response is lost; reconciliation prevents duplicate retry.
9. A provider returns success without the expected state; reconciliation marks divergence.
10. A partial result creates open obligations rather than false completion.
11. Process exit enters completion evaluation rather than `complete`.
12. A completion verifier identifies missing proof.
13. Settlement blocks because one attempt remains unknown.
14. A Receipt regenerates after projection loss.
15. Operator steering invalidates stale autonomous direction.
16. Malicious website content attempts to grant authority and is rejected.
17. A client attempts to skip directly from proposal to execution and is blocked.
18. A single successful run attempts global learning promotion and is quarantined.
19. A restart resumes exact protocol stage and Workpoint.
20. A low-risk read completes through the fast path without unnecessary model calls.

---

## 37. Performance requirements

- Fast-path reads SHOULD add negligible overhead beyond existing scope/capability checks.
- Deterministic validation MUST run before model verification.
- Governance Context projections SHOULD be cached by canonical revision and invalidated by event.
- Large artifacts remain behind handles.
- Protocol stage transitions SHOULD avoid full generated-surface regeneration; targeted invalidation is required.
- Reconciliation polling MUST use provider-appropriate backoff and budgets.
- Model verification SHOULD be skipped when deterministic policy fully decides.
- Settlement generation MUST be resumable and idempotent.
- Benchmarks MUST report latency and cost by lane and stage.

---

## 38. Documentation and developer experience

Required documentation:

- protocol overview;
- state-machine reference;
- record schema reference;
- reason-code catalog;
- policy-authoring guide;
- adapter implementation guide;
- C.R.I.S.T. integration guide;
- ontology/domain-pack guide;
- Secondary Cognition guide;
- recovery and unknown-outcome runbook;
- migration guide;
- conformance-level guide;
- troubleshooting/doctor guide.

Required developer tools:

```text
focusa protocol inspect <subject>
focusa protocol trace <correlation-id>
focusa protocol reconcile <execution-intent-id>
focusa protocol retry-check <execution-intent-id>
focusa protocol settlement inspect <subject>
focusa doctor protocol
```

CLI commands are projections over canonical APIs and MUST NOT create a second authority path.

---

## 39. Release and closure gates

Spec 136 cannot close until:

- every feature-ledger requirement is verified;
- every required schema and generated client is in parity;
- the state machine rejects illegal transitions;
- the reason taxonomy is implemented across API, CLI, Pi, daemon, C.R.I.S.T., generated UI, and adapters;
- P2 candidate-to-canonical slice passes;
- P5 first external reconciled slice passes;
- P6 full C.R.I.S.T. settled dogfood traversal passes;
- P7 completion/settlement policies pass across required closure classes;
- P8 UIAI and Silent Session recovery passes;
- P9 controlled learning proves improvement without unsafe self-promotion;
- restart, replay, migration, downgrade, and recovery scenarios pass;
- false-done, duplicate-side-effect, wrong-scope, stale-authority, and unknown-outcome tests pass;
- all user-facing protocol states are available through generated nontechnical UI;
- browser proof uses UIAI Engine Eval;
- no parallel reducer, store, registry, permission system, receipt ledger, or error taxonomy exists;
- benchmark results demonstrate net improvement against the post-Spec-135 baseline;
- a final Spec 136 Completion Receipt and proof matrix are committed.

---

## 40. Success condition

Spec 136 is successful when Focusa can demonstrate, across C.R.I.S.T., ontology-driven actions, adversarial spec planning, Workpoints, continuous execution, UIAI Engine, provider adapters, completion, and learning, that:

- probabilistic cognition remains useful but non-sovereign;
- canonical truth is reducer-governed;
- authority is explicit and operation-specific;
- side effects begin from durable execution intent;
- ambiguous external outcomes are reconciled before retry;
- completion is evidence-backed rather than asserted;
- settlement preserves open obligations and residual risk;
- Receipts expose complete causal lineage;
- restarts and model changes preserve exact protocol posture;
- domain packs reuse one protocol;
- generated UI is calm, truthful, and actionable;
- the system improves only from governed, evaluated, scoped learning;
- the first working slices deliver value long before every advanced capability is complete.

The final systemic rule is:

> **Models may observe, reason, propose, predict, execute within granted authority, and critique. Only Focusa’s governed proposal-to-settlement protocol may establish operational truth.**
