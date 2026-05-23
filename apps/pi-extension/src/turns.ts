// Turn lifecycle + per-call event handlers — ONE handler per event type
// Spec: §7.1 (10 ASCC slots), §7.4 (ECS thresholds), §33.2 (context), §33.3 (ECS replace),
//        §33.4 (tool usage), §34.2B (turns), §35.2 (behavioral), §35.5 (tokens),
//        §35.7 (correction), §36.1 (streaming), §36.2 (error signals), §36.3 (input),
//        §36.6 (injection layering), §36.7 (budget), §37.3 (widget), §37.8 (model),
//        §30 (metacognitive indicators)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import type { PiGoverningPriorKind } from "./state.js";
import { S, focusaFetch, focusaPost, extractText, getFocusState, getEffectiveFocusSnapshot, estimateTokens, wbExec, storeEcsArtifact, classifyCurrentAsk, deriveQueryScope, isOperatorSteeringInput, selectRelevantItems, selectRelevantRankedItems, shouldIncludeMissionContext, buildSliceSection, selectionRelevanceScore, retentionBucketsFromSelection, formatWorkingSetItems, formatVerifiedDeltaItems, buildCanonicalReferenceAliases, orderSliceSections, rescopePiFrameFromCurrentAsk, stripQuotedFocusaContext, detectForbiddenVisibleOutputLeakClasses, detectScopeFailureSignals, getSemanticMemorySummary, getEcsHandlesSummary, getScopedWorkpointPacket, ensureContinuityId, isProjectRootAuthoritySafe, isWorkpointPacketScopedToCurrentSession, refreshTrajectoryClarityLifecycle, stampWorkpointPacketForCurrentPiSession, adoptPiProjectRoot, projectRootConfirmationRequired } from "./state.js";
import { checkCompactionTier, checkMicroCompact, contextTierLabel } from "./compaction.js";
import { fetchWbmContext, catalogueFromMessages } from "./wbm.js";
import { pushDelta } from "./tools.js";
import { buildFocusaUtilityCard } from "./awareness.js";
import { selectFocusSliceToolAffordances } from "./tool-contracts.js";

let traceBatch: any[] = [];

function queueTraceTelemetry(event: Record<string, any>): void {
  traceBatch.push(event);
}

function flushTraceTelemetryBatch(reason = "turn_end"): void {
  const totalQueued = traceBatch.length;
  const events = traceBatch.splice(0, 100);
  if (!events.length) return;
  focusaPost("/telemetry/trace/batch", {
    batch_id: `pi-trace-batch-${S.turnCount}-${Date.now()}`,
    surface: "pi",
    event_count: events.length,
    total_queued: totalQueued,
    truncated: totalQueued > events.length,
    omitted: Math.max(totalQueued - events.length, 0),
    flush_reason: reason,
    events,
  });
}


async function checkpointDiscontinuity(reason: string, extra: Record<string, any> = {}): Promise<void> {
  if (!S.focusaAvailable) return;
  const root = S.sessionCwd || process.cwd();
  if (!isProjectRootAuthoritySafe(root)) return;
  try {
    await focusaFetch("/workpoint/checkpoint", {
      method: "POST",
      body: JSON.stringify({
        mission: S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || "Pi discontinuity boundary",
        next_slice: S.lastFocusSnapshot.currentFocus || "Resume from typed Workpoint after discontinuity.",
        checkpoint_reason: reason,
        canonical: true,
        promote: true,
        continuity_id: ensureContinuityId(root),
        session_id: S.sessionFrameKey,
        project_root: root,
        source_turn_id: `pi-turn-${S.turnCount}`,
        action_intent: { action_type: "resume_workpoint", target_ref: S.activeFrameId || "pi-session", verification_hooks: [reason], status: "ready" },
        ...extra,
      }),
    });
    const packet = await focusaFetch("/workpoint/resume", { method: "POST", body: JSON.stringify({ mode: "compact_prompt", continuity_id: ensureContinuityId(root), session_id: S.sessionFrameKey, project_root: root }) });
    if (packet?.status === "rejected_scope_mismatch") {
      S.activeWorkpointPacket = null;
      S.activeWorkpointSummary = "";
      return;
    }
    if (packet?.status === "completed") {
      const candidate = packet.resume_packet || packet;
      if (!isWorkpointPacketScopedToCurrentSession(candidate)) {
        S.activeWorkpointPacket = null;
        S.activeWorkpointSummary = "";
        return;
      }
      S.activeWorkpointPacket = stampWorkpointPacketForCurrentPiSession(candidate);
      S.activeWorkpointSummary = packet.rendered_summary || packet.next_step_hint || "";
    }
  } catch { /* best effort */ }
}


function formatWorkpointContextSections(): string[] {
  const packet: any = getScopedWorkpointPacket();
  if (!packet) return [];
  const action = packet.action_intent || {};
  const evidence = Array.isArray(packet.verification_records) ? packet.verification_records : [];
  const blockers = Array.isArray(packet.blockers) ? packet.blockers : [];
  const driftBoundaries = String(packet.next_slice || "")
    .split(/\n+/)
    .filter((line) => /DO_NOT_DRIFT:/i.test(line))
    .map((line) => line.replace(/.*DO_NOT_DRIFT:\s*/i, "").trim())
    .filter(Boolean);
  const activeObjects = Array.isArray(packet.active_object_refs) && packet.active_object_refs.length ? packet.active_object_refs : ["(none)"];
  const verificationHooks = [
    ...(Array.isArray(action.verification_hooks) ? action.verification_hooks : []),
    ...evidence.map((v: any) => v.result || v.evidence_ref || v.target_ref).filter(Boolean),
  ].slice(0, 8);
  const boundaryItems = driftBoundaries.length
    ? driftBoundaries
    : blockers.map((b: any) => b.reason).filter(Boolean).slice(0, 6).length
      ? blockers.map((b: any) => b.reason).filter(Boolean).slice(0, 6)
      : ["Do not override WorkpointResumePacket from transcript tail."];
  return [
    `WORKPOINT: ${S.activeWorkpointSummary || packet.next_slice || packet.mission || "active typed packet present"}`,
    `WORKPOINT_CANONICAL: ${packet.canonical !== false}`,
    `ACTIVE_OBJECT_SET:\n${activeObjects.map((x: string) => `  - ${x}`).join("\n")}`,
    `ACTION_INTENT: ${action.action_type || "unknown"}${action.target_ref ? ` -> ${action.target_ref}` : ""}`,
    `VERIFICATION_HOOKS:\n${(verificationHooks.length ? verificationHooks : ["(none)"]).map((x: string) => `  - ${x}`).join("\n")}`,
    `DRIFT_BOUNDARIES:\n${boundaryItems.map((x: string) => `  - ${x}`).join("\n")}`,
  ];
}


function boundedTrajectoryText(value: any, max = 180): string {
  const text = String(value ?? "").replace(/\s+/g, " ").trim();
  if (!text) return "";
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

function formatTrajectoryFocusSlice(view: any): string[] {
  if (!view || typeof view !== "object") return [];
  const project = view.project_identity || {};
  const trajectory = view.trajectory || {};
  const intelligence = view.intelligence_view || {};
  const sufficiency = intelligence.context_sufficiency || {};
  const candidate = intelligence.next_workpoint_candidate || {};
  const projectApi = project.project_identity_api || {};
  const continuityId = boundedTrajectoryText(S.continuityId, 120);
  const projectRoot = boundedTrajectoryText(project.project_root || projectApi.project_root || S.sessionCwd || process.cwd(), 160);
  const canonicalName = boundedTrajectoryText(project.canonical_name || projectApi.canonical_name, 80);
  const projectId = boundedTrajectoryText(project.project_id || projectApi.project_id, 80);
  const workspaceKind = boundedTrajectoryText(project.workspace_kind || projectApi.workspace_kind, 80);
  const repoRemote = boundedTrajectoryText(project.repo_remote || projectApi.repo_remote, 140);
  const beadsPrefix = boundedTrajectoryText(project.beads_prefix || projectApi.beads_prefix, 40);
  const identityParts = [
    `status=${boundedTrajectoryText(project.status || view.status || "unknown", 40)}`,
    `project_root=${projectRoot}`,
    continuityId ? `continuity_id=${continuityId}` : "continuity_id=(unavailable)",
    project.session_id ? `session_id=${boundedTrajectoryText(project.session_id, 120)}` : "",
    project.confidence ? `confidence=${boundedTrajectoryText(project.confidence, 40)}` : "",
  ].filter(Boolean);
  const infraParts = [
    canonicalName ? `name=${canonicalName}` : "",
    projectId ? `project_id=${projectId}` : "",
    workspaceKind ? `workspace_kind=${workspaceKind}` : "",
    repoRemote ? `repo=${repoRemote}` : "",
    beadsPrefix ? `beads_prefix=${beadsPrefix}` : "",
    "architecture_boundary=use project docs/ontology/evidence; do not infer from folder name alone",
  ].filter(Boolean);
  const goals = [
    trajectory.long_term_goal ? `high=${boundedTrajectoryText(trajectory.long_term_goal, 180)}` : "",
    trajectory.mid_level_goal ? `mid=${boundedTrajectoryText(trajectory.mid_level_goal, 160)}` : "",
    trajectory.low_level_goal ? `low=${boundedTrajectoryText(trajectory.low_level_goal, 160)}` : "",
    trajectory.desired_end_state ? `desired=${boundedTrajectoryText(trajectory.desired_end_state, 180)}` : "",
    trajectory.short_term_goal ? `short=${boundedTrajectoryText(trajectory.short_term_goal, 160)}` : "",
  ].filter(Boolean).join("; ");
  const similarityGroup = trajectory.similarity_group || intelligence.similarity_group || {};
  const similarityBits = [
    similarityGroup.high_level_group_key ? `high_key=${boundedTrajectoryText(similarityGroup.high_level_group_key, 80)}` : "",
    similarityGroup.mid_level_group_key ? `mid_key=${boundedTrajectoryText(similarityGroup.mid_level_group_key, 80)}` : "",
    similarityGroup.low_level_group_key ? `low_key=${boundedTrajectoryText(similarityGroup.low_level_group_key, 80)}` : "",
    `advisory_only=${similarityGroup.advisory_only !== false}`,
    "authority=project_root+continuity_id",
    similarityGroup.must_not_merge_sessions ? "must_not_merge_sessions=true" : "",
  ].filter(Boolean).join("; ");
  const evidence = Array.isArray(trajectory.evidence_refs)
    ? trajectory.evidence_refs.slice(0, 4).map((item: any) => boundedTrajectoryText(item.evidence_ref || item.result || item.target_ref, 120)).filter(Boolean)
    : [];
  const doNotUse = Array.isArray(intelligence.do_not_use)
    ? intelligence.do_not_use.slice(0, 6).map((item: any) => boundedTrajectoryText(item, 100)).filter(Boolean)
    : [];
  const missingFacts = Array.isArray(sufficiency.missing_facts)
    ? sufficiency.missing_facts.slice(0, 6).map((item: any) => boundedTrajectoryText(item, 80)).filter(Boolean)
    : [];
  const candidateBits = [
    candidate.workpoint_id ? `id=${boundedTrajectoryText(candidate.workpoint_id, 80)}` : "",
    candidate.work_item_id ? `work_item=${boundedTrajectoryText(candidate.work_item_id, 80)}` : "",
    candidate.next_slice ? `next=${boundedTrajectoryText(candidate.next_slice, 180)}` : "",
    "advisory_only=true",
  ].filter(Boolean).join("; ");
  const lines = [
    `PROJECT_IDENTITY: ${identityParts.join(" ")}`,
    infraParts.length ? `PROJECT_INFRA: ${infraParts.join("; ")}` : "PROJECT_INFRA: unknown; use focusa_project_identity plus focusa_traverse before architectural assumptions",
    goals ? `TRAJECTORY_GOALS: ${goals}` : "TRAJECTORY_GOALS: definition_status=unclear",
    `TRAJECTORY_SIMILARITY_GROUP: ${similarityBits || "advisory_only=true; authority=project_root+continuity_id; must_not_merge_sessions=true"}`,
    `CURRENT_VERIFIED_STATE: ${boundedTrajectoryText(trajectory.current_state, 220) || "unclear"}`,
    `ACTIVE_GAP: ${boundedTrajectoryText(trajectory.active_gap, 220) || "unclear"}`,
    `WORKPOINT_CANDIDATE: ${candidateBits || "none · advisory_only=true"}`,
    evidence.length ? `TRAJECTORY_EVIDENCE: ${evidence.join("; ")}` : "TRAJECTORY_EVIDENCE: (none)",
    doNotUse.length ? `TRAJECTORY_DO_NOT_USE: ${doNotUse.join("; ")}` : "TRAJECTORY_DO_NOT_USE: (none)",
    `CONTEXT_SUFFICIENCY: score=${sufficiency.score ?? "unknown"} status=${boundedTrajectoryText(sufficiency.status || trajectory.definition_status || view.status || "unknown", 60)} missing=${missingFacts.join(", ") || "none"} recommended=${boundedTrajectoryText(sufficiency.recommended_action, 180) || "none"}`,
  ];
  if (view.canonical === false || view.degraded === true) {
    lines.push("TRAJECTORY_WARNING: advisory degraded projection; verify before treating as canonical");
  }
  return lines;
}


async function getResourceModeFocusSliceLines(): Promise<string[]> {
  if (!S.focusaAvailable) return [];
  try {
    const body = await focusaFetch("/resource/mode");
    const mode = body?.resource_mode || body?.mode || {};
    const budget = mode.budget || body?.budget || {};
    const cold = Array.isArray(mode.cold_surfaces_deferred) ? mode.cold_surfaces_deferred : [];
    const resourceMode = String(mode.mode || body?.mode || "normal");
    if (resourceMode === "normal") return [];
    return [
      `RESOURCE_MODE: ${resourceMode}`,
      `RESOURCE_REASON: ${mode.reason || body?.reason || "unknown"}`,
      `LOWMEM_BUDGET: hot_timeout_ms=${budget.hot_route_timeout_ms ?? "unknown"} default_limit=${budget.max_items_default ?? "unknown"} payload_bytes=${budget.hot_payload_bytes ?? "unknown"}`,
      "CONTEXT_POSTURE: surgical_summary_only",
      "BEST_NEXT_TOOLS: focusa_trajectory_view(mode=summary); focusa_workpoint_resume(mode=compact_prompt); focusa_traverse(limit<=budget)",
      `DO_NOT_USE_BY_DEFAULT: ${(cold.length ? cold : ["full_lineage_tree", "full_ontology_graph", "deep_work_loop_status", "replay_bundles"]).join(", ")}`,
      `PRUNED_COUNTS: transition_omitted_count=${mode.transition_omitted_count ?? 0} rehydrate_ref_budget=${budget.max_rehydrate_refs ?? "unknown"}`,
      "REHYDRATE_REFS: use focusa_traverse with surface/selector/anchor/fields instead of full payload reads",
    ];
  } catch {
    return [];
  }
}

function getToolAffordanceFocusSliceLines(options: {
  resourceModeActive: boolean;
  hasTrajectory: boolean;
  hasWorkpoint: boolean;
  hasOntologyAmbiguity: boolean;
}): string[] {
  const affordances = selectFocusSliceToolAffordances(options);
  return [
    "TOOL_AFFORDANCES:",
    "  best_next:",
    ...affordances.best_next.map((item) => `    - ${item}`),
    "  recovery:",
    ...affordances.recovery.map((item) => `    - ${item}`),
    "  do_not_use:",
    ...affordances.do_not_use.map((item) => `    - ${item}`),
  ];
}

async function getTrajectoryFocusSliceLines(): Promise<string[]> {
  if (!S.focusaAvailable) return [];
  const root = S.sessionCwd || process.cwd();
  if (!isProjectRootAuthoritySafe(root)) return [];
  try {
    const params = new URLSearchParams();
    params.set("mode", "summary");
    params.set("project_root", root);
    if (S.sessionFrameKey) params.set("session_id", S.sessionFrameKey);
    if (S.continuityId) params.set("continuity_id", S.continuityId);
    const view = await focusaFetch(`/trajectory/view?${params.toString()}`);
    return formatTrajectoryFocusSlice(view);
  } catch {
    return [];
  }
}

function providerStatusSuggestsContextOverflow(status: number, headers: Record<string, string> = {}): boolean {
  if ([413].includes(status)) return true;
  if (![400, 422].includes(status)) return false;
  const joined = Object.entries(headers).map(([k, v]) => `${k}:${v}`).join(" ").toLowerCase();
  return /context[_ -]?length|token|too large|payload|maximum context|input exceeds/.test(joined) || joined.length === 0;
}

function textSuggestsContextOverflow(text: string): boolean {
  return /context_length_exceeded|input exceeds the context|maximum context|prompt too long|too many tokens/i.test(text || "");
}

export function registerTurns(pi: ExtensionAPI) {
  // ── before_agent_start (§35.2 behavioral + §29 WBM injection) ────────────
  pi.on("before_agent_start", async (event, ctx) => {
    // Reconnect check
    if (!S.focusaAvailable) {
      const h = await focusaFetch("/health");
      if (h?.ok) {
        S.focusaAvailable = true;
        ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
      }
    }

    // §35.2: Behavioral instructions (ONE TIME per prompt — §36.6 layering)
    const behavioral = [
      "\n## Focusa Cognitive Guidance",
      "You are operating within Focusa, a cognitive runtime that preserves focus and decisions.\n",
      "RULES:",
      "- Use the focusa_decide tool when you make a significant decision",
      "- Use the focusa_constraint tool ONLY for hard constraints (e.g. 'NEVER delete production data', 'must preserve X')",
      "- Use the focusa_failure tool when something fails",
      "- Do NOT record internal monologue, reasoning, or self-referential notes as constraints",
      "  (e.g. 'cannot advance without operator direction' is NOT a constraint — it's context)",
      "- Check the CONSTRAINTS in Focus State before acting — do not violate them",
      "- The DECISIONS listed below were made earlier — do not contradict without explanation",
      "- If context was compacted, Focus State below is your source of truth",
    ].join("\n");
    const scopedWorkpointForPrompt = getScopedWorkpointPacket();
    const workpointLaw = scopedWorkpointForPrompt ? [
      "\n## Focusa Workpoint Continuity Law",
      "If a Focusa WorkpointResumePacket is present, treat it as the authoritative continuation anchor unless the operator explicitly steers elsewhere.",
      "Do not use raw transcript tail to override the active workpoint.",
      ...formatWorkpointContextSections(),
    ].join("\n") : "";
    const utilityCard = "\n" + buildFocusaUtilityCard("system");

    (event as any).systemPrompt = ((event as any).systemPrompt || "") + "\n" + behavioral + workpointLaw + utilityCard;

    if (!S.seenFirstBeforeAgentStart) {
      S.seenFirstBeforeAgentStart = true;
      const visibleCard = buildFocusaUtilityCard("visible");
      pi.sendMessage({ customType: "focusa-utility-card", content: visibleCard, display: true });
      queueTraceTelemetry({ event_type: "focusa_utility_card_visible", turn_id: `pi-turn-${S.turnCount}`, surface: "pi", bytes: visibleCard.length });
    }

    // §29: WBM inbound context injection
    if (S.wbmEnabled) {
      const wbmCtx = await fetchWbmContext();
      (event as any).systemPrompt += "\n\n" + wbmCtx;
    }
  });

  // ── context — DECISIONS ONLY (§36.6, §33.5)
  // ── context (§33.2 live refresh per LLM call) ─────────────────────────────────
  // Focusa Minimal Applicable Slice routing lives here.
  // Consultation trace surfaces emitted from this hot path include:
  // constraints_consulted, decisions_consulted, working_set_used, prior_mission_reused,
  // current_ask_determined, query_scope_built, relevant_context_selected, irrelevant_context_excluded.
  // Per spec G1-07 §AsccSections: all 10 slots must be represented in prompt.
  // Per spec doc 44 §Prompt Serialization: uppercase headers + bullets for list items.
  // Per spec doc 44 §7.1: all 10 ASCC slots in compaction strategy.
  // Per spec doc 44 §33.2: compute a bounded Focusa slice for each LLM call.
  pi.on("context", async (event: any, ctx: any) => {
    if (!S.focusaAvailable || !S.activeFrameId) return;

    const data = await getFocusState();
    if (!data?.fs) return;
    const { fs, frame } = data;

    // §7.1: Format each of the 10 ASCC slots per §Prompt Serialization spec
    const fmt = (label: string, items: string[] | undefined) =>
      items?.length
        ? `${label}:\n${items.map((x: string) => `  - ${x}`).join("\n")}`
        : `${label}:\n  (none)`;

    // §36.7: Budget check — cap injection to 15% of headroom, max 1500 tokens
    const usage = ctx.getContextUsage?.();
    const window = usage?.contextWindow || S.activeContextWindow || 128000;
    if (typeof usage?.contextWindow === "number" && usage.contextWindow > 0) {
      S.activeContextWindow = usage.contextWindow;
    }
    const headroom = usage?.tokens ? window - usage.tokens - 16384 : window;
    const maxTokens = Math.min(Math.max(Math.floor(headroom * 0.15), 200), 1500);

    const scopeKind = S.queryScope?.scopeKind || "mission_carryover";
    const askText = S.currentAsk?.text || "";
    const missionIncluded = shouldIncludeMissionContext(askText, scopeKind, [fs.intent || "", fs.current_focus || "", fs.current_state || "", frame?.title || ""]);
    const projectionKind = "operator_view";
    const viewProfile = "pi_operator_view";
    const activeGoverningPriors: PiGoverningPriorKind[] = [
      "hard_safety_prior",
      "identity_prior",
      "current_ask_prior",
      "affordance_reality_prior",
    ];
    if (scopeKind === "mission_carryover") {
      activeGoverningPriors.push("mission_commitment_prior");
    }

    const relevantDecisions = selectRelevantItems(fs.decisions, askText, { maxItems: 3, fallbackItems: scopeKind === "mission_carryover" ? 2 : 0, minScore: 2 });
    const relevantConstraints = selectRelevantItems(fs.constraints, askText, { maxItems: 3, fallbackItems: scopeKind === "mission_carryover" ? 2 : 0, minScore: 2 });
    const decisionRetention = retentionBucketsFromSelection(relevantDecisions, { maxDecayed: 2, maxHistorical: 2 });
    const constraintRetention = retentionBucketsFromSelection(relevantConstraints, { maxDecayed: 2, maxHistorical: 2 });
    const decayedContextItems = [
      ...constraintRetention.decayed.map((value) => `constraint: ${value}`),
      ...decisionRetention.decayed.map((value) => `decision: ${value}`),
    ];
    const historicalContextItems = [
      ...constraintRetention.historical.map((value) => `constraint: ${value}`),
      ...decisionRetention.historical.map((value) => `decision: ${value}`),
    ];
    const recentResults = selectRelevantItems(fs.recent_results, askText, { maxItems: 2, fallbackItems: scopeKind === "mission_carryover" ? 1 : 0, minScore: 2 });
    const nextSteps = selectRelevantItems(fs.next_steps, askText, { maxItems: 2, fallbackItems: scopeKind === "mission_carryover" ? 1 : 0, minScore: 2 });
    const openQuestions = selectRelevantItems(fs.open_questions, askText, { maxItems: 2, fallbackItems: 0, minScore: 2 });
    const failures = selectRelevantItems(fs.failures, askText, { maxItems: 2, fallbackItems: scopeKind === "correction" ? 1 : 0, minScore: 2 });
    const artifactLabels = fs.artifacts?.map((a: any) => `${a.kind}:${a.label}${a.path_or_id ? "@" + a.path_or_id : ""}`) || [];
    const relevantArtifacts = selectRelevantItems(artifactLabels, askText, { maxItems: 2, fallbackItems: scopeKind === "mission_carryover" ? 1 : 0, minScore: 2 });
    const includeAuxContext = maxTokens >= 350;
    const [semanticMemory, ecsHandles] = includeAuxContext
      ? await Promise.all([getSemanticMemorySummary(), getEcsHandlesSummary()])
      : [null, null];
    const workingSetItems = formatWorkingSetItems(semanticMemory?.semantic);
    const relevantWorkingSet = selectRelevantRankedItems(workingSetItems, askText, {
      maxItems: 3,
      fallbackItems: scopeKind === "mission_carryover" ? 2 : 0,
      minScore: 2,
      allowStaleFallback: scopeKind === "mission_carryover",
      governingPriors: activeGoverningPriors,
    });
    const verifiedDeltaItems = formatVerifiedDeltaItems(ecsHandles?.handles);
    const relevantVerifiedDeltas = selectRelevantRankedItems(verifiedDeltaItems, askText, {
      maxItems: 2,
      fallbackItems: scopeKind === "mission_carryover" ? 1 : 0,
      minScore: 2,
      allowStaleFallback: scopeKind === "mission_carryover",
      governingPriors: activeGoverningPriors,
    });
    const canonicalReferenceAliases = buildCanonicalReferenceAliases(relevantVerifiedDeltas.items);
    let ontologyContext: any = null;
    if (S.focusaAvailable && includeAuxContext) {
      try {
        ontologyContext = await focusaFetch("/ontology/context", {
          method: "POST",
          body: JSON.stringify({
            current_ask: S.currentAsk?.text || askText,
            frame_id: S.activeFrameId,
            workpoint_id: getScopedWorkpointPacket()?.workpoint_id || getScopedWorkpointPacket()?.workpoint?.workpoint_id,
            target_refs: canonicalReferenceAliases.slice(0, 6),
            budget_tokens: Math.min(maxTokens, 800),
            operator_steering_detected: isOperatorSteeringInput(askText, S.currentAsk?.kind || "unknown"),
            active_object_refs: [],
          }),
        });
      } catch {
        ontologyContext = null;
      }
    }
    const ontologyPayload = ontologyContext?.ontology_context || ontologyContext;
    const ontologyObjectLines = Array.isArray(ontologyPayload?.active_object_set)
      ? ontologyPayload.active_object_set.slice(0, 6).map((item: any) => `${item.object_type || "object"}:${item.id || "unknown"} (${item.uncertainty || "unknown"})`)
      : [];
    const ontologyLinkLines = Array.isArray(ontologyPayload?.relevant_link_paths)
      ? ontologyPayload.relevant_link_paths.slice(0, 6).map((item: any) => String(item.path || item))
      : [];
    const ontologyActionLines = Array.isArray(ontologyPayload?.valid_next_actions)
      ? ontologyPayload.valid_next_actions.slice(0, 4).map((item: any) => String(item.name || "unknown_action"))
      : [];
    const ontologyBlockedLines = Array.isArray(ontologyPayload?.blocked_affordances)
      ? ontologyPayload.blocked_affordances.slice(0, 4).map((item: any) => String(item.name || item.id || item))
      : [];
    const ontologyEvidenceLines = Array.isArray(ontologyPayload?.evidence_handles)
      ? ontologyPayload.evidence_handles.slice(0, 4).map((item: any) => `${item.kind || "evidence"}:${item.label || item.id || "unknown"}`)
      : [];
    const ontologyUncertaintyLines = Array.isArray(ontologyPayload?.uncertainty_flags)
      ? ontologyPayload.uncertainty_flags.slice(0, 6).map((item: any) => String(item))
      : [];
    const trajectoryLines = await getTrajectoryFocusSliceLines();
    const resourceModeLines = await getResourceModeFocusSliceLines();
    const toolAffordanceLines = getToolAffordanceFocusSliceLines({
      resourceModeActive: resourceModeLines.length > 0,
      hasTrajectory: trajectoryLines.length > 0,
      hasWorkpoint: Boolean(S.activeWorkpointPacket),
      hasOntologyAmbiguity: ontologyObjectLines.length > 1 || ontologyUncertaintyLines.length > 0,
    });

    const sectionEntries = [
      { key: "projection_kind", text: `PROJECTION_KIND: ${projectionKind}`, include: true, selectedCount: 1, excludedCount: 0, priority: 0, relevanceScore: 100 },
      { key: "view_profile", text: `VIEW_PROFILE: ${viewProfile}`, include: true, selectedCount: 1, excludedCount: 0, priority: 1, relevanceScore: 100 },
      { key: "current_ask", text: `CURRENT_ASK: ${S.currentAsk?.text || askText || "(none)"}`, include: Boolean(S.currentAsk?.text || askText), selectedCount: 1, excludedCount: 0, priority: 2, relevanceScore: 100 },
      { key: "query_scope", text: `QUERY_SCOPE: ${scopeKind} · ${S.queryScope?.carryoverPolicy || "allow_if_relevant"}`, include: true, selectedCount: 1, excludedCount: 0, priority: 3, relevanceScore: 100 },
      buildSliceSection("resource_mode", "RESOURCE_MODE", resourceModeLines, resourceModeLines.length > 0, (values) => values.join("\n"), 0, 4, 100),
      buildSliceSection("trajectory", "PROJECT_TRAJECTORY", trajectoryLines, trajectoryLines.length > 0, (values) => `PROJECT_TRAJECTORY:\n${values.map((value) => `  - ${value}`).join("\n")}`, 0, 5, 100),
      buildSliceSection("workpoint", "WORKPOINT", formatWorkpointContextSections(), Boolean(S.activeWorkpointPacket), (values) => values.join("\n"), 0, 6, 100),
      buildSliceSection("tool_affordances", "TOOL_AFFORDANCES", toolAffordanceLines, toolAffordanceLines.length > 0, (values) => values.join("\n"), 0, 7, 95),
      { key: "focus_frame", text: `FOCUS_FRAME: ${frame?.title || "(untitled)"}`, include: missionIncluded && Boolean(frame?.title), selectedCount: frame?.title ? 1 : 0, excludedCount: 0, priority: 10, relevanceScore: missionIncluded ? 50 : 0 },
      { key: "current_focus", text: `CURRENT_FOCUS: ${fs.current_focus || fs.current_state || "(none)"}`, include: missionIncluded && Boolean(fs.current_focus || fs.current_state), selectedCount: (fs.current_focus || fs.current_state) ? 1 : 0, excludedCount: 0, priority: 11, relevanceScore: missionIncluded ? 45 : 0 },
      { key: "intent", text: `INTENT: ${fs.intent || "(none)"}`, include: missionIncluded && Boolean(fs.intent), selectedCount: fs.intent ? 1 : 0, excludedCount: 0, priority: 12, relevanceScore: missionIncluded ? 40 : 0 },
      { key: "projection_boundary", text: `PROJECTION_BOUNDARY: token_budget=${maxTokens} carryover=${S.queryScope?.carryoverPolicy || "allow_if_relevant"} mission=${missionIncluded ? "included" : "suppressed"}` , include: true, selectedCount: 1, excludedCount: 0, priority: 13, relevanceScore: 90 },
      { key: "canonical_sources", text: `CANONICAL_SOURCES: focus_state semantic_memory ecs_handles reference_index`, include: true, selectedCount: 4, excludedCount: 0, priority: 14, relevanceScore: 90 },
      buildSliceSection("canonical_references", "REFERENCE_ALIASES", canonicalReferenceAliases, canonicalReferenceAliases.length > 0, (values) => fmt("REFERENCE_ALIASES", values), 0, 15, 85),
      buildSliceSection("ontology_active_objects", "ACTIVE_OBJECT_SET", ontologyObjectLines, ontologyObjectLines.length > 0, (values) => fmt("ACTIVE_OBJECT_SET", values), 0, 16, 92),
      buildSliceSection("ontology_link_paths", "RELEVANT_LINK_PATHS", ontologyLinkLines, ontologyLinkLines.length > 0, (values) => fmt("RELEVANT_LINK_PATHS", values), 0, 17, 90),
      buildSliceSection("ontology_next_actions", "VALID_NEXT_ACTIONS", ontologyActionLines, ontologyActionLines.length > 0, (values) => fmt("VALID_NEXT_ACTIONS", values), 0, 18, 88),
      buildSliceSection("ontology_blocked_affordances", "BLOCKED_AFFORDANCES", ontologyBlockedLines, ontologyBlockedLines.length > 0, (values) => fmt("BLOCKED_AFFORDANCES", values), 0, 19, 82),
      buildSliceSection("ontology_evidence_handles", "EVIDENCE_HANDLES", ontologyEvidenceLines, ontologyEvidenceLines.length > 0, (values) => fmt("EVIDENCE_HANDLES", values), 0, 20, 84),
      buildSliceSection("ontology_uncertainty", "UNCERTAINTY_FLAGS", ontologyUncertaintyLines, ontologyUncertaintyLines.length > 0, (values) => fmt("UNCERTAINTY_FLAGS", values), 0, 21, 80),
      buildSliceSection("working_set", "WORKING_SET", relevantWorkingSet.items, relevantWorkingSet.items.length > 0, (values) => fmt("WORKING_SET", values), relevantWorkingSet.excluded.length, 20, selectionRelevanceScore(relevantWorkingSet)),
      buildSliceSection("constraints", "CONSTRAINTS", relevantConstraints.items, relevantConstraints.items.length > 0, (values) => fmt("CONSTRAINTS", values), relevantConstraints.excluded.length, 20, selectionRelevanceScore(relevantConstraints)),
      buildSliceSection("decisions", "DECISIONS", relevantDecisions.items, relevantDecisions.items.length > 0, (values) => fmt("DECISIONS", values), relevantDecisions.excluded.length, 20, selectionRelevanceScore(relevantDecisions)),
      buildSliceSection("decayed_context", "DECAYED_CONTEXT", decayedContextItems, (scopeKind === "mission_carryover" || scopeKind === "correction" || scopeKind === "meta") && decayedContextItems.length > 0, (values) => fmt("DECAYED_CONTEXT", values), 0, 21, 6),
      buildSliceSection("historical_context", "HISTORICAL_CONTEXT", historicalContextItems, (scopeKind === "mission_carryover" || scopeKind === "meta") && historicalContextItems.length > 0, (values) => fmt("HISTORICAL_CONTEXT", values), 0, 22, 4),
      buildSliceSection("verified_deltas", "VERIFIED_DELTAS", relevantVerifiedDeltas.items, relevantVerifiedDeltas.items.length > 0, (values) => fmt("VERIFIED_DELTAS", values), relevantVerifiedDeltas.excluded.length, 20, selectionRelevanceScore(relevantVerifiedDeltas)),
      buildSliceSection("recent_results", "RECENT_RESULTS", recentResults.items, scopeKind !== "fresh_question" && recentResults.items.length > 0, (values) => fmt("RECENT_RESULTS", values), recentResults.excluded.length, 20, selectionRelevanceScore(recentResults)),
      buildSliceSection("failures", "FAILURES", failures.items, (scopeKind === "correction" || scopeKind === "mission_carryover") && failures.items.length > 0, (values) => fmt("FAILURES", values), failures.excluded.length, 20, selectionRelevanceScore(failures)),
      buildSliceSection("next_steps", "NEXT_STEPS", nextSteps.items, scopeKind === "mission_carryover" && nextSteps.items.length > 0, (values) => fmt("NEXT_STEPS", values), nextSteps.excluded.length, 20, selectionRelevanceScore(nextSteps)),
      buildSliceSection("artifacts", "ARTIFACT_HANDLES", relevantArtifacts.items, relevantArtifacts.items.length > 0, (values) => fmt("ARTIFACT_HANDLES", values), relevantArtifacts.excluded.length, 20, selectionRelevanceScore(relevantArtifacts)),
      buildSliceSection("open_questions", "OPEN_QUESTIONS", openQuestions.items, scopeKind === "meta" && openQuestions.items.length > 0, (values) => fmt("OPEN_QUESTIONS", values), openQuestions.excluded.length, 20, selectionRelevanceScore(openQuestions)),
    ];

    const scopedEntries = orderSliceSections(sectionEntries).filter((entry) => entry.include);
    const scopeExcludedLabels = sectionEntries.filter((entry) => !entry.include).map((entry) => entry.key);
    const retainedDecisionHistoryCount = decisionRetention.decayed.length + decisionRetention.historical.length;
    const retainedConstraintHistoryCount = constraintRetention.decayed.length + constraintRetention.historical.length;
    const irrelevantExcludedLabels = [
      ...(relevantDecisions.excluded.length > retainedDecisionHistoryCount ? ["decisions"] : []),
      ...(relevantConstraints.excluded.length > retainedConstraintHistoryCount ? ["constraints"] : []),
      ...(relevantWorkingSet.excluded.length ? ["working_set"] : []),
      ...(relevantVerifiedDeltas.excluded.length ? ["verified_deltas"] : []),
      ...(recentResults.excluded.length ? ["recent_results"] : []),
      ...(nextSteps.excluded.length ? ["next_steps"] : []),
      ...(openQuestions.excluded.length ? ["open_questions"] : []),
      ...(failures.excluded.length ? ["failures"] : []),
      ...(relevantArtifacts.excluded.length ? ["artifacts"] : []),
    ];

    // §Prompt Serialization: uppercase section headers, bullets for list items
    const lines: string[] = [
      `[Focusa Focus Slice — minimal applicable context]`,
      ...scopedEntries.map((entry) => entry.text),
    ];

    // §36.7: Budget cap — truncate if over token budget
    let text = lines.join("\n");
    const fullTokens = estimateTokens(text);
    const truncated = fullTokens > maxTokens;
    if (truncated) {
      // Truncate from bottom (NOTES → FAILURES → RECENT_RESULTS, etc.)
      text = lines.slice(0, 4).join("\n") +
        `\n[... Focus State truncated — ${fullTokens - maxTokens} tokens over budget]`;
    }
    const injectedTokens = estimateTokens(text);

    // Minimal context-injection trace telemetry for SPEC 56 / doc 78 gap closure.
    // Emit explicit typed trace events for the fields we can objectively compute today,
    // without pretending the hot path already has richer routing/hijack semantics.
    const lastUserMsg = [...(event.messages || [])].reverse().find((m: any) => m?.role === "user");
    const lastUserText = extractText(lastUserMsg?.content || "").slice(0, 200);
    const priorMissionReused = scopeKind === "mission_carryover" && Boolean(fs.intent || fs.current_focus || fs.current_state || (fs.decisions && fs.decisions.length));
    const budgetExcludedLabels = truncated ? ["artifacts", "verified_deltas", "working_set", "constraints", "open_questions", "next_steps", "recent_results", "failures"] : [];
    const relevantContextLabels = scopedEntries.map((entry) => entry.key);
    const focusSliceRelevanceScore = scopedEntries.length
      ? scopedEntries.reduce((sum, entry) => sum + (entry.relevanceScore || 0), 0) / scopedEntries.length
      : 0;
    const excludedContext = Array.from(new Set([...scopeExcludedLabels, ...irrelevantExcludedLabels, ...budgetExcludedLabels]));
    const contextTurnId = `pi-turn-${S.turnCount}`;
    const scopeSourceTurnId = S.queryScope?.sourceTurnId || S.currentAsk?.sourceTurnId || contextTurnId;
    const workingSetPriorHits = relevantWorkingSet.scores
      .filter(({ value, priorBoost }) => relevantWorkingSet.items.includes(value) && (priorBoost || 0) > 0)
      .map(({ value, priorBoost, appliedPriors }) => ({ value, priorBoost: priorBoost || 0, appliedPriors: appliedPriors || [] }));
    const verifiedDeltaPriorHits = relevantVerifiedDeltas.scores
      .filter(({ value, priorBoost }) => relevantVerifiedDeltas.items.includes(value) && (priorBoost || 0) > 0)
      .map(({ value, priorBoost, appliedPriors }) => ({ value, priorBoost: priorBoost || 0, appliedPriors: appliedPriors || [] }));
    const resetReason = scopeKind === "fresh_question"
      ? "fresh_scope"
      : scopeKind === "correction"
        ? "correction_reset"
        : null;
    const exclusionReason = truncated
      ? "budget_truncation"
      : resetReason || (irrelevantExcludedLabels.length ? "irrelevance" : "none");
    S.excludedContext = {
      labels: excludedContext,
      reason: exclusionReason,
      sourceTurnId: scopeSourceTurnId,
      updatedAt: Date.now(),
    };

    if (S.focusaAvailable) {
      focusaPost("/work-loop/context", {
        excluded_context_reason: exclusionReason,
        excluded_context_labels: excludedContext,
        source_turn_id: scopeSourceTurnId,
      });
    }

    if (S.cfg?.emitMetrics) {
      const common = {
        turn_id: contextTurnId,
        frame_id: S.activeFrameId,
        surface: "pi",
        routing_mode: "minimal_focus_slice_builder",
        focus_slice_estimated_tokens: injectedTokens,
        focus_slice_full_tokens: fullTokens,
        focus_slice_truncated: truncated,
        excluded_context: excludedContext,
        current_ask_kind: S.currentAsk?.kind,
        query_scope_kind: S.queryScope?.scopeKind,
        carryover_policy: S.queryScope?.carryoverPolicy,
        projection_kind: projectionKind,
        view_profile: viewProfile,
      };
      if (lastUserText) {
        queueTraceTelemetry({
          event_type: "operator_subject",
          ...common,
          operator_subject_preview: lastUserText,
        });
        queueTraceTelemetry({
          event_type: "active_subject_after_routing",
          ...common,
          active_subject_after_routing: lastUserText,
        });
      }
      queueTraceTelemetry({
        event_type: "prior_mission_reused",
        ...common,
        prior_mission_reused: priorMissionReused,
      });
      queueTraceTelemetry({
        event_type: "focus_slice_size",
        ...common,
        focus_slice_size: lines.length,
      });
      queueTraceTelemetry({
        event_type: "focus_slice_relevance_score",
        ...common,
        focus_slice_relevance_score: focusSliceRelevanceScore,
      });
      queueTraceTelemetry({
        event_type: "mission_frame_context",
        ...common,
        projection_boundary: {
          token_budget: maxTokens,
          carryover_policy: S.queryScope?.carryoverPolicy,
          mission_included: missionIncluded,
        },
        canonical_sources: ["focus_state", "semantic_memory", "ecs_handles", "reference_index"],
        retention_policy: "active_use_reduction_over_destructive_loss",
        retention_buckets: {
          decisions: {
            active: decisionRetention.active.length,
            decayed: decisionRetention.decayed.length,
            historical: decisionRetention.historical.length,
          },
          constraints: {
            active: constraintRetention.active.length,
            decayed: constraintRetention.decayed.length,
            historical: constraintRetention.historical.length,
          },
        },
        resolved_reference_count: canonicalReferenceAliases.length,
      });
      queueTraceTelemetry({
        event_type: "relevant_context_selected",
        ...common,
        relevant_context_labels: relevantContextLabels,
        selected_counts: Object.fromEntries(scopedEntries.map((entry) => [entry.key, entry.selectedCount || 0])),
      });
      queueTraceTelemetry({
        event_type: "governing_priors_applied",
        ...common,
        governing_priors: activeGoverningPriors,
        ranking_consumers: ["working_set", "verified_deltas"],
        prior_hits: {
          working_set: workingSetPriorHits,
          verified_deltas: verifiedDeltaPriorHits,
        },
      });
      if (relevantWorkingSet.items.length) {
        queueTraceTelemetry({
          event_type: "working_set_used",
          ...common,
          working_set_used: relevantWorkingSet.items,
          selected_count: relevantWorkingSet.items.length,
          pruned_count: workingSetItems.length - relevantWorkingSet.items.length,
          retention_policy: "active_use_reduction_over_destructive_loss",
        });
      }
      if (relevantVerifiedDeltas.items.length) {
        queueTraceTelemetry({
          event_type: "verification_result",
          ...common,
          verification_surface: "verified_deltas",
          selected_count: relevantVerifiedDeltas.items.length,
          pruned_count: verifiedDeltaItems.length - relevantVerifiedDeltas.items.length,
          retention_policy: "active_use_reduction_over_destructive_loss",
          resolved_reference_count: canonicalReferenceAliases.length,
          resolved_reference_aliases: canonicalReferenceAliases,
        });
      }
      if (excludedContext.length) {
        queueTraceTelemetry({
          event_type: "irrelevant_context_excluded",
          ...common,
          exclusion_reason: exclusionReason,
          excluded_context_labels: excludedContext,
        });
      }
      if (!missionIncluded && (scopeKind === "fresh_question" || scopeKind === "correction" || excludedContext.length > 0)) {
        queueTraceTelemetry({
          event_type: "subject_hijack_prevented",
          ...common,
          subject_hijack_prevented: true,
          prevented_by: exclusionReason,
        });
      }
    }

    // §33.2: Prepend Focus State as first message before every LLM call
    return { messages: [{ role: "user" as const, content: [{ type: "text" as const, text }] }, ...(event.messages || [])] };
  });

  // ── input (§36.3 signal + §35.7 correction — single handler) ──────────────
  pi.on("input", async (event, _ctx) => {
    const text = (event as any).text || (event as any).message || "";
    const cleanedText = stripQuotedFocusaContext(String(text));
    // Input is the pre-turn boundary for the upcoming model call.
    // Use the next turn id so CurrentAsk/QueryScope survive unchanged into context injection.
    const sourceTurnId = `pi-turn-${S.turnCount + 1}`;
    const packageUpdateCommand = /^\s*(update|pi\s+update|\/update)\s*$/i.test(String(text));
    const askKind = packageUpdateCommand ? "meta" : classifyCurrentAsk(String(text));
    const storedAskText = cleanedText || (askKind === "meta" ? "" : String(text));
    S.currentAsk = {
      text: storedAskText.slice(0, 500),
      kind: askKind,
      sourceTurnId,
      updatedAt: Date.now(),
    };
    const queryScope = deriveQueryScope(askKind);
    const steeringDetected = isOperatorSteeringInput(String(text), askKind);
    S.queryScope = {
      ...queryScope,
      sourceTurnId,
      updatedAt: Date.now(),
    };
    S.excludedContext = {
      labels: [],
      reason: askKind === "question"
        ? "fresh_scope"
        : askKind === "correction"
          ? "correction_reset"
          : "none",
      sourceTurnId,
      updatedAt: Date.now(),
    };

    const projectRoot = adoptPiProjectRoot((_ctx as any)?.cwd);
    const rootConfirmed = !projectRootConfirmationRequired(projectRoot);
    if (S.focusaAvailable && S.activeFrameId && !packageUpdateCommand && rootConfirmed) {
      await rescopePiFrameFromCurrentAsk(projectRoot, "pi-post-input-rescope").catch(() => null);
      await getFocusState().catch(() => null);
    }

    if (S.focusaAvailable) {
      focusaFetch("/work-loop/context", {
        method: "POST",
        headers: { "x-focusa-writer-id": `pi-${process.pid}` },
        body: JSON.stringify({
          current_ask: S.currentAsk.text,
          ask_kind: S.currentAsk.kind,
          scope_kind: S.queryScope.scopeKind,
          carryover_policy: S.queryScope.carryoverPolicy,
          excluded_context_reason: S.excludedContext.reason,
          excluded_context_labels: [],
          source_turn_id: sourceTurnId,
          operator_steering_detected: steeringDetected,
        }),
      }).catch(() => null);
      if (steeringDetected && rootConfirmed) {
        refreshTrajectoryClarityLifecycle("operator_steering", projectRoot).catch(() => null);
      }
    }

    if (S.cfg?.emitMetrics) {
      const common = {
        turn_id: sourceTurnId,
        frame_id: S.activeFrameId,
        surface: "pi",
        current_ask_kind: S.currentAsk.kind,
        query_scope_kind: S.queryScope.scopeKind,
        carryover_policy: S.queryScope.carryoverPolicy,
      };
      queueTraceTelemetry({
        event_type: "operator_subject",
        ...common,
        operator_subject_preview: S.currentAsk.text.slice(0, 200),
      });
      queueTraceTelemetry({
        event_type: "current_ask_determined",
        ...common,
        current_ask_text_preview: S.currentAsk.text.slice(0, 200),
      });
      queueTraceTelemetry({
        event_type: "query_scope_built",
        ...common,
        query_scope_kind: S.queryScope.scopeKind,
        carryover_policy: S.queryScope.carryoverPolicy,
      });
      queueTraceTelemetry({
        event_type: "steering_detected",
        ...common,
        steering_detected: steeringDetected,
      });
    }

    if (S.focusaAvailable) {
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "user_input", surface: "pi",
        payload: { length: text.length, preview: String(text).slice(0, 200) },
      });
    }

    const lower = String(text).toLowerCase();
    const corrections = ["no that is wrong", "revert", "undo", "that's incorrect", "wrong approach", "go back", "not what i asked"];
    if (corrections.some(c => lower.includes(c))) {
      // Correction is steering signal, not canonical failure.
      // Keep as telemetry/trust update to avoid stale Known Failures contamination.
      if (S.focusaAvailable) {
        queueTraceTelemetry({
          event_type: "operator_correction_detected",
          turn_id: `pi-turn-${S.turnCount}`,
          frame_id: S.activeFrameId,
          surface: "pi",
          correction_preview: String(text).slice(0, 160),
        });
      }
      // §35.7/§29: WBM trust metric update on correction
      if (S.wbmEnabled) {
        wbExec(["trust", "set", "--corrections", "+1"]).catch(() => {});
      }
    }
  });

  // ── turn_start (§34.2B) ───────────────────────────────────────────────────
  pi.on("turn_start", async (_event, _ctx) => {
    S.turnCount++;
    S.lastStreamLen = 0;
    S.toolUsageBatch = [];
    // Reset dedup flag so next compaction can re-trigger auto-resume
    S.compactResumePending = false;
    if (S.focusaAvailable) {
      focusaPost("/turn/start", { turn_id: `pi-turn-${S.turnCount}`, frame_id: S.activeFrameId });
    }
  });

  // ── turn_end (§35.5 tokens + §37.3 widget + §10.4 badges + §20 tier + §21 micro) ─
  pi.on("turn_end", async (event, ctx) => {
    const ev = event as any;
    const cfg = S.cfg;

    // §35.5: Token counts + assistant output
    if (S.focusaAvailable) {
      const assistantOutput = extractText(ev.message?.content || ev.message || "");
      const detectedLeakClasses = detectForbiddenVisibleOutputLeakClasses(assistantOutput);
      if (detectedLeakClasses.length) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "visible_output_leak",
          surface: "pi",
          frame_id: S.activeFrameId,
          payload: {
            leak_classes: detectedLeakClasses,
            preview: assistantOutput.slice(0, 280),
          },
        });
        queueTraceTelemetry({
          event_type: "visible_output_leak_detected",
          turn_id: `pi-turn-${S.turnCount}`,
          frame_id: S.activeFrameId,
          surface: "pi",
          leak_classes: detectedLeakClasses,
        });
      }

      const scopeFailures = detectScopeFailureSignals({
        askText: S.currentAsk?.text || "",
        askKind: S.currentAsk?.kind || "unknown",
        scopeKind: S.queryScope?.scopeKind || "mission_carryover",
        assistantOutput,
        leakClasses: detectedLeakClasses,
      });
      const scopeTraceBase = {
        turn_id: `pi-turn-${S.turnCount}`,
        frame_id: S.activeFrameId,
        surface: "pi",
        ask_kind: S.currentAsk?.kind || "unknown",
        scope_kind: S.queryScope?.scopeKind || "mission_carryover",
        carryover_policy: S.queryScope?.carryoverPolicy || "allow_if_relevant",
      };
      if (scopeFailures.length || detectedLeakClasses.length) {
        refreshTrajectoryClarityLifecycle("failure_or_degradation", S.sessionCwd || process.cwd()).catch(() => null);
      }
      if (scopeFailures.length === 0) {
        queueTraceTelemetry({
          event_type: "scope_verified",
          ...scopeTraceBase,
          verified: true,
          excluded_context_reason: S.excludedContext?.reason || "none",
        });
      } else {
        for (const failure of scopeFailures) {
          if (failure.kind === "scope_contamination") {
            queueTraceTelemetry({
              event_type: "scope_contamination_detected",
              ...scopeTraceBase,
              failure_kind: failure.kind,
              severity: failure.severity,
              reason: failure.reason,
            });
          } else if (failure.kind === "wrong_question_answered") {
            queueTraceTelemetry({
              event_type: "wrong_question_detected",
              ...scopeTraceBase,
              failure_kind: failure.kind,
              severity: failure.severity,
              reason: failure.reason,
            });
          } else if (failure.kind === "answer_broadening") {
            queueTraceTelemetry({
              event_type: "answer_broadening_detected",
              ...scopeTraceBase,
              failure_kind: failure.kind,
              severity: failure.severity,
              reason: failure.reason,
            });
          }

          queueTraceTelemetry({
            event_type: "scope_failure_recorded",
            ...scopeTraceBase,
            failure_kind: failure.kind,
            severity: failure.severity,
            reason: failure.reason,
          });
        }
      }

      if (textSuggestsContextOverflow(assistantOutput)) {
        await checkpointDiscontinuity("context_overflow", { active_object_refs: ["provider_error_text:context_length_exceeded"] });
      }

      const expectedActionType = S.activeWorkpointPacket?.action_intent?.action_type;
      if (expectedActionType && assistantOutput.trim()) {
        focusaFetch("/workpoint/drift-check", {
          method: "POST",
          body: JSON.stringify({
            latest_action: assistantOutput.slice(0, 2000),
            expected_action_type: expectedActionType,
            emit: true,
          }),
        }).then((drift: any) => {
          queueTraceTelemetry({
            event_type: drift?.drift_detected ? "workpoint_drift_detected" : "workpoint_drift_checked",
            turn_id: `pi-turn-${S.turnCount}`,
            frame_id: S.activeFrameId,
            surface: "pi",
            workpoint_id: drift?.workpoint_id || S.activeWorkpointPacket?.workpoint_id,
            expected_action_type: expectedActionType,
            drift_detected: Boolean(drift?.drift_detected),
            next_step_hint: drift?.next_step_hint,
          });
        }).catch(() => {
          queueTraceTelemetry({
            event_type: "workpoint_drift_check_unavailable",
            turn_id: `pi-turn-${S.turnCount}`,
            frame_id: S.activeFrameId,
            surface: "pi",
            expected_action_type: expectedActionType,
          });
        });
      }

      focusaPost("/turn/complete", {
        turn_id: `pi-turn-${S.turnCount}`,
        frame_id: S.activeFrameId,
        assistant_output: assistantOutput,
        artifacts: [],
        errors: [],
        prompt_tokens: ev.usage?.inputTokens || ev.usage?.input || 0,
        completion_tokens: ev.usage?.outputTokens || ev.usage?.output || 0,
        tokens: {
          input: ev.usage?.inputTokens || ev.usage?.input || 0,
          output: ev.usage?.outputTokens || ev.usage?.output || 0,
          cache_read: ev.usage?.cacheReadInputTokens || 0,
          cache_write: ev.usage?.cacheCreationInputTokens || 0,
        },
      });
    }

    // §33.4: Flush batched tool usage
    if (S.focusaAvailable && S.toolUsageBatch.length) {
      focusaPost("/telemetry/tool-usage", { turn_id: `pi-turn-${S.turnCount}`, tools: S.toolUsageBatch });
      queueTraceTelemetry({
        event_type: "tools_invoked",
        turn_id: `pi-turn-${S.turnCount}`,
        frame_id: S.activeFrameId,
        surface: "pi",
        tools: S.toolUsageBatch,
      });
      S.toolUsageBatch = [];
    }

    // §37.3 + §10.4: Widget with all badges
    const w: string[] = [];
    let liveFocus: Awaited<ReturnType<typeof getFocusState>> = null;
    if (S.focusaAvailable) liveFocus = await getFocusState();
    const snapshot = getEffectiveFocusSnapshot(liveFocus?.fs);
    if (snapshot.decisions.length) w.push(`📌 ${snapshot.decisions.length} decisions`);
    if (snapshot.constraints.length) w.push(`🔒 ${snapshot.constraints.length} constraints`);
    if (snapshot.failures.length) w.push(`⚠️ ${snapshot.failures.length} failures`);
    if (S.wbmEnabled) w.push(S.wbmDeep ? "⚡ WBM deep" : "🤖 WBM on");
    if (S.currentTier && typeof S.currentContextPct === "number") {
      const label = contextTierLabel(S.currentTier);
      w.push(`📦 Context ${S.currentContextPct.toFixed(0)}% ${label}`);
    }
    // §10.4: Degraded-context badge
    if (!S.focusaAvailable) w.push("⚪ degraded");
    // §10.4: Thesis snippet
    if (liveFocus?.frame?.thread_thesis) w.push(`🎯 ${liveFocus.frame.thread_thesis.slice(0, 50)}`);
    // §30: Metacognitive indicator
    if (S.lastMetacogEvent) w.push(`✨ ${S.lastMetacogEvent}`);
    ctx.ui.setWidget("focusa", w.length ? w : undefined);

    // §34.2C: Update Focus State on significant progress
    if (S.focusaAvailable && S.activeFrameId) {
      const hasSignificant = S.localDecisions.length > 0 || S.localConstraints.length > 0 || S.localFailures.length > 0;
      if (hasSignificant) {
        await pushDelta({
          decisions: S.localDecisions.slice(-5),
          constraints: S.localConstraints.slice(-5),
          failures: S.localFailures.slice(-3),
        }).catch(() => null);
      }
    }

    flushTraceTelemetryBatch("turn_end");

    // §20: Compaction tier check
    await checkCompactionTier(ctx);
    // §21: Micro-compact check
    await checkMicroCompact();
  });

  // ── message_update (§36.1 streaming delta) ────────────────────────────────
  pi.on("message_update", async (event, _ctx) => {
    if (!S.focusaAvailable) return;
    const fullText = extractText((event as any).message?.content);
    if (S.turnCount % 10 !== 0 && fullText.length - S.lastStreamLen < 500) return;
    const delta = fullText.slice(S.lastStreamLen);
    if (!delta) return;
    S.lastStreamLen = fullText.length;
    focusaPost("/turn/append", { turn_id: `pi-turn-${S.turnCount}`, delta: delta.slice(-500) });
  });

  // ── model_select (§37.8) ──────────────────────────────────────────────────
  pi.on("model_select", async (event, _ctx) => {
    if (!S.focusaAvailable) return;
    const model = (event as any).model;
    S.activeContextWindow = model?.contextWindow || S.activeContextWindow;
    // §37.8: Wire model change to Focusa with frame context
    await checkpointDiscontinuity("model_switch", { active_object_refs: [model?.id || "unknown-model"] });
    focusaPost("/focus-gate/ingest-signal", {
      signal_type: "model_change",
      surface: "pi",
      frame_id: S.activeFrameId,
      payload: { model_id: model?.id || "unknown", context_window: model?.contextWindow || S.activeContextWindow },
    });
  });


  // Provider overflow boundary: Pi auto-compacts, but Focusa checkpoints first when HTTP status exposes overflow-like failure.
  (pi as any).on("after_provider_response", async (event: any, _ctx: any) => {
    const status = Number((event as any).status || 0);
    const headers = ((event as any).headers || {}) as Record<string, string>;
    if (!providerStatusSuggestsContextOverflow(status, headers)) return;
    await checkpointDiscontinuity("context_overflow", { active_object_refs: [`provider_status:${status}`] });
  });

  // ── agent_end (§29 WBM catalogue + signals — single handler) ──────────────
  pi.on("agent_end", async (event, ctx) => {
    // §29: WBM outbound cataloguing
    if (S.wbmEnabled && !S.wbmNoCatalogue) {
      const messages = (event as any).messages || [];
      catalogueFromMessages(messages).catch(() => {});
    }

    // Long session detection
    const elapsed = (Date.now() - S.sessionStartTime) / 60_000;
    if (elapsed > 45 && !S.longSessionSignaled) {
      S.longSessionSignaled = true;
      if (S.focusaAvailable) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "long_session", surface: "pi",
          payload: { minutes: Math.round(elapsed), turns: S.turnCount },
        });
      }
    }

    // Tool error rate detection
    const recentErrors = S.compilationErrors.filter(t => Date.now() - t < 300_000);
    if (recentErrors.length >= 3) {
      ctx.ui.notify(`⚠️ ${recentErrors.length} compilation errors in 5 min — consider a different approach`, "warning");
      if (S.focusaAvailable) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "error_rate_high", surface: "pi",
          payload: { count: recentErrors.length, window_ms: 300_000 },
        });
      }
    }
  });

  // ── tool_result (§36.2 errors + §33.3 ECS REPLACE + §7.4 thresholds + §34.2D churn) ─
  pi.on("tool_result", async (event, _ctx) => {
    const ev = event as any;
    const toolName = ev.toolName || ev.name || "";
    const content = extractText(ev.content);
    const isError = ev.isError || /error|failed|ENOENT|EPERM/i.test(content.slice(0, 200));
    const cfg = S.cfg;

    // §36.2: Error signals
    if (isError && S.focusaAvailable) {
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "tool_error", surface: "pi",
        payload: { tool: toolName, error: content.slice(0, 500) },
      });
    }

    if (isError && /compil|tsc|typecheck|build|lint/i.test(toolName + " " + content.slice(0, 200))) {
      S.compilationErrors.push(Date.now());
    }

    if (S.focusaAvailable) {
      const targetRefs = [ev.params?.path, ev.input?.path, ev.params?.url, ev.input?.url]
        .map((value: any) => String(value || "").trim())
        .filter(Boolean)
        .slice(0, 8);
      focusaPost("/ontology/tool-result-proposals", {
        tool_name: toolName || "unknown_tool",
        status: isError ? "failed" : "completed",
        ok: !isError,
        target_refs: targetRefs,
        evidence_refs: [],
        workpoint_id: S.activeWorkpointPacket?.workpoint_id || S.activeWorkpointPacket?.workpoint?.workpoint_id || null,
        action_intent: S.currentAsk?.text || null,
        summary: content.slice(0, 500),
        error: isError ? content.slice(0, 500) : null,
        emit_proposals: false,
      });
    }

    // §7.4 + §33.3: ECS externalization — check BOTH thresholds, REPLACE content
    const byteThreshold = cfg?.externalizeThresholdBytes || 8192;
    const tokenThreshold = cfg?.externalizeThresholdTokens || 800;
    const tokens = estimateTokens(content);
    if ((content.length > byteThreshold || tokens > tokenThreshold) && S.focusaAvailable) {
      const handle = await focusaFetch("/ecs/store", {
        method: "POST",
        body: JSON.stringify({
          kind: "text", label: `${toolName}-output-${Date.now()}`,
          content: content.slice(0, 32_000), surface: "pi", turn_id: `pi-turn-${S.turnCount}`,
        }),
      });
      if (handle?.id) {
        // §33.3: REPLACE content with handle reference
        // §7.4: Also cache locally so handles resolve even if Focusa is temporarily down
        storeEcsArtifact("text", content);
        return {
          content: [{
            type: "text",
            text: `[HANDLE:text:${handle.id} "${toolName} output" (${content.length} bytes, ~${tokens} tokens)]\nUse /focusa-rehydrate ${handle.id} to retrieve full content.\n\n` +
                  content.slice(0, 1000) + (content.length > 1000 ? "\n...[truncated, full content in ECS]" : ""),
          }],
        };
      }
    }

    // §7.4 + §33.3: If Focusa unavailable but content still exceeds threshold,
    // store locally so the handle resolves without hitting Focusa.
    if (!S.focusaAvailable && (content.length > byteThreshold || tokens > tokenThreshold)) {
      const localId = storeEcsArtifact("text", content);
      return {
        content: [{
          type: "text",
          text: `[HANDLE:text:${localId} "${toolName} output" (${content.length} bytes, ~${tokens} tokens)]\nFocusa offline — content cached locally. Use /focusa-rehydrate ${localId} when available.\n\n` +
                content.slice(0, 500) + (content.length > 500 ? "\n...[truncated]" : ""),
        }],
      };
    }

    // §34.2D: File churn tracking
    if (toolName === "edit" || toolName === "write") {
      const path = ev.params?.path || ev.input?.path || "";
      if (path) {
        S.fileEditCounts[path] = (S.fileEditCounts[path] || 0) + 1;
        if (S.fileEditCounts[path] >= 5 && S.focusaAvailable) {
          focusaPost("/focus-gate/ingest-signal", {
            signal_type: "file_churn", surface: "pi",
            payload: { path, count: S.fileEditCounts[path] },
          });
        }
      }
    }
  });

  // ── tool_call (§33.4 batched usage) ───────────────────────────────────────
  pi.on("tool_call", async (event, _ctx) => {
    S.toolUsageBatch.push((event as any).toolName || (event as any).name || "");
  });
}
