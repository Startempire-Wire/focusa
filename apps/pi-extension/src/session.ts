// Session lifecycle events — ONE handler per event type (merged)
// Spec: §11 (outage audit + backoff), §30 (SSE metacog), §34.2A (instance),
//        §35.1 (auto-frame), §36.4 (resume), §36.5 (fork/tree), §37.5 (flags),
//        §35.8 (session display ownership), §37.9 (Context Core), §37.10 (cross-surface SSE),
//        §38.3 (health toggle)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, focusaPost, checkFocusa, kickstartFocusaDaemon, persistState, persistAuthoritativeState, getFocusState, createPiFrame, ensurePiFrame, classifyCurrentAsk, isNonTaskStatusLikeText, isGenericPiFrameForCwd, trimFrameText, stripQuotedFocusaContext, ensureContinuityId, adoptPersistedContinuityForSession, isProjectRootAuthoritySafe, isWorkpointPacketScopedToCurrentSession, normalizeWorkpointResumePacketEnvelope, refreshTrajectoryClarityLifecycle, stampWorkpointPacketForCurrentPiSession, resetPiSessionScopedState, adoptPiProjectRoot, normalizeProjectRoot, confirmPiProjectRoot, projectRootConfirmationRequired, projectRootConfirmationSummary } from "./state.js";
import { pushDelta } from "./tools.js";

// §30 + §37.10: SSE connection for metacognitive + cross-surface events
let sseAbort: AbortController | null = null;
let sseReconnectTimer: ReturnType<typeof setTimeout> | null = null;


function vitalPromptSurfaceEnabled(surface: string): boolean {
  const raw = String(S.cfg?.vitalInfoPromptSurfaces || "project_root,workpoint,trajectory");
  return raw.split(",").map((part) => part.trim()).includes(surface);
}

type WorkpointDraft = { label?: string; mission: string; next_slice: string; action_type?: string; target_ref?: string; confidence?: "low" | "medium" | "verified" };

async function ensureLowConfidenceWorkpoint(reason: string, draft?: WorkpointDraft): Promise<void> {
  if (!S.focusaAvailable) return;
  if (!isProjectRootAuthoritySafe(S.sessionCwd || process.cwd())) return;
  const mission = draft?.mission || S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || S.lastFocusSnapshot.currentFocus;
  const nextSlice = draft?.next_slice || S.lastFocusSnapshot.currentFocus || S.activeFrameGoal || S.currentAsk?.text;
  if (!mission && !nextSlice) return;
  await focusaFetch("/workpoint/checkpoint", {
    method: "POST",
    body: JSON.stringify({
      mission: mission || "Pi session resume",
      next_slice: nextSlice || "Resume from low-confidence session state and immediately refine Workpoint.",
      checkpoint_reason: reason === "session_start" ? "session_start" : "session_resume",
      confidence: draft?.confidence || "low",
      canonical: true,
      promote: true,
      continuity_id: ensureContinuityId(S.sessionCwd || process.cwd()),
      session_id: S.sessionFrameKey,
      project_root: S.sessionCwd || process.cwd(),
      source_turn_id: `pi-turn-${S.turnCount}`,
      action_intent: { action_type: draft?.action_type || "resume_workpoint", target_ref: draft?.target_ref || S.activeFrameId || S.sessionFrameKey || "pi-session", verification_hooks: ["low-confidence checkpoint created because no active workpoint existed"], status: "needs_refinement" },
    }),
  }).catch(() => null);
}

async function refreshSessionWorkpointPacket(reason: string): Promise<void> {
  if (!S.focusaAvailable) return;
  if (!isProjectRootAuthoritySafe(S.sessionCwd || process.cwd())) {
    S.activeWorkpointPacket = null;
    S.activeWorkpointSummary = "";
    return;
  }
  try {
    const packet = await focusaFetch("/workpoint/resume", {
      method: "POST",
      body: JSON.stringify({ mode: "compact_prompt", continuity_id: ensureContinuityId(S.sessionCwd || process.cwd()), session_id: S.sessionFrameKey, project_root: S.sessionCwd || process.cwd() }),
    });
    if (packet?.status === "rejected_scope_mismatch") {
      S.activeWorkpointPacket = null;
      S.activeWorkpointSummary = "";
      return;
    }
    if (packet?.status === "completed") {
      const candidate = normalizeWorkpointResumePacketEnvelope(packet);
      if (!isWorkpointPacketScopedToCurrentSession(candidate)) {
        S.activeWorkpointPacket = null;
        S.activeWorkpointSummary = "";
        return;
      }
      S.activeWorkpointPacket = stampWorkpointPacketForCurrentPiSession(candidate);
      S.activeWorkpointSummary = packet.rendered_summary || packet.resume_packet_v2?.rendered_summary || packet.next_step_hint || "";
      focusaPost("/telemetry/trace", {
        event_type: "workpoint_resume_packet_loaded",
        payload: { reason, workpoint_id: packet.workpoint_id, canonical: packet.canonical },
      });
    }
  } catch { /* best effort */ }
}

async function promptForConfirmedProjectRoot(ctx: any, proposedRoot: string, reason: string): Promise<string | null> {
  if (!vitalPromptSurfaceEnabled("project_root")) return proposedRoot;
  if (!projectRootConfirmationRequired(proposedRoot)) return proposedRoot;
  const mode = S.cfg?.vitalInfoPromptMode || "prompt";
  const summary = projectRootConfirmationSummary(proposedRoot);
  focusaPost("/telemetry/trace", { event_type: "pi_vital_project_root_agent_inference_required", payload: { reason, project_root: proposedRoot, summary, mode, session_id: S.sessionFrameKey } });
  return null;
}

function queueProjectIdentityBootstrapTurn(pi: ExtensionAPI, ctx: any, proposedRoot: string, reason: string): void {
  if (!S.focusaAvailable || !vitalPromptSurfaceEnabled("project_root")) return;
  if (!projectRootConfirmationRequired(proposedRoot)) return;
  const key = `project_identity_bootstrap:${S.sessionFrameKey || "no-session"}:${normalizeProjectRoot(ctx?.cwd || proposedRoot || process.cwd())}`;
  if (S.vitalInfoPrompted[key]) return;
  S.vitalInfoPrompted[key] = Date.now();
  persistState();
  const summary = projectRootConfirmationSummary(proposedRoot);
  const prompt = [
    "Focusa auto-bootstrap: infer the correct project_root for this Pi session now.",
    `Current detected root is unsafe/unconfirmed: ${summary}`,
    "Use Focusa tools and repo evidence first: call focusa_project_identity with the best explicit project_root candidate, then use project_summary/summary_lines to orient.",
    "If cwd is broad like /root, inspect the session/project context and likely repo folders before asking.",
    "If multiple plausible project folders remain after inference, ask the operator directly in chat which project folder to bind.",
    "Do not show modal/select/input UI. Do not perform durable project-aware writes until identity is verified.",
  ].join("\n");
  focusaPost("/telemetry/trace", { event_type: "pi_vital_project_root_send_user_message", payload: { reason, project_root: proposedRoot, session_id: S.sessionFrameKey } });
  try {
    pi.sendUserMessage(prompt);
  } catch {
    try { pi.sendUserMessage(prompt, { deliverAs: "followUp" } as any); } catch { /* best effort */ }
  }
}

type TrajectoryGoalDraft = { long_term_goal: string; desired_end_state: string; short_term_goal?: string; current_state?: string; goal_source?: string };

function projectNameFromRoot(projectRoot: string): string {
  return String(projectRoot || "project").split("/").filter(Boolean).pop() || "project";
}

function cleanTrajectorySeed(value: unknown): string {
  return trimFrameText(stripQuotedFocusaContext(String(value || "").replace(/\s+/g, " ").trim()), 220);
}

function currentAskForProject(projectRoot: string): string {
  const ask: any = S.currentAsk;
  if (!ask?.text) return "";
  if (ask.sessionId && ask.sessionId !== S.sessionFrameKey) return "";
  if (ask.projectRoot && adoptPiProjectRoot(ask.projectRoot) !== projectRoot) return "";
  if (ask.continuityId && S.continuityId && ask.continuityId !== S.continuityId) return "";
  return cleanTrajectorySeed(ask.text);
}

function trajectoryClarityForProject(projectRoot: string): any | null {
  const clarity: any = S.lastTrajectoryClarity || null;
  if (!clarity) return null;
  if (clarity.project_root && adoptPiProjectRoot(clarity.project_root) !== projectRoot) return null;
  if (clarity.continuity_id && S.continuityId && clarity.continuity_id !== S.continuityId) return null;
  if (clarity.session_id && S.sessionFrameKey && clarity.session_id !== S.sessionFrameKey && clarity.fallback_prior_project_trajectory !== true) return null;
  return clarity;
}

function scopedWorkpointSeed(projectRoot: string): string {
  const packet: any = isWorkpointPacketScopedToCurrentSession(S.activeWorkpointPacket) ? S.activeWorkpointPacket : null;
  if (!packet) return "";
  if (packet.project_root && adoptPiProjectRoot(packet.project_root) !== projectRoot) return "";
  return cleanTrajectorySeed(packet.mission || packet.next_slice);
}

function trajectoryDraftOptions(projectRoot: string): Array<{ label: string; draft: TrajectoryGoalDraft | null }> {
  const projectName = projectNameFromRoot(projectRoot);
  const ask = currentAskForProject(projectRoot);
  const workpointMission = scopedWorkpointSeed(projectRoot);
  const frameGoal = "";
  const focus = "";
  const seed = workpointMission || ask || `Improve ${projectName}`;
  const short = ask || workpointMission || `Continue ${projectName} work`;
  const current = "Current verified state unclear";
  const repoGoal = `Improve and verify ${projectName} as the active project`;
  return [
    {
      label: `A) Learning-informed project card — bootstrap from ontology + predictions + metacog signals`,
      draft: {
        long_term_goal: `Strengthen ${projectName} project intelligence and next-step quality through ontology-grounded trajectory, prediction, evidence, and metacog loops`,
        desired_end_state: `${projectName} has an up-to-date trajectory hierarchy, project card, evidence-backed next step, evaluated predictions, and condensed reusable lessons`,
        short_term_goal: short,
        current_state: current,
        goal_source: "inferred_context",
      },
    },
    {
      label: `B) Infer from current task — ${short}`,
      draft: {
        long_term_goal: seed,
        desired_end_state: `Completed and verified: ${seed}`,
        short_term_goal: short,
        current_state: current,
        goal_source: "inferred_context",
      },
    },
    {
      label: `C) Project-level default — ${repoGoal}`,
      draft: {
        long_term_goal: repoGoal,
        desired_end_state: `${projectName} has a clear trajectory, active Workpoint, and passing evidence for the current change path`,
        short_term_goal: short,
        current_state: current,
        goal_source: "inferred_context",
      },
    },
    {
      label: "D) Short-term only for now — keep high-level goal broad",
      draft: {
        long_term_goal: repoGoal,
        desired_end_state: `${projectName} remains directionally aligned while the current short-term goal is refined`,
        short_term_goal: short,
        current_state: current,
        goal_source: "inferred_context",
      },
    },
    { label: "E) Skip for now", draft: null },
    { label: "F) Custom edit (typing)", draft: null },
  ];
}

function workpointDraftOptions(projectRoot: string): Array<{ label: string; draft: WorkpointDraft | null }> {
  const projectName = projectNameFromRoot(projectRoot);
  const ask = currentAskForProject(projectRoot);
  const clarity = trajectoryClarityForProject(projectRoot);
  const trajectoryShort = cleanTrajectorySeed(clarity?.short_term_goal || clarity?.active_gap);
  const focus = "";
  const frameGoal = "";
  const mission = ask || trajectoryShort || `Continue ${projectName} work`;
  const next = ask || trajectoryShort || `Identify next useful ${projectName} action`;
  return [
    {
      label: `A) Current task checkpoint — ${mission}`,
      draft: { mission, next_slice: next, action_type: "resume_current_task", target_ref: S.activeFrameId || S.sessionFrameKey || "pi-session", confidence: "low" },
    },
    {
      label: `B) Trajectory gap follow-up — ${trajectoryShort || next}`,
      draft: { mission: trajectoryShort || mission, next_slice: trajectoryShort || next, action_type: "trajectory_gap_followup", target_ref: "trajectory_gap", confidence: "low" },
    },
    {
      label: "C) Verify first — collect proof before changing code",
      draft: { mission: `Verify ${projectName} current state before acting`, next_slice: "Run bounded verification and link evidence before committing to implementation", action_type: "verify_current_state", target_ref: projectRoot, confidence: "low" },
    },
    { label: "D) Skip for now", draft: null },
    { label: "E) Custom edit (typing)", draft: null },
  ];
}

function parseWorkpointEditor(text: string): WorkpointDraft | null {
  const raw = String(text || "").trim();
  if (!raw) return null;
  const value = (label: string) => {
    const match = raw.match(new RegExp(`^${label}:\\s*(.+)$`, "im"));
    return String(match?.[1] || "").trim();
  };
  const mission = value("MISSION");
  const next_slice = value("NEXT_SLICE");
  if (!mission || !next_slice) return null;
  return {
    mission,
    next_slice,
    action_type: value("ACTION_TYPE") || "resume_workpoint",
    target_ref: value("TARGET_REF") || undefined,
    confidence: "low",
  };
}

function parseTrajectoryEditor(text: string): TrajectoryGoalDraft | null {
  const raw = String(text || "").trim();
  if (!raw) return null;
  const value = (label: string) => {
    const match = raw.match(new RegExp(`^${label}:\\s*(.+)$`, "im"));
    return String(match?.[1] || "").trim();
  };
  const long_term_goal = value("LONG_TERM_GOAL");
  const desired_end_state = value("DESIRED_END_STATE");
  if (!long_term_goal || !desired_end_state) return null;
  return {
    long_term_goal,
    desired_end_state,
    short_term_goal: value("SHORT_TERM_GOAL") || undefined,
    current_state: value("CURRENT_STATE") || undefined,
  };
}

async function promptForProjectVerifyIfNeeded(ctx: any, projectRoot: string, reason: string): Promise<boolean> {
  const mode = S.cfg?.vitalInfoPromptMode || "prompt";
  if (!vitalPromptSurfaceEnabled("project_verify") || mode === "off" || !isProjectRootAuthoritySafe(projectRoot)) return true;
  const payload = { cwd: projectRoot, project_root: projectRoot };
  const res = await focusaFetch("/project/verify", { method: "POST", body: JSON.stringify(payload) }).catch(() => null);
  S.lastProjectVerify = res || null;
  persistState();
  if (res?.verification?.verified === true || res?.canonical === true) {
    focusaPost("/telemetry/trace", { event_type: "pi_vital_project_verify_passed", payload: { reason, project_root: projectRoot, status: res?.project_identity?.status || res?.status || "verified" } });
    return true;
  }
  const status = String(res?.project_identity?.status || res?.status || "unknown");
  const recovery = String(res?.verification?.required_recovery || "verify project identity before durable cross-project state writes");
  ctx.ui.setWidget("focusa-vital", ["🧭 Focusa project verify needed", `project_root=${projectRoot}`, `status=${status}; ${recovery}`], { placement: "belowEditor" });
  ctx.ui.notify(`Focusa project verify is not clean for ${projectRoot}: ${status}`, "warning");
  focusaPost("/telemetry/trace", { event_type: "pi_vital_project_verify_failed", payload: { reason, project_root: projectRoot, status, mode } });
  if (mode === "warn_only" || mode === "notify") return false;
  const ok = await ctx.ui.confirm(
    "Focusa project verify needs attention",
    `Project root is confirmed, but project_verify is ${status}. Continue scope-limited with this project_root?`,
  );
  if (ok) {
    ctx.ui.setWidget("focusa-vital", undefined);
    focusaPost("/telemetry/trace", { event_type: "pi_vital_project_verify_operator_continue", payload: { reason, project_root: projectRoot, status } });
  }
  return ok;
}

async function promptForWorkpointIfNeeded(ctx: any, projectRoot: string, reason: string): Promise<boolean> {
  const mode = S.cfg?.vitalInfoPromptMode || "prompt";
  if (!vitalPromptSurfaceEnabled("workpoint") || mode !== "prompt" || !isProjectRootAuthoritySafe(projectRoot) || S.activeWorkpointPacket) return false;
  const mission = S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || S.lastFocusSnapshot.currentFocus;
  const nextSlice = S.lastFocusSnapshot.currentFocus || S.activeFrameGoal || S.currentAsk?.text;
  if (!mission && !nextSlice) return false;
  const key = `workpoint:${projectRoot}:${reason}`;
  if (S.vitalInfoPrompted[key]) return false;
  S.vitalInfoPrompted[key] = Date.now();
  persistState();
  if (typeof ctx.ui?.select !== "function") {
    ctx.ui?.notify?.("Focusa Workpoint prompt skipped: Pi UI select is unavailable.", "warning");
    focusaPost("/telemetry/trace", { event_type: "pi_vital_workpoint_prompt_unavailable", payload: { reason, project_root: projectRoot, missing_ui: "select" } });
    return false;
  }
  const options = workpointDraftOptions(projectRoot);
  const choice = await ctx.ui.select(
    "Focusa Workpoint is missing — choose a checkpoint draft",
    options.map((option) => option.label),
  );
  if (!choice || String(choice).startsWith("D)")) return false;
  let draft = options.find((option) => option.label === choice)?.draft || null;
  if (String(choice).startsWith("E)")) {
    const seed = options[0]?.draft;
    const template = [
      `MISSION: ${seed?.mission || ""}`,
      `NEXT_SLICE: ${seed?.next_slice || ""}`,
      `ACTION_TYPE: ${seed?.action_type || "resume_workpoint"}`,
      `TARGET_REF: ${seed?.target_ref || "pi-session"}`,
    ].join("\n");
    const edited = await ctx.ui.editor("Define Focusa Workpoint", template);
    draft = parseWorkpointEditor(String(edited || ""));
    if (!draft) {
      ctx.ui.notify("Workpoint not saved: MISSION and NEXT_SLICE are required.", "warning");
      return false;
    }
  }
  if (!draft) return false;
  await ensureLowConfidenceWorkpoint(reason, draft);
  await refreshSessionWorkpointPacket(`${reason}_operator_selected_workpoint`);
  return Boolean(S.activeWorkpointPacket);
}

async function promptForTrajectoryIfNeeded(ctx: any, projectRoot: string, reason: string): Promise<void> {
  const mode = S.cfg?.vitalInfoPromptMode || "prompt";
  if (!vitalPromptSurfaceEnabled("trajectory") || mode !== "prompt" || !isProjectRootAuthoritySafe(projectRoot)) return;
  const clarity: any = trajectoryClarityForProject(projectRoot) || {};
  const priorProjectFallbackLoaded = clarity.fallback_prior_project_trajectory === true && Boolean(clarity.long_term_goal || clarity.desired_end_state || clarity.trajectory_id);
  if (priorProjectFallbackLoaded) {
    focusaPost("/telemetry/trace", {
      event_type: "pi_trajectory_prompt_suppressed_prior_project_fallback",
      payload: { reason, project_root: projectRoot, continuity_id: S.continuityId || null, session_id: S.sessionFrameKey || null, trajectory_id: clarity.trajectory_id || null, fallback_source_continuity_id: clarity.fallback_source_continuity_id || null },
    });
    return;
  }
  const status = String(clarity.status || "unknown");
  const action = String(clarity.recommended_action || "unknown");
  const unclear = ["unknown", "unclear", "not_found", "not_set", "missing"].includes(status) || /define_goal|operator_required/.test(action);
  const key = `trajectory:${projectRoot}:${S.continuityId || "no-continuity"}:${S.sessionFrameKey || "no-session"}:${status}:${action}`;
  if (!unclear || S.vitalInfoPrompted[key]) return;
  S.vitalInfoPrompted[key] = Date.now();
  persistState();
  const options = trajectoryDraftOptions(projectRoot);
  const choice = await ctx.ui.select(
    "Focusa trajectory is not set — choose a draft",
    options.map((option) => option.label),
  );
  if (!choice || String(choice).startsWith("D)")) return;
  let parsed = options.find((option) => option.label === choice)?.draft || null;
  if (String(choice).startsWith("E)")) {
    const seed = options[0]?.draft;
    const template = [
      `LONG_TERM_GOAL: ${seed?.long_term_goal || ""}`,
      `DESIRED_END_STATE: ${seed?.desired_end_state || ""}`,
      `SHORT_TERM_GOAL: ${seed?.short_term_goal || ""}`,
      `CURRENT_STATE: ${seed?.current_state || ""}`,
    ].join("\n");
    const edited = await ctx.ui.editor("Define Focusa trajectory", template);
    parsed = parseTrajectoryEditor(String(edited || ""));
    if (!parsed) {
      ctx.ui.notify("Trajectory not saved: LONG_TERM_GOAL and DESIRED_END_STATE are required.", "warning");
      return;
    }
  }
  if (!parsed) return;
  const body = {
    ...parsed,
    project_root: projectRoot,
    continuity_id: ensureContinuityId(projectRoot),
    session_id: S.sessionFrameKey,
    goal_source: parsed.goal_source || "operator_selected_inference",
    operator_confirmed: true,
  };
  const res = await focusaFetch("/trajectory/define-goal", { method: "POST", body: JSON.stringify(body) }).catch(() => null);
  if (res?.canonical === true || res?.persisted === true) {
    ctx.ui.notify("Focusa trajectory defined for this project.", "info");
    await refreshTrajectoryClarityLifecycle(`${reason}_trajectory_defined`, projectRoot);
    persistState();
  } else {
    ctx.ui.notify(`Trajectory define_goal did not persist: ${res?.failure_class || res?.status || "unknown"}`, "warning");
  }
}

function seedCurrentAskFromPersistedState(ctx: any, data: any) {
  const restoredAsk = data?.currentAsk;
  const cleanedRestoredAsk = stripQuotedFocusaContext(restoredAsk?.text || "");
  const cwd = adoptPiProjectRoot(ctx?.cwd || S.sessionCwd || process.cwd());
  if (cleanedRestoredAsk && !isNonTaskStatusLikeText(cleanedRestoredAsk)) {
    if (restoredAsk.sessionId && restoredAsk.sessionId !== S.sessionFrameKey) return;
    if (restoredAsk.projectRoot && adoptPiProjectRoot(restoredAsk.projectRoot) !== cwd) return;
    S.currentAsk = {
      text: trimFrameText(cleanedRestoredAsk, 500),
      kind: restoredAsk.kind || classifyCurrentAsk(cleanedRestoredAsk),
      sourceTurnId: restoredAsk.sourceTurnId || "restored",
      updatedAt: restoredAsk.updatedAt || Date.now(),
      sessionId: restoredAsk.sessionId || S.sessionFrameKey,
      projectRoot: restoredAsk.projectRoot || cwd,
      continuityId: restoredAsk.continuityId || S.continuityId,
    };
    if (data?.queryScope) S.queryScope = data.queryScope;
    return;
  }

  const goal = stripQuotedFocusaContext(String(data?.frameGoal || "").trim());
  const title = String(data?.frameTitle || "").trim();
  if (!goal || isNonTaskStatusLikeText(goal) || isGenericPiFrameForCwd(cwd, title, goal)) return;
  if (!/^Pi (Task|Question|Correction): /.test(title)) return;

  S.currentAsk = {
    text: trimFrameText(goal, 500),
    kind: classifyCurrentAsk(goal),
    sourceTurnId: "restored-frame-goal",
    updatedAt: Date.now(),
    sessionId: S.sessionFrameKey,
    projectRoot: cwd,
    continuityId: S.continuityId,
  };
}

async function ensureActiveFrame(ctx: any, sessionId?: string) {
  return ensurePiFrame(adoptPiProjectRoot(ctx.cwd), sessionId, "pi-auto");
}

async function ensureFocusaSession(ctx: any) {
  const status = await focusaFetch("/status").catch(() => null);
  if (status?.session?.status === "active") return status.session;
  const cwd = adoptPiProjectRoot(ctx.cwd || S.sessionCwd || "pi-workspace");
  return focusaFetch("/session/start", {
    method: "POST",
    body: JSON.stringify({
      adapter_id: "pi",
      workspace_id: cwd,
      project_root: cwd,
      continuity_id: ensureContinuityId(cwd),
    }),
  });
}

function connectSSE() {
  if (sseReconnectTimer) {
    clearTimeout(sseReconnectTimer);
    sseReconnectTimer = null;
  }
  if (sseAbort) sseAbort.abort();
  if (!S.focusaAvailable) return;

  const base = S.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  const controller = new AbortController();
  sseAbort = controller;

  fetch(`${base}/events/stream`, { signal: controller.signal })
    .then(async (res) => {
      if (!res.body) return;
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";
        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          try {
            const evt = JSON.parse(line.slice(6));
            handleSSEEvent(evt);
          } catch { /* malformed SSE */ }
        }
      }
    })
    .catch(() => {
      if (controller.signal.aborted || !S.focusaAvailable) return;
      // §30: "If background work fails, the extension shows nothing (fail silent)"
      // Reconnect with backoff — use same exponential backoff as health checks (§11)
      sseReconnectTimer = setTimeout(() => {
        sseReconnectTimer = null;
        if (S.focusaAvailable) connectSSE();
      }, S.healthBackoffMs);
    });
}

// §30: Metacognitive awareness indicators + §37.10: Cross-surface events
function handleSSEEvent(evt: any) {
  switch (evt.type) {
    case "worker_started":
      S.lastMetacogEvent = "thinking...";
      break;
    case "extraction_complete":
      S.lastMetacogEvent = `extracted ${evt.count || "N"} items`;
      setTimeout(() => { S.lastMetacogEvent = ""; }, 5000);
      break;
    case "thesis_updated":
      S.lastMetacogEvent = "thesis updated";
      setTimeout(() => { S.lastMetacogEvent = ""; }, 5000);
      break;
    case "quality_flag":
      S.lastMetacogEvent = `⚠️ ${evt.message || "quality issue"}`;
      break;
    case "focus_state_updated":
      // §37.10: Cross-surface decision notification
      if (evt.source && evt.source !== "pi" && evt.decisions?.length) {
        S.pi?.exec("echo", [], { timeout: 1 }).catch(() => {}); // no-op to access ctx
      }
      break;
    default:
      break;
  }
}

export function registerSession(pi: ExtensionAPI) {
  // ── session_start — single merged handler ──────────────────────────────────
  pi.on("session_start", async (event, ctx) => {
    S.pi = pi;
    S.sessionStartTime = Date.now();
    const eventSessionId = (event as any).sessionId || `pi-${process.pid}-${Date.now()}`;
    S.sessionFrameKey = eventSessionId;
    S.sessionCwd = adoptPiProjectRoot(ctx.cwd);
    resetPiSessionScopedState("session_start");

    // §37.5: Check CLI flags FIRST
    if (pi.getFlag("--no-focusa")) {
      S.focusaAvailable = false;
      ctx.ui.setStatus("focusa", "⏸️ Focusa disabled");
      return;
    }
    if (pi.getFlag("--wbm")) S.wbmEnabled = true;

    // Health check
    await checkFocusa();

    // §36.4 + §33.5: Restore decisions from Pi session entries.
    // CRITICAL §33.5: Never restore activeFrameId from previous sessions — that
    // points to Wirebot/TEP frames and pollutes Pi sessions with stale Wirebot
    // state. Pi ALWAYS gets its own FRESH frame. Only WBM mode may reuse frames.
    const entries = (event as any).entries || (ctx as any).sessionManager?.getEntries?.() || [];
    for (let i = entries.length - 1; i >= 0; i--) {
      const e = entries[i];
      if ((e.customType === "focusa-wbm-state" || e.customType === "focusa-state") && e.data && String(e.data.sessionId || "") === eventSessionId) {
        // §33.5 + §33.7: restore resumable session metadata and safe local shadow,
        // but do not blindly reuse stale frame identity outside WBM mode.
        S.localDecisions = e.data.decisions || [];
        S.turnCount = e.data.turnCount || 0;
        S.wbmEnabled = e.data.wbmEnabled || S.wbmEnabled;
        S.wbmNoCatalogue = e.data.wbmNoCatalogue || false;
        S.cataloguedDecisions = e.data.cataloguedDecisions || [];
        S.cataloguedFacts = e.data.cataloguedFacts || [];
        S.totalCompactions = e.data.totalCompactions || 0;
        S.lastCompactResumeKey = e.data.lastCompactResumeKey || "";
        S.lastCompactResumeAt = e.data.lastCompactResumeAt || 0;
        S.activeFrameTitle = e.data.frameTitle || "";
        S.activeFrameGoal = e.data.frameGoal || "";
        seedCurrentAskFromPersistedState(ctx, e.data);
        S.lastFocusSnapshot = {
          decisions: e.data.authoritativeDecisions || [],
          constraints: e.data.authoritativeConstraints || [],
          failures: e.data.authoritativeFailures || [],
          intent: e.data.intent || "",
          currentFocus: e.data.currentFocus || "",
        };
        if (e.data.projectRootResolution) S.lastProjectRootResolution = e.data.projectRootResolution;
        if (e.data.lastProjectIdentity) {
          const pi = e.data.lastProjectIdentity;
          const piRoot = pi.project_root ? normalizeProjectRoot(pi.project_root) : "";
          const cwdRoot = normalizeProjectRoot(ctx.cwd);
          S.lastProjectIdentity = piRoot && piRoot === cwdRoot ? pi : null;
        }
        if (e.data.lastTrajectoryClarity) {
          const c = e.data.lastTrajectoryClarity;
          const cRoot = c.project_root ? adoptPiProjectRoot(c.project_root) : "";
          const cwdRoot = adoptPiProjectRoot(ctx.cwd);
          S.lastTrajectoryClarity = (!cRoot || cRoot === cwdRoot) && (!c.session_id || c.session_id === eventSessionId || c.fallback_prior_project_trajectory === true) ? c : null;
        }
        if (e.data.lastProjectVerify) S.lastProjectVerify = e.data.lastProjectVerify;
        if (e.data.latestReportSummary?.handle) S.latestReportSummary = e.data.latestReportSummary;
        if (e.data.toolOutputPressure?.recapRequired) S.toolOutputPressure = e.data.toolOutputPressure;
        if (Array.isArray(e.data.projectSwitchLedger)) S.projectSwitchLedger = e.data.projectSwitchLedger.slice(0, 12);
        if (e.data.vitalInfoPrompted) S.vitalInfoPrompted = e.data.vitalInfoPrompted;
        adoptPersistedContinuityForSession(e.data, eventSessionId, adoptPiProjectRoot(ctx.cwd, e.data.activeWorkpointPacket));
        // Explicitly clear stale pollution — do NOT carry across sessions
        S.localConstraints = [];
        S.localFailures = [];
        break;
      }
    }
    // §33.5: Always NULL out activeFrameId — force-push fresh Pi frame.
    // This prevents Wirebot/TEP frame state from leaking into Pi sessions.
    // WBM mode may override this via --wbm flag above.
    if (!S.wbmEnabled) S.activeFrameId = null;

    if (!S.focusaAvailable) {
      ctx.ui.setStatus("focusa", "📡 Focusa offline");
      return;
    }

    const detectedProjectRoot = adoptPiProjectRoot(ctx.cwd);
    const projectRoot = await promptForConfirmedProjectRoot(ctx, detectedProjectRoot, "session_start");
    if (!projectRoot) {
      focusaPost("/telemetry/trace", { event_type: "pi_session_state_bind_blocked_unconfirmed_project_root", payload: { project_root: detectedProjectRoot, summary: projectRootConfirmationSummary(detectedProjectRoot), session_id: eventSessionId, prompt_mode: S.cfg?.vitalInfoPromptMode || "prompt" } });
      queueProjectIdentityBootstrapTurn(pi, ctx, detectedProjectRoot, "session_start");
      return;
    }
    ensureContinuityId(projectRoot);
    await promptForProjectVerifyIfNeeded(ctx, projectRoot, "session_start");
    await ensureFocusaSession({ ...ctx, cwd: projectRoot });
    await ensureActiveFrame({ ...ctx, cwd: projectRoot }, (event as any).sessionId || `pi-session-${Date.now()}`);
    await refreshSessionWorkpointPacket("session_start");
    await refreshTrajectoryClarityLifecycle("session_start", projectRoot);
    await promptForTrajectoryIfNeeded(ctx, projectRoot, "session_start");
    if (!S.activeWorkpointPacket) {
      const prompted = await promptForWorkpointIfNeeded(ctx, projectRoot, "session_start");
      if (!prompted) {
        await ensureLowConfidenceWorkpoint("session_start");
        await refreshSessionWorkpointPacket("session_start_low_confidence");
      }
      await refreshTrajectoryClarityLifecycle("session_start_low_confidence", projectRoot);
      await promptForTrajectoryIfNeeded(ctx, projectRoot, "session_start_low_confidence");
    }

    // §35.8: Pi owns the session display name (/name, session selector).
    // Focusa may cache its scoped frame title for context/status, but must not call the Pi session naming API.
    const data = await getFocusState().catch(() => null);
    if (data?.frame?.title) {
      S.activeFrameTitle = data.frame.title;
      S.activeFrameGoal = data.frame.goal || S.activeFrameGoal;
    }

    // §37.9: Context Core activity signal + wb me --set pi_active
    focusaPost("/telemetry/activity", { surface: "pi", event: "session_start", cwd: ctx.cwd });
    pi.exec("wb", ["me", "--set", "pi_active=true"]).catch(() => {});

    // §30 + §37.10: Start SSE connection for metacognitive + cross-surface events
    connectSSE();

    // Keep Pi footer task label fresh between explicit commands.
    // Default is event-driven (no periodic polling); polling can be enabled explicitly.
    if (S.footerSyncInterval) clearInterval(S.footerSyncInterval);
    S.footerSyncInterval = null;
    const bridgeSyncMode = S.cfg?.bridgeSyncMode || "event-driven";
    if (bridgeSyncMode === "polling") {
      const footerRefreshMs = Math.max(5_000, S.cfg?.bridgePollMs || 15_000);
      let footerSyncInFlight = false;
      S.footerSyncInterval = setInterval(async () => {
        if (!S.focusaAvailable || footerSyncInFlight) return;
        footerSyncInFlight = true;
        try {
          await getFocusState().catch(() => null);
        } finally {
          footerSyncInFlight = false;
        }
      }, footerRefreshMs);
    }

    // Debounce transient health blips to reduce false "offline" warnings.
    // Require consecutive failures before disabling tools.
    const offlineWarnThreshold = 2;
    let outageMode = false;

    // §38.3 + §11: Health check with exponential backoff via recursive setTimeout
    function scheduleHealthCheck() {
      if (S.healthInterval) clearTimeout(S.healthInterval);
      S.healthInterval = setTimeout(async () => {
        await checkFocusa();

        if (!S.focusaAvailable && !outageMode && S.healthFailCount >= offlineWarnThreshold) {
          // Confirmed outage (not single blip) — preserve tool availability, enter holdover, and kickstart daemon.
          ctx.ui.setStatus("focusa", "🛟 Focusa holdover · restarting");
          ctx.ui.notify(`Focusa daemon unavailable (${S.healthFailCount} checks) — holdover active; kickstarting daemon without restarting session`, "warning");
          if (sseAbort) { sseAbort.abort(); sseAbort = null; }
          outageMode = true;
          const recovered = await kickstartFocusaDaemon("session_health_check");
          if (recovered) {
            await checkFocusa();
            ctx.ui.notify("Focusa daemon kickstarted — session preserved", "info");
          }
        } else if (!S.focusaAvailable && outageMode) {
          ctx.ui.setStatus("focusa", "🛟 Focusa holdover · retrying");
          await kickstartFocusaDaemon("session_health_retry");
        } else if (S.focusaAvailable && outageMode) {
          // Came back — reconnect SSE and reconcile holdover state; tools were never disabled.
          ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
          ctx.ui.notify("Focusa daemon reconnected — holdover reconciled; session preserved", "info");
          await ensureFocusaSession(ctx);
          await ensureActiveFrame(ctx);
          connectSSE();

          // §11/§25.7: Soft resync — reconcile local shadow with Focusa on reconnect
          if (S.activeFrameId) {
            // Push any local shadow accumulated during outage
            if (S.localDecisions.length || S.localConstraints.length || S.localFailures.length) {
              await pushDelta({
                decisions: S.localDecisions.slice(-10),
                constraints: S.localConstraints.slice(-10),
                failures: S.localFailures.slice(-5),
                notes: ["Reconciled after Focusa outage"],
              }).catch(() => null);
            }
            // Fetch fresh state + recent candidates
            const data = await getFocusState();
            if (data?.fs) {
              ctx.ui.notify(`Resync complete — ${data.fs.decisions?.length || 0} decisions, ${data.fs.constraints?.length || 0} constraints`, "info");
            }
            // Fetch recent Focus Gate candidates
            focusaFetch("/focus-gate/candidates?limit=5").then((r: any) => {
              if (r?.candidates?.length) {
                ctx.ui.notify(`Focus Gate: ${r.candidates.length} pending candidates`, "info");
              }
            }).catch(() => {});
          }
          outageMode = false;
        }

        // Schedule next check with (possibly updated) backoff interval
        scheduleHealthCheck();
      }, S.healthBackoffMs);
    }
    scheduleHealthCheck();

    ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
  });

  // ── session_shutdown — single handler (§33.8, §34.2A, §37.9) ──────────────
  pi.on("session_shutdown", async (_event, _ctx) => {
    await persistAuthoritativeState();

    // §37.9: Tell Context Core Pi is no longer active
    S.pi?.exec("wb", ["me", "--set", "pi_active=false"]).catch(() => {});

    // Close SSE
    if (sseReconnectTimer) { clearTimeout(sseReconnectTimer); sseReconnectTimer = null; }
    if (sseAbort) { sseAbort.abort(); sseAbort = null; }

    if (S.focusaAvailable) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ reason: "pi_session_shutdown" }),
      });
    }
    if (S.focusaAvailable) {
      focusaPost("/instance/disconnect", { instance_id: `pi-${process.pid}` });
      focusaPost("/telemetry/activity", { surface: "pi", event: "session_shutdown" });
    }
    if (S.healthInterval) { clearInterval(S.healthInterval); S.healthInterval = null; }
    if (S.footerSyncInterval) { clearInterval(S.footerSyncInterval); S.footerSyncInterval = null; }
  });

  // ── session_before_switch (§37.7) ─────────────────────────────────────────
  pi.on("session_before_switch", async (_event, _ctx) => {
    await persistAuthoritativeState();
    if (S.focusaAvailable && S.activeFrameId) {
      await pushDelta({
        decisions: S.localDecisions.slice(-5),
        constraints: S.localConstraints.slice(-5),
      }).catch(() => null);
    }
    if (S.focusaAvailable) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ reason: "pi_session_switch" }),
      });
    }
  });

  // ── session_switch (§37.7) ────────────────────────────────────────────────
  pi.on("session_switch", async (event, ctx) => {
    const eventSessionId = (event as any).sessionId || `pi-${process.pid}-${Date.now()}`;
    S.sessionFrameKey = eventSessionId;
    S.sessionCwd = adoptPiProjectRoot(ctx.cwd);
    resetPiSessionScopedState("session_switch");

    const switchEntries = (event as any).entries || (ctx as any).sessionManager?.getEntries?.() || [];
    S.forkSuggested = false;
    for (let i = switchEntries.length - 1; i >= 0; i--) {
      if ((switchEntries[i].customType === "focusa-wbm-state" || switchEntries[i].customType === "focusa-state") && switchEntries[i].data && String(switchEntries[i].data.sessionId || "") === eventSessionId) {
        const d = switchEntries[i].data;
        S.localDecisions = d.decisions || [];
        S.localConstraints = d.constraints || [];
        S.localFailures = d.failures || [];
        S.turnCount = d.turnCount || 0;
        S.wbmEnabled = d.wbmEnabled || false;
        S.wbmNoCatalogue = d.wbmNoCatalogue || false;
        S.totalCompactions = d.totalCompactions || 0;
        S.lastCompactResumeKey = d.lastCompactResumeKey || "";
        S.lastCompactResumeAt = d.lastCompactResumeAt || 0;
        S.activeFrameTitle = d.frameTitle || "";
        S.activeFrameGoal = d.frameGoal || "";
        seedCurrentAskFromPersistedState(ctx, d);
        S.lastFocusSnapshot = {
          decisions: d.authoritativeDecisions || [],
          constraints: d.authoritativeConstraints || [],
          failures: d.authoritativeFailures || [],
          intent: d.intent || "",
          currentFocus: d.currentFocus || "",
        };
        if (d.projectRootResolution) S.lastProjectRootResolution = d.projectRootResolution;
        if (d.lastProjectIdentity) {
          const pi = d.lastProjectIdentity;
          const piRoot = pi.project_root ? normalizeProjectRoot(pi.project_root) : "";
          const cwdRoot = normalizeProjectRoot(ctx.cwd);
          S.lastProjectIdentity = piRoot && piRoot === cwdRoot ? pi : null;
        }
        if (d.lastTrajectoryClarity) {
          const c = d.lastTrajectoryClarity;
          const cRoot = c.project_root ? adoptPiProjectRoot(c.project_root) : "";
          const cwdRoot = adoptPiProjectRoot(ctx.cwd);
          S.lastTrajectoryClarity = (!cRoot || cRoot === cwdRoot) && (!c.session_id || c.session_id === eventSessionId || c.fallback_prior_project_trajectory === true) ? c : null;
        }
        if (d.lastProjectVerify) S.lastProjectVerify = d.lastProjectVerify;
        if (d.latestReportSummary?.handle) S.latestReportSummary = d.latestReportSummary;
        if (d.toolOutputPressure?.recapRequired) S.toolOutputPressure = d.toolOutputPressure;
        if (Array.isArray(d.projectSwitchLedger)) S.projectSwitchLedger = d.projectSwitchLedger.slice(0, 12);
        if (d.vitalInfoPrompted) S.vitalInfoPrompted = d.vitalInfoPrompted;
        adoptPersistedContinuityForSession(d, eventSessionId, adoptPiProjectRoot(ctx.cwd, d.activeWorkpointPacket));
        break;
      }
    }

    if (!S.wbmEnabled) S.activeFrameId = null;
    if (S.focusaAvailable) {
      const detectedProjectRoot = adoptPiProjectRoot(ctx.cwd);
      const projectRoot = await promptForConfirmedProjectRoot(ctx, detectedProjectRoot, "session_switch");
      if (!projectRoot) {
        focusaPost("/telemetry/trace", { event_type: "pi_session_switch_bind_blocked_unconfirmed_project_root", payload: { project_root: detectedProjectRoot, summary: projectRootConfirmationSummary(detectedProjectRoot), session_id: eventSessionId, prompt_mode: S.cfg?.vitalInfoPromptMode || "prompt" } });
        queueProjectIdentityBootstrapTurn(pi, ctx, detectedProjectRoot, "session_switch");
        return;
      }
      await promptForProjectVerifyIfNeeded(ctx, projectRoot, "session_resume");
      await ensureFocusaSession({ ...ctx, cwd: projectRoot });
      await ensureActiveFrame({ ...ctx, cwd: projectRoot }, eventSessionId || "unknown");
      await refreshSessionWorkpointPacket("session_switch");
      await refreshTrajectoryClarityLifecycle("session_resume", projectRoot);
      await promptForTrajectoryIfNeeded(ctx, projectRoot, "session_resume");
      if (!S.activeWorkpointPacket) {
        const prompted = await promptForWorkpointIfNeeded(ctx, projectRoot, "session_resume");
        if (!prompted) {
          await ensureLowConfidenceWorkpoint("session_resume");
          await refreshSessionWorkpointPacket("session_switch_low_confidence");
        }
        await refreshTrajectoryClarityLifecycle("session_resume_low_confidence", projectRoot);
        await promptForTrajectoryIfNeeded(ctx, projectRoot, "session_resume_low_confidence");
      }
    }
  });

  // ── session_before_fork (§36.5) ───────────────────────────────────────────
  pi.on("session_before_fork", async (_event, _ctx) => {
    if (S.focusaAvailable) {
      await focusaFetch("/workpoint/checkpoint", {
        method: "POST",
        body: JSON.stringify({
          mission: S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || "Pi fork boundary",
          next_slice: S.lastFocusSnapshot.currentFocus || "Resume from fork WorkpointResumePacket.",
          checkpoint_reason: "fork",
          canonical: true,
          promote: true,
          continuity_id: ensureContinuityId(S.sessionCwd || process.cwd()),
          session_id: S.sessionFrameKey,
          project_root: S.sessionCwd || process.cwd(),
          source_turn_id: `pi-turn-${S.turnCount}`,
          action_intent: { action_type: "resume_workpoint", target_ref: S.activeFrameId || "pi-fork", verification_hooks: ["fork refreshes workpoint"], status: "ready" },
        }),
      }).catch(() => null);
      await refreshSessionWorkpointPacket("fork");
      await refreshTrajectoryClarityLifecycle("handoff_fork", S.sessionCwd || process.cwd());
    }
    await persistAuthoritativeState();
    if (S.focusaAvailable && S.activeFrameId) {
      focusaPost("/focus/update", {
        frame_id: S.activeFrameId,
        project_root: normalizeProjectRoot(S.sessionCwd || process.cwd()),
        continuity_id: ensureContinuityId(S.sessionCwd || process.cwd()),
        turn_id: `pi-turn-${S.turnCount}`,
        delta: { meta: { event: "fork", timestamp: Date.now() } },
      });
    }
  });

  // ── session_fork (§36.5) ──────────────────────────────────────────────────
  pi.on("session_fork", async (_event, _ctx) => {
    // §36.5: Take Focusa snapshot of branch point before fork diverges
    if (S.focusaAvailable && S.activeFrameId) {
      focusaPost("/focus/update", {
        frame_id: S.activeFrameId,
        project_root: normalizeProjectRoot(S.sessionCwd || process.cwd()),
        continuity_id: ensureContinuityId(S.sessionCwd || process.cwd()),
        turn_id: `pi-turn-${S.turnCount}`,
        delta: { meta: { event: "fork", turn_count: S.turnCount, decisions_count: S.localDecisions.length } },
      });
    }
  });

  // ── session_before_tree (§36.5) ───────────────────────────────────────────
  pi.on("session_before_tree", async (_event, _ctx) => {
    await persistAuthoritativeState();
  });
}
