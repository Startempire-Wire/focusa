# Visual/UI Invention and Variation

## Purpose

Define how Focusa supports imaginative UI work, not only reverse engineering.

This document covers:
- generating new layouts from goals/specs
- exploring multiple viable interface directions
- preserving constraints while inventing
- producing variant options without losing implementation discipline

It sits above the Visual/UI ontology core and below target-framework execution.

---

## Core Thesis

A strong developer or designer does not only recreate what already exists.

They can imagine:
- new layout directions
- new interaction patterns
- new component compositions
- new visual systems

while still preserving:
- product intent
- usability
- implementation realism
- completion discipline

Focusa should support invention as a first-class operation over the visual ontology.

---

## Design Laws

1. Invention must remain constrained, not arbitrary.
2. Variations should share an explicit semantic core where possible.
3. Novelty should not destroy implementation realism.
4. Variations should be comparable against one another.
5. Invention should produce ontology objects, not just pretty descriptions.

---

# 1. Invention Inputs

Focusa should support invention from:
- product spec
- mission/goal
- brand constraints
- accessibility constraints
- functional requirements
- prior reference patterns
- existing design system or token system

---

# 2. Invention Objects

## VisualConcept
Represents a proposed visual direction.

## LayoutOption
Represents one candidate structural layout.

## InteractionOption
Represents one candidate interaction pattern.

## VariationSet
Represents a grouped set of viable alternatives.

## SelectionRationale
Represents why one option was chosen over another.

---

# 3. Invention Actions

## propose_layout_options
Generate multiple candidate layout directions from the same constraints.

## propose_interaction_options
Generate multiple candidate interaction patterns.

## vary_component_composition
Generate alternate component compositions for a target region/page.

## vary_token_application
Generate alternate visual styling directions while preserving semantics.

## compare_variations
Compare candidate options on quality, feasibility, and fit.

## select_direction
Choose one option and record rationale.

---

# 4. Required Invention Outputs

Every invention pass should produce:
- one or more `VisualConcept`s
- one or more `LayoutOption`s
- optional `InteractionOption`s
- explicit constraints carried forward
- implementation feasibility notes
- selection rationale if narrowed

---

# 5. Success Condition

Visual/UI Invention and Variation is successful when Focusa can generate, compare, and narrow UI options creatively without losing constraints, implementation realism, or completion discipline.
