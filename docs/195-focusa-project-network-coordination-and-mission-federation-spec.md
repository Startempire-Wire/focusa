# Proposed Spec 195 — Focusa Project Network Coordination and Mission Federation

**Status:** complete authoring candidate for owner review; documentation-only; implementation not admitted.  
**Version:** 0.1.0. **Prepared:** 2026-11-04.  
**Numbering:** 195 is a candidate, not a repository reservation. Main was inspected through numbered document 193; an existing integration tree also contains 194-event-ledger-growth-control.md. Recheck the target branch before adoption. Historical issue #503 proposed number 184, which is now occupied by Ambient Operator; this document does not overwrite either artifact.  
**Direction authorized:** translate the useful shared-workspace ideas into Focusa-native primitives, without a Busabase dependency; standardize the future project-network/communication layer before task decomposition. Detailed design below remains proposed until owner review.  
**Immediate product priority:** the existing qualified-issue-closure release. This future capability is not a new prerequisite for that release.  
**Canonical architecture authority:** the deployment's Canonical Owner Principal, under the repository architecture-authority policy. Agent authorship, source presence, and a content hash are not architecture approval.

## 1. Outcome and product boundary

Focusa should understand related projects, explain their interfaces and dependencies, locate the responsible authorized participants, exchange bounded requests and evidence across machines, and coordinate approved shared missions without merging any project's identity, goals, permissions, execution state, or completion authority.

The differentiator is not another chat channel. It is the combination of an inspectable project relationship network, revision-aware contract knowledge, governed cross-scope coordination, and evidence-backed outcomes.

There are three distinct products of the same substrate:

1. **Persistent project network:** accepted relationships among independently owned scopes; useful without a shared mission or live agents.
2. **Operational coordination:** presence-filtered, policy-bounded exchange and handoff among exact participants; useful without creating a federation.
3. **Mission Federation:** an explicitly approved shared objective over independently admitted project/workstream contributions; not a super-project or execution scope.

The implementation is future work. This specification records semantics and proof obligations now. Implementation tasks, live federation records, peer enrollments, budgets, writer leases, and release gates are not created by authoring it.

## 2. Normative model and conformance

Binding requirements are the individually identified `PNC-Rnnn` paragraphs below and the field/state contracts they explicitly activate. The companion requirement ledger maps each paragraph to ownership, dependencies, proof obligations, and a planning status. All other prose is explanatory context unless explicitly activated by an identified requirement.

Conformance labels are `documentation_only`, `contract_only`, `partial_runtime`, `implemented_unverified`, `verified_slice`, and `full_spec_conformance`. A slice is not full conformance. A source package is not an installed behavior claim.

**PNC-R001 — Release separation.** This specification MUST preserve the existing release workset, qualified-closure threshold, mandatory issue-specific acceptance, retained prior obligations, and original issue criteria; MUST NOT add the network layer as a prerequisite, count spec authoring as issue closure, or use it to waive recently discovered release-critical defects.

**PNC-R002 — Owner approval and freeze.** Canonical adoption MUST record the owner-approved specification revision, exact content/contract digests, decision provenance, and unresolved dispositions; any normative change MUST invalidate affected coverage and decomposition approval until regeneration and review. A hash or agent-authored statement alone MUST NOT count as approval.

**PNC-R003 — No software dependency.** Implementation MUST translate the adopted ideas into Focusa's existing primitive owners and MUST NOT require Busabase, a public messaging service, a particular harness, SSH, a particular operating system, or an additional daemon merely to use the baseline project-network capability.

## 3. Primitive ownership: reuse first

| Concern | Existing owner | Treatment here |
|---|---|---|
| Typed project/host scope, Workstream, attachment | `98-project-root-crdt-reconciliation-foundation-spec.md`, `104-typed-scoped-runtime-and-singleton-elimination-spec.md`, `164-workstream-rooted-canonical-runtime-design.md` | Reuse exact identities; no NetworkScope |
| Objects, directional relationships, properties, provenance, verification | `46-ontology-core-primitives.md`, `85-relation-type-parity-spec.md`; semantic registry/Spec 144 | Register profiles and predicates once; no parallel graph authority |
| Proposals and their governed resolution | `41-proposal-resolution-engine.md`, Spec 136 | Reuse proposal, approval, commit and settlement owners |
| Evidence and receipts | Existing Evidence/Reference Store, Spec 119 and Spec 136 | Reference immutable producer records; do not copy a second outcome ledger |
| Agent identity and role | Spec 72 | Stable principal versus actor instance remain distinct |
| Presence, topology, placement, leases, handoff | Spec 139 | Extend its operational vocabulary and registered operations, not another presence service |
| Time, expiry and budgets | Spec 137/137A | Use its clock/freshness and budget authority |
| Instructions, capability, context and secret access | Spec 140, Spec 156, existing Context Authority | Treat received content as data; obtain exact target grants separately |
| Task membership and completion contracts | `149-focusa-workset-flow-ledger-and-release-completion-spec.md` | Keep each Workset project-scoped |
| Execution dependencies and eligible frontier | Spec 155; existing Work Loop | Cross-scope contributions use admitted child operations, not a global scheduler |
| Agent runtime and runs | Spec 133; existing team/fanout owners | Use protected runners and exact native session/run/generation |
| Intended architecture | Program Design Runtime | Reference design revisions; do not replace design ownership |
| Bounded context | Specs 100, 111, 140 | Project a minimal relevant network slice |
| Recovery and discovery | Spec 97 | Add narrow reflex routes to existing primitives |
| Generated tools and surface contracts | Spec 141 | One registered operation contract, generated consumers |
| Project-responsible role | Spec 182 | Foreman is an optional role projection, not another memory or permission owner |
| Proactive signals | Spec 183 | Changes become signals for investigation, not facts promoted by notification |
| Shared-mission meaning | Issue #503, reconciled by this proposed spec | MissionFederation is the narrow missing semantic aggregate; settlement still belongs to Spec 136 |

Number collisions in older documents are resolved by exact path and subject, not by assuming that a number uniquely identifies an owner. In particular, the two current Spec 149 titles describe different subjects.

**PNC-R004 — Single ownership.** Every contract, predicate, state transition, validation rule and persistence family introduced or extended here MUST identify one primitive owner and reuse its registered reducers, canonical constructors, authority and storage mechanism; clients MUST NOT maintain divergent authoritative copies.

**PNC-R005 — Design-versus-runtime truth.** Discovery and documentation MUST distinguish specified, source-present, installed, configured, operationally verified and unavailable capability; Spec 139 and Spec 182 draft declarations MUST NOT be presented as installed support.

**PNC-R006 — Scope isolation.** Every authority-bearing mutation MUST resolve the exact verified ScopeRef, WorkstreamKey and applicable attachment/execution identity; project names, local paths, common Git origins, shared machines, agent labels, mission IDs and UI selection MUST NOT establish or merge those identities.

## 4. Current source findings and implications

The inspected main baseline is `8cfb52eac5fdc34d14b80f8e6c51e7e6ab06a29a` (2026-10-31). Source observations are not live acceptance:

- `crates/focusa-core/src/scoped_state.rs` already defines ScopeRef, ProjectRootKey, WorkstreamKey and AttachmentKey.
- `crates/focusa-api/src/routes/ontology.rs` already lists typed object, provenance, status and link vocabularies, including `depends_on`, `blocks`, `verifies`, `implements` and `supersedes`. It is a large route module, not proof that every proposed semantic registry behavior is a portable core service.
- `crates/focusa-core/src/work_item/scheduler.rs` explicitly rejects cross-project dependencies; a regression test preserves that boundary. This spec does not remove that guard.
- Spec 139 already defines DependencyRelationship, WorkIntent, ActivityClaim, ResourceLease and PresenceHandoff. Its inspected grouped feature ledger has 37 `not_started` rows and requires further normalization; this is neither runtime proof nor a reason to duplicate its primitives.
- Issue #503 already proposes explicit many-to-many Mission Federation, two-sided admission and aggregate settlement. Its historical numbering is stale, not its scope-isolation rationale.
- Existing identity/goal bleed, wrong-workstream readiness and invocation-routing issues make isolation a prerequisite for autonomous delegation, but not for harmless public documentation or permitted read-only discovery.

External design inspiration was inspected at Busabase commit `a525be9c5e3d74a21eec022469f9973378a49448`: typed related records, reviewable changes, human-readable provenance and live/replay event interpretation. No external implementation is imported. Permission-dependent auto-merge is distinct from mandatory human review, and self-reported agent attribution is distinct from authenticated identity.

**PNC-R007 — Core/adapters boundary.** New shared semantics MUST live in the owning portable core/service contract, with thin API, CLI, harness and UI adapters; integration MUST avoid copying the ontology route's logic into the Pi extension or requiring a wholesale unrelated rewrite as a prerequisite.

## 5. Persistent relationship model

A network is a bounded authorized ObjectSet/projection over existing project identities, ontology links, contract artifacts and evidence. It is not another root scope, catalogue of all private projects, or source of child authority.

A relationship is expressed as a registered ontology link profile with these fields:

```text
relationship_id; revision; previous_revision_ref
source_scope_ref; target_scope_ref
registered_link_type_ref; semantic_profile_ref
source_object_ref; target_object_ref
contract_artifact_ref?; contract_revision_digest?
proposal_ref; source_acceptance_receipt_ref?; target_acceptance_receipt_ref?
binding_digest; status
provenance_refs[]; verification_refs[]
visibility_policy_ref; freshness_policy_ref
created_at; effective_at; superseded_by_ref?; revoked_at?
```

These fields are a profile of existing primitives, not a separate graph database schema owner. Cross-host wire representations use authorized opaque refs or exact verified identity references; a local filesystem path is placement metadata, not a globally unique project ID.

Initial semantic profiles include `depends_on_contract`, `provides_contract_to`, `validates_for`, `integrates_with`, and `complements`. They compile to existing predicates where equivalent. A new predicate needs a demonstrated semantic gap and canonical registry registration. Dependency edges distinguish **advisory**, **aggregate-blocking**, and **child-accepted execution** effects.

**PNC-R008 — Typed relationship contracts.** Relationship profiles MUST enforce typed endpoints, direction, multiplicity, semantic meaning, revision, evidence/promotion policies, freshness, visibility and lifecycle through the existing ontology/semantic registry, and MUST expose why the relationship exists.

**PNC-R009 — Discovery is advisory.** Manifest parsing, repository links, service references, deployment observations, model inference and similar goals MAY produce attributed candidates only; candidates MUST NOT acquire accepted membership, foreign visibility, scheduling effects or instruction authority by inference.

**PNC-R010 — Two-sided commitments.** A relationship asserting a mutual cross-scope commitment MUST become active only after independently authorized acceptance in both endpoint scopes over the same binding digest. Unilateral observations remain clearly unilateral and cannot falsely claim the other side's acceptance.

**PNC-R011 — Revision-safe edits.** Acceptance MUST bind exact endpoint identities, contract/dependency semantics, visibility and revisions; changes to those binding fields MUST invalidate affected acceptance and require revalidation. Concurrent conflicting decisions MUST enter governed conflict resolution rather than last-writer-wins.

**PNC-R012 — Relationship lifecycle.** Relationship states MUST support `candidate`, `proposed`, `partially_accepted`, `active`, `stale`, `conflicted`, `superseded` and `revoked`; missing acceptance cannot produce `active`, and revocation/supersession MUST retain history while fencing future use.

**PNC-R013 — Project-network projection.** Network views MUST be authorized, bounded, cursor-paged and revision/freshness-bearing; queries and exports MUST avoid revealing inaccessible endpoints, edge existence, labels or counts, and MUST distinguish an unreadable result from an empty graph.

## 6. Contract revisions and downstream impact

A contract is an existing versioned artifact or schema/interface object with an immutable digest and owner. Dependency semantics say which consumer relies on which declared compatibility range or exact revision. A producer's successful test does not establish consumer compatibility.

An impact record is an evidence/proposal profile referencing:

```text
accepted_relationship_revision
producer_contract_before_ref; producer_contract_after_ref
consumer_scope_ref; consumer_workstream_ref?
compatibility_policy_ref; relevant_acceptance_requirement_refs[]
source_evidence_refs[]; verification_request_exchange_ref?
consumer_verification_receipt_ref?; disposition; freshness
```

**PNC-R014 — Revision-specific impact.** Contract changes MUST identify the affected accepted relationship revisions and consumer obligations; a producer change MUST mark affected compatibility claims stale or pending revalidation under their policy, without inventing consumer failure or silently changing the consumer's project goal.

**PNC-R015 — Consumer proof.** Compatibility or downstream readiness MUST require the consumer authority's evidence for the exact declared artifact/contract and execution context; producer tests, message delivery, agent assertions and old-revision results MUST NOT satisfy that proof.

**PNC-R016 — Controlled dependency activation.** A network relationship MUST NOT change a child scheduler's frontier until that child accepts a corresponding external dependency through its existing Workset/CallGraph contract. Existing cross-project scheduler hard stops MUST remain intact.

## 7. Identity, presence and routing

The routing chain is:

```text
accepted visible relationship / exact explicitly authorized request
→ target project identity
→ target Workstream or responsible role
→ eligible actor attachment
→ current presence and placement
→ declared transport adapter
```

A stable role remains addressable while actor sessions come and go. A Foreman role is useful when implemented and authorized, but the protocol does not require a daemon-global Foreman or a permanently running model.

Supported topology classes are:

- multiple agents and projects under one authoritative daemon;
- remote workers controlled through an authorized execution/communication adapter, without a remote daemon;
- explicitly paired, separately authorized daemons under Spec 139 once that capability is actually verified;
- offline or unsupported endpoints with an inspectable queue or export path when policy permits, and no hidden execution fallback.

No topology mandates starting another daemon. The current deployment's authoritative-daemon policy remains unchanged.

**PNC-R017 — Stable identity versus presence.** Routing MUST distinguish principal, actor instance, role, project, Workstream, session/run/generation, machine, daemon and daemon epoch; missing or stale presence MUST render `unknown`, `stale` or `unavailable`, not proof of absence, release of a writer lease, or completion.

**PNC-R018 — Exact recipient selection.** Resolution MUST return one authorized exact recipient/role target or an explicit ambiguity/recovery result; session names, cwd, a daemon-global current project and cached UI focus MUST NOT silently select a mutation target.

**PNC-R019 — Transport neutrality.** The registered adapter contract MUST declare supported topology, authentication, delivery, cancellation, replay, maximum payload, timeout, platform and recovery semantics. Unsupported capability MUST be explicit; no platform-specific transport or public broker is a mandatory primitive dependency.

## 8. Coordination exchange protocol

`CoordinationExchange` is an operational protocol record owned with Spec 139's coordination/handoff surface and Spec 136's governed request/outcome lifecycle. It is not an alternative task, chat memory or settlement ledger.

An exchange carries:

```text
schema; exchange_id; revision; correlation_id; causation_ref?
source_scope_ref; source_workstream_ref?; source_actor_ref
source_authority_provenance_ref; authenticated_transport_principal_ref?
target_scope_ref; target_workstream_or_role_ref
target_actor_attachment_ref?; relationship_binding_ref?; mission_objective_ref?
intent: observe | request_status | propose_change | request_verification | offer_handoff
operation_descriptor_ref; expected_target_revision?
payload_artifact_ref; payload_digest; classification; disclosure_policy_ref
idempotency_key; issued_at; expires_at; temporal_policy_ref
reply_contract_ref; required_acceptance_refs[]; outcome_deadline_policy_ref?
request_receipt_ref?; recipient_receipt_ref?; execution_receipt_refs[]
status_projection; reason_code?; evidence_refs[]
```

The existing common event/result envelope provides actor, schema and scope fields rather than retyping a parallel envelope. A human-friendly message may accompany this record, but it is not the authority-bearing contract.

**PNC-R020 — Authenticated provenance.** The receiver MUST separate transport-authenticated identity, verified authority/delegation, and self-reported agent/tool labels; a message header, SSH account or possession of a transport connection MUST NOT substitute for target capability or architecture authority.

**PNC-R021 — Bounded exchanges.** An exchange MUST bind source, exact target, intent, payload digest/reference, disclosure policy, expiry, correlation, idempotency and reply/acceptance contract; payload and fanout budgets MUST be enforced before transport.

**PNC-R022 — Delivery truth.** Exchange state MUST distinguish `prepared`, `queued`, `transport_accepted`, `received`, `recipient_accepted`, `rejected`, `expired`, `cancel_requested`, `cancelled`, `outcome_reported`, `outcome_verified` and `unknown`; a transport acknowledgement MUST NOT imply model consumption, accepted work, execution, verified outcome or settlement.

**PNC-R023 — Durable delivery and deduplication.** Outbound intent and inbox acceptance MUST be durably recorded through the existing persistence/outbox owners before claiming their respective states; duplicate messages MUST reuse semantic request identity, preserve receipts and prevent duplicate consequential target effects.

**PNC-R024 — Loss, reordering and reconciliation.** Timeout, crash, partition, duplicate delivery and out-of-order receipts MUST reconcile by correlation, revision and target effect evidence before retry; unknown effects MUST remain unknown until reconciled, and later transport events cannot regress verified terminal truth.

**PNC-R025 — Cancellation semantics.** Cancellation MUST distinguish withdrawal before target admission, a request to stop admitted work, and a confirmed stop/terminal outcome; revocation or cancellation cannot erase prior effects, forge rollback or imply that an unreachable executor stopped.

**PNC-R026 — Recipient admission.** Receiving a request MUST NOT create a child task, session, execution run, budget or credential grant unless the target's current operation-specific authority admits that effect; legitimate permitted read-only exchanges MUST NOT require a full executable trajectory as an artificial prerequisite.

## 9. Handoffs and execution boundaries

A handoff extends the existing `focusa.presence_handoff.v1` contract. It references exact source/target identities, the canonical Workpoint and action, accepted boundaries, required evidence, resource footprint and receipts. Cross-project handoff normally creates a separately admitted child assignment; it is not transfer of another project's mutable cognition.

**PNC-R027 — Handoff ownership.** Execution handoff MUST preserve the existing handoff owner, exact Workpoint/action or separately admitted child assignment, native run/generation, required acceptance, writer lease and fencing semantics; old executors MUST remain fenced after an authorized ownership transition.

**PNC-R028 — Existing executors only.** Admitted work MUST run through existing Work Loop, CallGraph, Silent Session/team/fanout and protected-runner contracts with target-owned budgets and leases; this layer MUST NOT introduce a global scheduler, cross-project Workpoint, shared Workset, uncontrolled subprocess spawn or terminal-input injection.

## 10. Mission Federation: narrow new semantic aggregate

Issue #503 identifies a real missing concept: persistent identity and acceptance for one explicitly shared mission involving independently governed project contributions. A persistent relationship graph alone does not own that mission's meaning.

`MissionFederationKey = coordination_realm_ref + mission_id` is **coordination identity, not a ScopeRef**. The registered mission coordinator owns the approved mission definition, membership proposals, objective/contribution references and aggregate projection. It has no implicit permission to mutate any child. Its own revisions require owner-rooted governance; peer trust follows Spec 139 rather than a new consensus scheme.

The proposed field contracts are:

- **MissionFederation:** `mission_id`, `realm_ref`, `revision`, `previous_revision_ref`, `title`, `mission_statement`, `desired_outcome_refs`, `constraint_refs`, `objective_refs`, `member_refs`, `completion_contract_ref`, `governance_policy_ref`, `visibility_policy_ref`, `created_by`, timestamps, digest and receipt refs.
- **MissionObjective:** stable ID, exact mission revision, desired outcome, mandatory/optional classification, contribution refs, dependency refs, acceptance refs, settlement-policy ref and freshness-bearing status projection.
- **MissionMemberBinding:** binding ID, exact mission revision, member ScopeRef/ProjectIdentity digest, role, objective contributions, exact two-sided acceptance refs/digest, lifecycle, visibility policy, and `authority_transfers=false`, `permission_transfers=false`, `credential_transfers=false`.
- **MissionWorkstreamBinding:** separately accepted exact WorkstreamKey, parent member binding, objective refs, child execution refs, acceptance digest/receipts, context policy and `writer_lease_transfers=false`.
- **MissionAlignmentClaim:** exact objective, member, Workstream, project HLT/spec/design revisions, child Workset/CallGraph/run/Workpoint refs and child validation receipt.
- **MissionDependency:** exact endpoint objective/contribution refs, registered dependency semantics, `advisory | aggregate_blocking | child_accepted` effect, child acceptance receipts and freshness.
- **MissionDispatchProposal:** exact mission/objective/member/Workstream/execution revisions, requested registered operation/effect, required admission references, idempotency, expiry and child admission/result receipts. It is not an execution permit.
- **MissionExecutionBinding:** read-only relation to exact child Worksets, graph definitions/revisions/runs/frames, Workpoints and assignments, with source validation receipts.

These are composed typed records of one semantic aggregate, not eight new authority engines. Their exact schemas are registered under the existing Operation/semantic registry before implementation admission; this authoring candidate does not claim those schema registrations exist today.

**PNC-R029 — Federation identity and cardinality.** Mission Federation MUST support many projects, many explicitly selected Workstreams per project, many child graphs/runs and many participating actors/hosts/daemons without assuming a one-to-one mapping; federation identity MUST NOT become a root scope, project HLT, writer lease or child mutation authority.

**PNC-R030 — Explicit member and Workstream acceptance.** Each mission member MUST have mission-side proposal and independently authorized project-side acceptance over the same exact binding; each participating Workstream MUST be separately admitted. Scope membership cannot implicitly admit all present or future Workstreams.

**PNC-R031 — Alignment and steering.** Mission alignment MUST reference exact project-owned goal/spec/design revisions; changes MUST invalidate affected alignment without overwriting child HLTs. Mission steering may prioritize proposals but MUST NOT override child operators, constitutions, stop directives or reserved decisions.

**PNC-R032 — Independent child dispatch.** Every federation dispatch MUST independently resolve and admit one exact child target and execution binding; fanout MUST consist of independently admitted child lanes with isolated grants, budgets, credentials, contexts, runs and receipts. Mission ID alone MUST fail target admission.

**PNC-R033 — Aggregate settlement.** Federation completion MUST be evaluated by the existing settlement authority against the approved aggregate completion contract and exact visible child evidence/receipts; stale, revoked, missing, conflicting, failed or indeterminate mandatory contributions MUST prevent a successful aggregate claim without rewriting or settling child objects.

**PNC-R034 — Independence and revocation.** A blocked member MUST NOT block unrelated eligible contributions unless an accepted dependency requires it; member/Workstream revocation MUST fence future use and active contribution while retaining historical outcomes and evidence, and cannot silently reduce the approved mandatory completion contract.

## 11. Security, privacy and instruction integrity

The network is not a shared global memory. Relationship visibility, message delivery, knowledge access, authority to propose, authority to accept, and authority to execute are separate permissions.

**PNC-R035 — Minimal disclosure.** Cross-scope projections and exchanges MUST apply the intersection of source, destination and operator visibility policies, classify/redact sensitive references before transport, and omit secrets, tokens, raw private transcripts, private endpoints and unauthorized context. Authorized secret use remains within the existing broker, not message payloads.

**PNC-R036 — Data is not instruction authority.** Received project descriptions, playbooks, relationship labels, evidence, messages and provider text MUST remain attributed data unless promoted through existing instruction-authority procedures; they MUST NOT add members, change goals, grant permissions or inject foreign executable next actions.

**PNC-R037 — Trust and replay protection.** Cross-daemon operations MUST use Spec 139's verified realm/principal, key possession, epoch, revocation and replay contracts; cryptographic integrity MUST NOT be confused with authorization, truth or valid consumer proof. Unsupported or unverified federation transport cannot silently downgrade to trusted mutation.

## 12. Persistence, eventing and recovery

Storage belongs to the existing ontology, proposal, event, receipt, session and settlement owners. A coordination inbox/outbox is an operational read/write model of exchange records, not another source of task completion. An accepted relationship changing two independently governed scopes is not one distributed database transaction: proposals, local commits, receipts, compensation and reconciliation expose partial acceptance.

**PNC-R038 — Durable source and derived views.** Canonical transitions MUST use the existing append-only/reducer-backed persistence contracts; graph/topology/status views MUST be reconstructible and versioned, with materialization revision and freshness. No shared SQLite file or private client cache may become cross-daemon consensus or canonical mission state.

**PNC-R039 — Partial commits and recovery.** Cross-scope operations MUST expose partial acceptance and recovery state without claiming distributed atomicity; local atomic changes and intent/outbox recording MUST follow their existing transaction boundary, and failed compensation MUST remain a visible unresolved effect.

**PNC-R040 — Replay parity.** Live and persisted replay MUST use the same registered event interpretation/state transitions and produce equivalent authorized semantic projections; unknown supported-extension fields/events MUST be retained or handled according to compatibility policy without fabricating progress, and unknown authority-bearing semantics MUST fail closed.

**PNC-R041 — Bounded resources.** Reads, subscriptions, fanout, inboxes/outboxes, retry attempts, payloads, retained events and graph traversal MUST have explicit enforced budgets, cursors, expiry and backpressure; policy-controlled coalescing MUST preserve consequential evidence and uncertainty. Low-memory mode cannot waive mandatory truth or safety.

## 13. Operations and portable interface contract

Logical operation families, not promises that named tools already exist:

| Logical operation | Effect and owner |
|---|---|
| `project_network.view / explain / candidates` | Authorized ontology/ObjectSet projection and advisory discovery |
| `project_network.relationship.propose / accept / revoke / history` | Existing proposal/ontology/receipt owners; endpoint-specific authority |
| `project_network.contract.impact / verification.request` | Existing artifact/evidence/verification and exchange contracts |
| `coordination.recipient.resolve / exchange.preview / send / inspect / cancel` | Spec 139 operational coordination; independent target admission |
| `coordination.handoff.offer / accept / inspect` | Existing PresenceHandoff and executor authorities |
| `mission_federation.create.preview / commit / revise / view / history` | Governed mission definition, not child mutation |
| `mission_federation.member.propose / accept_in_project / revoke` | Exact member binding and receipt |
| `mission_federation.workstream.propose / accept_in_workstream` | Separate exact Workstream binding |
| `mission_federation.alignment / dependency / dispatch.preview / admit_in_project` | Existing child-owned validation/admission |
| `mission_federation.settlement.evaluate / doctor` | Existing settlement and bounded diagnosis |

Every operation descriptor identifies read/proposal/canonical/external-effect class, input/output schema, actor/scope requirements, capability and visibility policy, revision preconditions, receipt/evidence behavior, error taxonomy, cost limits and recovery route. Generated route/tool/CLI names derive from those approved descriptors rather than hand-maintained copies.

**PNC-R042 — Generated surface parity.** API, CLI, Pi and other harness tools, MCP descriptors, generated clients and applicable UI surfaces MUST derive from the single approved operation/semantic registry and preserve exact error details, authority requirements, revision and receipt semantics; direct tools and child/background dispatch MUST resolve the same verified identity.

**PNC-R043 — Honest degraded behavior.** Unavailable presence, transport, daemon, registry, browser or UI capability MUST produce operation-specific supported fallback or an explicit resumable unavailability result; harmless allowed inspection cannot be globally blocked by unrelated execution admission, while degraded modes MUST NOT grant weaker authority or claim missing capability is complete.

Error construction reuses the canonical error envelope. Required semantic reasons include unresolved identity/target, recipient ambiguity, revision conflict, missing endpoint acceptance, inaccessible evidence, stale presence/epoch, revoked binding/grant, unsupported transport, expired exchange, unknown effect and missing consumer proof. Existing equivalent reason codes are reused; new ones are registered once.

## 14. Human and agent experience

A project detail view explains what the project provides, consumes, validates and currently owes; why each relationship exists; which revision was accepted; who is responsible; and what evidence is missing. The activity view distinguishes proposal, delivery, acceptance, execution and verification. Review shows a field-level diff and scope consequences. Permission-dependent low-risk automation is possible; every routine write need not cause a human approval ritual.

**PNC-R044 — Inspectable review and presentation.** Applicable surfaces MUST show understandable relationship/mission meaning, proposed versus accepted state, exact revision, attribution versus authenticated principal, freshness, proof gaps, pending commitments and recovery actions; selection or visualization MUST NOT silently change the active child attachment or authority.

**PNC-R045 — Bounded context and continuity.** Agent bootstrap, compaction/resume and model/harness changes MUST preserve only the relevant authorized network/mission slice with source revisions and rehydrate references; foreign member goals, Workpoints and next actions MUST NOT become the recipient's own executable continuation. Foreman/role continuity cannot depend on hidden model memory.

## 15. Migration and compatibility

Existing repository links, issue umbrellas, systemic Program Designs, multi-project Canvas groups, descriptions and chat exchanges are potential evidence—not accepted network membership. Current direct Intercom/SSH use remains usable independently and is not retroactively called the new protocol.

**PNC-R046 — Safe migration.** Existing material MUST import as attributed candidates/observations unless exact current acceptance is proven; migration MUST preserve project/workstream IDs, historical receipts and child authority, and MUST NOT silently enroll projects, start peers, transfer writers or convert chat acknowledgements into execution evidence.

**PNC-R047 — Version negotiation and rollback.** Schemas and adapters MUST support explicit capability/version negotiation, tested old/new producer-consumer behavior and additive compatibility where declared; unknown authority fields MUST NOT widen access. Rollback MUST stop new dispatch, retain receipts and reconcile in-flight effects without erasing canonical history or silently re-enabling revoked bindings.

## 16. Acceptance matrix

The companion proof matrix records test handles and required evidence classes, with no passing runtime claim. Minimum integrated scenarios are:

| Scenario | Consumer-visible outcome |
|---|---|
| Two related projects, no shared mission | Inspect an accepted contract relationship and explain evidence without changing either HLT |
| Contract revision change | Producer publishes an exact revision; affected consumer is asked to reverify; stale proof is not reused |
| Cross-machine exchange | Actual recipient receives and accepts a bounded request; independent execution and verification receipts remain distinguishable |
| Native agent restart/model switch | The role rehydrates relevant state; routing finds the new attachment without recreating project identity |
| Duplicate, timeout and out-of-order events | No duplicate consequential effect; uncertainty and reconciliation are visible |
| Cancellation during execution | Stop request, actual stop, prior effects and unresolved remote status remain distinct |
| One daemon, multiple projects/Workstreams | No goal, context, invocation, lease or receipt bleed |
| Remote worker without remote daemon | Supported adapter proves exact dispatch and outcome without inventing a second authority |
| Paired-daemon federation | Real authorized multi-daemon proof only when that profile is activated; partitions fence unsafe effects |
| Many-to-many mission | At least four independent projects, two Workstreams in one project, multiple Worksets/graphs/runs and multiple actors; no graph/frontier merging |
| Mission alignment revision/revocation | Affected bindings become stale/revoked; history remains; mandatory acceptance is not silently removed |
| Aggregate settlement | Exact child receipts are consumed; a missing mandatory receipt blocks mission completion |
| Adversarial content and disclosure | Forged identity, prompt injection, inaccessible edge enumeration, stale grants/epochs and replay cannot cross boundaries |
| Mixed versions and supported platforms | The same semantics hold on supported OS/harness/API/UI profiles; unsupported features remain explicit |
| Live/replay | Reconnection yields the same semantic state as live observation, with version/freshness provenance |

**PNC-R048 — Full proof standard.** Every activated requirement MUST have versioned contract, producer, actual receiving-consumer, declared cross-version and live end-to-end evidence on each claimed environment; fixtures, schema checks, message acknowledgements, installation or source tests alone MUST NOT establish installed outcome or whole-spec completion.

**PNC-R049 — Inherited acceptance preservation.** Adopting this spec MUST reconcile and map the full issue #503 acceptance and closure prohibitions, applicable Spec 139/144 requirements and all activated owner obligations without silently dropping them; spec authoring, a smaller pilot or a graph UI MUST NOT close those broader issues or claim their full conformance.

## 17. Decomposition readiness, later implementation and activation

The intended dependency order is conceptual, not a newly admitted task queue:

1. source/owner reconciliation and approved requirement coverage;
2. exact scope and identity reuse, typed relations and revision-bound acceptance;
3. bounded network discovery/projection and contract-impact verification;
4. exchange persistence, transport contracts, replay and target admission;
5. handoff/runner integration and independently admitted child work;
6. mission membership/alignment and aggregate settlement;
7. cross-surface, migration, adversarial, platform and installed proof.

Read-only network slices can become useful before automated delegation is activated. Verified slices stay labeled as slices; this ordering never waives the full mandatory criteria.

**PNC-R050 — Decomposition gate.** Before implementation task decomposition or admission, the approved package MUST contain hash-bound normative source coverage, clause-level requirement/ownership/dependency mapping, exact resolved schema and operation registry bindings, cross-spec amendments, migration/rollback and surface/proof matrices; no mandatory row may be orphaned, weakened, mapped only to prose follow-up, or falsely marked implemented. Task creation and execution admission remain separate later actions.

**PNC-R051 — No premature activation.** This documentation package MUST NOT create implementation tasks, live network/federation records, agents, grants, schedules, budgets, peer connections, releases or issue closures; future implementation MUST obtain its own approved scope and actual runtime admission after the release priority is reconciled.

**PNC-R052 — Whole-outcome closure.** Full conformance MUST require all active mandatory rows and inherited obligations to be verified with current producer and consumer evidence; missing, stale, inaccessible, conflicted or unimplemented proof MUST remain visible and prevent an unqualified completion claim.

## 18. Cross-spec amendment register

| Owner | Proposed additive reconciliation | Prohibited change |
|---|---|---|
| Ontology/semantic registry | Project/contract relationship profiles, endpoint acceptance and visibility semantics | Parallel relationship registry or inferred canonical links |
| Spec 139 | Exchange protocol profile, stable recipient resolution and existing handoff extensions | Second presence service or transport-created authority |
| Spec 136/PRE | Relationship/member proposals, partial cross-scope acceptance and aggregate mission outcome profile | New approval/settlement engine or fabricated global transaction |
| Workset/CallGraph | Exact child-accepted external dependency and contribution receipt bindings | Removing cross-project scheduler isolation |
| Spec 140/100/111 | Bounded foreign-data context profile with instruction/visibility boundaries | Global memory or foreign next-action injection |
| Spec 141 | Generated operation/error/discovery surfaces | Hand-maintained client authority contracts |
| Spec 182 | Optional target role resolution and hydration references | Mandatory always-running agent or new Foreman memory |
| Spec 183 | Revision-aware dependency impact signals | Notification directly promotes truth or schedules work |
| Issue #503 | Current numbering, explicit relationship-network distinction, native mission aggregate and complete acceptance mapping | Paper closure or replacement by a smaller dashboard feature |
| Current release workset | No new requirement; retained priority and acceptance unchanged | Making this future capability a release blocker |

## 19. Package state and explicit remaining approval work

This candidate includes a requirement ledger, primitive ownership matrix, operation/cross-surface matrix, proof matrix, schema field-contract catalogue, and hash-bound source manifest. They describe intended work and are not live registry registrations, implementation evidence or approval receipts.

Remaining before decomposition: owner review of the exact candidate; canonical document-number allocation; exact schema-ID/operation-ID resolution against the current registries; clause-level mapping of activated inherited requirements (including all issue #503 acceptance scenarios); reconciliation of current source changes; and recorded approval of the complete source/contract digest set. These are visible authoring/approval gates, not hidden implementation deferrals or a claim that the draft is ready for execution today.

## 20. Final principle

**Projects keep their own truth; relationships explain their obligations; messages carry bounded proposals and evidence; only the owning authority admits work and accepts its outcome.**
