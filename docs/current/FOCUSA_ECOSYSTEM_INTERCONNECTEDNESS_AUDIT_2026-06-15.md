# Focusa Ecosystem Interconnectedness Audit — 2026-06-15

Parent work: `focusa-4jo5.2` + Spec108 inventory/design  
Related specs: Spec106 Vision Tightening, Spec107 Spec-first/claim discipline, Spec108 Pi plugin awareness algorithm  
Status: audit round in progress; no implementation changes made here.

## 1. Purpose

Evaluate how the Focusa ecosystem talks to itself so the next design phase can decide what parts should become more intelligent.

This is not a surface wording pass. Focusa is a multi-surface system with many tools and state projections. The important question is: **which subsystem should supply authority, which subsystem should supply context, which subsystem should supply proof, and which renderer should decide what the agent/operator actually sees?**

## 2. Current ecosystem nodes

| Node | Role | Current source refs |
| --- | --- | --- |
| Pi plugin awareness renderer | Visible reload card + system prompt utility card | `apps/pi-extension/src/awareness.ts`, `apps/pi-extension/src/turns.ts` |
| Pi plugin compaction engine | context pressure, checkpoint, compaction, hidden auto-resume | `apps/pi-extension/src/compaction.ts` |
| Pi plugin state helpers | scope, current ask, attention recall, Workpoint scoping, project-switch ledger | `apps/pi-extension/src/state.ts` |
| Pi tool registry | 96 `focusa_*` tools and tool outputs | `apps/pi-extension/src/tools.ts` |
| Tool contracts/choreography | API/CLI/core/doc/next-tool metadata | `apps/pi-extension/src/tool-contracts.ts`, `docs/current/focusa-tool-contracts.json`, `docs/current/focusa-tool-choreography.json` |
| Focusa daemon API | canonical/read-model endpoints | `crates/focusa-api/src/routes/*` |
| Focusa core | durable types/rules/reducers/reports | `crates/focusa-core/src/*` |
| CLI | operator commands and local proof path | `crates/focusa-cli/src/commands/*` |
| Workpoint | immediate action authority | `/v1/workpoint/*`, `focusa_workpoint_resume/checkpoint/link_evidence` |
| Trajectory ladder | north-star HLT/MLG/STG/Waypoints context | `/v1/trajectory/*`, `focusa_trajectory_view/resume/assess` |
| Context Cognition | advisory scoped context packet | `/v1/context-cognition/*`, `focusa_context_cognition(_render/_proof/_curate)` |
| Evidence | proof handles and confidence updates | `focusa_evidence_capture`, `focusa_workpoint_link_evidence`, UIAI diagnostics intake |
| Session transfer | save/continue handoff semantics | `focusa_session_transfer` |
| DX/UX digest | doability/recovery/status summary | `/v1/dxux/digest`, `focusa_dxux_digest` |
| Utility card daemon model | current card model but not Pi reload renderer | `/v1/utility/card`, `focusa_utility_card` |
| UIAI Engine bridge | browser/product diagnostics and visual proof | `focusa_browser_diagnostics_intake`, UIAI skill/tool routes |
| Beads | durable work decomposition/closure state | `.beads/`, `bd` CLI |
| Public docs/release proof | claims and proof boundaries | `README.md`, `docs/current/*`, `docs/evidence/*` |

## 3. Measured tool graph snapshot

From `docs/current/focusa-tool-contracts.json` and choreography registry:

- Tool contracts: 96
- Pi `focusa_*` tools: 96
- Families:
  - diagnostics_hygiene: 21
  - focus_state: 11
  - metacognition: 13
  - project_identity: 4
  - session_transfer: 6
  - trajectory: 14
  - traversal: 2
  - tree_lineage: 9
  - work_loop: 7
  - workpoint: 9
- Parity:
  - full: 67
  - domain: 22
  - pi_only: 6
  - local_only: 1
- Side-effect classes are mixed: read-state, write-state, checkpoint, evidence-link, process-control, device-pair write, optimizer/eval writes, etc.

Implication:

- The Pi plugin cannot intelligently list tools. It needs a **tool-family selector** that picks the top few tools by current state and side-effect risk.

## 4. Important communication edges

### 4.1 Pi reload → awareness renderer → state helpers

Flow:

```text
before_agent_start → turns.ts → buildFocusaUtilityCard → state helpers → visible/system card
```

Source refs:

- `apps/pi-extension/src/turns.ts:488`
- `apps/pi-extension/src/turns.ts:501-502`
- `apps/pi-extension/src/awareness.ts::buildFocusaUtilityCard`
- `apps/pi-extension/src/state.ts` helpers

Important:

- This is the first thing the agent sees after reload.
- It must not be stale, verbose by default, or disconnected from Workpoint/Trajectory authority.

Current gap:

- `awareness.ts` uses fixed labels/prose instead of dynamic selection.

Intelligence opportunity:

- Replace fixed card renderer with `AwarenessPacket` line selection using authority, risk, actionability, novelty, and proof value.

### 4.2 Context pressure → compaction engine → Workpoint/Trajectory checkpoint → hidden resume

Flow:

```text
checkCompactionTier → warnings/status → checkpointBeforeCompaction → checkpointTrajectoryBeforeCompaction → compact command → session_compact → refresh Workpoint/Trajectory → hidden resume turn
```

Source refs:

- `apps/pi-extension/src/compaction.ts::checkCompactionTier`
- `contextPressureWarningCopy`
- `checkpointBeforeCompaction`
- `checkpointTrajectoryBeforeCompaction`
- `refreshWorkpointResumePacket`
- `refreshTrajectoryResumePacket`
- `submitCompactionResumeTurn`

Important:

- This is the reliability layer during context pressure.
- It should preserve mission and reduce operator panic.

Current gap:

- Hard/handoff warnings repeat while risk state is unchanged.
- Compaction warnings are not coordinated with the same awareness/card algorithm.

Intelligence opportunity:

- Add a context-pressure state machine: dedupe, throttle, escalate only on state/risk transitions, status-bar persistent state, visible warning only when needed.

### 4.3 Workpoint → Trajectory → Context Cognition

Flow:

```text
Workpoint resume/checkpoint = immediate authority
Trajectory view/resume = north-star context
Context Cognition = advisory context selection/proof map
```

Important:

- These are three different authority levels.
- The handoff must not blur them.

Current gap:

- Pi cards sometimes treat all context as one block.
- Context Cognition output is good but not integrated as rich-mode substrate.

Intelligence opportunity:

- Introduce explicit authority layer:
  - canonical Workpoint = action anchor
  - Trajectory = goal/gap/waypoint context
  - Context Cognition = advisory curated context

### 4.4 Session transfer → Workpoint/Trajectory continuity

Flow:

```text
focusa_session_transfer status/save/continue → project identity + Workpoint + trajectory refs
```

Important:

- This is a portable handoff concept.
- It is different from compaction resume.

Current gap:

- Session transfer appears detached from reload/post-compaction awareness.
- Saved=false/resume=not_found is useful but should not clutter default cards.

Intelligence opportunity:

- Show session-transfer state only when save/continue/handoff is requested or when a saved packet exists.
- Reconcile continuity mismatches visibly.

### 4.5 UIAI Engine → diagnostics intake → evidence/workpoint/prediction/metacog

Flow:

```text
UIAI browser diagnostics → focusa_browser_diagnostics_intake → evidence refs + active object hints + prediction + optional metacog
```

Important:

- UIAI is the real browser/product proof path.
- Browser failures must become structured blockers/evidence, not hand-waved.

Current gap:

- UIAI-first rules exist in instructions/docs but are not fully integrated into card/handoff layers.
- Browser pressure/private URL blocks should appear in risk/proof layer.

Intelligence opportunity:

- Treat UIAI browser state as proof/risk candidates in AwarenessPacket.
- Auto-suggest diagnostics intake after browser failures.

### 4.6 Tool contracts → Pi tools → docs/tests

Flow:

```text
tool-contracts.ts → docs/current/focusa-tool-contracts.json → Pi tool schemas/descriptions → generated summary/tests/docs
```

Important:

- Tool registry is the source of the ecosystem map.
- The card algorithm should use it to choose next/recovery tools.

Current gap:

- Tool metadata is rich but not yet used as a runtime selector.
- 39 tools need explicit description/snippet audit; 7 descriptions and 5 snippets are long.

Intelligence opportunity:

- Build a tool metadata freshness audit.
- Add runtime top-tool selection by family, side-effect risk, and current state.

### 4.7 Public docs → claims → beads/evidence

Flow:

```text
README/docs claims → beads acceptance → proof handles → closure policy
```

Important:

- Public docs influence operator/model expectations.
- Stale docs can create false confidence.

Current gap:

- Menubar proof wording had to be corrected to “in testing.”
- Spec107/claim discipline exists but implementation is not yet active.

Intelligence opportunity:

- Docs claim checker: public claims should link to current evidence class and open blockers.

## 5. What matters most in how parts talk

### 5.1 Authority separation

The system must always know which surface has action authority:

1. operator steering
2. verified project identity
3. canonical scoped Workpoint
4. Trajectory ladder as goal context
5. Context Cognition/traverse/evidence as advisory support
6. transcript tail never authority

### 5.2 Evidence provenance

Every proof-bearing surface should say:

- what was proven
- where it was proven
- whether evidence is actual/partial/surrogate/blocked/missing
- which Workpoint/bead it supports

### 5.3 State freshness

Cards/tools/prompts should degrade when stale instead of printing stale certainty.

Freshness inputs:

- project_root safety
- continuity match
- Workpoint packet canonical/scoped
- Trajectory clarity
- Context Cognition advisory/canonical=false
- browser pressure/failure
- tool contract count/version
- Jiti/runtime cache source age

### 5.4 Selection over dumping

For 96 tools, intelligence means selecting:

- top 1 exact next tool
- up to 3 next tools
- up to 3 recovery tools
- only relevant family hints

### 5.5 Cadence

Warnings/cards should be state transitions, not repeated background noise.

Cadence applies to:

- context pressure
- unsafe root
- missing Workpoint anchor
- tool-output flood
- browser pressure
- stale/cross-project scope

## 6. Candidate intelligence upgrades

Ranked by leverage:

### 6.1 Shared AwarenessPacket builder

One internal packet feeds:

- reload visible card
- system awareness kernel
- post-compaction card
- context-pressure warning
- utility card command output

Inputs:

- ProjectIdentity
- WorkpointResumePacket
- TrajectoryView
- ContextCognitionRender
- DXUX digest
- SessionTransfer status
- UIAI pressure/diagnostics state
- tool contract/choreography graph
- evidence/prediction/metacog relevance

Output:

- identity layer
- authority layer
- mission/gap layer
- action layer
- risk layer
- proof layer
- recovery layer
- optional learning layer

### 6.2 Context-pressure warning state machine

Replace direct repeated `ctx.ui.notify` warnings with:

- dedupe key
- last shown timestamp
- pct band
- tier transition
- anchor state transition
- compaction count band
- risk escalation flag

### 6.3 Tool-family selector

Given current state, select tools by:

- family relevance
- side-effect safety
- next-tool choreography
- current blocker
- authority needed
- proof needed

### 6.4 Handoff composer

A smart handoff should combine:

- canonical Workpoint
- Trajectory gap
- session save/transfer if available
- Context Cognition advisory packet
- evidence refs
- do-not-drift

### 6.5 UIAI proof bridge

Browser diagnostics should become first-class proof/risk candidates in cards:

- actual browser proof
- blocked browser proof
- private URL guard proof
- missing native proof

### 6.6 Tool metadata freshness auditor

Static audit that fails when:

- new tool lacks description/snippet
- description too long without reason
- snippet lacks when-to-use
- side-effect guardrail missing
- contract next tools disagree with source next tools

### 6.7 Runtime cache/source freshness check

Because `/tmp/jiti/src-awareness.*.mjs` can hold stale generated code, reload proof should verify:

- source hash
- generated cache age/hash if accessible
- actual rendered card output

## 7. Proposed final architecture direction

Do not make each card smarter independently. Make one awareness substrate smarter.

Proposed internal shape:

```text
AwarenessInput
  project_identity
  workpoint_resume
  trajectory_view
  context_cognition
  session_transfer
  dxux_digest
  uiai_state
  tool_graph
  pressure_state
  operator_steering

AwarenessCandidateLine
  layer
  text
  authority_value
  actionability
  risk_reduction
  novelty
  proof_value
  redundancy_penalty
  staleness_penalty
  mode_allowed

AwarenessPacket
  mode
  status
  visible_lines
  system_lines
  next_tools
  recovery_tools
  suppressed_lines_with_reasons
```

This lets Focusa be verbose when useful and succinct when the state is already clear.

## 8. Immediate implications for Spec108

Spec108 should require:

1. Shared AwarenessPacket builder.
2. Context-pressure warning cadence engine.
3. Tool-family selector.
4. Workpoint/Trajectory/Context authority reconciliation.
5. UIAI proof/risk bridge.
6. Tool metadata freshness audit.
7. Actual renderer/reload proof.

## 9. Open questions before design

- Should daemon `/v1/utility/card` become an input to Pi AwarenessPacket, or should Pi reimplement the same algorithm locally for low-latency/offline behavior?
- Should context pressure warnings be stored in `S` only or also emitted to daemon telemetry for cross-session suppression?
- Should tool metadata freshness limits be hard thresholds or advisory warnings initially?
- Should session-transfer state appear in default reload only when saved=true, or also when no canonical Workpoint exists?
- How much Trajectory ladder belongs in minimal mode: gap only, STG+gap, or HLT+STG+gap?

## 10. Current conclusion

The most important next design choice is the shared AwarenessPacket substrate. Without it, reload cards, compaction warnings, post-compaction handoffs, tool descriptions, and UIAI proof messages will continue evolving independently and drifting.

The first intelligence upgrade should be the substrate and scoring/cadence model, not a local text edit.
