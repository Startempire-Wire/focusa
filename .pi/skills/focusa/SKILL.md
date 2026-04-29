---
name: focusa
description: Use when preserving Focusa cognitive state, resuming after compaction/model switch/context overflow, linking evidence to Workpoints, using Focus State, work-loop, lineage/tree, metacognition, state-hygiene, or diagnosing Focusa tool readiness.
---

# Focusa Cognitive Runtime Skill

Use this skill for any task where durable meaning matters more than transcript memory: multi-step implementation, compaction recovery, release proof, evidence capture, long-running work loops, reusable learning, or Focusa tool troubleshooting.

## Skill loading rules

Pi skills must start with YAML frontmatter. Required fields:

```yaml
---
name: focusa
description: Use when preserving Focusa cognitive state, resuming after compaction/model switch/context overflow, linking evidence to Workpoints, using Focus State, work-loop, lineage/tree, metacognition, state-hygiene, or diagnosing Focusa tool readiness.
---
```

If Pi reports:

```text
[Skill conflicts]
~/.pi/skills/focusa/SKILL.md
  description is required
```

then the installed `SKILL.md` is missing/has invalid frontmatter. Repair both copies:

- project source: `/home/wirebot/focusa/apps/pi-extension/skills/focusa/SKILL.md`
- installed global: `/root/.pi/skills/focusa/SKILL.md`

Validate with:

```bash
node --input-type=module - <<'NODE'
import { loadSkillsFromDir } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
for (const dir of ['/root/.pi/skills','/home/wirebot/focusa/apps/pi-extension/skills']) {
  const r = loadSkillsFromDir({ dir, source: 'user' });
  console.log(JSON.stringify({ dir, skills: r.skills.map(s => s.name), diagnostics: r.diagnostics }, null, 2));
}
NODE
```

## Best-practice principles

Research/docs basis:

- Pi skill docs and Agent Skills spec: skills are progressive disclosure packages; name and description are always visible, full content loads on demand.
- Required frontmatter: `name` and `description`; `name` must match parent directory and use lowercase hyphen format.
- Description should be specific, include “when to use”, and stay below 1024 chars.
- Keep SKILL.md as a dispatcher: short core rules plus pointers to detailed docs/evidence.
- Use consistent family headings, concrete tool-selection rules, validation commands, and recovery paths.
- Avoid dumping raw logs or giant references into the skill; use evidence handles and docs paths.

## Default pickup sequence

When uncertain, resumed, compacted, or after context overflow:

1. `focusa_workpoint_resume` — retrieve canonical/degraded continuation contract.
2. `focusa_tool_doctor` — diagnose daemon, active Workpoint, Focus State, and next repair.
3. `focusa_active_object_resolve` — resolve active object candidates without inventing canonical refs.
4. `focusa_evidence_capture` or `focusa_workpoint_link_evidence` — capture proof as handles and link to active Workpoint.
5. Use the task-specific family below.

If `canonical=false` or `degraded=true`, treat output as recovery hint only until a canonical read confirms it.

## Focus State family

Use for compact cognitive state. Do not store raw transcripts here.

- `focusa_scratch` — working notes, reasoning, task lists, hypotheses, self-correction. Scratchpad only; not Focus State.
- `focusa_decide` — one crystallized architectural decision, max 280 chars; use after scratchpad reasoning.
- `focusa_constraint` — discovered hard requirement from operator/spec/API/environment.
- `focusa_failure` — specific failure plus diagnosis and recovery.
- `focusa_intent` — session mission/frame intent.
- `focusa_current_focus` — active work in 1–3 sentences.
- `focusa_next_step` — bounded next action.
- `focusa_open_question` — unresolved question.
- `focusa_recent_result` — completed result or evidence reference.
- `focusa_note` — small miscellaneous note; bounded and decays.

Validation discipline:

- Working notes never go into `focusa_decide`.
- Decisions are architectural choices, not task lists or debug narratives.
- Constraints are discovered requirements, not agent commitments.
- Failures name the failing component and why.

## Workpoint family

Use for continuity across compaction/resume/model switch/fork/risky work.

- `focusa_workpoint_checkpoint` — create typed checkpoint before discontinuity or risky continuation.
- `focusa_workpoint_resume` — fetch active WorkpointResumePacket; use immediately after compaction or uncertainty.
- `focusa_workpoint_link_evidence` — attach stable evidence refs/results to active canonical Workpoint.
- `focusa_active_object_resolve` — resolve likely active objects; returns candidates, not invented truth.
- `focusa_evidence_capture` — capture bounded evidence and optionally link to Workpoint.

Real release behavior as of Spec89:

- Checkpoint `accepted` means reducer-visible active Workpoint is materialized.
- Evidence link `accepted` means the verification record is visible in Workpoint state/resume.
- `pending` means accepted by command path but not yet safe to rely on; retry current/resume.

## Work-loop family

Use for continuous execution control and ownership checks.

- `focusa_work_loop_writer_status` — read active writer/ownership without mutation.
- `focusa_work_loop_status` — read loop state, budgets, replay consumer state.
- `focusa_work_loop_control` — `on`, `pause`, `resume`, `stop`; use `preflight=true` when ownership is uncertain.
- `focusa_work_loop_context` — update continuation decision context.
- `focusa_work_loop_checkpoint` — checkpoint continuous-loop state; not the same as Workpoint checkpoint.
- `focusa_work_loop_select_next` — defer blocked work and select next ready work item.

Writer conflicts are healthy blocked taxonomy, not generic failure. Respect `active_writer`.

## Tree, lineage, and snapshot family

Use for recoverable ancestry and state comparison.

- `focusa_tree_head` — safe starting point for current branch/head context.
- `focusa_tree_path` — ancestry lookup for a CLT node.
- `focusa_tree_snapshot_state` — create recoverable checkpoint before risky work.
- `focusa_tree_recent_snapshots` — find snapshot IDs.
- `focusa_tree_snapshot_compare_latest` — create snapshot and compare against latest/baseline.
- `focusa_tree_diff_context` — compare snapshots.
- `focusa_tree_restore_state` — restore snapshot; use only when rollback is intended.
- `focusa_lineage_tree` — fetch lineage tree.
- `focusa_li_tree_extract` — extract decision/constraint/risk/reflection signals from lineage.

Snapshot before risky restore or state-changing comparisons.

## Metacognition family

Use for reusable learning, not journaling.

- `focusa_metacog_capture` — store reusable signal; include rationale, confidence, strategy class, and evidence refs when possible.
- `focusa_metacog_retrieve` — retrieve prior learning before planning/reflection.
- `focusa_metacog_reflect` — generate hypotheses/strategy updates from recent turns.
- `focusa_metacog_plan_adjust` — turn reflection into tracked adjustment.
- `focusa_metacog_evaluate_outcome` — evaluate whether adjustment improved results.
- `focusa_metacog_recent_reflections` — find reflection IDs/update sets.
- `focusa_metacog_recent_adjustments` — find adjustment IDs.
- `focusa_metacog_loop_run` — compressed capture/retrieve/reflect/adjust/evaluate loop.
- `focusa_metacog_doctor` — diagnose signal quality/retrieval usefulness.

Respect `quality_gate`, `evidence_refs`, and suggested metrics. Improve weak signals before promotion.

## State hygiene family

Use for safe cleanup planning, never silent deletion.

- `focusa_state_hygiene_doctor` — diagnose stale/duplicate Focus State signals without mutation.
- `focusa_state_hygiene_plan` — produce proposal-style hygiene plan.
- `focusa_state_hygiene_apply` — approval-gated, non-destructive placeholder until reducer-backed hygiene events exist.

No existing Focusa tools should be demoted; weak tools should be redesigned, clarified, merged upward, or hardened.

## Tool-doctor and evidence entrypoints

- `focusa_tool_doctor` — first diagnostic for Focusa readiness, active Workpoint continuity, daemon health, and likely repair action.
- `focusa_evidence_capture` — convert proof into stable handles; avoid prompt bloat.
- `focusa_active_object_resolve` — use before editing/claiming canonical refs when object identity is uncertain.

## Commands

- `/focusa-status` — connection/frame/decision/constraint/failure counts.
- `/focusa-stack` — Focus Stack frames.
- `/focusa-checkpoint` — ASCC checkpoint.
- `/focusa-rehydrate <handle>` — retrieve externalized ECS content.
- `/focusa-explain-decision [query]` — search decisions.
- `/focusa-lineage` — CLT lineage path.
- `/wbm on` — Wirebot Mode.

## Result-envelope contract

Every `focusa_*` Pi tool should preserve a visible text summary and add `details.tool_result_v1` with common fields:

- `ok`, `status`, `canonical`, `degraded`
- `summary`, `retry`, `side_effects`, `evidence_refs`, `next_tools`
- `error`, `raw`

Use `status`, `retry.posture`, `canonical/degraded`, and `next_tools` for recovery decisions instead of parsing prose.

## Real release evidence

Current released proof:

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
- final live Workpoint: `019dd69d-2e7e-74a0-a722-a6ed804d040f`
- proof marker: `DIRECT_REAL_RELEASE_PROOF=PASS`

Operator guide:

- `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md`

## Maintenance checklist

After editing this skill:

1. Keep project and installed copies identical.
2. Validate Pi skill loader diagnostics are empty for Focusa.
3. Confirm `/skill:focusa` appears as a loaded skill in Pi’s skill set.
4. Keep description specific and under 1024 chars.
5. Keep detailed release proof in docs/evidence, not in active transcript.
