# Spec 135K — UXP/UFI Adaptive Generated UI, Friction Learning, and Nontechnical Usability

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 14](14-uxp-ufi-schema.md), [Spec 135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md), [Spec 135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md), [Spec 135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md), and [Spec 135J](135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135K.  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) governs any conflict.

---

## 0. One-line definition

Focusa’s generated C.R.I.S.T. UI MUST begin with a safe nontechnical baseline and adapt explanation depth, pacing, optional confirmation density, review cadence, and presentation through the existing transparent UXP/UFI system rather than creating a second user mode, expertise score, or hidden personalization engine.

---

## 1. Reuse law

[Spec 14](14-uxp-ufi-schema.md) is authoritative.

```text
UXP
  Slow-moving, confidence-weighted, cited, reversible preferences.

UFI
  Fast-moving, per-interaction, observable friction records.
```

Forbidden:

- `NontechnicalUserProfile`;
- `SimpleModeProfile`;
- `ExpertiseScore`;
- hidden technical-skill inference;
- emotion or personality labels;
- a second explanation/confirmation store;
- client-local permanent personalization.

---

## 2. Default nontechnical baseline

Before sufficient UXP evidence exists, every Project Genesis and C.R.I.S.T. surface uses:

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
no hidden safety or authority state
```

Adaptation MUST NOT remove safety, authority, Evidence, privacy, recovery, or approval information.

---

## 3. Canonical dimensions

Use existing Spec 14 dimensions:

```text
verbosity_preference
explanation_depth
confirmation_preference
interruption_sensitivity
review_cadence
risk_tolerance
autonomy_tolerance
```

These dimensions modify presentation and advisory pacing only. They MUST NOT modify canonical correctness, required Evidence, permissions, Workpoint scope, provider authority, or approval gates.

---

## 4. Generated Surface integration

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

Resolution:

```text
canonical stage/read model
+ capabilities and authority
+ Workspace View Profile and domain packs
+ UXP projection
+ client and accessibility capabilities
→ UiInteractionIntent
→ A2UI surface
```

---

## 5. UFI capture

Generated UI records only observable Spec 14 friction signals.

High-value:

```text
manual_override
immediate_correction
undo_or_revert
explicit_rejection
task_reopened
```

Medium-value:

```text
rephrase
repeat_request
scope_clarification
forced_simplification
```

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

Completion time, opening Advanced details, requesting help, or using accessibility controls MUST NOT be interpreted as friction or expertise.

---

## 6. Adaptation rules

### Slow and reversible

UXP changes follow Spec 14 trend windows, confidence, citations, bounded alpha, and user-override rules. One interaction never changes permanent behavior.

### Transparent

The UI MUST answer:

```text
Why is Focusa presenting this this way?
What interactions informed it?
How confident is it?
How can I change it?
```

### User override

The operator can set explanation depth, summary length, optional confirmation frequency, review cadence, and optional interruption timing. An override freezes learning for that dimension until released.

### Safety invariants

UXP/UFI MUST NOT:

- hide required approval;
- remove risk or contradiction disclosure;
- suppress Evidence requirements;
- skip readiness gates;
- change connector/provider permissions;
- grant autonomy;
- auto-accept recommendations;
- weaken accessibility semantics;
- change canonical action bindings.

---

## 7. Stage adaptation

### Context

Adapt explanation length for permissions, source scope, privacy, retention, and claims. Import scope, credentials, and retention consequences remain explicit.

### Role

Adapt summary/detail presentation. Responsibility and permission remain visibly separated.

### Interview

Adapt recommendation depth, examples, and branch-progress detail. Always show one primary question. Interruption sensitivity defers optional follow-ups, not blocker questions.

### Spec

Adapt section summaries and objection depth. Approval consequences remain explicit.

### Tasks

Adapt graph summary/detail. Acceptance, Evidence, blockers, provider destination, and mutation preview remain accurate and accessible.

---

## 8. PlainLanguageProjection

Consumes:

```text
canonical terminology
Workspace View Profile terminology
domain packs
approved glossary
ToolResult/error envelope
UXP dimensions
client/accessibility capabilities
```

Produces user-facing labels, explanations, recommendation summaries, consequences, recovery copy, advanced labels, and help. It MUST NOT invent facts or reinterpret failure classes.

---

## 9. API integration

```text
GET  /v1/uxp/profile
GET  /v1/uxp/profile/explain
POST /v1/uxp/profile/override/preview
POST /v1/uxp/profile/override/commit
GET  /v1/ufi/recent
POST /v1/ufi/record
```

Existing equivalent routes MUST be extended rather than duplicated. UXP/UFI uses the canonical local SQLite model and the generated Operation Registry.

---

## 10. Speed and reuse

- Reuse Spec 14 storage and reducers.
- Derive UFI from typed Focusa actions, corrections, reversals, and outcomes.
- Use deterministic catalog variants and design tokens for UXP presentation.
- Do not call an LLM to choose font size, expansion state, confirmation count, or summary length.
- Maintain deterministic A2UI fixtures for baseline and each required UXP variant.

Required fixture matrix:

```text
new-user baseline
high explanation depth
low verbosity
high interruption sensitivity
high review cadence
explicit overrides
low-confidence UXP
conflicting scope-specific UXP
```

---

## 11. Testing and usability proof

Required:

- reducer/property tests for bounded, reversible learning;
- citation and confidence tests;
- override-freeze tests;
- user/agent/model/harness scope separation;
- A2UI generated-surface fixtures for UXP variants;
- Vitest and Svelte Testing Library interaction tests;
- UIAI Engine Eval scenarios proving identical tasks remain completable across variants;
- UIAI Engine Eval browser accessibility, responsive, visual, reconnect, and recovery proof;
- tests proving safety and approval controls cannot be hidden;
- nontechnical evaluator study using the Cross-Functional Alpha;
- friction analysis based on cited observable behavior.

Focusa MUST NOT use Playwright for UXP/UFI proof.

### Nontechnical completion benchmark

A new evaluator with no Focusa vocabulary MUST complete:

```text
start Project Genesis
add local Context
understand and approve a Role
answer a Grill question
understand Spec progress
approve a task plan
start the first Workpoint
recover from one connector or validation failure
close the client
resume exact state
```

The evaluator can use generated help. They MUST NOT require CLI, raw JSON, schema, route names, or developer intervention.

---

## 12. Agent directive

```text
Use Spec 14 UXP/UFI for all generated-UI adaptation. Do not create a simple
mode, expert mode, expertise score, emotion model, or second personalization
store.

Start every C.R.I.S.T. surface from the safe nontechnical baseline. Adapt only
presentation and advisory pacing through cited, confidence-weighted, reversible
UXP dimensions. Record only observable UFI signals.

UXP never changes permissions, authority, Evidence, approvals, scope, action
bindings, or safety information. Every adaptation is explainable and adjustable.

Use A2UI fixtures, Vitest, and Svelte Testing Library for deterministic proof.
Use UIAI Engine Eval for every browser, responsive, visual, reconnect, recovery,
and browser-accessibility scenario. Do not add Playwright.
```

---

## 13. Acceptance criteria

Spec 135K is accepted when:

1. Generated UI reuses canonical UXP/UFI.
2. The default baseline is usable without prior calibration.
3. Existing UXP dimensions control presentation and pacing only.
4. UFI records contain observable, cited signals and exact scope.
5. One interaction cannot change permanent UXP behavior.
6. User overrides freeze learning per dimension.
7. Every adaptation is explainable and adjustable.
8. Safety, authority, Evidence, privacy, and approvals cannot be hidden.
9. PlainLanguageProjection uses UXP without inventing facts.
10. A2UI fixtures and UIAI Engine Eval cover all required variants.
11. The nontechnical completion benchmark passes.
12. No Playwright dependency, fixture, or config exists.

## 14. Closure blockers

Spec 135K cannot close while:

- generated UI uses a separate simple/expert profile;
- preferences exist only in local storage;
- hidden inference changes permanent presentation;
- emotion or technical-skill labels are inferred;
- UFI lacks citations or exact scope;
- one interaction changes UXP;
- safety or approval controls can be hidden;
- browser proof bypasses UIAI Engine Eval;
- Playwright exists in the Spec 135 implementation path;
- the nontechnical benchmark requires raw technical interfaces.
