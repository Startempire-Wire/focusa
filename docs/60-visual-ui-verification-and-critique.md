# Visual/UI Verification and Critique

## Purpose

Define how Focusa evaluates whether a UI is:
- structurally correct
- visually faithful
- interactionally coherent
- state-complete
- responsive
- implementationally honest

This document sits above:
- `58-visual-ui-ontology-core.md`
- `59-visual-ui-reverse-engineering.md`

It defines the verification and critique layer for visual/UI work.

It does **not** define:
- workflow phase sequencing
- target-framework code generation
- WordPress-specific scoring rules
- orchestration logic for iterative build loops

---

## Core Thesis

A UI is not complete because it looks close.

A UI is complete only when Focusa can verify:
- the structure is correct
- the intended components and slots exist
- tokens and layout rules are respected
- interaction/state behavior is coherent
- responsive behavior is acceptable
- the built output meaningfully matches the intended visual world

Critique and verification should therefore be explicit, typed, and evidence-backed.

---

## Design Laws

1. Verification must operate on typed ontology objects, not vague prose.
2. Critique must produce structured findings, not only subjective commentary.
3. Comparison to reference and quality evaluation must be separate concerns.
4. Verification should support both reverse-engineered targets and newly invented designs.
5. Verification must include state, behavior, and responsiveness, not only static layout.
6. Findings must map to remediable actions.
7. Evidence must remain attached to every major verification result.

---

# 1. Verification Scope

Focusa should be able to verify across these categories:

## 1.1 Structural verification
- page structure present
- regions present and ordered correctly
- component tree matches intended structure
- required content slots exist

## 1.2 Token and layout verification
- expected token usage present
- spacing/layout rules respected
- alignment/grid behavior acceptable
- responsive overrides applied where required

## 1.3 Interaction and state verification
- required interactions exist
- state transitions are coherent
- empty/loading/error/success states exist where required
- disabled/active/hover/focus semantics are not omitted when required

## 1.4 Binding and validation verification
- required data bindings exist
- form and interactive plumbing is attached
- validation rules exist where required
- validation feedback is represented correctly

## 1.5 Visual fidelity verification
- built output resembles intended design at a meaningful level
- critical visual differences are detectable
- implementation drift is surfaced explicitly

## 1.6 Quality critique
- layout quality
- readability
- contrast/accessibility
- interaction clarity
- responsiveness
- cohesion/polish

---

# 2. Verification Objects

## ComparisonResult
Represents structured comparison output between target and built UI.

### Required properties
- `id`
- `comparison_kind`
- `status`
- `match_scope`

### Optional properties
- `target_artifact_id`
- `built_artifact_id`
- `confidence`
- `provenance`

---

## CritiqueResult
Represents structured quality evaluation.

### Required properties
- `id`
- `critique_kind`
- `status`

### Optional properties
- `overall_score`
- `quality_dimensions`
- `priority_findings`
- `provenance`

---

## VerificationFinding
Represents a concrete detected issue or confirmation.

### Required properties
- `id`
- `finding_kind`
- `severity`
- `status`

### Optional properties
- `target_ref`
- `expected_state`
- `observed_state`
- `remediation_hint`
- `evidence_refs`

---

## VerificationDimension
Represents a named verification or critique dimension.

### Required properties
- `id`
- `dimension_kind`
- `status`

### Optional properties
- `score`
- `weight`
- `threshold`

---

---

# 3. Verification Relations

## compares_against
Source:
- ComparisonResult

Target:
- VisualArtifact
- Page
- Region
- Component

---

## evaluates
Source:
- CritiqueResult
- VerificationDimension

Target:
- Page
- Region
- Component
- Interaction
- UIState

---

## finds_issue_in
Source:
- VerificationFinding

Target:
- Page
- Region
- Component
- Variant
- ContentSlot
- Token
- LayoutRule
- Interaction
- UIState
- Binding
- ValidationRule

---

## supported_by_evidence
Source:
- ComparisonResult
- CritiqueResult
- VerificationFinding

Target:
- VisualArtifact

---

## remediated_by
Source:
- VerificationFinding

Target:
- Component
- LayoutRule
- Token
- Interaction
- Binding
- ValidationRule

Use:
Represents the ontology-level object most likely to resolve the finding.

---

# 4. Verification Action Types

## verify_structure
Confirm page/region/component/slot structure matches target intent.

## verify_tokens
Confirm token use or token-family fidelity.

## verify_spacing
Confirm spacing scale, grid, alignment, and container rules.

## verify_interaction
Confirm interaction affordances and state transitions.

## verify_state_coverage
Confirm necessary UI states exist.

## verify_bindings
Confirm data/state bindings exist where required.

## verify_validation
Confirm validation and feedback behavior exist where required.

## compare_visual_fidelity
Compare built output against reference artifacts.

## critique_quality
Evaluate UI quality dimensions and produce structured critique.

## synthesize_priority_fixes
Convert findings into prioritized remediation actions.

---

# 5. Quality Dimensions

Focusa should critique UI using explicit dimensions.

Recommended base dimensions:
- `structure`
- `layout`
- `spacing`
- `contrast`
- `readability`
- `component_clarity`
- `interaction_clarity`
- `state_completeness`
- `responsiveness`
- `cohesion`

These dimensions are intentionally broader than any one product-specific rubric.

---

# 6. Comparison Modes

## 6.1 Reference fidelity mode
Used when there is a screenshot, mockup, or explicit target.

Questions:
- does the built output preserve intended regions?
- are major components and slots present?
- do tokens, spacing, and hierarchy resemble the reference?
- what important differences remain?

## 6.2 Spec fidelity mode
Used when the target is a requirement or design brief rather than a screenshot.

Questions:
- does the output satisfy the required layout and interaction semantics?
- are all important states present?
- is the UI coherent and finishable?

## 6.3 Invention quality mode
Used when the UI is novel and there is no direct reference.

Questions:
- is the layout coherent?
- are interactions understandable?
- is the design system consistent?
- is the implementation likely to be maintainable?

---

# 7. Finding Model

Every finding should include:
- severity
- target_ref
- expected
- observed
- remediation_hint
- evidence_refs

Recommended severities:
- `critical`
- `major`
- `moderate`
- `minor`
- `informational`

Recommended finding classes:
- `missing_region`
- `missing_component`
- `missing_slot`
- `token_mismatch`
- `spacing_mismatch`
- `alignment_issue`
- `interaction_missing`
- `state_missing`
- `binding_missing`
- `validation_missing`
- `responsive_issue`
- `fidelity_gap`
- `cohesion_issue`

---

# 8. Evidence Requirements

Verification should attach evidence such as:
- source screenshot
- built screenshot
- comparison diff artifact
- extracted blueprint
- token map
- spacing map
- interaction/state trace if available
- rendered implementation artifact

No major critique result should exist without evidence refs.

---

# 9. Output Model

A verification/critique run should produce:
- zero or more `ComparisonResult`s
- zero or more `CritiqueResult`s
- zero or more `VerificationFinding`s
- one or more `VerificationDimension`s if scoring is used
- prioritized remediation actions
- evidence refs
- confidence and provenance

---

# 10. What This Enables

With this layer, Focusa can:
- judge whether a built UI actually matches the intended visual world
- detect what is missing beyond static layout
- prevent “looks done but is not done” outcomes
- provide structured remediation guidance
- support screenshot fidelity work and novel UI invention equally

---

# 11. Success Condition

Visual/UI Verification and Critique is successful when Focusa can produce typed, evidence-backed judgments about UI correctness, fidelity, and quality that are actionable enough to drive the next implementation step.
