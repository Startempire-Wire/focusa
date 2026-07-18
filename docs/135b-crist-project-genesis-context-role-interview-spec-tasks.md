# Spec 135B — C.R.I.S.T. Project Genesis: Context, Role, Interview, Spec, and Tasks

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135B.  
**Scope:** Project Genesis, context ingestion and continuous growth, domain-pack selection, candidate/canonical semantic integration, Role Composer, dynamic interview compendium, Spec 120 integration, provider-neutral task decomposition, Project Genesis Profile, ongoing amendment, project onboarding UI, and authority/privacy boundaries.

---

## 0. One-line definition

C.R.I.S.T. is Focusa’s governed project-genesis workflow that turns source-linked project context and operator knowledge into an approved agent role, a persistent interview corpus, an adversarially approved Project Genesis Spec, provider-neutral tasks, and the first safe Workpoint.

```text
C — Context
R — Role
I — Interview
S — Spec
T — Tasks
```

C.R.I.S.T. consumes the shared cognition registry and active domain packs defined by [Spec 135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md). It may propose semantic objects, links, actions, claims, and Workpoint candidates, but canonical promotion remains reducer-governed, policy-verified, and operator-gated where required.

---

## 1. C.R.I.T. basis and Focusa modification

C.R.I.T., attributed to Geoff Woods, stands for Context, Role, Interview, and Task. Its key interaction change is that the AI interviews the user before completing the task, asking questions one at a time to surface deeper context.

Focusa modifies this into C.R.I.S.T. because Focusa’s spec-first lifecycle requires two governed stages where C.R.I.T. has one Task stage:

```text
C.R.I.T.
Context → Role → Interview → Task

Focusa C.R.I.S.T.
Context → Role → Interview → Spec → Tasks
```

C.R.I.S.T. is a Focusa adaptation inspired by C.R.I.T. It must not be presented as Geoff Woods’ published framework or endorsement.

Reference:

- `https://www.aileadership.com/newsletter/crit-happens-the-viral-framework-transforming-how-leaders-think`

---

## 2. Why Project Genesis exists

The current First Mission proves that Focusa can bind a project, create a Workpoint, attach proof, and render a resume packet. It does not establish a complete project operating profile.

A durable project needs to answer:

```text
What is this project?
Why does it exist?
What source context is available?
Which extracted claims are accepted as project truth?
What role should the agent serve?
What does the operator know that documents do not reveal?
What remains uncertain or contradicted?
What approved high-level spec governs the project?
What tasks follow from the spec?
Which workspace should present the project?
What authority and evidence rules apply?
What is the first safe Workpoint?
```

---

## 3. Entry points

Recommended command hierarchy:

```text
focusa project new
focusa project genesis
focusa project profile
focusa project context
focusa project role
focusa project interview
focusa project spec
focusa project tasks
```

Mission Deck entry:

```text
New Project
├── Quick Mission
└── Full C.R.I.S.T. Genesis
```

### 3.1 Quick Mission

Preserves the existing short proof route:

```text
Bind project
→ create Workpoint
→ link proof
→ render resume packet
```

### 3.2 Full C.R.I.S.T. Genesis

```text
Bind/create project
→ verify scope
→ select or accept recommended workspace
→ Context
→ Role
→ Interview
→ Spec Workbench
→ Task decomposition
→ first Workpoint
→ operational workspace
```

Quick Mission and Full Genesis serve different purposes. The existing First Mission must not be overloaded into one mandatory long questionnaire.

---

## 4. State machine

```text
created
→ project_scope_verified
→ context_collecting
→ context_ready
→ role_drafting
→ role_pending_operator
→ role_approved
→ interviewing
→ interview_ready
→ spec_workbench_created
→ spec_in_review
→ spec_approved
→ task_plan_drafting
→ task_plan_pending_operator
→ tasks_materialized
→ first_workpoint_ready
→ operational
```

Project Genesis remains iterative:

```text
operational
→ new context observed
→ impact assessment
→ role revision candidate
→ interview extension
→ spec amendment candidate
→ task-plan revision
→ operational
```

No approved role, spec, or task plan may be silently rewritten by new context.

---

# Part I — Context

## 5. Context definition

Context is a source-linked, continuously growing project knowledge substrate from which Focusa builds bounded Context Cognition projections.

```text
Project corpus
≠
turn prompt
```

The corpus may be large. Each action receives only the relevant, scoped, permission-safe projection.

---

## 6. Required context sources

### 6.1 Local

- PDF;
- Markdown and text;
- word-processing documents;
- presentations;
- spreadsheets;
- images;
- email export files;
- archives;
- project folders;
- repository docs and code;
- configuration;
- existing task data.

### 6.2 Connected

- Google Drive;
- Microsoft OneDrive;
- SharePoint;
- Gmail;
- Outlook or Microsoft mail;
- task providers;
- other approved project systems.

### 6.3 External research

- UIAI web search;
- public websites;
- browser reads and snapshots;
- screenshots;
- API documentation;
- product/competitor/reference research.

### 6.4 Focusa-native

- ProjectIdentity;
- Project Card;
- Trajectory;
- Workpoints;
- Evidence;
- Receipts;
- project events;
- prior interviews;
- approved specs;
- task history;
- operator notes;
- predictions and metacognition where relevant.

---

## 7. Context-source adapter

```rust
pub trait ProjectContextSourceAdapter {
    async fn detect(&self) -> AdapterHealth;
    async fn authorize(&self, request: AuthorizationRequest) -> AuthorizationResult;
    async fn enumerate(&self, scope: SourceScope) -> SourceInventory;
    async fn fetch_delta(&self, cursor: Option<String>) -> SourceDelta;
    async fn revoke(&self) -> RevokeResult;
    async fn health(&self) -> AdapterHealth;
}
```

Required adapter facts:

- provider ID;
- supported source kinds;
- read/write posture;
- OAuth scopes;
- incremental-sync method;
- cursor/subscription state;
- rate-limit posture;
- health;
- last successful sync;
- recovery action;
- revocation behavior.

---

## 8. OAuth and connector laws

Connectors must:

- request minimum required scopes;
- default to read-only;
- allow account, folder, label, sender, date, and source filters;
- avoid full-mailbox/full-drive ingestion by default;
- keep secrets out of project files and model context;
- store credential references rather than raw credentials in project state;
- support revocation;
- expose health and sync state;
- persist incremental cursors;
- process idempotent deltas;
- recover from cursor expiry and subscription loss;
- produce receipts/evidence for consequential connector changes.

Connector success is not OAuth success alone. Complete behavior is:

```text
connect
→ enumerate
→ bounded import
→ normalize
→ index
→ incremental update
→ recover
→ revoke
→ display health
→ preserve provenance
```

---

## 9. Context pipeline

```text
discovered
→ authorized
→ fetched
→ normalized
→ classified
→ extracted
→ chunked
→ indexed
→ deduplicated
→ source-linked
→ contradiction-checked
→ candidate context and candidate semantic graph
→ registered verification-policy evaluation
→ operator accepted / advisory / rejected
→ canonical promotion where permitted
→ retained / superseded / archived
```

Required processing properties:

- MIME detection;
- content hashing;
- source revision;
- extraction diagnostics;
- lexical indexing;
- semantic indexing;
- source-preserving chunks;
- sensitivity classification;
- freshness;
- retention;
- contradiction links;
- domain-pack and semantic-type classification;
- verification-policy references;
- candidate/canonical graph separation;
- bounded retrieval.

---

## 10. Project Context Artifact

```yaml
schema: focusa.project_context_artifact.v1

artifact_id:
source_kind:
source_ref:
source_revision:
title:
mime_type:
content_handle:
content_sha256:
created_at:
observed_at:

scope:
  project_root:
  continuity_id:

provenance:
  connector_id:
  account_ref:
  author:
  source_url:
  page_or_message_ref:

classification:
  sensitivity:
  confidentiality:
  retention_class:
  freshness_status:

extraction:
  status:
  diagnostic_refs: []
  extracted_claim_ids: []
  entity_refs: []
  date_refs: []
  task_refs: []
  contradiction_refs: []

semantic:
  domain_pack_refs: []
  candidate_object_refs: []
  candidate_link_refs: []
  verification_policy_refs: []
```

---

## 11. Project Context Claim

Extracted text does not silently become canonical project truth.

```yaml
schema: focusa.project_context_claim.v1

claim_id:
claim:
source_artifact_refs: []
domain_pack_refs: []
semantic_object_ref:
verification_policy_ref:
verification_refs: []
confidence:
freshness:
status: observed | candidate | accepted | contradicted | superseded | rejected
accepted_by:
accepted_at:
supersedes:
```

Accepted claims must remain traceable to their sources, semantic identity, verification policy, and acceptance record. Acceptance promotes only the claim/object versions authorized by policy; it does not silently canonize every inferred relation derived from the source artifact.

---

## 12. Continuous growth and impact assessment

Every new Focusa source may contribute context, subject to scope, privacy, retention, and relevance rules.

New context emits a `ProjectContextDelta` and an impact assessment:

```yaml
schema: focusa.project_profile_impact_assessment.v1

trigger_ref:
affected:
  context_claims: []
  role_fields: []
  interview_questions: []
  spec_sections: []
  tasks: []
  trajectory_fields: []

severity: informational | review | blocker
recommended_actions: []
automatic_mutations: []
operator_approval_required: true
```

Approved project artifacts are revised only through explicit revision paths.

---

# Part II — Role

## 13. Role definition

Role defines the expert function the Focusa-powered agent should serve for the project.

It does not define permission or authority.

---

## 14. Role Composer inputs

- operator role seed;
- accepted context claims;
- source artifacts;
- workspace profile;
- stakeholders;
- expected deliverables;
- constraints;
- evidence expectations;
- known handoff boundaries;
- interview answers already available.

---

## 15. Required role draft

The AI Role Composer proposes:

```text
Role title
Purpose
Domain expertise
Primary responsibilities
Secondary responsibilities
Expected deliverables
Quality standards
Decision principles
Evidence behavior
Communication posture
Stakeholder posture
Non-responsibilities
Forbidden assumptions
Escalation triggers
Handoff boundaries
Tool preferences
Reviewer/challenger lenses
```

The Role Composer must identify assumptions and source grounding.

---

## 16. Role lifecycle

```text
operator seed
→ AI role draft
→ context-grounding check
→ contradiction and authority review
→ operator edit
→ operator approval
→ active Role Profile
```

UI requirements:

- original seed;
- generated draft;
- grounding/source indicators;
- assumptions;
- unresolved questions;
- before/after redline;
- edit;
- regenerate section;
- approve;
- reject;
- defer;
- revision history.

---

## 17. Project Agent Role Profile

```yaml
schema: focusa.project_agent_role_profile.v1

role_profile_id:
project_root:
continuity_id:
revision:

title:
purpose:
expertise: []
responsibilities: []
deliverables: []
quality_standards: []
decision_principles: []
evidence_expectations: []
communication_posture:
non_responsibilities: []
forbidden_assumptions: []
escalation_triggers: []
handoff_boundaries: []
secondary_lenses: []

grounding:
  context_artifact_refs: []
  context_claim_refs: []
  interview_answer_refs: []
  operator_seed_ref:

status: draft | pending_operator | approved | superseded
approved_by:
approved_at:
```

---

## 18. Role and permission separation

A role may declare:

```text
Act as a world-class legal research and matter-strategy assistant.
```

It may not grant:

```text
permission to file;
permission to send email;
permission to trade;
permission to modify production;
permission to access an unapproved source.
```

Those remain PermissionProfile, operational policy, and explicit operator-gate concerns.

---

# Part III — Interview

## 19. Interview definition

The agent interviews the operator to uncover project context not available in documents and resolve gaps that would weaken the Project Genesis Spec.

Questions are asked one at a time but stored in a visible, persistent compendium.

---

## 20. Dynamic question generation

Questions are generated from:

- missing Project Genesis sections;
- contradictory documents;
- low-confidence context claims;
- role ambiguity;
- unclear desired state;
- missing users/stakeholders;
- missing non-goals;
- evidence gaps;
- authority ambiguity;
- privacy/compliance gaps;
- integration uncertainty;
- task-decomposition uncertainty;
- new context impact;
- missing required fields, relations, verification evidence, or slice-policy inputs from active domain packs.

The interview must not become a universal static long form.

---

## 21. Interview tranches

```text
Interview tranche
→ ask one highest-value question
→ wait for answer
→ persist answer
→ reassess gaps
→ ask next question
→ stop when tranche goal is satisfied or operator stops
```

The operator can:

- pause;
- stop;
- skip;
- defer;
- answer later;
- request more questions;
- open a new tranche;
- amend an answer;
- withdraw an answer;
- attach files or links.

---

## 22. Question record

```yaml
schema: focusa.project_interview_question.v1

question_id:
session_id:
question:
reason_for_asking:
triggering_gap:
linked_spec_sections: []
linked_context_refs: []
priority: blocker | high | normal | optional
answer_type: text | long_text | boolean | select | multi_select | number | date | link | file | approval
sensitivity:
status: queued | asked | answered | deferred | skipped | superseded
created_at:
answered_at:
```

---

## 23. Answer record

```yaml
schema: focusa.project_interview_answer.v1

answer_id:
question_id:
answer:
attachment_refs: []
operator_id:
status: active | amended | superseded | withdrawn
confidence:
notes:
created_at:
supersedes:
```

AI-generated summaries never replace the operator answer; they remain linked projections.

---

## 24. Question domains

Only relevant domains are selected:

```text
Purpose and desired outcome
Current state
Primary users
Stakeholders
Decision makers
Constraints
Non-goals
Success criteria
Evidence expectations
Risks
Legal/compliance requirements
Data sensitivity
Approvals
Deadlines
Dependencies
Integrations
Existing systems
Workflow and cadence
Known failures
Unknowns
Resource boundaries
Handoff expectations
```

---

## 25. Interview readiness gate

The interview is ready to proceed to Spec when:

- no unresolved blocker information gap remains, or blocker gaps are explicitly operator-accepted;
- desired state is explicit;
- scope and non-goals are sufficiently clear;
- stakeholders and authority are sufficiently clear;
- evidence expectations are defined or explicitly open;
- unknowns are recorded rather than hidden;
- contradictions are surfaced;
- the operator approves moving to Spec.

Interview readiness does not mean every conceivable question is answered.

---

## 26. Continuing later

The operational workspace permanently exposes:

```text
Continue Interview
Add Context
Revisit Answer
Ask About New Context
Resolve Contradiction
```

The interview is a project asset, not a disposable onboarding transcript.

---

# Part IV — Spec

## 27. Spec ownership

The `S` stage invokes [Spec 120](120-adversarial-spec-workbench-and-operator-approval-gates.md).

C.R.I.S.T. must not implement another:

- spec generator;
- adversarial loop;
- reference auditor;
- section approval system;
- spec-to-task model.

---

## 28. C.R.I.S.T. handoff packet

```yaml
schema: focusa.crist_spec_handoff.v1

project_root:
continuity_id:
current_ask:
workspace_profile_ref:
active_domain_pack_refs: []
semantic_registry_version:
context_pack_refs: []
accepted_project_claim_refs: []
role_profile_ref:
interview_session_refs: []
unresolved_questions: []
known_contradictions: []
desired_spec_template: project_genesis
```

---

## 29. Project Genesis Spec template

Required sections:

1. Project title and one-line definition.
2. Problem or opportunity.
3. Project identity and current-state reality.
4. Long-term desired state / mandatory HLT.
5. Users and stakeholders.
6. Approved project agent role.
7. Context sources and provenance.
8. Scope.
9. Non-goals.
10. Constraints.
11. Risks.
12. Authority and approval boundaries.
13. Data, privacy, retention, and connector posture.
14. Workspace and visual profile.
15. Evidence and proof policy.
16. Core workflows.
17. Initial architecture or operating model.
18. Success criteria.
19. Milestones and Waypoints.
20. Known unknowns and open questions.
21. Initial task-decomposition policy.
22. Final approval record.

Every section uses Spec 120 grounding, reality classification, adversarial challenge, reference audit, and operator gates.

---

## 30. Reality classification

Every claim is classified:

```text
implemented
partial
docs_only
normative_target
planned
speculative
stale
blocked
unknown
```

Docs-only behavior cannot be presented as runtime behavior.

---

## 31. Trajectory proposal

After final approval, the Workbench may propose:

```text
HLT
MLG
STG
Waypoints
Definition of done
Evidence requirements
First Workpoint candidate
```

Trajectory and Workpoint promotion remain governed Focusa actions, not automatic consequences of generated prose.

---

## 32. Sidebar and Workbench boundary

```text
Sidebar
→ progress, pending approvals, alerts, next action.

Full Spec Workbench
→ research, arguments, references, section editing, approval, reconciliation.
```

Do not duplicate the Workbench inside the sidebar.

---

# Part V — Tasks

## 33. Task ownership

The `T` stage uses:

- Spec 120 provider-neutral decomposition;
- Spec 116 provider adapters and closure authority;
- Workpoints for active execution;
- Spec 119 Receipts;
- Spec 135A Work Rail.

---

## 34. Task-plan flow

```text
Approved Project Genesis Spec
→ decomposition proposal
→ parent/child dependency graph
→ acceptance/proof mapping
→ operator preview
→ edit/split/merge/reorder
→ operator approval
→ provider-neutral materialization
→ provider adapters
→ Work Rail
→ selected task
→ first Workpoint
```

No provider mutation occurs during draft decomposition.

---

## 35. Required task model

```yaml
task:
  provider_neutral_id:
  title:
  description:
  linked_spec_sections: []
  acceptance_criteria: []
  evidence_requirements: []
  semantic_object_refs: []
  allowed_action_type_ids: []
  verification_policy_ref:
  allowed_scope: []
  dependencies: []
  blockers: []
  task_class:
  closure_kind:
  closure_policy_ref:
  preferred_provider:
  provider_ref:
```

---

## 36. Provider capability truth

The UI distinguishes:

```text
configured and operational
configured but unhealthy
read-only
credentials missing
adapter unavailable
schema-only support
mutation approval required
```

### Required adapter graph

- Beads;
- GitHub Issues;
- Linear;
- Asana;
- Markdown Checklist.

Each remains required for series closure unless explicitly removed by a versioned operator amendment.

---

## 37. Workpoint activation

Task creation does not make every task active.

A task becomes the active execution object only when:

- selected by the operator or approved work-loop policy;
- bound to a Workpoint;
- verified within the correct project and continuity scope;
- assigned an allowed action/evidence posture;
- resolved against the active domain packs and canonical semantic graph;
- promoted through the existing Workpoint reducer path from a previewable ontology-derived or operator-authored Workpoint candidate.

---

## 38. Receipts

Final genesis approval should produce or preview Receipts for:

```text
project_genesis
role_approval
interview_readiness
spec_approval
spec_repo_write
spec_task_decomposition
task_materialization
first_workpoint
```

The completed-task experience eventually opens its work Receipt with ask, scope, Workpoint revision, actions, artifacts, evidence, authority, closure, unfinished work, and next safe action.

---

# Part VI — Persistent Project Profile

## 39. Project Genesis Record

```yaml
schema: focusa.project_genesis.v1

genesis_id:
project_root:
continuity_id:
status:
revision:

workspace:
  active_profile_ref:
  visual_variant:
  project_overrides_ref:

domain_semantics:
  registry_version:
  active_domain_pack_refs: []
  compatibility_profile_ref:
  semantic_projection_ref:

context:
  source_refs: []
  accepted_claim_refs: []
  pending_claim_refs: []
  contradictions: []
  last_growth_at:

role:
  active_role_profile_ref:
  pending_revision_ref:

interview:
  session_refs: []
  answered_count:
  open_count:
  blocker_count:

spec:
  workbench_session_ref:
  approved_spec_ref:
  spec_revision:
  reconciliation_status:

tasks:
  task_plan_ref:
  materialization_status:
  provider_refs: []

trajectory:
  trajectory_proposal_ref:
  active_workpoint_ref:

receipts: []
created_at:
updated_at:
```

---

## 40. Resolved Project Operating Profile

Clients consume a bounded projection:

```yaml
schema: focusa.resolved_project_operating_profile.v1

project_identity:
workspace_projection:
domain_semantic_summary:
active_domain_pack_refs: []
agent_role:
context_summary:
interview_summary:
approved_spec_summary:
task_summary:
trajectory_summary:
active_workpoint:
evidence_summary:
authority_summary:
connector_health:
stale_components: []
required_operator_actions: []
next_safe_action:
```

This projection is not a second canonical store.

---

## 41. Project Genesis UI

```text
┌ PROJECT GENESIS ──────────────────────────────────────────────────────┐
│ Project / Workspace                                                  │
├───────────────────────────────────────────────────────────────────────┤
│ C Context      sources · artifacts · contradictions                  │
│ R Role         draft / review / approved                             │
│ I Interview    answered · open · blockers                            │
│ S Spec         Workbench status                                      │
│ T Tasks        plan / materialization                                │
├───────────────────────────────────────────────────────────────────────┤
│ [Add Context] [Continue Interview] [Review Role] [Open Workbench]     │
└───────────────────────────────────────────────────────────────────────┘
```

Each stage must autosave and remain resumable.

---

## 42. Authority and privacy laws

1. Selected project convenience state does not mint canonical scope.
2. Context remains source-linked.
3. Raw connector credentials never enter project files or prompts.
4. Role does not grant permission.
5. Interview answers remain operator-owned.
6. Spec approval remains operator-controlled.
7. Tasks require approved decomposition.
8. Raw connected data remains local by default.
9. UIAI research remains proposal/evidence material until Focusa captures/links it.
10. Operator steering wins.
11. Export requires classification, redaction preview, approval, and Receipt.
12. Connector revocation must stop future sync and clearly define historical evidence retention.

---

## 43. Migration for existing projects

Existing projects continue operating without C.R.I.S.T.

They show:

```text
Project Genesis
Not completed

[Start C.R.I.S.T.]
[Import from existing project]
[Dismiss for now]
```

Import may seed Context from:

- project marker;
- repository docs/code;
- Project Card;
- current Trajectory;
- active Workpoint;
- work items;
- Evidence;
- project settings.

All inferred fields are labeled and require operator review. Existing projects remain on the legacy compatibility projection until an explicit domain-pack composition is accepted; migration may recommend `focusa.core.cognition@1`, `focusa.software@1`, or `focusa.general@1` but may not silently change authority or invalidate current Workpoints.

---

## 44. Acceptance criteria

Spec 135B is accepted when:

1. Quick Mission remains available.
2. Full C.R.I.S.T. can be created, paused, resumed, and completed.
3. Local and connected sources use one provenance-preserving context model.
4. Google Drive, OneDrive/SharePoint, Gmail, Outlook/Microsoft mail, UIAI, local files, Focusa state, and task-provider context operate.
5. Incremental sync, cursor recovery, health, and revocation work.
6. Context claims are candidate/accepted/contradicted/superseded/rejected and source-linked.
7. Context growth emits impact assessments without silent rewrites.
8. Role Composer creates a grounded, editable, versioned draft.
9. Role approval is explicit and separate from permission.
10. Interview questions are dynamic, one-at-a-time, persistent, amendable, and resumable.
11. Interview readiness is explicit and operator-approved.
12. C.R.I.S.T. launches an operational Spec 120 Workbench with the Project Genesis template.
13. Whole-spec reconciliation and final operator approval operate.
14. Provider-neutral task preview, approval, and materialization operate.
15. Required provider adapters display truthful capability state.
16. The first selected task can become a scoped Workpoint.
17. Project Genesis Receipts are produced.
18. Existing projects migrate without hidden authority changes.
19. Actual connector, role, interview, Workbench, task, and resume evidence is captured.
20. Context artifacts and claims enter candidate semantic state before policy-backed canonical promotion.
21. Active domain packs shape relevant interview gaps, semantic task references, evidence requirements, and Workpoint candidates without granting permission.
22. Existing C.R.I.S.T.-free projects continue through the V1 compatibility projection and can opt into reviewed domain-pack migration.

---

## 45. Closure blockers

This spec cannot close while:

- context is stored only as prompt text;
- accepted claims lose provenance;
- any accepted connector is missing or full-resync-only without recovery;
- Role is not versioned/operator-approved;
- Interview is transcript-only or disposable;
- Spec is generated without Spec 120 gates;
- task plans mutate providers without preview/approval;
- required adapters exist only as types;
- first task cannot become a scoped Workpoint;
- Project Genesis cannot resume after closing the UI;
- actual end-to-end evidence is missing;
- candidate context or inferred semantic relations can bypass registered verification/promotion policy;
- a first Workpoint is activated with unresolved semantic references or an unavailable required domain pack.
