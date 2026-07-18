# Spec 135F — Domain-General Ontology Core, Semantic Graph, Domain Packs, and Reactive Context

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135F.  
**Scope:** core-owned ontology registry, shared cognition pack, domain packs, candidate/canonical semantic graphs, verification and promotion policies, ontology-derived Workpoint candidates, generalized slice policies, semantic delta subscriptions, V1 compatibility projections, snapshot/event compatibility, migration, conformance, isolation, and cross-client generated contracts.

---

## 0. One-line definition

Focusa should operate one versioned, reducer-governed semantic substrate in which shared cognition primitives and composable domain packs define typed working worlds for software, legal, markets, research, general, custom, and composite projects without creating separate runtimes or allowing projections, models, connectors, or clients to mint canonical truth.

---

## 1. Why this companion exists

Spec 135 requires one canonical runtime to support multiple professional domains. Workspace profiles alone cannot provide that guarantee.

A Workspace View Profile can change:

- terminology;
- layout;
- visual grammar;
- renderer selection;
- panel hierarchy;
- evidence emphasis.

It cannot by itself define:

- what domain entities exist;
- how entities relate;
- which actions are valid;
- which evidence proves a claim or outcome;
- which statuses and transitions are legal;
- which semantic objects belong in a bounded working set;
- how semantic changes become canonical;
- how agents react to semantic deltas.

Those responsibilities require a shared domain-semantic substrate beneath the workspace and Project Operating Profile layers.

---

## 2. Normative basis

This spec implements and composes rather than replaces:

- [45 Ontology Overview](45-ontology-overview.md);
- [46 Ontology Core Primitives](46-ontology-core-primitives.md);
- [47 Software World Ontology](47-ontology-software-world.md);
- [48 Ontology Links and Actions](48-ontology-links-actions.md);
- [49 Working Sets and Slices](49-working-sets-and-slices.md);
- [50 Ontology Classification and Reducer](50-ontology-classification-and-reducer.md);
- [61 Domain-General Cognition Core](61-domain-general-cognition-core.md);
- [67 Query Scope and Relevance Control](67-query-scope-and-relevance-control.md);
- [70 Shared Interfaces, Statuses, and Lifecycle](70-shared-interfaces-statuses-and-lifecycle.md);
- [72 Agent Identity, Role, and Self Model](72-agent-identity-role-and-self-model-ontology.md);
- [74 Identity and Reference Resolution](74-identity-and-reference-resolution.md);
- [75 Projection and View Semantics](75-projection-and-view-semantics.md);
- [77 Ontology Governance, Versioning, and Migration](77-ontology-governance-versioning-and-migration.md);
- [88 Ontology-backed Workpoint Continuity](88-ontology-backed-workpoint-continuity.md);
- [100 Context Cognition](100-context-cognition-spec.md);
- [109 Agent-first API Redesign](109-agent-first-api-redesign-ax-spec.md);
- [111 Agent Context Bootstrap and Delivery](111-agent-context-bootstrap-and-delivery-spec.md);
- [125 Mandatory Trajectory and Ontology Interlock](125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md);
- [130 Compaction Mission Packet and Context Firewall](130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md);
- [133 Durable Silent Sessions](133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md);
- Spec 135A workspace projection;
- Spec 135B C.R.I.S.T. Project Genesis;
- Spec 135C UIAI artifact and research integration;
- Spec 135D implementation and no-deferral law;
- Spec 135E compatibility and migration law.

Primitive-owning specs retain ownership of their internal semantics. Spec 135F owns their integrated domain-general implementation contract for the Spec 135 product path.

---

## 3. Current codebase reality

### 3.1 Implemented foundations

The current repository includes:

- `OntologyState` inside `FocusaState`;
- reducer-visible object, link, status, membership, proposal, promotion, rejection, verification, and working-set events;
- persisted ontology objects, links, proposals, verification records, working-set refresh records, and delta logs;
- ontology world, primitive, contract, slice, adjacency, working-set, context, affordance, retrieval, critic, reflection, memory, and action routes;
- a broad catalog of software, visual, cognition, identity, projection, governance, and execution names;
- Pi turn-time retrieval of bounded ontology context;
- SQLite event and state persistence;
- Workpoint, Trajectory, Evidence, Context Cognition, and SSE foundations.

### 3.2 Partial foundations

The current implementation is useful but incomplete:

- object and relation state use generic JSON values;
- most type, action, relation, and status definitions are route-local string catalogs;
- proposal records do not uniformly require evidence, confidence, expiry, schema version, or typed property payloads;
- candidate and canonical objects share collections and are commonly distinguished by status strings;
- verification and promotion are not uniformly coupled through enforceable policies;
- Workpoints are ontology-shaped but are substantially caller-authored rather than reproducibly derived from a canonical semantic revision;
- working sets are useful projections but not yet a complete core-owned typed ObjectSet/SlicePolicy substrate;
- current semantic consumption is primarily pull-based turn context rather than a durable semantic subscription system;
- snapshot and event readers are not sufficiently tolerant of unknown future authoritative event types;
- current state versioning does not cleanly separate reducer revision from persisted schema version.

### 3.3 Required result

This spec strengthens the existing ontology without invalidating current routes, names, records, Workpoints, Pi behavior, or specifications.

---

## 4. Authority model

```text
Source adapter / parser / model / UIAI / agent
                    │
                    ▼
            Candidate semantic graph
                    │
           evidence + verification policy
                    │
                    ▼
             Reducer promotion gate
                    │
                    ▼
            Canonical semantic graph
                    │
        ┌───────────┼─────────────┐
        ▼           ▼             ▼
 bounded slices  Workpoint      V1 compatibility
 and context     candidates     projection
```

### 4.1 Registry authority

The registry defines valid semantic contracts. It does not contain project facts.

### 4.2 Candidate authority

Candidate state preserves proposed, inferred, observed, imported, or unresolved semantic material. Candidate state is not canonical action authority.

### 4.3 Canonical authority

Only reducer-approved writes that satisfy the applicable promotion policy become canonical semantic state.

### 4.4 Workpoint authority

The ontology may derive a Workpoint candidate. The existing Workpoint reducer path remains the immediate continuation and next-action authority.

### 4.5 Projection authority

Workspace views, Context Cognition, Project Cards, Pi Focus Slices, dashboards, and V1 compatibility output are projections. They may not mutate truth by being rendered or selected.

---

## 5. Hard design laws

1. One Focusa runtime; no per-domain cognitive cores.
2. One core-owned semantic registry; no route-local or client-local competing registries.
3. Shared cognition and domain-specific semantics remain separate layers.
4. Candidate and canonical semantic state remain structurally distinct.
5. No canonical promotion without the registered policy result.
6. Evidence records are referenced, stable, scoped, and independently auditable.
7. Workspace selection is not domain-pack activation unless explicitly previewed and committed.
8. Domain-pack activation is not permission escalation.
9. Role is not permission; domain expertise is not operational authority.
10. Workpoint remains continuation authority after governed promotion.
11. Unknown semantic IDs and events are preserved, never guessed.
12. Existing V1 behavior remains available through an explicit compatibility projection.
13. Migration never silently converts unreadable state into fresh empty state.
14. Older incompatible writers are blocked from mutating newer canonical state.
15. Semantic subscriptions produce observations or proposals unless an existing policy explicitly authorizes more.
16. UI invalidation events are not semantic promotion events.
17. Domain packs are composable and project-scoped.
18. Cross-domain links require declared compatibility and authority.
19. Large artifacts and corpora remain externalized behind stable references.
20. All hot context and graph traversals are bounded.

---

## 6. Required terminology

### Shared Cognition Pack

The versioned implementation of domain-neutral Mission, Goal, Task, Decision, Constraint, Risk, Blocker, OpenLoop, WorkingSet, ActionIntent, Verification, Checkpoint, and EvidenceArtifact semantics.

### Domain Pack

A versioned package extending the shared cognition pack with domain-specific object, relation, action, status, verification, slice, artifact-interpretation, and migration definitions.

### Semantic Registry

The resolved set of shared and active domain definitions available to a project scope.

### Candidate Graph

Typed proposed, observed, inferred, imported, unresolved, contradicted, or pending semantic state.

### Canonical Graph

Reducer-promoted semantic state satisfying registered identity, verification, authority, and lifecycle rules.

### Verification Policy

A registered policy that defines the evidence, verifier, freshness, authority, independence, and outcome requirements for a promotion or completion claim.

### Promotion Policy

Rules determining whether a verified candidate may become canonical, remain advisory, require operator approval, or be rejected.

### Slice Policy

A versioned bounded-selection contract defining which semantic objects, links, evidence, uncertainty, and actions may enter a purpose-specific context projection.

### Semantic Delta

A versioned, scoped, replayable description of candidate or canonical graph change.

### Semantic Subscription

A durable cursor-based filter over semantic deltas used by runtimes, agents, or services to refetch bounded state or propose reactions.

### V1 Compatibility Projection

The preserved legacy objects, links, action names, statuses, routes, slice names, and output shapes derived from V2 state where possible and retained directly where not yet migrated.

---

## 7. Core semantic registry

The authoritative registry belongs in `focusa-core`.

### 7.1 Object type definition

```yaml
schema: focusa.object_type_definition.v2

type_id: focusa.core/task@1
legacy_names:
  - task
domain_pack_id: focusa.core.cognition
version: 1
id_strategy:
required_properties: []
optional_properties: []
allowed_link_type_ids: []
allowed_action_type_ids: []
status_vocabulary_id:
identity_policy_ref:
validation_policy_ref:
deprecated: false
replacement_type_id:
```

### 7.2 Link type definition

```yaml
schema: focusa.link_type_definition.v2

link_type_id: focusa.core/depends_on@1
legacy_names:
  - depends_on
source_type_ids: []
target_type_ids: []
multiplicity:
directionality:
evidence_policy_ref:
promotion_policy_ref:
identity_policy_ref:
reversible:
deprecated: false
replacement_link_type_id:
```

### 7.3 Action type definition

```yaml
schema: focusa.action_type_definition.v2

action_type_id: focusa.core/complete_task@1
legacy_names:
  - complete_task
target_type_ids: []
input_schema_ref:
output_schema_ref:
precondition_policy_refs: []
permission_policy_refs: []
side_effect_classes: []
verification_policy_ref:
promotion_policy_ref:
rollback_policy_ref:
emitted_event_type_ids: []
tool_mapping_refs: []
idempotency_policy_ref:
timeout_policy_ref:
retry_policy_ref:
deprecated: false
replacement_action_type_id:
```

### 7.4 Status and lifecycle definition

```yaml
schema: focusa.status_vocabulary.v2

status_vocabulary_id:
statuses: []
allowed_transitions: []
terminal_statuses: []
verified_statuses: []
advisory_statuses: []
legacy_aliases: {}
```

### 7.5 Registry laws

- Definitions are immutable by ID/version after release.
- Changes create a new version and compatibility declaration.
- Legacy names remain aliases until a governed deprecation closes.
- Registry lookup is deterministic and project/domain-pack scoped.
- Generated API/client contracts come from the same definitions.
- Unknown definitions remain unsupported references.
- A route, UI, connector, or model cannot invent a new canonical type on demand.

---

## 8. Domain packs

### 8.1 Manifest

```yaml
schema: focusa.domain_pack_manifest.v1

pack_id: focusa.software
pack_version: 1
compatibility_version: 1
minimum_core_version:
minimum_registry_version:
extends:
  - focusa.core.cognition@1
object_type_ids: []
link_type_ids: []
action_type_ids: []
status_vocabulary_ids: []
verification_policy_ids: []
promotion_policy_ids: []
slice_policy_ids: []
artifact_interpretation_policy_ids: []
identity_policy_ids: []
migration_refs: []
legacy_aliases: {}
security_classification:
license:
```

### 8.2 Required built-in packs

```text
focusa.core.cognition@1
focusa.general@1
focusa.software@1
focusa.legal@1
focusa.markets@1
focusa.research@1
focusa.custom@1
```

Composite projects may activate multiple compatible packs.

### 8.3 Composition

```text
shared cognition pack
→ required vertical packs
→ optional domain overlays
→ project semantic extensions approved through governance
→ resolved project semantic registry
```

### 8.4 Workspace relationship

Workspace profiles may declare:

- required domain packs;
- recommended domain packs;
- semantic renderer bindings;
- terminology mappings;
- degraded behavior when a pack is absent.

Workspace profiles may not redefine canonical semantic contracts.

### 8.5 Operational policy relationship

A domain pack may describe possible actions. Permission and operational policy determine whether the active actor may execute them.

### 8.6 Custom packs

Custom packs require:

- schema validation;
- namespace ownership;
- compatibility declaration;
- preview;
- migration plan;
- operator approval;
- import/export classification;
- conformance tests;
- explicit fallback behavior.

Dynamic native-code plugins are not required for the first implementation. Declarative or statically compiled packs are preferred until a stable plugin ABI is explicitly governed.

---

## 9. Candidate and canonical semantic graphs

### 9.1 Candidate record

```yaml
schema: focusa.semantic_candidate.v2

candidate_id:
semantic_kind: object | link | status_change | membership | action_result | identity_resolution
semantic_id:
definition_id:
domain_pack_id:
payload:
source_ref:
provenance_refs: []
evidence_refs: []
confidence:
freshness:
expires_at:
scope:
  project_root:
  continuity_id:
  workpoint_id:
status: proposed | pending_verification | verified_candidate | rejected | expired | superseded
created_at:
updated_at:
```

### 9.2 Canonical record

```yaml
schema: focusa.canonical_semantic_record.v2

semantic_id:
definition_id:
domain_pack_id:
revision:
payload:
status:
identity_resolution_ref:
promotion_record_ref:
verification_record_refs: []
provenance_refs: []
scope:
  project_root:
  continuity_id:
created_at:
updated_at:
```

### 9.3 Structural separation

Candidate and canonical records use separate stores and indexes. A status field alone is insufficient separation.

### 9.4 Promotion

Promotion creates or revises a canonical record and retains the candidate, evidence, verification, and promotion history.

### 9.5 Rejection and expiry

Rejected, expired, and superseded candidates remain auditable but do not appear as canonical truth. Retention and pruning follow declared policy.

### 9.6 Identity

Canonical identity resolution occurs before promotion where ambiguity exists. Duplicate IDs, aliases, and cross-pack equivalence use Spec 74 records rather than string coincidence.

---

## 10. Verification and promotion policies

### 10.1 Verification policy contract

```yaml
schema: focusa.verification_policy.v1

policy_id:
applies_to_definition_ids: []
required_evidence_kinds: []
required_verifier_classes: []
minimum_independent_sources:
freshness_limit:
operator_approval_required:
automated_check_ids: []
allow_degraded_result:
positive_outcomes: []
negative_outcomes: []
```

### 10.2 Verification record

```yaml
schema: focusa.semantic_verification_record.v2

verification_id:
policy_id:
target_candidate_id:
target_semantic_id:
method:
verifier_ref:
evidence_refs: []
outcome: passed | failed | inconclusive | stale | blocked
confidence:
performed_at:
expires_at:
```

### 10.3 Promotion record

```yaml
schema: focusa.semantic_promotion_record.v2

promotion_id:
candidate_id:
semantic_id:
promotion_policy_id:
verification_record_refs: []
operator_approval_ref:
result: promoted | advisory_only | deferred | rejected
reason:
canonical_revision:
created_at:
```

### 10.4 Default behavior

- New V2 actions default to `auto_verify: false`.
- A generic string such as `accepted` is not proof unless a compatibility policy explicitly classifies it as legacy advisory verification.
- Promotion checks verification policy before mutation.
- Existing V1 verified entries retain their historical status with a trust classification such as `legacy_assumed`, `legacy_route_verified`, `evidence_backed`, or `operator_confirmed`.
- Historical state is not retroactively deleted because later policies are stronger.

### 10.5 Domain examples

```text
software.test_verified_completion
legal.source_and_authority_verified
markets.research_only_claim
research.multi_source_claim
context.operator_accepted_claim
artifact.scope_provenance_verified
```

---

## 11. V1 compatibility projection

### 11.1 Preserved public behavior

The following remain compatible until a versioned deprecation closes them:

- current object and link short names;
- current action names;
- current status strings;
- `/v1/ontology/*` route families;
- current world/slice/context projection behavior;
- current software slice names;
- current Pi bounded ontology context consumption;
- current Workpoint fields and routes;
- current JSON consumers that ignore additive fields.

### 11.2 Legacy fields

Existing `OntologyState.objects`, `links`, `proposals`, `verifications`, `working_set_refreshes`, and `delta_log` remain readable. During transition they are retained or generated as an explicit V1 projection.

### 11.3 Dual operation

Migration stages may use:

```text
V1 read + V1 write
→ V1 read + dual write/shadow V2
→ V1 projection from V2 + verified equivalence
→ V2 canonical write + V1 compatibility read
```

No stage may silently change existing client authority.

### 11.4 Unknown values

Unknown V2 definitions appear in V1 output only as explicit unsupported references or opaque legacy payloads. They are never coerced into the nearest known type.

---

## 12. Ontology-derived Workpoint candidates

### 12.1 Derivation

```text
canonical mission/goal/task
+ active semantic objects
+ dependencies
+ constraints
+ blockers
+ verification evidence
+ applicable action definition
+ slice policy
→ Workpoint candidate
```

### 12.2 Candidate fields

Additive fields include:

```yaml
source_ontology_revision:
source_registry_version:
source_domain_pack_versions: []
projection_policy_id:
resolved_object_refs: []
unresolved_object_refs: []
projection_evidence_refs: []
projection_hash:
```

### 12.3 Rollout modes

```text
legacy
hybrid_validation
shadow_derived
derived_candidate
```

- `legacy`: current caller-authored Workpoint behavior.
- `hybrid_validation`: caller fields are resolved and validated against active packs.
- `shadow_derived`: Focusa independently derives and compares a candidate without changing authority.
- `derived_candidate`: ontology produces the candidate, then the existing Workpoint reducer preview/promotion path decides authority.

### 12.4 Drift and freshness

A Workpoint records the semantic revision used for derivation. Relevant canonical graph changes invalidate or mark the Workpoint candidate stale; they do not silently replace an active Workpoint.

---

## 13. Generalized slice policies

### 13.1 Policy contract

```yaml
schema: focusa.slice_policy.v2

slice_policy_id:
domain_pack_id:
purpose:
allowed_object_type_ids: []
allowed_link_type_ids: []
required_anchor_kinds: []
membership_rules: []
exclusion_rules: []
verification_preference:
uncertainty_rules: []
max_objects:
max_links:
max_evidence_refs:
max_historical_deltas:
max_tokens:
fallback_policy_id:
```

### 13.2 Required policy families

```text
focusa.core/active_mission
focusa.software/debugging
focusa.software/refactor
focusa.software/regression
focusa.software/architecture
focusa.legal/matter_strategy
focusa.legal/citation_review
focusa.markets/thesis_review
focusa.markets/catalyst_monitoring
focusa.research/claim_validation
focusa.research/contradiction_resolution
focusa.general/planning
focusa.general/recovery
```

Current short slice names remain aliases.

### 13.3 Selection order

```text
operator current ask
→ project/workstream scope
→ authority and permission filter
→ active domain packs
→ freshness and verification filter
→ anchor traversal
→ lexical/semantic relevance
→ policy bounds
→ uncertainty and omitted-detail metadata
```

### 13.4 Client behavior

Clients render slices. They do not recompute or override canonical membership rules.

---

## 14. Semantic subscriptions and reactions

### 14.1 Separate planes

```text
Workspace invalidation stream
  purpose: refetch and rerender bounded read models

Semantic delta stream
  purpose: inform governed cognition, retrieval, proposals, checkpoints, or escalation
```

The streams may share transport infrastructure but remain separately typed and authorized.

### 14.2 Subscription contract

```yaml
schema: focusa.semantic_subscription.v1

subscription_id:
subscriber_ref:
scope:
  project_root:
  continuity_id:
domain_pack_ids: []
semantic_kind_filters: []
definition_id_filters: []
status_filters: []
canonicality: candidate | canonical | both
cursor:
delivery: pull | sse
reaction_policy_ref:
max_batch:
created_at:
```

### 14.3 Delta envelope

```yaml
schema: focusa.semantic_delta_event.v1

event_id:
event_type:
event_version:
registry_version:
domain_pack_id:
project_root:
continuity_id:
semantic_refs: []
evidence_refs: []
canonicality:
revision:
invalidate: []
created_at:
```

### 14.4 Delivery guarantees

- at-least-once delivery;
- durable cursor;
- idempotent consumption;
- bounded batches;
- ordering metadata;
- reconnect and missed-event recovery;
- backpressure;
- scope isolation;
- unknown-event preservation;
- explicit degraded state.

### 14.5 Reaction authority

A subscription event may trigger:

- bounded refetch;
- context invalidation;
- Workpoint stale warning;
- retrieval request;
- checkpoint proposal;
- semantic proposal;
- operator alert;
- policy-approved durable-session work.

It does not itself authorize destructive execution or canonical promotion.

---

## 15. Persistence, snapshots, events, and replay

### 15.1 Separate version axes

```yaml
snapshot_schema_version:
stored_event_envelope_version:
runtime_writer_version:
minimum_reader_version:
minimum_writer_version:
ontology_registry_version:
shared_cognition_pack_version:
domain_pack_versions: []
state_reducer_revision:
```

`state_reducer_revision` must not substitute for persisted schema version.

### 15.2 Snapshot migration

```text
read envelope
→ verify supported reader/writer range
→ preserve original backup
→ execute ordered idempotent migration
→ validate registry and state conformance
→ compare V1 compatibility projection
→ atomically publish migrated snapshot
```

Migration failure preserves the original snapshot and fails closed or starts an explicit read-only recovery mode. It does not return an empty fresh state as though no data existed.

### 15.3 Stored event envelope

Events are read in two stages:

```text
raw versioned envelope
→ known typed event: reducer apply
→ unknown non-authoritative event: preserve and skip with diagnostics
→ unknown authoritative event: stop canonical replay/write and require newer reader
```

### 15.4 Replay

Required replay proofs:

- archived V1 event log reconstructs equivalent V1 projection;
- mixed V1/V2 events remain ordered and scoped;
- duplicate delivery is idempotent;
- unknown events survive export/import;
- failed events do not corrupt following state;
- domain-pack migrations preserve history and evidence references.

### 15.5 Downgrade protection

After V2 canonical state exists, a runtime below `minimum_writer_version` may render a bounded compatible view when safe but cannot mutate the canonical store.

---

## 16. API, CLI, Pi, and client contracts

### 16.1 Required API families

```text
GET  /v2/ontology/registry
GET  /v2/ontology/domain-packs
POST /v2/ontology/domain-packs/resolve
GET  /v2/ontology/candidates
GET  /v2/ontology/canonical
POST /v2/ontology/proposals/preview
POST /v2/ontology/proposals/commit
POST /v2/ontology/verifications
POST /v2/ontology/promotions/preview
POST /v2/ontology/promotions/commit
POST /v2/ontology/slices
GET  /v2/ontology/deltas
POST /v2/ontology/subscriptions
GET  /v2/ontology/subscriptions/{id}
```

Exact routes may be reconciled through Spec 109, but the capability set is required.

### 16.2 CLI families

```text
focusa ontology registry
focusa ontology packs
focusa ontology candidates
focusa ontology canonical
focusa ontology propose
focusa ontology verify
focusa ontology promote
focusa ontology slice
focusa ontology subscribe
focusa ontology migrate
focusa ontology conformance
```

### 16.3 Generated contracts

```text
Rust authoritative definitions
→ JSON Schema/OpenAPI
→ TypeScript/client generation
→ Pi/PWA/Tauri/menubar/TUI/connector/test consumers
```

### 16.4 Pi

Pi continues to consume bounded ontology context. V2 may add domain-pack, policy, canonicality, revision, and uncertainty metadata without removing current fields. Semantic subscriptions are an optimization and governance surface, not a reason to remove the pull-based safe fallback.

### 16.5 Capability truth

Every surface distinguishes:

```text
operational
read-only
legacy compatibility
schema-only
pack missing
migration required
unsupported future definition
writer blocked
verification required
operator approval required
degraded
```

---

## 17. Migration plan

### 17.1 Registry extraction

Move current catalogs into core with exact name and output parity before changing behavior.

### 17.2 Shared/software packs

Represent existing domain-neutral and software definitions as built-in packs while retaining all legacy aliases.

### 17.3 Candidate/canonical dual state

Add defaulted V2 state beside V1 fields. Do not repurpose V1 collections in place.

### 17.4 Shadow projection

Generate V1 projections from V2 in tests and shadow runtime, compare them with current output, and block cutover on unexplained differences.

### 17.5 Verification profiles

Use explicit compatibility profiles:

```text
legacy_v1
strict_v2
regulated_v1
```

Existing historical records remain readable and retain original trust provenance.

### 17.6 Workpoint rollout

Run hybrid and shadow-derived Workpoint modes before derived candidates become the preferred path.

### 17.7 Domain rollout

Introduce additional packs only after shared and software pack conformance passes. Research or General may be the first non-software proof domain because they exercise semantic breadth without requiring destructive action authority.

### 17.8 Subscription rollout

Retain pull-based Pi context and existing SSE invalidation until semantic subscriptions prove replay, idempotency, scoping, and recovery.

---

## 18. Cross-spec ownership

| Concern | Primitive owner | Spec 135F integration responsibility |
|---|---|---|
| Ontology primitives | 45–50 | core registry, candidate/canonical graph, policy enforcement |
| Domain-general cognition | 61 | shared cognition pack |
| Status/lifecycle | 70 | generated versioned vocabularies |
| Identity/aliases | 74 | semantic and domain-pack identity resolution |
| Projection/views | 75 | preserve canonical/projection distinction |
| Governance/migration | 77 | versions, compatibility, migration, deprecation, conformance |
| Workpoint | 88 | derive candidates; never replace Workpoint authority |
| Context Cognition | 100 | consume generalized slice policies |
| API contracts | 109 | generated schemas, envelopes, preview/commit |
| Bootstrap | 111 | bounded semantic preload |
| Trajectory | 125 | mission/goal semantic interlock |
| Compaction | 130 | bounded refs, no corpus/graph dumps |
| Durable execution | 133 | governed semantic reactions |
| Workspace UX | 135A | render semantic projections and degraded states |
| C.R.I.S.T. | 135B | create reviewed semantic candidates and first Workpoint candidate |
| UIAI bridge | 135C | artifact/evidence candidates and separate invalidations |
| Implementation order | 135D | place this substrate in Orders 0–2 |
| Compatibility | 135E | prove migration and cross-spec closure |

---

## 19. Security and privacy

Required:

- project/workstream isolation;
- domain-pack namespace validation;
- permission-aware slices;
- sensitivity and retention classification;
- no raw credentials in semantic state;
- no private corpus blobs in event payloads;
- evidence access checks;
- cross-domain leakage tests;
- malicious manifest/schema rejection;
- bounded custom-pack resources;
- signature or trusted-origin policy for distributed packs;
- export classification and redaction preview;
- audit trail for pack activation, migration, promotion, and semantic reactions;
- explicit private/public pack and artifact boundaries.

---

## 20. Performance and resource laws

1. Registry definitions are cached by content hash and version.
2. Candidate/canonical graphs use indexed scoped storage.
3. Hot reads use bounded purpose-specific projections.
4. Large payloads remain behind handles.
5. Graph traversal has depth, node, edge, time, and token bounds.
6. Domain-pack composition is deterministic and cacheable.
7. Semantic subscriptions use bounded batches and backpressure.
8. Snapshot migrations are streaming or bounded where possible.
9. V1 compatibility projection is incremental or cached.
10. LowMem modes preserve authority metadata while reducing detail.
11. Background classification, indexing, and migration do not hold canonical locks for long-running work.
12. Performance degradation must be visible and recoverable.

---

## 21. Required implementation order

Spec 135D controls the full order. Within its Orders 0–2, this spec requires:

### Order 0

- compatibility constitution;
- current-code ontology Reality Pack;
- V1 snapshot/event/output fixtures;
- ownership map;
- domain-pack matrix;
- security model;
- migration and downgrade matrix;
- proof plan.

### Order 1

- core registry schemas;
- namespaced IDs and aliases;
- pack manifests;
- verification/promotion policies;
- slice policies;
- event/snapshot envelopes;
- semantic subscription contracts;
- generated clients.

### Order 2

- core registry runtime;
- candidate graph;
- canonical graph;
- verification ledger;
- V1 compatibility projection;
- persistence and migrations;
- Workpoint candidate projection;
- semantic delta stream;
- conformance and replay tests.

No professional vertical may claim semantic completion before its domain pack and verification/slice policies are operational.

---

## 22. Acceptance criteria

Spec 135F is accepted when:

1. Current ontology catalogs are core-owned and generated without breaking V1 names, routes, ordering-sensitive fixtures, or Pi consumption.
2. Shared cognition and software definitions operate as versioned built-in packs.
3. General, Legal, Markets, Research, Custom, and composite pack contracts are implemented and conformance-tested.
4. Candidate and canonical semantic state are structurally separate.
5. Proposals require typed definition IDs, scope, provenance, confidence/freshness where applicable, and policy references.
6. Promotion cannot occur without the registered verification/promotion decision.
7. Legacy verification is classified honestly rather than silently treated as strict V2 proof.
8. Workpoint candidates can be derived and compared in shadow mode before governed promotion.
9. Existing Workpoint authority and routes remain intact.
10. Generalized slice policies operate for software and non-software domains with bounded output and explicit uncertainty.
11. Pi and other clients consume generated semantic metadata without local policy duplication.
12. Semantic subscriptions provide cursoring, replay, idempotency, backpressure, scoping, and recovery.
13. Workspace invalidation and semantic delta authority remain distinct.
14. Archived V1 snapshots and events migrate or replay to equivalent V1 projections.
15. Unknown future definitions/events are preserved; unknown authoritative events fail closed.
16. Incompatible older writers are blocked after V2 canonical state exists.
17. Failed migration preserves the original state and never silently initializes empty state.
18. Domain-pack activation cannot change permission, workspace, role, evidence profile, or Workpoint authority implicitly.
19. Cross-domain composition and isolation tests pass.
20. API, CLI, Pi, PWA, Tauri, menubar, native TUI, headless/RPC, and generated-contract parity are proven where the capability is exposed.
21. Performance, security, privacy, migration, and release-proof requirements pass.
22. Actual end-to-end evidence exists for candidate creation, verification, promotion, slice retrieval, Workpoint candidate derivation, V1 projection, replay, migration, and semantic subscription recovery.

---

## 23. Closure blockers

This spec cannot close while:

- the registry remains duplicated across API routes, clients, or domain modules;
- objects and links remain only unvalidated generic JSON with no registry-backed V2 path;
- candidate and canonical state share one indistinguishable store;
- promotion can bypass verification policy;
- domain packs exist only as enums, docs, or visual terminology;
- a workspace profile substitutes for missing semantic capability;
- Workpoint derivation silently replaces existing Workpoint authority;
- non-software slices are unbounded or client-computed;
- semantic subscriptions are non-replayable or can mint authority;
- UI invalidation is conflated with semantic action authority;
- V1 snapshots/events/routes/Pi behavior cannot be preserved;
- migration can lose unknown data or silently start fresh;
- old writers can corrupt newer state;
- cross-project or cross-domain isolation is unproven;
- a required client exposes semantically divergent contracts;
- implementation or proof is deferred outside the Spec 135 closure graph.
