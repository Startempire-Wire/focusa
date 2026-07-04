# Focusa Awareness Algorithm — Typed Design
**Spec108 §6 extension** | Parent bead: `focusa-4jo5.3` | 2026-06-15

## 1. Overview

This document is the typed design for Spec108 §6. It defines the schemas, scoring model, mode selection, cadence logic, and sub-components of the shared awareness substrate.

Status: design in progress — no implementation changes made here.

Related:
- `docs/108-pi-plugin-awareness-card-tool-algorithm-spec.md` — spec summary
- `docs/current/FOCUSA_ECOSYSTEM_INTERCONNECTEDNESS_AUDIT_2026-06-15.md` — edge map
- `docs/current/PI_PLUGIN_AWARENESS_HANDOFF_INVENTORY_2026-06-15.md` — exact source refs

## 2. Design Principle

**One substrate, many outputs.** The awareness substrate takes a rich input bundle, generates candidate lines, scores them, and produces typed output packets for every surface that needs to tell the agent/operator something.

Surfaces served:
- Pi reload visible card
- Pi system awareness kernel
- Post-compaction handoff card
- Context-pressure warning
- Tool guidance / next-tool selection
- UIAI proof/risk bridge messages

Each surface gets a typed `AwarenessPacket` with the same algorithm. Mode and surface type determine which lines appear.

## 3. AwarenessInput Schema

```typescript
interface AwarenessInput {
  // Authority layer
  projectIdentity: {
    projectRoot: string;          // e.g. "<project-root>"
    canonicalName: string;         // e.g. "focusa"
    continuityId: string;          // e.g. "focusa-cont-focusa-..."
    sessionId: string;             // e.g. "pi-967593-..."
    confidence: "high" | "medium" | "low";
    verified: boolean;
  };

  projectRootSafety: {
    safe: boolean;                 // false if project_root is /root or outside workspace
    path: string;
    reason?: string;               // if unsafe
  };

  workpointResume: {
    workpointId: string;
    canonical: boolean;
    degraded: boolean;
    mission: string;
    nextAction: string;
    targetObjects: string[];
    verifiedEvidence: string[];
    blockers: string[];
    doNotDrift: string[];
    actionAuthority: boolean;
    continuityId: string;
    sessionId: string;
  } | null;

  trajectoryView: {
    trajectoryId: string;
    canonical: boolean;
    degraded: boolean;
    hlt: string | null;
    mlg: string | null;
    stg: string | null;
    desiredEndState: string | null;
    activeGap: string | null;
    waypoints: string[];
    clarityGate: "clear" | "unclear" | "provisional";
    nextTools: string[];
  } | null;

  contextCognition: {
    rehydrateId: string;
    workpointId: string;
    trajectoryId: string;
    actionAuthority: string;     // "workpoint" | "operator" | "unknown"
    scopeStatus: string;
    score: number;                // 0-100
    evidenceRefs: string[];
    advisory: boolean;
    canonical: boolean;
  } | null;

  sessionTransfer: {
    action: "save" | "continue" | "status";
    saved: boolean;
    resumeFound: boolean;
    continuityId: string;
    mission?: string;
    nextAction?: string;
  };

  dxuxDigest: {
    status: string;
    authority: string;
    why: string;
    exactNextAction: string;
    evidenceRefs: string[];
    rehydrateRefs: string[];
    canonical: boolean;
  } | null;

  // Risk / pressure layer
  contextPressure: {
    percentage: number;           // 0-100
    tier: "low" | "medium" | "high" | "critical";
    compactionPending: boolean;
    compactionCount: number;     // total compactions this session
    lastCompactionAt?: number;    // timestamp ms
  };

  uiaiState: {
    pressure: number;             // 0-100
    sessionCount: number;
    saturated: boolean;
    browserFailures: number;
    privateUrlBlocks: number;
  };

  // Operator steering layer
  operatorSteering: {
    currentAsk: string;
    explicitSteer?: string;       // explicit operator directive
    scopeKind?: string;
    carryoverPolicy?: string;
    excludedContextLabels?: string[];
  };

  // Evidence / learning layer
  evidence: {
    recentRefs: string[];         // evidence handles from recent work
    proofGaps: string[];          // what still needs proof
    activeObjectRefs: string[];
  };

  prediction: {
    recentPredictions: Array<{
      id: string;
      predictedOutcome: string;
      confidence: number;
      evaluated: boolean;
    }>;
    stats: {
      total: number;
      evaluated: number;
      accuracy: number;
    };
  };

  metacog: {
    recentLessons: Array<{
      kind: string;
      content: string;
      confidence: number;
    }>;
  };

  // Tool ecosystem layer
  toolGraph: {
    totalTools: number;
    families: Record<string, number>;
    topNextTools: string[];       // from choreography
    topRecoveryTools: string[];
    nextToolsByFamily: Record<string, string[]>;
    sideEffectProfiles: Record<string, string>;
  };

  // Cadence / state layer
  cadenceState: ContextPressureState | null;

  // Mode
  mode: "minimal" | "standard" | "rich" | "onboarding";
  surface: "reload" | "post_compaction" | "warning" | "tool_guidance" | "uiai_bridge";
}
```

## 4. AwarenessCandidateLine Schema

```typescript
interface AwarenessCandidateLine {
  id: string;                     // unique per run
  layer: AwarenessLayer;
  category: string;              // e.g. "authority", "mission", "risk", "proof", "recovery"
  text: string;                   // the line text
  authorityValue: number;         // 0-10
  actionability: number;          // 0-10
  riskReduction: number;           // 0-10
  novelty: number;                // 0-10; 0 if same line shown in last N turns
  proofValue: number;            // 0-10
  redundancyPenalty: number;      // 0-10; higher if shown recently
  stalenessPenalty: number;       // 0-10; higher if stale source
  dvs: number;                    // computed: sum of above - penalties
  modeAllowed: ("minimal" | "standard" | "rich" | "onboarding")[];
  surfaceAllowed: ("reload" | "post_compaction" | "warning" | "tool_guidance" | "uiai_bridge")[];
  suppressReason?: string;        // if excluded from output
  sourceRef?: string;             // which helper/contract produced this line
  evidenceRef?: string;           // proof handle if this line is evidence-backed
}

type AwarenessLayer =
  | "identity"       // project root, scope, safety
  | "authority"     // Workpoint, action authority
  | "mission"        // mission, next action, target objects
  | "goal"           // Trajectory HLT/MLG/STG/gap
  | "risk"           // blockers, pressure, UIAI failures
  | "proof"          // evidence refs, proof gaps
  | "recovery"       // next tools, recovery paths
  | "learning"       // prediction/metacog hooks
  | "tool"           // tool-family selection
  | "suppressed";    // candidate that was scored but excluded
```

## 5. AwarenessPacket Schema

```typescript
interface AwarenessPacket {
  schema: "focusa.awareness_packet.v1";
  generatedAt: number;            // timestamp ms
  mode: "minimal" | "standard" | "rich" | "onboarding";
  surface: "reload" | "post_compaction" | "warning" | "tool_guidance" | "uiai_bridge";
  status: "fresh" | "degraded" | "partial";

  // Primary output
  visibleLines: AwarenessCandidateLine[];    // lines for visible card / operator view
  systemLines: AwarenessCandidateLine[];    // lines for system awareness kernel

  // Tool guidance
  nextTools: ToolGuidance[];
  recoveryTools: ToolGuidance[];

  // Suppressed candidates with reasons
  suppressedLines: Array<{
    line: AwarenessCandidateLine;
    suppressReason: string;
    dvs: number;
  }>;

  // Metadata
  metadata: {
    dvsCutoff: number;             // minimum DVS to appear in visible
    totalCandidates: number;
    visibleCount: number;
    suppressedCount: number;
    freshnessScore: number;       // 0-100
    authorityScore: number;       // 0-100
    confidence: "high" | "medium" | "low";
    modeReason: string;            // why this mode was selected
    surfaceReason: string;         // why this surface was selected
  };

  // Rehydrate ref for continuation
  rehydrateId: string;
}
```

## 6. Decision Value Score (DVS) Formula

```
DVS = (
  authority_value × 3.0 +
  actionability × 2.5 +
  risk_reduction × 2.0 +
  novelty × 1.5 +
  proof_value × 1.5
) - (
  redundancy_penalty × 2.0 +
  staleness_penalty × 1.5
)
```

Weight rationale:
- Authority and actionability are highest weight — they are the core purpose.
- Risk reduction is high because blockers and pressure need to be surfaced.
- Novelty prevents stale repetition.
- Proof value ensures evidence-backed lines get priority.
- Redundancy penalty is higher than staleness because repeated noise is worse than stale content.

DVS thresholds (configurable):
- `minimal`: DVS ≥ 7.0 OR authority_value ≥ 8
- `standard`: DVS ≥ 4.0
- `rich`: DVS ≥ 1.5
- `onboarding`: DVS ≥ 0.5

## 7. Line Generation Sources

Each line is generated from a specific source, not hardcoded prose.

| Layer | Source | Authority |
|---|---|---|
| identity.project_root | `projectIdentity.projectRoot` | verified |
| identity.safety | `projectRootSafety` | verified |
| identity.continuity | `projectIdentity.continuityId` | verified |
| authority.workpoint | `workpointResume.workpointId` + canonical flag | canonical |
| authority.action | `workpointResume.nextAction` | canonical |
| authority.mission | `workpointResume.mission` | canonical |
| authority.targets | `workpointResume.targetObjects` | canonical |
| goal.hlt | `trajectoryView.hlt` | advisory |
| goal.mlg | `trajectoryView.mlg` | advisory |
| goal.stg | `trajectoryView.stg` | advisory |
| goal.gap | `trajectoryView.activeGap` | advisory |
| goal.waypoints | `trajectoryView.waypoints` | advisory |
| risk.blockers | `workpointResume.blockers` | canonical |
| risk.pressure | `contextPressure` | verified |
| risk.uiai | `uiaiState` | verified |
| risk.degraded | `workpointResume.degraded` or `trajectoryView.degraded` | verified |
| proof.evidence | `evidence.recentRefs` | evidence-backed |
| proof.gaps | `evidence.proofGaps` | advisory |
| recovery.next_tools | `toolGraph.topNextTools` | choreography |
| recovery.recovery_tools | `toolGraph.topRecoveryTools` | choreography |
| learning.predictions | `prediction.recentPredictions` | optional |
| learning.metacog | `metacog.recentLessons` | optional |
| tool.family | `toolGraph.families` | contract |
| tool.next_by_family | `toolGraph.nextToolsByFamily` | contract |

## 8. Mode Selection Algorithm

```typescript
function selectMode(input: AwarenessInput): Mode {
  // Priority 1: safety and authority signals
  if (!input.projectRootSafety.safe) return "standard";
  if (input.workpointResume?.actionAuthority === false) return "standard";

  // Priority 2: post-compaction always gets standard
  if (input.surface === "post_compaction") return "standard";

  // Priority 3: warnings get standard
  if (input.surface === "warning") return "standard";

  // Priority 4: operator asks for architecture/design
  if (input.operatorSteering.explicitSteer?.match(/architecture|design|explain/)) {
    return "rich";
  }

  // Priority 5: first-ever project onboarding
  if (input.sessionTransfer.action === "continue" && !input.sessionTransfer.resumeFound) {
    return "onboarding";
  }

  // Priority 6: high context pressure
  if (input.contextPressure.tier === "critical" || input.contextPressure.tier === "high") {
    return "minimal";
  }

  // Priority 7: canonical Workpoint present
  if (input.workpointResume?.canonical === true) {
    return "minimal";
  }

  // Priority 8: UIAI bridge
  if (input.surface === "uiai_bridge") return "standard";

  // Default
  return "standard";
}
```

## 9. ContextPressureState (Cadence/Dedupe)

```typescript
interface ContextPressureState {
  lastShownAt: number;             // timestamp ms
  lastPct: number;                // percentage at last shown
  lastTier: ContextPressureTier;
  lastAnchorState: string;         // Workpoint anchor at last shown
  compactionCountAtLastShown: number;
  transitionCount: number;         // number of state transitions shown
  suppressionCount: number;        // consecutive suppressions
}

function shouldShowPressureWarning(
  state: ContextPressureState,
  input: AwarenessInput
): { show: boolean; reason: string; escalation: "none" | "soft" | "hard" } {
  const now = Date.now();
  const pct = input.contextPressure.percentage;
  const tier = input.contextPressure.tier;
  const anchor = input.workpointResume?.workpointId ?? "none";
  const compCount = input.contextPressure.compactionCount;

  // Never show within 30 seconds of last shown
  if (state.lastShownAt && now - state.lastShownAt < 30_000) {
    return { show: false, reason: "within_30s_dedupe", escalation: "none" };
  }

  // Show if tier escalated
  const tierOrder = { low: 0, medium: 1, high: 2, critical: 3 };
  if (tierOrder[tier] > tierOrder[state.lastTier]) {
    return { show: true, reason: "tier_escalation", escalation: "hard" };
  }

  // Show if Workpoint anchor changed
  if (anchor !== state.lastAnchorState) {
    return { show: true, reason: "anchor_changed", escalation: "soft" };
  }

  // Show if percentage jumped by >20 and tier is high/critical
  if (pct - state.lastPct > 20 && (tier === "high" || tier === "critical")) {
    return { show: true, reason: "pct_jump", escalation: "soft" };
  }

  // Show if compaction count increased significantly
  if (compCount - state.compactionCountAtLastShown >= 3) {
    return { show: true, reason: "compaction_count_escalation", escalation: "hard" };
  }

  // After 5 minutes, re-show if still pressure
  if (state.lastShownAt && now - state.lastShownAt > 300_000 && pct > 50) {
    return { show: true, reason: "stale_reminder", escalation: "soft" };
  }

  return { show: false, reason: "no_state_change", escalation: "none" };
}
```

## 10. Tool-Family Selector

```typescript
interface ToolGuidance {
  toolName: string;
  family: string;
  whyIncluded: string;           // one sentence
  authorityValue: number;
  actionability: number;
  sideEffectRisk: "safe" | "moderate" | "risky";
  nextTools: string[];             // from choreography
}

function selectTopTools(
  input: AwarenessInput,
  count: number = 3
): ToolGuidance[] {
  const candidates: ToolGuidance[] = [];

  for (const tool of input.toolGraph.topNextTools) {
    const family = input.toolGraph.families[tool] ?? "unknown";
    const sideEffect = input.toolGraph.sideEffectProfiles[tool] ?? "unknown";

    // Map current blockers to relevant families
    const blockerRelevant = input.workpointResume?.blockers.some(b =>
      b.toLowerCase().includes(family) || b.toLowerCase().includes(tool)
    );

    candidates.push({
      toolName: tool,
      family,
      whyIncluded: blockerRelevant
        ? `directly relevant to current blocker`
        : `top next tool from choreography graph`,
      authorityValue: blockerRelevant ? 8 : 5,
      actionability: blockerRelevant ? 9 : 6,
      sideEffectRisk: sideEffectToRisk(sideEffect),
      nextTools: input.toolGraph.nextToolsByFamily[tool] ?? [],
    });
  }

  // Sort by authority + actionability, filter risky in minimal mode
  return candidates
    .filter(t => input.mode !== "minimal" || t.sideEffectRisk !== "risky")
    .sort((a, b) => (b.authorityValue + b.actionability) - (a.authorityValue + a.actionability))
    .slice(0, count);
}

function sideEffectToRisk(profile: string): "safe" | "moderate" | "risky" {
  if (profile.includes("write_state") || profile.includes("control_state")) return "risky";
  if (profile.includes("write_")) return "moderate";
  return "safe";
}
```

## 11. Handoff Composer

```typescript
interface HandoffPacket {
  workpoint: {
    id: string;
    mission: string;
    nextAction: string;
    targetObjects: string[];
    verifiedEvidence: string[];
    blockers: string[];
    doNotDrift: string[];
  };
  trajectory: {
    trajectoryId: string;
    hlt: string | null;
    mlg: string | null;
    stg: string | null;
    activeGap: string | null;
    waypoints: string[];
  } | null;
  sessionTransfer: {
    action: "save" | "continue" | "status";
    saved: boolean;
    resumeFound: boolean;
    continuityId: string;
  };
  contextCognition: {
    rehydrateId: string;
    score: number;
    evidenceRefs: string[];
  } | null;
  evidence: {
    recentRefs: string[];
    proofGaps: string[];
  };
  suppressedLines: string[];      // human-readable suppressed reasons
}

function composeHandoff(input: AwarenessInput): HandoffPacket {
  return {
    workpoint: input.workpointResume ? {
      id: input.workpointResume.workpointId,
      mission: input.workpointResume.mission,
      nextAction: input.workpointResume.nextAction,
      targetObjects: input.workpointResume.targetObjects,
      verifiedEvidence: input.workpointResume.verifiedEvidence,
      blockers: input.workpointResume.blockers,
      doNotDrift: input.workpointResume.doNotDrift,
    } : { id: "none", mission: "", nextAction: "", targetObjects: [], verifiedEvidence: [], blockers: [], doNotDrift: [] },

    trajectory: input.trajectoryView ? {
      trajectoryId: input.trajectoryView.trajectoryId,
      hlt: input.trajectoryView.hlt,
      mlg: input.trajectoryView.mlg,
      stg: input.trajectoryView.stg,
      activeGap: input.trajectoryView.activeGap,
      waypoints: input.trajectoryView.waypoints,
    } : null,

    sessionTransfer: {
      action: input.sessionTransfer.action,
      saved: input.sessionTransfer.saved,
      resumeFound: input.sessionTransfer.resumeFound,
      continuityId: input.sessionTransfer.continuityId,
    },

    contextCognition: input.contextCognition ? {
      rehydrateId: input.contextCognition.rehydrateId,
      score: input.contextCognition.score,
      evidenceRefs: input.contextCognition.evidenceRefs,
    } : null,

    evidence: {
      recentRefs: input.evidence.recentRefs,
      proofGaps: input.evidence.proofGaps,
    },

    suppressedLines: [], // populated by caller
  };
}
```

## 12. UIAI Proof/Risk Bridge

```typescript
interface UIAIProofCandidate {
  kind: "actual_browser_proof" | "blocked_browser_proof" | "private_url_guard_proof" | "missing_native_proof";
  result: string;
  evidenceRef: string;
  targetRef: string;
  actionability: number;
  riskReduction: number;
  proofValue: number;
}

function buildUIAIProofCandidates(input: AwarenessInput): UIAIProofCandidate[] {
  const candidates: UIAIProofCandidate[] = [];

  if (input.uiaiState.browserFailures > 0) {
    candidates.push({
      kind: "actual_browser_proof",
      result: `${input.uiaiState.browserFailures} browser failure(s) captured`,
      evidenceRef: `uiai_diagnostics:${input.uiaiState.browserFailures}`,
      targetRef: "uiai_browser_session",
      actionability: 8,
      riskReduction: 7,
      proofValue: 9,
    });
  }

  if (input.uiaiState.privateUrlBlocks > 0) {
    candidates.push({
      kind: "private_url_guard_proof",
      result: `${input.uiaiState.privateUrlBlocks} private URL block(s) captured`,
      evidenceRef: `uiai_private_url_block:${input.uiaiState.privateUrlBlocks}`,
      targetRef: "uiai_browser",
      actionability: 6,
      riskReduction: 5,
      proofValue: 7,
    });
  }

  if (input.uiaiState.saturated) {
    candidates.push({
      kind: "missing_native_proof",
      result: "UIAI pressure saturated; Focusa should use native tool fallback",
      evidenceRef: "uiai_saturated:true",
      targetRef: "uiai_engine",
      actionability: 7,
      riskReduction: 6,
      proofValue: 6,
    });
  }

  return candidates;
}
```

## 13. Implementation Sequence

| Step | Task | Focusa bead |
|---|---|---|
| 1 | Write typed schemas + algorithm design doc | `focusa-4jo5.3` ← **here** |
| 2 | Build `AwarenessInput` gatherer — read from state helpers | `focusa-4jo5.4` |
| 3 | Build `AwarenessCandidateLine` generator from source map | `focusa-4jo5.4` |
| 4 | Build DVS scorer + mode selector | `focusa-4jo5.4` |
| 5 | Build `AwarenessPacket` renderer per surface | `focusa-4jo5.4` |
| 6 | Build `ContextPressureState` dedupe engine | `focusa-4jo5.4` |
| 7 | Build tool-family selector | `focusa-4jo5.4` |
| 8 | Build handoff composer | `focusa-4jo5.4` |
| 9 | Build UIAI proof bridge | `focusa-4jo5.4` |
| 10 | Refresh tool descriptions/snippets from algorithm design | `focusa-4jo5.5` |
| 11 | Per-tool audit table | `focusa-4jo5.5` |
| 12 | Pi reload proof test | `focusa-4jo5.6` |
| 13 | Cache refresh / Jiti freshness proof | `focusa-4jo5.6` |

## 14. Open Questions Before Implementation

1. **Daemon vs. local**: Should `AwarenessInput` be gathered from daemon API calls (authoritative) or from local state helpers (low-latency)? Or a hybrid where canonical surfaces use daemon and advisory surfaces use local?

2. **DVS weights**: Are the weights in §6 correct? Should they be tunable per surface?

3. **Novelty window**: How many turns of "recent" determines novelty penalty? (suggest: 3 turns)

4. **Staleness thresholds**: What age makes a source "stale"? (suggest: Workpoint >5min, Trajectory >10min, tool graph >1hr)

5. **Onboarding mode**: Should it be a one-time dismissible flag, or always available?

6. **Mode selection override**: Should operators be able to force a mode (e.g., `/focusa mode rich`)?

7. **Cadence state persistence**: Should `ContextPressureState` survive compaction, or reset per session?

8. **Tool guidance volume**: How many tools in `nextTools`? (suggest: 1 exact + up to 3 candidates)

## 15. Design Sign-off

This design document is the authoritative reference for Spec108 §6 implementation. Before coding, read this document and the ecosystem audit. Do not re-derive the algorithm from scattered prose.

Schema versions:
- `AwarenessInput`: v1
- `AwarenessCandidateLine`: v1
- `AwarenessPacket`: v1
- `ContextPressureState`: v1
- `HandoffPacket`: v1
- `UIAIProofCandidate`: v1
