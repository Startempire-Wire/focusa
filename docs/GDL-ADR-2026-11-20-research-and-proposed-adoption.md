# GDL-ADR-2026-11-20 — ADR GitHub Research & Proposed Adoption Addendum

**Status:** RESEARCH / PROPOSED ENHANCEMENTS / NOT OPERATOR-APPROVED  
**Related draft:** [GDL-001](GDL-001-global-decision-ledger-contract.md) and [GDL-002](GDL-002-continuous-decision-mining-reconciliation-daemon-spec.md)  
**Source:** https://adr.github.io/ (reviewed 2026-11-20)  
**Scope discipline:** This document is **not** an amendment to the approved scope or an activation instruction. It identifies already-covered parts and distinctly labeled optional/additive proposals. Original GDL-001/002 remains unchanged until operator approval of exact changes.

## 1. Finding

The ADR GitHub organization is a **community knowledge and tooling hub**, not a mandatory authoritative standard or a reason to migrate storage to Markdown. Its homepage locates ADRs in **Architectural Knowledge Management (AKM)**: each record preserves one decision, its rationale, consequences and trade-offs. It explicitly discusses extension beyond architecture to any decision.

**Recommended outcome:** Keep the Focusa daemon as the sole canonical store, approval and real-time resolver. Borrow ADR/MADR/GADR's well-established **explanation, comparison, history, reuse and verification** concepts as typed metadata and optional projections. No additional privileged service, Markdown authority, forced ADR writing or unrelated scope expansion.

## 2. Prior art reconciliation: what we already cover

| ADR-origin idea | GDL draft already covers | Do not duplicate |
| --- | --- | --- |
| Single distinct choice | GDL-001 atomic <=160-char statement | Second ADR-shaped decision store |
| Status and history | GDL-001 immutable revision, explicit supersession, operator approval | Editable Markdown status as authority |
| Source and rationale | GDL-002 exact original spans, attribution, provenance and adversarial reconciliation | Unverified model-generated justifications |
| Cross-project history | GDL-002 project alias/rename/split/retire lineage | Automatically importing old decisions as current policy |
| API/search | Focusa-native resolver, FTS5 projection, MCP/CLI/REST/SSE | New external search daemon and broker |
| ADR export | GDL-001 §8 optional Markdown derived view | Mandatory per-decision repository files |
| Review gates | GDL-002 G0–G10, Focusa Spec120, authority constitution | A separate approval/governance subsystem |

## 3. Improvements worth proposing (require explicit approval before normative adoption)

### A. Generalized Architectural Decision Guidance (GADR): portable *knowledge*, not portable authority — HIGH VALUE

The GADR project distinguishes **problem space** (reusable decision needs, patterns, options, evaluation criteria) from **solution space** (particular choice for an individual project). It explicitly differentiates **Decisions Required** from **Decisions Made**.

**Proposal:** Add three typed **advisory** cognitive objects alongside GDL, implemented within existing Focusa semantic/ontology boundaries, not as new independent stores:

- `DecisionQuestion`: the unresolved choice that a scoped project needs to make; required outcome, affected qualities, known constraints, best latest responsible moment, source refs; no active governing decision implied.
- `DecisionPattern`: generalized problem, possible options with conditional pros/cons, applicability assumptions, sources/projects where observed and evidence quality. It is **guidance only**, not authority.
- `DecisionInstanceLink`: relation from an authorized project's decided GDL record to the generalized problem/pattern and source examples, while preserving its separate scope, actor, effective interval and approval.

One pattern may explain many project decisions, even contradictory ones valid in different circumstances. Reusing a pattern **never** means automatically reusing or activating a project-specific answer. Do not use inaccessible third-party/private decisions for cross-tenant suggestions; only approved shareable abstractions can cross scopes.

*Benefit:* When creating a new product or reviving an old one, the daemon can say, "We solved this kind of decision in these projects; here were the drivers/trade-offs" and prepare one source-grounded recommendation rather than begin from a blank page.

*Impact to scope:* Adds advisory typed problem/pattern graph and retrieval view. No new approval permission; existing GDL DecisionRecord remains sole governing object.

### B. MADR 4.0 format: canonical structured enrichment + optional projections — LOW/MEDIUM VALUE, HIGH REUSE

MADR 4.0.0 has **minimal** and **full** templates. The full version includes problem/context, drivers, alternatives, decision outcome, positive/negative/neutral trade-offs, confirmation and related material. It also records status/date/decision-makers/consulted/informed as optional YAML front matter.

**Proposal:** Add *optional refs* (not required per trivial decision) to a versioned `decision_explanation` metadata projection:

```json
{
  "schema": "focusa.gdl.adr_explanation.proposal.v1",
  "decision_ref": "gdl-record-revision",
  "profile": "minimal|full",
  "problem_ref": "evidence-or-question-ref",
  "context_refs": ["source-span-ref"],
  "decision_driver_refs": [],
  "considered_options": [],
  "tradeoffs": {
    "positive_refs": [],
    "negative_refs": [],
    "neutral_refs": [],
    "unknown_refs": []
  },
  "rationale_refs": ["source-span-ref"],
  "consulted_actor_refs": [],
  "informed_actor_refs": [],
  "confirmation_plan_ref": null,
  "implementation_evidence_refs": [],
  "revisit_trigger_refs": [],
  "source_template": "MADR-4.0.0"
}
```

This is an **illustrative candidate**, not a new runtime or implemented schema. Each inferred value must carry `observed|quoted|extracted|inferred|approved` provenance, not be collapsed into a false first-person historical claim. If the source contains no alternative, say "not recorded"; **never fabricate rejected options** to fill a template.

**Views:** Keep the short GDL card; optional "why" panel; optional Y-statement; optional MADR-minimal or MADR-full Markdown export with source-linked YAML front matter. Record `gdl_decision_id`, pinned revision, source digest, generated-at, and explicit derivative status. Repo export requires existing repo rights and a separate operator-approved write.

*Impact to scope:* New optional metadata fields and serializer profiles, not new decision authority.

### C. Y-Statements for compact rationale — HIGH VALUE, VERY SMALL ADDITION

The ADR site uses a concise structure: "In context X, facing concern Y, choose Z to achieve quality Q, accepting downside R." It makes the **trade-off** explicit, not just the action.

**Proposal:** Render a source-backed `decision_why` field, perhaps one line, assembled from typed context, concern, choice, desired property and accepted downside refs. The **canonical statement remains <=160 chars** and need not contain the entire Y-statement; if evidence is missing, show unknown instead of fabricating the downside.

*Impact to scope:* Optional human/agent explanatory projection; no new mandatory questions.

### D. Readiness / completion checks: avoid premature or unverified choices — HIGH VALUE

The site's practices point to Olaf Zimmermann's **START** decision-readiness criteria (stakeholders, timing/most responsible moment, alternatives, requirements/context, template) and **ecADR** decision-completion criteria (evidence, compared criteria, agreement, documentation and realization/review plan).

**Proposal:** Reuse existing Focusa/Spec120 adversarial review gates as *derived quality diagnostics*:
- `decision_question_ready` — enough evidence/context to ask operator for a choice? If not, continue bounded discovery.
- `decision_record_valid` — operator's approved text, authority and scope accepted? Only this can promote GDL.
- `decision_realization_planned` — owner/target/time/verification/revisit policy linked? This is follow-through, **not** operator approval.
- `decision_implementation_confirmed` — actual evidence confirms delivery and observed behavior; separate from acceptance.

Avoid a formal approval ceremony for low-risk decisions; check what's relevant according to materiality. **A decision may be validly approved yet not implemented or verified.** Do not mark deployment successful merely because an ADR/PR was accepted. Unknowns stay visible.

*Impact to scope:* More precise quality and realization statuses; reuse existing Focusa evidence/Workpoint/verification, not extra activity tracking.

### E. Materiality-based ADR escalation — MEDIUM VALUE

MADR's significance discussion and the ADR practice pages support focusing on decisions with meaningful downstream cost, time, irreversibility, dependency chains or system-quality impact.

**Proposal:** A `materiality_profile` flags architecture/significant/normal with evidence-supported drivers; default to GDL-only; offer an ADR-full view if the choice alters security/trust boundaries, ownership, persistence/consistency, public API compatibility, cross-product topology, licensing/commercial constraints or other costly-to-reverse contracts. This flag recommends review depth; it must not auto-promote.

*Impact to scope:* Extra ranking/optional explanation, **not** mandatory PRs or artificial process.

### F. Search, linking and code proximity — MEDIUM VALUE

MADR links other ADRs; adr.github.io lists Backstage ADR plugin for searches across multiple repositories and embedded ADRs for linking source code to decision records.

**Proposal:** Build on Focusa's existing typed links + FTS5; add optional `governing_decision_refs` in code module/manifests, architecture contract, PR review, generated agent context and release proof. Add a human-readable decision index derived from GDL (not a second source of truth). This enables agent questions like "Which decisions govern this module?" without parsing every Markdown file.

*Impact to scope:* Optional code/ADR indexing and consumer links, not a new portal installation. The Backstage ADR plugin is an **integration pattern**, not a recommended new dependency.

## 4. Addendum to historical mining behavior

When mining historical ADRs, issues, README architecture sections, PR design debates and operator chats, distinguish:

1. `decision_required`: an unresolved question/request or problem, no governing choice.
2. `options_discussed`: alternatives and trade-offs, no acceptance.
3. `decision_made_source_claim`: source text claims a choice was made; **not yet an authenticated GDL approval**.
4. `decision_retracted_or_superseded`: old choice replaced/rejected/retired with original lineage.
5. `decision_realized`: external code/deployment evidence supports implementation but cannot mint prior approval.
6. `decision_pattern`: reusable guidance extracted across projects, with cross-scope permissions intact.

*Example:* historical "Use Redis on Project A" and "Use SQLite on Project B" may both be valid. The miner learns a reusable datastore-selection **problem** and options/criteria, not a contradictory universal policy. Likewise, "ADR-0001 Accepted" from an archived repository remains an **attributed historical status**, not permission for Focusa to treat it as a newly signed authoritative rule.

## 5. Decision-complete adoption path: one recommended implementation route

1. **Do not change approved/active GDL requirements until this addendum is explicitly reviewed.** No automatic scope promotion from this research.
2. If approved, first implement cheap *derived metadata* (MADR minimal fields, Y-statement view and confirmed versus implemented states) using the same Focusa Decision object.
3. Then add a lightweight `DecisionQuestion` + optional cross-project `DecisionPattern` advisory ontology mapping, using already-ingested and approved source spans.
4. Lastly add ADR serializer/importers and optional code-near decision references. Use no dedicated database, service, scheduling loop or new UI workflow.
5. Measure benefits: recovered decisions with verified source; accurate unresolved/decided distinction; correct cross-project suggestions without unauthorized inheritance; zero fabricated options; verified explanation consistency; retrieval parity; unchanged <1s interactive budget; no duplicate operator questioning.

**Proposed acceptance fixtures:**
- Two projects choose different valid storage systems; do not produce a global conflict or override.
- Operator asks "what still needs deciding?" and sees `DecisionQuestion`, not already-active GDL entries.
- Mining assistant's alleged ADR "accepted" from historical project remains advisory until verified auth.
- A candidate with no documented downside yields "not recorded", no plausible-sounding invented trade-off.
- The Y-statement and MADR-minimal/full views reference exactly one canonical decision ID and revision.
- Consulted/informed names never count as operator authority.
- Source code links and Markdown export update after supersession without rewriting historic approved records.
- Decision accepted but not built is explicitly `approved_not_verified` (status projection), no false delivery receipt.
- One source becomes revoked: derived context and exposed snippets respect privacy and ACL change.

## 6. Direct sources reviewed

- ADR GitHub homepage (mission, AKM, any-decision view): https://adr.github.io/
- ADR templates incl. Nygard, Y-statements, MADR: https://adr.github.io/adr-templates/
- MADR 4.0.0 overview, minimal/full, large-project practices: https://adr.github.io/madr/
- MADR full template and confirmation: https://adr.github.io/madr/decisions/adr-template.html
- MADR project's "consulted/informed" RACI decision: https://adr.github.io/madr/decisions/0015-include-consulting-informed-of-raci.html
- Generalized Architectural Decision Records problem/solution space: https://adr.github.io/gadr/
- ADR decision-making practices: https://adr.github.io/ad-practices/
- ADR tooling catalog including Backstage/Log4brains: https://adr.github.io/adr-tooling/
- Zimmermann START readiness: https://www.ozimmer.ch/practices/2023/12/01/ADDefinitionOfReady.html
- Zimmermann ecADR completion: https://ozimmer.ch/practices/2020/05/22/ADDefinitionOfDone.html

## 7. Operator disposition requested

**Recommended:** approve the *principles* of (A) reusable advisory decision-problem/pattern graph, (B) optional MADR metadata, (C) compact Y rationale and (D) approval-versus-implementation separation, then reconcile these against GDL-001/002 and existing Focusa code prior to writing new code.

**Not recommended:** importing an external ADR app as canonical, copying full templates into every 160-character decision, auto-accepting historical ADR statuses, mandatory alternatives for past choices that never considered them, duplicating a search engine or requiring manual ADR files for every small choice.

This research addendum by itself authorizes **no expansion, deletion, deferral, code changes, source reads, new privileges or deployment**.
