// apps/pi-extension/src/awareness-substrate.ts
// Spec108 §6 implementation — dynamic awareness line-scoring substrate
// Design ref: docs/current/FOCUSA_AWARENESS_ALGORITHM_DESIGN_2026-06-15.md

import {
  getAttachmentRuntime,
  getScopedWorkpointPacket,
  isProjectRootAuthoritySafe,
  normalizeProjectRoot,
  focusaFetch,
  getLastProjectIdentity,
} from "./state.js";

// ─── Type definitions ───────────────────────────────────────────────────────

export type AwarenessMode = "minimal" | "standard" | "rich" | "onboarding";
export type AwarenessSurface =
  | "reload"
  | "post_compaction"
  | "warning"
  | "tool_guidance"
  | "uiai_bridge"
  | "agent_preload"
  | "preload_fail"
  | "preload_remediation"
  | "preload_receipt";
export type AwarenessLayer =
  "identity" | "authority" | "mission" | "goal" | "risk" | "proof" | "recovery" | "learning" | "tool";

export interface AwarenessInput {
  // Authority layer
  projectIdentity: {
    projectRoot: string;
    activeWorktreeRoot: string;
    workingSubpathId: string;
    canonicalName: string;
    continuityId: string;
    sessionId: string;
    confidence: "high" | "medium" | "low";
    verified: boolean;
  };
  projectRootSafety: {
    safe: boolean;
    path: string;
    reason?: string;
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
  // Risk / pressure layer
  contextPressure: {
    percentage: number;
    tier: "low" | "medium" | "high" | "critical";
    compactionPending: boolean;
    compactionCount: number;
    lastCompactionAt?: number;
  };
  // Operator steering layer
  operatorSteering: {
    currentAsk: string;
    explicitSteeer?: string;
    scopeKind?: string;
  };
  // Tool ecosystem layer
  toolGraph: {
    totalTools: number;
    families: Record<string, number>;
    topNextTools: string[];
    topRecoveryTools: string[];
    nextToolsByFamily: Record<string, string[]>;
    sideEffectProfiles: Record<string, string>;
  };
  // Cadence state
  cadenceState: ContextPressureState | null;
  // Mode + surface
  mode: AwarenessMode;
  surface: AwarenessSurface;
}

export interface AwarenessCandidateLine {
  id: string;
  layer: AwarenessLayer;
  category: string;
  text: string;
  authorityValue: number;
  actionability: number;
  riskReduction: number;
  novelty: number;
  proofValue: number;
  redundancyPenalty: number;
  stalenessPenalty: number;
  dvs: number;
  modeAllowed: AwarenessMode[];
  surfaceAllowed: AwarenessSurface[];
  suppressReason?: string;
  sourceRef?: string;
  evidenceRef?: string;
}

export interface ContextPressureState {
  lastShownAt: number;
  lastPct: number;
  lastTier: "low" | "medium" | "high" | "critical";
  lastAnchorState: string;
  compactionCountAtLastShown: number;
  transitionCount: number;
  suppressionCount: number;
}

export interface ToolGuidance {
  toolName: string;
  family: string;
  whyIncluded: string;
  authorityValue: number;
  actionability: number;
  sideEffectRisk: "safe" | "moderate" | "risky";
  nextTools: string[];
}

export interface AwarenessPacket {
  schema: "focusa.awareness_packet.v1";
  generatedAt: number;
  mode: AwarenessMode;
  surface: AwarenessSurface;
  status: "fresh" | "degraded" | "partial";
  visibleLines: AwarenessCandidateLine[];
  systemLines: AwarenessCandidateLine[];
  nextTools: ToolGuidance[];
  recoveryTools: ToolGuidance[];
  suppressedLines: Array<{ line: AwarenessCandidateLine; suppressReason: string; dvs: number }>;
  metadata: {
    dvsCutoff: number;
    totalCandidates: number;
    visibleCount: number;
    suppressedCount: number;
    freshnessScore: number;
    authorityScore: number;
    confidence: "high" | "medium" | "low";
    modeReason: string;
    surfaceReason: string;
  };
  rehydrateId: string;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function contextTierFromString(tier: "" | "warn" | "auto" | "hard"): "low" | "medium" | "high" | "critical" {
  if (tier === "hard") return "critical";
  if (tier === "auto") return "high";
  if (tier === "warn") return "medium";
  return "low";
}

// ─── DVS formula constants ───────────────────────────────────────────────────

const DVS_WEIGHTS = {
  authorityValue: 3.0,
  actionability: 2.5,
  riskReduction: 2.0,
  novelty: 1.5,
  proofValue: 1.5,
  redundancyPenalty: 2.0,
  stalenessPenalty: 1.5,
};

const DVS_CUTOFF = { minimal: 7.0, standard: 4.0, rich: 1.5, onboarding: 0.5 };

// ─── AwarenessInput gatherer ─────────────────────────────────────────────────

export async function gatherAwarenessInput(surface: AwarenessSurface): Promise<AwarenessInput> {
  const scopedPacket = getScopedWorkpointPacket();
  const projectRoot = normalizeProjectRoot(
    scopedPacket?.project_root || getAttachmentRuntime().sessionCwd || process.cwd()
  );
  const authoritySafe = isProjectRootAuthoritySafe(projectRoot);
  const continuityId = scopedPacket
    ? String(scopedPacket.continuity_id || "")
    : String(getAttachmentRuntime().continuityId || "");
  const sessionId = scopedPacket
    ? String(scopedPacket.session_id || "")
    : String(getAttachmentRuntime().sessionFrameKey || "");

  // Trajectory view (async, non-blocking)
  let trajectoryView: AwarenessInput["trajectoryView"] = null;
  try {
    if (getAttachmentRuntime().focusaAvailable && authoritySafe) {
      const tv = await focusaFetch("/trajectory/view");
      if (tv && tv.ok) {
        trajectoryView = {
          trajectoryId: String(tv.trajectory_id || tv.trajectoryId || ""),
          canonical: Boolean(tv.canonical),
          degraded: Boolean(tv.degraded),
          hlt: tv.hlt || null,
          mlg: tv.mlg || null,
          stg: tv.stg || null,
          desiredEndState: tv.desired_end_state || tv.desiredEndState || null,
          activeGap: tv.active_gap || tv.activeGap || null,
          waypoints: Array.isArray(tv.waypoints) ? tv.waypoints : [],
          clarityGate: (tv.clarity_gate || tv.clarityGate || "provisional") as
            "clear" | "unclear" | "provisional",
          nextTools: Array.isArray(tv.next_tools || tv.nextTools) ? tv.next_tools || tv.nextTools : [],
        };
      }
    }
  } catch {
    /* non-blocking */
  }

  // Tool graph from tool-contracts
  const toolGraph = await gatherToolGraph();

  // Operator steering
  const currentAsk = getAttachmentRuntime().currentAsk?.text || "";

  // Select mode
  const mode = selectMode({
    projectRootSafety: { safe: authoritySafe },
    workpointResume: scopedPacket
      ? {
          workpointId: String(scopedPacket.workpoint_id || ""),
          canonical: Boolean(scopedPacket.canonical),
          degraded: Boolean(scopedPacket.degraded),
          mission: String(scopedPacket.mission || ""),
          nextAction: String(scopedPacket.next_slice || scopedPacket.next_action || ""),
          targetObjects: Array.isArray(scopedPacket.target_objects) ? scopedPacket.target_objects : [],
          verifiedEvidence: Array.isArray(scopedPacket.verified_evidence)
            ? scopedPacket.verified_evidence
            : [],
          blockers: Array.isArray(scopedPacket.blockers) ? scopedPacket.blockers : [],
          doNotDrift: [],
          actionAuthority: Boolean(scopedPacket.action_authority_for_current_ask ?? true),
          continuityId,
          sessionId,
        }
      : null,
    trajectoryView,
    surface,
    currentAsk,
    contextPressure: {
      percentage: getAttachmentRuntime().currentContextPct ?? 0,
      tier: contextTierFromString(getAttachmentRuntime().currentTier),
      compactionPending: false,
      compactionCount: getAttachmentRuntime().compactsThisHour,
    },
  });

  const cachedIdentity: any = getLastProjectIdentity() || {};
  const activeWorktreeRoot = normalizeProjectRoot(
    cachedIdentity.active_worktree_root || cachedIdentity.working_context?.active_worktree_root || projectRoot
  );
  const workingSubpathId = String(
    cachedIdentity.working_context?.working_subpath?.working_subpath_id || "primary"
  );

  return {
    projectIdentity: {
      projectRoot,
      activeWorktreeRoot,
      workingSubpathId,
      canonicalName: authoritySafe ? "focusa" : "unknown",
      continuityId,
      sessionId,
      confidence: authoritySafe ? "high" : "low",
      verified: authoritySafe,
    },
    projectRootSafety: {
      safe: authoritySafe,
      path: projectRoot,
      reason: authoritySafe ? undefined : "unsafe root path",
    },
    workpointResume: scopedPacket
      ? {
          workpointId: String(scopedPacket.workpoint_id || ""),
          canonical: Boolean(scopedPacket.canonical),
          degraded: Boolean(scopedPacket.degraded),
          mission: String(scopedPacket.mission || ""),
          nextAction: String(scopedPacket.next_slice || scopedPacket.next_action || ""),
          targetObjects: Array.isArray(scopedPacket.target_objects) ? scopedPacket.target_objects : [],
          verifiedEvidence: Array.isArray(scopedPacket.verified_evidence)
            ? scopedPacket.verified_evidence
            : [],
          blockers: Array.isArray(scopedPacket.blockers) ? scopedPacket.blockers : [],
          doNotDrift: [],
          actionAuthority: Boolean(scopedPacket.action_authority_for_current_ask ?? true),
          continuityId,
          sessionId,
        }
      : null,
    trajectoryView,
    contextPressure: {
      percentage: getAttachmentRuntime().currentContextPct ?? 0,
      tier: contextTierFromString(getAttachmentRuntime().currentTier),
      compactionPending: false,
      compactionCount: getAttachmentRuntime().compactsThisHour,
      lastCompactionAt: getAttachmentRuntime().lastCompactTime || undefined,
    },
    operatorSteering: { currentAsk },
    toolGraph,
    cadenceState: getAttachmentRuntime().awarenessCadenceState || null,
    mode,
    surface,
  };
}

// ─── Tool graph gatherer ─────────────────────────────────────────────────────

async function gatherToolGraph(): Promise<AwarenessInput["toolGraph"]> {
  const families: Record<string, number> = {};
  const sideEffectProfiles: Record<string, string> = {};
  const nextToolsByFamily: Record<string, string[]> = {};
  const allNextTools: string[] = [];
  const allRecoveryTools: string[] = [];

  try {
    const tc = await focusaFetch("/tool-contracts");
    if (tc && Array.isArray(tc.contracts)) {
      for (const contract of tc.contracts) {
        const name = String(contract.name || "");
        const family = String(contract.family || "unknown");
        const sideEffect = String(contract.side_effect_profile || "read_state");
        if (!name.startsWith("focusa_")) continue;
        families[name] = families[name] || 0;
        if (!families[family]) families[family] = 0;
        families[family]++;
        sideEffectProfiles[name] = sideEffect;
        if (Array.isArray(contract.next_tools)) {
          nextToolsByFamily[name] = contract.next_tools.slice(0, 3);
          for (const nt of contract.next_tools) {
            if (!allNextTools.includes(nt)) allNextTools.push(nt);
          }
        }
        if (contract.parity_status === "full" || contract.parity_status === "domain") {
          if (!allRecoveryTools.includes(name)) allRecoveryTools.push(name);
        }
      }
    }
  } catch {
    /* non-blocking */
  }

  return {
    totalTools: Object.keys(families).filter((k) => k.startsWith("focusa_")).length || 96,
    families,
    topNextTools: allNextTools.slice(0, 8),
    topRecoveryTools: allRecoveryTools.slice(0, 8),
    nextToolsByFamily,
    sideEffectProfiles,
  };
}

// ─── Mode selector ────────────────────────────────────────────────────────────

function selectMode(input: {
  projectRootSafety: { safe: boolean };
  workpointResume: AwarenessInput["workpointResume"];
  trajectoryView: AwarenessInput["trajectoryView"];
  surface: AwarenessSurface;
  currentAsk: string;
  contextPressure: AwarenessInput["contextPressure"];
}): AwarenessMode {
  const { projectRootSafety, workpointResume, trajectoryView, surface, currentAsk, contextPressure } = input;

  if (!projectRootSafety.safe) return "standard";
  if (workpointResume?.actionAuthority === false) return "standard";
  if (surface === "post_compaction") return "standard";
  if (surface === "warning") return "standard";
  if (currentAsk.match && currentAsk.match(/architecture|design|explain|how/)) return "rich";
  if (contextPressure.tier === "critical" || contextPressure.tier === "high") return "minimal";
  if (workpointResume?.canonical === true) return "minimal";
  if (surface === "uiai_bridge") return "standard";
  return "standard";
}

// ─── Candidate line generator ────────────────────────────────────────────────

export function generateCandidateLines(input: AwarenessInput): AwarenessCandidateLine[] {
  const lines: AwarenessCandidateLine[] = [];
  let id = 0;
  const now = Date.now();
  const wpStaleMs = 5 * 60 * 1000; // 5 min

  // ── Identity layer ────────────────────────────────────────────────────────
  lines.push(
    makeLine({
      id: String(++id),
      layer: "identity",
      category: "authority",
      text: `canonical_parent=${input.projectIdentity.projectRoot} working_root=${input.projectIdentity.activeWorktreeRoot} working_subpath=${input.projectIdentity.workingSubpathId}`,
      authorityValue: input.projectIdentity.verified ? 9 : 0,
      actionability: 5,
      riskReduction: input.projectRootSafety.safe ? 5 : 8,
      novelty: 5,
      proofValue: 3,
      redundancyPenalty: 0,
      stalenessPenalty: 0,
      modeAllowed: ["minimal", "standard", "rich", "onboarding"],
      surfaceAllowed: ["reload", "post_compaction", "warning", "tool_guidance", "uiai_bridge"],
      sourceRef: "state.getScopedWorkpointPacket + normalizeProjectRoot",
    })
  );

  if (!input.projectRootSafety.safe) {
    lines.push(
      makeLine({
        id: String(++id),
        layer: "risk",
        category: "safety",
        text: `⚠️ Unsafe root: ${input.projectRootSafety.path} — ${input.projectRootSafety.reason || "not a verified workspace"}`,
        authorityValue: 0,
        actionability: 9,
        riskReduction: 9,
        novelty: 9,
        proofValue: 5,
        redundancyPenalty: 0,
        stalenessPenalty: 0,
        modeAllowed: ["standard", "rich", "onboarding"],
        surfaceAllowed: ["reload", "warning"],
        sourceRef: "state.isProjectRootAuthoritySafe",
      })
    );
  }

  // ── Authority layer ────────────────────────────────────────────────────────
  if (input.workpointResume) {
    const wp = input.workpointResume;
    const isStale = wp.canonical && now - (getAttachmentRuntime().lastWorkpointUpdate || 0) > wpStaleMs;

    lines.push(
      makeLine({
        id: String(++id),
        layer: "authority",
        category: "workpoint",
        text: `Workpoint ${wp.workpointId} ${wp.canonical ? "✓ canonical" : "○ non-canonical"}${wp.degraded ? " [degraded]" : ""}`,
        authorityValue: wp.canonical ? 9 : 4,
        actionability: wp.actionAuthority ? 8 : 3,
        riskReduction: wp.degraded ? 7 : 0,
        novelty: 5,
        proofValue: 5,
        redundancyPenalty: 0,
        stalenessPenalty: isStale ? 4 : 0,
        modeAllowed: ["minimal", "standard", "rich"],
        surfaceAllowed: ["reload", "post_compaction"],
        sourceRef: "state.getScopedWorkpointPacket",
      })
    );

    if (wp.nextAction) {
      lines.push(
        makeLine({
          id: String(++id),
          layer: "mission",
          category: "next_action",
          text: `next: ${trunc(wp.nextAction, 200)}`,
          authorityValue: wp.canonical ? 8 : 3,
          actionability: 9,
          riskReduction: 4,
          novelty: 5,
          proofValue: 3,
          redundancyPenalty: 0,
          stalenessPenalty: isStale ? 3 : 0,
          modeAllowed: ["minimal", "standard", "rich"],
          surfaceAllowed: ["reload", "post_compaction"],
          sourceRef: "state.getScopedWorkpointPacket.next_slice",
        })
      );
    }

    if (wp.mission) {
      lines.push(
        makeLine({
          id: String(++id),
          layer: "mission",
          category: "mission",
          text: `mission: ${trunc(wp.mission, 300)}`,
          authorityValue: wp.canonical ? 7 : 2,
          actionability: 6,
          riskReduction: 3,
          novelty: 3,
          proofValue: 3,
          redundancyPenalty: 0,
          stalenessPenalty: isStale ? 2 : 0,
          modeAllowed: ["standard", "rich"],
          surfaceAllowed: ["reload", "post_compaction"],
          sourceRef: "state.getScopedWorkpointPacket.mission",
        })
      );
    }

    if (wp.blockers.length > 0) {
      for (const blocker of wp.blockers.slice(0, 3)) {
        lines.push(
          makeLine({
            id: String(++id),
            layer: "risk",
            category: "blocker",
            text: `blocker: ${trunc(blocker, 200)}`,
            authorityValue: wp.canonical ? 7 : 2,
            actionability: 8,
            riskReduction: 9,
            novelty: 6,
            proofValue: 5,
            redundancyPenalty: 0,
            stalenessPenalty: 0,
            modeAllowed: ["minimal", "standard", "rich"],
            surfaceAllowed: ["reload", "post_compaction", "warning"],
            sourceRef: "state.getScopedWorkpointPacket.blockers",
          })
        );
      }
    }

    if (wp.targetObjects.length > 0) {
      lines.push(
        makeLine({
          id: String(++id),
          layer: "authority",
          category: "targets",
          text: `active objects: ${wp.targetObjects.slice(0, 5).join(", ")}`,
          authorityValue: wp.canonical ? 6 : 2,
          actionability: 7,
          riskReduction: 3,
          novelty: 4,
          proofValue: 3,
          redundancyPenalty: 0,
          stalenessPenalty: 0,
          modeAllowed: ["standard", "rich"],
          surfaceAllowed: ["reload", "post_compaction"],
          sourceRef: "state.getScopedWorkpointPacket.target_objects",
        })
      );
    }
  }

  // ── Goal layer ────────────────────────────────────────────────────────────
  if (input.trajectoryView) {
    const tv = input.trajectoryView;
    if (tv.hlt) {
      lines.push(
        makeLine({
          id: String(++id),
          layer: "goal",
          category: "hlt",
          text: `HLT: ${trunc(tv.hlt, 200)}`,
          authorityValue: 5,
          actionability: 4,
          riskReduction: 3,
          novelty: 3,
          proofValue: 3,
          redundancyPenalty: 0,
          stalenessPenalty: 2,
          modeAllowed: ["rich"],
          surfaceAllowed: ["reload", "post_compaction"],
          sourceRef: "/trajectory/view.hlt",
        })
      );
    }
    if (tv.activeGap) {
      lines.push(
        makeLine({
          id: String(++id),
          layer: "goal",
          category: "gap",
          text: `active gap: ${trunc(tv.activeGap, 200)}`,
          authorityValue: 6,
          actionability: 7,
          riskReduction: 6,
          novelty: 5,
          proofValue: 4,
          redundancyPenalty: 0,
          stalenessPenalty: 2,
          modeAllowed: ["standard", "rich"],
          surfaceAllowed: ["reload", "post_compaction"],
          sourceRef: "/trajectory/view.active_gap",
        })
      );
    }
    if (tv.stg) {
      lines.push(
        makeLine({
          id: String(++id),
          layer: "goal",
          category: "stg",
          text: `STG: ${trunc(tv.stg, 200)}`,
          authorityValue: 5,
          actionability: 5,
          riskReduction: 4,
          novelty: 3,
          proofValue: 3,
          redundancyPenalty: 0,
          stalenessPenalty: 2,
          modeAllowed: ["rich"],
          surfaceAllowed: ["reload", "post_compaction"],
          sourceRef: "/trajectory/view.stg",
        })
      );
    }
  }

  // ── Context pressure layer ────────────────────────────────────────────────
  const cp = input.contextPressure;
  if (cp.tier === "high" || cp.tier === "critical") {
    lines.push(
      makeLine({
        id: String(++id),
        layer: "risk",
        category: "pressure",
        text: `context pressure: ${cp.percentage.toFixed(0)}% (${cp.tier}) — ${cp.compactionPending ? "compaction pending" : "compaction available"}`,
        authorityValue: 0,
        actionability: cp.compactionPending ? 8 : 5,
        riskReduction: 8,
        novelty: 5,
        proofValue: 4,
        redundancyPenalty: 0,
        stalenessPenalty: 0,
        modeAllowed: ["minimal", "standard", "rich"],
        surfaceAllowed: ["warning", "reload"],
        sourceRef: "state.contextPressure",
      })
    );
  }

  // ── Recovery / tool layer ─────────────────────────────────────────────────
  const topTools = input.toolGraph.topNextTools.slice(0, 3);
  if (topTools.length > 0 && (input.mode === "standard" || input.mode === "rich")) {
    lines.push(
      makeLine({
        id: String(++id),
        layer: "tool",
        category: "next_tools",
        text: `next tools: ${topTools.join(", ")}`,
        authorityValue: 4,
        actionability: 6,
        riskReduction: 3,
        novelty: 4,
        proofValue: 2,
        redundancyPenalty: 0,
        stalenessPenalty: 1,
        modeAllowed: ["standard", "rich"],
        surfaceAllowed: ["reload", "tool_guidance"],
        sourceRef: "tool-contracts.top_next_tools",
      })
    );
  }

  return lines;
}

function makeLine(
  partial: Partial<AwarenessCandidateLine> & { id: string; text: string }
): AwarenessCandidateLine {
  const authorityValue = partial.authorityValue ?? 0;
  const actionability = partial.actionability ?? 0;
  const riskReduction = partial.riskReduction ?? 0;
  const novelty = partial.novelty ?? 0;
  const proofValue = partial.proofValue ?? 0;
  const redundancyPenalty = partial.redundancyPenalty ?? 0;
  const stalenessPenalty = partial.stalenessPenalty ?? 0;
  const dvs =
    authorityValue * DVS_WEIGHTS.authorityValue +
    actionability * DVS_WEIGHTS.actionability +
    riskReduction * DVS_WEIGHTS.riskReduction +
    novelty * DVS_WEIGHTS.novelty +
    proofValue * DVS_WEIGHTS.proofValue -
    (redundancyPenalty * DVS_WEIGHTS.redundancyPenalty + stalenessPenalty * DVS_WEIGHTS.stalenessPenalty);
  return {
    layer: partial.layer ?? "recovery",
    category: partial.category ?? "general",
    authorityValue,
    actionability,
    riskReduction,
    novelty,
    proofValue,
    redundancyPenalty,
    stalenessPenalty,
    dvs,
    modeAllowed: partial.modeAllowed ?? ["standard"],
    surfaceAllowed: partial.surfaceAllowed ?? ["reload"],
    suppressReason: partial.suppressReason,
    sourceRef: partial.sourceRef,
    evidenceRef: partial.evidenceRef,
    ...partial,
  };
}

// ─── Line scorer ─────────────────────────────────────────────────────────────

function scoreLines(
  lines: AwarenessCandidateLine[],
  mode: AwarenessMode,
  surface: AwarenessSurface
): AwarenessCandidateLine[] {
  const cutoff = DVS_CUTOFF[mode] ?? 4.0;
  return lines
    .filter((line) => line.modeAllowed.includes(mode) && line.surfaceAllowed.includes(surface))
    .filter((line) => {
      // Authority gate: show if authorityValue is very high even if DVS is low
      if (line.authorityValue >= 8) return true;
      return line.dvs >= cutoff;
    });
}

// ─── Tool-family selector ────────────────────────────────────────────────────

export function selectTopTools(input: AwarenessInput, count = 3): ToolGuidance[] {
  const candidates: ToolGuidance[] = [];
  const blockers = input.workpointResume?.blockers ?? [];

  // Iterate over tool names from topNextTools, keyed by tool name in sideEffectProfiles
  for (const toolName of input.toolGraph.topNextTools.slice(0, 12)) {
    const sideEffect = input.toolGraph.sideEffectProfiles[toolName] ?? "read_state";
    const blockerRelevant = blockers.some((b) => b.toLowerCase().includes(toolName));

    candidates.push({
      toolName,
      family: "tool",
      whyIncluded: blockerRelevant
        ? "directly relevant to current blocker"
        : "top next tool from choreography",
      authorityValue: blockerRelevant ? 8 : 5,
      actionability: blockerRelevant ? 9 : 6,
      sideEffectRisk: sideEffectToRisk(sideEffect),
      nextTools: input.toolGraph.nextToolsByFamily[toolName] ?? [],
    });
  }

  // Add top family representative tools for diversity
  const topFamilies = Object.entries(input.toolGraph.families)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
    .map(([fam]) => fam);
  for (const family of topFamilies) {
    const byFamily = input.toolGraph.nextToolsByFamily[family];
    if (byFamily && byFamily.length > 0) {
      const toolName = byFamily[0];
      if (!candidates.find((c) => c.toolName === toolName)) {
        const sideEffect = input.toolGraph.sideEffectProfiles[toolName] ?? "read_state";
        candidates.push({
          toolName,
          family,
          whyIncluded: `top tool in ${family} family (${input.toolGraph.families[family] ?? 0} tools)`,
          authorityValue: 4,
          actionability: 5,
          sideEffectRisk: sideEffectToRisk(sideEffect),
          nextTools: input.toolGraph.nextToolsByFamily[toolName] ?? [],
        });
      }
    }
  }

  return candidates
    .filter((t) => input.mode !== "minimal" || t.sideEffectRisk !== "risky")
    .sort((a, b) => b.authorityValue + b.actionability - (a.authorityValue + a.actionability))
    .slice(0, count);
}

function sideEffectToRisk(profile: string): "safe" | "moderate" | "risky" {
  if (profile.includes("write_state") || profile.includes("control_state")) return "risky";
  if (profile.includes("write_")) return "moderate";
  return "safe";
}

// ─── Context pressure dedupe ─────────────────────────────────────────────────

export function shouldShowPressureWarning(
  state: ContextPressureState | null,
  input: AwarenessInput
): { show: boolean; reason: string; escalation: "none" | "soft" | "hard" } {
  const now = Date.now();
  const pct = input.contextPressure.percentage;
  const tier = input.contextPressure.tier;
  const anchor = input.workpointResume?.workpointId ?? "none";
  const compCount = input.contextPressure.compactionCount;

  if (!state) {
    return { show: true, reason: "first_warning", escalation: tier === "critical" ? "hard" : "soft" };
  }

  // Never show within 30 seconds
  if (state.lastShownAt && now - state.lastShownAt < 30_000) {
    return { show: false, reason: "within_30s_dedupe", escalation: "none" };
  }

  const tierOrder: Record<string, number> = { low: 0, medium: 1, high: 2, critical: 3 };
  if (tierOrder[tier] > tierOrder[state.lastTier]) {
    return { show: true, reason: "tier_escalation", escalation: "hard" };
  }

  if (anchor !== state.lastAnchorState) {
    return { show: true, reason: "anchor_changed", escalation: "soft" };
  }

  if (pct - state.lastPct > 20 && (tier === "high" || tier === "critical")) {
    return { show: true, reason: "pct_jump", escalation: "soft" };
  }

  if (compCount - state.compactionCountAtLastShown >= 3) {
    return { show: true, reason: "compaction_count_escalation", escalation: "hard" };
  }

  // After 5 minutes, re-show if still pressure
  if (state.lastShownAt && now - state.lastShownAt > 300_000 && pct > 50) {
    return { show: true, reason: "stale_reminder", escalation: "soft" };
  }

  return { show: false, reason: "no_state_change", escalation: "none" };
}

export function updateCadenceState(
  state: ContextPressureState | null,
  input: AwarenessInput,
  showed: boolean
): ContextPressureState {
  const now = Date.now();
  if (!state) {
    return {
      lastShownAt: showed ? now : 0,
      lastPct: input.contextPressure.percentage,
      lastTier: input.contextPressure.tier,
      lastAnchorState: input.workpointResume?.workpointId ?? "none",
      compactionCountAtLastShown: showed ? input.contextPressure.compactionCount : 0,
      transitionCount: showed ? 1 : 0,
      suppressionCount: showed ? 0 : 1,
    };
  }
  return {
    lastShownAt: showed ? now : state.lastShownAt,
    lastPct: input.contextPressure.percentage,
    lastTier: input.contextPressure.tier,
    lastAnchorState: input.workpointResume?.workpointId ?? state.lastAnchorState,
    compactionCountAtLastShown: showed
      ? input.contextPressure.compactionCount
      : state.compactionCountAtLastShown,
    transitionCount: showed ? state.transitionCount + 1 : state.transitionCount,
    suppressionCount: showed ? 0 : state.suppressionCount + 1,
  };
}

// ─── AwarenessPacket builder ─────────────────────────────────────────────────

export async function buildAwarenessPacket(surface: AwarenessSurface): Promise<AwarenessPacket> {
  const input = await gatherAwarenessInput(surface);
  const candidates = generateCandidateLines(input);
  const scored = scoreLines(candidates, input.mode, surface);
  const suppressed = candidates.filter((c) => !scored.includes(c));
  const nextTools = selectTopTools(input, 3);
  const recoveryTools = selectTopTools(input, 5).filter((t) => t.sideEffectRisk !== "risky");

  const modeReason = `mode=${input.mode}; safety=${input.projectRootSafety.safe}; canonical=${input.workpointResume?.canonical ?? false}; pressure=${input.contextPressure.tier}`;
  const surfaceReason = `surface=${surface}`;

  // Authority score
  let authorityScore = 50;
  if (input.workpointResume?.canonical) authorityScore += 30;
  if (input.projectRootSafety.safe) authorityScore += 20;
  authorityScore = Math.min(100, authorityScore);

  // Freshness score
  let freshnessScore = 70;
  if (input.workpointResume?.canonical) freshnessScore += 15;
  if (input.trajectoryView?.canonical) freshnessScore += 10;
  freshnessScore = Math.min(100, freshnessScore);

  const rehydrateId = `awareness:${surface}:${Date.now()}`;

  return {
    schema: "focusa.awareness_packet.v1",
    generatedAt: Date.now(),
    mode: input.mode,
    surface,
    status:
      input.projectRootSafety.safe && input.workpointResume?.canonical
        ? "fresh"
        : input.workpointResume?.degraded
          ? "partial"
          : "degraded",
    visibleLines: scored,
    systemLines: scored.filter((l) => l.layer !== "tool" || input.mode === "rich"),
    nextTools,
    recoveryTools,
    suppressedLines: suppressed.map((line) => ({
      line,
      suppressReason: line.suppressReason ?? `DVS below cutoff (${DVS_CUTOFF[input.mode]})`,
      dvs: line.dvs,
    })),
    metadata: {
      dvsCutoff: DVS_CUTOFF[input.mode],
      totalCandidates: candidates.length,
      visibleCount: scored.length,
      suppressedCount: suppressed.length,
      freshnessScore,
      authorityScore,
      confidence: authorityScore >= 80 ? "high" : authorityScore >= 50 ? "medium" : "low",
      modeReason,
      surfaceReason,
    },
    rehydrateId,
  };
}

// ─── Text renderer for a packet ──────────────────────────────────────────────

export function renderAwarenessPacketText(packet: AwarenessPacket): string {
  const authority = packet.visibleLines.find(
    (line) => line.category === "workpoint" || line.category === "authority"
  );
  const mission = packet.visibleLines.find((line) => line.category === "mission");
  const next = packet.visibleLines.find((line) => line.category === "next_action");
  const risk = packet.visibleLines.find((line) => line.category === "blocker" || line.layer === "risk");
  const tools = packet.nextTools.slice(0, 2).map((tool) => tool.toolName);
  return [
    "# Focusa",
    `Status: ${packet.status} · confidence=${packet.metadata.confidence}`,
    authority ? `Scope: ${authority.text}` : "Scope: verification required for durable writes",
    mission ? `Mission: ${mission.text}` : "Mission: follow the newest operator request",
    next ? `Next: ${next.text}` : "Next: continue the newest operator request",
    risk ? `Risk: ${risk.text}` : "Boundary: operator steering leads; scoped tools enforce durable writes",
    tools.length ? `Tools: ${tools.join(" · ")}` : "",
  ]
    .filter(Boolean)
    .join("\n");
}

// ─── Utility ─────────────────────────────────────────────────────────────────

function trunc(text: string, max: number): string {
  const t = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  return t.length > max ? `${t.slice(0, Math.max(0, max - 1))}…` : t;
}
