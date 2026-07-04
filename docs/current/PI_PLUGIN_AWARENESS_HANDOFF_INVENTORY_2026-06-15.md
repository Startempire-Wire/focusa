# Pi Plugin Awareness + Handoff Inventory — 2026-06-15

Parent: `focusa-4jo5.2`  
Specs: Spec106 Vision Tightening + Spec108 Pi Plugin Awareness/Card/Tool Algorithm  
Status: inventory in progress; no renderer implementation changes in this document.

## Why this inventory exists

The stale Pi reload card came from the Pi plugin, not the daemon utility-card endpoint. Operator direction: this is not a small wording cleanup. Focusa needs the smartest current communication algorithm for a 70+ tool ecosystem while preserving Spec106 vocabulary and cognitive architecture.

Spec106 says not to flatten the model. Spec108 says not to dump the whole ecosystem. The target is **maximum decision usefulness per token**.

## Confirmed stale source

Source file:

- `apps/pi-extension/src/awareness.ts`

Function:

- `buildFocusaUtilityCard(mode)`

Stale/default labels found in source:

- `MISSION_PACKET` — line 59
- `UNBOUND_UNSAFE_ROOT` — line 60
- `RECONCILIATION_ENVELOPE` — line 71
- `NOW_CARD` — line 81
- `WHY_CARD` — line 85
- `HEALTH_CARD` — line 88
- `DO_CARD` — line 90
- `Friendly Focusa Q` — line 96
- `Golden route` — line 107
- `Missing active Pi frame fallback` — line 108
- `Project-bound Workpoint` — lines 121, 135
- `Suggested first route` — lines 123–124

Runtime/cache proof:

- `/tmp/jiti/src-awareness.e8ca9e29.mjs` is generated from `apps/pi-extension/src/awareness.ts` and contains the same stale labels.

Injection points:

- `apps/pi-extension/src/turns.ts:488` injects `buildFocusaUtilityCard("system")` into the system prompt.
- `apps/pi-extension/src/turns.ts:501-502` sends `buildFocusaUtilityCard("visible")` as the visible reload card.

## Compaction warning inventory

Operator-observed repeated warnings:

```text
Warning: 💡 Context at 93% — Focusa anchors are unconfirmed; checkpoint/resume Workpoint, /fork optional for UI isolation
Warning: ⚠️ Context 93% — Focusa will try checkpointed compaction; scoped Workpoint anchor not yet confirmed
Warning: 💡 62 compactions with unconfirmed Workpoint anchor — resume/checkpoint Workpoint; handoff optional
```

Exact source:

- `apps/pi-extension/src/compaction.ts:358-363`
  - `contextPressureWarningCopy(kind, pct, totalCompactions)` builds all three strings.
- `apps/pi-extension/src/compaction.ts:689`
  - `auto_suggest` warning fires once behind `S.forkSuggested`.
- `apps/pi-extension/src/compaction.ts:696`
  - `hard_unconfirmed` warning fires whenever hard-tier pressure is checked and Focusa continuity is unhealthy.
- `apps/pi-extension/src/compaction.ts:699`
  - `handoff_unconfirmed` warning fires whenever hard-tier pressure is checked and `S.totalCompactions >= autoSuggestHandoffAfterNCompactions`.
- `apps/pi-extension/src/compaction.ts:346-354`
  - `isFocusaContextContinuityHealthy()` treats continuity as healthy only when cwd is a safe project root, Focusa is available, continuity id exists, and no degraded/unscoped Workpoint packet is active.
- `apps/pi-extension/src/config.ts:100-101`
  - defaults: `autoSuggestForkPct=90`, `autoSuggestHandoffAfterNCompactions=3`.
- `apps/pi-extension/src/state.ts:281-284`
  - tracks `totalCompactions` and `forkSuggested`, but no equivalent dedupe/throttle state exists for hard/handoff unconfirmed warnings.

Root cause:

- The hard/handoff warnings are correct in meaning but not smart in cadence.
- `auto_suggest` is one-shot; `hard_unconfirmed` and `handoff_unconfirmed` are not deduped/throttled/consolidated.
- When context pressure stays high, Pi can emit repeated warning lines that say almost the same thing.

Desired algorithm:

- Collapse the three warnings into one current-state message per pressure episode.
- Escalate wording by tier only when the tier or anchor state changes.
- Dedupe by `(tier, anchor_state, rounded_pct_band, compaction_count_band)`.
- Throttle repeated notifications within a time window unless risk increases.
- Prefer status bar for persistent state; visible warning only on transition/escalation.
- Include exact action once: `resume/checkpoint Workpoint`; `/fork` optional only when UI isolation is genuinely useful.
- When a canonical scoped Workpoint becomes available, clear unconfirmed-anchor warning state.

Proposed concise warning shape:

```text
⚠️ Context 93% · Workpoint anchor unconfirmed — checkpoint/resume now; Focusa will compact with best-effort handoff.
```

Optional escalation after repeated compactions:

```text
⚠️ Repeated compactions without scoped Workpoint anchor (62) — resume/checkpoint before more durable work.
```

Spec108 impact:

- Compaction warnings are a card class with `risk + exact action + cadence`, not generic utility-card prose.
- This belongs in the same dynamic awareness algorithm as reload/post-compaction cards.

## Current live output samples

### Project identity

`focusa_project_identity(project_root=<project-root>)` returns verified/high confidence:

- project: Focusa
- project_id: `focusa`
- root: `<project-root>`
- repo: `https://github.com/Startempire-Wire/focusa.git`
- workspace: Rust monorepo
- local URL: `http://127.0.0.1:8787`

Useful handoff value: strong scope authority.

Gap: Pi reload card should use this when inferable instead of dumping `/root` unsafe state as the whole card.

### Trajectory ladder

`focusa_trajectory_view` sample:

- HLT: Address backlog beads and prepare for MVP rollout by focusing on core Focusa software-wide improvements and gap closures.
- Desired: Focusa core software is MVP-rollout ready...
- Current: real browser/product QA found product binding/polish issues after Spec106 static completion.
- Gap: Finalize Spec107 draft / child beads / anti-false-claim workflow.

Useful handoff value: preserves Spec106 HLT/MLG/STG/Waypoints and current gap.

Gap: output is a good summary, but Pi plugin cards need to carry only the relevant ladder slice, not full prose every time.

### Workpoint resume

`focusa_workpoint_resume` sample:

- canonical=true
- workpoint_id: `019ecb37-72d5-7b62-aa42-56665be38308`
- mission: write and commit Spec107, then decompose `focusa-bwky` before implementation.
- next: finalize Spec107 / create child beads / commit and push spec/decomp only.
- do_not_drift:
  - do not implement claim gate before spec/decomp
  - do not stage ECS/runtime/release-proof residue
  - do not close unfinished Mac E2E or `focusa-bwky`

Useful handoff value: strongest immediate action authority.

Gap: stale utility card should defer to canonical Workpoint when available instead of printing onboarding questions.

### Session transfer

`focusa_session_transfer status` sample:

- project=Focusa
- saved=false
- resume=not_found
- inferred_wp=verify_or_fix_tests
- shortest=execute_path
- continuity_id differed from active Workpoint continuity in sample.

Useful handoff value: can expose save/continue state and whether a portable handoff exists.

Gap: current output is compact but not yet integrated into Pi reload handoff. It also needs authority reconciliation when continuity differs.

### Context Cognition render

`focusa_context_cognition_render` sample:

- advisory/read-only/canonical=false
- schema: `focusa.context_cognition_packet.v1`
- workpoint_id + trajectory_id
- authority: workpoint (`canonical_mutation_allowed=false`)
- next tools: active object resolve, workpoint checkpoint, evidence capture
- do_not_drift: transcript tail as authority; cross-project scope fallbacks

Useful handoff value: excellent compact advisory context.

Gap: Pi reload could use this as a rich-mode card substrate, but must label advisory/canonical correctly.

### DX/UX digest

`focusa_dxux_digest` sample:

- can_continue=true
- exact next action: run Workpoint resume, verify project scope, execute preflight before durable closure.

Useful handoff value: recovery/doability snapshot.

Gap: exact next action can conflict with Workpoint next action unless reconciled. Pi card algorithm needs precedence: operator steering > verified identity > canonical Workpoint > trajectory/context/dxux.

### Utility card endpoint/tool

`focusa_utility_card` sample is compact:

- status=completed
- next tools: utility card, Workpoint resume, Trajectory view, evidence capture.

Daemon endpoint `/v1/utility/card` has richer fields:

- authority boundary
- usefulness bar
- scope gate
- bootstrap card
- post-compaction card
- exact next actions
- do-not-drift
- evidence policy
- recovery order
- proof commands

Useful handoff value: current daemon utility model is closer to desired than stale Pi renderer.

Gap: Pi reload renderer does not consume or mirror this current shape.

## Handoff quality assessment

Current Focusa handoff is powerful but fragmented:

| Surface | Strength | Gap |
| --- | --- | --- |
| Workpoint resume | Best immediate authority; mission/next/do-not-drift | Not always first in visible card; stale utility card can drown it |
| Trajectory view | Preserves HLT/MLG/STG/Waypoints and desired/current/gap | Needs line selection for relevance |
| Context Cognition | Compact advisory packet with authority labels | Not integrated into reload card mode selection |
| Session transfer | Save/continue primitive | Needs clearer role in Pi reload handoff and continuity reconciliation |
| DX/UX digest | Doability/recovery action | Needs precedence reconciliation with Workpoint |
| Utility endpoint | Modern card model | Pi renderer stale and independent |
| Pi awareness renderer | Always visible/injected | Static labels/prose; not algorithmic; can be stale |
| Tool descriptions/snippets | Large ecosystem guide | Needs current freshness/usefulness audit |

## Clean Focusa-enhanced handoff target

A clean handoff should be a layered packet, not a wall of unrelated cards:

1. **Identity layer** — project, root, continuity, scope safety.
2. **Authority layer** — canonical Workpoint if verified; trajectory advisory; transcript never authority.
3. **Mission layer** — HLT/MLG/STG only as needed for current gap.
4. **Action layer** — exact next action + top tools.
5. **Risk layer** — do-not-drift, blockers, stale surfaces.
6. **Proof layer** — evidence handles and missing proof.
7. **Recovery layer** — one recovery route.
8. **Learning layer** — prediction/metacog only when it changes the next action.

The visible card should normally show layers 1–5. Rich/onboarding mode may show layers 6–8.


## Post-compaction / compaction source map

Key source functions in `apps/pi-extension/src/compaction.ts`:

- `buildLearningCompactionCard` — prediction/metacog wrap-up context for compaction.
- `buildCompactionFallbackSummary` — fallback summary when canonical packets are unavailable.
- `refreshWorkpointResumePacket` — fetches Workpoint resume packet.
- `checkpointTrajectoryBeforeCompaction` — preserves Trajectory ladder north-star context.
- `refreshTrajectoryResumePacket` — hydrates Trajectory resume context.
- `checkpointBeforeCompaction` — creates Workpoint checkpoint before compaction.
- `contextPressureWarningCopy` — emits current repeated context-pressure warnings.
- `setContextStatus` — status bar context pressure state.
- `submitCompactionResumeTurn` / `scheduleCompactionResumeRetry` — hidden auto-resume turn submission after compaction.
- `checkCompactionTier` — pressure threshold logic and warning calls.

Current issue:

- Compaction/handoff logic contains many valuable Focusa hooks, but warning emission is not governed by a unified awareness algorithm.
- Pressure warnings use `ctx.ui.notify` directly instead of a dedupe/scoring/cadence layer.
- The output is correct in intent but can repeat and compete with Workpoint/Trajectory handoff cards.

Testing gap:

- No current test was found that fails on default reload labels such as `MISSION_PACKET`/`NOW_CARD`.
- No current test was found that proves `contextPressureWarningCopy` is deduped or consolidated.
- Existing checks cover many contract/spec surfaces, but not the actual Pi reload/post-compaction operator-visible output.

## Tool description inventory status

Preliminary scan of `apps/pi-extension/src/tools.ts` shows approximately one registered Pi tool per contract registry entry (96 current contract count), with descriptions and prompt snippets embedded in source.

Inventory still needs per-tool classification:

- current and decision-useful
- too verbose but useful
- stale behavior claim
- missing guardrail
- missing next-tool relationship
- output summary mismatch

This cannot be closed from a spot check; it needs a tool-by-tool audit table.

## Required inventory still to complete

- `apps/pi-extension/src/compaction.ts` post-compaction renderer paths.
- `apps/pi-extension/src/tools.ts` all `focusa_*` descriptions and prompt snippets.
- UIAI Engine browser integration paths, including `focusa_browser_diagnostics_intake`, UIAI-first routing, diagnostics evidence linkage, and browser pressure/recovery messages.
- `apps/pi-extension/src/tool-contracts.ts` next-tool choreography vs actual tool behavior.
- `apps/pi-extension/prompts/*` persistent instructions.
- `apps/pi-extension/skills/*` skill guidance freshness.
- Renderer tests and current test coverage.

## Preliminary design implications

- `buildFocusaUtilityCard` should become a selector over candidate lines from current state, not fixed card labels.
- The plugin should use WorkpointResumePacket as the immediate authority when canonical and scoped.
- Trajectory ladder must be preserved but compressed to the current gap/waypoint unless rich mode is selected.
- Session transfer should appear only when saved/resumable or when handoff is requested.
- Context Cognition should be used as advisory rich context, not default prompt bulk.
- Tool ecosystem should be represented by top next/recovery tools plus family hints, not exhaustive lists.
- UIAI Engine browser guidance must stay aligned with Focusa handoff layers: browser diagnostics become evidence/proof handles, UIAI pressure belongs in risk/recovery, and UIAI-first rules must not be bypassed by generic web tooling unless the documented fallback conditions are met.

## Acceptance for inventory bead

This inventory is not complete until every required inventory item above is checked and either mapped to a source path or explicitly marked not applicable.

## Deeper state helper source map

Key helpers in `apps/pi-extension/src/state.ts`:

- `toolOutputVisibleRecapReason` / `formatToolOutputVisibleRecapLines` — detects tool-output flood and requires visible recap. Useful but separate from utility-card algorithm; should become another candidate input for risk/recap mode.
- `formatProjectSwitchLedgerLines` — exposes recent project switch evidence. Useful for scope conflict, but should not be printed unless current ask/project signals conflict.
- `buildCurrentAskScopeVerdict` / `formatCurrentAskScopeVerdictLines` — classifies whether current ask matches saved project scope. This is high-authority input for identity/authority layer.
- `buildAttentionRecallVerdict` / `formatAttentionRecallFocusSliceLines` — compaction/reload attention gate. Useful for visible recap only when required.
- `isWorkpointPacketScopedToCurrentSession` / `getScopedWorkpointPacket` — core Workpoint authority filter. This must be a first-class source for every reload/post-compaction card.
- `buildCompactInstructions` — compaction instruction builder; must align with any new post-compaction card algorithm.

Implication:

- Current state helpers already contain most authority signals. The missing piece is orchestration: a shared selector should decide which lines appear in reload, compaction, and warning outputs.

## Compaction event flow notes

Key flow in `apps/pi-extension/src/compaction.ts`:

1. `before_compact` hook builds ASCC/Focusa compaction instructions, falling back to `buildCompactInstructions`.
2. `session_compact` trims local shadow state, stamps last compact decision, refreshes Workpoint/Trajectory packets, and schedules hidden auto-resume.
3. `submitCompactionResumeTurn` sends hidden `focusa-compact-resume` message and notifies `✅ Compaction done — auto-resume turn submitted`.
4. `scheduleCompactionResumeRetry` repeats hidden submit attempts while pending.
5. `checkCompactionTier` reads context usage, updates status, emits warnings, checkpoints Workpoint/Trajectory, and submits compact command.

Good current behavior:

- Attempts checkpoint before compaction.
- Refreshes Workpoint and Trajectory resume packets.
- Tracks hard context pressure.
- Can auto-resume after compaction.

Gaps:

- Warnings are emitted directly from `checkCompactionTier` instead of through a shared cadence/scoring layer.
- Hard/handoff warnings lack transition-based dedupe.
- Hidden auto-resume success notification is concise, but the actual resume card output still needs alignment with Workpoint/Trajectory authority layers.
- Learning card (`buildLearningCompactionCard`) includes prediction/metacog context; useful for end-of-task but may be too much in every compaction handoff unless mode is rich/wrap-up.

## Tool metadata audit notes

Preliminary automated scan of `apps/pi-extension/src/tools.ts` and `docs/current/focusa-tool-contracts.json`:

- Registered `focusa_*` tools in Pi source: 96.
- Tool contracts in generated registry: 96.
- Tools with missing explicit description or promptSnippet in direct `registerTool` block scan: 39. Some of these may use registry/default metadata wrappers, but they still need explicit audit.
- Long descriptions (>240 chars): 7.
- Long prompt snippets (>180 chars): 5.

Flagged examples from scan:

- Long descriptions: `focusa_workpoint_checkpoint`, `focusa_workpoint_resume`, `focusa_context_cognition`, `focusa_device_pair_start`, `focusa_context_cognition_curate_eval`, `focusa_context_cognition_curate_optimize`, `focusa_call_stack_design`.
- Missing/implicit snippet group includes Focus State slot tools, work-loop tools, state-hygiene tools, tree-lineage tools, and reflex primitives.

Interpretation:

- This is not proof those tools are bad; it proves there is no complete freshness audit yet.
- Spec108 implementation must include a tool-by-tool table with current behavior, ideal description, ideal prompt snippet, guardrail, next tools, and stale-risk notes.

## Prompt and skill freshness inventory

Project Pi prompt/skill files found:

- `apps/pi-extension/prompts/focusa-context.md` — 31 lines; already emphasizes project scope, Workpoint, Trajectory, Context Cognition, transcript-tail avoidance.
- `apps/pi-extension/skills/focusa/SKILL.md` — 246 lines; broad Focusa skill with Workpoint/Trajectory/UIAI/canonical/degraded/stale/compaction concepts.
- `apps/pi-extension/skills/focusa-troubleshooting/SKILL.md` — 53 lines; recovery path with Workpoint/Trajectory/UIAI/stale/degraded coverage.
- Focused skill docs: `focusa-workpoint`, `focusa-work-loop`, `focusa-metacognition`, `focusa-cli-api`, `focusa-docs-maintenance`, `predictive-power`.

Gaps:

- Prompt and skills are conceptually aligned with Spec106, but they were not audited against Spec108 dynamic card/handoff algorithm.
- The broad `focusa` skill may duplicate utility-card awareness. It should be treated as rich/onboarding/reference, not default reload text.
- Skill docs need references to the new card/handoff algorithm once implemented.

## UIAI integration deeper notes

Source refs found across `apps/pi-extension/src/tools.ts`, `apps/pi-extension/src/tool-contracts.ts`, `apps/pi-extension/skills`, `docs/current`, and README:

- `focusa_browser_diagnostics_intake` is the Focusa bridge for browser diagnostics → evidence/active-object/prediction/metacog.
- README and Spec106 docs already state UIAI diagnostics are Focusa evidence surfaces.
- Operator/global directive requires UIAI-first for browser/web/docs/research tasks.

Gaps:

- UIAI pressure/failure state is not yet a first-class candidate line in the Pi awareness selector.
- Browser private/internal URL guard failures should appear as blocker evidence, not proof.
- Tool descriptions/snippets must preserve UIAI-first fallback conditions without bloating every card.

## Current research conclusion

The Pi plugin already has valuable subsystems:

- project scope inference
- Workpoint scoping
- Trajectory hydration
- compaction checkpointing
- attention recall/current ask verdicts
- tool-output recap pressure
- UIAI diagnostics intake
- tool contract/choreography registry

The problem is fragmentation:

- reload card has a static legacy renderer
- compaction warnings have direct repeated notifications
- post-compaction handoff has separate packet logic
- tool descriptions/snippets have no freshness audit artifact
- UIAI alignment is distributed across docs/tools/skills

Spec108 should design a shared `AwarenessCandidate`/`AwarenessPacket` algorithm used by reload cards, compaction warnings, and post-compaction handoff rendering.
