// Compaction handlers + tier logic + micro-compact
// Spec: §20 (tier), §21 (micro-compact), §25.7 (non-canonical), §33.1 (ASCC),
//        §33.10 (customInstructions), §35.6 (files), §38.1 (trim)

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { requestCoordinatedCompaction } from "./auto-compaction.js";
import { buildProjectWorkstreamKey, scopedQueryParams, type WorkstreamKey } from "./scoped-state.js";
import {
  getAttachmentRuntime,
  focusaFetch,
  getFocusState,
  buildCompactInstructions,
  persistState,
  persistAuthoritativeState,
  sanitizeFocusFailures,
  getScopedWorkpointPacket,
  isWorkpointPacketScopedToCurrentSession,
  isProjectRootAuthoritySafe,
  projectRootAuthorityFailure,
  normalizeWorkpointResumePacketEnvelope,
  normalizeProjectRoot,
  refreshTrajectoryClarityLifecycle,
  stampWorkpointPacketForCurrentPiSession,
  isExplicitContinuationAsk,
  isNonTaskStatusLikeText,
  buildAttentionRecallVerdict,
  formatAttentionRecallFocusSliceLines,
  toolOutputVisibleRecapReason,
  formatToolOutputVisibleRecapLines,
  formatProjectSwitchLedgerLines,
  buildCurrentAskScopeVerdict,
  formatCurrentAskScopeVerdictLines,
  getTurnCount,
  getActiveFrameId,
  getContinuityId,
  getSessionFrameKey,
  getSessionCwd,
  getActiveWorkpointPacket,
  setActiveWorkpointPacket,
  getActiveWorkpointSummary,
  setActiveWorkpointSummary,
  getLastTrajectoryClarity,
  setLastTrajectoryClarity,
  getLastProjectVerify,
  getLatestReportSummary,
  setLatestReportSummary,
  getTotalCompactions,
  incrementTotalCompactions,
} from "./state.js";
import { pushDelta } from "./tools.js";

function basename(value: string): string {
  const parts = String(value || "")
    .split("/")
    .filter(Boolean);
  return parts[parts.length - 1] || String(value || "file");
}

function normalizeCompactionArtifacts(
  files: any[]
): Array<{ kind: "file"; label: string; path_or_id: string }> {
  return (Array.isArray(files) ? files : [])
    .map((file) => String(file || "").trim())
    .filter(Boolean)
    .slice(0, 20)
    .map((file) => ({ kind: "file" as const, label: basename(file), path_or_id: file }));
}

function compactLines(values: any, mapper?: (value: any) => string): string[] {
  return (Array.isArray(values) ? values : [])
    .map((value) => (mapper ? mapper(value) : String(value || "").trim()))
    .filter(Boolean);
}

function packetField(packet: any, key: string): string {
  return String(packet?.[key] || "").trim();
}

type CompactionMemorySample = {
  at: number;
  rssBytes: number;
  heapUsedBytes: number;
  externalBytes: number;
};

function compactionMemorySample(): CompactionMemorySample {
  const usage = process.memoryUsage();
  return {
    at: Date.now(),
    rssBytes: usage.rss,
    heapUsedBytes: usage.heapUsed,
    externalBytes: usage.external,
  };
}

function scheduleCompactionMemoryEvaluation() {
  const before = (getAttachmentRuntime() as any).compactionMemoryBefore as CompactionMemorySample | undefined;
  if (!before) return;
  setTimeout(() => {
    const after = compactionMemorySample();
    const warningMiB = Number(process.env.FOCUSA_PI_COMPACTION_RSS_WARN_MIB || 2500);
    const rssWarnBytes = Math.max(512, warningMiB) * 1024 * 1024;
    const rssRatio = before.rssBytes > 0 ? after.rssBytes / before.rssBytes : 0;
    const heapRatio = before.heapUsedBytes > 0 ? after.heapUsedBytes / before.heapUsedBytes : 0;
    const retainedUnderPressure = after.rssBytes >= rssWarnBytes && rssRatio >= 0.9;
    (getAttachmentRuntime() as any).lastCompactionMemory = {
      schema: "focusa.compaction_memory_verdict.v1",
      before,
      after,
      rssRatio,
      heapRatio,
      status: retainedUnderPressure ? "warn_retained_under_pressure" : "within_budget",
      warningThresholdMiB: warningMiB,
    };
    delete (getAttachmentRuntime() as any).compactionMemoryBefore;
    persistState();
    if (retainedUnderPressure) {
      console.warn(
        `[focusa] compaction retained ${Math.round(after.rssBytes / 1024 / 1024)} MiB RSS; checkpoint and start a bounded fresh Pi session before host OOM`
      );
    }
  }, 5_000);
}

function compactText(value: unknown, fallback = "unknown", max = 180): string {
  const text = String(value ?? "")
    .replace(/\s+/g, " ")
    .trim();
  if (!text) return fallback;
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

function currentCompactionScope(): WorkstreamKey | null {
  const projectRoot = normalizeProjectRoot(getSessionCwd());
  const continuityId = String(getContinuityId() || "").trim();
  if (!isProjectRootAuthoritySafe(projectRoot) || !continuityId) return null;
  return buildProjectWorkstreamKey(projectRoot, continuityId);
}

async function buildLearningCompactionCard(
  currentAsk: string,
  mission: string,
  nextSlice: string
): Promise<string> {
  if (!getAttachmentRuntime().focusaAvailable) {
    return [
      "## Learning Loop",
      "- Predictive/metacog context unavailable because Focusa is offline.",
      "- End-of-task report still should record: task summary, prediction outcome, reusable lesson, and next possibility.",
    ].join("\n");
  }
  const ask = compactText(currentAsk || nextSlice || mission || "current session", "current session", 240);
  const lines: string[] = [
    "## Task Summary",
    `- Mission: ${compactText(mission || ask)}`,
    `- Current/next slice: ${compactText(nextSlice || ask)}`,
  ];
  try {
    const scope = currentCompactionScope();
    const stats = scope
      ? await focusaFetch(`/predictions/stats?${scopedQueryParams(scope).toString()}`)
      : null;
    if (stats?.authority?.status === "accepted") {
      const predictionStats = stats.data || {};
      lines.push("## Predictive Context");
      lines.push(
        `- Stats: total=${stats.total_predictions ?? stats.total ?? "unknown"}; evaluated=${stats.evaluated_predictions ?? stats.evaluated ?? "unknown"}; accuracy=${compactText(stats.global_accuracy ?? stats.accuracy ?? "unknown", "unknown", 60)}`
      );
      lines.push(
        "- At end-of-task: evaluate relevant open predictions, then record the next bounded prediction."
      );
    }
  } catch {
    lines.push(
      "## Predictive Context",
      "- Prediction stats unavailable; call focusa_predict_recent/stats before final task report when possible."
    );
  }
  try {
    const metacog = await focusaFetch("/metacognition/retrieve", {
      method: "POST",
      body: JSON.stringify({
        current_ask: ask,
        scope_tags: ["end_of_task", "compaction", "trajectory_review"],
        k: 3,
      }),
    });
    const candidates = Array.isArray(metacog?.candidates) ? metacog.candidates : [];
    lines.push("## Metacog Context");
    if (candidates.length) {
      candidates
        .slice(0, 3)
        .forEach((c: any, i: number) =>
          lines.push(`- Lesson ${i + 1}: ${compactText(c.content || c.kind || c.capture_id, "signal", 180)}`)
        );
    } else {
      lines.push("- No matching lessons; capture one at end-of-task if outcome teaches a reusable strategy.");
    }
    lines.push(
      "- At end-of-task: capture/refine reusable lesson only when evidence or outcome changed future behavior."
    );
  } catch {
    lines.push(
      "## Metacog Context",
      "- Metacog retrieve unavailable; run focusa_metacog_doctor/retrieve in trajectory review or wrap-up."
    );
  }
  lines.push(
    "## Possibilities",
    "- Next possibilities should be framed as bounded predictions + trajectory gaps, not vague brainstorms."
  );
  return lines.join("\n");
}

function semanticCurrentAsk(): string {
  const text = String(getAttachmentRuntime().currentAsk?.text || "").trim();
  if (!text || isExplicitContinuationAsk(text) || isNonTaskStatusLikeText(text)) return "";
  return text;
}

function renderCompactionMissionPacket(packet: any): string {
  const trajectory = packet?.trajectory || {};
  const workpoint = packet?.workpoint || {};
  const scope = packet?.scope || {};
  const next = packet?.next || {};
  const warnings = compactLines(trajectory?.warnings).slice(0, 6);
  return [
    "## CompactionMissionPacket",
    `STATUS: ${compactText(packet?.status, "blocked", 32)}`,
    `SCOPE_STATUS: ${compactText(scope?.scope_status, "missing", 32)}`,
    `HLT_STATUS: ${compactText(trajectory?.hlt_status, "missing_required", 48)}`,
    `ACTION_AUTHORITY_FROM_TRAJECTORY: ${trajectory?.action_authority_from_trajectory === true}`,
    `WORKPOINT_STATUS: ${compactText(workpoint?.status, "missing", 32)}`,
    `WORKPOINT_ACTION_AUTHORITY: ${workpoint?.action_authority === true}`,
    `HLT: ${compactText(trajectory?.hlt, "missing", 300)}`,
    `MISSION: ${compactText(workpoint?.mission, "missing", 300)}`,
    `NEXT_SLICE: ${compactText(workpoint?.next_slice, "missing", 300)}`,
    `EXACT_NEXT_TOOL: ${compactText(next?.exact_next_tool, "focusa_workpoint_resume", 80)}`,
    `WARNINGS: ${warnings.length ? warnings.join(" | ") : "none"}`,
    `PACKET_ID: ${compactText(packet?.packet_id, "missing", 80)}`,
    "AUTHORITY: advisory packet only; Trajectory, Workpoint, Focus State, and evidence remain canonical.",
    "DO_NOT_USE: transcript tail, raw tool history, or generic trajectory as authority.",
  ].join("\n");
}

async function buildCompactionMissionPacket(resumeSource: string): Promise<any | null> {
  if (!getAttachmentRuntime().focusaAvailable) return null;
  const scope = currentCompactionScope();
  if (!scope) return null;
  try {
    const visibleRecapReason = toolOutputVisibleRecapReason();
    const packet = await focusaFetch("/compaction/build", {
      method: "POST",
      body: JSON.stringify({
        resume_source: resumeSource,
        scope,
        project_root: scope.root_scope.root_path,
        continuity_id: scope.continuity_id,
        session_id: getSessionFrameKey() || undefined,
        current_ask: semanticCurrentAsk() || undefined,
        ask_kind: getAttachmentRuntime().currentAsk?.kind || "unknown",
        source_turn_id: `pi-turn-${getTurnCount()}`,
        omitted_sections: visibleRecapReason ? ["raw_tool_history"] : [],
        rehydrate_refs: ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"],
      }),
    });
    if (packet?.schema_version !== "focusa.compaction_mission_packet.v1") return null;
    (getAttachmentRuntime() as any).lastCompactionMissionPacket = packet;
    return packet;
  } catch {
    return null;
  }
}

async function buildCompactionFallbackSummary(fs: any, workpointPacket: any): Promise<string> {
  const candidatePacket =
    normalizeWorkpointResumePacketEnvelope(workpointPacket) || getScopedWorkpointPacket() || {};
  const packet = isWorkpointPacketScopedToCurrentSession(candidatePacket) ? candidatePacket : {};
  const rendered = String(
    Object.keys(packet).length ? packet?.rendered_summary || getActiveWorkpointSummary() || "" : ""
  ).trim();
  const ask = semanticCurrentAsk();
  const mission =
    packetField(packet, "mission") ||
    ask ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().lastFocusSnapshot.intent ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    getAttachmentRuntime().activeFrameTitle;
  const nextSlice =
    packetField(packet, "next_slice") ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    getAttachmentRuntime().lastCompactDecision ||
    ask ||
    getAttachmentRuntime().activeFrameGoal;
  const currentFocus =
    fs?.current_focus ||
    fs?.current_state ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    nextSlice ||
    mission;
  const decisions = compactLines(fs?.decisions)
    .concat(getAttachmentRuntime().localDecisions.slice(-5))
    .filter((v, i, a) => a.indexOf(v) === i);
  const constraints = compactLines(fs?.constraints)
    .concat(getAttachmentRuntime().localConstraints.slice(-5))
    .filter((v, i, a) => a.indexOf(v) === i);
  const failures = compactLines(sanitizeFocusFailures(fs?.failures || []))
    .concat(sanitizeFocusFailures(getAttachmentRuntime().localFailures).slice(-3))
    .filter((v, i, a) => a.indexOf(v) === i);
  const nextSteps = compactLines(fs?.next_steps);
  if (nextSlice) nextSteps.unshift(`Continue from Workpoint next_slice: ${nextSlice}`);
  const blockers = compactLines(packet?.blockers, (b) => String(b?.reason || b || "").trim());
  const openQuestions = compactLines(fs?.open_questions);
  const recentResults = compactLines(fs?.recent_results);
  compactLines(packet?.verification_records, (r) =>
    String(r?.result || r?.evidence_ref || "").trim()
  ).forEach((r) => recentResults.push(`Verified evidence: ${r}`));
  if (packetField(packet, "workpoint_id"))
    recentResults.push(`Canonical Workpoint available: ${packetField(packet, "workpoint_id")}`);
  const artifactLines = compactLines(
    fs?.artifacts,
    (a) =>
      `${a?.kind || "artifact"}:${a?.label || a?.path_or_id || "unlabeled"}${a?.path_or_id ? "@" + a.path_or_id : ""}`
  );
  compactLines(packet?.active_object_refs).forEach((ref) => artifactLines.push(`active_object:${ref}`));
  if (packetField(packet, "project_root"))
    artifactLines.push(`project_root:${packetField(packet, "project_root")}`);
  if (packetField(packet, "session_id"))
    artifactLines.push(`session_id:${packetField(packet, "session_id")}`);
  const notes = compactLines(fs?.notes);
  if (!decisions.length && mission) decisions.push(`Continuation anchored to mission: ${mission}`);
  if (!constraints.length && packetField(packet, "project_root"))
    constraints.push(`Resume scope is bound to project_root ${packetField(packet, "project_root")}`);
  if (!openQuestions.length) openQuestions.push("No open questions recorded by Focusa or Workpoint.");
  if (!blockers.length) blockers.push("No open blockers recorded by Focusa or Workpoint.");
  if (!failures.length) failures.push("No active failure records in Focusa state.");
  if (!notes.length && rendered) notes.push(`Workpoint summary: ${rendered}`);
  const bullet = (items: string[]) =>
    items.length
      ? items
          .slice(0, 12)
          .map((x) => `- ${x}`)
          .join("\n")
      : "- Not populated by Focusa; no safe related fallback available.";
  const learningCard = await buildLearningCompactionCard(ask, mission, nextSlice);
  const v2Prompt = formatResumePacketV2ForPrompt(packet);
  const visibleRecapReason = toolOutputVisibleRecapReason();
  const attentionSection = [
    "# Attention Recall Anchor",
    ...formatAttentionRecallFocusSliceLines(
      buildAttentionRecallVerdict({
        focusState: fs,
        workpointPacket: packet,
        currentAskText: ask,
        currentAskKind: getAttachmentRuntime().currentAsk?.kind,
        queryScopeKind: getAttachmentRuntime().queryScope?.scopeKind,
        projectRoot: getSessionCwd(),
        continuityId: getContinuityId(),
        visibleRecapReason,
      })
    ),
    ...formatCurrentAskScopeVerdictLines(
      buildCurrentAskScopeVerdict({
        currentAskText: ask,
        workpointPacket: packet,
        projectRoot: getSessionCwd(),
        continuityId: getContinuityId(),
      })
    ),
    ...formatToolOutputVisibleRecapLines(visibleRecapReason),
    "",
  ].join("\n");
  const workpointSection = v2Prompt ? ["# Workpoint Resume Packet", v2Prompt, ""].join("\n") : "";
  const trajectorySection = formatTrajectoryPacketForPrompt(getLastTrajectoryClarity())
    ? ["# Trajectory Resume Packet", formatTrajectoryPacketForPrompt(getLastTrajectoryClarity()), ""].join(
        "\n"
      )
    : "";
  const projectLedgerSection = formatProjectSwitchLedgerLines(ask).length
    ? [
        "# Project Switch Ledger",
        ...formatProjectSwitchLedgerLines(ask).map((value) => `- ${value}`),
        "",
      ].join("\n")
    : "";
  return [
    attentionSection,
    workpointSection,
    trajectorySection,
    projectLedgerSection,
    "# Focusa Cognitive Summary",
    `## Intent\n${fs?.intent || mission || "Continue current operator-directed work."}`,
    `## Current Focus\n${currentFocus || "Continue current operator-directed work."}`,
    `## Decisions Made\n${bullet(decisions)}`,
    `## Active Constraints\n${bullet(constraints)}`,
    `## Failures Encountered\n${bullet(failures)}`,
    `## Next Steps\n${bullet(nextSteps.length ? nextSteps : ["Continue with the next bounded action from the canonical Workpoint/current operator ask."])}`,
    `## Open Questions\n${bullet(openQuestions)}`,
    `## Recent Results\n${bullet(recentResults.length ? recentResults : ["No recent_results slot entries; use Workpoint packet, git/beads, and evidence docs as the related fallback source."])}`,
    `## Artifacts\n${bullet(artifactLines.length ? artifactLines : ["No artifact slot entries; use active project root and Workpoint refs as fallback anchors."])}`,
    `## Notes\n${bullet(notes.length ? notes : ["Fallback summary hydrated from Workpoint, Focus State shadow, current ask, and session metadata."])}`,
    learningCard,
    "## End-of-task Report Contract\n- Include task summary, evidence/proof, prediction outcome/evaluation, metacog lesson, next possibility, and follow-up prediction.",
  ]
    .join("\n\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

let compactResumeRetryTimer: ReturnType<typeof setTimeout> | null = null;

async function refreshWorkpointResumePacket(mode = "compact_prompt"): Promise<any | null> {
  if (!getAttachmentRuntime().focusaAvailable) return null;
  const scope = currentCompactionScope();
  const root = scope?.root_scope.root_path || "";
  if (!scope || !isProjectRootAuthoritySafe(root)) {
    setActiveWorkpointPacket(null);
    setActiveWorkpointSummary("");
    return null;
  }
  try {
    const packet = await focusaFetch("/workpoint/resume", {
      method: "POST",
      body: JSON.stringify({
        mode,
        scope,
        continuity_id: scope.continuity_id,
        session_id: getSessionFrameKey(),
        project_root: scope.root_scope.root_path,
        current_ask: getAttachmentRuntime().currentAsk?.text || "",
      }),
    });
    if (packet && packet.status === "rejected_scope_mismatch") {
      setActiveWorkpointPacket(null);
      setActiveWorkpointSummary("");
      return null;
    }
    if (packet && packet.status === "completed") {
      const candidate = normalizeWorkpointResumePacketEnvelope(packet);
      if (!isWorkpointPacketScopedToCurrentSession(candidate)) {
        setActiveWorkpointPacket(null);
        setActiveWorkpointSummary("");
        return null;
      }
      setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(candidate));
      setActiveWorkpointSummary(
        packet.rendered_summary || packet.resume_packet_v2?.rendered_summary || packet.next_step_hint || ""
      );
      getAttachmentRuntime().lastWorkpointUpdate = Date.now();
      return packet;
    }
  } catch {
    /* best effort */
  }
  return null;
}

async function checkpointTrajectoryBeforeCompaction(reason = "before_compaction"): Promise<any | null> {
  if (!getAttachmentRuntime().focusaAvailable) return null;
  const scope = currentCompactionScope();
  if (!scope) return null;
  const root = scope.root_scope.root_path;
  const continuityId = scope.continuity_id;
  try {
    return await focusaFetch("/trajectory/checkpoint", {
      method: "POST",
      body: JSON.stringify({
        summary: `Pi ${reason}: preserve Trajectory Ladder north-star context across compaction.`,
        scope,
        continuity_id: continuityId,
        session_id: getSessionFrameKey(),
        project_root: root,
        idempotency_key: `pi-trajectory-${reason}-${getSessionFrameKey() || "session"}-${getTurnCount()}`,
      }),
    });
  } catch {
    return null;
  }
}

async function refreshTrajectoryResumePacket(reason = "compaction"): Promise<any | null> {
  if (!getAttachmentRuntime().focusaAvailable) return null;
  const scope = currentCompactionScope();
  if (!scope) return null;
  const root = scope.root_scope.root_path;
  const continuityId = scope.continuity_id;
  try {
    const packet = await focusaFetch("/trajectory/resume", {
      method: "POST",
      body: JSON.stringify({
        mode: "summary",
        scope,
        continuity_id: continuityId,
        session_id: getSessionFrameKey(),
        project_root: root,
      }),
    });
    const view = packet?.resume_packet || packet?.trajectory_checkpoint || packet;
    const trajectory = view?.trajectory || {};
    setLastTrajectoryClarity({
      ...(getLastTrajectoryClarity() || {}),
      reason,
      refreshed_at: Date.now(),
      project_root: root,
      continuity_id: continuityId,
      session_id: getSessionFrameKey() || null,
      status: String(
        view?.intelligence_view?.clarity_gate?.status ||
          trajectory.definition_status ||
          packet?.status ||
          "unknown"
      ),
      recommended_action: String(
        view?.intelligence_view?.clarity_gate?.recommended_action ||
          view?.intelligence_view?.context_sufficiency?.recommended_action ||
          "unknown"
      ),
      canonical: packet?.canonical === true || view?.canonical === true,
      degraded: packet?.degraded === true || view?.degraded === true,
      trajectory_id: trajectory.trajectory_id || getLastTrajectoryClarity()?.trajectory_id || null,
      fallback_prior_project_trajectory: trajectory.fallback_prior_project_trajectory === true,
      fallback_source_continuity_id: trajectory.fallback_source_continuity_id || null,
      long_term_goal:
        trajectory.long_term_goal ||
        trajectory.trajectory_ladder?.hlt ||
        getLastTrajectoryClarity()?.long_term_goal ||
        null,
      desired_end_state:
        trajectory.desired_end_state || getLastTrajectoryClarity()?.desired_end_state || null,
      mid_level_goal:
        trajectory.mid_level_goal ||
        trajectory.trajectory_ladder?.mlg ||
        getLastTrajectoryClarity()?.mid_level_goal ||
        null,
      short_term_goal:
        trajectory.short_term_goal ||
        trajectory.trajectory_ladder?.stg ||
        getLastTrajectoryClarity()?.short_term_goal ||
        null,
      waypoints:
        trajectory.waypoints ||
        trajectory.trajectory_ladder?.waypoints ||
        getLastTrajectoryClarity()?.waypoints ||
        [],
      current_state: trajectory.current_state || getLastTrajectoryClarity()?.current_state || null,
      active_gap: trajectory.active_gap || getLastTrajectoryClarity()?.active_gap || null,
      resume_packet: view || null,
    });
    return packet || getLastTrajectoryClarity();
  } catch {
    return getLastTrajectoryClarity() || null;
  }
}

function formatTrajectoryPacketForPrompt(packet: any): string {
  const view =
    packet?.resume_packet || packet?.trajectory_checkpoint || packet?.resume_packet?.resume_packet || packet;
  const trajectory = view?.trajectory || packet?.trajectory || packet || {};
  const hlt = compactText(
    trajectory.long_term_goal || trajectory.trajectory_ladder?.hlt || packet?.long_term_goal,
    "missing",
    220
  );
  const mlg = compactText(
    trajectory.mid_level_goal || trajectory.trajectory_ladder?.mlg || packet?.mid_level_goal,
    "missing",
    180
  );
  const stg = compactText(
    trajectory.short_term_goal || trajectory.trajectory_ladder?.stg || packet?.short_term_goal,
    "missing",
    180
  );
  const desired = compactText(trajectory.desired_end_state || packet?.desired_end_state, "missing", 220);
  const gap = compactText(trajectory.active_gap || packet?.active_gap, "none", 220);
  const waypoints = Array.isArray(trajectory.waypoints || packet?.waypoints)
    ? (trajectory.waypoints || packet?.waypoints)
        .slice(0, 5)
        .map((item: any) => compactText(item, "", 100))
        .filter(Boolean)
    : [];
  if (hlt === "missing" && mlg === "missing" && stg === "missing" && desired === "missing") return "";
  // Spec 125 §9.1-9.4: v3 packet fields.
  const hltStatus = packet?.hlt_status || view?.hlt_status || "missing_required";
  const trajectoryRequired = packet?.trajectory_required ?? view?.trajectory_required ?? true;
  const hltRequired = packet?.hlt_required ?? view?.hlt_required ?? true;
  const actionAuthority =
    packet?.action_authority_from_trajectory ?? view?.action_authority_from_trajectory ?? false;
  const genericBootstrap = packet?.generic_bootstrap ?? view?.generic_bootstrap ?? false;
  const loudWarning = packet?.loud_warning || view?.loud_warning || null;
  const fallbackLevel = packet?.fallback_level || view?.fallback_level || "not_applicable";
  const fallbackSourceScope = packet?.fallback_source_scope || view?.fallback_source_scope || null;
  const warnings = Array.isArray(packet?.warnings || view?.warnings)
    ? packet?.warnings || view?.warnings
    : [];

  return [
    "## TrajectoryResumePacket",
    "## TrajectoryResumePacketV3",
    `SCHEMA_VERSION: focusa.trajectory_resume_packet.v3`,
    `STATUS: ${String(packet?.status || view?.status || trajectory.definition_status || "unknown")}`,
    `CANONICAL: ${packet?.canonical === true || view?.canonical === true}`,
    `DEGRADED: ${packet?.degraded === true || view?.degraded === true}`,
    `TRAJECTORY_ID: ${compactText(trajectory.trajectory_id || packet?.trajectory_id, "unknown", 120)}`,
    // Spec 125 §9.3: v3 HLT status fields.
    `HLT_STATUS: ${hltStatus}`,
    `TRAJECTORY_REQUIRED: ${trajectoryRequired}`,
    `HLT_REQUIRED: ${hltRequired}`,
    `ACTION_AUTHORITY_FROM_TRAJECTORY: ${actionAuthority}`,
    `GENERIC_BOOTSTRAP: ${genericBootstrap}`,
    `FALLBACK_LEVEL: ${fallbackLevel}`,
    `FALLBACK_SOURCE_SCOPE: ${fallbackSourceScope || "not_applicable"}`,
    // Deprecated aliases.
    `FALLBACK_PRIOR_PROJECT_TRAJECTORY: ${trajectory.fallback_prior_project_trajectory === true || packet?.fallback_prior_project_trajectory === true}`,
    `FALLBACK_SOURCE_CONTINUITY_ID: ${compactText(trajectory.fallback_source_continuity_id || packet?.fallback_source_continuity_id, "none", 120)}`,
    `HLT: ${hlt}`,
    `MLG: ${mlg}`,
    `STG: ${stg}`,
    `DESIRED_END_STATE: ${desired}`,
    `ACTIVE_GAP: ${gap}`,
    `WAYPOINTS: ${waypoints.join(" → ") || "derive_next"}`,
    "AUTHORITY: TL is north-star route context; Workpoint remains immediate action authority.",
    "NEXT_TOOLS: focusa_workpoint_resume, focusa_trajectory_view, focusa_active_object_resolve, focusa_trajectory_define_goal",
    // Spec 125 §9.3: loud warning above ordinary guidance.
    ...(loudWarning ? [`LOUD_WARNING: ${loudWarning}`] : []),
    ...(warnings.length > 0 ? [`WARNINGS: ${warnings.join(" | ")}`] : []),
    "STRUCTURED_PACKET_JSON:",
    JSON.stringify(view || packet, null, 2).slice(0, 3500),
  ].join("\n");
}

function recordLocalWorkpointFallback(reason: string): void {
  const fallback = {
    status: "partial",
    canonical: false,
    reason,
    mission:
      semanticCurrentAsk() ||
      getAttachmentRuntime().activeFrameGoal ||
      getAttachmentRuntime().lastFocusSnapshot.intent ||
      "unknown mission",
    next_slice:
      getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
      getAttachmentRuntime().lastCompactDecision ||
      getAttachmentRuntime().activeFrameGoal ||
      "resume from local degraded fallback",
    source_turn_id: `pi-turn-${getTurnCount()}`,
    recorded_at: new Date().toISOString(),
  };
  setActiveWorkpointPacket(fallback);
  setActiveWorkpointSummary(`NON-CANONICAL WORKPOINT FALLBACK: ${fallback.next_slice}`);
  getAttachmentRuntime().lastWorkpointUpdate = Date.now();
  try {
    getAttachmentRuntime().pi?.appendEntry("focusa-workpoint-fallback", fallback);
  } catch {
    /* best effort */
  }
  persistState();
}

async function checkpointBeforeCompaction(): Promise<any | null> {
  if (!getAttachmentRuntime().focusaAvailable) return null;
  const scope = currentCompactionScope();
  if (!scope) return null;
  const root = scope.root_scope.root_path;
  const continuityId = scope.continuity_id;
  const ask = semanticCurrentAsk();
  const mission =
    ask ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().lastFocusSnapshot.intent ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    "Pi work before compaction";
  const nextSlice =
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    getAttachmentRuntime().lastCompactDecision ||
    getAttachmentRuntime().activeFrameGoal ||
    ask ||
    "Resume current task from typed Workpoint packet after compaction.";
  try {
    return await focusaFetch("/workpoint/checkpoint", {
      method: "POST",
      body: JSON.stringify({
        mission,
        next_slice: nextSlice,
        work_item_id: getAttachmentRuntime().currentAsk?.sourceTurnId,
        checkpoint_reason: "before_compact",
        canonical: true,
        promote: true,
        scope,
        continuity_id: continuityId,
        session_id: getSessionFrameKey(),
        project_root: root,
        source_turn_id: `pi-turn-${getTurnCount()}`,
        idempotency_key: `pi-before-compact-${getSessionFrameKey() || "session"}-${getTurnCount()}`,
        action_intent: {
          action_type: "resume_workpoint",
          target_ref: getAttachmentRuntime().currentAsk?.sourceTurnId || getActiveFrameId() || "pi-session",
          verification_hooks: [
            "resume packet appears in compaction instructions",
            "post-compact steer uses WorkpointResumePacket",
          ],
          status: "ready",
        },
      }),
    });
  } catch {
    return null;
  }
}

export interface CompactionRolloverPreparation {
  ready: boolean;
  scope: WorkstreamKey | null;
  workpoint_checkpoint: any | null;
  trajectory_checkpoint: any | null;
  compaction_packet: any | null;
  reason: string;
}

export async function prepareCompactionRollover(): Promise<CompactionRolloverPreparation> {
  const scope = currentCompactionScope();
  if (!scope) {
    return {
      ready: false,
      scope: null,
      workpoint_checkpoint: null,
      trajectory_checkpoint: null,
      compaction_packet: null,
      reason: "typed_verified_project_workstream_scope_required",
    };
  }
  const workpointCheckpoint = await checkpointBeforeCompaction();
  const trajectoryCheckpoint = await checkpointTrajectoryBeforeCompaction("session_rollover");
  const compactionPacket = await buildCompactionMissionPacket("session_rollover");
  const ready = Boolean(workpointCheckpoint && trajectoryCheckpoint && compactionPacket);
  return {
    ready,
    scope,
    workpoint_checkpoint: workpointCheckpoint,
    trajectory_checkpoint: trajectoryCheckpoint,
    compaction_packet: compactionPacket,
    reason: ready ? "checkpoint_transaction_ready" : "checkpoint_transaction_incomplete",
  };
}

export function isFocusaContextContinuityHealthy(): boolean {
  const scope = currentCompactionScope();
  if (!scope) return false;
  const continuityId = scope.continuity_id;
  const packet = getScopedWorkpointPacket();
  const rawPacket = getActiveWorkpointPacket();
  const noDegradedWorkpoint = !rawPacket || Boolean(packet);
  return Boolean(
    getAttachmentRuntime().focusaAvailable && String(continuityId || "").trim() && noDegradedWorkpoint
  );
}

export type ContextPressureWarningKind = "auto_suggest" | "hard_unconfirmed" | "handoff_unconfirmed";

export type BloatgaurdPressureAction = "hard" | "auto" | "warn" | "none";

export interface BloatgaurdPressureThresholds {
  warnPct: number;
  compactPct: number;
  hardPct: number;
}

export function classifyBloatgaurdPressureAction(
  pct: number,
  cfg: BloatgaurdPressureThresholds,
  canCompact: boolean
): BloatgaurdPressureAction {
  if (pct >= cfg.hardPct) return "hard";
  if (pct >= cfg.compactPct && canCompact) return "auto";
  if (pct >= cfg.warnPct) return "warn";
  return "none";
}

export function resetLiveContextPressureAfterCompaction(now = Date.now()): void {
  const runtime = getAttachmentRuntime();
  runtime.lastCompactTime = now;
  runtime.turnsSinceCompact = 0;
  runtime.currentTier = "";
  runtime.currentContextPct = null;
  runtime.forkSuggested = false;
}

export function contextPressureWarningCopy(
  kind: ContextPressureWarningKind,
  pct: number,
  totalCompactions = getTotalCompactions()
): string {
  const pctLabel = Number.isFinite(pct) ? pct.toFixed(0) : "unknown";
  if (kind === "auto_suggest")
    return `💡 Context at ${pctLabel}% — Focusa anchors are unconfirmed; checkpoint/resume Workpoint, /fork optional for UI isolation`;
  if (kind === "hard_unconfirmed")
    return `⚠️ Context ${pctLabel}% — Focusa will try checkpointed compaction; scoped Workpoint anchor not yet confirmed`;
  return `💡 ${totalCompactions} compactions with unconfirmed Workpoint anchor — resume/checkpoint Workpoint; handoff optional`;
}

export function contextTierLabel(tier: "" | "warn" | "auto" | "hard"): string {
  if (tier === "warn") return "monitor";
  if (tier === "auto") return "compacting";
  if (tier === "hard")
    return isFocusaContextContinuityHealthy() ? "critical · Focusa anchors" : "critical · anchor unconfirmed";
  return "";
}

function setContextStatus(
  ctx: any,
  tier: "" | "warn" | "auto" | "hard",
  pct?: number,
  focusaContinuityReady = isFocusaContextContinuityHealthy()
) {
  getAttachmentRuntime().currentContextPct = typeof pct === "number" ? pct : null;
  const mode = getAttachmentRuntime().cfg?.contextStatusMode || "actionable";
  if (mode === "off") {
    ctx.ui.setStatus("focusa-ctx", "");
    return;
  }
  if (tier === "warn") {
    if (mode === "all" && typeof pct === "number")
      ctx.ui.setStatus("focusa-ctx", `📦 Context ${pct.toFixed(0)}% monitor`);
    else ctx.ui.setStatus("focusa-ctx", "");
    return;
  }
  if (tier === "auto" && typeof pct === "number") {
    ctx.ui.setStatus("focusa-ctx", `🗜️ Context ${pct.toFixed(0)}% compacting`);
    return;
  }
  if (tier === "hard" && typeof pct === "number") {
    const label = focusaContinuityReady ? "Focusa anchors" : "anchor unconfirmed";
    ctx.ui.setStatus("focusa-ctx", `🚧 Context ${pct.toFixed(0)}% ${label}`);
    return;
  }
  ctx.ui.setStatus("focusa-ctx", "");
}

function formatResumePacketV2ForPrompt(packet: any): string {
  const v2 =
    packet?.resume_packet_v2 ||
    (packet?.schema_version === "focusa.workpoint_resume_packet.v2" ? packet : null);
  if (!v2 || typeof v2 !== "object") return "";
  const packetProjectRoot = normalizeProjectRoot(packet?.project_root || v2?.workpoint?.project_root);
  const packetContinuityId = String(packet?.continuity_id || v2?.workpoint?.continuity_id || "").trim();
  if (!isProjectRootAuthoritySafe(packetProjectRoot)) return "";
  if (!packetContinuityId || (getContinuityId() && packetContinuityId !== getContinuityId())) return "";
  if (v2.canonical === false) return "";
  const affordances = v2.tool_affordances || {};
  const bestNext = Array.isArray(affordances.best_next) ? affordances.best_next : v2.next_tools || [];
  const doNotUse = Array.isArray(affordances.do_not_use_by_default) ? affordances.do_not_use_by_default : [];
  return [
    "## WorkpointResumePacketV2",
    `SCHEMA_VERSION: ${v2.schema_version || "focusa.workpoint_resume_packet.v2"}`,
    `CANONICAL: ${v2.canonical !== false}`,
    `FAILURE_CLASS: ${String(v2.failure_class ?? "not_applicable")}`,
    v2.rendered_summary ? `RENDERED_SUMMARY: ${v2.rendered_summary}` : "",
    "BEST_NEXT_TOOLS:",
    ...(bestNext.length
      ? bestNext
      : ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse", "focusa_tool_doctor"]
    ).map((tool: string) => `  - ${tool}`),
    "DO_NOT_USE_BY_DEFAULT:",
    ...(doNotUse.length
      ? doNotUse
      : ["transcript tail as authority", "full lineage tree", "full ontology graph", "deep work-loop status"]
    ).map((item: string) => `  - ${item}`),
    "RECOVERY_ORDER:",
    "  - focusa_workpoint_resume",
    "  - focusa_trajectory_view",
    "  - focusa_traverse",
    "  - focusa_active_object_resolve when object identity is ambiguous",
    "  - focusa_tool_doctor when canonical=false, degraded=true, blocked, or stale",
    "AUTHORITY_BOUNDARY: project_root + continuity_id; trajectory similarity is advisory grouping only.",
    "STRUCTURED_PACKET_JSON:",
    JSON.stringify(v2, null, 2).slice(0, 6000),
  ]
    .filter(Boolean)
    .join("\n");
}

function submitCompactionResumeTurn(ctx: any, steerMessage: string): boolean {
  const pi2 = getAttachmentRuntime().pi;
  if (!pi2) return false;
  pi2.sendMessage(
    {
      customType: "focusa-compact-resume",
      content: steerMessage,
      display: false,
    },
    { triggerTurn: true }
  );
  getAttachmentRuntime().compactResumePending = false;
  persistState();
  ctx.ui.notify(`✅ Compaction done — auto-resume turn submitted`, "info");
  return true;
}

function scheduleCompactionResumeRetry(ctx: any, steerMessage: string, retryAttempt = 1) {
  if (!getAttachmentRuntime().compactResumePending) return;
  const nextAttempt = retryAttempt + 1;
  compactResumeRetryTimer = setTimeout(
    () => {
      compactResumeRetryTimer = null;
      if (!getAttachmentRuntime().compactResumePending) return;
      try {
        submitCompactionResumeTurn(ctx, steerMessage);
        scheduleCompactionResumeRetry(ctx, steerMessage, retryAttempt + 1);
      } catch (e) {
        console.warn(`[focusa] compaction auto-resume retry ${retryAttempt} failed:`, e);
        if (!getAttachmentRuntime().compactResumePending) return;
        scheduleCompactionResumeRetry(ctx, steerMessage, nextAttempt);
      }
    },
    Math.min(30_000, 2_000 * retryAttempt)
  );
}

function scheduleCompactionResumeWatchdog(ctx: any, steerMessage: string) {
  if (!getAttachmentRuntime().compactResumePending) return;
  scheduleCompactionResumeRetry(ctx, steerMessage, 1);
}

export function registerCompaction(pi: ExtensionAPI) {
  // ── session_before_compact (§33.1 ASCC replacement, §33.10 fallback) ───────
  pi.on("session_before_compact", async (event, _ctx) => {
    (getAttachmentRuntime() as any).compactionMemoryBefore = compactionMemorySample();
    // Sync local shadow → Focusa before compaction
    // §33.1 + N5: Use pushDelta() for ALL writes — enforces validateSlot() on every delta.
    // session_compact bypassed validation before this fix — every compaction refilled
    // recent_results with verbose entries that validateSlot would have rejected.
    if (getAttachmentRuntime().focusaAvailable && getActiveFrameId()) {
      await pushDelta({
        decisions: getAttachmentRuntime().localDecisions.slice(-10),
        constraints: getAttachmentRuntime().localConstraints.slice(-10),
        failures: sanitizeFocusFailures(getAttachmentRuntime().localFailures).slice(-5),
      });
    }
    await checkpointBeforeCompaction();
    await checkpointTrajectoryBeforeCompaction("before_compaction");
    await refreshTrajectoryClarityLifecycle(
      "before_compaction",
      currentCompactionScope()?.root_scope.root_path || ""
    );
    const trajectoryPacket = await refreshTrajectoryResumePacket("before_compaction");
    void trajectoryPacket;
    const workpointPacket = await refreshWorkpointResumePacket("compact_prompt");
    const missionPacket = await buildCompactionMissionPacket("before_compaction");

    // Always persist to Pi session entries as backup
    await persistAuthoritativeState();

    // Spec 130: a bounded typed mission packet supersedes ad-hoc prompt
    // reconstruction when the daemon can build one for the verified scope.
    if (missionPacket) {
      const ev = event as any;
      return {
        compaction: {
          summary: renderCompactionMissionPacket(missionPacket),
          firstKeptEntryId: ev.preparation?.firstKeptEntryId,
          tokensBefore: ev.preparation?.tokensBefore,
        },
      };
    }

    // §33.1: Try Focusa ASCC replacement FIRST
    if (getAttachmentRuntime().focusaAvailable) {
      try {
        const ascc = await focusaFetch("/ascc/state");
        if (ascc?.focus_state) {
          const fs = ascc.focus_state;
          const summary = await buildCompactionFallbackSummary(fs, workpointPacket);
          const ev = event as any;
          return {
            compaction: {
              summary,
              firstKeptEntryId: ev.preparation?.firstKeptEntryId,
              tokensBefore: ev.preparation?.tokensBefore,
            },
          };
        }
      } catch {
        /* ASCC unavailable — fall through to §33.10 */
      }

      // §33.10: Softer fallback — customInstructions to guide Pi's compaction
      return {
        customInstructions: buildCompactInstructions(
          "Preserve Focusa Focus State (decisions, constraints, intent). Summarize older turns."
        ),
      };
    }

    // Focusa offline — fall through to Pi's default compaction
    return undefined;
  });

  // ── session_compact (§38.1 trim, §35.6 files + auto-resume) ───────────────
  pi.on("session_compact", async (event, ctx) => {
    // The compaction entry is already saved when this event fires. Reset only
    // live-model context pressure; append-only native-session pressure remains authoritative.
    resetLiveContextPressureAfterCompaction();
    setContextStatus(ctx, "");

    // §38.1: Trim local shadow only after Focusa confirms state.
    // NOTE: getAttachmentRuntime().lastCompactDecision is saved BEFORE trimming (used in steer below).
    const lastDecision =
      getAttachmentRuntime().localDecisions[getAttachmentRuntime().localDecisions.length - 1] ??
      "pre-compaction work";
    getAttachmentRuntime().lastCompactDecision = lastDecision;

    // §5.12: On compaction, force re-emit recent-turns slice on the resumed loop.
    // Reset idempotency guard so the next before_agent_start injects fresh.
    (getAttachmentRuntime() as any).lastRecentTurnsSliceTurn = -1;

    if (getAttachmentRuntime().focusaAvailable && getActiveFrameId()) {
      const data = await getFocusState();
      if (data?.fs?.decisions?.length || data?.fs?.constraints?.length) {
        getAttachmentRuntime().localDecisions = [];
        getAttachmentRuntime().localConstraints = [];
        getAttachmentRuntime().localFailures = [];
      }
    }

    // §35.6: Feed modified/read files to Focusa as canonical artifact lines
    const compaction = (event as any).compactionEntry;
    const modifiedFiles = compaction?.details?.modifiedFiles || compaction?.details?.fileOps || [];
    const readFiles = compaction?.details?.readFiles || [];
    const artifacts = normalizeCompactionArtifacts(modifiedFiles);
    const compactNotes: string[] = [];
    if (artifacts.length)
      compactNotes.push(`Session compacted. Modified: ${artifacts.map((a) => a.path_or_id).join(", ")}`);
    if (Array.isArray(readFiles) && readFiles.length)
      compactNotes.push(`Session compacted. Read: ${readFiles.slice(0, 20).join(", ")}`);
    if (
      getAttachmentRuntime().focusaAvailable &&
      getActiveFrameId() &&
      (artifacts.length || compactNotes.length)
    ) {
      await focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: getActiveFrameId(),
          project_root: currentCompactionScope()?.root_scope.root_path,
          continuity_id: currentCompactionScope()?.continuity_id,
          turn_id: `pi-turn-${getTurnCount()}`,
          delta: {
            ...(artifacts.length ? { artifacts } : {}),
            ...(compactNotes.length ? { notes: compactNotes } : {}),
          },
        }),
      }).catch(() => {});
      await persistAuthoritativeState();
    }

    scheduleCompactionMemoryEvaluation();

    // §38.3 CRITICAL FIX: queueMicrotask defers to next event-loop tick,
    // AFTER compaction_end fires (which calls flushCompactionQueue first,
    // then hasQueuedMessages() -> agent.continue()). Without deferral,
    // sendMessage is still async when hasQueuedMessages() fires -> miss.
    // Also dedup: only resume once per compaction cycle.
    const compactionEntry = (event as any).compactionEntry || {};
    const compactOrdinal = getTotalCompactions() || compactionEntry.details?.totalCompactions || "unknown";
    const compactResumeKey = String(
      compactionEntry.id ||
        compactionEntry.uuid ||
        compactionEntry.timestamp ||
        `${getSessionFrameKey() || "session"}:compact:${compactOrdinal}`
    );
    const recentlySubmitted =
      getAttachmentRuntime().lastCompactResumeKey === compactResumeKey ||
      (Date.now() - getAttachmentRuntime().lastCompactResumeAt < 30_000 && compactOrdinal !== "unknown");
    if (!getAttachmentRuntime().compactResumePending && !recentlySubmitted) {
      await refreshWorkpointResumePacket("compact_prompt");
      await refreshTrajectoryClarityLifecycle(
        "after_compaction",
        currentCompactionScope()?.root_scope.root_path || ""
      );
      const trajectoryPacket = await refreshTrajectoryResumePacket("after_compaction");
      const missionPacket = await buildCompactionMissionPacket("after_compaction");
      getAttachmentRuntime().lastCompactResumeKey = compactResumeKey;
      getAttachmentRuntime().lastCompactResumeAt = Date.now();
      persistState();
      if (compactResumeRetryTimer) {
        clearTimeout(compactResumeRetryTimer);
        compactResumeRetryTimer = null;
      }
      getAttachmentRuntime().compactResumePending = true;
      const pi2 = getAttachmentRuntime().pi;
      if (pi2) {
        queueMicrotask(() => {
          // lastDecision saved above, before localDecisions was cleared
          const scopedPacket = getScopedWorkpointPacket();
          const v2Prompt = formatResumePacketV2ForPrompt(scopedPacket);
          const trajectoryPrompt = formatTrajectoryPacketForPrompt(
            trajectoryPacket || getLastTrajectoryClarity()
          );
          const visibleRecapReason = toolOutputVisibleRecapReason();
          const attentionPrompt = [
            ...formatAttentionRecallFocusSliceLines(
              buildAttentionRecallVerdict({
                workpointPacket: scopedPacket,
                currentAskText: semanticCurrentAsk(),
                currentAskKind: getAttachmentRuntime().currentAsk?.kind,
                queryScopeKind: getAttachmentRuntime().queryScope?.scopeKind,
                projectRoot: getSessionCwd(),
                continuityId: getContinuityId(),
                visibleRecapReason,
              })
            ),
            ...formatCurrentAskScopeVerdictLines(
              buildCurrentAskScopeVerdict({
                currentAskText: semanticCurrentAsk(),
                workpointPacket: scopedPacket,
                projectRoot: getSessionCwd(),
                continuityId: getContinuityId(),
              })
            ),
            ...formatToolOutputVisibleRecapLines(visibleRecapReason),
          ].join("\n");
          const directive = v2Prompt
            ? `Call focusa_workpoint_resume first if uncertain; treat WorkpointResumePacketV2 as canonical only when canonical=true and project_root+continuity_id match. Use the injected TrajectoryResumePacket as TL north-star context, then use focusa_trajectory_view for refresh and focusa_traverse for bounded supporting slices. Include prediction/metacog context in trajectory review and final task report. Never use transcript tail as authority.`
            : `No verified WorkpointResumePacketV2 is available for this exact project_root+continuity_id; call focusa_workpoint_resume, focusa_trajectory_view, focusa_metacog_doctor, focusa_predict_recent/stats, or focusa_tool_doctor before trusting any carryover.`;
          const note = getTotalCompactions() > 0 ? ` [compaction #${getTotalCompactions()}]` : "";
          const missionPrompt = missionPacket
            ? renderCompactionMissionPacket(missionPacket)
            : "## CompactionMissionPacket\nUNAVAILABLE; rehydrate from canonical Workpoint and Trajectory routes.";
          const steerMessage = `# Compaction Complete${note}
${missionPrompt}
## Last Active Focus
${getAttachmentRuntime().lastCompactDecision || "pre-compaction work"}
## AttentionRecallVerdict
${attentionPrompt}
## WorkpointResumePacketV2
${v2Prompt || `No project-bound WorkpointResumePacketV2 recorded (${projectRootAuthorityFailure(currentCompactionScope()?.root_scope.root_path || "") || "v2 packet unavailable"}); continue from Last Active Focus only after a fresh safe resume/orientation call.`}
## TrajectoryResumePacket
${trajectoryPrompt || `No project-bound TrajectoryResumePacket recorded (${projectRootAuthorityFailure(currentCompactionScope()?.root_scope.root_path || "") || "trajectory packet unavailable"}); call focusa_trajectory_view before treating TL context as current.`}
## Directive
${directive}

---

## End-of-task Learning Loop
Before claiming task completion or writing a final work report:
- Summarize the task outcome and proof.
- Run/consult focusa_predict_recent or focusa_predict_stats; evaluate relevant predictions with focusa_predict_evaluate.
- Run/consult focusa_metacog_doctor or focusa_metacog_retrieve; capture reusable lessons with focusa_metacog_capture when evidence-backed.
- Cross-reference the next possibility as a bounded prediction plus trajectory gap.

## Focusa Tool Guidance
When using focusa_scratch / focusa_decide / focusa_constraint / focusa_failure:
- **Working notes** → focusa_scratch (all internal monologue welcome)
- **Crystallized decision** → focusa_decide (ONE sentence, max 160 chars, architectural choice)
- **Discovered requirement** → focusa_constraint (hard boundary from environment/architecture)
- **Failure diagnosis** → focusa_failure (specific component + why it failed)
- **Validation** fails if: task patterns (Fix/Add/Check), debug patterns (error/failed), self-reference (I think/I tried), or exceeding char limits

See: ls /tmp/pi-scratch/ | cat /tmp/pi-scratch/turn-NNNN/notes.txt`;
          try {
            submitCompactionResumeTurn(ctx, steerMessage);
          } catch (e) {
            console.warn("[focusa] auto-resume failed:", e);
            getAttachmentRuntime().compactResumePending = false;
          }
        });
      }
    } else if (recentlySubmitted) {
      ctx.ui.notify(
        "↩️ Compaction auto-resume already submitted for this compact cycle; suppressing duplicate.",
        "info"
      );
    }
  });
}

// ── Compaction tier check — called from turn_end in turns.ts (§20) ───────────
export async function checkCompactionTier(ctx: any): Promise<void> {
  const cfg = getAttachmentRuntime().cfg;
  if (!cfg) return;
  getAttachmentRuntime().turnsSinceCompact++;

  const usage = ctx.getContextUsage?.();
  if (!usage?.tokens) return;
  if (typeof usage.contextWindow === "number" && usage.contextWindow > 0) {
    getAttachmentRuntime().activeContextWindow = usage.contextWindow;
  }
  const pct =
    typeof usage.percent === "number"
      ? usage.percent
      : (usage.tokens / (usage.contextWindow || getAttachmentRuntime().activeContextWindow)) * 100;

  // Reset hourly counter
  if (Date.now() - getAttachmentRuntime().compactHourStart > 3_600_000) {
    getAttachmentRuntime().compactsThisHour = 0;
    getAttachmentRuntime().compactHourStart = Date.now();
  }

  const cooldownOk = Date.now() - getAttachmentRuntime().lastCompactTime > cfg.cooldownMs;
  const hourlyOk = getAttachmentRuntime().compactsThisHour < cfg.maxCompactionsPerHour;
  const turnsOk = getAttachmentRuntime().turnsSinceCompact >= cfg.minTurnsBetweenCompactions;
  const canCompact = cooldownOk && hourlyOk && turnsOk;
  const pressureAction = classifyBloatgaurdPressureAction(pct, cfg, canCompact);

  const onDone = () => {
    resetLiveContextPressureAfterCompaction();
    getAttachmentRuntime().compactsThisHour++;
    incrementTotalCompactions();
  };

  const focusaContinuityReady = isFocusaContextContinuityHealthy();

  // §18: autoSuggestForkPct — generic fork/new guidance is only actionable when scoped Focusa anchors are unconfirmed.
  if (pct >= cfg.autoSuggestForkPct && !getAttachmentRuntime().forkSuggested && !focusaContinuityReady) {
    getAttachmentRuntime().forkSuggested = true;
    ctx.ui.notify(contextPressureWarningCopy("auto_suggest", pct), "warning");
  }

  if (pressureAction === "hard") {
    getAttachmentRuntime().currentTier = "hard";
    setContextStatus(ctx, "hard", pct, focusaContinuityReady);
    if (!focusaContinuityReady) {
      ctx.ui.notify(contextPressureWarningCopy("hard_unconfirmed", pct), "warning");
      // §18: Suggest handoff after N compactions only when Workpoint continuity is not healthy.
      if (getTotalCompactions() >= cfg.autoSuggestHandoffAfterNCompactions) {
        ctx.ui.notify(
          contextPressureWarningCopy("handoff_unconfirmed", pct, getTotalCompactions()),
          "warning"
        );
      }
    }
    const requestResult = requestCoordinatedCompaction(ctx, {
      triggerClass: "hard_pressure",
      customInstructions: buildCompactInstructions(
        "HARD COMPACT: preserve canonical Focusa authority and release live Pi context."
      ),
      onComplete: onDone,
      onError: (error) => ctx.ui.notify(`Compaction failed: ${error.message}`, "error"),
    });
    if (requestResult === "coordinator_unavailable") {
      ctx.ui.notify("Compaction blocked: Focusa compaction coordinator is unavailable", "error");
    }
  } else if (pressureAction === "auto") {
    getAttachmentRuntime().currentTier = "auto";
    setContextStatus(ctx, "auto", pct);
    ctx.ui.notify(`📊 Context ${pct.toFixed(0)}% — compacting`, "info");
    const requestResult = requestCoordinatedCompaction(ctx, {
      triggerClass: "predicted_pressure",
      customInstructions: buildCompactInstructions(
        "Compact live Pi context while preserving canonical Focusa authority."
      ),
      onComplete: onDone,
      onError: (error) => ctx.ui.notify(`Compaction failed: ${error.message}`, "error"),
    });
    if (requestResult === "coordinator_unavailable") {
      ctx.ui.notify("Compaction blocked: Focusa compaction coordinator is unavailable", "error");
    }
  } else if (pressureAction === "warn") {
    getAttachmentRuntime().currentTier = "warn";
    setContextStatus(ctx, "warn", pct);
  } else {
    getAttachmentRuntime().currentTier = "";
    setContextStatus(ctx, "");
  }
}

// ── Periodic micro-compact (§21) — called from turn_end ─────────────────────
export async function checkMicroCompact(): Promise<void> {
  const n = getAttachmentRuntime().cfg?.microCompactEveryNTurns || 5;
  if (getTurnCount() > 0 && getTurnCount() % n === 0 && getAttachmentRuntime().focusaAvailable) {
    // §21: Request micro-compact via Focusa API (not extension-owned summarization)
    focusaFetch("/commands/submit", {
      method: "POST",
      body: JSON.stringify({
        command: "micro-compact",
        args: { turn_count: getTurnCount(), surface: "pi" },
        idempotency_key: `micro-${getTurnCount()}-${Date.now()}`,
      }),
    }).catch(() => {});
  }
}
