---
name: focusa
description: Use when preserving Focusa cognitive state, resuming after compaction/model switch/context overflow, linking evidence to Workpoints, using Focus State, work-loop, lineage/tree, metacognition, state-hygiene, or diagnosing Focusa tool readiness.
---

# Focusa Cognitive Runtime Skill

Use this skill for any task where durable meaning matters more than transcript memory: multi-step implementation, compaction recovery, release proof, evidence capture, long-running work loops, reusable learning, or Focusa tool troubleshooting.

## Progressive disclosure

Read `references/01-focusa-core-agent-runbook.md` for the current end-to-end authority, discovery, recovery, and evidence sequence. Use the generated Spec141 Pi registry for all tool schemas instead of loading every definition into context.

## Context Authority mutation boundary

Before risky mutation, run Context Authority preflight: classify prompt mode, inspect environment contract/runtime inventory/binary compatibility as relevant, run action preflight, then mutate only on an allowing verdict. Risky mutation includes binary replacement, daemon restart, deploy, release publish, git push, destructive file operation, database migration, broad refactor, cross-project edit, generated-code overwrite, secret/config change, live service action, and pairing/install/update ambiguity. Valid verdicts: `allow`, `block`, `ask_operator`, `verify_first`, `diagnosis_only`, `planning_only`.

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

then the installed `SKILL.md` is missing/has invalid frontmatter. Repair the project source copy and the user runtime copy resolved from `PI_SKILLS_DIR` or `$HOME/.pi/skills`.

Validate with the portable harness:

```bash
node scripts/validate-skill-hygiene.mjs
```

## Companion skills

The main `focusa` skill is the router/mental model. Load focused companion skills for deeper playbooks:

- `/skill:focusa-workpoint` — Workpoint checkpoint/resume/evidence/drift workflows.
- `/skill:focusa-metacognition` — reusable learning, reflection, adjustments, evaluation.
- `/skill:focusa-work-loop` — continuous work-loop writer/status/control workflows.
- `/skill:focusa-cli-api` — direct daemon/CLI/API release proof and troubleshooting.
- `/skill:focusa-troubleshooting` — offline/degraded/pending/blocked recovery.
- `/skill:focusa-docs-maintenance` — public docs, skill docs, evidence, snapshot wording.

Focused tool docs live under `docs/focusa-tools/README.md`.

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

- Working notes never go into `focusa_decide`; put reasoning, task lists, failed wording, and retries in `focusa_scratch`.
- Decisions are architectural choices, not task lists or debug narratives.
- Constraints are discovered requirements, not agent commitments. Phrase constraints as declarative architecture boundaries: `Workpoint continuity identity uses project_root plus continuity_id`, not `Need to fix...` or `Do not...`.
- If a Focus State write tool rejects validation, treat that as a phrasing error: save the detailed note to scratchpad, retry once with compliant noun-phrase wording, then continue without looping.
- Failures name the failing component and why.

## Workpoint family

Use for continuity across compaction/resume/model switch/fork/risky work.

- `focusa_workpoint_checkpoint` — create typed checkpoint before discontinuity or risky continuation.
- `focusa_workpoint_resume` — fetch active WorkpointResumePacket; use immediately after compaction or uncertainty.
- `focusa_workpoint_link_evidence` — attach stable evidence refs/results to active canonical Workpoint.
- `focusa_active_object_resolve` — resolve likely active objects; returns candidates, not invented truth.
- `focusa_evidence_capture` — capture bounded evidence and optionally link to Workpoint.

Identity and isolation rules:

- `project_root` is the project folder/container: the place that holds the files related to the project.
- Navigation metaphor: `project_root` is the vessel/hull; better project-root confidence improves travel reliability, but it is not the functional route or destination.
- `continuity_id` is the stable logical session/workstream identity; every same-root active session needs a distinct continuity_id.
- `session_id` is temporal metadata across compaction/model switch/fork; it must not merge or split logical sessions.
- Trajectory is the route model: current functional state, desired destination/outcome, and waypoint goals.
- Trajectory, work-item, frame tags, and goals are corroborating alignment signals only; they never override `project_root + continuity_id` hard gates.
- Post-compaction agents should call `focusa_workpoint_resume`; a same-project packet resumes cleanly only when continuity_id also matches.

Context pressure UX:

- Context pressure means the transcript window is tight; it does not mean Focusa lost project memory.
- Focusa preserves continuity through scoped project identity, trajectory, Workpoint packets, evidence handles, and post-compact resume guidance.
- Generic `/fork`, `/new`, or session-handoff warnings are redundant when Focusa has healthy scoped anchors.
- Surface operator-visible warnings only when scoped Focusa anchors are unconfirmed; phrase them as checkpoint/resume guidance with `/fork` optional for UI isolation.

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
- `focusa_state_hygiene_apply` — approval-gated, non-destructive apply that records an auditable Focus State note via `/v1/focus/update`.

No existing Focusa tools should be demoted; weak tools should be redesigned, clarified, merged upward, or hardened.

## Trajectory, resource, and background-session utilities

- `focusa_trajectory_view` / `focusa_trajectory_resume` — advisory project goal/state/gap orientation; corroborates Workpoint, never overrides identity gates.
- `focusa_trajectory_define_goal` / `focusa_trajectory_assess` / `focusa_trajectory_propose_workpoint` / `focusa_trajectory_checkpoint` — manage project trajectory and propose next Workpoint candidates.
- `focusa_resource_mode` — inspect or activate LowMem when resources are constrained; LowMem changes fidelity/budgets, not tool availability.
- `focusa_traverse` — read-only bounded traversal across large Focusa surfaces; use instead of full tree/store/log payloads by default.
- `focusa_silent_sessions` — list/reopen/start/tail/send/kill tmux-backed background Pi sessions; mutating process actions require approval and kill requires force.

## Tool-doctor and evidence entrypoints

- `focusa_tool_doctor` — first diagnostic for Focusa readiness, active Workpoint continuity, daemon health, and likely repair action; reads UIAI browser health/metrics only, never opens target URLs.
- `focusa_evidence_capture` — convert proof into stable handles; avoid prompt bloat.
- `focusa_active_object_resolve` — use before editing/claiming canonical refs when object identity is uncertain.
- `focusa_browser_diagnostics_intake` — after UIAI/browser diagnostics or action failure envelopes, convert browser console/network/runtime evidence into Workpoint evidence, active-object hints, prediction context, and optional metacog learning; it consumes UIAI output and does not call UIAI itself.

Browser evidence route: UIAI `browser_diagnostics` → `focusa_browser_diagnostics_intake` → `focusa_active_object_resolve`/prediction/evidence verification. UIAI `url_not_allowed` means the browser target was private/internal under hardened policy; capture it as policy evidence or use an explicit local/dev UIAI profile, not a Focusa failure. Under Focusa emergency resource mode, prefer summary trajectory/traverse views and avoid cold/full payload routes unless explicitly needed.

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

- `ok`, `status`, `failure_class`, `canonical`, `degraded`
- `summary`, `retry`, `side_effects`, `evidence_refs`, `next_tools`
- `error`, `raw`

Use `status`, `failure_class`, `retry.posture`, `canonical/degraded`, and `next_tools` for recovery decisions instead of parsing prose.

## KH/OVH execution boundary

- Pi and the authoritative Focusa daemon remain on KH.
- KH `localhost:7456` is a compatibility SSH forward; UIAI workers, Chromium, and browser memory execute on OVH.
- Focusa-scoped Cargo/npm/npx commands route to the synchronized OVH `wirebot` workspace through `/usr/local/bin/focusa-ovh-build`.
- Remote `cargo test` uses an isolated temporary OVH daemon and must never target KH production state.
- GitHub-hosted release builds are unchanged; the KH `focusa-deploy` runner downloads/verifies/installs artifacts only.
- Canonical topology, proof, and rollback: `/root/dual-server-master-plan/runbooks/21-focusa-ovh-uiai-and-build-offload-runbook.md`.

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
