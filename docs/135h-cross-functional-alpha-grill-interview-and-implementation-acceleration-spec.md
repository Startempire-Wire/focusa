# Spec 135H — Cross-Functional Alpha, Grill Interview Strategy, and Full-Series Implementation Acceleration

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md), [Spec 135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md), and [Spec 135E](135e-cross-spec-amendments-migration-and-closure-matrix.md)  
**Closure relationship:** mandatory acceleration amendment; Spec 135 decomposition and implementation may not proceed without applying this document.  
**Scope:** decided implementation accelerators, C.R.I.S.T. Interview strategy, cross-functional walking skeleton, vertical tracer-bullet decomposition, generated contracts, selected open-source frameworks, license automation, dogfood integration, parallel work frontiers, and decomposition instructions.

---

## 0. One-line definition

Implement the complete Spec 135 series fastest by establishing one thin but real end-to-end path through every major function immediately, keeping that path continuously operational, and widening all remaining requirements in parallel through decided open-source frameworks and existing Focusa/UIAI/Pi primitives rather than custom reinvention.

---

## 1. Normative posture

This document makes implementation decisions. It does not present implementation menus.

Agents decomposing or implementing the Spec 135 series must:

```text
use the decided stack;
reuse existing Focusa/UIAI/Pi ownership;
create vertical tracer-bullet work;
produce the Cross-Functional Alpha first;
open parallel expansion frontiers immediately after their seams stabilize;
keep every accepted requirement in the Complete Feature Ledger;
and request a versioned amendment only when a decided dependency fails a recorded conformance gate.
```

Agents must not pause decomposition to ask the operator to choose among frameworks already decided here or in Spec 135 §17 and Spec 135D §7.

Implementation ordering is designed for speed. It does not authorize omission or indefinite deferral.

---

## 2. Amendment authority

### 2.1 Spec 135B amendment

This document makes `focusa.interview.strategy.grill-with-docs.v1` the required initial C.R.I.S.T. Interview strategy and adds the records, behavior, and safety boundaries defined in Part I.

### 2.2 Spec 135D amendment

This document adds a second mandatory execution axis alongside Orders 0–11:

```text
Full Completion DAG
+ Cross-Functional Alpha
+ parallel expansion frontiers
```

The Full Completion DAG remains the closure graph. The Cross-Functional Alpha governs the earliest integration order.

### 2.3 Spec 135E amendment

This document adds the third-party dependency, model-license, notice, SBOM, replacement, and abandonment policies defined in Part IV.

### 2.4 Conflict rule

Where earlier Spec 135 text says a framework or workflow should be selected, qualified, or considered, this document’s explicit decision governs unless a later operator-approved amendment supersedes it.

---

# Part I — C.R.I.S.T. Interview Acceleration

## 3. Grill-with-Docs decision

Focusa will adapt the MIT-licensed `grill-with-docs`, `grilling`, and `domain-modeling` disciplines from `mattpocock/skills` into a versioned Focusa Interview Strategy.

Upstream references:

- `https://github.com/mattpocock/skills/blob/main/skills/engineering/grill-with-docs/SKILL.md`
- `https://github.com/mattpocock/skills/blob/main/skills/productivity/grilling/SKILL.md`
- `https://github.com/mattpocock/skills/blob/main/skills/engineering/domain-modeling/SKILL.md`
- `https://github.com/mattpocock/skills/blob/main/LICENSE`

The upstream MIT copyright and permission notice must be preserved in the vendored/adapted skill package and third-party notices.

### 3.1 Ownership boundary

```text
Focusa Interview Engine
  Canonical questions, answers, decisions, provenance, sensitivity,
  supersession, readiness, project scope, and Spec handoff.

Grill-with-Docs Interview Strategy
  Questioning discipline, branch traversal, recommendations, fact lookup,
  terminology challenge, edge cases, and decision/glossary/ADR candidates.
```

The skill is not a second Interview store, workflow engine, authority system, or repository writer.

### 3.2 Required strategy identifier

```text
focusa.interview.strategy.grill-with-docs.v1
```

The first production Interview implementation must use this strategy by default unless a project’s approved domain pack adds a stricter compatible strategy overlay.

---

## 4. Interview Strategy contract

```rust
pub trait InterviewStrategy {
    async fn generate_next_question(
        &self,
        context: InterviewContext,
    ) -> NextQuestionProposal;

    async fn evaluate_answer(
        &self,
        question: InterviewQuestion,
        answer: InterviewAnswer,
    ) -> AnswerAssessment;

    async fn assess_readiness(
        &self,
        state: InterviewState,
    ) -> InterviewReadinessAssessment;
}
```

The generated contract must also be available through OpenAPI/JSON Schema/TypeScript.

### 4.1 Next-question proposal

```yaml
schema: focusa.interview_next_question_proposal.v1

strategy_id: focusa.interview.strategy.grill-with-docs.v1
strategy_version: 1
session_id:
parent_question_id:
decision_branch_id:
question:
reason_for_asking:
triggering_gap:
recommendation:
recommendation_basis_refs: []
environment_facts_checked: []
contradiction_refs: []
linked_context_refs: []
linked_spec_sections: []
domain_term_candidates: []
architecture_decision_candidates: []
decision_required: true
priority: blocker | high | normal | optional
answer_type:
readiness_effect:
stop_condition:
```

### 4.2 Fact-versus-decision law

Before asking a question, the strategy must determine whether the answer is discoverable from:

- project files;
- code;
- repository history;
- Focusa canonical state;
- connected sources;
- UIAI research;
- provider state;
- approved context claims.

Discoverable facts must be retrieved and cited rather than asked of the operator.

Questions are reserved for:

- preferences;
- tradeoffs;
- priorities;
- authority;
- acceptance boundaries;
- ambiguous intent;
- unresolved contradiction;
- operator-owned facts unavailable from approved sources.

### 4.3 Recommendation law

Every operator decision question includes one recommended answer and its basis.

The recommendation is advisory. The operator’s answer remains authoritative.

### 4.4 One-question law

The Interview presents one primary question at a time. It may show supporting facts, recommendation, consequences, and source references, but it may not present a batch of unrelated questions.

### 4.5 Branch-completion law

The strategy walks one decision branch until:

- its prerequisite facts are resolved;
- dependent decisions are resolved;
- contradictions are recorded;
- glossary and ADR candidates are captured;
- the branch reaches a declared stop condition.

It then moves to the highest-value unresolved branch.

---

## 5. Required Grill tranches

The initial implementation includes these decided tranches:

```text
Discovery Grill
  Purpose, desired state, users, stakeholders, present reality.

Boundary Grill
  Scope, non-goals, authority, privacy, retention, handoffs.

Failure Grill
  Edge cases, known failures, recovery, degraded behavior, not-done-if rules.

Evidence Grill
  Success criteria, proof, freshness, verification, closure requirements.

Architecture Grill
  Hard-to-reverse system boundaries, integrations, ownership, compatibility.

Spec-Readiness Grill
  Remaining blockers, contradictions, unknowns, and operator approval to hand off.
```

Domain packs may add questions and evidence policies to these tranches. They may not remove the core tranches.

---

## 6. Glossary and ADR candidate behavior

The upstream domain-modeling discipline writes glossary terms and ADRs directly. Focusa converts them into governed candidates.

```yaml
schema: focusa.domain_term_candidate.v1

candidate_id:
term:
proposed_definition:
conflicting_terms: []
source_question_ref:
source_answer_ref:
context_refs: []
status: candidate | approved | rejected | superseded
```

```yaml
schema: focusa.architecture_decision_candidate.v1

candidate_id:
title:
context:
decision:
alternatives_considered: []
consequences: []
source_question_refs: []
source_answer_refs: []
status: candidate | approved | rejected | superseded
```

Direct repository writes are forbidden during Interview.

Approved glossary or ADR projections use:

```text
preview
→ operator approval
→ governed write
→ Receipt
```

An ADR candidate is created only when the decision is hard to reverse, surprising without context, and the result of a real tradeoff.

---

## 7. Interview speed and fatigue controls

The strategy must maximize information gained per operator interaction.

Required behavior:

- rank questions by blocker value and downstream dependency count;
- avoid asking facts already available;
- collapse equivalent questions;
- show a concise recommendation;
- autosave every answer;
- allow pause/resume after every answer;
- checkpoint after each completed branch;
- stop a tranche when further questions have low expected spec impact;
- reopen automatically when new context invalidates a prior answer or creates a material gap.

The Interview UI displays:

```text
why this question matters
what Focusa already knows
recommended answer
consequences of the decision
linked spec gaps
branch progress
```

---

# Part II — Cross-Functional Alpha

## 8. Two-axis implementation model

The implementation uses two simultaneous axes.

### Axis A — Full Completion DAG

Spec 135D Orders 0–11 remain the authoritative complete closure graph.

### Axis B — Cross-Functional Alpha

A narrow, production-shaped, end-to-end path through every major product function is implemented first.

The Cross-Functional Alpha is not a reduced product contract. It is the first integrated traversal through the complete architecture.

### Axis C — Expansion frontiers

As soon as an Alpha seam is stable, all unblocked expansion tasks begin in parallel:

- remaining connectors;
- remaining providers;
- remaining vertical/domain packs;
- remaining client surfaces;
- remaining renderers;
- hardening, migration, performance, accessibility, and security.

---

## 9. Alpha completion path

The following slices are mandatory and ordered. Every slice must be real, demoable, testable, and evidence-backed.

### Alpha 0 — Generated contract spine

Deliver:

```text
Rust canonical schemas
→ OpenAPI + JSON Schema
→ generated TypeScript contracts/client
→ one CI drift gate
```

Minimum covered contracts:

- Project Genesis;
- Context Artifact and Claim;
- Role Profile;
- Interview Strategy, Question, and Answer;
- Spec Workbench handoff;
- provider-neutral task;
- Workspace Artifact;
- Mission Canvas and Work Surface;
- ProjectRootKey, WorkstreamKey, and AttachmentKey.

### Alpha 1 — Real Context

```text
create/bind project
→ ingest one Markdown/code source
→ ingest one real PDF through UIAI Documents/Docling
→ preserve provenance and page refs
→ chunk and index
→ hybrid retrieve
→ display bounded Context in Mission Canvas
```

### Alpha 2 — Real Role and Grill Interview

```text
operator role seed
→ grounded AI Role draft
→ operator approval
→ Grill-with-Docs strategy asks one question
→ recommendation and source basis shown
→ answer persisted
→ client closes
→ Interview resumes correctly
```

### Alpha 3 — Real Spec and Task

```text
Context + Role + Interview
→ Spec 120 Project Genesis handoff
→ one real adversarial challenge/reconciliation cycle
→ operator approval
→ provider-neutral task plan
→ one real Beads task materialized
```

### Alpha 4 — Workpoint, proof, closure, and Receipt

```text
Beads task
→ scoped Workpoint
→ Work Rail
→ governed work action
→ Evidence link
→ closure reconciliation
→ Receipt
→ verified strike-through
```

### Alpha 5 — UIAI rich artifact and live refresh

```text
UIAI browser read or screenshot
→ Workspace Artifact descriptor
→ Focusa Evidence link
→ targeted SSE invalidation
→ related Work Surface refreshes automatically
```

### Alpha 6 — Multiplexing and isolation

```text
one Pi Work Surface
+ one UIAI browser Work Surface
+ explicit Attachment identities
+ explicit steering target
+ one isolated browser context
+ restart/rehydration proof
```

### Alpha 7 — Vertical projection

The same live canonical project switches through:

```text
General
→ Software
→ Research
```

The switch changes layout, terminology, theme, artifact renderer, and evidence emphasis without changing canonical project/session authority.

### Alpha 8 — Spec 135 dogfood loop

The implementation of the Spec 135 series itself becomes the first permanent end-to-end dogfood project:

```text
Context
→ Role
→ Grill Interview
→ Project Genesis Spec
→ Tasks
→ Workpoint
→ UIAI artifact
→ Evidence
→ Receipt
→ multiplexed Mission Canvas
```

This path becomes a required merge and release gate.

---

## 10. Alpha slice law

Every Alpha ticket cuts through all required layers:

```text
schema
→ reducer/persistence
→ API
→ generated client
→ UI
→ real integration
→ tests
→ evidence
```

A horizontal-only task may support an Alpha slice, but it does not satisfy the slice by itself.

Forbidden Alpha completion evidence:

- mock-only provider;
- static card;
- placeholder success envelope;
- unpersisted UI state;
- hand-written client DTO diverging from server schema;
- transcript-only Interview;
- manually refreshed artifact;
- one global active-session assumption;
- provider closure without Focusa verification;
- generated Markdown without Spec 120 approval lifecycle.

---

# Part III — Decided Implementation Accelerators

## 11. Contract generation stack

The decided contract stack is:

```text
Serde
+ Schemars
+ Utoipa / utoipa-axum
→ generated OpenAPI 3.1 and JSON Schema
→ openapi-typescript
→ shared TypeScript contracts and fetch client
```

Licenses:

- Schemars: MIT;
- Utoipa: MIT OR Apache-2.0;
- openapi-typescript: MIT.

Manual duplicate client DTOs are forbidden when generation can represent the contract.

A schema-drift gate runs in CI and fails when generated artifacts differ from committed/generated expected output.

---

## 12. Interview and implementation skill stack

Focusa will adapt these MIT-licensed disciplines into agent instructions:

```text
grill-with-docs
  C.R.I.S.T. Interview questioning and domain-language discipline.

to-tickets
  Vertical tracer-bullet decomposition with explicit blocking edges.

prototype
  Throwaway state-machine or UI experiments for unresolved high-risk seams.

implement + tdd + code-review
  Focused implementation, seam-level tests, review, and commit discipline.
```

These skills guide agents. They do not replace Focusa canonical systems, Spec 120, provider-neutral work items, Evidence, or Receipts.

---

## 13. Document stack

The decided document path is:

```text
UIAI Engine Documents
→ pinned Docling Serve v1 API
→ DoclingDocument JSON
→ Docling HybridChunker
→ Focusa normalization/provenance adapter
→ FTS5/vector indexes
```

Docling and Docling Serve are MIT licensed.

Focusa does not create another document parser, layout model, OCR pipeline, table parser, or generic chunker.

Direct ingestion remains for Markdown, plain text, source code, JSON, JSONL, and CSV.

---

## 14. Retrieval stack

The decided retrieval stack remains:

```text
SQLite FTS5
+ sqlite-vec behind a pinned adapter
+ fastembed-rs local embeddings/reranking
```

No external vector database is introduced before a repeatable Focusa benchmark proves the selected local stack violates an approved performance requirement.

---

## 15. Shared UI stack

The decided UI stack is:

```text
SvelteKit 2
Svelte 5
Tailwind CSS 4
shadcn-svelte
Bits UI
Paneforge
TanStack Query for Svelte
TanStack Table
TanStack Virtual
Svelte Flow
Tauri 2 in UIAI Engine Cockpit
```

Ownership:

- shadcn-svelte/Bits UI: generic accessible controls;
- Paneforge: resizable Mission Canvas panes;
- TanStack Query: server-state cache and targeted invalidation;
- TanStack Table: Work Rail, task, artifact, source, and session grids;
- TanStack Virtual: large interviews, histories, artifacts, sources, and session inventories;
- Svelte Flow: task, ontology, claim/evidence, matter, and session/dependency graphs.

Focusa builds only domain-specific interaction and visual grammar on top of these primitives.

---

## 16. Graph and code-reality stack

The decided analysis stack is:

```text
petgraph
  canonical DAG traversal, cycle detection, dependency ordering, graph algorithms.

Tree-sitter
  incremental multi-language syntax parsing.

ast-grep
  structural code search, architecture rules, codemods, and wide migrations.
```

Use these for:

- Reality Scanner;
- Call Stack Verify;
- dependency graphs;
- route/action discovery;
- singleton detection;
- Pi namespace migration;
- generated-contract migration;
- architecture contradiction checks.

Focusa does not build its own parser or structural search engine.

---

## 17. Artifact renderer stack

The decided renderer foundations are:

```text
PDF.js
  PDF and source-page rendering.

CodeMirror Merge
  code diffs, text comparisons, and legal redline foundation.

Apache ECharts
  markets, telemetry, proof, and diagnostic charts.

Svelte Flow
  graph projections.
```

All are wrapped behind Focusa Workspace Artifact and renderer contracts. Renderer-internal data models do not become canonical Focusa state.

---

## 18. Connector stack

The decided connector substrate is:

```text
oauth2-rs
keyring-rs
reqwest
serde
provider delta cursor storage
wiremock fixtures
```

Provider clients:

```text
Google Drive / Gmail
  generated google-apis-rs clients plus Focusa adapter.

GitHub
  Octocrab plus Focusa adapter.

Linear
  graphql_client generated queries plus Focusa adapter.

Microsoft Graph
  typed reqwest/OData adapter owned by Focusa; no unverified third-party SDK dependency.

Asana
  typed reqwest adapter from the provider’s published API contract.
```

Provider SDK objects stop at the adapter boundary. Every adapter returns Focusa source, delta, health, authorization, work-item, and closure contracts.

---

## 19. Test and proof stack

The decided Rust test stack is:

```text
cargo-nextest
rstest
proptest
insta
wiremock
```

Required uses:

- cargo-nextest: parallel test execution;
- rstest: fixture and parameter matrices;
- proptest: reducer, scope, replay, migration, and multiplexing invariants;
- insta: schemas, read models, Receipts, render packets, and migration projections;
- wiremock: Google, Microsoft, GitHub, Linear, Asana, UIAI, and failure/recovery fixtures.

Required property tests include:

```text
ProjectRootKeys never bleed state.
Focused Work Surface never changes canonical authority.
Replay produces the same canonical projection.
Candidate semantic state never promotes without policy.
Closing a view never terminates a session implicitly.
Isolated browser contexts never share storage.
Provider closure never implies verified Focusa completion.
Interview answer supersession never destroys operator history.
```

---

## 20. License, advisory, and SBOM automation stack

The decided compliance stack is:

```text
cargo-deny
cargo-about
pnpm/npm license inventory for JavaScript packages
Syft SBOM generation
GitHub dependency review and advisory scanning
```

Default allowed dependency licenses:

```text
MIT
Apache-2.0
BSD-2-Clause
BSD-3-Clause
ISC
Zlib
Unicode-3.0
```

Any copyleft, source-available, custom, unknown, model, dataset, binary, font, or container-image license requires an explicit recorded compatibility decision before merge.

The automated license gate is engineering screening, not legal advice.

---

## 21. Reuse decision record

Every implementation ticket includes:

```yaml
reuse_assessment:
  existing_focusa_owner:
  existing_uiai_owner:
  existing_pi_owner:
  decided_framework:
  framework_version_ref:
  license:
  notice_required:
  conformance_fixture:
  integration_mode: adopt | wrap | configure | extend | custom
  custom_code_justification:
```

The allowed implementation order is:

```text
Adopt
→ Wrap
→ Configure
→ Extend
→ Custom only when the conformance fixture proves no decided primitive satisfies the requirement
```

A custom implementation task without a completed reuse assessment is blocked.

---

# Part IV — License, Dependency, and Replacement Governance

## 22. Third-party provenance ledger

Maintain one generated and reviewable ledger containing:

```yaml
component:
version:
source_repository:
package_or_image:
license_expression:
license_file_hash:
notice_text_ref:
code_model_dataset_or_binary:
embedded_or_external:
distribution_surfaces: []
security_advisory_source:
owner:
replacement_boundary:
last_reviewed_at:
```

Code licenses and model/dataset licenses are tracked separately.

---

## 23. Notices and redistribution

Release and installer workflows must produce:

- third-party notices;
- Rust dependency license report;
- JavaScript dependency license report;
- Python/Docling dependency and model report;
- container-image provenance;
- CycloneDX or SPDX-compatible SBOM;
- hashes for vendored skills and notices.

The adapted Matt Pocock skill package must retain its MIT notice.

---

## 24. Dependency replacement law

Every high-leverage dependency must remain behind a Focusa-owned interface or generated contract.

Examples:

```text
Docling
→ DocumentExtractionAdapter

sqlite-vec
→ VectorIndexAdapter

fastembed-rs
→ EmbeddingProfileAdapter

TanStack Query
→ focusa-client query-key and invalidation layer

Svelte Flow
→ graph projection contract

provider SDK
→ ProjectContextSourceAdapter / WorkItemAdapter
```

If a dependency becomes abandoned, insecure, incompatible, or relicensed:

```text
freeze known-good version
→ open a blocker with evidence
→ select the replacement through a versioned amendment
→ implement compatibility adapter/migration
→ preserve canonical Focusa contracts
```

Implementation agents may not replace a decided framework informally.

---

# Part V — Decomposition and Agent Instructions

## 25. Mandatory decomposition method

The Spec 120 decomposer must produce:

1. the complete Feature Ledger and closure DAG;
2. the Cross-Functional Alpha ticket chain;
3. parallel expansion frontiers;
4. explicit blocking edges;
5. reuse assessments;
6. generated-contract tasks;
7. license/SBOM tasks;
8. actual proof tasks.

Tickets must be vertical tracer bullets wherever possible.

A vertical ticket:

- cuts through every necessary layer;
- is demoable or independently verifiable;
- has explicit blockers;
- fits one fresh agent context;
- leaves the repository green or belongs to a declared expand-contract migration chain.

---

## 26. Mandatory decomposer prompt

The following instruction is normative and must be supplied verbatim or equivalently to every agent decomposing the Spec 135 series:

```text
Implement the complete Spec 135 series for maximum speed without reducing its
scope. Do not present framework, sequence, or product-option menus. The operator
has already decided the stack and implementation strategy in Specs 135, 135D,
and 135H.

Create two linked plans:

1. the complete closure DAG containing every accepted requirement; and
2. the Cross-Functional Alpha, a narrow real path through contracts, Context,
   Role, Grill Interview, Spec Workbench, Tasks, Workpoint, Evidence, Receipt,
   UIAI artifact refresh, multiplexed Mission Canvas, and vertical projection.

Build the Cross-Functional Alpha first while preserving every remaining
requirement in the closure DAG. As soon as a shared seam is stable, open all
unblocked connector, provider, vertical, client, renderer, migration, security,
accessibility, performance, and proof tasks as parallel frontiers.

Use vertical tracer-bullet tickets. Each ticket must cross the necessary schema,
reducer/persistence, API, generated client, UI, real integration, test, and
evidence layers. Do not claim a slice from a horizontal backend or static UI
stub alone.

Use the decided frameworks and ownership boundaries. Do not ask the operator to
choose alternatives. Do not introduce a second runtime, spec engine, interview
store, browser engine, task authority, evidence store, session model, vector
database, document parser, UI state system, or desktop shell.

Every ticket must include a reuse assessment. Adopt, wrap, configure, or extend
existing Focusa, UIAI Engine, Pi, or the selected open-source framework before
custom-building. Custom code requires a failing conformance fixture and an
explicit justification.

Generate OpenAPI, JSON Schema, and TypeScript contracts from the Rust source of
truth before independent client lanes diverge. Manual duplicate DTOs are
forbidden when generation can represent the contract.

Use focusa.interview.strategy.grill-with-docs.v1 for the initial C.R.I.S.T.
Interview. Retrieve discoverable facts instead of asking the operator, ask one
decision question at a time, provide a recommended answer with sources, persist
every answer, and create governed glossary/ADR candidates rather than writing
repository documents directly.

Keep the Spec 135 dogfood path continuously green. A change that breaks the
end-to-end Context → Role → Interview → Spec → Tasks → Workpoint → Evidence →
Receipt → UIAI artifact → Mission Canvas path does not merge.

Use expand-contract migration for wide changes such as Pi namespace convergence,
schema envelopes, renamed types, and compatibility projections. Add the new
form beside the old, migrate consumers in bounded batches, then remove the old
form only after all consumers and proofs pass.

Do not use “later,” “future enhancement,” “post-MVP,” “nice to have,” or “out
of scope for now” to remove a requirement. A blocked requirement remains open
and blocks parent closure unless the operator approves a versioned amendment.
```

---

## 27. Parallel implementation lanes

After Alpha 0 contracts stabilize, start these lanes concurrently:

```text
Lane A — Rust contracts, reducers, read models, migrations
Lane B — Mission Canvas, design system, generated client
Lane C — Context, Docling, hybrid retrieval, Google Drive
Lane D — Role, Grill Interview, Project Genesis UI
Lane E — Spec 120 integration, decomposition, Beads, Receipts
Lane F — UIAI artifacts, browser contexts, FPV, SSE refresh
Lane G — provider and connector expansion
Lane H — domain packs, verticals, artifact renderers
Lane I — tests, security, licenses, SBOM, performance, accessibility
```

Agents use scoped worktrees, writer leases, explicit Workpoints, and Spec 135G Attachments. Parallelism must not become shared dirty-worktree mutation.

---

## 28. Merge laws

A change may merge only when:

- its generated contracts are current;
- its reuse assessment is complete;
- its focused tests pass;
- affected static/schema gates pass;
- actual integration evidence exists where the ticket claims integration;
- the Spec 135 dogfood path remains green;
- dependency/license gates pass;
- no accepted requirement is silently removed;
- no authority-bearing singleton is introduced.

The full suite runs at defined convergence points and before closure/release rather than after every isolated documentation edit.

---

## 29. Speed anti-patterns

The decomposer must reject tasks that introduce:

- LangChain or LlamaIndex as a second orchestration/authority runtime;
- Temporal, Airflow, Celery, or another durable session/workflow authority;
- an external vector database without approved benchmark evidence;
- XState or client stores as canonical backend state;
- a generic survey platform as the Interview engine;
- another Tauri/desktop shell;
- another task, Evidence, Receipt, session, or context store;
- provider SDK types in Focusa core;
- custom document parsing where Docling supports the format;
- custom generic UI primitives already supplied by the selected Svelte stack;
- custom graph parsing/search engines where petgraph, Tree-sitter, or ast-grep applies.

---

## 30. Acceptance criteria

Spec 135H is accepted when:

1. `focusa.interview.strategy.grill-with-docs.v1` is implemented as the initial Interview strategy.
2. The MIT notice for adapted Matt Pocock skills is preserved.
3. Interview facts are retrieved before questions are asked.
4. Every operator decision question includes a recommendation and basis.
5. Questions are asked one at a time and persisted canonically.
6. Glossary and ADR outputs are governed candidates rather than direct writes.
7. The Cross-Functional Alpha exists as a real ticket chain.
8. Alpha 0–8 pass actual end-to-end proof.
9. Every remaining requirement remains in the complete closure DAG.
10. Expansion frontiers begin as soon as their shared seams stabilize.
11. Rust/OpenAPI/JSON Schema/TypeScript generation prevents client contract drift.
12. The selected document, retrieval, UI, graph, connector, renderer, test, and compliance stacks are used through Focusa-owned interfaces.
13. Every implementation ticket contains a reuse assessment.
14. The Spec 135 dogfood path is a permanent merge/release gate.
15. License, notice, model, container, and SBOM reports are generated.
16. Wide migrations use expand-contract sequencing.
17. Decomposition contains no operator option menus for decisions already made.
18. No speed optimization creates duplicate canonical authority or defers accepted scope.

---

## 31. Closure blockers

This specification cannot close while:

- Interview is implemented as a static form only;
- Grill behavior writes glossary/ADR files without Focusa approval;
- decomposition is horizontal-only;
- the first usable end-to-end path waits for every horizontal subsystem to finish;
- an Alpha slice uses mock-only integration;
- generated server/client contracts can drift;
- a custom implementation bypasses an applicable selected framework without a failing conformance fixture;
- dependency or model licenses are unknown;
- notices or SBOM are missing;
- implementation agents ask the operator to reselect decided frameworks;
- the Spec 135 dogfood path is not continuously exercised;
- remaining requirements disappear from the Complete Feature Ledger;
- parallel agents share authority-bearing singleton or dirty-writer state.
