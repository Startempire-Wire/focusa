# GDL-001 — Global Decision Ledger (Focusa canonical contract)

**Status:** PROPOSED / NOT APPROVED / NOT IMPLEMENTED  
**Date:** 2026-11-17  
**Owner:** Focusa cognition/authority substrate  
**Companion:** [GDL-002 continuous decision mining](GDL-002-continuous-decision-mining-reconciliation-daemon-spec.md)  
**Purpose:** One short, searchable, source-backed decision record can govern all *authorized* consumers without becoming a competing memory, task, approval, or execution authority.

## 0. Constitutional and runtime boundaries

This is a **proposal** only. Creating a document or merging a PR does not constitute operator approval, activation, or a canonical architecture promotion. Preserve:

- [Current architecture authority policy](current/ARCHITECTURE_AUTHORITY_POLICY.md): external/agent/source assertions are advisory by default; valid operator-rooted authority is required to promote architecture decisions.
- [Current authority model](current/AUTHORITY_MODEL.md): exact ScopeRef/ProjectRootKey, WorkstreamId, ContinuityId, and applicable AttachmentKey; current operator steering wins; transcript tails, cached summaries, and imported material never grant authority.
- [Domain-general Decision primitive](61-domain-general-cognition-core.md) and [Pi decision validator](../apps/pi-extension/src/decision-validation.ts): concise, single crystallized sentence, at most **160 characters**. Extended rationale is metadata.
- [Existing tamper-evident event chain](current/TAMPER_EVIDENT_EVENT_CHAIN.md), [Context Sources](../crates/focusa-api/src/routes/context_sources.rs), [Spec 135F domain ontology](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md), [Spec 119 receipts](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md), [Spec 120 operator approval gates](120-adversarial-spec-workbench-and-operator-approval-gates.md), and [Spec 78 bounded cognition](78-bounded-secondary-cognition-and-persistent-autonomy.md).
- Wirebot/SOVOS/UIAI are consumers of exact permitted projections. They do not gain authority from this ledger. UIAI owns browser execution evidence, Focusa owns its own cognitive/operational state, existing tasks remain with their owning systems; approval state has **one** canonical owner.

**Global** means authorized cross-product reach *inside an explicit tenant and authority scope*, not visibility across tenants or arbitrary project roots.

## 1. Objects and identity

The ledger MUST distinguish:

1. `DecisionCandidate`: a model/human/source-originated assertion pending reconciliation, **not effective**.
2. `DecisionRecord`: an operator-approved, authorized, immutable substantive decision revision, with original text and exact scope.
3. `DecisionRelation`: typed `duplicates`, `clarifies`, `amends`, `supersedes`, `depends_on`, `conflicts_with`, `derives_from`; causal and temporal provenance are not interchangeable.
4. `DecisionResolution`: a deterministic, policy-checked applicable decision set or `ambiguous|blocked|no_applicable_decisions|stale`, including excluded conflicting candidates.
5. `DecisionApprovalReceipt`: authenticated, explicit operator disposition over a pinned candidate revision, scope, source evidence digests, and policy version.
6. `DecisionExposureReceipt`: proof that a specific authorized consumer received the governing decision revision; read/exposure does not prove execution or compliance.

Stable ID: opaque UUID/ULID, with friendly `GDL-` aliases optional. A semantic `decision_key` groups revisions but MUST NOT itself authorize latest-wins. Decisions are *append-only in substance*; corrections, scope changes, withdrawals and supersession append new events/revisions. No update-in-place of accepted historical assertions. UI projections are freely editable as long as the audited canonical assertions do not mutate silently.

## 2. Required decision shape

Canonical storage and API MUST use versioned JSON Schema and JCS / RFC 8785 for hashes/cryptographic attestations, in accordance with existing architecture policy. Text/Markdown/TOON are derived displays, not authoritative serialization. A decision MUST contain:

```json
{
  "schema": "focusa.gdl.decision.v1",
  "id": "gdl_01example",
  "decision_key": "ecosystem.decision.statement_length",
  "revision": 1,
  "statement": "Keep decisions to one sentence; place rationale in attached metadata.",
  "kind": "architecture",
  "status": "active",
  "tenant_ref": "tenant-ref",
  "scope": {
    "level": "ecosystem",
    "project_refs": [],
    "workstream_refs": [],
    "applicability": ["focusa", "wirebot", "uiai-engine"]
  },
  "authority": {
    "root_policy_ref": "docs/current/ARCHITECTURE_AUTHORITY_POLICY.md",
    "actor_principal_ref": "verified-operator-ref",
    "approval_receipt_ref": "receipt-id"
  },
  "provenance_refs": ["source-span-ref"],
  "rationale_ref": "evidence-ref",
  "supersedes": [],
  "effective_at": "2026-11-17T00:00:00Z",
  "expires_at": null,
  "created_at": "2026-11-17T00:00:00Z"
}
```

Example values are non-live test data. Full schema should include privacy labels, subject/entity refs, role and kind, confidence *about extraction only*, content digests, version/reducer sequence, source lineage, policy version, lifecycle/effective intervals, attestation references, and evidence visibility. Cryptographic attestation is independently verified; a field claiming approval is never sufficient.

**Status:** `candidate`, `under_review`, `rejected`, `active`, `superseded`, `withdrawn`, `expired`, `disputed`. Only properly approved `active` versions with satisfied applicability and freshness may govern. Rejected historical candidates remain auditable under retention policy, not active.

**Canonical statement:** <=160 characters, exactly one crystallized decision; no tasks, speculative implementation claims, debug narration, or multi-sentence rationales. Human views show 1–2.5 lines and fold all metadata behind an expansion.

## 3. Deterministic resolution — not semantic search alone

`decision_resolve(context, query?, action_class?)` MUST:

1. Authenticate tenant/principal/capabilities, verify current scope, Workstream/continuity/attachment when applicable, and apply source and decision ACLs.
2. Select only authorized, active, effective revisions whose precise applicability includes the requested domain/action; do not flatten archived project decisions into the global namespace.
3. Apply precedence from existing constitutional/authority policy and recorded delegated scopes. Newer date, more repetitions, higher model confidence, Git merge, or product deployment are **never** automatic promotion/override.
4. Resolve explicit `supersedes/amends/withdrawn` relations under approved scope; keep exact source lineage and policy-version proof.
5. If multiple applicable records materially conflict or lineage is ambiguous, return **blocked / requires operator disposition**, with the conflicting IDs and evidence.
6. Return bounded verified metadata and compact statement to agent. Do not force full rationale into every prompt. Preserve no-data versus denied/no-coverage/stale distinctions.

Required result fields: `status`, `canonical`, `scope_ref`, `policy_version`, `decisions[]`, `exclusions[]`, `conflicts[]`, `freshness`, `receipt_ref`, `tool_result_v1`. Offline snapshots may support **bounded advisory context** but must not confer stale mutation authority.

## 4. Shared surfaces and operations

Proposed routes (not yet registered):

| REST | CLI | MCP |
| --- | --- | --- |
| `GET /v1/gdl/decisions/{id}` | `focusa gdl get` | `focusa_gdl_get` |
| `POST /v1/gdl/resolve` | `focusa gdl resolve` | `focusa_gdl_resolve` |
| `GET /v1/gdl/search` | `focusa gdl search` | `focusa_gdl_search` |
| `GET /v1/gdl/history/{id}` | `focusa gdl history` | `focusa_gdl_history` |
| `POST /v1/gdl/candidates` | `focusa gdl propose` | `focusa_gdl_propose` |
| `POST /v1/gdl/review/{id}` | `focusa gdl review` | `focusa_gdl_review` |

All writes use authenticated capability, exact tenant and verified scope, bound expected revision, idempotency key, checked evidence digests, Context Authority preflight as applicable, and verifiable receipts. MCP/CLI/UI are **adapters to the same reducer**, not parallel stores. Approval and supersession use an explicit trusted operator-auth path; agent credentials have propose/read but no self-approval. Provider grants cannot imply operator authority.

Event projections use CloudEvents-compatible envelopes (`id`, `source`, `type`, `specversion`) and durable delivery/outbox with per-consumer checkpoints. Consumer receipt ≠ consumer compliance; enforcement is proved by separate execution evidence.

## 5. Adoption

- **First:** implement a readable, queryable candidate-only ledger/resolver and source verification; no automatic downstream application.
- **Then:** operator-reviewed individual approvals and proper current-policy gating; preserve append-only events.
- **Then:** emit signed/authorized decision projections into OpenClaw, Pi, Wirebot, SOVOS, UIAI, Workstreams, GitHub checks, and human UI as applicable.
- **Finally:** reconcile drifts and contradictory consumer behavior using existing Evidence/Workpoint/receipts; never mark decisions "implemented" from ingestion alone.

Existing source implementations must be audited for reuse before writing new reducers, stores, approval queues or schedulers. Deployment and runtime-readiness claims require real producer, consumer, cross-version and live end-to-end proof under [production consistency](current/PRODUCTION_CONSISTENCY_POLICY.md).

## 6. Reference standards

- [RFC 8785 / JCS](https://www.rfc-editor.org/rfc/rfc8785.html) — invariant canonical JSON for hashing/signatures.
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — entities, activities, agents and derived-from/attributed-to provenance.
- [CloudEvents specification](https://github.com/cloudevents/spec/blob/main/cloudevents/spec.md) — interoperable event envelopes and producer-scoped event IDs.

**Decision awaiting operator approval:** accept GDL-001 as the canonical Focusa extension, not a new independent authority or a claim of current runtime functionality.


## 7. Runtime architecture clarification — Focusa daemon owns the hot path (2026-11-17)

**Normative target, not implemented or activated by this PR.** The decision ledger MUST be a **first-class Focusa daemon module**: the existing Focusa daemon (authoritative on KH under current deployment policy) owns canonical decision events, approval, immutable revisions, scope/authority resolution, the search read model, and agent-serving API. **Agents query Focusa directly.** A separate globally authoritative decision-miner daemon, external ledger DB, agent-memory authority, or per-agent decision store is prohibited.

The **Decision Miner** is a named Focusa-owned **background workload**, *not* an independently authoritative server. It MAY run in separate supervised worker processes/execution capacity (OVH under current KH/OVH policy), using existing Focusa durable-work scheduling, worker admission, receipts and bounded job facilities. Heavy historical crawls, parsing, embeddings, and LLM reconciliation must not run on the interactive daemon request path or create resource spikes on KH. Workers submit **untrusted candidate proposals** through authorized Focusa APIs; only the Focusa reducer can promote or supersede accepted records.

~~~text
Connected, explicitly granted sources
  -> bounded Focusa-scheduled Decision Miner jobs (heavy work isolated on OVH)
  -> source-evidenced CANDIDATE submissions (never privileged DB writes)
  -> Focusa daemon / existing reducer / explicit operator approval
      -> canonical append-only DecisionRecord and version
      -> daemon-owned rebuildable FTS5 + optional vector read projection
      -> existing durable SSE stream invalidation/replay
      -> REST / CLI / generated Pi/MCP/agent tool contracts
  -> Pi / OpenClaw / Wirebot / SOVOS / UIAI / human surfaces

No miner/model process is a second decision authority.
~~~

### 7.1 Real-time agent reads

**Read the already-ingested canonical ledger, never synchronously crawl external sources or call an LLM to answer a decision query.** Separate these operations:

| Purpose | Proposed query | Behavior |
| --- | --- | --- |
| Exact identity lookup | focusa_gdl_get(id, scope) | Primary key/revision; bounded metadata and proof |
| Discovery | focusa_gdl_search(text, filters, scope) | FTS5 exact/lexical + optional hybrid retrieval; results carry authority/status; candidate search separate |
| Governance | focusa_gdl_resolve(scope, action_class, context) | Deterministically select current applicable **approved** decisions and report conflicts |
| Work admission/context | focusa_gdl_context(scope, workpoint_ref, budget) | Bounded, scope-checked relevant decision slice with source revision/authority |
| Freshness | focusa_gdl_status(scope) | Last committed decision revision, index projection revision, worker watermark, source coverage and stale/partial flags |

Existing implementation evidence to reuse, not fork:
- [ContextRetrievalIndex](../crates/focusa-core/src/runtime/context_retrieval.rs) is already a rebuildable SQLite FTS5 + sqlite-vec context projection with bounded exact-scope retrieval and deterministic lexical fallback. GDL should **reuse the infrastructure/patterns** while maintaining its own typed decision search projection, **not** co-mingle unapproved source text with active governing decisions.
- [Focusa SSE](../crates/focusa-api/src/routes/sse.rs) already replays durable events and tails a broadcast stream; add GDL event projections/invalidation there, not another event broker. Stream is **notification/invalidation**, not proof of query completion or a replacement for the authorized read.
- [Agent Capabilities](../crates/focusa-api/src/routes/agent_capabilities.rs) and [tool discovery](focusa-tools/tools/focusa_tool_search.md) supply progressive machine-readable tool discovery; register Focusa GDL as an operation family, preserving generated Pi/MCP/OpenAI/CLI/REST parity and typed ToolResultV1 error envelopes.
- [Spec 135 performance budgets](contracts/spec135-q3-performance-budgets.v1.yaml) sets interactive read target 1,000 ms. GDL adds an **aspirational** 250 ms p95 local lexical query target under representative data and permissions, with <=1,000 ms p95 overall interactive read under normal measured workloads; benchmark before promising these numbers to users.

**Hard rule:** search finds possible matches; resolve determines actual governing decisions. A lexical or vector similarity score MUST NEVER make a candidate active, widen authority, resolve a constitutional conflict, or override explicit scope verification. Unknown, stale, denied, no applicable records, missing grant and read-model lag are distinct results.

### 7.2 Read-your-writes, revision parity and failure modes

1. Approved decision append uses the existing reducer/event-chain and atomic version-fenced commit. The daemon tracks canonical decision revision and the derived index projection revision.
2. Before returning a query as current, ensure the search read model covers the committed canonical revision; otherwise perform a **bounded canonical fallback** for exact queries or return an explicit projection-lag/retry result. Never return stale searchable facts labeled current.
3. Event stream publishes bounded decision-reference invalidation after commit. Consumers may refresh by decision/version, but **must** verify latest authorization on action admission.
4. Mining outage only degrades *new candidate discovery*: existing approved ledger reads remain available. Index rebuild is restart-safe and does not mutate canonical events. Daemon unavailable fails closed for governance/actions; never infer permission from cache.
5. Security filters apply **before** ranked snippets, semantic results, counts and facets leave the trusted daemon. Cross-tenant or unauthorized project candidates cannot leak through search results or timing-dependent answer metadata.
6. Indexing and source freshness are separate: a decision index can be current as of the last approved commit while the source miner's coverage is incomplete; expose both.
7. Pre-action gates resolve exact approved constraints (not solely injected prompt memory). Agent startup/Workpoint context may receive a minimal decision slice, but on-demand lookup remains callable across all authorized agent surfaces.

**Acceptance:** 1) activation then immediate read-your-writes search and resolve; 2) revoke/supersede invalidation visible without restart; 3) near-real-time queries during a large historical backfill; 4) stable results under vector outage; 5) cross-tenant and wrong-Workstream negatives; 6) candidate never appears as binding; 7) crash between commit/index update and deterministic recovery; 8) working REST/CLI/Pi/MCP consumers, measured latency and accurate degradation.

## 8. Architecture Decision Record (ADR) compatibility — derived view, not second authority

The GDL is a **generalized lightweight decision record system**. It should recover the value of Architecture Decision Records without requiring the operator to manually write or maintain Markdown ADRs.

A traditional ADR (Nygard-style) preserves **Status, Context, Decision and Consequences**. MADR adds detailed drivers, considered options, rationale and trade-offs. Make these fields **typed, source-backed GDL metadata**; do not expand the short canonical decision statement. In this design:

- **GDL decision** = small atomic authoritative record usable by software. Nonarchitectural operational, life, business and product decisions also belong in GDL under their own scope.
- **ADR profile** = an *optional metadata/Markdown projection* for architecturally consequential decisions. Its status, subject, text, timestamps and supersession links are resolved from GDL. **ADR Markdown is not another independently editable architecture authority.**
- **Materiality trigger:** mark decisions about service/data ownership, identity/authorization, storage/consistency, public API/compatibility, cross-product contracts, deployment topology, or costly-to-reverse strategic technical choices as \`adr_recommended\` (or \`adr_required\` if an existing approved project policy requires it). Routine low-risk choices stay GDL-only.
- **Record-only default:** capture tiny decisions automatically as candidates, not mandatory ADR files or PRs. On approval of a significant decision, prepare the ADR profile and offer optional, separately authorized repo export as \`docs/adr/NNNN-short-slug.md\`. A source-repository export requires proper repo identity, write/PR permission and explicit approval.
- **Provenance fields:** title, problem_context_refs, decision_drivers, considered_options (with trade-offs), rationale_refs, positive_consequences, negative_consequences, tradeoff_unknowns, acceptance_criteria, consequences_verified_refs, affected_systems, project_and_workstream_refs, canonical_decision_ids, source_refs, effective_at, superseded_by and confirmation_refs. If an LLM derives a consequence, mark it inferred until reviewed, not settled fact.
- **One-to-one preferred; one-to-many allowed:** a focused ADR may project one significant GDL decision; a broad ADR can group several atomic GDL IDs as a **derived decision set**, never collapse their independent approvals, scopes and lifecycles.
- **Historical import:** existing ADRs, specs and README architecture sections are high-value mined evidence. Even an imported ADR marked Accepted is **not** newly authenticated GDL authority solely by filename/merge/status; it must be reconciled to the applicable original approval and exact project scope.
- **Supersession:** new approved GDL revision creates a new ADR projection and explicitly references the former ADR; old records remain inspectable with superseded status. Edits to generated Markdown cannot silently change GDL.
- **Do not mix epistemic and implementation states:** ADR decision 'accepted' does not imply deployed, observed or verified behavior. Retain distinct implementation evidence and consequences assessment.

For operator convenience, a decision card shows the concise statement, **why**, original proof, impact and supersession. When architecturally material, the same record can expand into the human-readable ADR: Context / Decision / Consequences / Alternatives / Sources / Status.

ADR references:
- [MADR project](https://adr.github.io/madr/) and [MADR template](https://adr.github.io/madr/decisions/).
- [arc42 section 9 — architecture decisions](https://docs.arc42.org/section-9/).
- [Nygard-style ADR structure](https://www.adr.zone/adr-template/nygard).

**Proposal, not owner approval:** Focusa-native real-time ledger plus isolated mining workers, and GDL-native optional ADR projections.
