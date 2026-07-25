// Turn lifecycle + per-call event handlers — ONE handler per event type
// Spec: §7.1 (10 ASCC slots), §7.4 (ECS thresholds), §33.2 (context), §33.3 (ECS replace),
//        §33.4 (tool usage), §34.2B (turns), §35.2 (behavioral), §35.5 (tokens),
//        §35.7 (correction), §36.1 (streaming), §36.2 (error signals), §36.3 (input),
//        §36.6 (injection layering), §36.7 (budget), §37.3 (widget), §37.8 (model),
//        §30 (metacognitive indicators)

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import fs from "node:fs";
import path from "node:path";
import type { PiGoverningPriorKind } from "./state.js";
import {
  getAttachmentRuntime,
  nativeSessionAllowsNonessentialPersistence,
  focusaFetch,
  focusaPost,
  compatibleWorkLoopStatusState,
  extractText,
  getFocusState,
  getCachedFocusState,
  getEffectiveFocusSnapshot,
  estimateTokens,
  wbExec,
  storeEcsArtifact,
  classifyCurrentAsk,
  deriveQueryScope,
  isOperatorSteeringInput,
  selectRelevantItems,
  selectRelevantRankedItems,
  shouldIncludeMissionContext,
  buildSliceSection,
  selectionRelevanceScore,
  retentionBucketsFromSelection,
  formatWorkingSetItems,
  formatVerifiedDeltaItems,
  buildCanonicalReferenceAliases,
  orderSliceSections,
  rescopePiFrameFromCurrentAsk,
  stripQuotedFocusaContext,
  detectForbiddenVisibleOutputLeakClasses,
  detectScopeFailureSignals,
  getSemanticMemorySummary,
  getCachedSemanticMemorySummary,
  getEcsHandlesSummary,
  getCachedEcsHandlesSummary,
  getScopedWorkpointPacket,
  ensureContinuityId,
  isProjectRootAuthoritySafe,
  isWorkpointPacketScopedToCurrentSession,
  refreshTrajectoryClarityLifecycle,
  stampWorkpointPacketForCurrentPiSession,
  adoptPiProjectRoot,
  persistState,
  projectRootConfirmationRequired,
  projectRootConfirmationSummary,
  buildAttentionRecallVerdict,
  formatAttentionRecallFocusSliceLines,
  maybeCaptureReportSummaryFromAssistantOutput,
  recordToolOutputPressure,
  toolOutputVisibleRecapReason,
  formatToolOutputVisibleRecapLines,
  markVisibleRecapEmittedIfPresent,
  observeProjectThreadHintsFromText,
  formatProjectSwitchLedgerLines,
  buildCurrentAskScopeVerdict,
  formatCurrentAskScopeVerdictLines,
  getActiveWorkpointPacket,
  setActiveWorkpointPacket,
  getActiveWorkpointSummary,
  setActiveWorkpointSummary,
  getLastTrajectoryClarity,
  setLastTrajectoryClarity,
  getLastProjectVerify,
  getLatestReportSummary,
  setLastStreamLen,
  resetToolUsageBatch,
  getToolUsageBatch,
  pushToToolUsageBatch,
  setLongSessionSignaled,
  getLongSessionSignaled,
  getCurrentTaskTurnStart,
  setCurrentTaskTurnStart,
  incrementTotalCompactions,
  getLastStreamLen,
  pushCompilationError,
  getCompilationErrors,
  incrementFileEditCount,
  getFileEditCounts,
  getTurnCount,
  setTurnCount,
  incrementTurnCount,
  getSessionCwd,
  getContinuityId,
  pushRecentTurn,
  getRecentTurns,
  formatRecentTurnsSection,
  shouldEmitRecentTurnsSlice,
  markRecentTurnsSliceEmitted,
  type RecentTurnSlice,
} from "./state.js";
import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";
import { checkCompactionTier, checkMicroCompact, contextTierLabel } from "./compaction.js";
import { catalogueFromMessages } from "./wbm.js";
import { pushDelta } from "./tools.js";
import { buildFocusaUtilityCard } from "./awareness.js";
import { buildAwarenessPacket, renderAwarenessPacketText } from "./awareness-substrate.js";
import {
  attachFocusSliceToNewestUser,
  CacheSafetyMonitor,
  normalizeCacheUsage,
  type CacheSafetyObservation,
} from "./cache-safe-context.js";
import { selectFocusSliceToolAffordances } from "./tool-contracts.js";

const cacheSafetyMonitor = new CacheSafetyMonitor();
const CACHE_SAFE_DEGRADED_RETAINED_SECTIONS = new Set([
  "current_ask",
  "current_ask_scope_verdict",
  "project_switch_ledger",
  "trajectory",
  "workpoint",
  "constraints",
  "ontology_next_actions",
  "ontology_blocked_affordances",
  "ontology_evidence_handles",
  "tool_affordances",
]);

// ─── §5.12 Recent-turns adapter (Pi implementation) ──────────────────────
// Spec: docs/101-focusa-bloatgaurd-spec.md §5.12 + §5.12.11 (adapter contract)
// Pi is Adapter 1; the daemon owns the canonical ring buffer. Other agents
// (Claude Code, Aider, Cursor, Cline, Gemini, Codex) implement the same contract.

async function turnWorkLoopWriterHeaders(): Promise<Record<string, string>> {
  const writerId = `pi-${process.pid}`;
  const status = await focusaFetch("/work-loop/status?summary_only=true");
  const partition = status?.execution_partition;
  const token = Number(partition?.fencing_token);
  const expiresAt = Date.parse(String(partition?.lease_expires_at || ""));
  if (
    compatibleWorkLoopStatusState(status) === "unsupported" ||
    partition?.writer_key !== writerId ||
    partition?.lease_freshness !== "current" ||
    !Number.isSafeInteger(token) ||
    token <= 0 ||
    !(expiresAt > Date.now())
  ) {
    throw new Error("current scoped Work Loop lease is missing, expired, or owned by another Pi writer");
  }
  return {
    "x-focusa-writer-id": writerId,
    "x-focusa-fencing-token": String(token),
  };
}

const ADAPTER_KIND = "pi-extension";

function shouldIncludeTurnInSlice(text: string, toolCallCount: number): boolean {
  // Drop status-only turns (test/cont/ack/...) and tool-empty turns when non-task text exists.
  if (isNonTaskStatusLikeText(text || "")) return false;
  if (toolCallCount === 0 && (!text || text.trim().length === 0)) return false;
  return true;
}

function deriveOutcome(text: string, hasFailureSignal: boolean): string {
  if (hasFailureSignal) return "blocked";
  if (!text || text.trim().length === 0) return "observed";
  return "tooled";
}

function captureRecentTurnSlice(assistantOutput: string): void {
  try {
    const turnCount = getTurnCount();
    const toolCount = (getAttachmentRuntime() as any).currentTaskToolCalls ?? 0;
    const text = assistantOutput || "";
    if (!shouldIncludeTurnInSlice(text, toolCount)) return;
    const slice: RecentTurnSlice = {
      turn_id: `pi-turn-${turnCount}`,
      mission_at_turn: (
        getAttachmentRuntime().activeFrameGoal ||
        getAttachmentRuntime().activeFrameTitle ||
        ""
      ).slice(0, 120),
      outcome: deriveOutcome(text, false) as RecentTurnSlice["outcome"],
      evidence_refs: [] as string[],
      tool_call_count: toolCount,
      emitted_at: Math.floor(Date.now() / 1000),
    };
    pushRecentTurn(slice);
    // Best-effort POST to daemon; focusaPost swallows failures internally.
    if (getAttachmentRuntime().focusaAvailable) {
      focusaPost("/v1/turns/recent", {
        turn_id: slice.turn_id,
        continuity_id: getContinuityId(),
        mission_at_turn: slice.mission_at_turn,
        outcome: slice.outcome,
        evidence_refs: slice.evidence_refs,
        tool_call_count: slice.tool_call_count,
        emitted_at: slice.emitted_at,
      });
    }
  } catch {
    // never throw from capture
  }
}

async function fetchRecentTurnsFromDaemon(n: number): Promise<RecentTurnSlice[]> {
  if (!getAttachmentRuntime().focusaAvailable) return [];
  try {
    const continuity = getContinuityId();
    if (!continuity) return [];
    const res: any = await focusaFetch(
      `/v1/turns/recent?n=${n}&continuity_id=${encodeURIComponent(continuity)}`
    );
    const value = res?.value;
    if (!value || Array.isArray(value?.error)) return [];
    const turns = Array.isArray(value?.turns) ? value.turns : [];
    return turns.map((t: any) => ({
      turn_id: String(t.turn_id || "?"),
      mission_at_turn: String(t.mission_at_turn || ""),
      outcome: String(t.outcome || "tooled") as RecentTurnSlice["outcome"],
      evidence_refs: Array.isArray(t.evidence_refs) ? t.evidence_refs : [],
      tool_call_count: Number(t.tool_call_count || 0),
      emitted_at: Number(t.emitted_at || 0),
    }));
  } catch {
    return [];
  }
}

async function buildRecentTurnsSlice(n: number = 4): Promise<string> {
  const currentTurn = getTurnCount();
  if (!shouldEmitRecentTurnsSlice(currentTurn)) return "";
  const daemonTurns = await fetchRecentTurnsFromDaemon(n);
  if (daemonTurns.length === 0) {
    const localSection = formatRecentTurnsSection(n);
    markRecentTurnsSliceEmitted(currentTurn);
    return localSection || "";
  }
  const lines = [`Recent turns (last ${daemonTurns.length}, daemon source):`];
  for (const t of daemonTurns) {
    const mission =
      t.mission_at_turn.length > 120 ? t.mission_at_turn.slice(0, 119) + "\u2026" : t.mission_at_turn;
    const refs = t.evidence_refs.length > 0 ? ` ev=${t.evidence_refs.join(",")}` : "";
    lines.push(
      `- T[${t.turn_id}] mission="${mission}" outcome=${t.outcome} tools=${t.tool_call_count}${refs}`
    );
  }
  markRecentTurnsSliceEmitted(currentTurn);
  return lines.join("\n");
}

function buildCachedRecentTurnsSlice(n: number = 4): string {
  const currentTurn = getTurnCount();
  if (!shouldEmitRecentTurnsSlice(currentTurn)) return "";
  const localSection = formatRecentTurnsSection(n);
  markRecentTurnsSliceEmitted(currentTurn);
  return localSection || "";
}

// Recall-intent detection (mirrors §5.12.10 word set).
const RECALL_INTENT_PATTERNS: Array<{ category: string; pattern: RegExp }> = [
  {
    category: "direct_recall",
    pattern: /\b(recall|remember|remind me|bring me back|catch up|orient me|refocus|rewind)\b/i,
  },
  {
    category: "implicit_prior",
    pattern:
      /\b(what did we|earlier|last time|previously|where were we|as we discussed|we talked about|you mentioned|you said|i asked|i said|i meant|didn't we)\b/i,
  },
  {
    category: "coherence_loss",
    pattern: /\bcontext\b.{0,40}\?|lost|confused|on track|where (were|are) we going|what's the state/i,
  },
  {
    category: "repetition",
    pattern: /\balready covered|already done|already filed|duplicate|going in circles/i,
  },
  { category: "operator_steering", pattern: /^wait$|^hold on$|^back up$|^scratch that$/i },
];

function detectRecallIntent(text: string): { matched_category: string; matched_phrase: string } | null {
  if (!text || text.length > 240) return null;
  for (const { category, pattern } of RECALL_INTENT_PATTERNS) {
    const m = text.match(pattern);
    if (m) return { matched_category: category, matched_phrase: m[0] };
  }
  return null;
}

function isNonTaskStatusLikeTextLocal(text: string): boolean {
  // Delegate to the canonical helper imported in the wider scope if available; otherwise re-implement narrowly.
  try {
    return (require("./state.js") as any).isNonTaskStatusLikeText?.(text) ?? false;
  } catch {
    return /^(test|cont|ack|ok|got it|continue|next|k|yes|no|y|n|\.+)\b/i.test(text.trim());
  }
}

// Expose isNonTaskStatusLikeText to captureRecentTurnSlice scope.
function isNonTaskStatusLikeText(text: string): boolean {
  return isNonTaskStatusLikeTextLocal(text);
}

const traceBatch: any[] = [];

function vitalPromptSurfaceEnabled(surface: string): boolean {
  const raw = String(
    getAttachmentRuntime().cfg?.vitalInfoPromptSurfaces || "project_root,project_verify,workpoint,trajectory"
  );
  return raw
    .split(",")
    .map((part) => part.trim())
    .includes(surface);
}

function hardGateVitalProjectRoot(ctx: any): string | null {
  if (!getAttachmentRuntime().focusaAvailable || !vitalPromptSurfaceEnabled("project_root")) return null;
  const detected = adoptPiProjectRoot(ctx.cwd || getSessionCwd() || process.cwd());
  if (!projectRootConfirmationRequired(detected)) {
    ctx.ui.setWidget("focusa-vital", undefined);
    return detected;
  }
  const summary = projectRootConfirmationSummary(detected);
  const mode = getAttachmentRuntime().cfg?.vitalInfoPromptMode || "prompt";
  focusaPost("/telemetry/trace", {
    event_type: "pi_vital_project_root_before_agent_inference_required",
    payload: { project_root: detected, summary, mode, session_id: getAttachmentRuntime().sessionFrameKey },
  });
  return null;
}

function queueTraceTelemetry(event: Record<string, any>): void {
  traceBatch.push(event);
}

function flushTraceTelemetryBatch(reason = "turn_end"): void {
  const totalQueued = traceBatch.length;
  const events = traceBatch.splice(0, 100);
  if (!events.length) return;
  focusaPost("/telemetry/trace/batch", {
    batch_id: `pi-trace-batch-${getTurnCount()}-${Date.now()}`,
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
  if (!getAttachmentRuntime().focusaAvailable) return;
  const root = getSessionCwd() || process.cwd();
  if (!isProjectRootAuthoritySafe(root)) return;
  try {
    await focusaFetch("/workpoint/checkpoint", {
      method: "POST",
      body: JSON.stringify({
        mission:
          getAttachmentRuntime().currentAsk?.text ||
          getAttachmentRuntime().activeFrameGoal ||
          getAttachmentRuntime().lastFocusSnapshot.intent ||
          "Pi discontinuity boundary",
        next_slice:
          getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
          "Resume from typed Workpoint after discontinuity.",
        checkpoint_reason: reason,
        canonical: true,
        promote: true,
        continuity_id: ensureContinuityId(root),
        session_id: getAttachmentRuntime().sessionFrameKey,
        project_root: root,
        source_turn_id: `pi-turn-${getTurnCount()}`,
        action_intent: {
          action_type: "resume_workpoint",
          target_ref: getAttachmentRuntime().activeFrameId || "pi-session",
          verification_hooks: [reason],
          status: "ready",
        },
        ...extra,
      }),
    });
    const packet = await focusaFetch("/workpoint/resume", {
      method: "POST",
      body: JSON.stringify({
        mode: "compact_prompt",
        continuity_id: ensureContinuityId(root),
        session_id: getAttachmentRuntime().sessionFrameKey,
        project_root: root,
      }),
    });
    if (packet?.status === "rejected_scope_mismatch") {
      setActiveWorkpointPacket(null);
      setActiveWorkpointSummary("");
      return;
    }
    if (packet?.status === "completed") {
      const candidate = packet.resume_packet || packet;
      if (!isWorkpointPacketScopedToCurrentSession(candidate)) {
        setActiveWorkpointPacket(null);
        setActiveWorkpointSummary("");
        return;
      }
      setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(candidate));
      setActiveWorkpointSummary(packet.rendered_summary || packet.next_step_hint || "");
      getAttachmentRuntime().lastWorkpointUpdate = Date.now();
    }
  } catch {
    /* best effort */
  }
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
  const activeObjects =
    Array.isArray(packet.active_object_refs) && packet.active_object_refs.length
      ? packet.active_object_refs
      : ["(none)"];
  const verificationHooks = [
    ...(Array.isArray(action.verification_hooks) ? action.verification_hooks : []),
    ...evidence.map((v: any) => v.result || v.evidence_ref || v.target_ref).filter(Boolean),
  ].slice(0, 8);
  const boundaryItems = driftBoundaries.length
    ? driftBoundaries
    : blockers
          .map((b: any) => b.reason)
          .filter(Boolean)
          .slice(0, 6).length
      ? blockers
          .map((b: any) => b.reason)
          .filter(Boolean)
          .slice(0, 6)
      : ["Do not override WorkpointResumePacket from transcript tail."];
  return [
    `WORKPOINT: ${getActiveWorkpointSummary() || packet.next_slice || packet.mission || "active typed packet present"}`,
    `WORKPOINT_CANONICAL: ${packet.canonical !== false}`,
    `ACTIVE_OBJECT_SET:\n${activeObjects.map((x: string) => `  - ${x}`).join("\n")}`,
    `ACTION_INTENT: ${action.action_type || "unknown"}${action.target_ref ? ` -> ${action.target_ref}` : ""}`,
    `VERIFICATION_HOOKS:\n${(verificationHooks.length ? verificationHooks : ["(none)"]).map((x: string) => `  - ${x}`).join("\n")}`,
    `DRIFT_BOUNDARIES:\n${boundaryItems.map((x: string) => `  - ${x}`).join("\n")}`,
  ];
}

function boundedTrajectoryText(value: any, max = 180): string {
  const text = String(value ?? "")
    .replace(/\s+/g, " ")
    .trim();
  if (!text) return "";
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

function formatHandleTrajectorySummary(handle: any): string {
  const trajectory = handle?.trajectory || {};
  const parts = [
    trajectory.trajectory_id ? `id=${boundedTrajectoryText(trajectory.trajectory_id, 80)}` : "",
    trajectory.hlt ? `HLT=${boundedTrajectoryText(trajectory.hlt, 140)}` : "",
    trajectory.mlg ? `MLG=${boundedTrajectoryText(trajectory.mlg, 120)}` : "",
    trajectory.stg ? `STG=${boundedTrajectoryText(trajectory.stg, 120)}` : "",
    Array.isArray(trajectory.waypoints) && trajectory.waypoints.length
      ? `waypoints=${trajectory.waypoints
          .slice(0, 3)
          .map((item: any) => boundedTrajectoryText(item, 80))
          .join(" | ")}`
      : "",
  ].filter(Boolean);
  return parts.length ? `TRAJECTORY_CONTEXT: ${parts.join("; ")}\n` : "";
}

function safeExists(root: string, rel: string): boolean {
  try {
    return fs.existsSync(path.join(root, rel));
  } catch {
    return false;
  }
}

function existingProjectDirs(root: string, dirs: string[]): string[] {
  return dirs.filter((dir) => safeExists(root, dir)).slice(0, 8);
}

function safeRead(root: string, rel: string, maxBytes = 4096): string {
  try {
    const file = path.join(root, rel);
    if (!fs.existsSync(file) || !fs.statSync(file).isFile()) return "";
    return fs.readFileSync(file, "utf8").slice(0, maxBytes);
  } catch {
    return "";
  }
}

function manifestName(root: string): string {
  const packageJson = safeRead(root, "package.json");
  if (packageJson) {
    try {
      return `package=${JSON.parse(packageJson).name || "unknown"}`;
    } catch {
      return "package=unparsed";
    }
  }
  const cargo = safeRead(root, "Cargo.toml");
  const cargoName = cargo.match(/^name\s*=\s*["']([^"']+)["']/m)?.[1];
  if (cargoName) return `cargo=${cargoName}`;
  return "manifest=unknown";
}

function buildProjectArchitectureDigestLine(root: string): string {
  const evidenceRefs: string[] = [];
  const mark = (rel: string, label = rel): boolean => {
    const exists = safeExists(root, rel);
    if (exists) evidenceRefs.push(label);
    return exists;
  };
  const cargo = mark("Cargo.toml");
  const packageJson = mark("package.json");
  const stacks = [
    cargo ? "rust" : "",
    packageJson || mark("pnpm-lock.yaml") || mark("bun.lockb") ? "node" : "",
    mark("go.mod") ? "go" : "",
    mark("pyproject.toml") || mark("requirements.txt") ? "python" : "",
    mark("composer.json") ? "php" : "",
  ].filter(Boolean);
  const keyDirs = existingProjectDirs(root, [
    "crates",
    "apps",
    "packages",
    "src",
    "docs",
    "tests",
    ".github",
    ".beads",
    "data",
  ]);
  keyDirs.forEach((dir) => {
    if (!evidenceRefs.includes(dir)) evidenceRefs.push(dir);
  });
  const deploy = [
    mark("Dockerfile") ? "Dockerfile" : "",
    mark("docker-compose.yml") || mark("compose.yml") ? "compose" : "",
    mark(".github/workflows") ? "github_actions" : "",
    mark("systemd") || mark(".service") ? "systemd" : "",
  ].filter(Boolean);
  const docs = existingProjectDirs(root, ["docs", "README.md", "AGENTS.md", ".focusa-project.json"]);
  const tests = existingProjectDirs(root, ["tests", "test", "spec", "crates", "apps/pi-extension"]);
  const confidence =
    stacks.length && docs.length && tests.length
      ? "high"
      : stacks.length && (docs.length || tests.length)
        ? "medium"
        : "low";
  return [
    `stack=${stacks.join("+") || "unknown"}`,
    manifestName(root),
    `key_dirs=${keyDirs.join(",") || "unknown"}`,
    `deploy=${deploy.join(",") || "unknown"}`,
    `docs=${docs.join(",") || "unknown"}`,
    `tests=${tests.join(",") || "unknown"}`,
    `confidence=${confidence}`,
    `evidence_refs=${evidenceRefs.slice(0, 10).join(",") || "none"}`,
    "verify_architecture_with=focusa_traverse+repo_docs+evidence",
  ].join("; ");
}

function formatTrajectoryFallbackFocusSlice(root: string, reason: string): string[] {
  const safe = isProjectRootAuthoritySafe(root);
  const displayRoot = boundedTrajectoryText(root || "(unknown)", 160);
  const continuityId = boundedTrajectoryText(getContinuityId(), 120);
  return [
    `PROJECT_IDENTITY: status=${safe ? "local_fallback" : "unsafe_scope"} project_root=${displayRoot} ${continuityId ? `continuity_id=${continuityId}` : "continuity_id=(unavailable)"}`,
    safe
      ? "PROJECT_INFRA: architecture_boundary=use project docs/ontology/evidence; do not infer from folder name alone"
      : "PROJECT_INFRA: withheld_until_safe_project_root; call focusa_project_identity with explicit project_root",
    safe
      ? "PROJECT_ENVIRONMENT: root_url=unknown; live_url=unknown; local_url=unknown; environment=unknown; deploy_target=unknown; deploy_location=unknown; source=missing_project_marker_or_trajectory_view"
      : "PROJECT_ENVIRONMENT: withheld_until_safe_project_root",
    safe
      ? `PROJECT_ARCHITECTURE: ${buildProjectArchitectureDigestLine(root)}`
      : "PROJECT_ARCHITECTURE: withheld_until_safe_project_root",
    `TRAJECTORY_GOALS: unavailable reason=${boundedTrajectoryText(reason, 80)}; call focusa_trajectory_view after safe scope`,
    "TRAJECTORY_SIMILARITY_GROUP: advisory_only=true; authority=project_root+continuity_id; must_not_merge_sessions=true",
    "CURRENT_VERIFIED_STATE: unclear",
    "ACTIVE_GAP: recover scoped trajectory/workpoint before trusting carryover",
    "WORKPOINT_CANDIDATE: none · advisory_only=true",
    "TRAJECTORY_EVIDENCE: (none)",
    "TRAJECTORY_DO_NOT_USE: transcript tail as authority; broad cwd as project identity",
    "CONTEXT_SUFFICIENCY: score=0 status=degraded missing=scoped_frame, trajectory_view recommended=focusa_project_identity -> focusa_trajectory_view -> focusa_workpoint_resume",
  ];
}

function formatTrajectoryFocusSlice(view: any): string[] {
  if (!view || typeof view !== "object") return [];
  const project = view.project_identity || {};
  const trajectory = view.trajectory || {};
  const intelligence = view.intelligence_view || {};
  const sufficiency = intelligence.context_sufficiency || {};
  const candidate = intelligence.next_workpoint_candidate || {};
  const projectApi = project.project_identity_api || {};
  const continuityId = boundedTrajectoryText(getContinuityId(), 120);
  const projectRoot = boundedTrajectoryText(
    project.project_root || projectApi.project_root || getSessionCwd() || process.cwd(),
    160
  );
  const canonicalName = boundedTrajectoryText(project.canonical_name || projectApi.canonical_name, 80);
  const projectId = boundedTrajectoryText(project.project_id || projectApi.project_id, 80);
  const workspaceKind = boundedTrajectoryText(project.workspace_kind || projectApi.workspace_kind, 80);
  const repoRemote = boundedTrajectoryText(project.repo_remote || projectApi.repo_remote, 140);
  const workingContext = project.working_context || projectApi.working_context || {};
  const workingSubpath = workingContext.working_subpath || {};
  const activeWorktreeRoot = boundedTrajectoryText(workingContext.active_worktree_root, 160);
  const workingSubpathId = boundedTrajectoryText(workingSubpath.working_subpath_id || "primary", 80);
  const beadsRoot = boundedTrajectoryText(workingSubpath.beads_root, 160);
  const beadsPrefix = boundedTrajectoryText(
    workingSubpath.beads_prefix || project.beads_prefix || projectApi.beads_prefix,
    40
  );
  const projectUrls = project.project_urls || projectApi.project_urls || {};
  const deployment = project.deployment || projectApi.deployment || {};
  const rootUrl = boundedTrajectoryText(
    projectUrls.root_url || projectUrls.live_url || projectUrls.production_url || deployment.root_url,
    140
  );
  const liveUrl = boundedTrajectoryText(
    projectUrls.live_url || projectUrls.production_url || deployment.live_url,
    140
  );
  const wpUrl = boundedTrajectoryText(
    projectUrls.wp_url || projectUrls.wordpress_url || projectUrls.site_url,
    140
  );
  const appUrl = boundedTrajectoryText(projectUrls.app_url || deployment.app_url, 140);
  const authUrl = boundedTrajectoryText(projectUrls.auth_url || deployment.auth_url, 140);
  const graphqlUrl = boundedTrajectoryText(projectUrls.graphql_url || deployment.graphql_url, 140);
  const localUrl = boundedTrajectoryText(projectUrls.local_url || deployment.local_url, 140);
  const deployEnvironment = boundedTrajectoryText(
    deployment.environment || deployment.deploy_environment || deployment.target_environment,
    80
  );
  const deployTarget = boundedTrajectoryText(
    deployment.deploy_target || deployment.target || deployment.host,
    120
  );
  const deployLocation = boundedTrajectoryText(
    deployment.deploy_location || deployment.path || deployment.document_root,
    160
  );
  const deployCommand = boundedTrajectoryText(deployment.deploy_command || deployment.command, 160);
  const environmentConfidence = boundedTrajectoryText(
    projectUrls.inference_confidence || deployment.inference_confidence,
    40
  );
  const identityParts = [
    `status=${boundedTrajectoryText(project.status || view.status || "unknown", 40)}`,
    `canonical_parent=${projectRoot}`,
    activeWorktreeRoot ? `working_root=${activeWorktreeRoot}` : `working_root=${projectRoot}`,
    `working_subpath=${workingSubpathId}`,
    continuityId ? `continuity_id=${continuityId}` : "continuity_id=(unavailable)",
    project.session_id ? `session_id=${boundedTrajectoryText(project.session_id, 120)}` : "",
    project.confidence ? `confidence=${boundedTrajectoryText(project.confidence, 40)}` : "",
  ].filter(Boolean);
  const infraParts = [
    canonicalName ? `name=${canonicalName}` : "",
    projectId ? `project_id=${projectId}` : "",
    workspaceKind ? `workspace_kind=${workspaceKind}` : "",
    repoRemote ? `repo=${repoRemote}` : "",
    beadsRoot ? `beads_root=${beadsRoot}` : "",
    beadsPrefix ? `beads_prefix=${beadsPrefix}` : "",
    "architecture_boundary=use project docs/ontology/evidence; do not infer from folder name alone",
  ].filter(Boolean);
  const environmentBits = [
    rootUrl ? `root_url=${rootUrl}` : "root_url=unknown",
    liveUrl ? `live_url=${liveUrl}` : "live_url=unknown",
    wpUrl ? `wp_url=${wpUrl}` : "",
    appUrl ? `app_url=${appUrl}` : "",
    authUrl ? `auth_url=${authUrl}` : "",
    graphqlUrl ? `graphql_url=${graphqlUrl}` : "",
    localUrl ? `local_url=${localUrl}` : "local_url=unknown",
    deployEnvironment ? `environment=${deployEnvironment}` : "environment=unknown",
    deployTarget ? `deploy_target=${deployTarget}` : "deploy_target=unknown",
    deployLocation ? `deploy_location=${deployLocation}` : "deploy_location=unknown",
    deployCommand ? `deploy_command=${deployCommand}` : "",
    environmentConfidence ? `confidence=${environmentConfidence}` : "confidence=unknown",
    "source=marker+bounded_repo_scan+optional_live_root_scan",
    "local_vs_live_boundary=project-root-relative by default, but DNS/live roots may live outside repo; verify sources before assuming .local is active",
  ]
    .filter(Boolean)
    .join("; ");
  const goals = [
    trajectory.long_term_goal ? `HLT=${boundedTrajectoryText(trajectory.long_term_goal, 180)}` : "",
    trajectory.mid_level_goal ? `MLG=${boundedTrajectoryText(trajectory.mid_level_goal, 160)}` : "",
    trajectory.short_term_goal ? `STG=${boundedTrajectoryText(trajectory.short_term_goal, 160)}` : "",
    trajectory.low_level_goal ? `low=${boundedTrajectoryText(trajectory.low_level_goal, 160)}` : "",
    trajectory.desired_end_state ? `desired=${boundedTrajectoryText(trajectory.desired_end_state, 180)}` : "",
  ]
    .filter(Boolean)
    .join("; ");
  const similarityGroup = trajectory.similarity_group || intelligence.similarity_group || {};
  const similarityBits = [
    similarityGroup.high_level_group_key
      ? `high_key=${boundedTrajectoryText(similarityGroup.high_level_group_key, 80)}`
      : "",
    similarityGroup.mid_level_group_key
      ? `mid_key=${boundedTrajectoryText(similarityGroup.mid_level_group_key, 80)}`
      : "",
    similarityGroup.low_level_group_key
      ? `low_key=${boundedTrajectoryText(similarityGroup.low_level_group_key, 80)}`
      : "",
    `advisory_only=${similarityGroup.advisory_only !== false}`,
    "authority=project_root+continuity_id",
    similarityGroup.must_not_merge_sessions ? "must_not_merge_sessions=true" : "",
  ]
    .filter(Boolean)
    .join("; ");
  const waypoints = Array.isArray(trajectory.waypoints)
    ? trajectory.waypoints
        .slice(0, 5)
        .map((item: any) => boundedTrajectoryText(item, 120))
        .filter(Boolean)
    : Array.isArray(trajectory.trajectory_ladder?.waypoints)
      ? trajectory.trajectory_ladder.waypoints
          .slice(0, 5)
          .map((item: any) => boundedTrajectoryText(item, 120))
          .filter(Boolean)
      : [];
  const evidence = Array.isArray(trajectory.evidence_refs)
    ? trajectory.evidence_refs
        .slice(0, 4)
        .map((item: any) => boundedTrajectoryText(item.evidence_ref || item.result || item.target_ref, 120))
        .filter(Boolean)
    : [];
  const doNotUse = Array.isArray(intelligence.do_not_use)
    ? intelligence.do_not_use
        .slice(0, 6)
        .map((item: any) => boundedTrajectoryText(item, 100))
        .filter(Boolean)
    : [];
  const missingFacts = Array.isArray(sufficiency.missing_facts)
    ? sufficiency.missing_facts
        .slice(0, 6)
        .map((item: any) => boundedTrajectoryText(item, 80))
        .filter(Boolean)
    : [];
  const candidateBits = [
    candidate.workpoint_id ? `id=${boundedTrajectoryText(candidate.workpoint_id, 80)}` : "",
    candidate.work_item_id ? `work_item=${boundedTrajectoryText(candidate.work_item_id, 80)}` : "",
    candidate.next_slice ? `next=${boundedTrajectoryText(candidate.next_slice, 180)}` : "",
    "advisory_only=true",
  ]
    .filter(Boolean)
    .join("; ");
  const lines = [
    `PROJECT_IDENTITY: ${identityParts.join(" ")}`,
    infraParts.length
      ? `PROJECT_INFRA: ${infraParts.join("; ")}`
      : "PROJECT_INFRA: unknown; use focusa_project_identity plus focusa_traverse before architectural assumptions",
    `PROJECT_ENVIRONMENT: ${environmentBits}`,
    `PROJECT_ARCHITECTURE: ${isProjectRootAuthoritySafe(projectRoot) ? buildProjectArchitectureDigestLine(projectRoot) : "withheld_until_safe_project_root"}`,
    goals
      ? `TRAJECTORY_LADDER: ${goals}; waypoints=${waypoints.join(" → ") || "derive_next"}; rule=operator_deference_plus_proactive_route_offers`
      : "TRAJECTORY_LADDER: definition_status=unclear; derive HLT→MLG→STG→Waypoints before durable work",
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
  if (!getAttachmentRuntime().focusaAvailable) return [];
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

function currentAskLooksLikeWebResearch(text: string | undefined): boolean {
  const value = String(text || "").toLowerCase();
  return (
    /https?:\/\//.test(value) ||
    /\b(url|website|webpage|browser|browse|research|docs?|article|blog|github\.com)\b/.test(value)
  );
}

function getUiaiFirstFocusSliceLines(askText: string | undefined): string[] {
  if (!currentAskLooksLikeWebResearch(askText)) return [];
  return [
    "UIAI_FIRST_WEB_RESEARCH: required=true",
    "UIAI_FIRST_ROUTE: pi_uiai_agent_card → uiai_health → uiai_browser_open/read or UIAI source/markdown/search",
    "UIAI_FALLBACK_RULE: web_search/fetch_content only after UIAI unavailable/saturated-with-no-closable-session/unsuitable",
    "UIAI_PRESSURE_RULE: close unused UIAI sessions before generic web fallback",
  ];
}

function getToolAffordanceFocusSliceLines(options: {
  resourceModeActive: boolean;
  hasTrajectory: boolean;
  hasWorkpoint: boolean;
  hasOntologyAmbiguity: boolean;
}): string[] {
  const affordances = selectFocusSliceToolAffordances(options);
  const next = affordances.best_next.slice(0, options.hasWorkpoint ? 1 : 2);
  const recovery = affordances.recovery.slice(0, 1);
  return [`FOCUSA_TOOLS: next=${next.join(" | ") || "none"}; recovery=${recovery.join(" | ") || "none"}`];
}

function getCachedTrajectoryFocusSliceLines(): string[] {
  const root = getSessionCwd() || process.cwd();
  if (!isProjectRootAuthoritySafe(root)) return [];
  return formatTrajectoryFallbackFocusSlice(root, "prompt_hot_path_cache");
}

async function getTrajectoryFocusSliceLines(): Promise<string[]> {
  const root = getSessionCwd() || process.cwd();
  if (!isProjectRootAuthoritySafe(root)) return [];
  if (!getAttachmentRuntime().focusaAvailable)
    return formatTrajectoryFallbackFocusSlice(root, "focusa_unavailable");
  try {
    const params = new URLSearchParams();
    params.set("mode", "summary");
    params.set("project_root", root);
    params.set("allow_prior_project_trajectory", "true");
    if (getAttachmentRuntime().sessionFrameKey)
      params.set("session_id", getAttachmentRuntime().sessionFrameKey);
    if (getContinuityId()) params.set("continuity_id", getContinuityId());
    const view = await focusaFetch(`/trajectory/view?${params.toString()}`);
    const lines = formatTrajectoryFocusSlice(view);
    return lines.length ? lines : formatTrajectoryFallbackFocusSlice(root, "empty_trajectory_view");
  } catch {
    return formatTrajectoryFallbackFocusSlice(root, "trajectory_view_unavailable");
  }
}

function providerStatusSuggestsContextOverflow(
  status: number,
  headers: Record<string, string> = {}
): boolean {
  if ([413].includes(status)) return true;
  if (![400, 422].includes(status)) return false;
  const joined = Object.entries(headers)
    .map(([k, v]) => `${k}:${v}`)
    .join(" ")
    .toLowerCase();
  return (
    /context[_ -]?length|token|too large|payload|maximum context|input exceeds/.test(joined) ||
    joined.length === 0
  );
}

function textSuggestsContextOverflow(text: string): boolean {
  return /context_length_exceeded|input exceeds the context|maximum context|prompt too long|too many tokens/i.test(
    text || ""
  );
}

export const BLOATGAURD_RECENT_TOOL_HISTORY_MESSAGES = 12;

export function elideOldRehydratableToolHistory(
  messages: any[],
  recentMessageWindow = BLOATGAURD_RECENT_TOOL_HISTORY_MESSAGES
): any[] {
  const safeWindow = Math.max(0, Math.floor(recentMessageWindow));
  const recentStart = Math.max(0, messages.length - safeWindow);

  return messages.map((message, index) => {
    if (index >= recentStart || message?.role !== "toolResult" || message?.isError) return message;

    const content = message?.content;
    const text =
      typeof content === "string"
        ? content
        : Array.isArray(content)
          ? content
              .filter((item: any) => item?.type === "text")
              .map((item: any) => item.text || "")
              .join("\n")
          : "";
    const handle = text.match(/^\[HANDLE:([^\n\]]+)\]/m);
    const identity = handle?.[1]?.split(/\s+/, 1)[0] || "";
    const separator = identity.indexOf(":");
    const handleId = separator >= 0 ? identity.slice(separator + 1) : "";
    if (!handle || !handleId) return message;

    const replacement = `${handle[0]}\nUse /focusa-rehydrate ${handleId} to retrieve full content.`;
    if (text.trim() === replacement) return message;
    return {
      ...message,
      content: typeof content === "string" ? replacement : [{ type: "text", text: replacement }],
    };
  });
}

function cacheSessionKey(): string {
  return getAttachmentRuntime().sessionFrameKey || getContinuityId() || "no-session";
}

function attachCacheSafeFocusSlice(_event: any, messages: any[], text: string): any[] {
  const cacheSafeLayoutEnabled = getAttachmentRuntime().cfg?.cacheSafePromptLayoutEnabled !== false;
  const snapshot = cacheSafetyMonitor.captureRequest(cacheSessionKey(), messages, text);
  queueTraceTelemetry({
    event_type: "prompt_cache_prefix_snapshot",
    turn_id: `pi-turn-${getTurnCount()}`,
    surface: "pi",
    session_cache_key_hash: snapshot.sessionCacheKeyHash,
    stable_system_prefix_hash: snapshot.stableSystemPrefixHash,
    history_prefix_hash: snapshot.historyPrefixHash,
    dynamic_slice_hash: snapshot.dynamicSliceHash,
    dynamic_slice_estimated_tokens: snapshot.dynamicSliceEstimatedTokens,
    cache_safe_degraded: cacheSafeLayoutEnabled && cacheSafetyMonitor.isDegraded(cacheSessionKey()),
    cache_safe_prompt_layout_enabled: cacheSafeLayoutEnabled,
    injection_position: cacheSafeLayoutEnabled ? "newest_user_turn_tail" : "legacy_history_prepend",
    historical_message_count: snapshot.historyMessageHashes.length,
  });
  return cacheSafeLayoutEnabled
    ? attachFocusSliceToNewestUser(messages, text)
    : [{ role: "user" as const, content: [{ type: "text" as const, text }] }, ...messages];
}

function emitCacheSafetyObservation(observation: CacheSafetyObservation, ctx: any): void {
  queueTraceTelemetry({
    event_type: "prompt_cache_observation",
    turn_id: `pi-turn-${getTurnCount()}`,
    surface: "pi",
    miss: observation.miss,
    miss_reason: observation.reason,
    provider: observation.provider,
    model: observation.model,
    session_cache_key_hash: observation.sessionCacheKeyHash,
    stable_system_prefix_hash: observation.stableSystemPrefixHash,
    history_prefix_hash: observation.historyPrefixHash,
    dynamic_slice_hash: observation.dynamicSliceHash,
    dynamic_slice_tokens: observation.dynamicSliceEstimatedTokens,
    input_tokens: observation.inputTokens,
    cache_read_tokens: observation.cacheReadTokens,
    cache_write_tokens: observation.cacheWriteTokens,
    estimated_rebilled_tokens: observation.estimatedRebilledTokens,
    cache_read_ratio: observation.cacheReadRatio,
    idle_duration_ms: observation.idleDurationMs,
    layout_mode: observation.layoutMode,
    consecutive_prefix_misses: observation.consecutivePrefixMisses,
    cache_safe_degraded: observation.cacheSafeDegraded,
    transitioned_to_degraded: observation.transitionedToDegraded,
  });
  if (
    observation.transitionedToDegraded &&
    getAttachmentRuntime().cfg?.cacheSafePromptLayoutEnabled !== false
  ) {
    ctx.ui?.notify?.(
      "Focusa cache-safe degraded mode: repeated same-model prefix misses; optional context is temporarily suppressed.",
      "warning"
    );
  }
}

export function registerTurns(pi: ExtensionAPI) {
  // ── before_agent_start (§35.2 behavioral + §29 WBM injection) ────────────
  pi.on("before_agent_start", (event, ctx) => {
    // Reconnect check
    if (!getAttachmentRuntime().focusaAvailable) {
      void focusaFetch("/health").then((h) => {
        if (h?.ok) {
          getAttachmentRuntime().focusaAvailable = true;
          ctx.ui.setStatus("focusa", getAttachmentRuntime().wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
        }
      });
    }

    const confirmedProjectRoot = hardGateVitalProjectRoot(ctx);

    // Cache boundary: system instructions must remain byte-stable across adjacent turns.
    // Project identity, Workpoint state, recaps, recent turns, and WBM context are volatile
    // and are attached by the context hook to the newest user turn instead.
    const behavioral = [
      "\n## Focusa Cognitive Guidance",
      "You are operating within Focusa, a cognitive runtime that preserves focus and decisions.\n",
      "RULES:",
      "- Use the focusa_decide tool when you make a significant decision",
      "- Use the focusa_constraint tool ONLY for hard constraints (e.g. 'NEVER delete production data', 'must preserve X')",
      "- Use the focusa_failure tool when something fails",
      "- Do NOT record internal monologue, reasoning, or self-referential notes as constraints",
      "  (e.g. 'cannot advance without operator direction' is NOT a constraint — it's context)",
      "- Check the dynamic Focusa Focus Slice before acting and do not violate its constraints",
      "- Do not contradict decisions in the dynamic Focusa Focus Slice without explanation",
      "- If context was compacted, a scoped canonical Workpoint packet outranks transcript tail",
      "- Project-aware writes fail closed unless the dynamic Focusa Focus Slice verifies project_root + continuity_id authority",
      "- If project identity is ambiguous, infer from bounded repository evidence and ask the operator only when multiple plausible roots remain",
    ].join("\n");
    const workpointLaw = [
      "\n## Focusa Workpoint Continuity Law",
      "If a scoped Focusa WorkpointResumePacket is present in the newest-turn Focus Slice, treat it as the continuation anchor unless the operator explicitly steers elsewhere.",
      "Do not use raw transcript tail to override the active scoped Workpoint.",
    ].join("\n");

    (event as any).systemPrompt = ((event as any).systemPrompt || "") + "\n" + behavioral + workpointLaw;
    if (getAttachmentRuntime().cfg?.cacheSafePromptLayoutEnabled === false) {
      const legacyRecentTurns = buildCachedRecentTurnsSlice(4);
      const legacyWbm = "";
      const visibleRecapReason = toolOutputVisibleRecapReason();
      const legacyDynamic = [
        confirmedProjectRoot
          ? `Focusa project_root confirmed for this turn: ${confirmedProjectRoot}`
          : "Focusa project_root is unconfirmed; project-aware writes remain blocked pending bounded identity verification.",
        ...formatWorkpointContextSections(),
        visibleRecapReason
          ? `FOCUSA VISIBLE RECAP REQUIRED: ${visibleRecapReason}; recap MEMORY_ANCHOR/latest_report_summary_ref before action.`
          : "",
        buildFocusaUtilityCard("system"),
        legacyRecentTurns,
        legacyWbm,
      ].filter(Boolean);
      (event as any).systemPrompt += `\n\n${legacyDynamic.join("\n")}`;
    }
    cacheSafetyMonitor.captureSystemPrompt(cacheSessionKey(), (event as any).systemPrompt);

    if (!getAttachmentRuntime().seenFirstBeforeAgentStart) {
      getAttachmentRuntime().seenFirstBeforeAgentStart = true;
      if (nativeSessionAllowsNonessentialPersistence()) {
        const fallbackCard = buildFocusaUtilityCard("visible");
        void buildAwarenessPacket("reload")
          .then((packet) => renderAwarenessPacketText(packet))
          .catch(() => fallbackCard)
          .then((visibleCard) => {
            const ctxUi = ctx.ui as any;
            const wasExpanded = ctxUi?.getToolsExpanded?.() ?? true;
            ctxUi?.setToolsExpanded?.(false);
            try {
              pi.sendMessage(
                { customType: "focusa-utility-card", content: visibleCard, display: true },
                { triggerTurn: false }
              );
            } finally {
              ctxUi?.setToolsExpanded?.(wasExpanded);
            }
            queueTraceTelemetry({
              event_type: "focusa_utility_card_visible",
              turn_id: `pi-turn-${getTurnCount()}`,
              surface: "pi",
              bytes: visibleCard.length,
              renderer: visibleCard === fallbackCard ? "cached_fallback" : "dvs_awareness_substrate",
            });
          });
      } else {
        queueTraceTelemetry({
          event_type: "focusa_utility_card_suppressed",
          turn_id: `pi-turn-${getTurnCount()}`,
          surface: "pi",
          reason: "native_session_hard_pressure",
        });
      }
    }
    // §130: utility-card persistence block end.
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
  pi.on("context", (event: any, ctx: any) => {
    const contextMessages = elideOldRehydratableToolHistory(event.messages || []);
    const cacheSafeLayoutEnabled = getAttachmentRuntime().cfg?.cacheSafePromptLayoutEnabled !== false;
    const cacheSafeDegraded = cacheSafeLayoutEnabled && cacheSafetyMonitor.isDegraded(cacheSessionKey());
    const cacheInjectionPosition = cacheSafeLayoutEnabled
      ? "newest_user_turn_tail"
      : "legacy_history_prepend";
    const [recentTurnsContext, wbmContext] =
      !cacheSafeLayoutEnabled || cacheSafeDegraded ? ["", ""] : [buildCachedRecentTurnsSlice(4), ""];
    if (!getAttachmentRuntime().focusaAvailable || !getAttachmentRuntime().activeFrameId) {
      const askText = getAttachmentRuntime().currentAsk?.text || "";
      const scopeKind = getAttachmentRuntime().queryScope?.scopeKind || "mission_carryover";
      const visibleRecapReason = toolOutputVisibleRecapReason();
      const verdict = buildAttentionRecallVerdict({
        currentAskText: askText,
        currentAskKind: getAttachmentRuntime().currentAsk?.kind,
        queryScopeKind: scopeKind,
        projectRoot: getSessionCwd(),
        workpointPacket: getScopedWorkpointPacket(),
        visibleRecapReason,
      });
      const lines = [
        "[Focusa advisory — operator steering remains authoritative]",
        ...formatAttentionRecallFocusSliceLines(verdict),
        ...formatCurrentAskScopeVerdictLines(
          buildCurrentAskScopeVerdict({
            currentAskText: askText,
            workpointPacket: getScopedWorkpointPacket(),
            projectRoot: getSessionCwd(),
            continuityId: getContinuityId(),
          })
        ),
        ...formatToolOutputVisibleRecapLines(visibleRecapReason),
        ...formatWorkpointContextSections().slice(0, 2),
        ...getToolAffordanceFocusSliceLines({
          resourceModeActive: false,
          hasTrajectory: false,
          hasWorkpoint: Boolean(getActiveWorkpointPacket()),
          hasOntologyAmbiguity: false,
        }),
      ].filter(Boolean);
      return { messages: attachCacheSafeFocusSlice(event, contextMessages, lines.join("\n")) };
    }

    const localSnapshot = getEffectiveFocusSnapshot();
    const data = getCachedFocusState() || {
      frame: {
        id: getAttachmentRuntime().activeFrameId,
        title: getAttachmentRuntime().activeFrameTitle,
        goal: getAttachmentRuntime().activeFrameGoal,
      },
      fs: {
        decisions: localSnapshot.decisions,
        constraints: localSnapshot.constraints,
        failures: localSnapshot.failures,
        intent: localSnapshot.intent,
        current_focus: localSnapshot.currentFocus,
        current_state: localSnapshot.currentFocus,
      },
      stack: null,
    };
    if (!data?.fs) {
      const askText = getAttachmentRuntime().currentAsk?.text || "";
      const visibleRecapReason = toolOutputVisibleRecapReason();
      const verdict = buildAttentionRecallVerdict({
        currentAskText: askText,
        currentAskKind: getAttachmentRuntime().currentAsk?.kind,
        queryScopeKind: getAttachmentRuntime().queryScope?.scopeKind,
        projectRoot: getSessionCwd(),
        workpointPacket: getScopedWorkpointPacket(),
        visibleRecapReason,
      });
      const lines = [
        "[Focusa advisory — cached state unavailable; operator flow continues]",
        ...formatAttentionRecallFocusSliceLines(verdict),
        ...formatCurrentAskScopeVerdictLines(
          buildCurrentAskScopeVerdict({
            currentAskText: askText,
            workpointPacket: getScopedWorkpointPacket(),
            projectRoot: getSessionCwd(),
            continuityId: getContinuityId(),
          })
        ),
        ...formatToolOutputVisibleRecapLines(visibleRecapReason),
        ...formatWorkpointContextSections().slice(0, 2),
      ].filter(Boolean);
      return { messages: attachCacheSafeFocusSlice(event, contextMessages, lines.join("\n")) };
    }

    const { fs, frame } = data;
    const fmt = (label: string, items: string[] | undefined) =>
      items?.length
        ? `${label}:\n${items.map((item: string) => `  - ${item}`).join("\n")}`
        : `${label}:\n  (none)`;

    // §36.7: Budget check — cap injection to 600 tokens, 250 under high pressure
    const usage = ctx.getContextUsage?.();
    const window = usage?.contextWindow || getAttachmentRuntime().activeContextWindow || 128000;
    if (typeof usage?.contextWindow === "number" && usage.contextWindow > 0) {
      getAttachmentRuntime().activeContextWindow = usage.contextWindow;
    }
    const headroom = usage?.tokens ? window - usage.tokens - 16384 : window;
    const maxTokens =
      (getAttachmentRuntime().currentContextPct || 0) >= 85
        ? 250
        : Math.min(Math.max(Math.floor(headroom * 0.08), 160), 600);

    const scopeKind = getAttachmentRuntime().queryScope?.scopeKind || "mission_carryover";
    const askText = getAttachmentRuntime().currentAsk?.text || "";
    const missionIncluded = shouldIncludeMissionContext(askText, scopeKind, [
      fs.intent || "",
      fs.current_focus || "",
      fs.current_state || "",
      frame?.title || "",
    ]);
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

    const relevantDecisions = selectRelevantItems(fs.decisions, askText, {
      maxItems: 3,
      fallbackItems: scopeKind === "mission_carryover" ? 2 : 0,
      minScore: 2,
    });
    const relevantConstraints = selectRelevantItems(fs.constraints, askText, {
      maxItems: 3,
      fallbackItems: scopeKind === "mission_carryover" ? 2 : 0,
      minScore: 2,
    });
    const decisionRetention = retentionBucketsFromSelection(relevantDecisions, {
      maxDecayed: 2,
      maxHistorical: 2,
    });
    const constraintRetention = retentionBucketsFromSelection(relevantConstraints, {
      maxDecayed: 2,
      maxHistorical: 2,
    });
    const decayedContextItems = [
      ...constraintRetention.decayed.map((value) => `constraint: ${value}`),
      ...decisionRetention.decayed.map((value) => `decision: ${value}`),
    ];
    const historicalContextItems = [
      ...constraintRetention.historical.map((value) => `constraint: ${value}`),
      ...decisionRetention.historical.map((value) => `decision: ${value}`),
    ];
    const recentResults = selectRelevantItems(fs.recent_results, askText, {
      maxItems: 2,
      fallbackItems: scopeKind === "mission_carryover" ? 1 : 0,
      minScore: 2,
    });
    const nextSteps = selectRelevantItems(fs.next_steps, askText, {
      maxItems: 2,
      fallbackItems: scopeKind === "mission_carryover" ? 1 : 0,
      minScore: 2,
    });
    const openQuestions = selectRelevantItems(fs.open_questions, askText, {
      maxItems: 2,
      fallbackItems: 0,
      minScore: 2,
    });
    const failures = selectRelevantItems(fs.failures, askText, {
      maxItems: 2,
      fallbackItems: scopeKind === "correction" ? 1 : 0,
      minScore: 2,
    });
    const artifactLabels =
      fs.artifacts?.map((a: any) => `${a.kind}:${a.label}${a.path_or_id ? "@" + a.path_or_id : ""}`) || [];
    const relevantArtifacts = selectRelevantItems(artifactLabels, askText, {
      maxItems: 2,
      fallbackItems: scopeKind === "mission_carryover" ? 1 : 0,
      minScore: 2,
    });
    const includeAuxContext = maxTokens >= 350;
    const [semanticMemory, ecsHandles] = includeAuxContext
      ? [getCachedSemanticMemorySummary(), getCachedEcsHandlesSummary()]
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
    // Provider request hooks are latency-critical. Ontology is refreshed by
    // background/session lifecycle work and omitted when no cached projection is
    // available; a daemon timeout must never delay the operator's prompt.
    const ontologyContext: any = null;
    const ontologyPayload = ontologyContext?.ontology_context || ontologyContext;
    const ontologyObjectLines = Array.isArray(ontologyPayload?.active_object_set)
      ? ontologyPayload.active_object_set
          .slice(0, 6)
          .map(
            (item: any) =>
              `${item.object_type || "object"}:${item.id || "unknown"} (${item.uncertainty || "unknown"})`
          )
      : [];
    const ontologyLinkLines = Array.isArray(ontologyPayload?.relevant_link_paths)
      ? ontologyPayload.relevant_link_paths.slice(0, 6).map((item: any) => String(item.path || item))
      : [];
    const ontologyActionLines = Array.isArray(ontologyPayload?.valid_next_actions)
      ? ontologyPayload.valid_next_actions
          .slice(0, 4)
          .map((item: any) => String(item.name || "unknown_action"))
      : [];
    const ontologyBlockedLines = Array.isArray(ontologyPayload?.blocked_affordances)
      ? ontologyPayload.blocked_affordances
          .slice(0, 4)
          .map((item: any) => String(item.name || item.id || item))
      : [];
    const ontologyEvidenceLines = Array.isArray(ontologyPayload?.evidence_handles)
      ? ontologyPayload.evidence_handles.slice(0, 4).map((item: any) => {
          const trajectory = item?.trajectory || {};
          const stg = boundedTrajectoryText(trajectory.stg || trajectory.short_term_goal, 70);
          return `${item.kind || "evidence"}:${item.label || item.id || "unknown"}${stg ? ` (STG=${stg})` : ""}`;
        })
      : [];
    const ontologyUncertaintyLines = Array.isArray(ontologyPayload?.uncertainty_flags)
      ? ontologyPayload.uncertainty_flags.slice(0, 6).map((item: any) => String(item))
      : [];
    const trajectoryLines = getCachedTrajectoryFocusSliceLines();
    const resourceModeLines: string[] = [];
    const toolAffordanceLines = getToolAffordanceFocusSliceLines({
      resourceModeActive: resourceModeLines.length > 0,
      hasTrajectory: trajectoryLines.length > 0,
      hasWorkpoint: Boolean(getActiveWorkpointPacket()),
      hasOntologyAmbiguity: ontologyObjectLines.length > 1 || ontologyUncertaintyLines.length > 0,
    });

    const visibleRecapReason = toolOutputVisibleRecapReason();
    const attentionLines = formatAttentionRecallFocusSliceLines(
      buildAttentionRecallVerdict({
        focusState: fs,
        workpointPacket: getScopedWorkpointPacket(),
        currentAskText: getAttachmentRuntime().currentAsk?.text || askText,
        currentAskKind: getAttachmentRuntime().currentAsk?.kind,
        queryScopeKind: scopeKind,
        projectRoot: getSessionCwd(),
        continuityId: getContinuityId(),
        visibleRecapReason,
      })
    );

    const sectionEntries = [
      {
        key: "projection_kind",
        text: `PROJECTION_KIND: ${projectionKind}`,
        include: true,
        selectedCount: 1,
        excludedCount: 0,
        priority: 0,
        relevanceScore: 100,
      },
      {
        key: "view_profile",
        text: `VIEW_PROFILE: ${viewProfile}`,
        include: true,
        selectedCount: 1,
        excludedCount: 0,
        priority: 1,
        relevanceScore: 100,
      },
      {
        key: "current_ask",
        text: `CURRENT_ASK: ${getAttachmentRuntime().currentAsk?.text || askText || "(none)"}`,
        include: Boolean(getAttachmentRuntime().currentAsk?.text || askText),
        selectedCount: 1,
        excludedCount: 0,
        priority: 2,
        relevanceScore: 100,
      },
      {
        key: "query_scope",
        text: `QUERY_SCOPE: ${scopeKind} · ${getAttachmentRuntime().queryScope?.carryoverPolicy || "allow_if_relevant"}`,
        include: true,
        selectedCount: 1,
        excludedCount: 0,
        priority: 3,
        relevanceScore: 100,
      },
      buildSliceSection(
        "current_ask_scope_verdict",
        "CURRENT_ASK_SCOPE_VERDICT",
        formatCurrentAskScopeVerdictLines(
          buildCurrentAskScopeVerdict({
            currentAskText: askText,
            workpointPacket: getScopedWorkpointPacket(),
            projectRoot: getSessionCwd(),
            continuityId: getContinuityId(),
          })
        ),
        true,
        (values) => values.join("\n"),
        0,
        4,
        100
      ),
      buildSliceSection(
        "project_switch_ledger",
        "PROJECT_SWITCH_LEDGER",
        formatProjectSwitchLedgerLines(askText),
        getAttachmentRuntime().projectSwitchLedger.length > 0,
        (values) => `PROJECT_SWITCH_LEDGER:\n${values.map((value) => `  - ${value}`).join("\n")}`,
        0,
        5,
        96
      ),
      buildSliceSection(
        "resource_mode",
        "RESOURCE_MODE",
        resourceModeLines,
        resourceModeLines.length > 0,
        (values) => values.join("\n"),
        0,
        4,
        100
      ),
      buildSliceSection(
        "trajectory",
        "PROJECT_TRAJECTORY",
        trajectoryLines,
        trajectoryLines.length > 0,
        (values) => `PROJECT_TRAJECTORY:\n${values.map((value) => `  - ${value}`).join("\n")}`,
        0,
        5,
        100
      ),
      buildSliceSection(
        "workpoint",
        "WORKPOINT",
        formatWorkpointContextSections(),
        Boolean(getActiveWorkpointPacket()),
        (values) => values.join("\n"),
        0,
        6,
        100
      ),
      buildSliceSection(
        "uiai_first_web_research",
        "UIAI_FIRST_WEB_RESEARCH",
        getUiaiFirstFocusSliceLines(askText),
        currentAskLooksLikeWebResearch(askText),
        (values) => values.join("\n"),
        0,
        4,
        98
      ),
      buildSliceSection(
        "tool_affordances",
        "TOOL_AFFORDANCES",
        toolAffordanceLines,
        toolAffordanceLines.length > 0,
        (values) => values.join("\n"),
        0,
        7,
        95
      ),
      {
        key: "focus_frame",
        text: `FOCUS_FRAME: ${frame?.title || "(untitled)"}`,
        include: missionIncluded && Boolean(frame?.title),
        selectedCount: frame?.title ? 1 : 0,
        excludedCount: 0,
        priority: 10,
        relevanceScore: missionIncluded ? 50 : 0,
      },
      {
        key: "current_focus",
        text: `CURRENT_FOCUS: ${fs.current_focus || fs.current_state || "(none)"}`,
        include: missionIncluded && Boolean(fs.current_focus || fs.current_state),
        selectedCount: fs.current_focus || fs.current_state ? 1 : 0,
        excludedCount: 0,
        priority: 11,
        relevanceScore: missionIncluded ? 45 : 0,
      },
      {
        key: "intent",
        text: `INTENT: ${fs.intent || "(none)"}`,
        include: missionIncluded && Boolean(fs.intent),
        selectedCount: fs.intent ? 1 : 0,
        excludedCount: 0,
        priority: 12,
        relevanceScore: missionIncluded ? 40 : 0,
      },
      {
        key: "projection_boundary",
        text: `PROJECTION_BOUNDARY: token_budget=${maxTokens} carryover=${getAttachmentRuntime().queryScope?.carryoverPolicy || "allow_if_relevant"} mission=${missionIncluded ? "included" : "suppressed"}`,
        include: true,
        selectedCount: 1,
        excludedCount: 0,
        priority: 13,
        relevanceScore: 90,
      },
      {
        key: "canonical_sources",
        text: `CANONICAL_SOURCES: focus_state semantic_memory ecs_handles reference_index`,
        include: true,
        selectedCount: 4,
        excludedCount: 0,
        priority: 14,
        relevanceScore: 90,
      },
      buildSliceSection(
        "canonical_references",
        "REFERENCE_ALIASES",
        canonicalReferenceAliases,
        canonicalReferenceAliases.length > 0,
        (values) => fmt("REFERENCE_ALIASES", values),
        0,
        15,
        85
      ),
      buildSliceSection(
        "ontology_active_objects",
        "ACTIVE_OBJECT_SET",
        ontologyObjectLines,
        ontologyObjectLines.length > 0,
        (values) => fmt("ACTIVE_OBJECT_SET", values),
        0,
        16,
        92
      ),
      buildSliceSection(
        "ontology_link_paths",
        "RELEVANT_LINK_PATHS",
        ontologyLinkLines,
        ontologyLinkLines.length > 0,
        (values) => fmt("RELEVANT_LINK_PATHS", values),
        0,
        17,
        90
      ),
      buildSliceSection(
        "ontology_next_actions",
        "VALID_NEXT_ACTIONS",
        ontologyActionLines,
        ontologyActionLines.length > 0,
        (values) => fmt("VALID_NEXT_ACTIONS", values),
        0,
        18,
        88
      ),
      buildSliceSection(
        "ontology_blocked_affordances",
        "BLOCKED_AFFORDANCES",
        ontologyBlockedLines,
        ontologyBlockedLines.length > 0,
        (values) => fmt("BLOCKED_AFFORDANCES", values),
        0,
        19,
        82
      ),
      buildSliceSection(
        "ontology_evidence_handles",
        "EVIDENCE_HANDLES",
        ontologyEvidenceLines,
        ontologyEvidenceLines.length > 0,
        (values) => fmt("EVIDENCE_HANDLES", values),
        0,
        20,
        84
      ),
      buildSliceSection(
        "ontology_uncertainty",
        "UNCERTAINTY_FLAGS",
        ontologyUncertaintyLines,
        ontologyUncertaintyLines.length > 0,
        (values) => fmt("UNCERTAINTY_FLAGS", values),
        0,
        21,
        80
      ),
      buildSliceSection(
        "working_set",
        "WORKING_SET",
        relevantWorkingSet.items,
        relevantWorkingSet.items.length > 0,
        (values) => fmt("WORKING_SET", values),
        relevantWorkingSet.excluded.length,
        20,
        selectionRelevanceScore(relevantWorkingSet)
      ),
      buildSliceSection(
        "constraints",
        "CONSTRAINTS",
        relevantConstraints.items,
        relevantConstraints.items.length > 0,
        (values) => fmt("CONSTRAINTS", values),
        relevantConstraints.excluded.length,
        20,
        selectionRelevanceScore(relevantConstraints)
      ),
      buildSliceSection(
        "decisions",
        "DECISIONS",
        relevantDecisions.items,
        relevantDecisions.items.length > 0,
        (values) => fmt("DECISIONS", values),
        relevantDecisions.excluded.length,
        20,
        selectionRelevanceScore(relevantDecisions)
      ),
      buildSliceSection(
        "decayed_context",
        "DECAYED_CONTEXT",
        decayedContextItems,
        (scopeKind === "mission_carryover" || scopeKind === "correction" || scopeKind === "meta") &&
          decayedContextItems.length > 0,
        (values) => fmt("DECAYED_CONTEXT", values),
        0,
        21,
        6
      ),
      buildSliceSection(
        "historical_context",
        "HISTORICAL_CONTEXT",
        historicalContextItems,
        (scopeKind === "mission_carryover" || scopeKind === "meta") && historicalContextItems.length > 0,
        (values) => fmt("HISTORICAL_CONTEXT", values),
        0,
        22,
        4
      ),
      buildSliceSection(
        "verified_deltas",
        "VERIFIED_DELTAS",
        relevantVerifiedDeltas.items,
        relevantVerifiedDeltas.items.length > 0,
        (values) => fmt("VERIFIED_DELTAS", values),
        relevantVerifiedDeltas.excluded.length,
        20,
        selectionRelevanceScore(relevantVerifiedDeltas)
      ),
      buildSliceSection(
        "recent_results",
        "RECENT_RESULTS",
        recentResults.items,
        scopeKind !== "fresh_question" && recentResults.items.length > 0,
        (values) => fmt("RECENT_RESULTS", values),
        recentResults.excluded.length,
        20,
        selectionRelevanceScore(recentResults)
      ),
      buildSliceSection(
        "failures",
        "FAILURES",
        failures.items,
        (scopeKind === "correction" || scopeKind === "mission_carryover") && failures.items.length > 0,
        (values) => fmt("FAILURES", values),
        failures.excluded.length,
        20,
        selectionRelevanceScore(failures)
      ),
      buildSliceSection(
        "next_steps",
        "NEXT_STEPS",
        nextSteps.items,
        scopeKind === "mission_carryover" && nextSteps.items.length > 0,
        (values) => fmt("NEXT_STEPS", values),
        nextSteps.excluded.length,
        20,
        selectionRelevanceScore(nextSteps)
      ),
      buildSliceSection(
        "artifacts",
        "ARTIFACT_HANDLES",
        relevantArtifacts.items,
        relevantArtifacts.items.length > 0,
        (values) => fmt("ARTIFACT_HANDLES", values),
        relevantArtifacts.excluded.length,
        20,
        selectionRelevanceScore(relevantArtifacts)
      ),
      buildSliceSection(
        "open_questions",
        "OPEN_QUESTIONS",
        openQuestions.items,
        scopeKind === "meta" && openQuestions.items.length > 0,
        (values) => fmt("OPEN_QUESTIONS", values),
        openQuestions.excluded.length,
        20,
        selectionRelevanceScore(openQuestions)
      ),
    ];

    const cacheSafeExcludedLabels = cacheSafeDegraded
      ? sectionEntries
          .filter((entry) => entry.include && !CACHE_SAFE_DEGRADED_RETAINED_SECTIONS.has(entry.key))
          .map((entry) => entry.key)
      : [];
    const scopedEntries = orderSliceSections(sectionEntries).filter(
      (entry) => entry.include && !cacheSafeExcludedLabels.includes(entry.key)
    );
    const scopeExcludedLabels = [
      ...sectionEntries.filter((entry) => !entry.include).map((entry) => entry.key),
      ...cacheSafeExcludedLabels,
    ];
    const retainedDecisionHistoryCount =
      decisionRetention.decayed.length + decisionRetention.historical.length;
    const retainedConstraintHistoryCount =
      constraintRetention.decayed.length + constraintRetention.historical.length;
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
    const receiptExcludedCount = scopeExcludedLabels.length + irrelevantExcludedLabels.length;
    const contextReceiptHelpful =
      receiptExcludedCount > 0 ||
      Boolean(visibleRecapReason) ||
      scopeKind === "correction" ||
      scopeKind === "mission_carryover" ||
      !missionIncluded;
    const staleOrAdvisory = [
      trajectoryLines.length ? "trajectory_advisory" : "",
      toolAffordanceLines.length ? "tool_affordances_advisory" : "",
      getActiveWorkpointPacket() ? "" : "workpoint_not_verified",
    ].filter(Boolean);
    const contextReceiptLines = contextReceiptHelpful
      ? [
          `CONTEXT_RECEIPT: included=${scopedEntries.length} excluded=${receiptExcludedCount} omitted_bytes=${Math.max(0, receiptExcludedCount * 96)} rehydrate_refs=focusa_workpoint_resume,focusa_trajectory_view,focusa_traverse reason=current_ask+Workpoint+trajectory_gap stale_or_advisory=${staleOrAdvisory.join(",") || "none"}`,
        ]
      : [];

    // §Prompt Serialization: uppercase section headers, bullets for list items
    const lines: string[] = [
      `[Focusa Focus Slice — minimal applicable context]`,
      ...attentionLines,
      ...formatToolOutputVisibleRecapLines(visibleRecapReason),
      ...contextReceiptLines,
      ...scopedEntries.map((entry) => entry.text),
      `CACHE_SAFETY: mode=${cacheSafeDegraded ? "cache_safe_degraded" : "normal"} injection=${cacheInjectionPosition}`,
      recentTurnsContext,
      wbmContext,
    ].filter(Boolean);

    // §36.7: Budget cap — truncate if over token budget
    let text = lines.join("\n");
    const fullTokens = estimateTokens(text);
    const truncated = fullTokens > maxTokens;
    if (truncated) {
      // Truncate from bottom while preserving the non-droppable attention/recall prefix.
      const attentionEnd = lines.findIndex((line) => line === "END_ATTENTION_RECALL");
      const protectedPrefixCount = Math.max(4, attentionEnd >= 0 ? attentionEnd + 1 : 0);
      text =
        lines.slice(0, protectedPrefixCount).join("\n") +
        `\n[... Focus State truncated — ${fullTokens - maxTokens} tokens over budget]`;
    }
    const injectedTokens = estimateTokens(text);

    // Minimal context-injection trace telemetry for SPEC 56 / doc 78 gap closure.
    // Emit explicit typed trace events for the fields we can objectively compute today,
    // without pretending the hot path already has richer routing/hijack semantics.
    const lastUserMsg = [...(event.messages || [])].reverse().find((m: any) => m?.role === "user");
    const lastUserText = extractText(lastUserMsg?.content || "").slice(0, 200);
    const priorMissionReused =
      scopeKind === "mission_carryover" &&
      Boolean(fs.intent || fs.current_focus || fs.current_state || (fs.decisions && fs.decisions.length));
    const budgetExcludedLabels = truncated
      ? [
          "artifacts",
          "verified_deltas",
          "working_set",
          "constraints",
          "open_questions",
          "next_steps",
          "recent_results",
          "failures",
        ]
      : [];
    const relevantContextLabels = scopedEntries.map((entry) => entry.key);
    const focusSliceRelevanceScore = scopedEntries.length
      ? scopedEntries.reduce((sum, entry) => sum + (entry.relevanceScore || 0), 0) / scopedEntries.length
      : 0;
    const excludedContext = Array.from(
      new Set([...scopeExcludedLabels, ...irrelevantExcludedLabels, ...budgetExcludedLabels])
    );
    const contextTurnId = `pi-turn-${getTurnCount()}`;
    const scopeSourceTurnId =
      getAttachmentRuntime().queryScope?.sourceTurnId ||
      getAttachmentRuntime().currentAsk?.sourceTurnId ||
      contextTurnId;
    const workingSetPriorHits = relevantWorkingSet.scores
      .filter(({ value, priorBoost }) => relevantWorkingSet.items.includes(value) && (priorBoost || 0) > 0)
      .map(({ value, priorBoost, appliedPriors }) => ({
        value,
        priorBoost: priorBoost || 0,
        appliedPriors: appliedPriors || [],
      }));
    const verifiedDeltaPriorHits = relevantVerifiedDeltas.scores
      .filter(
        ({ value, priorBoost }) => relevantVerifiedDeltas.items.includes(value) && (priorBoost || 0) > 0
      )
      .map(({ value, priorBoost, appliedPriors }) => ({
        value,
        priorBoost: priorBoost || 0,
        appliedPriors: appliedPriors || [],
      }));
    const resetReason =
      scopeKind === "fresh_question" ? "fresh_scope" : scopeKind === "correction" ? "correction_reset" : null;
    const exclusionReason = truncated
      ? "budget_truncation"
      : resetReason ||
        (cacheSafeExcludedLabels.length
          ? "cache_safe_degraded"
          : irrelevantExcludedLabels.length
            ? "irrelevance"
            : "none");
    getAttachmentRuntime().excludedContext = {
      labels: excludedContext,
      reason: exclusionReason,
      sourceTurnId: scopeSourceTurnId,
      updatedAt: Date.now(),
    };

    if (getAttachmentRuntime().focusaAvailable) {
      turnWorkLoopWriterHeaders()
        .then((headers) =>
          focusaFetch("/work-loop/context", {
            method: "POST",
            headers,
            body: JSON.stringify({
              excluded_context_reason: exclusionReason,
              excluded_context_labels: excludedContext,
              source_turn_id: scopeSourceTurnId,
            }),
          })
        )
        .catch(() => null);
    }

    if (getAttachmentRuntime().cfg?.emitMetrics) {
      const common = {
        turn_id: contextTurnId,
        frame_id: getAttachmentRuntime().activeFrameId,
        surface: "pi",
        routing_mode: "minimal_focus_slice_builder",
        focus_slice_estimated_tokens: injectedTokens,
        focus_slice_full_tokens: fullTokens,
        focus_slice_truncated: truncated,
        excluded_context: excludedContext,
        current_ask_kind: getAttachmentRuntime().currentAsk?.kind,
        query_scope_kind: getAttachmentRuntime().queryScope?.scopeKind,
        carryover_policy: getAttachmentRuntime().queryScope?.carryoverPolicy,
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
          carryover_policy: getAttachmentRuntime().queryScope?.carryoverPolicy,
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
        selected_counts: Object.fromEntries(
          scopedEntries.map((entry) => [entry.key, entry.selectedCount || 0])
        ),
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
      if (
        !missionIncluded &&
        (scopeKind === "fresh_question" || scopeKind === "correction" || excludedContext.length > 0)
      ) {
        queueTraceTelemetry({
          event_type: "subject_hijack_prevented",
          ...common,
          subject_hijack_prevented: true,
          prevented_by: exclusionReason,
        });
      }
    }

    // Lifecycle guidance is non-triggering: session hooks queue one idempotent
    // advisory, and the next real operator turn receives it in the cache-safe tail.
    const lifecycleSessionId = String(
      ctx?.sessionManager?.getSessionId?.() || getAttachmentRuntime().sessionFrameKey || "no-session"
    );
    const lifecycleAdvisory = getAttachmentRuntime().pendingLifecycleAdvisories[lifecycleSessionId];
    if (lifecycleAdvisory) {
      text += `\n\n[Focusa deferred lifecycle advisory]\n${lifecycleAdvisory.text}`;
      delete getAttachmentRuntime().pendingLifecycleAdvisories[lifecycleSessionId];
      persistState();
      queueTraceTelemetry({
        event_type: "pi_lifecycle_advisory_delivered_in_next_turn",
        session_id: lifecycleSessionId,
        idempotency_key: lifecycleAdvisory.key,
        reason: lifecycleAdvisory.reason,
        outcome: "delivered",
      });
    }

    // Cache-safe layout: preserve historical ordering and append volatile Focusa state
    // only to the newest user turn so the system/history prefix remains reusable.
    return {
      messages: attachCacheSafeFocusSlice(event, contextMessages, text),
    };
  });

  // ── input (§36.3 signal + §35.7 correction — single handler) ──────────────
  pi.on("input", (event, _ctx) => {
    const text = (event as any).text || (event as any).message || "";
    const cleanedText = stripQuotedFocusaContext(String(text));

    // §5.12.10: recall-intent trigger — detect and force re-emit.
    const intent = detectRecallIntent(cleanedText);
    if (intent) {
      getAttachmentRuntime().lastRecentTurnsSliceTurn = -1;
      if (getAttachmentRuntime().focusaAvailable) {
        const ringSize = (getAttachmentRuntime().recentTurns || []).length;
        focusaPost("/v1/events/recall-trigger", {
          matched_category: intent.matched_category,
          matched_phrase: intent.matched_phrase,
          slice_size: 0,
          ring_size: ringSize,
          forced_re_emit: true,
          alternative_tools_surfaced:
            ringSize === 0 ? ["focusa_lineage_tree", "focusa_awareness_packet"] : [],
          continuity_id: getContinuityId(),
          agent_kind: ADAPTER_KIND,
        });
      }
    }
    // Input is the pre-turn boundary for the upcoming model call.
    // Use the next turn id so CurrentAsk/QueryScope survive unchanged into context injection.
    const sourceTurnId = `pi-turn-${getTurnCount() + 1}`;
    const packageUpdateCommand = /^\s*(update|pi\s+update|\/update)\s*$/i.test(String(text));
    const askKind = packageUpdateCommand ? "meta" : classifyCurrentAsk(String(text));
    const storedAskText = cleanedText || (askKind === "meta" ? "" : String(text));
    const newTaskText = storedAskText.slice(0, 500);
    getAttachmentRuntime().currentTaskStartTime = Date.now();
    getAttachmentRuntime().currentTaskLabel = newTaskText;
    setCurrentTaskTurnStart(getTurnCount() + 1);
    getAttachmentRuntime().currentTaskInputTokenEstimate = estimateTokens(newTaskText);
    getAttachmentRuntime().currentTaskOutputTokenEstimate = 0;
    getAttachmentRuntime().currentTaskProviderInputTokens = 0;
    getAttachmentRuntime().currentTaskProviderOutputTokens = 0;
    getAttachmentRuntime().currentTaskToolCalls = 0;
    getAttachmentRuntime().currentAsk = {
      text: newTaskText,
      kind: askKind,
      sourceTurnId,
      updatedAt: Date.now(),
      sessionId: getAttachmentRuntime().sessionFrameKey,
      projectRoot: getSessionCwd(),
      continuityId: getContinuityId(),
    };
    observeProjectThreadHintsFromText(newTaskText, sourceTurnId, "current_ask", "current_ask_project_hints");
    const queryScope = deriveQueryScope(askKind);
    const steeringDetected = isOperatorSteeringInput(String(text), askKind);
    getAttachmentRuntime().queryScope = {
      ...queryScope,
      sourceTurnId,
      updatedAt: Date.now(),
    };
    getAttachmentRuntime().excludedContext = {
      labels: [],
      reason: askKind === "question" ? "fresh_scope" : askKind === "correction" ? "correction_reset" : "none",
      sourceTurnId,
      updatedAt: Date.now(),
    };

    const projectRoot = adoptPiProjectRoot((_ctx as any)?.cwd);
    const rootConfirmed = !projectRootConfirmationRequired(projectRoot);
    if (
      getAttachmentRuntime().focusaAvailable &&
      getAttachmentRuntime().activeFrameId &&
      !packageUpdateCommand &&
      rootConfirmed
    ) {
      void rescopePiFrameFromCurrentAsk(projectRoot, "pi-post-input-rescope")
        .then(() => getFocusState())
        .catch(() => null);
    }

    if (getAttachmentRuntime().focusaAvailable) {
      void turnWorkLoopWriterHeaders()
        .then((headers) =>
          focusaFetch("/work-loop/context", {
            method: "POST",
            headers,
            body: JSON.stringify({
              current_ask: getAttachmentRuntime().currentAsk.text,
              ask_kind: getAttachmentRuntime().currentAsk.kind,
              scope_kind: getAttachmentRuntime().queryScope.scopeKind,
              carryover_policy: getAttachmentRuntime().queryScope.carryoverPolicy,
              excluded_context_reason: getAttachmentRuntime().excludedContext.reason,
              excluded_context_labels: [],
              source_turn_id: sourceTurnId,
              operator_steering_detected: steeringDetected,
            }),
          })
        )
        .catch(() => null);
      if (steeringDetected && rootConfirmed) {
        refreshTrajectoryClarityLifecycle("operator_steering", projectRoot).catch(() => null);
      }
    }

    if (getAttachmentRuntime().cfg?.emitMetrics) {
      const common = {
        turn_id: sourceTurnId,
        frame_id: getAttachmentRuntime().activeFrameId,
        surface: "pi",
        current_ask_kind: getAttachmentRuntime().currentAsk.kind,
        query_scope_kind: getAttachmentRuntime().queryScope.scopeKind,
        carryover_policy: getAttachmentRuntime().queryScope.carryoverPolicy,
      };
      queueTraceTelemetry({
        event_type: "operator_subject",
        ...common,
        operator_subject_preview: getAttachmentRuntime().currentAsk.text.slice(0, 200),
      });
      queueTraceTelemetry({
        event_type: "current_ask_determined",
        ...common,
        current_ask_text_preview: getAttachmentRuntime().currentAsk.text.slice(0, 200),
      });
      queueTraceTelemetry({
        event_type: "query_scope_built",
        ...common,
        query_scope_kind: getAttachmentRuntime().queryScope.scopeKind,
        carryover_policy: getAttachmentRuntime().queryScope.carryoverPolicy,
      });
      queueTraceTelemetry({
        event_type: "steering_detected",
        ...common,
        steering_detected: steeringDetected,
      });
    }

    if (getAttachmentRuntime().focusaAvailable) {
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "user_input",
        surface: "pi",
        payload: { length: text.length, preview: String(text).slice(0, 200) },
      });
    }

    const lower = String(text).toLowerCase();
    const corrections = [
      "no that is wrong",
      "revert",
      "undo",
      "that's incorrect",
      "wrong approach",
      "go back",
      "not what i asked",
    ];
    if (corrections.some((c) => lower.includes(c))) {
      // Correction is steering signal, not canonical failure.
      // Keep as telemetry/trust update to avoid stale Known Failures contamination.
      if (getAttachmentRuntime().focusaAvailable) {
        queueTraceTelemetry({
          event_type: "operator_correction_detected",
          turn_id: `pi-turn-${getTurnCount()}`,
          frame_id: getAttachmentRuntime().activeFrameId,
          surface: "pi",
          correction_preview: String(text).slice(0, 160),
        });
      }
      // §35.7/§29: WBM trust metric update on correction
      if (getAttachmentRuntime().wbmEnabled) {
        wbExec(["trust", "set", "--corrections", "+1"]).catch(() => {});
      }
    }
  });

  // ── turn_start (§34.2B) ───────────────────────────────────────────────────
  pi.on("turn_start", async (_event, _ctx) => {
    incrementTurnCount();
    setLastStreamLen(0);
    resetToolUsageBatch();
    // Reset dedup flag so next compaction can re-trigger auto-resume
    getAttachmentRuntime().compactResumePending = false;
    if (getAttachmentRuntime().focusaAvailable) {
      focusaPost("/turn/start", {
        turn_id: `pi-turn-${getTurnCount()}`,
        frame_id: getAttachmentRuntime().activeFrameId,
      });
    }
  });

  // ── turn_end (§35.5 tokens + §37.3 widget + §10.4 badges + §20 tier + §21 micro) ─
  pi.on("turn_end", async (event, ctx) => {
    const ev = event as any;
    const cfg = getAttachmentRuntime().cfg;
    const assistantOutput = extractText(ev.message?.content || ev.message || "");

    // §35.5: Token counts + assistant output
    if (getAttachmentRuntime().focusaAvailable) {
      const reportSummary = maybeCaptureReportSummaryFromAssistantOutput(
        assistantOutput,
        `pi-turn-${getTurnCount()}`
      );
      if (reportSummary) {
        queueTraceTelemetry({
          event_type: "report_summary_captured",
          turn_id: `pi-turn-${getTurnCount()}`,
          frame_id: getAttachmentRuntime().activeFrameId,
          surface: "pi",
          latest_report_summary_ref: reportSummary.handle,
        });
      }
      if (markVisibleRecapEmittedIfPresent(assistantOutput)) {
        queueTraceTelemetry({
          event_type: "visible_recap_emitted",
          turn_id: `pi-turn-${getTurnCount()}`,
          frame_id: getAttachmentRuntime().activeFrameId,
          surface: "pi",
          reason: "tool_output_flood",
        });
      }
      const detectedLeakClasses = detectForbiddenVisibleOutputLeakClasses(assistantOutput);
      if (detectedLeakClasses.length) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "visible_output_leak",
          surface: "pi",
          frame_id: getAttachmentRuntime().activeFrameId,
          payload: {
            leak_classes: detectedLeakClasses,
            preview: assistantOutput.slice(0, 280),
          },
        });
        queueTraceTelemetry({
          event_type: "visible_output_leak_detected",
          turn_id: `pi-turn-${getTurnCount()}`,
          frame_id: getAttachmentRuntime().activeFrameId,
          surface: "pi",
          leak_classes: detectedLeakClasses,
        });
      }

      const scopeFailures = detectScopeFailureSignals({
        askText: getAttachmentRuntime().currentAsk?.text || "",
        askKind: getAttachmentRuntime().currentAsk?.kind || "unknown",
        scopeKind: getAttachmentRuntime().queryScope?.scopeKind || "mission_carryover",
        assistantOutput,
        leakClasses: detectedLeakClasses,
      });
      const scopeTraceBase = {
        turn_id: `pi-turn-${getTurnCount()}`,
        frame_id: getAttachmentRuntime().activeFrameId,
        surface: "pi",
        ask_kind: getAttachmentRuntime().currentAsk?.kind || "unknown",
        scope_kind: getAttachmentRuntime().queryScope?.scopeKind || "mission_carryover",
        carryover_policy: getAttachmentRuntime().queryScope?.carryoverPolicy || "allow_if_relevant",
      };
      if (scopeFailures.length || detectedLeakClasses.length) {
        refreshTrajectoryClarityLifecycle("failure_or_degradation", getSessionCwd() || process.cwd()).catch(
          () => null
        );
      }
      if (scopeFailures.length === 0) {
        queueTraceTelemetry({
          event_type: "scope_verified",
          ...scopeTraceBase,
          verified: true,
          excluded_context_reason: getAttachmentRuntime().excludedContext?.reason || "none",
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
        await checkpointDiscontinuity("context_overflow", {
          active_object_refs: ["provider_error_text:context_length_exceeded"],
        });
      }

      const expectedActionType = getActiveWorkpointPacket()?.action_intent?.action_type;
      if (expectedActionType && assistantOutput.trim()) {
        focusaFetch("/workpoint/drift-check", {
          method: "POST",
          body: JSON.stringify({
            latest_action: assistantOutput.slice(0, 2000),
            expected_action_type: expectedActionType,
            emit: true,
          }),
        })
          .then((drift: any) => {
            queueTraceTelemetry({
              event_type: drift?.drift_detected ? "workpoint_drift_detected" : "workpoint_drift_checked",
              turn_id: `pi-turn-${getTurnCount()}`,
              frame_id: getAttachmentRuntime().activeFrameId,
              surface: "pi",
              workpoint_id: drift?.workpoint_id || getActiveWorkpointPacket()?.workpoint_id,
              expected_action_type: expectedActionType,
              drift_detected: Boolean(drift?.drift_detected),
              next_step_hint: drift?.next_step_hint,
            });
          })
          .catch(() => {
            queueTraceTelemetry({
              event_type: "workpoint_drift_check_unavailable",
              turn_id: `pi-turn-${getTurnCount()}`,
              frame_id: getAttachmentRuntime().activeFrameId,
              surface: "pi",
              expected_action_type: expectedActionType,
            });
          });
      }

      const promptTokens = ev.usage?.inputTokens || ev.usage?.input || 0;
      const completionTokens = ev.usage?.outputTokens || ev.usage?.output || 0;
      getAttachmentRuntime().currentTaskProviderInputTokens += promptTokens;
      getAttachmentRuntime().currentTaskProviderOutputTokens += completionTokens;
      getAttachmentRuntime().currentTaskOutputTokenEstimate += estimateTokens(assistantOutput);

      focusaPost("/turn/complete", {
        turn_id: `pi-turn-${getTurnCount()}`,
        frame_id: getAttachmentRuntime().activeFrameId,
        assistant_output: assistantOutput,
        artifacts: [],
        errors: [],
        prompt_tokens: promptTokens,
        completion_tokens: completionTokens,
        tokens: {
          input: promptTokens,
          output: completionTokens,
          cache_read: ev.usage?.cacheReadInputTokens || 0,
          cache_write: ev.usage?.cacheCreationInputTokens || 0,
        },
      });
    }

    const {
      inputTokens: usageInputTokens,
      cacheReadTokens: usageCacheReadTokens,
      cacheWriteTokens: usageCacheWriteTokens,
    } = normalizeCacheUsage(ev.usage || ev.message?.usage);
    const selectedModel = ev.model?.id || ev.message?.model?.id || ev.message?.model || ev.model || "unknown";
    const selectedProvider =
      ev.provider?.id || ev.message?.provider?.id || ev.message?.provider || ev.provider || "unknown";
    const cacheObservation = cacheSafetyMonitor.observeUsage({
      sessionKey: cacheSessionKey(),
      provider: String(selectedProvider),
      model: String(selectedModel),
      inputTokens: usageInputTokens,
      cacheReadTokens: usageCacheReadTokens,
      cacheWriteTokens: usageCacheWriteTokens,
      layoutMode:
        getAttachmentRuntime().cfg?.cacheSafePromptLayoutEnabled === false
          ? "legacy_prepend"
          : "cache_safe_tail",
    });
    if (cacheObservation) emitCacheSafetyObservation(cacheObservation, ctx);

    // §33.4: Flush batched tool usage
    if (getAttachmentRuntime().focusaAvailable && getToolUsageBatch().length) {
      getAttachmentRuntime().currentTaskToolCalls += getToolUsageBatch().length;
      focusaPost("/telemetry/tool-usage", {
        turn_id: `pi-turn-${getTurnCount()}`,
        tools: getToolUsageBatch(),
      });
      queueTraceTelemetry({
        event_type: "tools_invoked",
        turn_id: `pi-turn-${getTurnCount()}`,
        frame_id: getAttachmentRuntime().activeFrameId,
        surface: "pi",
        tools: getToolUsageBatch(),
      });
      resetToolUsageBatch();
    }

    // §37.3 + §10.4: Widget with all badges
    const w: string[] = [];
    const liveFocus = getCachedFocusState();
    const snapshot = getEffectiveFocusSnapshot(liveFocus?.fs);
    if (snapshot.decisions.length) w.push(`📌 ${snapshot.decisions.length} decisions`);
    if (snapshot.constraints.length) w.push(`🔒 ${snapshot.constraints.length} constraints`);
    if (snapshot.failures.length) w.push(`⚠️ ${snapshot.failures.length} failures`);
    if (getAttachmentRuntime().wbmEnabled)
      w.push(getAttachmentRuntime().wbmDeep ? "⚡ WBM deep" : "🤖 WBM on");
    if (getAttachmentRuntime().currentTier && typeof getAttachmentRuntime().currentContextPct === "number") {
      const label = contextTierLabel(getAttachmentRuntime().currentTier);
      w.push(`📦 Context ${getAttachmentRuntime().currentContextPct.toFixed(0)}% ${label}`);
    }
    // §10.4: Degraded-context badge
    if (!getAttachmentRuntime().focusaAvailable) w.push("⚪ degraded");
    // §10.4: Thesis snippet
    if (liveFocus?.frame?.thread_thesis) w.push(`🎯 ${liveFocus.frame.thread_thesis.slice(0, 50)}`);
    // §30: Metacognitive indicator
    if (getAttachmentRuntime().lastMetacogEvent) w.push(`✨ ${getAttachmentRuntime().lastMetacogEvent}`);
    const workRailWidget = workRailSnapshotFromPacket(getActiveWorkpointPacket());
    workRailWidget.badges = w;
    const asciiWorkRail = process.env.FOCUSA_ASCII_UI === "1" || process.env.TERM === "dumb";
    // Pi ExtensionContext exposes hasUI, not a runtime mode discriminator.
    // Keep widgets out of print/RPC surfaces while remaining compatible across Pi builds.
    if (ctx.hasUI) {
      ctx.ui.setWidget("focusa", (_tui, theme) => ({
        render(width: number) {
          return renderWorkRailWidget(
            workRailWidget,
            width,
            {
              accent: (text) => theme.fg("accent", text),
              dim: (text) => theme.fg("dim", text),
              good: (text) => theme.fg("accent", text),
            },
            asciiWorkRail
          );
        },
        invalidate() {},
      }));
    } else {
      const plain = {
        accent: (text: string) => text,
        dim: (text: string) => text,
        good: (text: string) => text,
      };
      ctx.ui.setWidget("focusa", renderWorkRailWidget(workRailWidget, 80, plain, true));
    }

    // §34.2C: Update Focus State on significant progress
    if (getAttachmentRuntime().focusaAvailable && getAttachmentRuntime().activeFrameId) {
      const hasSignificant =
        getAttachmentRuntime().localDecisions.length > 0 ||
        getAttachmentRuntime().localConstraints.length > 0 ||
        getAttachmentRuntime().localFailures.length > 0;
      if (hasSignificant) {
        await pushDelta({
          decisions: getAttachmentRuntime().localDecisions.slice(-5),
          constraints: getAttachmentRuntime().localConstraints.slice(-5),
          failures: getAttachmentRuntime().localFailures.slice(-3),
        }).catch(() => null);
      }
    }

    flushTraceTelemetryBatch("turn_end");

    // §5.12: Capture recent-turn slice for cross-agent ring buffer
    captureRecentTurnSlice(assistantOutput);

    // §20: Compaction tier check
    await checkCompactionTier(ctx);
    // §21: Micro-compact check
    await checkMicroCompact();
  });

  // ── message_update (§36.1 streaming delta) ────────────────────────────────
  pi.on("message_update", async (event, _ctx) => {
    if (!getAttachmentRuntime().focusaAvailable) return;
    const fullText = extractText((event as any).message?.content);
    if (getTurnCount() % 10 !== 0 && fullText.length - getLastStreamLen() < 500) return;
    const delta = fullText.slice(getLastStreamLen());
    if (!delta) return;
    setLastStreamLen(fullText.length);
    focusaPost("/turn/append", { turn_id: `pi-turn-${getTurnCount()}`, delta: delta.slice(-500) });
  });

  // ── model_select (§37.8) ──────────────────────────────────────────────────
  pi.on("model_select", async (event, _ctx) => {
    if (!getAttachmentRuntime().focusaAvailable) return;
    const model = (event as any).model;
    getAttachmentRuntime().activeContextWindow =
      model?.contextWindow || getAttachmentRuntime().activeContextWindow;
    // §37.8: Wire model change to Focusa with frame context
    await checkpointDiscontinuity("model_switch", { active_object_refs: [model?.id || "unknown-model"] });
    focusaPost("/focus-gate/ingest-signal", {
      signal_type: "model_change",
      surface: "pi",
      frame_id: getAttachmentRuntime().activeFrameId,
      payload: {
        model_id: model?.id || "unknown",
        context_window: model?.contextWindow || getAttachmentRuntime().activeContextWindow,
      },
    });
    // §5.12: force dynamic newest-turn context re-emit and reset cache comparison
    // because model/provider discontinuities are not prefix regressions.
    getAttachmentRuntime().lastRecentTurnsSliceTurn = -1;
    cacheSafetyMonitor.resetForDiscontinuity(cacheSessionKey());
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
    if (getAttachmentRuntime().wbmEnabled && !getAttachmentRuntime().wbmNoCatalogue) {
      const messages = (event as any).messages || [];
      catalogueFromMessages(messages).catch(() => {});
    }

    // Long session detection
    const elapsed = (Date.now() - getAttachmentRuntime().sessionStartTime) / 60_000;
    if (elapsed > 45 && !getLongSessionSignaled()) {
      setLongSessionSignaled(true);
      if (getAttachmentRuntime().focusaAvailable) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "long_session",
          surface: "pi",
          payload: { minutes: Math.round(elapsed), turns: getTurnCount() },
        });
      }
    }

    // Tool error rate detection
    const recentErrors = getCompilationErrors().filter((t) => Date.now() - t < 300_000);
    if (recentErrors.length >= 3) {
      ctx.ui.notify(
        `⚠️ ${recentErrors.length} compilation errors in 5 min — consider a different approach`,
        "warning"
      );
      if (getAttachmentRuntime().focusaAvailable) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "error_rate_high",
          surface: "pi",
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
    const cfg = getAttachmentRuntime().cfg;

    // §36.2: Error signals
    if (isError && getAttachmentRuntime().focusaAvailable) {
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "tool_error",
        surface: "pi",
        payload: { tool: toolName, error: content.slice(0, 500) },
      });
    }

    if (isError && /compil|tsc|typecheck|build|lint/i.test(toolName + " " + content.slice(0, 200))) {
      pushCompilationError(Date.now());
    }

    const targetRefs = [ev.params?.path, ev.input?.path, ev.params?.url, ev.input?.url]
      .map((value: any) => String(value || "").trim())
      .filter(Boolean)
      .slice(0, 8);
    const projectHintText = [
      toolName,
      ev.params?.path,
      ev.input?.path,
      ev.params?.cwd,
      ev.input?.cwd,
      ev.params?.command,
      ev.input?.command,
      ...targetRefs,
    ]
      .map((value: any) => String(value || "").trim())
      .filter(Boolean)
      .join(" ");
    observeProjectThreadHintsFromText(
      projectHintText,
      `pi-turn-${getTurnCount()}`,
      "tool_evidence",
      `tool=${toolName || "unknown_tool"}`
    );

    if (getAttachmentRuntime().focusaAvailable) {
      focusaPost("/ontology/tool-result-proposals", {
        tool_name: toolName || "unknown_tool",
        status: isError ? "failed" : "completed",
        ok: !isError,
        target_refs: targetRefs,
        evidence_refs: [],
        workpoint_id:
          getActiveWorkpointPacket()?.workpoint_id ||
          getActiveWorkpointPacket()?.workpoint?.workpoint_id ||
          null,
        action_intent: getAttachmentRuntime().currentAsk?.text || null,
        summary: content.slice(0, 500),
        error: isError ? content.slice(0, 500) : null,
        emit_proposals: false,
      });
    }

    // §7.4 + §33.3: ECS externalization — check BOTH thresholds, REPLACE content
    const byteThreshold = cfg?.externalizeThresholdBytes || 8192;
    const tokenThreshold = cfg?.externalizeThresholdTokens || 800;
    const tokens = estimateTokens(content);
    const pressure = recordToolOutputPressure(toolName, content.length, tokens);
    if (pressure.recapRequired) {
      queueTraceTelemetry({
        event_type: "visible_recap_required",
        turn_id: `pi-turn-${getTurnCount()}`,
        frame_id: getAttachmentRuntime().activeFrameId,
        surface: "pi",
        reason: pressure.recapReason,
        tool_output_bytes: pressure.totalBytes,
        tool_output_tokens: pressure.totalTokens,
        tool_output_results: pressure.resultCount,
        latest_report_summary_ref: getLatestReportSummary()?.handle || "none",
      });
    }
    if (
      (content.length > byteThreshold || tokens > tokenThreshold) &&
      getAttachmentRuntime().focusaAvailable
    ) {
      const handle = await focusaFetch("/ecs/store", {
        method: "POST",
        body: JSON.stringify({
          kind: "text",
          label: `${toolName}-output-${Date.now()}`,
          content: content.slice(0, 32_000),
          surface: "pi",
          turn_id: `pi-turn-${getTurnCount()}`,
        }),
      });
      if (handle?.id) {
        // §33.3: REPLACE content with handle reference
        // §7.4: Also cache locally so handles resolve even if Focusa is temporarily down
        storeEcsArtifact("text", content);
        return {
          content: [
            {
              type: "text",
              text:
                `[HANDLE:text:${handle.id} "${toolName} output" (${content.length} bytes, ~${tokens} tokens)]\n` +
                formatHandleTrajectorySummary(handle) +
                `Use /focusa-rehydrate ${handle.id} to retrieve full content.\n\n` +
                content.slice(0, 1000) +
                (content.length > 1000 ? "\n...[truncated, full content in ECS]" : ""),
            },
          ],
        };
      }
    }

    // §7.4 + §33.3: If Focusa unavailable but content still exceeds threshold,
    // store locally so the handle resolves without hitting Focusa.
    if (
      !getAttachmentRuntime().focusaAvailable &&
      (content.length > byteThreshold || tokens > tokenThreshold)
    ) {
      const localId = storeEcsArtifact("text", content);
      return {
        content: [
          {
            type: "text",
            text:
              `[HANDLE:text:${localId} "${toolName} output" (${content.length} bytes, ~${tokens} tokens)]\nFocusa offline — content cached locally. Use /focusa-rehydrate ${localId} when available.\n\n` +
              content.slice(0, 500) +
              (content.length > 500 ? "\n...[truncated]" : ""),
          },
        ],
      };
    }

    // §34.2D: File churn tracking
    if (toolName === "edit" || toolName === "write") {
      const path = ev.params?.path || ev.input?.path || "";
      if (path) {
        incrementFileEditCount(path);
        if (getFileEditCounts()[path] >= 5 && getAttachmentRuntime().focusaAvailable) {
          focusaPost("/focus-gate/ingest-signal", {
            signal_type: "file_churn",
            surface: "pi",
            payload: { path, count: getFileEditCounts()[path] },
          });
        }
      }
    }
  });

  // ── tool_call (§33.4 batched usage) ───────────────────────────────────────
  pi.on("tool_call", async (event, _ctx) => {
    pushToToolUsageBatch((event as any).toolName || (event as any).name || "");
  });
}
