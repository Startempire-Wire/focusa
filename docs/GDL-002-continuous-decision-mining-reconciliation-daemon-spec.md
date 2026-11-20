# GDL-002 — Continuous Decision Mining, Reconciliation & Activation Daemon

**Status:** DRAFT / ADVISORY / OPERATOR APPROVAL REQUIRED; no code or source access activated by this document  
**Date:** 2026-11-17  
**Canonical owner:** Focusa; consumers include Wirebot, OpenClaw, Pi, UIAI Engine, SOVOS, WINS and approved external surfaces  
**Depends on:** [GDL-001 canonical decision ledger](GDL-001-global-decision-ledger-contract.md)  
**Machine contracts:** [source classes](contracts/gdl-002-source-classes.v1.json) and [candidate schema](contracts/gdl-002-decision-candidate.v1.schema.json)

## 0. Product definition

A Focusa-governed, **continuously running, resumable decision-mining worker** reviews every *explicitly authorized* current and historical source; distills concise decisions; reconciles them against each other, project lineage, current approved decisions and constitutional authority; then asks the appropriate human to **accept, correct, reject, defer or investigate**. It never grants itself access or promotes its own findings.

**Prime directive:** do not ask the operator to reconstruct history that source-backed mining can recover. Show the smallest valuable, evidenced decisions first. Never trade this speed for fictitious authority.

```text
Authorized sources (connected apps, historic projects, documents, conversations)
    -> source manifest + incremental discovery + coverage ledger
    -> scoped read-only ingestion and immutable source-span references
    -> decision candidate extraction and concise normalization
    -> independent attribution/provenance/security verification
    -> deduplication + project lineage + existing GDL comparison
    -> temporal/scope/authority-aware contradiction analysis
    -> bounded adversarial challenge + readiness classification
    -> prioritized proposal inbox (human / API / CLI / MCP)
    -> explicit authenticated operator disposition
    -> Focusa reducer promotion under GDL-001
    -> bounded consumer projections + delivery receipts + drift checks
    -> incremental new/changed sources / repeat
```

Nothing is canonically activated at `mined`, `proposed`, `ready_for_approval`, `reviewed`, `PR_merged`, or `deployed`. Only a separately valid decision approval and promotion action may do so.

## 1. Source-of-truth reconciliation: re-use before building

Existing work to reconcile; **do not create a competing subsystem**:

| Existing source | Reuse or constraint |
| --- | --- |
| `docs/MEMORY_EXTRACTION_PIPELINE_SPEC.md` | Historically proposed OpenClaw/Pi batch session extraction, WINS memory approval queue, multi-sink delivery. **Decision approval is distinct from memory approval**; share ingestion/source infrastructure and human UI plumbing, never auto-promote a memory to a governing decision. Code-path claims in that document are not re-verified here. |
| `docs/UNIFIED_ORGANISM_SPEC.md` | Multi-source extraction, wiki/Obsidian, mem0/Letta/Focusa roles. Historical status and source ownership may have changed; use current runtime and source verification rather than treating dated inventories as live. |
| `docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md` | C.R.I.S.T. context ingestion and ongoing amendments; GDL supplies discovered governing candidates to Context/Interview, not its own project genesis. |
| `docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md` | Typed cognition, candidate/canonical ontology and semantic deltas. |
| `docs/120-adversarial-spec-workbench-and-operator-approval-gates.md` | Proposer, critic, evidence audit and operator gates. Adversarial review is reused in a bounded fashion. |
| `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md` | Bounded loops, checkpoints, recovery, scoring and stop controls; no runaway autonomous agent. |
| `crates/focusa-api/src/routes/context_sources.rs` | Scoped context source commit/list/retrieval, evidence and idempotency infrastructure. Existing endpoints are not assumed to be a complete multi-app connector system. |
| `apps/pi-extension/src/decision-validation.ts` | One-sentence, 160-character decision validation. |
| `docs/current/AUTHORITY_MODEL.md` and `docs/current/ARCHITECTURE_AUTHORITY_POLICY.md` | Exact scope, provenance versus authority, and human-rooted approval. No redefinition. |
| `docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md`, Specs 119, 133, 116 | Existing hash-chain/event receipts, durable bounded work/permission, and closure/admission semantics. |
| Wirebot App `docs/05-wirebot-app-core-process-contracts.md` | Existing first-run connected-source discovery + progressive life/business mapping; use the current authenticated app surface, not a second competing onboarding wizard. |

**Important integration nuance:** The prior memory spec routes approved memories to multiple sinks. GDL-approved decisions are **only** canonical in Focusa's governing decision owner. Wiki, mem0, Letta, Portable Memory, generated UI, OpenClaw instructions and application caches are *versioned read models* with source links and revocation/deprecation propagation, not alternative decision stores.

## 2. Source coverage: ANY source type, only within a verified grant

There is no universal permission to read all apps. **All connected sources that are authorized and compatible** should be discoverable through an extensible registry. Source classes and implementation priority are machine-readable in `contracts/gdl-002-source-classes.v1.json`.

### 2.1 Source classes

1. **Conversation and voice history:** current and archived ChatGPT conversation exports or supported connector reads; OpenClaw chat/session logs; Pi JSONL sessions; Discord, Slack, Teams, SMS and email conversations; voice assistant/meeting transcripts; customer support; agent-agent handoffs. Conversations require sender-role/quote/reply/turn attribution and source timestamps. ChatGPT history is **not** assumed programmatically accessible without explicit export or a supported authorized integration.
2. **Current, legacy and retired projects:** GitHub/GitLab repositories across **all operator-authorized organizations**, default and historical branches/tags, archived repos, milestones, pull requests/reviews/comments, commits, issues, worktrees, release notes, AGENTS/SOUL, wiki pages, specs, ADRs, backlog/Beads and migration logs. A code change or merged PR is *evidence of implementation*, never proof of the operator's decision authority.
3. **Documents and knowledge:** local filesystem, Obsidian, Wiki.js, Markdown, text, PDFs, DOCX, spreadsheets, slides, diagrams and OCR/transcriptions when needed; Google Drive/Docs, Microsoft OneDrive/SharePoint, Dropbox, Box, Notion, Confluence and other approved document systems. Preserve revisions, comments, embedded context and page/line/span provenance. Unsupported binary regions are gaps, never silently claimed as read.
4. **Business coordination:** Gmail/Outlook, calendars, Asana, Linear, Jira, Trello, ClickUp, HubSpot CRM, meetings, tasks, customer correspondence and project-management decisions. Human-authored commitments and customer requests are distinguishable from operator-approved product policy.
5. **Design/product and engineering evidence:** Figma, prototypes, code reviews, design decision annotations, deployment manifests, tests, UIAI evidence/receipts, Focusa Workstream/Trajectory/Workpoint decisions, agent completion logs and runtime observations. Execution evidence corroborates delivery, not intent.
6. **Imports and future connectors:** manual user uploads, authorized APIs, webhooks, internal event buses, signed ZIP/JSONL exports, plugin/tool integrations, and future self-hosted products via the same provider-neutral SourceAdapter contract.

### 2.2 Every SourceAdapter MUST declare

```text
provider_id; instance_id; tenant_ref; authenticated_principal_ref;
data_class; source_scope; acl_model; required_scopes; grants;
discover/list/read_span/read_delta capabilities; optional push/webhook;
cursor/watermark/version semantics; field freshness; delete/revoke handling;
pagination and rate limits; privacy/retention/region; content-type/parser;
source-link renderer; media/attachment support; health and test fixture.
```

Adapter contract:
`discover` -> `inventory` -> `read_delta(cursor)` -> `read_evidence_span(ref,version)` -> `checkpoint` -> `reconcile_deletions` / `revoke`. Each operation uses tenant- and item-level authorization, bounded quotas, retries/backoff and durable idempotency keys. Missing provider capability yields `unsupported`, not `zero decisions`.

Onboarding shows a **source coverage matrix**, including `connected`, `granted`, `unavailable`, `excluded`, `rate_limited`, `partially_indexed`, `up_to_date` and `stale`. "All sources processed" is only truthful relative to the explicit approved manifest.

## 3. Onboarding: recover past choices, do not start blank

**Stage A — Inventory (no broad private reads without consent):** show available connector types, authenticated scopes, candidate projects including aliases and historical identity, source types, date windows, archive ranges, exclusions and costs. Ask the user to grant *specific sources and read boundaries*, not a universal OAuth permission.

**Stage B — Evidence-complete historical sweep:** start from existing approved GDL decisions and current governance docs; crawl metadata to determine scope; backfill authorized history in recoverable pages; record per-source/partition/revision cursor; prioritize high-value recent and pivotal project decisions while still backfilling the approved full time range. Do not read only today's default branch or last conversation message. Resume after disconnect without claiming coverage of unseen files.

**Stage C — Recover project lineages:** build a reviewable graph of `same_project`, `renamed_from`, `forked_from`, `split_from`, `merged_into`, `superseded_by`, `retired`, `paused` with evidence refs, confidence and effective intervals. Resolve name collisions before inheriting authority. An archived project is an evidence source, not automatically an active scope. Historical decisions remain attached to their original temporal/project scope.

**Stage D — Produce an approval-ready baseline:** cluster duplicate observations; compare to existing canonical records; show only high-value novel decisions and genuine conflicts with source excerpts; preserve `unknown` and `awaiting_source`. For each proposed record, show why the miner believes the operator made that choice, what products it would affect, and whether a later authoritative choice superseded it.

**Stage E — Controlled activation:** approval is **per exact decision revision and exact scope**, authenticated, audited, revocable/supersedable. Offer batches *only* where all decisions are low-impact, same tenant/scope, individually previewed and policy permits; no blind accept-all. Existing architecture, access rights, finance, external effects, legal/policy/security and irreversible changes always demand focused approval.

**Stage F — Handoff to ongoing mining:** save coverage, cursors, challenges and user review preferences. Re-enable the same process on every newly authorized source or project. It is not tied to a once-only onboarding flag.

## 4. Continuous operation and bounded cadence

The "daemon" is a Focusa-owned **durable work loop**, not a second sovereign scheduler. Prefer existing source change subscriptions/webhooks plus Focusa's proven durable worker/Perpetua/Silent Sessions scheduling and recovery; avoid polling where native deltas exist.

- **On a source change:** debounce; permission-check; index changed spans, metadata and dependency-neighbor impacts; queue extraction/reconciliation.
- **On new connection or discovered historical project:** launch a checkpointed backfill limited to that grant, not a global re-index.
- **On GDL approval/supersession or authority-policy revision:** deterministically re-evaluate affected candidate clusters and consumer projections; show newly invalidated applicability.
- **Default digest proposal:** deliver a quiet daily summary of at most 3–5 valuable `ready_for_approval` items; exceptional material contradiction can alert promptly; no finding means no notification. User can choose different cadence, DND, domain filters, pause/resume and backlog limits.
- **Recurring audit:** use cursor/health drift checks and periodic rescan of sources without reliable change feeds. Source health and coverage staleness visible independently of candidate throughput.
- **Resource caps:** per-source tokens, bytes, model-call budgets, processing time, concurrency, queue backlog, retry ceilings, outage circuit-breaker and retention quotas. Use targeted deterministic filters before expensive LLM calls; only high-impact/uncertain candidates get independent second-model adversarial review.
- **Dedupe delivery:** idempotent `(source-instance, item-id, revision, segment-hash, extractor-version)` with decision-cluster stable identifiers; never repeatedly ask for an already rejected exact revision without material new evidence.

Mining and UI remain useful even when a connector is down; affected items are marked `source_unavailable` or `freshness_unknown`, never silently discarded. The worker may continue other independently authorized sources.

## 5. Evidence, extraction and attribution contract

For every candidate preserve `evidence_ref`, `source_instance`, `native_item_id`, `revision/etag/commit_sha`, `item_owner`, `speaker/author`, `quoted_party`, `utterance_role`, `captured_at`, `authored_at`, `event_time`, `source_time_zone`, `exact_span/page/line/turn`, `content_digest`, `grant_ref`, `visibility`, `parser_version`, and `extractor_version`.

**Extractor labels** (never collapse):
- `explicit_operator_decision` — direct, attributable, scope-aware statement made by the operator.
- `operator_preference`, `operator_instruction`, `operator_question`, `operator_hypothesis`, `operator_reconsideration` — different intent classes; questions are not decisions.
- `agent_recommendation`, `third_party_request`, `quoted_assertion`, `external_advice`, `documented_design`, `implemented_behavior`, `observed_outcome` — advisory/evidence only.
- `undetermined` — source or attribution insufficient.

The system MUST preserve exact quoted source evidence and identify whether the speaker is affirming, rejecting, exploring, quoting or recalling it. A second document copying the same AI summary is **not** independent corroboration. Better semantic match cannot upgrade speaker identity.

Decision statements: one bounded, imperative/declarative crystallized sentence <=160 characters, matching Focusa validation. Strip verbosity but never silently reverse polarity, omit essential qualifiers or infer an unstated approval. Multiple atomic choices from one paragraph become separate candidates with the same source span.

Model output MUST be accepted only as schema-validated **untrusted candidate data**; no tool-call instructions, embedded web links, shell commands, delegated approvals or executable code from retrieved content. Store evidence pointers preferentially, and enforce source ACL on dereference.

## 6. Reconciliation — the intelligence that justifies this product

Each candidate is compared against:
1. **Authority:** who actually said it, trusted identity, policy class, explicit approval and delegation.
2. **Temporal validity:** when expressed versus when effective; later choice *in the same scope* may conflict but is not automatically superior.
3. **Subject scope:** tenant, person/organization, life versus business, product, project lineage, Workstream, environment, customer, version, audience and applicability condition.
4. **Canonical GDL:** exact duplicates, semantic duplicates, amendments, compatible refinements, conflicts, withdrawn and superseded decisions.
5. **Current technical reality:** repo and deployed evidence distinguish `specified`, `implemented`, `observed`, `verified`, and `unknown`; implementation does not grant approval.
6. **Consequences:** affected project inheritance, enforcement boundaries, customers, cost and reversibility; detect when a project was renamed, sold, shelved, split or revived.
7. **Evidence quality:** accessible source spans, attribution certainty, contradictory citations, independence, staleness, parser gaps and provenance tampering.
8. **Constitutional constraints:** no candidate may promote a broader authority root, grant its miner new access or weaken an existing higher-order rule.

**Conflict graph** labels: `same_intent`, `refinement`, `supersedes_candidate`, `superseded_by_active`, `mutually_exclusive`, `scope_ambiguous`, `time_ambiguous`, `speaker_ambiguous`, `policy_violation`. The agent SHOULD recommend one of: `promote_new`, `amend_existing`, `replace_with_supersession`, `link_as_support`, `hold_for_question`, `reject_obsolete`, `no_action`; recommendation is never an authorization.

No opaque "confidence score" should replace proof. Show distinct `extraction_confidence`, `attribution_confidence`, `scope_confidence`, `freshness_status`, `contradiction_status` and a **boolean valid approval/authority check**. All percentages, if supplied, are judgmental uncertainty indicators—not proof of acceptance.

## 7. Adversarial guardrails and activation gates

Pre-approval pipeline with numbered checks:

| Gate | Must prove | Failure |
| --- | --- | --- |
| G0 Source admission | Verified principal, exact consent, tenant ACL, data-class boundary | No source read |
| G1 Provenance | Native ref, span, revision hash, author/speaker attribution | Hold as ungrounded |
| G2 Decision classification | A durable choice, not question/AI suggestion/implementation | Advisory-only |
| G3 Security | Secret/PII policy, injection quarantine, sanitized output | Quarantine/redact |
| G4 Scope and lineage | Project identity and applicability are not ambiguous | Hold for clarification |
| G5 Consistency | Compared against applicable active GDL and evidence history | Conflict card |
| G6 Independent critique | Material adversarial objections considered; original evidence inspectable | Hold/contest |
| G7 Approval readiness | Concise statement, metadata, effects, alternative disposition | Queue only |
| G8 Operator disposition | Authenticated human, pinned revision/scope/evidence/policy, permitted action | No promotion |
| G9 Canonical commit | Reducer/event transaction, idempotency, receipt, version fence | No active record |
| G10 Consumer delivery | Authorized versioned projection and acknowledgment | Mark projection pending |

The critic's tasks: attempt to disprove author attribution, detect an omitted reversal or newer explicit instruction, find inconsistent scope/project alias, identify AI-authored copy loops, and detect authority spoofing/prompt injection. Critic cannot rewrite the primary candidate without a new revision and audit trail.

**Non-negotiable:** Agent proposals are **never pre-approved decisions**. They may be **pre-screened / approval-ready**. Even an agent's apparently perfect quote is not a canonical operator approval event.

**Activation protocol:** `review(candidate_id, candidate_digest, expected_ledger_revision, action, exact_scope, grant_ref, auth_session)`. Valid dispositions: `approve`, `edit_and_approve`, `reject`, `defer`, `request_more_evidence`, `link_existing`, `supersede`. For edit-and-approve, show final text, new digest and fresh operator confirmation. Reject stale review tokens. Writes must use existing approved auth/Context Authority/receipt systems; no unsigned voice utterance or model output can approve a high-consequence change.

## 8. Human/agent experience: small surface, deep metadata

**Approval card, mobile-first:**

```text
[Decision proposition — one sentence, 1–2.5 lines]
GDL candidate • Project alias → current project • Historical source date
Why suggested: direct operator statement / 2 source spans
Current ledger: no match / compatible / contradicts GDL-183
Impact: Focusa + Wirebot • Risk: low / high
[View exact source] [Compare versions] [Preview affected products]
[Approve] [Edit] [Reject] [Snooze] [Investigate]
```

Default inbox sorting: high value/relevance, strong direct evidence, material current conflicts, expected number of downstream affected products, reversibility, and reduced operator workload. Do not optimize for number of notifications. Suppress already applied/no-op, quoted, unactionable or trivially duplicate findings.

Human review always provides a **jump to native exact source**, source coverage, extracted original quote, chronology, why/why-not, lineage map and full candidate JSON on demand. When native linking or access is unavailable, label that evidence gap and never fake a clickable citation. A correction is a new version/reconciliation event.

Expose review in existing trusted Wirebot/WINS/Pi/CLI surfaces; Discord can provide proposal links and low-risk disposition only if its verified auth context is sufficient. High-risk final approvals require step-up trusted auth. A source's own edit rights do not grant ledger approval.

## 9. Proposed API, CLI, MCP, events

Read/search operations:

- `GET /v1/gdl/miner/sources` — grant/connector health, counts, gaps and coverage per source, ACL filtered.
- `POST /v1/gdl/miner/inventory` — consent-limited metadata discovery only.
- `POST /v1/gdl/miner/backfill` — authorized bounded historical pass with checkpoint.
- `GET /v1/gdl/miner/jobs/{id}` — checkpoint, coverage and failure classes.
- `GET /v1/gdl/miner/candidates?status=ready_for_approval` — ranked queue with cursor.
- `GET /v1/gdl/miner/candidates/{id}` — full revision, evidence and conflict review.
- `POST /v1/gdl/miner/reconcile` — schedule targeted reconciliation, no canonical approval.
- `POST /v1/gdl/miner/review/{id}` — gated operator disposition via GDL-001 promotion boundary.

CLI: `focusa gdl sources`, `focusa gdl mine --scope ...`, `focusa gdl backfill --source ... --since ...`, `focusa gdl queue`, `focusa gdl candidate <id>`, `focusa gdl review <id>`, `focusa gdl pause|resume`, `focusa gdl coverage`.

MCP: `focusa_gdl_mine_status`, `focusa_gdl_sources`, `focusa_gdl_candidates`, `focusa_gdl_candidate_get`, `focusa_gdl_reconcile`, `focusa_gdl_propose`; approval tool only if trusted operator-auth and existing capability paths actually enforce it. A general-purpose LLM tool credential may not gain approval rights.

Events: `gdl.source.grant_changed`, `gdl.source.delta_ingested`, `gdl.candidate.ready`, `gdl.candidate.contested`, `gdl.candidate.disposed`, `gdl.decision.activated`, `gdl.decision.superseded`, `gdl.consumer.projection_stale`. CloudEvents envelope, durable outbox, at-least-once, idempotent consumers, no secret/raw text in broad event fanout.

Error classes include `permission_denied`, `source_revoked`, `source_unavailable`, `scope_mismatch`, `coverage_incomplete`, `parser_unsupported`, `evidence_missing`, `attribution_ambiguous`, `candidate_revision_stale`, `conflicting_canonical_decisions`, `approval_required`, `read_model_lag`, `rate_limited`. Never represent an error as empty findings.

## 10. Security, privacy, and availability

- Explicit, revocable, least-privilege **per account/project/source** grants before private reads. Source access and result access are both verified; a candidate view never broadens its source ACL.
- Source contents are untrusted *data*. Sandboxed/externalized parsers, no execution of retrieved commands, no SSRF from embedded links, no arbitrary filesystem traversal, no recursive MCP tool-instruction compliance, no source-to-source data exfiltration.
- Secret/key/credential/financial/health/private-person data classification and redaction before model calls or notifications. Prefer on-prem/local inference when required by data sovereignty; model endpoint policies are operator-controlled. Sensitive material does not go to external LLMs merely because an adapter can read it.
- Raw bytes remain under the original source or approved encrypted evidence owner where possible; indexes keep minimum needed digest, span and authorized excerpt. Tokenized search indexes and backups preserve tenant partitioning and deletion/retention policy.
- Revocation stops scans, future extraction, future dereference and consumer projections based solely on that grant. Existing approved decisions need targeted privacy/retention review; do not silently erase audit or preserve revoked private excerpts in notifications.
- Retired/archived projects do not supply current production policy by default. If context is stale, denied or absent, fail closed for mutation decisions.
- All review actions are auditable, idempotent, signed/authenticated as applicable, with optimistic revision fencing. Reject replays and role confusion.
- No automatic writes back to GitHub/Drive/CRM/chat and no data export to third parties as a consequence of *mining*. Publishing, modifying source documents, scheduling, buying, installing or deploying are separate grants.
- Pause, disconnect, delete authorized private index data, portability/export, per-source retention and explainability are first-class operator controls.

## 11. Persistence, checkpointing and project migrations

A `SourceRun` has statuses: `discovered -> awaiting_grant -> admitted -> indexing -> extracting -> reconciling -> review_ready -> checkpointed`, with `paused|blocked|degraded|stale|retired` as typed exceptions. Each source partition has stable item keys and independent watermarks. Never advance a cursor before durable record/coverage acceptance.

One source revision can yield multiple candidates. A `CandidateCluster` groups duplicates without erasing differing claims, source attribution, scope or effective dates. A `ProjectLineage` maps legacy names/paths to verified identities, but does not move decision authority automatically. Any merge/split produces explicit migration candidates and operator dispositions.

Use existing Focusa event-chain, reducer, context-source infrastructure and durable work scheduling. Database/queue topology and exact runtime owners are implementation-phase questions to verify against deployed code; this spec does not authorize a new standalone privileged service.

## 12. Tests and acceptance matrix

**Source coverage:** synthetic public/private connectors; ChatGPT export, OpenClaw/Pi multi-turn logs, Git history including archived branches, Wiki/Obsidian, Drive docs, email, PM tool, CRM, meeting transcripts. Verify receipts for all claimed spans, pagination, cursor integrity, revocation and unavailable-source labels.

**Evidence:** distinguish operator instruction from agent suggestion and quoted third party; correct speaker/role; ambiguous multi-party conversation; reversible wording; negated/retracted decision; malformed document; deleted or moved original; duplicated AI summary; conflicting effective dates.

**Reconciliation:** same decision across 10 sources, contradictory choices, exact and semantic duplicates, approved supersession, archived project renamed/merged/split, cross-domain and cross-tenant collisions, historical accepted choice in a retired repo, current explicit operator override.

**Adversarial:** prompt injection disguised as spec/AGENTS.md/decision, fake signed approval, forged operator handle, malicious MCP tool output, model attempts to self-approve, hallucinated source URI, stale digest replay, bulk-approval scope leak, race with policy revision, poison source to external model.

**Runtime:** crash/restart between extraction and cursor, webhook duplicate/reorder, backfill versus live event race, slow or revoked OAuth, expensive file, model timeout, queue backpressure, large archives; ensure no missed or duplicate activation. Verify that daemon being down cannot break normal operating tasks.

**Consumer:** approved decision appears in authorized Focusa resolver and consumer after event, receipt and exact revision; rejected candidate never appears as policy; UIAI/other consuming agents honor exclusions and stale statuses; loss of connector grant blocks future dereference; no fake implemented/verified claims.

**Acceptance gates:** source manifest/coverage accurate; evidence citation verifiable; no unapproved active decisions even under attacks; operator can approve/edit/reject in compact UI; restart preserves state; exact-scope resolver parity in REST/CLI/MCP; at least one real connected source and two independent consumer surfaces proven end-to-end with receipt; runtime proof before release. Source code docs/contracts are NOT proof of installed behavior.

## 13. Minimal useful shipping sequence (no fake ceremony)

1. **Audit/reuse:** map existing memory queue, historical processors, context sources, approval gate, GDL Decision primitive, event chain, scheduler, GUI. Produce concrete `reuse|extend|replace|absent` map.
2. **Read-only historical backfill:** first adapter local/project Git + Pi/OpenClaw logs + consented conversation export; store source coverage and operator-attributed candidate cards. Existing sources continue independently if one adapter fails.
3. **Reconciliation:** stable candidate clustering, project alias lineage, source-span links, exact GDL comparison, conflict cards and independent critique for consequential cases.
4. **Human approval:** reuse trusted WINS/Wirebot app UI and Focusa governance; require digest-bound confirmation; output canonical GDL record/receipt. **Never default to auto-approve.**
5. **Cross-product reads:** REST/CLI/MCP parity, scoped OpenClaw/Pi integration, consumer event projections and drift checks.
6. **Connector expansion:** Gmail/Drive, HubSpot, Asana, calendars, Figma, other verified sources using standard SourceAdapter rather than app-specific memory silos.
7. **Continuous mode:** incremental changes/webhooks, low-noise Perpetua-style queue delivery, provenance monitoring and bounded re-evaluation.

Do not hold up the first valuable historic extraction on integrating every provider. Do not claim "all sources" when some have no grant, no extractor, or no available API. The manifest exposes all inclusions and gaps.

## 14. Decisions explicitly reserved for the operator

The following are *not* authorized by this draft: enabling any connector, opening historical private correspondence, sending data to a model, choosing source regions or retention, accepting any candidate, activating any architectural constitution/delegation, bulk high-risk approvals, deploying new jobs, sending notifications, or broadcasting private evidence to other tenants/products. Configure each through existing trusted access/approval paths as the implementation progresses.

## 15. External standards and references

- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — evidence lineage with entity/activity/agent attribution.
- [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html) — canonical JSON for deterministic digest and signature.
- [CloudEvents](https://github.com/cloudevents/spec/blob/main/cloudevents/spec.md) — at-least-once interoperable event metadata; application must implement its own idempotency.
- [MCP official documentation](https://modelcontextprotocol.io/specification) — adapter/tool discovery and authorization/security model; remote MCP content remains untrusted.

**Requested disposition:** operator review of GDL-002 as a candidate Focusa subsystem. Approval of this *specification* would authorize planning/decomposition, not any connected-source read, decision promotion, or production deployment.


## 16. Runtime boundary settlement — miner is a Focusa-owned worker, not the agent query server (2026-11-17)

**Normative architecture refinement; see [GDL-001 sections 7–8](GDL-001-global-decision-ledger-contract.md).** "Decision Mining Daemon" is a *functional name* for a persistent mining capability, not a mandate for a second authoritative daemon/process with an independent database or API.

**Required ownership:**
1. **Focusa daemon on KH** (under currently documented KH/OVH topology) is the one canonical decision store, reducer, authority resolver, authenticated approval endpoint, search index owner and real-time agent query service.
2. **Decision Miner workers** are separate and resource-bounded; heavy parsing, embeddings, history backfills, external connector reads and LLM adjudication run on approved OVH execution capacity when available. Schedule/stop/retry/reconcile via existing Focusa durable worker/Silent Session/background capability, not an independently authoritative cron manager. Do not run resource-spiking scans on KH or in request handlers.
3. Workers send schema-valid **candidate submissions** with evidence/coverage through approved API and capability grants. No direct SQL, event-chain append, approval state writes, cross-tenant global index or private GDL authority token on workers. Submission can return duplicate/no-op under idempotency.
4. **Agent clients** Pi, OpenClaw, Wirebot, SOVOS, UIAI and other MCP/REST/CLI consumers query Focusa's **committed read model** directly. They never need to call miner workers to resolve a decision. On-demand discovery and governing resolution are distinct, policy-checked operations.
5. Existing **Focusa SSE durable replay/broadcast** carries decision-reference invalidations and revision/watermark hints to consumers; it is not a new event bus and must not stream arbitrary raw mined private excerpts.
6. When mining lags or is offline, previously approved decisions remain searchable with truthful source-coverage freshness. No inferred refresh, stale authority or false denial of existing state.
7. If a source changes while an agent searches, return the last **verified canonical decision revision** and independently disclose the miner's coverage/watermark. New unreviewed source text does not alter governing decisions.

### 16.1 Proposed agent-facing retrieval behavior

- Agent asks **"What was decided about Wirebot deployment levels?"** -> Focusa search yields source-backed active decisions in authorized scope; include optional clearly advisory candidate results only through a separate explicitly requested mode.
- Agent asks **"What do I have to honor before this release?"** -> Focusa resolve computes all applicable current approved constraints from exact project/Workstream/continuity/attachment and action class; conflicting or stale governance yields an explicit denial/needs-review result, not probabilistic retrieval.
- Agent asks **"What changed since I last loaded decisions?"** -> use durable stream cursor and bounded decision-version refresh.
- Agent asks **"Why was this decided?"** -> use metadata/evidence refs with permission-bound exact source spans; ADR projection, if present, is human-readable expansion of the same GDL decision.
- Agent asks **"Can you find older choices from archived projects?"** -> Search the historical/evidence corpus through an explicit **advisory historical mode**, preserving original project scopes and approval status, never applying retired decisions to a live project without reconciliation and owner approval.

Use the existing Focusa machine-readable capability discovery/contract generation for read-only tools: focusa_gdl_search, focusa_gdl_resolve, focusa_gdl_get, focusa_gdl_context, focusa_gdl_status. None of them may spawn unbounded mining or model work. Review/promotion stays on the separate governed write path.

### 16.2 ADR candidate enrichment without bureaucracy

When a mined candidate is architecturally significant, extraction should also attach ADR-context source refs (problem, options, drivers, rationale and consequences). This **does not change the 160-character canonical statement**. Prefer Nygard/MADR-compatible derived templates. Mark speculative rationale/consequences as inferred; user review must distinguish direct historical choices from model reconstructions.

After authenticated approval of the exact significant GDL decision, offer a Markdown ADR export **only when requested or required by an approved project policy**. Do not automatically open hundreds of PRs for every project-history decision. Do not mistake a historical document labeled \`ADR: Accepted\` for current signed operator approval. Keep original source and project lineage in the metadata graph.

### 16.3 Additional acceptance cases

- 10,000-asset historical backfill is in progress on bounded execution worker, while the Focusa daemon still resolves approved decisions within its interactive performance budget.
- Index goes behind the canonical ledger for one approval: read-your-writes uses safe canonical fallback or honest read-model-lag posture, never silently stale data.
- Scanner/connector/model unavailable: Focusa agent search stays available for previously approved decisions.
- A newly approved decision issues a bounded durable invalidation; a connected Pi or OpenClaw agent sees the new applicable revision without process restart and cannot apply an outdated revoked record to a consequential action.
- Search for a deliberately injected unapproved but high-similarity candidate does not include it among governing decisions.
- Three old project identities with overlapping names cannot silently import each other's ADRs or decisions as active authority.
- ADR projection and compact GDL statement agree on the decision ID, scope, approval revision and supersession history.


## 17. Focusa ontology binding and mining-candidate migration (2026-11-20 audit correction)

**Source-linked:** [GDL ↔ Focusa two-pass data-model reconciliation](GDL-FOCUSA-DATA-MODEL-RECONCILIATION-2026-11-20.md) and [Focusa candidate JSON Schema](contracts/gdl-002-decision-candidate.v1.schema.json).

The miner is a Focusa-supervised **producer of candidate semantic Decision objects**. It must submit its GDL-002 decision-specific payload **through the existing Focusa authorization/candidate ingestion owner**, mapped to \`focusa.semantic_candidate.v2\` per Spec135F, with registered \`Decision\` definition, verified scope, source/evidence refs, parser/attribution policy and stable candidate identity. Candidate status or LLM confidence **never** authorizes promotion. Only the Focusa semantic verification + operator-rooted approval + promotion policy/reducer may create \`focusa.canonical_semantic_record.v2\` Decision state.

**Existing code distinctions that must survive:**
- Source and claim reads link to \`ContextSourceRecord\` / \`ContextClaimRecord\`. A historical \`ContextDecisionRecord\` is a narrow claim-review or contradiction-resolution outcome, not operator architectural approval. Link it as evidence only.
- \`FocusState.decisions\` (30 strings) and Context Cognition are bounded read models, not historical decision storage. Do not fill context with every mined record.
- Project and Workstream identity must be resolved from actual ScopeRef/WorkstreamRoot binding; \`tenant\`/\`ecosystem\` candidate labels describe applicability, not a canonical Focusa scope kind. A \`continuity_id\` alone cannot own a decision. Historical cross-project decisions remain in originating scopes unless explicitly authorized by approved reference/migration.
- Shared cognition and Spec135F semantic registry MUST own type/relationship/verification/promotion/slice rules; don't create GDL-only route-local type IDs without registration, or a new "mining approval" store.
- GDL candidate and active search indexes must be **separate derived projections**. Agent queries never run miner LLMs or crawl external history synchronously.
- Existing SSE + Spec135F semantic delta contracts own change signals; model/proposal update events cannot appear as canonical activation.

**Candidate readiness check:** Before an item is \`ready_for_approval\`, it must have a verified \`focusa_mapping\` (actual Decision type definition ID, source/owner plane, canonical scope binding, applicable WorkstreamRoot if required, verification policy ref and promotion policy ref). The [JSON Schema](contracts/gdl-002-decision-candidate.v1.schema.json) now enforces this mapping for approval-ready candidates. An ambiguous historical project remains \`reconciling/contested/blocked\`, not eligible for approval. Neither a fictional \`ecosystem\` ScopeRef nor a reused GDL agent credential can bypass it.

**Rollout gates:** 1) audit current installed Spec135F/158/164 capabilities; 2) map and verify shared cognition Decision semantics and versioned candidate/canonical schema; 3) ingest one Workstream's historical source revisions as proposals; 4) verify explicit human approval and semantic promotion; 5) prove correct read-only search/scope inheritance across two Workstreams; 6) verify SSE, revision lag, cross-version replay, privacy and exported ADR consistency; 7) then broaden connectors and backfill. Do not silently enlarge Focusa's foundations under GDL.
