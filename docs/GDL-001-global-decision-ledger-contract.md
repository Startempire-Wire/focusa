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
