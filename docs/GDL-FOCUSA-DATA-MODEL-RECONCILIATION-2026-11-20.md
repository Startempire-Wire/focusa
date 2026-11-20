# GDL ↔ Focusa — Two-Pass Data-Model Reconciliation (2026-11-20)

**Status:** SOURCE/DOCUMENT AUDIT + INTEGRATION CORRECTION PROPOSAL. This document does not promote architectural authority, merge, activate a source grant, deploy code or authorize a new scope.  
**Companions:** [GDL-001](GDL-001-global-decision-ledger-contract.md), [GDL-002](GDL-002-continuous-decision-mining-reconciliation-daemon-spec.md), [ADR research](GDL-ADR-2026-11-20-research-and-proposed-adoption.md)  
**Audited:** Focusa **main** documents and relevant Rust/TypeScript source plus the draft GDL branch. No installed runtime, live daemon, schema migration, performance or E2E behavior was independently exercised.  
**Goal:** fit GDL **into the existing Focusa cognition/ontology/authority data model**, not bolt another decision store onto the daemon.

## Executive disposition

**Recommended architecture:** GDL is the **durable governed Decision specialization** of Focusa's domain-general cognition and typed ontology, using the *existing* reducer/evidence/provenance/identity/workstream path. The Decision Miner is Focusa-supervised, resource-isolated background **candidate producer**. Agents query the Focusa daemon's own authenticated, revision-aware **derived decision search/resolution projection**. ADR/MADR/GADR are optional projections and advisory reference models, never independent canonical stores.

**Critical correction to original GDL draft:** “global” is a **governed, permission-filtered cross-scope query/visibility posture**, **NOT** one global Focusa cognitive bag or generic global status array. Spec 158 forbids daemon-global project cognition and its ScopeRef kinds are currently project and host; host is **not** an organization/global shortcut. An ecosystem/tenant-wide decision must have a verified, explicit original authority owner and a policy-approved applicability edge to every consuming scope; do not invent a new ScopeRef kind or copy decisions into foreign Workstreams.

## PASS 1 — Existing model and source reality

| Existing source / type | Code and/or specification reality | GDL treatment | Status |
|---|---|---|---|
| [Doc 61 Decision](61-domain-general-cognition-core.md) | Decision already defined as id, statement, decision_kind, status; durable conclusion guiding future action | **Reuse semantic primitive.** GDL adds typed identity, provenance, lifecycle/authority payload, NOT a second generic Decision kind | Normative concept |
| [FocusState.decisions](../crates/focusa-core/src/types.rs) | Hot Focus State contains **bounded 30 strings** for decisions; Pi validation checks one sentence and <=160 chars in [decision-validation.ts](../apps/pi-extension/src/decision-validation.ts) | **Projection ONLY** of a small number of active relevant decisions; never ledger storage or approval source; no automatic bulk stuffing | Actual source |
| [ContextClaimRecord](../crates/focusa-core/src/types.rs) | Source-linked reviewed/unreviewed claims with revision, confidence and contradiction refs | Evidence-backed claim candidate / context input; can support a GDL proposal but must not automatically be an approved Decision | Actual source |
| [ContextDecisionRecord](../crates/focusa-core/src/types.rs) + [context_claims.rs](../crates/focusa-api/src/routes/context_claims.rs) | Specialized append-only **review decision** with decision_kind \`claim_review\` / \`contradiction_resolution\`, target_ref, outcome, rationale, actor, evidence, receipt | **Link as review disposition/evidence**, not same as an approved durable product/architecture GDL decision; no rename/reuse of this struct as the general DecisionRecord | Actual source |
| [ContextSourceRecord](../crates/focusa-core/src/types.rs) + [context_sources.rs](../crates/focusa-api/src/routes/context_sources.rs) | Scoped source identity, content, hash, source locator/revision, adapter, ingestion health, evidence and receipt; current path uses project_root/continuity/attachment fields | Link exact source revisions + spans; **do not copy entire source corpus into GDL**. Before reuse, verify scope bindings and whether source material can be retained under existing policies | Actual source |
| [OntologyState](../crates/focusa-core/src/types.rs) and [reducer](../crates/focusa-core/src/reducer.rs) | V1 objects/links as generic JSON; proposals/verifications/deltas with WorkstreamKey attributes; proposal promote code exists | Use common reducer/event substrate and typed registered graph shape; **do not create an independent Decision table plus an unconnected ontology object**. Avoid treating existing generic proposal status as sufficient authorization | Actual source, partial target |
| [Spec 135F candidate/canonical graphs](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) | Calls for **structurally separate** candidate and canonical semantic graph, registered type IDs, verification/promotion policies, typed slices/subscriptions and V1 compatibility | GDL candidate/canonical must be **typed payload profiles** of those shared contracts, with subtype registry mappings and adoption phase/compatibility, not a duplicate parallel generic graph | Normative draft; actual V2 implementation incomplete according to §3 |
| [semantic_registry.rs](../crates/focusa-core/src/semantic_registry.rs) | Current Rust SemanticRegistry tracks **versioned semantic artifacts, namespaces and named graphs**; not proof that every Spec135F type/action/status definition is fully registered | Inspect for reuse of artifact registration, but **do not assume** it is already a complete general GDL type registry; follow 135F type/action/status/promotion registration when available | Actual source / compatibility review needed |
| [Spec 158 scope/workstream](158-workstream-rooted-cognitive-runtime-canonical-state-partitioning-and-foundation-migration-spec.md), [Spec 164](164-workstream-rooted-canonical-runtime-design.md) | Normative \`ScopeRef + stable WorkstreamId\`; ContinuityId is *lineage*, Attachment is runtime binding; project-shared vs Workstream-owned cognition are different. Source [WorkstreamStateStore](../crates/focusa-api/src/workstream_store.rs) is an **ongoing migration** with scoped persistence and global compatibility projection | Every governing record must have declared **owner plane and typed scope**, never daemon-global project cognition. Cross-Workstream and cross-project reuse is a typed, authorized reference, never an implicit copy | Normative plus partial actual migration |
| [scoped_state.rs](../crates/focusa-core/src/scoped_state.rs), [WorkstreamRoot](../crates/focusa-core/src/workstream_root.rs) | Current \`scoped_state::WorkstreamKey\` represents root scope + continuity_id, while Spec158's target distinguishes stable WorkstreamId from continuation lineage; WorkstreamRoot also exists | **Do not encode continuity_id as the permanent decision owner key.** Resolve actual canonical WorkstreamRoot ID via installed binding, compatibility mapping and migration receipts | Actual code / model disparity |
| [Spec 74 identity](74-identity-and-reference-resolution.md) | CanonicalEntity, ReferenceAlias, ResolutionCandidate, ResolutionDecision, SupersessionRecord distinguish alias/equivalence/supersession | Use canonical identity and provenance for renamed/merged projects, duplicate decisions and supersession; an alias is not an authority or automatic cross-scope transfer | Normative |
| [Spec 70 shared interfaces](70-shared-interfaces-statuses-and-lifecycle.md) | Scoped, Verifiable, ArtifactBacked, Ownable, RiskBearing etc and shared statuses | Type GDL decision, candidate, relation, evidence, projection and review with these shared interfaces rather than route-local vocabularies | Normative |
| [Spec 75 projection](75-projection-and-view-semantics.md), [Spec 100 Context Cognition](100-context-cognition-spec.md) | Purpose-specific views must not mutate truth; Context Cognition is bounded and advisory | Source-specific search results, compact prompt slice, human ADR, TOON/JSONL and dashboards are **projections**. Agent context injection is not action authorization | Normative |
| [PRE](41-proposal-resolution-engine.md) | Resolves concurrent operational focus/thesis/autonomy proposals in a scoped window | Can inform conflict-window semantics, but **must not select a winning architectural decision by score** or substitute for operator approval and constitutional promotion | Normative historic; Spec158 governs scope migration |
| [Spec 135F policies](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) §§10,13–15 | \`semantic_candidate.v2\`, \`canonical_semantic_record.v2\`, \`semantic_verification_record.v2\`, \`semantic_promotion_record.v2\`, slice policy, semantic delta/subscription and compatibility migration | GDL schema must declare exact **definition ID, verification policy, promotion policy, slice policy and relation types**; use semantically distinct candidate/active storage/indexes | Normative draft |
| [Focusa search](../crates/focusa-core/src/runtime/context_retrieval.rs) | SQLite FTS5 + optional sqlite-vec/fastembed derived source retrieval, cited chunks, exact-scope bounded reads | Reuse index technology and source citation conventions in a **separate typed Decision projection** with pre-filtered authorization; do not mix untrusted mined text with active governing decisions | Actual source |
| [SSE](../crates/focusa-api/src/routes/sse.rs) and [Spec 135F subscription](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) | Durable replay and live broadcast exist; 135F separates workspace invalidation from semantic delta | Publish bounded decision references + revisions through **existing event transport**, keeping delta semantics separate; no second broker, no raw private snippets in general events | Actual source + normative |
| [Spec 90 tool contracts](90-ontology-backed-tool-contracts-parity-spec.md) + [Agent capabilities](../crates/focusa-api/src/routes/agent_capabilities.rs) | Generated capability/tool schema, CLI/Pi/MCP/REST parity and error/verification rules | Register GDL as a **native Focusa operation family** in owning generated contract pipeline; no unregistered bespoke MCP or server | Actual and normative |
| [Spec 111 bootstrap](111-agent-context-bootstrap-and-delivery-spec.md), [agent journey](agent/02-focusa-cohesive-project-flow.md) | Preload bounded context with exact Workstream/attachment/Workpoint identity and validation; Workpoint/frontier separately admit action | Pull active applicable decisions in bounded context slice at bootstrap; re-resolve immediately before consequential actions; decision lookup cannot replace Workpoint or Context Authority | Normative and established workflow |
| [Retention](76-retention-forgetting-and-decay-policy.md), [evidence ledger](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md) | Retention/decay distinct from canonical truth; evidence receipts and event-chain structure already exist | Keep immutable decision history under policy and bounded/externalized original source refs; privacy revocation affects visibility and derived copies. Do not create GDL-only evidence or backup authority | Normative + existing infrastructure |

## PASS 2 — Corrected data model and activation sequence

### 2.1 Registered semantic type and storage

**GDL is not a second root-level store with autonomous truth.** The canonical decision's semantic meaning is the existing Focusa \`Decision\` primitive in \`focusa.core.cognition\` as formalized through 135F's versioned object definition. Proposed canonical name \`focusa.core/decision@1\` is **a proposed registry ID** and MUST be checked against the actual registered ID/version; it is not evidence of an existing installed type.

Map the prior GDL-specific payload to the existing 135F envelopes:

\`\`\`text
Source evidence/ContextClaim (advisory, per originating source ACL)
  -> focusa.semantic_candidate.v2
       semantic_kind=object; definition_id=<registered Decision type>
       payload=<GDL-002 DecisionCandidate schema-compatible details>
       status=proposed/verified_candidate; owner ScopeRef/Workstream binding
  -> focusa.semantic_verification_record.v2
       verification policy for source/speaker/scope/identity/contradiction
  -> operator approval (separate authenticated signed/grant-rooted receipt)
  -> focusa.semantic_promotion_record.v2
       promotion policy verifies proof and grants; not just status='accepted'
  -> focusa.canonical_semantic_record.v2
       definition_id=<registered Decision type>; payload=<GDL decision>
       owner, revision, provenance, verification, effective status
  -> Focusa reducer/event-chain, index and compact derived projections
\`\`\`

**Implementation compatibility:** If the installed 135F V2 candidate/canonical substrate remains incomplete, implementation work must first evaluate and extend that owning substrate. A temporary compatible adapter MAY map legacy V1 records into **read-only advisory projections**, but MUST NOT create a second independent canonical write store, imply a \`ContextClaimReviewed\` is operator approval, or silently add GDL status strings to generic ontology values without registration/migration policy.

**GDL candidate JSON Schema** is a *decision-specific payload contract*, not a claim of an independent candidate database. Add a versioned mapping/adapter to \`semantic_candidate.v2\` when that root schema is implemented, with testable one-to-one identity and permission invariants.

### 2.2 Owner plane and inheritance

Define an **owner plane** and **applicability** separately on every candidate and active decision:

| Owner plane | How it is owned | How an agent consumes it |
| --- | --- | --- |
| Operator-rooted architecture constitution / approved delegated policy | Existing external approved constitution/authority owner; ledger stores **references/attestations**, not a replacement constitutional root | The resolver validates the canonical policy provenance before including an applicable governing constraint |
| Project-shared decision | Project Scope owns verified shared truth as permitted by Spec158; not one active project Focus State | Explicit authorized bounded reference into each eligible Workstream in that project |
| Workstream decision | Exact verified ScopeRef + stable WorkstreamId owns durable cognition; Continuity and Attachment are checked at the action/read boundary as applicable | That Workstream's scope-checked read; foreign Workstreams cannot mutate or implicitly inherit it |
| Historical/retired project decision | Original owning project/Workstream retained with historical validity and lineage | Advisory historical query or an explicit authorized migration/supersession/acceptance; **never** automatically active in new project |
| Tenant/product-wide "global" view | Derived query across **explicit approved policy applicability edges**, ACLs and actual owner scopes. \`tenant_ref\` may be an external security/visibility boundary but is **not a new Focusa ScopeRef kind by itself** | A virtual **resolved view** of applicable records, not shared physical canonical cognition |

**Important:** current ScopeRef supports project and host. Do not use Host Scope or an invented \`ecosystem\` / \`tenant\` ScopeRef as a universal cognition owner. If native organizational policies need a new owner kind, that is a **separately proposed Spec158 amendment** requiring operator approval; do not silently add it under GDL.

**Identity compatibility:** Spec158 envisions stable WorkstreamId separate from ContinuityId, while an existing \`scoped_state::WorkstreamKey\` still stores \`root_scope + continuity_id\`. Require a versioned resolution edge to \`WorkstreamRoot\`, and reject any ambiguous/missing mapping before canonical read/write. Never make the GDL decision's stable ID depend on changing continuity, cwd, session or worktree path.

### 2.3 Identity and relations

Register decision and relation definitions through the shared semantic registry contracts (Spec135F) and Spec74:

- Decision identity: stable opaque ID; \`semantic_id\`/revision; exact typed owner scope; original meaning immutable; optional human alias.
- Alias and duplicate source mention are **not** equivalence or supersession; use Spec74 \`ResolutionCandidate\` / \`ResolutionDecision\`.
- \`supersedes\`, \`clarifies\`, \`supports\`, \`conflicts_with\`, \`derived_from\`, \`governs\` are **proposed typed relation names** to reconcile to existing 48/74 link types; do not ship unregistered route-local strings.
- ADR/GADR \`DecisionQuestion\` and \`DecisionPattern\` stay *advisory proposed* additions (research addendum), not new canonical primitive types until separately approved.
- Source attribution and effective time are distinct; do not trust agent sentiment, Git merge, later timestamp, filename \`Accepted\` or model similarity to mint approval.

### 2.4 Typed lifecycle vs unrelated review decisions

**Two independent axes** (do not conflate):

1. **Decision authority:** mined candidate → verified candidate → approval pending → operator approved and promoted → active/superseded/withdrawn/expired. Only verified + policy-promoted active governs.
2. **Implementation truth:** unimplemented / planned / implemented-claim / observed / independently verified / failed. A decision can be approved but not implemented, or implemented in code without proof of approval.

\`ContextDecisionRecord\` with \`decision_kind=claim_review\` or \`contradiction_resolution\` is a **review/settlement event about claims**. It may be associated as \`source_review_ref\` and support evidence quality but must not serve as \`operator_approval_ref\` for GDL policy promotion. PRE winner is likewise **not** product/architecture approval.

### 2.5 Search, context, subscriptions and events

- The **canonical write** is the existing Focusa single-writer reducer/event chain routed by declared owner plane.
- The **read model** is derived/epoch-indexed typed Decision FTS5 (optional vector), with **separate** candidate corpus, original source corpus, and active-governing projection. Pre-filter by permissions/tenant, policy scope, verified Workstream/attachment, and effective times, *before* ranking/snippet exposure.
- The **agent resolver** is deterministic over registered Decision status, approved scope/applicability/policy version; semantic search locates candidate IDs but does not decide governance.
- The **hot Focus State** receives a compact relevant subset through Spec135F slice and Spec111 preload policy, with limits, ref + revision, omission count and provenance. It is *not* the ledger.
- Use existing durable SSE infrastructure, but keep **semantic delta** (\`canonical_decision_activated/superseded\`) conceptually distinct from **workspace invalidation** (refetch UI). A general cross-product event is a derived adapter envelope, not another canonical event bus.
- On index lag, return bounded canonical exact lookup or truthful \`read_model_lag\`; do not present stale index as current. Worker failure affects candidate discovery, not current approved-decision queries.

### 2.6 Dependency-aligned implementation sequence (replaces no existing obligations)

0. **Source reality gate:** detect the *installed* Focusa API, exact Workstream binding/partition, schema revisions and current 135F/158/164 migration status. This documentation audit alone is not runtime proof.
1. **Owner registry mapping:** approve the specific Decision type ID, field schema, lifecycle, relation IDs, verification and promotion policy within existing shared cognition framework. Reference existing architecture constitution, Spec70/74/77.
2. **Candidate ingestion:** adapt GDL-002 DecisionCandidate as typed \`semantic_candidate.v2\` payload. Reuse Context Sources/Evidence/claims provenance and existing bounded execution. No model writes to canonical state.
3. **Promotion:** attach source verification, adversarial review and authenticated operator approval to existing semantic promotion/reducer transaction and receipt. Dual-state compatibility follows 135F and Spec158 migration; no silently competing canonical store.
4. **Derived real-time agent read:** typed authorized index / deterministic resolve / compact slice, parity through existing generated API/CLI/Pi/MCP contracts; projection freshness and source-miner freshness separated.
5. **Rollout:** one Workstream first, then a second concurrent Workstream, one project-shared decision, and an authorized operator-rooted policy overlay. Verify no foreign leakage and rollback; historical aliases and ADR profiles remain advisory.
6. **Continuous source expansion:** only after proven path, add connector adapters and backfill under explicit grants. No source-opening or third-party write permissions implied.
7. **Do not silently piggyback foundation repair.** If the relevant 135F/158 feature is absent or incomplete, report a concrete dependency and request explicit program admission rather than manufacturing an alternative canonical GDL store.

### 2.7 Core acceptance tests

1. An agent's \`ContextClaimReviewed\` **cannot** activate a GDL Decision, even when its \`decided_by\` string names the operator.
2. A high-similarity GDL candidate is retrievable only as advisory, **never** as governing.
3. No Workstream data leaks into another's search, counts, facets, context injection, event payload or cross-project reconciliation.
4. Continuity rollover and worktree switching preserve stable Workstream-rooted decision identity; ambiguous workstream mapping blocks without leakage.
5. Project-shared/constitutional policy appears in exactly the authorized Workstream decision view **without duplicating canonically owned decision objects**.
6. \`FocusState.decisions\` overflows only by bounded projection logic, never triggers loss of ledger history or auto-approval.
7. Historical ADR marked \`Accepted\` remains an imported source claim unless verified approval/policy promotion says otherwise.
8. Before/after each accepted supersession: canonical revision, derived search revision, human ADR view and Pi/MCP consumers agree.
9. Replaying snapshots and events across schema versions preserves unknown fields/deltas and refuses incompatible writer downgrade per Specs135F/158.
10. An outage of the mining worker leaves approved decision query/resolve available, source coverage honestly stale.
11. A rejected/corrected candidate remains auditable and does not appear as active by status-string match.
12. Producer, consumer, cross-version, live E2E, permissions and performance proof meet [production consistency](current/PRODUCTION_CONSISTENCY_POLICY.md) before release claims.

## Scope/provenance disposition

This audit is **integration corrective guidance**. It does not silently approve optional ADR/GADR additions, increase GDL source permissions, replace Focusa's current runtime state or create new organization scope semantics. Proposed additions to the shared semantic registry/type definitions, explicit project-shared decision ownership and any new ScopeRef kind remain separately authorizable decisions.

**Recommendation:** Update GDL-001 and GDL-002 to explicitly cite this mapping as their implementation-preflight reference. **No GDL implementation should proceed without resolving the data-model owner and migration compatibility questions above.**
