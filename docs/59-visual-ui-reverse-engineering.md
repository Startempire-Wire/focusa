# Visual/UI Reverse Engineering

## Purpose

Define how Focusa converts visual references into a typed Visual/UI Ontology world.

This document sits above `58-visual-ui-ontology-core.md`.

It specifies the reverse-engineering process for turning:
- screenshots
- mockups
- wireframes
- reference pages
- rendered UI artifacts

into:
- page structure
- regions
- component inventory
- content slots
- design tokens
- layout rules
- interactions
- UI states
- bindings and validation candidates
- evidence records

This document still does **not** define:
- workflow phase gates
- critique scoring systems
- target-framework code generation rules
- WordPress-specific execution details

---

## Core Thesis

Reverse engineering UI is not “look at an image and guess.”

It is a structured extraction process that progressively derives:
1. structure
2. components
3. semantic slots
4. tokens
5. spacing/layout rules
6. interactions and state
7. implementation-relevant semantics
8. evidence and comparison hooks

---

## Design Laws

1. Extraction should move from coarse structure to finer semantic detail.
2. Each stage should emit typed ontology proposals, not freeform prose.
3. Weak or ambiguous inferences must remain proposal-level until verified.
4. Visual evidence must remain attached throughout the pipeline.
5. Extraction must support both pixel-faithful reference analysis and semantically abstract blueprint creation.
6. Reverse engineering should produce implementation-relevant structures, not only descriptions.

---

# 1. Reverse Engineering Inputs

## Supported input artifact kinds
- screenshot
- mockup
- wireframe
- rendered page capture
- multi-breakpoint reference set
- partial crop / component crop

## Input requirements
Every reverse-engineering run should capture:
- `artifact_id`
- `artifact_kind`
- `source_ref`
- `capture_context`
- `viewport_context` if known
- `provenance`

---

# 2. Reverse Engineering Stages

## Stage 1: derive_structure

### Goal
Infer high-level page and region structure.

### Inputs
- one or more `VisualArtifact`s

### Outputs
- `Page`
- `Region`s
- `contains` relations
- preliminary `LayoutRule` candidates

### Questions answered
- What kind of page is this?
- What major regions exist?
- What is the region order/hierarchy?
- What is the coarse layout pattern?

### Example outputs
- hero region
- nav/header region
- 2-column content region
- footer region
- modal overlay region

---

## Stage 2: extract_components

### Goal
Infer the reusable component inventory.

### Inputs
- page/region structure
- source visual artifacts

### Outputs
- `Component`s
- `Variant`s
- `composed_of` relations
- `variants_of` relations

### Questions answered
- What reusable UI units exist?
- Which are distinct components versus repeated instances?
- Which variants appear?

### Example outputs
- button component with primary/secondary variants
- card component
- accordion component
- form/input component family
- navigation component

---

## Stage 3: derive_slots

### Goal
Infer semantic content roles inside regions and components.

### Inputs
- regions
- components
- source visual artifacts

### Outputs
- `ContentSlot`s
- `fills_slot` relations
- slot usage proposals

### Questions answered
- Where is headline vs subheadline?
- Which component area acts as metadata or media?
- Where are primary and secondary actions?
- What semantic zones does this component expose?

### Example outputs
- hero headline slot
- card media slot
- form helper text slot
- dialog action area slot

---

## Stage 4: infer_tokens

### Goal
Infer design-token candidates from the visual reference.

### Inputs
- source visual artifacts
- regions/components/slots

### Outputs
- `Token`s
- `inherits_token` relations
- token maps

### Token classes
- color
- typography
- spacing
- radius
- shadow
- border
- sizing

### Questions answered
- What is the primary color system?
- What typography hierarchy exists?
- What radius/shadow system exists?
- Which components or regions appear to inherit which tokens?

---

## Stage 5: infer_spacing

### Goal
Infer layout and spacing rules.

### Inputs
- structure
- components
- token candidates
- visual artifacts

### Outputs
- `LayoutRule`s
- alignment relationships
- spacing-scale candidates
- responsive layout-rule candidates

### Questions answered
- What grid or alignment system is used?
- What container width behavior exists?
- What vertical rhythm and spacing scale exist?
- What responsive shifts appear across breakpoints?

---

## Stage 6: infer_interaction_and_state

### Goal
Infer visible interaction/state semantics from the reference.

### Inputs
- components
- slots
- variant candidates
- reference artifacts

### Outputs
- `Interaction`s
- `UIState`s
- `transitions_to` relations
- `binds_to` candidates
- `ValidationRule` candidates

### Questions answered
- What looks interactive?
- What states are visible or implied?
- Where are loading, error, empty, active, disabled, or success states implied?
- Where do form fields imply validation?

---

## Stage 7: derive_implementation_semantics

### Goal
Produce implementation-relevant structure without generating framework-specific code yet.

### Inputs
- structure
- components
- slots
- tokens
- layout rules
- interactions/states

### Outputs
- implementation-ready component tree
- binding candidates
- validation candidates
- responsive requirements
- required evidence hooks for later comparison

### Questions answered
- What component tree would implement this faithfully?
- Which elements require data binding?
- Which elements require validation plumbing?
- Which layout behaviors must be preserved in implementation?

---

# 3. Output Model

A reverse-engineering run should produce a typed blueprint containing:
- extracted page/regions
- extracted components/variants
- extracted slots
- token map
- spacing/layout map
- interaction/state map
- implementation-semantics summary
- evidence refs
- confidence by extraction stage

This blueprint should be storable as:
- ontology proposals
- blueprint artifact
- comparison baseline for later verification

---

# 4. Confidence and Promotion Rules

## Deterministic vs inferred
Some outputs may be stronger than others.

### Higher-confidence outputs
- explicit visible regions
- repeated components
- obvious tokens such as dominant colors
- visible states like disabled/error/loading

### Lower-confidence outputs
- hidden interactions
- inferred validation rules
- unseen responsive behavior from a single screenshot
- invisible data bindings

## Rule
Low-confidence outputs must remain proposal-level until:
- multi-artifact evidence confirms them
- implementation behavior confirms them
- operator review confirms them
- comparison/verification later promotes them

---

# 5. Required Evidence

Every reverse-engineering run should preserve:
- source screenshots/mockups
- blueprint artifact
- extracted token map
- extracted spacing map
- component inventory
- provenance metadata
- confidence by object/relation/action result

This allows later:
- comparison to built output
- critique
- refinement
- auditability

---

# 6. Failure Modes

The reverse-engineering layer should explicitly detect and surface:
- ambiguous component boundaries
- ambiguous slot assignments
- multiple plausible layout interpretations
- uncertain token inference
- insufficient evidence for responsive behavior
- insufficient evidence for binding/validation semantics
- low-fidelity or partial screenshots

These should generate:
- uncertainty markers
- proposal-only outputs
- missing-evidence blockers
- requests for more artifacts if needed

---

# 7. What This Enables

With this layer, Focusa can:
- turn screenshots into semantic UI blueprints
- reverse engineer existing interfaces into reusable mental structure
- carry visual/UI understanding across interruption
- provide better implementation handoff to coding agents
- establish a reference baseline for later critique and fidelity comparison

---

# 8. Success Condition

Visual/UI Reverse Engineering is successful when Focusa can turn a visual reference into a typed, evidence-backed, implementation-relevant ontology blueprint without collapsing into freeform image description or target-specific workflow logic.
