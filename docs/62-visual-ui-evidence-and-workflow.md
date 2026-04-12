# Visual/UI Evidence and Workflow

## Purpose

Define the evidence chain and workflow layer for visual/UI work in Focusa.

This document sits above:
- `58-visual-ui-ontology-core.md`
- `59-visual-ui-reverse-engineering.md`
- `60-visual-ui-verification-and-critique.md`

This layer governs how visual/UI work progresses through:
- reference intake
- extraction
- planning
- build
- critique
- refinement
- completion

while preserving a durable evidence chain.

---

## Core Thesis

Visual/UI work should not be a vague loop of generate-and-hope.

It should be a structured working process with explicit evidence at every major step so Focusa can:
- preserve progress
- explain decisions
- compare iterations
- recover from wrong turns
- know when a design is actually ready

---

## Design Laws

1. Workflow must remain separate from ontology primitives.
2. Every major workflow step should emit evidence.
3. Each iteration should be comparable to both the reference and prior attempts.
4. The workflow should support both mimic/reverse-engineering and invention paths.
5. Workflow state should be resumable after interruption.
6. Findings and fixes must feed back into the working set.

---

# 1. Workflow Phases

## Phase 1: intake_reference
Capture and register source visual artifacts, specs, and constraints.

## Phase 2: derive_blueprint
Run reverse-engineering or invention planning to create the initial blueprint.

## Phase 3: implementation_plan
Translate the blueprint into an implementation-oriented plan.

## Phase 4: build_iteration
Produce a UI implementation attempt.

## Phase 5: verify_and_critique
Run comparison and critique against the target or quality criteria.

## Phase 6: refine
Apply prioritized fixes and update the working set.

## Phase 7: completion_review
Decide whether the current UI is complete enough to close the loop.

---

# 2. Workflow Objects

## VisualRun
Represents one end-to-end UI work run.

## VisualIteration
Represents a single build/review/refine cycle.

## EvidenceRecord
Represents a workflow-step evidence entry.

## VisualFix
Represents a remediation action derived from critique findings.

## CompletionReview
Represents the final readiness decision for a UI outcome.

---

# 3. Evidence Requirements by Phase

## intake_reference
- source artifacts
- source metadata
- target constraints

## derive_blueprint
- blueprint artifact
- extracted structure/components/tokens/slots
- confidence notes

## implementation_plan
- implementation plan artifact
- required plumbing summary
- dependency assumptions

## build_iteration
- built artifact
- implementation diff or summary
- produced component tree snapshot

## verify_and_critique
- comparison result
- critique result
- findings list
- prioritized fixes

## refine
- selected fixes
- applied changes
- updated iteration artifact

## completion_review
- final comparison result
- final critique result
- completion decision
- unresolved risks/open loops

---

# 4. Workflow Actions

## start_visual_run
Create a new visual run.

## register_reference_artifacts
Attach source artifacts and spec constraints.

## create_blueprint
Persist the initial blueprint.

## start_iteration
Open a new build/refine iteration.

## record_build_output
Attach current implementation evidence.

## record_comparison
Attach comparison results.

## record_critique
Attach critique results.

## synthesize_fixes
Create prioritized remediation actions.

## apply_fix_set
Record the chosen fixes for an iteration.

## close_visual_run
Close the run with completion evidence.

---

# 5. Completion Rules

A visual run should not be marked complete unless Focusa can show:
- the intended structure exists
- required interactions/states exist
- critical findings are resolved or explicitly accepted
- the output is evidence-backed
- unresolved risks/open loops are below threshold or explicitly recorded

---

# 6. Success Condition

Visual/UI Evidence and Workflow is successful when Focusa can move UI work through structured phases with durable evidence, resumability, and actionable refinement rather than vague iteration.
