---
name: focusa-workpoint
description: "Use when preserving or recovering Focusa Workpoint continuity: checkpoint/resume after compaction, link evidence, resolve active objects, or run drift-safe handoffs."
---

# Focusa Workpoint Playbook

Use when preserving or recovering Focusa Workpoint continuity: checkpoint/resume after compaction, link evidence, resolve active objects, or run drift-safe handoffs.

## Progressive disclosure

Read `references/01-focusa-workpoint-runbook.md` for exact-scope resume, action authority, evidence linkage, and continuation checkpoints.

## Start here

1. Load the main Focusa skill if you need the whole system model: `/skill:focusa`.
2. Read the focused tool doc: `docs/focusa-tools/workpoint.md`.
3. Prefer canonical Focusa state over transcript memory.
4. Preserve proof as evidence refs, not pasted logs.

## Primary docs

- Focused tools: `docs/focusa-tools/workpoint.md`
- Tool index: `docs/focusa-tools/README.md`
- Operator guide: `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md`
- Live release proof: `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`

## Safety rules

- Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as recovery states, not success.
- Use Workpoint resume/checkpoint around compaction, context overflow, model switch, fork, or risky release work.
- Healthy Workpoint continuity makes generic `/fork`/`/new` context-pressure warnings redundant; treat them as recovery prompts only when Focusa is degraded.
- Workpoint continuity is gated by `project_root + continuity_id`; temporal Pi `session_id` changes after compaction/model switch/fork are metadata only.
- Trajectory/goals/work-item/frame tags are alignment evidence, not identity gates.
- Use writer-status/preflight before mutating work-loop state.
- Do not describe Focusa as complete or frozen; use current snapshot/version language.
