# Spec120 — Adversarial Spec Workbench and Operator Approval Gates

**Status:** proposed / implementation-ready specification  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-07  
**Scope:** Focusa core, daemon API, CLI, Pi extension, UIAI Engine integration, local PWA, future Tauri desktop shell, menubar launch/status surface, repo writer, export adapters, task manager adapters, Workpoint, Context Cognition, Evidence, Receipts, work-item closure authority, and proof gates.  
**Authority:** This spec defines a governed specification-authoring workflow. It does **not** create a new canonical cognition authority, does **not** bypass Workpoint, does **not** replace Context Cognition, and does **not** replace provider-neutral work-item closure authority.

---

## 0. One-line definition

Focusa should turn rough operator ideas into approved, evidence-grounded, section-composed, exportable, task-decomposed specs through bounded adversarial cognition and explicit operator gates.

Short name:

```text
Adversarial Spec Workbench
```

Product description:

```text
A Focusa-governed spec authoring system where ideas are grounded in current documentation and codebase reality, researched through UIAI/Context Cognition, challenged by adversarial agents, approved by the operator section by section, exported/shareable, committed back into the repo, and decomposed into executable tasks.
```

---

## 1. Normative basis

This spec is intentionally built as the last major manual spec before this feature exists.

It extends and preserves the following Focusa specs and current documents.

| Source | This spec uses it for |
|---|---|
| `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md` | Requires the lifecycle `Idea → New Spec → bd/task decomposition → Implementation → tests/proofs → bd/task closure`; blocks implementation before the spec and decomposition gates exist. |
| `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md` | Requires actual/partial/surrogate/blocked/missing evidence classification and blocks closure/final reports when evidence is insufficient. |
| `docs/116-provider-neutral-work-item-closure-authority-spec.md` | Preserves the rule: Focusa validates closure truth; providers store and display closure. |
| `docs/116-provider-neutral-work-item-closure-authority-spec.md` | Uses `WorkItem`, `WorkItemRef`, `WorkItemProvider`, `ClosureClaim`, `ClosurePolicy`, `ClosureValidationResult`, `ProviderAdapter`, and `ProviderCapabilities` so this feature does not lock itself to bd. |
| `docs/116-provider-neutral-work-item-closure-authority-spec.md` | Aligns spec-to-task decomposition and provider task closure with `prepare → validate → authorize → submit → reconcile → audit`. |
| `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md` | Supplies the bounded loop model: observe, propose, verify/critique, evaluate, promote/reject/archive, checkpoint, recover, decay, repeat. |
| `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md` | Reuses adversarial falsification, explicit objections, and fail-closed verifier availability for spec-section approval. |
| `docs/100-context-cognition-spec.md` | Uses Context Cognition as advisory project/context curation; Workpoint remains action authority, Evidence remains proof boundary, and packet generation cannot mutate canonical state. |
| `docs/100-context-cognition-spec.md` | Treats UIAI/browser diagnostics as proposal material until captured/linked; uses UIAI status mapping: `proposal_only`, `capture_pending`, `captured`, `linked`, `scope_mismatch`, `stale`. |
| `docs/88-ontology-backed-workpoint-continuity.md` | Preserves the principle that Focusa should preserve typed Workpoint state, not transcript tail. |
| `docs/90-ontology-backed-tool-contracts-parity-spec.md` | Requires new tools to have canonical machine-readable contracts linking tools to ontology actions, API routes, CLI parity, docs, result envelopes, and health checks. |
| `docs/111-agent-context-bootstrap-and-delivery-spec.md` | Reuses the idea that Focusa can prove what project, mission, Workpoint, next action, evidence refs, and drift boundaries an agent received before acting. |
| `docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md` | Requires final spec approval, exports, repo writes, decomposition, and closure readiness to become receipt-producing work where appropriate. |
| `docs/current/FOCUSA_MENUBAR_AUTHORITY_STATE_CONTRACT.md` | Menubar/local UI may display and call scoped routes, but it must never mint canonical authority; canonicality comes from reducer-backed API/Workpoint envelopes. |
| `docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md` | Exports/cloud projections must preserve the rule: cloud coordinates, node decides, receipts prove, private state stays local. |

Any mismatch between this spec and current implementation is an implementation gap, not a reason to weaken this spec.

---

## 2. Purpose

Spec120 creates a first-class Focusa surface for turning ideas into implementation-ready specs.

It exists because manual spec writing currently depends too much on:

- the operator remembering what to include;
- the assistant remembering prior context;
- loose conversation rather than typed gates;
- agents making confident claims without current repo/doc proof;
- coding agents beginning implementation before adversarial specification review;
- final specs being written as prose but not converted into tasks, receipts, exports, or closure policy.

Spec120 makes specification itself a governed workflow.

---

## 3. Core thesis

A strong Focusa spec should not be written in one pass.

It should be assembled section by section through:

```text
current docs/code reality
  + UIAI/resource research
  + proposer argument
  + adversarial challenge
  + reference audit
  + operator approval
  + final reconciliation
  + task decomposition
  + proof/receipt trail
```

The spec workbench is not an autonomous author.

It is a bounded adversarial drafting and approval system where the operator remains the authority.

---

## 4. Product promise

Before an AI coding agent implements a new Focusa feature, Focusa can prove:

```text
1. What idea the operator introduced.
2. Which current docs and code surfaces were consulted.
3. Which research resources informed the spec.
4. Which claims are implemented, partial, docs-only, planned, speculative, stale, or blocked.
5. Which objections were raised by adversarial agents.
6. Which objections were resolved, deferred, or rejected.
7. Which sections the operator approved.
8. Which final spec version was approved.
9. Which task/provider items were created from the spec.
10. Which exports were generated and where they went.
11. Which receipt proves the spec approval/decomposition/export result.
```

---

## 5. Non-goals

Spec120 is not:

- a replacement for Workpoint;
- a replacement for Context Cognition;
- a replacement for Evidence;
- a replacement for Spec116 work-item closure authority;
- a replacement for bd, Linear, Asana, GitHub Issues, or other task managers;
- a chat app whose transcript is authority;
- a generic LLM debate toy;
- a source of canonical truth outside reducer/governance paths;
- a way for agents to self-approve specs;
- a way for repeated argument loops to bypass the operator;
- an export system that publishes private repo/context by default;
- a cloud memory system;
- a Tauri-first product claim.

---

## 6. Hard design laws

### 6.1 Operator authority wins

The operator controls:

- initial idea;
- section list;
- loop mode;
- number of argument rounds;
- infinite/repeat-until-stopped behavior;
- approval/rejection;
- final spec approval;
- repo write approval;
- export approval;
- task decomposition approval;
- implementation-start approval.

Agent outputs remain advisory until operator-approved and promoted through the correct Focusa path.

### 6.2 Reality grounding before persuasion

No spec claim may advance unless grounded against current documentation, current codebase reality, captured research, or explicitly marked as:

```text
unverified | planned | speculative | docs_only | blocked | stale
```

### 6.3 Section-level approval, whole-spec reconciliation

Each section is approved independently.

Final approval still requires whole-document reconciliation to catch contradictions across approved sections.

### 6.4 Infinite argument loops may refine forever, but never promote automatically

The operator may enable repeat/infinite mode like a music player repeat button.

Continuous mode may keep generating new proposal/critique/synthesis rounds, but:

```text
Only operator approval moves a section to Approved.
Only final operator approval promotes the full spec.
Only explicit gates allow repo writes, exports, task creation, or implementation start.
```

### 6.5 Every section needs references

Every spec section must include accurate:

- research references;
- Focusa documentation links;
- codebase references when making implementation/runtime claims;
- evidence refs when making proof/verification claims;
- freshness status for external/current references.

### 6.6 Research is proposal-only until captured

UIAI Engine, web research, browser diagnostics, and external examples are useful, but not authoritative until captured/linked through Focusa evidence or explicitly accepted as advisory research.

### 6.7 Exports are projections

Markdown, PDF, email, Google Drive, Microsoft/OneDrive, public cards, and cloud-hosted spec summaries are projections.

The local Focusa node retains authority.

### 6.8 Task manager writes follow provider-neutral policy

The spec workbench may decompose into task provider items, but it must do so through a provider-neutral intermediate model and compatible provider adapters.

### 6.9 Closure follows Spec116

Created tasks must not be closable as complete unless closure evidence satisfies the provider-neutral closure authority.

### 6.10 Receipts prove the workflow

Final approval, repo write, task decomposition, export, and implementation-start authorization should produce Focusa Receipts or receipt projections.

---

## 7. Workflow overview

```text
Operator Idea
  → SpecWorkbenchSession created
  → Reality Scanner builds current docs/code reality pack
  → UIAI / Context Cognition builds initial research/resource pack
  → Proposed section outline generated
  → Operator accepts/edits section outline
  → Each section enters its own loop:
      research
      proposer draft
      challenger critique
      reference audit
      synthesis
      operator gate
      approved / rejected / repeat
  → Approved sections accumulate
  → Whole-spec reconciliation pass
  → Final spec preview
  → Final operator approval
  → Repo write gate
  → Export/share gates
  → Task decomposition gate
  → Provider-neutral task creation
  → Receipt generation
  → Implementation-start gate
```

---

## 8. UI concept

The primary UI should be an independent local Workbench, not buried inside Pi or the coding agent.

Implementation order:

```text
Phase 1: Local PWA popout served by the Focusa daemon
Phase 2: Tauri desktop shell around the same UI
Phase 3: Menubar quick-launch/status/approval badge
```

The menubar remains a launcher/status surface, not the authority source.

### 8.1 Three primary tabs

```text
Chat
Pending
Approved
```

### 8.2 Chat tab

The Chat tab shows the full argument transcript.

It must display messages by actor:

- operator;
- reality scanner;
- UIAI researcher;
- spec proposer;
- adversarial challenger;
- reference auditor;
- synthesis arbiter;
- system/gate.

It must show:

- round number;
- loop mode;
- current section;
- evidence/docs/code refs;
- unresolved objections;
- stale/missing reference warnings;
- cost/runtime counters when available;
- pause/stop/repeat controls.

### 8.3 Pending tab

The Pending tab shows sections waiting for operator decision.

Each pending card shows:

```text
Section title
Section kind
Current round
Loop mode
Research refs count
Focusa doc refs count
Code refs count
Evidence refs count
Unresolved blocker objections
Unverified claims
Stale refs
Reference audit status
Promotion gate status
```

Actions:

```text
Approve section text only
Approve section for final spec
Reject section
Request more research
Run one more round
Repeat until stopped
Split section
Merge with another section
Pause loop
Stop loop
```

### 8.4 Approved tab

The Approved tab shows locked approved sections.

Each approved card shows:

```text
Approved section text
Approved revision id
Operator gate id
Approval timestamp
Approved by
Evidence refs
Research refs
Docs/code refs
Export status
Repo write status
Task decomposition status
Receipt refs
```

Approved sections can be reopened only through an explicit operator gate.

---

## 9. Operator loop controls

Each section has its own loop policy.

### 9.1 UI controls

```text
Rounds: [ - ] 3 [ + ]

Mode:
○ One pass
● Fixed rounds
○ Repeat until clean
○ Repeat until stopped

Controls:
[▶ Start]
[⏸ Pause]
[⏭ Next round]
[⏹ Stop]
[↻ Repeat]
```

### 9.2 Modes

```rust
pub enum ArgumentLoopMode {
    Off,
    OnePass,
    FixedRounds,
    RepeatUntilClean,
    RepeatUntilStopped,
}
```

### 9.3 Loop policy

```rust
pub struct SpecSectionArgumentLoopPolicy {
    pub mode: ArgumentLoopMode,
    pub requested_rounds: Option<u32>,
    pub max_rounds_guardrail: Option<u32>,
    pub max_runtime_ms: Option<u64>,
    pub max_cost_units: Option<f64>,
    pub stop_on_no_new_evidence: bool,
    pub stop_on_repeated_objections: bool,
    pub stop_on_scope_contamination_risk: bool,
    pub stop_on_low_confidence: bool,
    pub require_operator_stop_for_continuous: bool,
    pub require_research_refs_each_round: bool,
    pub require_repo_doc_links_each_round: bool,
    pub require_code_refs_for_runtime_claims: bool,
    pub argument_similarity_threshold: Option<f64>,
}
```

### 9.4 Stop reasons

```rust
pub enum SpecLoopStopReason {
    OperatorStopped,
    OperatorRejected,
    OperatorApproved,
    NoNewEvidenceDelta,
    RepeatedObjectionSet,
    ScopeContaminationRisk,
    LowConfidence,
    VerifierUnavailable,
    MaxGuardrailReached,
    SectionAccepted,
    SectionSuperseded,
    ResearchStale,
    ExportGatePending,
    FinalApprovalPending,
}
```

---

## 10. Standard final spec section format

Every generated spec should use this standardized section layout unless the operator chooses a different template.

```text
1. Title / one-line definition
2. Problem
3. Current docs + codebase reality
4. Goals
5. Non-goals
6. User/operator workflow
7. Agent workflow
8. Data model
9. API / CLI / UI surfaces
10. Authority and side effects
11. Research/resources
12. Risks and adversarial objections
13. Acceptance criteria
14. Evidence/proof requirements
15. Export/share behavior
16. Repo commit behavior
17. Task decomposition plan
18. Open questions
19. Final approval record
```

Every section must include a grounding block.

```yaml
grounding:
  research_refs:
    - title:
      url_or_artifact_ref:
      source_kind: web | uiai | repo_doc | repo_code | operator | evidence
      status: proposal_only | captured | linked | stale | scope_mismatch
      relevance_reason:
      checked_at:
      source_date:
      stale_after_days:
  focusa_doc_refs:
    - path:
      section_or_lines:
      relevance_reason:
      currentness: current | older_design_intent | normative_target | implementation_gap
  codebase_refs:
    - path:
      symbol_or_route:
      current_runtime_status: implemented | partial | docs_only | unknown | blocked
      relevance_reason:
  evidence_refs:
    - ref:
      evidence_class: actual | partial | surrogate | blocked | missing
  unverified_claims: []
```

---

## 11. Section lifecycle

```text
draft_requested
  → research_running
  → research_ready
  → proposer_drafted
  → challenger_reviewed
  → reference_audited
  → synthesis_ready
  → pending_operator_acceptance
  → approved_section
  → integrated_into_final_spec
```

Rejected path:

```text
pending_operator_acceptance
  → operator_rejected
  → revision_requested
  → research_running | proposer_drafted
```

Reopen path:

```text
approved_section
  → operator_reopen_requested
  → reopen_gate_approved
  → revision_requested
```

---

## 12. Agent roles

### 12.1 Reality Scanner

Purpose:

- ground the idea in current docs and code;
- separate implemented runtime facts from docs-only or planned claims;
- detect stale/older docs;
- produce current code/doc reality pack.

Must emit:

```yaml
reality_pack:
  docs_consulted: []
  code_surfaces_consulted: []
  route_inventory_refs: []
  cli_inventory_refs: []
  type_schema_refs: []
  tests_proof_refs: []
  current_runtime_status:
  implementation_gap_notes: []
  docs_only_claims: []
  blocked_claims: []
```

### 12.2 UIAI Researcher

Purpose:

- use UIAI Engine/browser/search diagnostics to suggest resources and external research;
- capture product examples, technical references, UI patterns, competitor signals, and API/library docs;
- never promote research to authority without capture/linking.

Must emit:

```yaml
research_packet:
  status: proposal_only | captured | linked | stale | scope_mismatch
  queries: []
  findings: []
  suggested_resources: []
  relevance_reasons: []
  freshness:
    checked_at:
    stale_after_days:
```

### 12.3 Spec Proposer

Purpose:

- draft or revise a section;
- transform idea/research/reality pack into structured spec text;
- propose data model, API/CLI/UI behavior, workflow, and acceptance criteria.

Must not:

- ignore unresolved objections;
- claim docs-only behavior is implemented;
- omit references;
- self-approve.

### 12.4 Adversarial Challenger

Purpose:

- attack the section;
- try to falsify claims;
- detect missing evidence, authority issues, UX gaps, contradiction, overclaim, and implementation risk.

Must emit:

```yaml
objections:
  - id:
    severity: blocker | should_fix | watchlist
    kind: scope_gap | unverifiable_acceptance | architecture_risk | authority_violation | missing_evidence | operator_friction | implementation_gap | export_privacy_risk | closure_policy_risk
    claim:
    why_it_matters:
    evidence_refs: []
    required_resolution:
```

### 12.5 Reference Auditor

Purpose:

- verify that every section has accurate research references, Focusa doc links, codebase refs, and evidence classification;
- detect stale sources;
- detect docs-only/runtime confusion;
- block approval when required refs are missing.

Must emit:

```yaml
reference_audit:
  status: passed | blocked | degraded
  missing_research_refs: []
  missing_focusa_doc_refs: []
  missing_code_refs: []
  stale_refs: []
  docs_only_overclaims: []
  invalid_links: []
  promotion_blockers: []
```

### 12.6 Synthesis Arbiter

Purpose:

- merge proposer + challenger + auditor outputs into a candidate section revision;
- preserve objections and resolution history;
- never become a sovereign third opinion source.

The arbiter may synthesize, but promotion remains gate-driven.

---

## 13. Promotion gates

### 13.1 Section promotion gate

A section cannot move to `approved_section` unless:

```text
1. Operator approval exists for this exact section revision.
2. Required research refs are present.
3. Focusa doc refs are present.
4. Codebase refs are present when implementation/runtime claims exist.
5. Evidence refs are present when proof/completion claims exist.
6. No unresolved blocker objections remain.
7. Docs-only/runtime distinction is stated.
8. Stale critical research is resolved or operator-overridden.
9. Reference audit status is passed or explicitly overridden.
10. Cross-section contradiction risk is not currently blocking.
```

### 13.2 Whole-spec final promotion gate

The final spec cannot be approved unless:

```text
1. All required sections are approved.
2. Whole-spec reconciliation passed.
3. Contradictions are resolved or explicitly accepted.
4. Terms are normalized.
5. Acceptance criteria are coherent.
6. Export/privacy warnings are resolved.
7. Task decomposition plan exists.
8. Provider-neutral task mapping exists.
9. Final operator approval exists.
10. Final receipt preview is generated.
```

### 13.3 Repo write gate

Writing the spec into the repo requires a separate operator gate.

Approval types are distinct:

```text
approve_section_text_only
approve_section_for_final_spec
approve_final_spec
approve_repo_write
approve_export
approve_task_decomposition
approve_implementation_start
```

One approval must not imply another.

---

## 14. Cross-section reconciliation

After all required sections are approved, Focusa runs a whole-spec reconciliation pass.

It checks:

```text
- contradiction across approved sections
- duplicate requirements
- terminology drift
- authority model inconsistency
- proof/evidence mismatch
- task decomposition mismatch
- export/share conflict
- provider adapter mismatch
- UI vs API vs CLI parity mismatch
- claims that became stale during long drafting
```

Output:

```yaml
whole_spec_reconciliation:
  status: passed | blocked | degraded
  contradictions: []
  duplicate_requirements: []
  terminology_updates: []
  unresolved_policy_conflicts: []
  stale_sections: []
  required_operator_decisions: []
```

---

## 15. Typed model

### 15.1 SpecWorkbenchSession

```rust
pub struct SpecWorkbenchSession {
    pub session_id: Uuid,
    pub schema_version: String,
    pub project_root: String,
    pub continuity_id: Option<String>,
    pub current_ask: String,
    pub status: SpecWorkbenchStatus,
    pub authority: SpecWorkbenchAuthority,
    pub sections: Vec<SpecSectionRecord>,
    pub research_packets: Vec<SpecResearchPacket>,
    pub reality_packs: Vec<SpecRealityPack>,
    pub transcript_refs: Vec<String>,
    pub final_spec_id: Option<Uuid>,
    pub receipt_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 15.2 SpecWorkbenchAuthority

```rust
pub struct SpecWorkbenchAuthority {
    pub canonical: bool,              // false by default
    pub advisory: bool,               // true by default
    pub operator_required: bool,
    pub reducer_mutation_allowed: bool,
    pub workpoint_authority: Option<String>,
    pub evidence_boundary: Vec<String>,
    pub approval_required_for: Vec<OperatorGateKind>,
}
```

### 15.3 SpecSectionRecord

```rust
pub struct SpecSectionRecord {
    pub section_id: Uuid,
    pub title: String,
    pub section_kind: SpecSectionKind,
    pub status: SpecSectionStatus,
    pub order_index: u32,
    pub loop_policy: SpecSectionArgumentLoopPolicy,
    pub current_round_id: Option<Uuid>,
    pub approved_revision_id: Option<Uuid>,
    pub grounding: SpecGroundingBlock,
    pub unresolved_objection_ids: Vec<Uuid>,
    pub operator_gate_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 15.4 SpecRoundRecord

```rust
pub struct SpecRoundRecord {
    pub round_id: Uuid,
    pub section_id: Uuid,
    pub round_index: u32,
    pub round_kind: SpecRoundKind,
    pub proposer_output_ref: Option<String>,
    pub challenger_output_ref: Option<String>,
    pub auditor_output_ref: Option<String>,
    pub synthesis_output_ref: Option<String>,
    pub transcript_ref: String,
    pub verdict: SpecRoundVerdict,
    pub stop_reason: Option<SpecLoopStopReason>,
    pub created_at: DateTime<Utc>,
}
```

### 15.5 SpecArgumentRecord

```rust
pub struct SpecArgumentRecord {
    pub argument_id: Uuid,
    pub round_id: Uuid,
    pub actor_role: SpecActorRole,
    pub claim: String,
    pub reasoning_summary: String,
    pub evidence_refs: Vec<String>,
    pub research_refs: Vec<String>,
    pub focusa_doc_refs: Vec<String>,
    pub codebase_refs: Vec<String>,
    pub confidence: f64,
    pub verification_status: VerificationStatus,
    pub created_at: DateTime<Utc>,
}
```

### 15.6 SpecOperatorGate

```rust
pub struct SpecOperatorGate {
    pub gate_id: Uuid,
    pub session_id: Uuid,
    pub section_id: Option<Uuid>,
    pub gate_kind: OperatorGateKind,
    pub status: OperatorGateStatus,
    pub approval_scope: ApprovalScope,
    pub rejection_reason: Option<String>,
    pub approved_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub evidence_refs: Vec<String>,
}
```

### 15.7 SpecResearchPacket

```rust
pub struct SpecResearchPacket {
    pub packet_id: Uuid,
    pub session_id: Uuid,
    pub section_id: Option<Uuid>,
    pub source: ResearchSource,
    pub status: ResearchStatus,
    pub query: String,
    pub findings: Vec<ResearchFinding>,
    pub suggested_resources: Vec<ResourceSuggestion>,
    pub evidence_refs: Vec<String>,
    pub freshness: ResearchFreshness,
    pub created_at: DateTime<Utc>,
}
```

### 15.8 SpecRealityPack

```rust
pub struct SpecRealityPack {
    pub pack_id: Uuid,
    pub session_id: Uuid,
    pub section_id: Option<Uuid>,
    pub docs_consulted: Vec<DocRealityRef>,
    pub code_surfaces_consulted: Vec<CodeRealityRef>,
    pub route_inventory_refs: Vec<String>,
    pub cli_inventory_refs: Vec<String>,
    pub type_schema_refs: Vec<String>,
    pub tests_proof_refs: Vec<String>,
    pub implementation_gap_notes: Vec<String>,
    pub docs_only_claims: Vec<String>,
    pub blocked_claims: Vec<String>,
    pub generated_at: DateTime<Utc>,
}
```

### 15.9 SpecExportArtifact

```rust
pub struct SpecExportArtifact {
    pub export_id: Uuid,
    pub spec_id: Uuid,
    pub export_kind: SpecExportKind,
    pub projection_status: ProjectionStatus,
    pub redaction_status: RedactionStatus,
    pub destination_ref: Option<String>,
    pub artifact_ref: Option<String>,
    pub operator_gate_id: Option<Uuid>,
    pub receipt_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

### 15.10 Provider-neutral task decomposition

```rust
pub enum TaskProvider {
    Beads,
    GitHubIssues,
    Linear,
    Asana,
    MarkdownChecklist,
    Unknown,
}

pub struct SpecTaskDecompositionPlan {
    pub plan_id: Uuid,
    pub spec_id: Uuid,
    pub provider: TaskProvider,
    pub parent_work_item: ProviderNeutralWorkItemDraft,
    pub child_work_items: Vec<ProviderNeutralWorkItemDraft>,
    pub spec_linked_task_packets: Vec<SpecLinkedTaskPacket>,
    pub operator_gate_id: Option<Uuid>,
    pub receipt_ref: Option<String>,
}
```

---

## 16. Grounding and reference policy

### 16.1 Research freshness

Every research reference must include:

```yaml
research_freshness:
  checked_at:
  source_date:
  stale_after_days:
  requires_recheck_before_final_approval: true
  recheck_status: fresh | stale | unavailable | superseded
```

### 16.2 Required reference classes

Each section requires:

```text
research_refs
focusa_doc_refs
codebase_refs when implementation/runtime claims exist
evidence_refs when proof/completion claims exist
unverified_claims list when claims are not yet proven
```

### 16.3 Docs-only/runtime distinction

A reference auditor must classify Focusa doc references as one of:

```text
current_runtime_truth
current_runtime_doc
older_design_intent
normative_target
implementation_gap
unknown_currentness
```

### 16.4 Stale critical reference blocker

A section cannot be approved when critical references are stale unless the operator explicitly approves the stale-reference override.

---

## 17. Reality Scanner requirements

The Reality Scanner must automate grounding.

Minimum scanner surfaces:

```text
- docs refs scanner
- route inventory scanner
- CLI command inventory scanner
- type/schema scanner
- test/proof scanner
- TODO/docs-only/spec-only detector
- implementation status classifier
```

Implementation status enum:

```rust
pub enum ImplementationRealityStatus {
    Implemented,
    Partial,
    DocsOnly,
    NormativeTarget,
    Unknown,
    Blocked,
}
```

Scanner output must be visible in each section.

---

## 18. UIAI Engine integration

### 18.1 Initial research phase

Before section drafting, Focusa should request or accept UIAI Engine research/resource suggestions.

Inputs:

```yaml
uiai_research_request:
  current_ask:
  section_title:
  project_root:
  continuity_id:
  target_refs: []
  research_questions: []
  budget:
  allowed_domains: []
  disallowed_domains: []
```

Outputs:

```yaml
uiai_research_result:
  status: proposal_only | capture_pending | captured | linked | scope_mismatch | stale
  resources: []
  findings: []
  screenshots_or_browser_diagnostics: []
  suggested_followups: []
  evidence_capture_candidates: []
```

### 18.2 Authority boundary

UIAI Engine may suggest resources and initial research.

UIAI Engine must not:

- approve sections;
- promote spec claims;
- write canonical Focusa state directly;
- bypass Evidence capture;
- export private data by default;
- decide implementation readiness.

---

## 19. Transcript policy

Full agent arguments are useful, but transcripts can become large and sensitive.

```yaml
transcript_policy:
  full_transcript_retention: local_only
  summarized_transcript_retention: active_session_plus_receipt_refs
  private_notes_included: false
  exportable_by_default: false
  rehydrate_refs_required: true
  redaction_required_for_external_export: true
```

Transcript display:

```text
Full transcript available in Chat tab.
Compressed section summary available in Pending/Approved.
Export excludes full transcript unless operator explicitly includes it.
```

---

## 20. Export and share behavior

Supported export kinds:

```rust
pub enum SpecExportKind {
    Markdown,
    Pdf,
    EmailDraft,
    GoogleDriveDoc,
    MicrosoftCloudDoc,
    PublicSafeCard,
    RepoCommit,
}
```

### 20.1 Export gate

Every external export must pass:

```text
export request
  → destination classification
  → privacy/redaction scan
  → operator preview
  → explicit operator approval
  → export
  → receipt/projection record
```

### 20.2 Email

Email export should support:

```text
- draft body
- attached PDF
- attached Markdown
- public-safe summary
- full internal version only by explicit approval
```

### 20.3 PDF

PDF export should include:

```text
- final spec
- grounding summary
- approved sections
- unresolved/deferred issues
- reference appendix
- approval record
```

### 20.4 Google Drive / Microsoft cloud

Cloud document export is a projection.

It requires:

```text
- redaction status
- operator approval
- destination account/workspace confirmation
- receipt/projection ref
```

No cloud export may become canonical Focusa truth.

---

## 21. Repo write behavior

Final spec repo write requires explicit operator approval.

Default path:

```text
docs/120-adversarial-spec-workbench-and-operator-approval-gates.md
```

Optional sidecar path:

```text
docs/spec-data/120-adversarial-spec-workbench.session.json
```

Repo write should record:

```text
- spec_id
- final revision id
- operator gate id
- approved section ids
- evidence refs
- task decomposition plan id
- receipt ref
```

---

## 22. Task decomposition behavior

After final approval, Focusa may create a provider-neutral task decomposition plan.

Minimum decomposition:

```text
Parent task:
  Implement Spec120 — Adversarial Spec Workbench

Child tasks:
  1. Core types and ledger
  2. Reality Scanner
  3. UIAI research packet intake
  4. Agent round runner
  5. Reference auditor
  6. Operator gates
  7. Local PWA UI
  8. Export adapters
  9. Repo writer
  10. Provider-neutral task adapter
  11. Receipt integration
  12. Tests/proof bundle
```

Each child task must include:

```text
- linked spec refs
- acceptance criteria
- proof requirements
- allowed scope
- verification tier
- dependencies
```

---

## 23. Spec116 integration

Spec120 must not create a task manager-specific model.

All task writes pass through provider-neutral structures.

Provider adapters may include:

```text
BdWorkItemAdapter
GitHubIssueAdapter
LinearWorkItemAdapter
AsanaWorkItemAdapter
MarkdownChecklistAdapter
```

When a task is later closed, Spec116 controls closure.

Spec120-created tasks should include closure evidence requirements at creation time so closure claims can be validated later.

Required created-task metadata:

```yaml
spec120_task_metadata:
  spec_id:
  section_ids: []
  acceptance_criteria: []
  required_evidence:
    code_refs: []
    spec_refs: []
    proof_refs: []
  closure_kind: code | docs | deploy | investigation | no_code | admin
  closure_policy_ref:
```

---

## 24. Spec107 integration

Spec120 is the automation layer for Spec107.

Mapping:

| Spec107 lifecycle gate | Spec120 implementation |
|---|---|
| Idea Gate | `SpecWorkbenchSession` created from operator idea |
| Spec Gate | section-by-section spec workbench |
| Decomposition Gate | provider-neutral `SpecTaskDecompositionPlan` |
| Implementation Gate | `approve_implementation_start` operator gate |
| Proof Gate | section/task proof requirements + evidence classification |
| Closure Gate | Spec116 closure authority + receipt |

Spec120 must block implementation-start approval until:

```text
- final spec approved
- repo write completed or explicitly deferred
- task decomposition exists or explicitly deferred
- proof requirements exist
- receipt preview exists
```

---

## 25. Receipt integration

Final approval should create or preview a Focusa Receipt.

Receipt type:

```text
spec_approval
```

Additional receipt types:

```text
spec_export
spec_repo_write
spec_task_decomposition
implementation_start_authorization
```

Receipt should include:

```yaml
spec_receipt:
  receipt_type: spec_approval
  spec_id:
  session_id:
  current_ask:
  project_root:
  continuity_id:
  approved_sections: []
  unresolved_deferred_items: []
  research_refs: []
  focusa_doc_refs: []
  codebase_refs: []
  evidence_refs: []
  operator_gates: []
  export_artifacts: []
  repo_commit_ref:
  task_decomposition_plan_id:
  final_approval:
    approved_by:
    approved_at:
  next_safe_action:
```

---

## 26. API surface

Add:

```text
GET  /v1/spec-workbench/sessions
POST /v1/spec-workbench/session
GET  /v1/spec-workbench/session/:session_id
POST /v1/spec-workbench/session/:session_id/sections
POST /v1/spec-workbench/section/:section_id/research
POST /v1/spec-workbench/section/:section_id/round/start
POST /v1/spec-workbench/section/:section_id/round/stop
POST /v1/spec-workbench/section/:section_id/round/repeat
POST /v1/spec-workbench/section/:section_id/approve
POST /v1/spec-workbench/section/:section_id/reject
POST /v1/spec-workbench/session/:session_id/reconcile
POST /v1/spec-workbench/session/:session_id/final-approve
POST /v1/spec-workbench/session/:session_id/export
POST /v1/spec-workbench/session/:session_id/repo-write
POST /v1/spec-workbench/session/:session_id/decompose
POST /v1/spec-workbench/session/:session_id/receipt-preview
POST /v1/spec-workbench/session/:session_id/implementation-start-authorize
```

All responses use shared Focusa envelope fields:

```text
ok
status
canonical
advisory
degraded
stale
scope_status
failure_class
side_effects
evidence_refs
next_tools
recovery_hint
misuse_hint
```

---

## 27. CLI surface

Add:

```bash
focusa spec-workbench new --ask "..."
focusa spec-workbench status <session-id>
focusa spec-workbench section add <session-id> --title "..."
focusa spec-workbench section research <section-id>
focusa spec-workbench section round <section-id> --rounds 3
focusa spec-workbench section repeat <section-id>
focusa spec-workbench section stop <section-id>
focusa spec-workbench section approve <section-id>
focusa spec-workbench section reject <section-id> --reason "..."
focusa spec-workbench reconcile <session-id>
focusa spec-workbench approve-final <session-id>
focusa spec-workbench export <session-id> --kind pdf
focusa spec-workbench repo-write <session-id>
focusa spec-workbench decompose <session-id> --provider bd
focusa spec-workbench receipt-preview <session-id>
```

---

## 28. Tool contracts

Pi and non-Pi tools must have Spec90-compatible contracts.

Initial tools:

```text
focusa_spec_workbench_new
focusa_spec_workbench_status
focusa_spec_section_research
focusa_spec_section_round
focusa_spec_section_repeat
focusa_spec_section_stop
focusa_spec_section_approve
focusa_spec_section_reject
focusa_spec_reconcile
focusa_spec_final_approve
focusa_spec_export
focusa_spec_repo_write
focusa_spec_decompose
focusa_spec_receipt_preview
```

Side-effect profiles:

```text
read_only
advisory_generation
operator_gate
repo_write
external_export
task_provider_write
receipt_preview
receipt_commit
```

---

## 29. Storage and ledger

SpecWorkbench should use append-only event/ledger semantics where practical.

Suggested storage:

```text
data/spec-workbench/{project_root_hash}/sessions.jsonl
data/spec-workbench/{project_root_hash}/sections.jsonl
data/spec-workbench/{project_root_hash}/rounds.jsonl
data/spec-workbench/{project_root_hash}/arguments.jsonl
data/spec-workbench/{project_root_hash}/operator-gates.jsonl
data/spec-workbench/{project_root_hash}/exports.jsonl
```

Rules:

```text
- no destructive edits to prior approved records
- reopening creates new revision
- approvals reference exact revision ids
- exports reference exact approved spec ids
- task decomposition references exact approved spec ids
- receipts reference exact gate ids and artifact refs
```

---

## 30. Security, privacy, and redaction

### 30.1 Private by default

All sessions are local/private by default.

### 30.2 Export deny-by-default

External export requires explicit operator approval.

### 30.3 Redaction scan

Before export, scan for:

```text
- secrets
- tokens
- private URLs
- internal hostnames
- unredacted file paths
- private transcript sections
- full raw tool output
- private browser diagnostics
- proprietary code snippets
- operator private notes
```

### 30.4 Export classes

```rust
pub enum ExportPrivacyClass {
    InternalOnly,
    TeamShareable,
    PublicSafe,
    BlockedSensitive,
}
```

---

## 31. Failure modes

| Failure | Required behavior |
|---|---|
| Missing docs refs | Block section approval |
| Missing code refs for runtime claim | Block or mark claim as planned/speculative |
| Stale critical research | Recheck or operator override required |
| Challenger repeats same objections | Stop if repeated-objection guard active |
| Infinite loop running too long | Show cost/runtime counters and stop/pause controls |
| UIAI unavailable | Continue degraded with repo/docs-only research |
| Reality Scanner unavailable | Block implemented-runtime claims |
| Export redaction fails | Block export |
| Repo dirty/unsafe | Block repo write or require explicit operator decision |
| Task provider unavailable | Produce Markdown checklist fallback only if operator approves |
| Provider closure policy unavailable | Mark task closure readiness degraded |
| Receipt commit fails | Emit partial failure and recovery command |

---

## 32. Acceptance criteria

Spec120 is accepted when:

1. This spec exists and is linked from docs index/current docs.
2. Spec120 explicitly references and preserves Spec107 lifecycle gates.
3. Spec120 explicitly references and preserves Spec116 provider-neutral work-item closure authority.
4. Core types exist for session, section, round, argument, research packet, reality pack, operator gate, export artifact, and task decomposition plan.
5. API routes expose session creation, section round control, approval/rejection, reconciliation, final approval, export, repo write, decomposition, and receipt preview.
6. CLI parity exists for the same core operations.
7. Local PWA shows Chat, Pending, and Approved tabs.
8. Per-section loop controls support one-pass, fixed rounds, repeat-until-clean, and repeat-until-stopped.
9. Infinite/repeat loop can be paused/stopped by operator.
10. Reference Auditor blocks missing research/doc/code refs.
11. Reality Scanner distinguishes implemented, partial, docs-only, normative target, unknown, and blocked claims.
12. UIAI research packets are proposal-only until captured/linked.
13. Whole-spec reconciliation catches cross-section contradictions.
14. Export gate performs redaction scan and requires operator approval.
15. Repo write gate requires operator approval.
16. Task decomposition creates provider-neutral task drafts and `SpecLinkedTaskPacket`s.
17. Spec116 closure metadata is attached to created tasks where possible.
18. Final approval produces or previews a Focusa Receipt.
19. Tests prove that no section can approve without required grounding.
20. Tests prove that final spec cannot approve with unresolved blocker objections unless operator explicitly overrides.
21. Tests prove that export cannot proceed when redaction blocks.
22. Tests prove that implementation-start authorization cannot proceed before final spec approval.

---

## 33. Implementation phases

### Phase 1 — Spec and static model

Deliver:

```text
docs/120-adversarial-spec-workbench-and-operator-approval-gates.md
core type definitions
static schema tests
docs index links
```

### Phase 2 — Ledger and API read/write skeleton

Deliver:

```text
session create/read
section create/read
round append
operator gate append
status projection
```

No real LLM calls yet.

### Phase 3 — Reality Scanner

Deliver:

```text
docs scanner
code/route/CLI/type/test scanner
implementation status classifier
reference audit blocker
```

### Phase 4 — Mock agent rounds

Deliver deterministic fixture-based:

```text
proposer
challenger
reference auditor
synthesis
```

### Phase 5 — Local PWA

Deliver:

```text
Chat tab
Pending tab
Approved tab
round controls
approval controls
reference visibility
```

### Phase 6 — UIAI research integration

Deliver:

```text
UIAI research request/response intake
resource suggestions
status mapping
capture/link workflow
```

### Phase 7 — Real model-backed adversarial rounds

Deliver:

```text
bounded proposer
bounded challenger
bounded reference auditor
strict JSON
fail-closed parsing
loop controls
cost/runtime counters
```

### Phase 8 — Final spec composition and reconciliation

Deliver:

```text
approved section composer
cross-section contradiction pass
final approval gate
```

### Phase 9 — Export/repo/task/receipt integration

Deliver:

```text
Markdown export
PDF export
email draft export
Google Drive / Microsoft cloud adapter stubs or connectors
repo write
provider-neutral task decomposition
receipt preview/commit
```

### Phase 10 — Tauri shell and menubar launch/status

Deliver:

```text
Tauri wrapper around local PWA
menubar quick-launch/status
approval count badge or ambient indicator
no canonical authority in UI shell
```

---

## 34. First implementation bead decomposition

Parent:

```text
focusa-spec120 — Implement Adversarial Spec Workbench and Operator Approval Gates
```

Children:

```text
focusa-spec120.1 — Add Spec120 docs and validation links
focusa-spec120.2 — Add core typed records and static schema tests
focusa-spec120.3 — Add append-only spec workbench ledger
focusa-spec120.4 — Add API route skeleton and status projection
focusa-spec120.5 — Add CLI parity skeleton
focusa-spec120.6 — Implement Reality Scanner
focusa-spec120.7 — Implement Reference Auditor
focusa-spec120.8 — Implement mock proposer/challenger/synthesis round runner
focusa-spec120.9 — Build local PWA with Chat/Pending/Approved tabs
focusa-spec120.10 — Add per-section loop controls including repeat-until-stopped
focusa-spec120.11 — Add UIAI research packet intake
focusa-spec120.12 — Add model-backed bounded adversarial rounds
focusa-spec120.13 — Add whole-spec reconciliation pass
focusa-spec120.14 — Add export/redaction gate
focusa-spec120.15 — Add repo writer gate
focusa-spec120.16 — Add provider-neutral task decomposition
focusa-spec120.17 — Add Spec119 receipt integration
focusa-spec120.18 — Add Spec107/Spec116 compliance tests
focusa-spec120.19 — Add Tauri shell preview
focusa-spec120.20 — Add menubar quick-launch/status integration
```

---

## 35. Definition of done

Spec120 is done when an operator can:

```text
1. Start from a rough idea.
2. See Focusa ground the idea in current docs and code.
3. Let UIAI suggest initial research/resources.
4. Generate a section outline.
5. Run each section through fixed or repeat argument rounds.
6. Watch full proposer/challenger/auditor transcript.
7. Approve/reject each section.
8. Require accurate research/doc/code refs for every section.
9. Compose approved sections into a final spec.
10. Run whole-spec reconciliation.
11. Approve final spec.
12. Export to Markdown/PDF/email/cloud with redaction gates.
13. Write the spec back into the repo.
14. Decompose the spec into provider-neutral tasks.
15. Preserve Spec107 lifecycle discipline.
16. Preserve Spec116 closure authority.
17. Produce a Focusa Receipt proving what was approved, exported, committed, and decomposed.
```

---

## 36. Final principle

Focusa should not ask agents to jump from idea to implementation.

Focusa should make the idea pass through structured thought, adversarial pressure, evidence grounding, operator approval, task decomposition, and proof.

Spec120 turns specification into a governed, replayable, operator-controlled Focusa workflow.
