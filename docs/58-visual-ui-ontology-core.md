# Visual/UI Ontology Core

## Purpose

Define the smallest reusable ontology needed for Focusa to:
- reverse engineer UI from screenshots, mockups, or references
- imagine new layouts from product/spec constraints
- translate visual intent into implementation-ready structure
- preserve interaction, state, validation, and responsiveness

This document defines only the **semantic core**.

It does **not** define:
- workflow sequencing
- critique scoring systems
- target-framework implementation details
- WordPress-specific execution
- orchestration pipelines

---

## Core Thesis

A UI is a structured world, not just pixels.

Focusa should model that world explicitly so an agent can think inside it instead of reconstructing it from scratch each turn.

---

## Design Laws

1. Keep the ontology primitive.
2. Keep workflow separate from ontology.
3. Keep evaluation separate from ontology.
4. Keep target-framework execution separate from ontology.
5. Make evidence first-class.
6. Support both reverse engineering and invention.
7. Support implementation handoff, not just visual description.

---

# 1. Visual Object Types

## Page
Represents a full page-level surface or route-level experience.

### Required properties
- `id`
- `name`
- `page_kind`
- `primary_goal`
- `status`

---

## Region
Represents a meaningful layout region.

### Required properties
- `id`
- `name`
- `region_kind`
- `status`

### Examples
- header
- hero
- sidebar
- content column
- footer
- modal body
- form section

---

## Component
Represents a reusable UI unit.

### Required properties
- `id`
- `name`
- `component_kind`
- `status`

### Examples
- button
- card
- navbar
- form
- input
- table
- dialog
- accordion

---

## Variant
Represents a named variation of a component.

### Required properties
- `id`
- `name`
- `variant_kind`
- `status`

### Examples
- primary button
- secondary button
- compact card
- destructive dialog
- mobile nav variant

---

## ContentSlot
Represents a semantic content role inside a region or component.

### Required properties
- `id`
- `slot_kind`
- `status`

### Examples
- headline
- subheadline
- primary_cta
- secondary_cta
- media
- metadata
- helper_text
- error_text
- action_area

---

## Token
Represents a design token.

### Required properties
- `id`
- `token_kind`
- `value`
- `status`

### Examples
- color.primary
- spacing.4
- radius.md
- font.heading
- shadow.card

---

## LayoutRule
Represents reusable layout logic, spatial constraints, and responsive overrides.

### Required properties
- `id`
- `rule_kind`
- `status`

### Optional properties
- `grid_definition`
- `container_width`
- `alignment_rule`
- `spacing_pattern`
- `responsive_scope`

### Examples
- 12-column grid
- centered hero alignment
- stack spacing rule
- mobile stacked override
- desktop persistent sidebar

---

## Interaction
Represents a user-actionable behavior or transition trigger.

### Required properties
- `id`
- `interaction_kind`
- `status`

### Examples
- click CTA
- open modal
- submit form
- expand accordion
- navigate to route

---

## UIState
Represents a named UI state.

### Required properties
- `id`
- `state_kind`
- `status`

### Examples
- default
- hover
- active
- loading
- success
- error
- disabled
- empty

---

## Binding
Represents a semantic data or state connection between UI and underlying data/state sources.

### Required properties
- `id`
- `binding_kind`
- `status`

### Examples
- form field bound to profile.email
- table bound to results list
- button bound to mutation action
- badge bound to status enum

---

## ValidationRule
Represents a UI-visible validation or completion rule.

### Required properties
- `id`
- `rule_kind`
- `status`

### Examples
- required email
- password min length
- uniqueness check
- submission lock until valid

---

## VisualArtifact
Represents source material or evidence for visual reasoning.

### Required properties
- `id`
- `artifact_kind`
- `status`

### Examples
- screenshot
- mockup
- wireframe
- comparison diff image
- rendered output image
- blueprint export

---

# 2. Visual Relation Types

## contains
Source:
- Page
- Region

Target:
- Region
- Component
- VisualArtifact

---

## composed_of
Source:
- Component
- Variant
- Region

Target:
- Component
- Variant
- Token
- LayoutRule
- ContentSlot

---

## variants_of
Source:
- Variant

Target:
- Component

---

## fills_slot
Source:
- Component
- Variant
- ContentSlot

Target:
- ContentSlot
- Region
- Component

Use:
Represents semantic placement within a region or component structure.

---

## aligns_with
Source:
- Region
- Component
- LayoutRule

Target:
- Region
- Component
- LayoutRule

---

## inherits_token
Source:
- Component
- Variant
- Region

Target:
- Token

---

## binds_to
Source:
- Component
- Interaction

Target:
- Binding
- UIState
- ValidationRule

---

## transitions_to
Source:
- Interaction
- UIState

Target:
- UIState
- Page

---

## validates
Source:
- ValidationRule

Target:
- Binding
- Component
- UIState

---

## derived_from_reference
Source:
- Page
- Region
- Component
- Token
- LayoutRule
- ContentSlot

Target:
- VisualArtifact

---

# 3. Visual Action Types

## derive_structure
Infer page and region structure from a visual reference or spec.

## extract_components
Infer reusable component inventory from a visual reference.

## derive_slots
Infer semantic content slots inside regions and components.

## infer_tokens
Infer colors, typography, spacing, radius, shadow, and related tokens.

## infer_spacing
Infer spacing scale, grid structure, container rules, and alignment patterns.

## map_component_tree
Translate visual structures into an implementation-ready component tree.

## attach_bindings
Attach data/state bindings to visual structures.

## attach_validation
Attach validation behavior to relevant bound UI structures.

## wire_interaction
Attach interaction and state-transition semantics.

## compare_to_reference
Compare built output against a reference artifact.

## critique_ui
Evaluate the quality of a UI against explicit quality dimensions.

---

# 4. Visual Evidence Types

## screenshot
Captured visual reference or runtime output.

## mockup
Designed but not necessarily implemented visual source.

## wireframe
Low-fidelity structural visual plan.

## blueprint
Structured extraction result containing layout/component/token information.

## token_map
Derived or defined token set.

## spacing_map
Derived or defined spacing/grid system.

## component_inventory
Derived list of reusable components and variants.

## comparison_diff
Visual or semantic diff between target and built output.

## critique_result
Structured quality evaluation over visual objects.

## implementation_artifact
Rendered or code-produced visual realization.

---

# 5. Provenance and Confidence

Every visual object, relation, action result, or evidence record should carry:
- provenance source
- evidence refs
- confidence
- freshness
- verification status

Minimum provenance classes:
- screenshot_derived
- mockup_derived
- spec_derived
- model_inferred
- user_asserted
- verification_confirmed

---

# 6. What This Enables

With this ontology layer, Focusa can support:
- screenshot-to-component-tree reasoning
- UI reverse engineering with reusable semantic memory
- visual invention from constraints/specs
- implementation-ready layout planning
- responsive/stateful UI reasoning
- visual fidelity comparison and critique

But this document defines only the semantic core.

Workflow, evaluation systems, and execution orchestration must be specified separately.

---

# 7. Success Condition

The Visual/UI Ontology Core is successful when Focusa can represent visual interface structure in a primitive, reusable way that supports:
- reverse engineering
- invention
- implementation handoff
- verification

without collapsing workflow and execution logic into the ontology itself.
