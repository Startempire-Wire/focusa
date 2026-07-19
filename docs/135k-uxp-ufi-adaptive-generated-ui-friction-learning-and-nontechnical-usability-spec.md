# Spec 135K — UXP/UFI Adaptive Generated UI, Friction Learning, and Nontechnical Usability

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 14](14-uxp-ufi-schema.md), [Spec 135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md), [Spec 135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md), [Spec 135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md), and [Spec 135J](135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135K.  
**Scope:** canonical UXP/UFI reuse, generated UI adaptation, nontechnical defaults, explanation depth, confirmation posture, interruption sensitivity, review cadence, observable friction, transparent calibration, accessibility, generated-surface projection, and usability proof.

---

## 0. One-line definition

Focusa’s generated C.R.I.S.T. UI must begin with a safe nontechnical baseline and adapt its explanation depth, pacing, confirmation density, review cadence, and presentation through the existing transparent UXP/UFI system rather than creating a second user-mode, expertise score, or hidden personalization engine.

---

## 1. Reuse law

[Spec 14](14-uxp-ufi-schema.md) is authoritative for user-experience calibration.

Spec 135 generated UI must reuse:

```text
UXP
  Slow-moving, confidence-weighted, cited, reversible user preferences.

UFI
  Fast-moving, per-interaction, observable friction records.
```

Do not create:

- `NontechnicalUserProfile`;
- `SimpleModeProfile`;
- `ExpertiseScore`;
- hidden technical-skill inference;
- emotion/personality labels;
- a second explanation or confirmation preference store;
- client-local permanent personalization.

---

## 2. Default nontechnical baseline

Before enough UXP evidence exists, all Project Genesis and C.R.I.S.T. surfaces use this baseline:

```text
plain language
one primary action
moderate explanation depth
recommendation with source basis
consequences before commitment
autosave confirmation
simple confirmation for reversible actions
consequential confirmation for authority-changing or irreversible actions
advanced details collapsed
no raw identifiers required
no hidden safety/authority state
```

Adaptation may reduce friction or increase detail. It may not remove mandatory safety, authority, evidence, privacy, or recovery information.

---

## 3. Canonical UXP dimensions used by generated UI

Use existing Spec 14 dimensions:

```text
verbosity_preference
  Length of generated explanations and summaries.

explanation_depth
  Amount of rationale, consequences, examples, and source context.

confirmation_preference
  Additional confirmation for reversible, non-safety-critical actions.

interruption_sensitivity
  Whether optional prompts/updates wait for a natural boundary.

review_cadence
  Frequency of compendium, role, context, spec, and task-plan review prompts.

risk_tolerance
  Presentation and emphasis of known risks; never changes permission.

autonomy_tolerance
  How strongly Focusa may suggest defaults or continue advisory preparation;
  never grants authority.
```

These dimensions modify presentation and advisory pacing only. They do not modify canonical correctness, required evidence, permission, Workpoint scope, or operator approval gates.

---

## 4. Generated Surface integration

`focusa.generated_surface.v1` must reference canonical UXP/UFI projections:

```yaml
experience:
  uxp_profile_ref:
  uxp_version:
  applied_dimensions:
    verbosity_preference:
    explanation_depth:
    confirmation_preference:
    interruption_sensitivity:
    review_cadence:
    risk_tolerance:
    autonomy_tolerance:
  user_overrides: []
  recent_friction_summary_ref:
  adaptation_explanation_ref:
```

The generated deterministic shell resolves UXP before producing presentation variants.

```text
canonical stage/read model
+ capabilities/authority
+ Workspace View Profile/domain packs
+ UXP projection
+ client/accessibility capabilities
→ UiInteractionIntent
→ A2UI surface
```

---

## 5. UFI capture from generated UI

Generated UI interactions may record only observable friction signals defined by Spec 14.

High-value examples:

```text
manual_override
immediate_correction
undo_or_revert
explicit_rejection
task_reopened
```

Medium-value examples:

```text
rephrase
repeat_request
scope_clarification
forced_simplification
```

Additional generated-UI observations may be proposed only through a versioned Spec 14 amendment and must remain behavioral rather than emotional.

### 5.1 Interaction linkage

```yaml
ufi_context:
  generated_surface_id:
  surface_revision:
  component_id:
  action_binding_id:
  crist_stage:
  project_root:
  continuity_id:
  attachment_id:
  workspace_profile_ref:
  agent_id:
  model_id:
  harness_id:
```

### 5.2 No friction from normal completion

Completing a form slowly, opening Advanced details, asking for help, or using accessibility controls is not automatically friction or low expertise.

---

## 6. Adaptation rules

### 6.1 Slow and reversible

UXP changes follow Spec 14’s trend-window, confidence, citation, alpha, and user-override rules.

One interaction never changes permanent UI behavior.

### 6.2 Transparent

The UI must answer:

```text
Why is Focusa explaining this this way?
What interactions informed it?
How confident is it?
Can I change it?
```

Provide an **Why this presentation?** advanced panel with citations and adjustment controls.

### 6.3 User override

The operator can explicitly adjust:

- explanation depth;
- summary length;
- optional confirmation frequency;
- review cadence;
- optional interruption timing.

A user override freezes learning for that dimension until released, as required by Spec 14.

### 6.4 Safety invariants

UXP/UFI may not:

- hide required approval;
- remove risk or contradiction disclosure;
- suppress evidence requirements;
- skip Context/Role/Interview/Spec/Task readiness gates;
- change provider or connector permission;
- grant autonomy;
- auto-accept a recommendation;
- weaken accessible semantics;
- change canonical action bindings.

---

## 7. Stage-specific adaptation

### Context

- Adapt explanation length for permissions, source scope, privacy, and claims.
- Never collapse import scope, credential, or retention consequences below required disclosure.

### Role

- Adapt whether responsibilities show as a summary first or expanded sections.
- Permission separation remains visible.

### Interview

- Adapt recommendation depth, examples, and branch-progress detail.
- Always ask one primary question.
- High interruption sensitivity defers optional follow-up branches, not blocker questions.

### Spec

- Adapt section summaries and objection explanation depth.
- Approval consequences remain explicit.

### Tasks

- Adapt task graph summary versus detail.
- Acceptance, evidence, blockers, provider destination, and mutation preview remain available and accurate.

---

## 8. PlainLanguageProjection integration

`PlainLanguageProjection` consumes:

```text
canonical terminology
Workspace View Profile terminology
active domain packs
glossary
shared ToolResult/error envelope
UXP dimensions
client/accessibility capabilities
```

It produces:

- user-facing labels;
- explanations;
- recommendation summaries;
- consequence summaries;
- recovery copy;
- advanced technical labels;
- help topics.

It does not create canonical facts or reinterpret failure classes.

---

## 9. Speed and implementation reuse

### 9.1 Reuse existing canonical storage

UXP/UFI use the existing local SQLite model defined by Spec 14. Do not create generated-UI preference tables unless they are explicit migrations/extensions of the canonical UXP/UFI schema.

### 9.2 Reuse generated events

UFI records derive from typed Focusa UI actions, corrections, reverts, and outcomes already flowing through the Operation Registry and canonical event stream. Do not add separate browser analytics as the canonical friction source.

### 9.3 Deterministic presentation variants

Use predefined catalog variants and token sets selected by UXP values. Do not use an LLM merely to decide font size, panel expansion, confirmation count, or summary length.

### 9.4 Fixture matrix

Create deterministic generated-surface fixtures for:

```text
new user baseline
high explanation-depth preference
low verbosity preference
high interruption sensitivity
high review cadence
explicit user overrides
low-confidence UXP
conflicting scope-specific UXP
```

This allows UI work before learning loops are fully live.

---

## 10. API integration

Required projections and operations:

```text
GET  /v1/uxp/profile
GET  /v1/uxp/profile/explain
POST /v1/uxp/profile/override/preview
POST /v1/uxp/profile/override/commit
GET  /v1/ufi/recent
POST /v1/ufi/record
```

If equivalent current routes exist, extend and reuse them rather than creating duplicates.

Operation Registry metadata marks UXP/UFI operations and prevents generated UI from treating inferred friction as authority.

---

## 11. Testing and usability proof

Required:

- reducer/property tests for slow, bounded, reversible learning;
- citation and confidence tests;
- user-override freeze tests;
- scope separation by user/agent/model/harness;
- generated-surface snapshot matrix for UXP variants;
- Playwright tests proving the same task remains completable across variants;
- accessibility tests for every presentation variant;
- tests proving safety and approval controls cannot be hidden;
- nontechnical evaluator study using the Cross-Functional Alpha;
- friction analysis showing where users correct, repeat, abandon, or reopen;
- proof that adaptation reduces repeated friction without hidden inference.

### 11.1 Nontechnical completion benchmark

A new evaluator with no Focusa vocabulary must be able to:

```text
start Project Genesis
add local Context
understand and approve a Role
answer a Grill question
understand Spec progress
approve a task plan
start the first Workpoint
recover from one connector or validation failure
resume after closing the client
```

The evaluator may use generated help. They may not require a CLI command, raw JSON, schema, route name, or developer intervention.

---

## 12. Agent decomposition directive

Every decomposing agent must receive this instruction verbatim or equivalently:

```text
Use Focusa's existing Spec 14 UXP/UFI model for all generated-UI adaptation.
Do not create a simple mode, expert mode, expertise score, hidden user profile,
emotion model, or second personalization store.

Start every C.R.I.S.T. surface from the safe nontechnical baseline. Adapt
verbosity, explanation depth, optional confirmation density, interruption
sensitivity, and review cadence through cited, confidence-weighted, reversible
UXP dimensions. Record only observable UFI signals. Never infer technical
ability from completion time, Advanced-details use, help use, or accessibility
controls.

UXP changes presentation only. It may not change permissions, authority,
required evidence, approval gates, action bindings, project scope, or safety
information. Every adaptation must be explainable and user-adjustable.
```

---

## 13. Acceptance criteria

Spec 135K is accepted when:

1. Generated UI uses canonical UXP/UFI rather than a parallel personalization model.
2. The default baseline is usable by a nontechnical operator without prior calibration.
3. Existing UXP dimensions control presentation and pacing as specified.
4. UFI records contain observable, cited interaction signals and exact scope.
5. One interaction cannot change permanent UXP behavior.
6. User overrides freeze learning per dimension.
7. Every adaptation is explainable and adjustable.
8. Safety, authority, evidence, privacy, and approval state cannot be hidden by adaptation.
9. `PlainLanguageProjection` uses UXP without inventing facts or errors.
10. Generated UI fixtures and Playwright tests cover required UXP variants.
11. The nontechnical completion benchmark passes.
12. No CLI, raw JSON, route, schema, or developer assistance is required for the default benchmark.

---

## 14. Closure blockers

This specification cannot close while:

- generated UI uses a separate nontechnical/expert-mode profile;
- explanation or confirmation preferences are stored only in client local storage;
- hidden inference changes permanent presentation;
- an emotion or technical-skill label is inferred;
- UFI signals lack citations or exact scope;
- one interaction changes UXP;
- a UXP variant hides required safety, evidence, authority, privacy, or approval information;
- adaptation requires a model call for deterministic layout/pacing choices;
- no real nontechnical evaluator completes the Alpha flow;
- a default user still requires CLI or raw technical details.
