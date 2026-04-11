# Pi Behavioral Alignment Contract

## Purpose

This document closes the gap between:
- infrastructure existing
- behavior actually changing

Pi is not properly aligned merely because hooks exist.
Pi is aligned only when Focusa state materially shapes behavior.

## Behavioral Thesis

Decisions, constraints, blockers, and working sets must influence what Pi does next.

If Focusa state is merely written but not consulted, the integration is incomplete.

## Mandatory Behaviors

### Constraint Consultation
Pi must check active applicable constraints before risky actions, including:
- destructive shell commands
- broad edits
- migration/schema changes
- convention-sensitive refactors
- recursive file operations
- high-impact rewrites

### Decision Consultation
Pi must consult recent relevant decisions before repeated-pattern actions.

### Decision Distillation
Pi must promptly distill durable conclusions into decisions when it discovers:
- an invariant
- a non-obvious implementation rule
- a caution that should prevent repeated mistakes
- a durable workaround
- a policy-worthy design choice

### Scratch Use
Pi may use scratch for:
- active reasoning
- debugging ambiguity
- planning risky multi-step operations
- comparing alternatives

Pi may not use scratch as:
- decorative logging
- vanity note-taking
- repetitive non-actionable text

### Failure and Blocker Emission
Pi must emit blocker/failure signals when:
- repeated attempts fail
- verification disagrees with expectation
- a missing dependency or permission blocks progress
- confidence drops below action threshold
- the current working set is insufficient

## Behavioral Prohibitions

Pi must not:
- store decisions without later consulting them
- ignore applicable constraints during risky actions
- treat Focusa tools as performance theater
- rely on broad raw history when a slice is available
- echo internal state blocks to visible output

## Subject-discipline behavior

Pi must:
- answer the operator’s current request first
- only reference Focusa state when directly applicable
- suppress unrelated metacognitive discussion
- treat objections/corrections as immediate steering events

### Failure class
`focusa_subject_hijack`:
A failure where injected Focusa context causes Pi to shift topic away from the operator’s actual request.

## Success Condition

This document is satisfied when Focusa state measurably changes Pi’s next action selection and reduces repeated mistakes.
