# 89 — Focusa Tool Suite Improvement and Hardening Spec

**Date:** 2026-04-28
**Status:** proposed
**Priority:** critical
**Bead:** `focusa-d0mv`
**Owner:** Focusa + Pi integration

---

## 1) Why this spec exists

Focusa now has a broad first-class Pi tool surface for Focus State, Workpoint continuity, tree/snapshots, metacognition, lineage intelligence, and continuous work-loop control.

The tools are functional and broadly aligned with the existing specs, but the suite is not yet uniformly **obvious, safe, evidence-linked, Workpoint-aware, or compound-learning optimized**.

This spec combines:

1. the missing parts identified in the plain-language Focusa purpose audit, and
2. prior tool improvement suggestions from the tool usefulness/rating review.

The result is a single hardening plan for making Focusa tools feel less like separate utilities and more like one coherent cognitive operating layer for agents.

---

## 2) Source alignment

This spec is grounded in the following existing contracts:

- `docs/44-pi-focusa-integration-spec.md`
  - Focusa is the single cognitive authority.
  - Pi extension is thin UX/observability glue.
  - No local cognitive DB in Pi.
  - Focus State writes must distinguish offline, no-active-frame, validation rejection, and write failure.
  - Critical write fallback goes to scratchpad as evidence, not canonical state.

- `docs/52-pi-extension-contract.md`
  - Pi is a disciplined harness-side adapter, not a parallel cognitive system.
  - Pi consumes bounded ontology slices and emits typed proposals/signals.

- `docs/55-tool-action-contracts.md`
  - Every important tool/action needs typed inputs, typed outputs, side effects, failure modes, idempotency, verification hooks, retry policy, degraded fallback, and recovery semantics.

- `docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md`
  - Tool suite must be high-quality, first-class, discoverable, reliable, typed, and useful for real workflows.
  - CLI should be first-class for the same workflows.
  - Metacognition should support compound learning loops.

- `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`
  - Tools must be desirable to agents: clear payoff, low friction, visible summaries, next-step hints, helper/composite workflows, and pickup evidence.

- `docs/88-ontology-backed-workpoint-continuity.md`
  - Focusa preserves typed Workpoint continuity, not raw transcript tail.
  - Workpoint checkpoint/resume are canonical continuation primitives.
  - API, CLI, and Pi tool semantics must align.

---

## 3) Core thesis

Focusa tools should not behave like isolated helper functions.

They should collectively make agent work:

- focused
- typed
- evidence-backed
- recoverable
- Workpoint-aware
- reducer-compatible
- safe under degradation
- easier to resume after compaction or model/session changes
- better over time through metacognitive compounding

In plain language:

> Focusa tools should make an agent work like a disciplined teammate with durable memory, proof habits, recovery checkpoints, and learning loops — not like a chatbot relying on transcript luck.

---

## 4) Success outcomes

The hardening work is successful when agents reliably:

1. know the active mission and current Workpoint before acting,
2. pick the right Focusa tool without hunting or guessing,
3. record decisions, constraints, failures, evidence, and next steps in the right semantic slots,
4. snapshot/checkpoint before risky or ambiguous work,
5. resume after compaction/model switch/session restart from canonical Workpoint state,
6. distinguish canonical Focusa state from degraded local fallback,
7. attach evidence to claims instead of making unsupported “done” statements,
8. avoid duplicate append-only cognitive writes,
9. close or decay stale state,
10. use metacognition to improve future behavior, not just document current work.

---

## 5) Current tool-family assessment

### 5.1 Strongest areas

| Area | Current strength |
|---|---|
| Workpoint checkpoint/resume | Best alignment with Focusa purpose; directly solves compaction/resume loss. |
| Focus State decisions/failures/constraints | Strong durable cognitive memory with validation. |
| Snapshot/tree tools | Strong safety and recoverability layer. |
| Metacog composites | Strong compound-learning potential. |
| Helper tools for recent ids | Good Spec87 low-friction pattern. |

### 5.2 Weakest areas

| Area | Weakness |
|---|---|
| `focusa_note` | Semantically loose; can become junk-drawer state. |
| Work-loop control | Powerful but unclear; writer-claim semantics are not agent-obvious. |
| Metacog adjust/evaluate | Valuable but ceremonial; agents need better guided metrics and promotion criteria. |
| Append-only writes | Duplicate risk on retry or uncertainty. |
| Evidence linking | Too much depends on agent manually deciding what to link. |
| Tool result shape | Useful text exists, but result envelopes are not uniformly typed across all tools. |

---

## 6) Required hardening themes

### 6.1 Unified tool result envelope

Every Focusa Pi tool should return a predictable machine-readable envelope in `details`, regardless of human-visible text.

Minimum shape:

```ts
type FocusaToolResult = {
  status: "accepted" | "completed" | "partial" | "unknown" | "rejected" | "blocked";
  canonical: boolean;
  degraded: boolean;
  tool: string;
  action_type?: string;
  target_refs?: string[];
  affected_refs?: string[];
  evidence_refs?: string[];
  side_effects?: string[];
  verification?: { verified: boolean; method?: string; refs?: string[] };
  retry?: { safe: boolean; reason: string; idempotency_key?: string };
  next_tools?: string[];
  next_action_hint?: string;
  warnings?: string[];
  error_code?: string;
};
```

Requirements:

- Read-only tools should mark `canonical=true`, `degraded=false`, and `side_effects=[]` unless telemetry writes occur.
- Degraded fallback must never be silently canonical.
- Append-only writes must mark retry as unsafe unless dedupe/idempotency is present.
- Timeout or ambiguous completion must return `status="unknown"`, not `failed`, unless state verification proves no side effect.

### 6.2 Workpoint spine integration

Workpoint should become the organizing spine of the entire suite.

Every stateful or evidence-producing tool should attempt to attach its result to the active Workpoint when relevant.

| Tool family | Workpoint linkage |
|---|---|
| Focus State writes | Link decisions/constraints/failures/results to active mission/action/object set when available. |
| Snapshot tools | Attach snapshot ids to active Workpoint as recovery/evidence refs. |
| Tree/path/diff tools | Attach lineage/diff refs when used for a Workpoint decision or verification. |
| Metacog tools | Attach learning signals/reflections to active mission/work item/component when scoped. |
| Work-loop tools | Sync current ask/scope/checkpoint with Workpoint mission/next slice. |
| Lineage intelligence tools | Promote extracted decisions/risks into Workpoint candidates. |

Implementation requirements:

- Add internal helper: `resolveActiveWorkpointContext()`.
- Add optional `attach_to_workpoint?: boolean` to relevant tool requests, default true for evidence-producing tools.
- Return `workpoint_linked: true|false` with reason.
- Never let Pi tool linkage canonize ontology truth directly; use reducer/API-approved paths.

### 6.3 Evidence autopromotion and handle discipline

Focusa should reduce reliance on the agent manually deciding what counts as evidence.

Required behavior:

1. Large tool outputs become ECS/reference handles.
2. Validation/test/API/CLI results can become concise evidence records.
3. Evidence records link to active Workpoint, active object, or Focus State result.
4. Visible summaries include proof status.

New helper capability:

```ts
focusa_evidence_capture
```

Purpose:

- Store a concise evidence conclusion plus optional raw handle.
- Link evidence to Workpoint, object, file, test, or failure.

Potential inputs:

```ts
{
  summary: string;
  evidence_type: "test" | "api" | "cli" | "diff" | "screenshot" | "log" | "manual_observation";
  target_refs?: string[];
  raw_handle?: string;
  command?: string;
  status: "passed" | "failed" | "partial" | "unknown";
  attach_to_workpoint?: boolean;
}
```

Acceptance:

- Evidence capture returns stable evidence ref.
- Workpoint resume packets prefer evidence refs over raw output.

### 6.4 Tool-suite doctor

Add one obvious diagnostic entrypoint for the whole Focusa tool layer.

New composite tool:

```ts
focusa_tool_doctor
```

Purpose:

Diagnose whether Focusa is currently safe and useful to trust from Pi.

Checks:

- daemon health
- active Focusa frame
- Focus State write path
- active Workpoint canonical/degraded status
- Workpoint resume availability
- work-loop status and active writer claim
- snapshot availability
- metacog retrieval health
- lineage path responsiveness
- CLI/API/Pi parity smoke
- recent Focusa tool failures

Acceptance:

- Doctor does not mutate canonical state by default.
- Doctor identifies false-offline bridge cases distinctly from real daemon outage.
- Doctor recommends one safe next action.

### 6.5 Active object resolver

Agents need a low-friction way to know what they are currently targeting.

New helper tool:

```ts
focusa_active_object_resolve
```

Purpose:

Resolve current mission/Workpoint/focus/bead into active objects and likely files/endpoints/tests.

Outputs:

- active Workpoint id
- active mission
- active object refs
- likely files
- likely tests
- likely endpoints
- evidence gaps
- recommended next tool/action

Acceptance:

- If no active object exists, tool returns a clear setup path instead of empty JSON.
- Tool never invents object refs; uncertain results are marked proposed/unverified.

### 6.6 Work-loop UX hardening

Work-loop tools need stronger agent-facing guidance because they control ongoing autonomous work.

Requirements:

- Add preflight mode to control actions.
- Show active writer and claim reason before mutation.
- Distinguish blocked-by-writer, blocked-by-policy, blocked-by-dirty-worktree, and daemon-degraded.
- Return safe available actions.
- Make `focusa_work_loop_checkpoint` explain how it differs from Workpoint checkpoint.
- Make `select_next` explain missing parent/root bead resolution.

Potential additions:

```ts
focusa_work_loop_preflight
focusa_work_loop_writer_status
```

### 6.7 Metacognition quality guardrails

Metacog tools should compound real intelligence, not create vague lessons.

Requirements:

- Reject or warn on vague reflection outputs.
- Require evidence refs or turn refs for durable adjustments.
- Add confidence thresholds and decay for weak learnings.
- Auto-suggest metrics for `evaluate_outcome` from tests/results/failures.
- `metacog_loop_run` should include a quality gate before creating an adjustment.
- Doctor should recommend capture/retrieve/reflect/adjust/evaluate next step based on signal quality.

### 6.8 Dedupe and idempotency for cognitive writes

Append-style cognitive writes are valuable but non-idempotent.

Affected tools:

- `focusa_decide`
- `focusa_constraint`
- `focusa_failure`
- `focusa_next_step`
- `focusa_open_question`
- `focusa_recent_result`
- `focusa_note`
- `focusa_scratch`

Requirements:

- Add optional `idempotency_key` where supported.
- Add semantic duplicate detection against recent Focus State slots.
- Return `duplicate_candidate=true` when similar entry exists.
- Offer merge/update/supersede path for supported slots.
- Never auto-dedupe silently; report what happened.

### 6.9 Stale-state hygiene lifecycle

Focus State needs lifecycle management.

Potential new tools:

```ts
focusa_state_hygiene_doctor
focusa_state_hygiene_plan
focusa_state_hygiene_apply
```

Checks:

- stale current focus
- obsolete next steps
- resolved open questions
- stale blockers
- duplicate notes/results
- old low-value notes
- Focus State vs Workpoint mismatch

Acceptance:

- Tool can identify stale state without mutating it.
- Apply path distinguishes archive, supersede, resolve, delete-not-supported, and corrective-write actions.

### 6.10 Agent pickup nudges

The tool layer should guide agents toward the right tool at the right time.

| Situation | Nudge |
|---|---|
| Before risky edit/restore | Use `focusa_tree_snapshot_state`. |
| After compaction/resume/uncertainty | Use `focusa_workpoint_resume`. |
| Before compaction/model switch/context overflow | Use `focusa_workpoint_checkpoint`. |
| After decision | Use `focusa_decide`. |
| After discovered hard boundary | Use `focusa_constraint`. |
| After failure | Use `focusa_failure`. |
| Before planning with uncertainty | Use `focusa_metacog_retrieve` or `focusa_metacog_doctor`. |
| After repeated failure | Use `focusa_metacog_reflect`. |
| Before handoff | Use Workpoint checkpoint + next step + recent result. |

Acceptance:

- Prompt-level tests prove agents pick Focusa tools in scenarios where specs say they should.

### 6.11 Tool descriptions and visible summaries

All tool descriptions should follow this shape:

```text
Best safe tool for <job>. Use when <scenario>. Avoid when <scenario>. Returns <summary>. Next tools: <x/y>.
```

Visible result summaries should include:

- what happened
- canonical/degraded status
- affected refs
- evidence refs
- next safe tool

---

## 7) Tool-family-specific hardening requirements

### 7.1 Focus State tools

Required improvements:

- Duplicate detection for decisions/constraints/failures/results.
- Optional evidence/source refs for decisions, constraints, failures, and results.
- Make `focusa_note` category-based or visibly low-authority.
- Sync `intent`, `current_focus`, and `next_step` with active Workpoint when appropriate.
- Add resolve/supersede lifecycle for open questions and next steps.

### 7.2 Workpoint tools

Required improvements:

- Add `focusa_workpoint_recent` helper.
- Add `focusa_workpoint_guard` composite: snapshot + checkpoint + resume + optional drift-check.
- Guard must return canonical/degraded status and never silently promote fallback.

### 7.3 Tree/snapshot tools

Required improvements:

- Compact summaries by default for large lineage/path/diff responses.
- Restore dry-run.
- Snapshot reason and changed summary in recent snapshot output.
- Link snapshots/diffs to active Workpoint/evidence when relevant.

### 7.4 Metacognition tools

Required improvements:

- Quality gates for reflect/adjust/evaluate.
- Auto-suggest metrics for evaluation.
- Ranking explanations in retrieve.
- Doctor recommendations for cleanup/promotion/capture.
- Link learnings to active Workpoint/object/bead.

### 7.5 Lineage intelligence tools

Required improvements:

- Capped compact lineage tree by default.
- `li_tree_extract` creates candidate decisions/constraints/failures/reflections, not canonical writes.
- Promote/reject workflow for extracted candidates.

### 7.6 Work-loop tools

Required improvements:

- Preflight and writer-status helpers.
- Better blocked result explanations.
- Explicit distinction between work-loop checkpoint and Workpoint checkpoint.
- Safe next actions in every result.

---

## 8) New proposed tools / composites

| Tool | Type | Purpose | Priority |
|---|---|---|---|
| `focusa_tool_doctor` | composite/read | Whole-suite health/readiness diagnostic. | P0 |
| `focusa_active_object_resolve` | helper/read | Resolve current mission/workpoint into active objects/files/tests/endpoints. | P0 |
| `focusa_evidence_capture` | write | Store/link concise evidence records. | P0 |
| `focusa_workpoint_recent` | helper/read | Find recent Workpoint ids/checkpoints. | P1 |
| `focusa_workpoint_guard` | composite/mixed | Snapshot + checkpoint + resume + optional drift-check. | P1 |
| `focusa_work_loop_preflight` | read | Determine whether control action is safe. | P1 |
| `focusa_work_loop_writer_status` | read | Explain active writer claim and allowed actions. | P1 |
| `focusa_state_hygiene_doctor` | read | Diagnose stale/duplicate/mismatched Focus State. | P1 |
| `focusa_state_hygiene_plan` | proposal | Produce proposed cleanup actions. | P2 |
| `focusa_state_hygiene_apply` | write | Apply approved cleanup/supersede/resolve actions. | P2 |

---

## 9) Implementation phases

### Phase 0 — Contract matrix

Create a complete tool contract matrix for all `focusa_*` Pi tools.

Columns:

- tool
- target surface
- read/write/mixed
- side effects
- idempotency class
- retry posture
- failure modes
- canonical/degraded behavior
- verification hook
- Workpoint linkage
- next-tool hints
- spec refs

Exit gate: matrix covers all tools in `apps/pi-extension/src/tools.ts`.

### Phase 1 — Unified result envelope

Implement shared result helpers in Pi extension.

Exit gate:

- All Focusa tools return common `details` fields.
- Tests cover read, write, degraded, validation rejection, timeout/unknown, and writer-conflict responses.

### Phase 2 — Workpoint spine and evidence linking

Implement active Workpoint resolver and evidence capture/linking.

Exit gate:

- Decision/failure/result/snapshot/metacog outputs can attach to active Workpoint.
- Workpoint resume packet can include new evidence refs.
- No direct Pi reducer bypass occurs.

### Phase 3 — Doctor and resolver tools

Implement:

- `focusa_tool_doctor`
- `focusa_active_object_resolve`
- `focusa_evidence_capture`

Exit gate:

- Doctor distinguishes ready/degraded/blocked.
- Resolver returns useful setup path when no active object exists.
- Evidence capture returns stable evidence refs.

### Phase 4 — Work-loop and metacog quality hardening

Implement work-loop preflight/writer clarity, metacog quality gates, metric suggestions, and promotion policy.

Exit gate:

- Work-loop conflicts are never generic failures.
- Metacog loop rejects or downgrades vague low-evidence learning artifacts.

### Phase 5 — Hygiene and dedupe

Implement duplicate detection for cognitive writes and stale-state doctor/plan/apply.

Exit gate:

- Duplicate decisions/failures/constraints are detected before append.
- Stale questions/next steps/focus can be proposed for cleanup.

### Phase 6 — Pickup proof and operational stress

Add prompt-level pickup tests, API/CLI/Pi parity tests, degraded fallback tests, long-session compaction/resume tests, and full stress suite.

Exit gate:

- Agent chooses correct Focusa tools in scenario prompts.
- Full stress suite passes with zero failures.
- Evidence doc cites code, specs, and live outputs.

---

## 10) Acceptance tests

Minimum test pack:

1. Tool envelope contract test.
2. Workpoint linkage test.
3. Degraded fallback test.
4. Duplicate cognitive write test.
5. Work-loop writer test.
6. Metacog quality test.
7. Tool doctor test.
8. Active object resolver test.
9. Prompt pickup test.
10. Live stress suite.

---

## 11) Non-goals

This spec does not require:

- replacing Focusa reducer semantics,
- letting Pi directly canonize ontology state,
- storing raw transcripts as primary memory,
- weakening Focus State validators,
- automatically deleting cognitive state without proposal/approval,
- making work-loop autonomy bypass operator steering.

---

## 12) Final success criteria

Spec 89 is complete when:

1. every Focusa tool has a typed, predictable, spec-compliant result envelope,
2. evidence-producing tools can link to active Workpoint,
3. agents have one obvious doctor for Focusa tool readiness,
4. agents can resolve active objects without guessing,
5. work-loop control is preflighted and understandable,
6. metacognition tools enforce quality and evidence standards,
7. append-only cognitive writes have duplicate/idempotency protection,
8. stale Focus State can be diagnosed and cleaned through proposal-first workflows,
9. prompt-level pickup tests prove agents choose the tools when useful,
10. full stress tests prove API/CLI/Pi behavior remains operationally reliable.

---

## 13) Design law

Focusa tools should reduce cognitive entropy.

Every tool should either:

- preserve meaning,
- prove state,
- recover continuity,
- reduce ambiguity,
- improve future behavior,
- or safely expose operator control.

If a tool does none of these, it remains in the suite but should be redesigned, clarified, merged upward, or hardened until its value is obvious.

No existing Focusa tool should be demoted as part of this hardening program.
