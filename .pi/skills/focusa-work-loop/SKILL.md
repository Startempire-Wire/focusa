---
name: focusa-work-loop
description: "Use when controlling Focusa continuous work-loop state, checking writer ownership, preflighting pause/resume/stop, or selecting next ready work."
---

# Focusa Work-loop Playbook

Use when controlling Focusa continuous work-loop state, checking writer ownership, preflighting pause/resume/stop, or selecting next ready work.

## Progressive disclosure

Read `references/01-focusa-work-loop-runbook.md` for writer ownership, preflighted mutation, Silent Session handoff, rollover, and evidence closure.

## Start here

1. Load the main Focusa skill if you need the whole system model: `/skill:focusa`.
2. Read the focused tool doc: `docs/focusa-tools/work-loop.md`.
3. Prefer canonical Focusa state over transcript memory.
4. Preserve proof as evidence refs, not pasted logs.

## Primary docs

- Focused tools: `docs/focusa-tools/work-loop.md`
- Tool index: `docs/focusa-tools/README.md`
- Operator guide: `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md`
- Live release proof: `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`

## Safety rules

- Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as recovery states, not success.
- Use Workpoint resume/checkpoint around compaction, context overflow, model switch, fork, or risky release work.
- Use writer-status/preflight before mutating work-loop state.
- Do not describe Focusa as complete or frozen; use current snapshot/version language.


## Multi-agent orchestration

Fan out N silent sessions bound to work items (docs/168-multi-agent-silent-session-orchestration-workflow.md).
The workloop stays the scheduler; sessions stay workloop-compatible
(work_item_ref + scope + budget). Completions join through the
silent-session completion stream + bg receipts — never raw shells.
