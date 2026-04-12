# Visual/UI Integration with Focusa

## Purpose

Define how the Visual/UI layers integrate back into the broader Focusa system.

This document ties together:
- Domain-General Cognition Core
- Visual/UI Ontology Core
- Visual/UI Reverse Engineering
- Visual/UI Verification and Critique
- Visual/UI Evidence and Workflow
- Visual/UI Invention and Variation
- Visual/UI to Implementation

with existing Focusa runtime components.

---

## Core Thesis

The visual/UI stack should not become a sidecar system.

It should plug into the same Focusa machinery that already governs:
- missions
- goals
- decisions
- constraints
- working sets
- proposals
- reducer truth
- trace/checkpoints/recovery
- Pi context shaping

---

## Integration Points

## 1. Domain-General Cognition Core
Visual/UI work should project into:
- missions
- goals
- tasks
- decisions
- constraints
- risks
- blockers
- open loops
- working sets
- checkpoints

## 2. Reducer and Proposals
Visual/UI reverse-engineering and invention outputs should enter as:
- deterministic projections where possible
- proposal-level semantic enrichments where ambiguous
- reducer-promoted canonical state when verified

## 3. Working Sets and Slices
Focusa should support visual/UI working sets such as:
- reference_analysis_set
- component_design_set
- interaction_state_set
- refinement_set
- completion_review_set

## 4. Prompt / Context Assembly
Pi and other harnesses should receive bounded visual/UI slices when relevant, not raw visual dumps.

## 5. Trace / Recovery
Visual/UI work should emit:
- blueprint creation traces
- iteration traces
- critique/verification traces
- completion traces
- resume/checkpoint traces

## 6. Tool / Action Contracts
Visual/UI actions should eventually map to explicit contract surfaces just like software actions do.

---

## Success Condition

Visual/UI Integration with Focusa is successful when the visual/UI stack behaves like a first-class Focusa domain layer rather than an isolated subsystem.
