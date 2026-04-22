# 87 — Focusa First-Class Tool Desirability and Pickup Spec

**Date:** 2026-04-22  
**Status:** active (spec-first)  
**Priority:** critical

## 1) Why this spec exists

Spec 81 delivered real tool surfaces, validation, runtime contracts, and high-order CLI workflows.

That solved correctness.

This spec raises the bar from **working tools** to **desirable tools**: tools that an LLM will willingly pick up because they clearly improve task outcomes versus guessing, manual chaining, or weak context inference.

---

## 2) Primary goal

Ship a first-class Focusa tool layer where the model sees the tools as:
- safe when they are read-only
- high-payoff when they compress workflow effort
- low-friction when ids or setup would otherwise block use
- clearly better than reasoning alone

---

## 3) Scope

## 3.1 In scope now

### A. Desirability-first upgrade of existing tool surfaces

Improve the current first-class tree/metacognition tools so they are more attractive to LLMs:
- stronger “when to use this” descriptions
- clearer safety/mutation signaling
- richer visible success summaries
- explicit next-step hints for chaining
- useful zero-result behavior where applicable

Applies to the existing core tool set:
- `focusa_tree_head`
- `focusa_tree_path`
- `focusa_tree_snapshot_state`
- `focusa_tree_restore_state`
- `focusa_tree_diff_context`
- `focusa_metacog_capture`
- `focusa_metacog_retrieve`
- `focusa_metacog_reflect`
- `focusa_metacog_plan_adjust`
- `focusa_metacog_evaluate_outcome`

### B. Low-friction helper and composite tools

Add new first-class tools that reduce setup cost and compress common workflows so the model can finish more in one move.

Required additions:
- helper tools for recent/latest ids where needed
- composite tools for high-value tree/metacognition flows

Minimum required composite coverage:
- one tree comparison helper/composite
- one metacognition loop/composite
- one metacognition doctor/diagnostic tool surface

### C. Pickup and outcome proof

Add tests that prove not only correctness, but desirability and actual use:
- prompt-level pickup smoke tests
- richer summary checks
- helper/composite tool runtime tests
- outcome-oriented evidence showing fewer steps or better completion quality

## 3.2 Out of scope

- unrelated daemon memory redesign
- unrelated service/process tuning
- replacing the existing CLI mission from Spec 81

---

## 4) Quality contract

Reject “done” if any of these are still true:
- tool descriptions explain mechanics but not payoff
- tools return correct data but weak visible summaries
- the model still has to hunt for ids too often before useful work can begin
- common tree/metacognition workflows still require too many separate tool calls
- no prompt-level evidence shows the model actually picks the tools up

---

## 5) Execution plan

### Phase 1 — Desirability audit + mapping
- map every current tool to payoff, safety, friction, and likely pickup blockers
- define helper/composite gaps

### Phase 2 — Improve existing tool desirability
- rewrite descriptions around use-case and payoff
- improve visible summaries
- add next-step hints and better zero-result guidance

### Phase 3 — Add low-friction helpers and composites
- recent/latest id helpers
- tree workflow helper/composite
- metacognition workflow helper/composite

### Phase 4 — Prove pickup and better outcomes
- runtime tests
- prompt-level pickup smoke tests
- completion/effectiveness evidence

### Phase 5 — Signoff packet
- line-cited completion matrix
- examples of improved model-facing value
- evidence linking pickup and usefulness claims to executable proof

---

## 6) Acceptance criteria

Done means all are true:
1. existing first-class tools are rewritten to be more obviously useful to LLMs
2. helper/composite tools reduce friction and workflow length in real scenarios
3. visible tool output gives materially better decision support than terse status strings
4. prompt-level pickup tests pass
5. evidence shows the tool layer is not just correct, but more attractive for better work outcomes
