# 196 — Focusa → Wirebot Go: governed Workstream / workforce / evidence read projection

**Status:** proposed integration contract, 2026-11-22; NOT a released API, endpoint, or operator approval of new authority.
**Scope:** an owner-authorized, bounded read projection for the existing Wirebot Go consumer. This doc does not create a Workstream, change a reducer, schedule a worker, publish an artifact, or mark an outcome done.

## Canonical sources — reuse, never replace
- [Spec 164 Workstream Root](164-workstream-rooted-canonical-runtime-design.md) and [Spec 158](158-workstream-rooted-cognitive-runtime-canonical-state-partitioning-and-foundation-migration-spec.md): Project + Workstream + Continuity partition and attachment.
- [Spec 182 Project Foreman](182-focusa-project-foreman-workstream-intelligence-projection-spec.md): one project-responsible Foreman role per Workstream, NOT another Chief or independent authority.
- [Spec 96 Trajectory](96-trajectory-projection-and-daemon-stability-spec.md), Workpoints, Worksets/CallGraph and Completion/Receipt owners: separate guidance, work readiness, execution, verification and settlement.
- [Focusa cohesive agent flow](agent/02-focusa-cohesive-project-flow.md): resolve exact ScopeRef / WorkstreamId / ContinuityId; no implicit choice of the most recent Workstream.
- [Focusa Workforce](https://github.com/Startempire-Wire/Focusa-Workforce-Extension/blob/main/docs/05-extension-runtime-data-and-integration-contracts.md) already describes `WorkforceSnapshot`, `ForemanProjection`, `AgentProjection`, `WorkProjection`, `EvidenceProjection`. Reuse compatible fields and source refs rather than inventing a Go-only task ontology.
- [UIAI Issue 106 EPWA](https://github.com/WPUIAI/uiai-engine/blob/main/docs/106o-uiai-evidence-pwa-autonomous-work-state-substrate-invariant.md): UIAI owns capture, artifact and EPWA delivery; Focusa owns canonical mutable work/review/settlement.

## Requested outcome

From a previously verified Wirebot owner session the owner can see:
1. authorized Workstreams under an **explicit selected Project / Workstream / Continuity**;
2. Project Foreman and worker roles/assignments (including subagents), work activity and current owner attention;
3. small evidence summaries for each Workpoint with the producer's artifact/EPWA refs and **independent** verification and completion postures;
4. a source-bound handoff to the owning Focusa Workforce/EPWA detailed inspection experience.

No separate Go daemon, focusa database, roster authority, approval inbox or progress-counter ledger.

## Consumer data contract (candidate DTO, not a live Focusa API)

A trusted Focusa-owned producer supplies bounded records. Wirebot Core may relay a reduced projection only after binding the same verified principal, customer runtime, exact allowed Project/Workstream/Continuity, current source permission, and snapshot revision. Recommended consumer view shape (see Go's independent `docs/50-...` and validator):

```text
focusa owner readback
  source: Focusa; snapshot_ref, revision, observed_at, freshness, scope
  workstreams[]
    project_ref, workstream_ref, continuity_id, title
    foreman? {foreman_ref, display_name, status}
    workers[] {agent_ref, role_ref, class, assignment_ref?, status, workstream_ref}
    work[] {work_ref, workpoint_ref?, state, attention_ref?, evidence_refs[]}
    evidence[] {evidence_ref, workpoint_ref?, artifact_ref?, receipt_ref?,
      ui_ai_delivery_ref?, delivery_state?, verification_state?, settlement_state?}
```

The fields are **projection suggestions**; actual wire format must be resolved against source-owned API and schemas before any production producer is claimed. Prefer existing `WorkforceSnapshot` fields when possible. Workstream is a *scope partition*, not a display group. Never group an unqualified item into a Workstream by name or recency. Rows with ambiguous/mismatched scope must be omitted with an explicit degraded/blocked reason, not silently reassigned.

## Evidence and EPWA evidence-read gap (highest priority)

The current Workforce Extension `src/lib/evidence-trail.mjs` says its registry offers evidence *writes* but no complete evidence-read projection; it builds a reference-only fallback from `evidence_refs` / `receipt_refs`. That fallback is NOT authoritative verification.

Make a **source-owned, read-only evidence list/detail projection** available under verified Project/Workstream/Continuity and current viewer grant. It must expose:
- original immutable Evidence/Artifact ref and revision, producer, observed/captured time, claim, Workpoint/work ref;
- EPWA delivery reference and delivery state **as observed from UIAI**; avoid duplicating portable media bytes inside Focusa;
- separate capture, delivery, integrity, independent review/verifier, completion-decision and settlement states;
- current freshness/staleness, retraction, redaction, custody/access posture and exact safe recovery;
- source-owned trusted handoff descriptors, never raw token-bearing URLs or user-provided endpoint guesses.

A UIAI `ready` EPWA **does not** prove Focusa review, acceptance, completion or settlement. A Focusa accepted outcome requires its own canonical Completion Decision/Receipt. Unknown and pending reconciliation remain visibly unknown.

## Security and delivery

- **Read only** for first Go iteration; mutations stay in the existing Focusa / Workforce / Core operations. Future Go approvals use existing exact target, revision and receipt semantics, not a second action router.
- Require owner/tenant identity and Project/Workstream/Continuity authorization on the **producer and every read**, not merely at Go page load. Independent Workstream roots, delegated grants and attachment bindings must remain isolated.
- No raw Focusa local paths, remote endpoints, UIAI private viewer URLs, auth headers, cookies, secrets, unredacted screenshots, transcripts or internal topology in the Go payload.
- Send a bounded page + cursor, stable source revision, observed timestamp, freshness/recovery, and explicit capability posture. No unbounded project list/scan or automatic cross-Workstream rollup.
- **No new public API path is approved here.** Reconcile existing Focusa registry/daemon API and Core's first-party consumer before selecting an endpoint; produce a versioned schema and consumer contract tests at the owning boundary.
- Disconnected, stale, revoked, ambiguous, blocked and unsupported are distinct responses; a missing source must not become `0 agents` or `no evidence`.

## Acceptance / owner implementation handoff

- Two Workstreams with the same project and different Continuity IDs never mix Foremen/workers/evidence.
- Evidence attached to a Workpoint never leaks to sibling Workstream/owner/tenant.
- UIAI package delivery `ready` + verifier `unknown` remains `not verified`, NOT `done`.
- An expired grant, revoked artifact or stale proof immediately removes actionable links and serves a scoped reason/recovery.
- Owner's existing Focusa Workforce extension, Go, CLI/agent and source read paths present equivalent postures, with no duplicate auth/payment/approval authority.
- Demonstrate live source-backed read + denied cross-tenant read and safe resume before marking this as delivered.

**Related implementation owners:** `Startempire-Wire/wirebot-go` (render/projection tests), `Startempire-Wire/wirebot-core` (owner-bound first-party admission), `Startempire-Wire/Focusa-Workforce-Extension` (projection parity), `WPUIAI/uiai-engine` (EPWA producer).