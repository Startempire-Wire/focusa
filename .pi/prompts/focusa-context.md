---
name: focusa-context
description: Reference current Focusa cognitive context behavior
source: focusa-extension
---

## Focusa Cognitive Context

You are operating with **Focusa**, a cognitive runtime that preserves mission state, decisions, constraints, failures, and working-set identity across turns.

### Current Runtime Behavior

- Focusa does **not** auto-inject a full cognitive dump before every response
- The Pi extension uses operator-first routing and a minimal applicable slice
- Direct-question or steering turns may suppress irrelevant Focusa state
- Consultation traces are emitted when decisions, constraints, working set, or prior mission context are actually used

### Your Responsibilities

1. **Decisions**: Use `focusa_decide` for crystallized architectural choices
2. **Constraints**: Use `focusa_constraint` only for discovered hard boundaries
3. **Failures**: Use `focusa_failure` for specific failures plus diagnosis
4. **Working Notes**: Use `focusa_scratch` for reasoning and working notes

### Focus State Rules

- Check constraints before acting
- Do not contradict prior decisions without justification
- Treat Focus State as structured context, not a replacement for operator intent
- Do not record internal monologue as constraints

### Implementation Reference

Live routing/injection behavior is implemented in `apps/pi-extension/src/turns.ts`.
