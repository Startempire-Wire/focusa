# Spec 107 — Spec-First Feature Lifecycle and Completion-Claim Discipline

Status: Draft  
Owner: Verious Smith  
Created: 2026-06-15  
Motivating incident: `focusa-ui0y.15` was over-closed from local/API/web proof even though actual macOS runtime evidence was unfinished; `focusa-bwky` prototype code was also implemented before a formal spec.

## 1. Purpose

Focusa must enforce a disciplined lifecycle for every new feature or materially new behavior:

```text
Idea → New Spec → bd/task decomposition → Implementation → tests/proofs → bd/task closure
```

This spec makes that sequence a hard product/process requirement, not a preference.

## 2. Problem

Two failure modes must be prevented:

1. **Spec bypass** — implementing new code before the idea is captured in a new or updated spec.
2. **Completion overclaim** — closing beads or writing final reports when evidence is partial, surrogate, local-only, blocked, or missing acceptance-critical runtime/platform proof.

Both failures damage Focusa reliability because they make the project state look more complete than reality.

## 3. Definitions

- **Idea**: an operator request, bug class, product concern, or proposed new capability.
- **New Spec**: a durable spec file or accepted spec amendment that states scope, non-goals, lifecycle gates, acceptance criteria, proof requirements, and closure rules.
- **bd/task decomposition**: beads created from the spec before implementation, including parent/child structure and dependencies.
- **Implementation**: code/docs/config/product changes made after the spec and bead decomposition exist.
- **Tests/proofs**: automated tests, live API/CLI/product proofs, screenshots/logs, artifact paths, or explicit blocker evidence that match the spec acceptance criteria.
- **bd/task closure**: closing only after matching evidence exists and the closure reason cites it.
- **Actual evidence**: evidence from the same runtime/platform/surface required by acceptance criteria.
- **Partial evidence**: useful but incomplete proof.
- **Surrogate evidence**: proof from a different surface than required, e.g. API/web proof for native Mac runtime.
- **Blocked evidence**: proof attempt failed because an environment or dependency boundary prevents validation.

## 4. Lifecycle Gate Requirements

### 4.1 Idea Gate

When an operator introduces a new feature/process/rule:

- Record the idea in scratch/Focusa context.
- Create or update a spec before implementation.
- Do not write production code for the new behavior until the spec exists.

### 4.2 Spec Gate

A spec or spec amendment must include:

- Problem statement.
- Scope and non-goals.
- Required lifecycle sequence.
- Acceptance criteria.
- Evidence/proof requirements.
- Closure policy.
- Known blockers and recovery path.
- Regression scenario when the spec is created because of a failure.

### 4.3 Decomposition Gate

Before implementation:

- Create a parent bead for the spec or spec slice.
- Create child beads for implementation, tests/proofs, docs, and release/closure as needed.
- Mark dependencies and blockers explicitly.
- A task may not be closed only because the parent goal is desirable; each acceptance criterion needs matching proof.

### 4.4 Implementation Gate

Implementation may start only when:

- The relevant spec exists.
- The relevant bd task exists and is `in_progress`.
- The implementation maps to a spec section or accepted amendment.

Prototype work done before the spec must be labeled **prototype**, cannot be used for closure, and must pass a spec-compliance review before being accepted.

### 4.5 Proof Gate

Proof must classify evidence as:

- `actual`
- `partial`
- `surrogate`
- `blocked`
- `missing`

Evidence metadata must include, when relevant:

- project root / continuity id
- work item id
- runtime/platform
- surface tested
- command/tool/session/artifact path
- result
- missing evidence
- blocker reason

Partial/surrogate/blocked evidence may be captured, but must not be reported as completion proof.

### 4.6 Closure Gate

Before `bd close` or a final completion report:

- Compare bead acceptance criteria against evidence classification.
- Block closure when required evidence is missing or only partial/surrogate/blocked.
- Closure reason must include `Evidence citations:` and stable proof refs.
- Final reports must state unfinished evidence plainly.

## 5. Anti-False-Claim Requirement

Focusa must provide a programmatic pre-close gate that rejects overclaims.

Minimum gate behavior:

- Input: work item id, claim text, acceptance criteria, evidence refs, evidence summaries.
- Output: decision `allow|block`.
- Output evidence class: `actual|partial|surrogate|blocked|missing`.
- Output missing evidence list.
- Output overclaim risks.
- Output recovery commands.

The Mac pairing regression must remain a fixture:

- Claim: “Mac menubar pairing E2E complete.”
- Acceptance: macOS `.app` launch, Keychain persistence, restart persistence, screenshots/logs, native Tauri runtime.
- Evidence: API/web-only pairing proof and local web build.
- Expected decision: `block`.

## 6. Tooling Surfaces

Required surfaces:

- Core evaluator for lifecycle/claim gate rules.
- API route for pre-close validation.
- CLI command for pre-close validation.
- Pi tool for agents before close/final report.
- Static/regression tests.
- Documentation for operator and agent behavior.

Optional later surfaces:

- Git hook / bd wrapper integration.
- Menubar/utility-card visibility.
- Work-loop automatic pause when closure gate blocks.

## 7. Non-Goals

- This spec does not require deleting prototype code immediately.
- This spec does not claim partial evidence is useless; it only prevents presenting partial evidence as completion.
- This spec does not replace operator judgment; it makes missing evidence explicit and blocks accidental overclaim.

## 8. Acceptance Criteria

Spec107 is accepted when:

1. This spec exists and is linked from README/tool docs where relevant.
2. A `bd` decomposition exists before additional implementation work.
3. A regression test blocks the Mac pairing API/web-only overclaim.
4. The pre-close gate classifies actual/partial/surrogate/blocked/missing evidence.
5. The gate is available through API, CLI, and Pi tool surfaces.
6. Closure/final-report workflow is updated or wrapped so agents are required to run the gate before claiming completion.
7. Prototype-before-spec work is labeled and reviewed before it can count as completion.

## 9. Closure Policy

Do not close Spec107 implementation beads until:

- All child beads are closed or explicitly blocked/deferred with operator-accepted rationale.
- Evidence citations include actual command/test/API/tool proof.
- Any remaining partial/surrogate/blocked evidence is named as unfinished, not completed.

