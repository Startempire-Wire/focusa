// All /focusa-* slash commands
// Spec: §10.3 — Commands registry, §34.2E (explain-decision), §34.2F (lineage)
// Plus: §33.5 isolation commands: /focusa-on, /focusa-off, /focusa-reset
import { getSettingsListTheme } from "@earendil-works/pi-coding-agent";
import { Container, Text, SettingsList } from "@earendil-works/pi-tui";
import { currentAttachmentKey, getAttachmentRuntime, focusaFetch, compatibleWorkLoopStatusState, getFocusState, getEffectiveFocusSnapshot, getEcsArtifact, persistState, persistAuthoritativeState, ensurePiFrame, getFocusaAvailable, getActiveFrameId, getTurnCount, getSessionCwd, getContinuityId, isProjectRootAuthoritySafe, getCurrentScopeStore, getTotalCompactions, setCompilationErrors, resetFileEditCounts, } from "./state.js";
import { loadConfig, resolveInteractionMode, saveConfigOverrides, } from "./config.js";
import { buildProjectWorkstreamKey } from "./scoped-state.js";
import { measureNativeSessionPressure, migrateNativeSessionBounded } from "./session-pressure.js";
import { prepareCompactionRollover } from "./compaction.js";
import { dirname, resolve } from "path";
import { MissionCanvasView } from "./mission-canvas-view.js";
import { refreshMissionCanvasWidget } from "./mission-canvas-widget.js";
import { workRailDetailRows, workRailSnapshotFromPacket } from "./work-rail-widget.js";
import { MAX_MISSION_CANVAS_ROWS, projectWorkSurfaces, workSurfaceDetail, workSurfaceLabel, } from "./mission-canvas-model.js";
import { projectSessionInventory, sessionInventoryLabel } from "./mission-canvas-session-inventory.js";
import { rehydrateHandle } from "./rehydrate.js";
async function commandWorkLoopWriterHeaders() {
    const writerId = `pi-${process.pid}`;
    const status = await focusaFetch("/work-loop/status?summary_only=true");
    const partition = status?.execution_partition;
    const token = Number(partition?.fencing_token);
    const expiresAt = Date.parse(String(partition?.lease_expires_at || ""));
    if (compatibleWorkLoopStatusState(status) === "unsupported" ||
        partition?.writer_key !== writerId ||
        partition?.lease_freshness !== "current" ||
        !Number.isSafeInteger(token) ||
        token <= 0 ||
        !(expiresAt > Date.now())) {
        throw new Error("current scoped Work Loop lease is missing, expired, or owned by another Pi writer");
    }
    return {
        "x-focusa-writer-id": writerId,
        "x-focusa-fencing-token": String(token),
    };
}
function nonEmptyLines(items) {
    return (items || []).map((v) => String(v || "").trim()).filter(Boolean);
}
function replayConsumerSurface(payload) {
    const replayPayload = payload?.secondary_loop_replay_consumer || payload;
    const continuityPayload = payload?.secondary_loop_continuity_gate || null;
    const objectiveProfile = payload?.secondary_loop_eval_bundle?.secondary_loop_objective_profile || null;
    const replayStatus = String(replayPayload?.status || (payload ? "ok" : "error"));
    const healthy = !!payload && replayStatus === "ok";
    const pairObserved = healthy && !!replayPayload?.secondary_loop_closure_replay_evidence?.evidence?.current_task_pair_observed;
    const pairLabel = healthy ? (pairObserved ? "observed" : "missing") : "unknown";
    const continuityGateRaw = String(continuityPayload?.state || (healthy ? "open" : "fail-closed"));
    const continuityGate = continuityGateRaw === "open" ? "open" : "fail-closed";
    const continuityFailClosed = continuityGate !== "open";
    const nonClosureObjectiveEvents = objectiveProfile?.non_closure_objective_events != null
        ? Number(objectiveProfile.non_closure_objective_events)
        : null;
    const nonClosureObjectiveRate = objectiveProfile?.non_closure_objective_rate != null
        ? Number(objectiveProfile.non_closure_objective_rate)
        : null;
    return {
        replayStatus,
        pairLabel,
        continuityGate,
        continuityFailClosed,
        nonClosureObjectiveEvents,
        nonClosureObjectiveRate,
    };
}
const WARN_OPTIONS = ["40", "50", "60", "70"];
const COMPACT_OPTIONS = ["50", "60", "70", "75", "80", "85", "90"];
const HARD_OPTIONS = ["75", "85", "92", "95", "97"];
const AUTO_COMPACTION_TOKEN_CAP_OPTIONS = ["128000", "192000", "256000", "384000", "0"];
const AUTO_COMPACTION_RESERVE_TOKEN_OPTIONS = ["8192", "16384", "32768", "65536"];
const AUTO_COMPACTION_RESERVE_PCT_OPTIONS = ["5", "10", "15", "20", "25"];
const AUTO_COMPACTION_COOLDOWN_OPTIONS = ["30000", "60000", "120000", "180000", "300000"];
const WORK_LOOP_PRESET_OPTIONS = ["conservative", "balanced", "push", "audit"];
const WORK_LOOP_TURN_OPTIONS = ["6", "10", "12", "16", "24", "60", "120", "200"];
const WORK_LOOP_WALL_CLOCK_OPTIONS = ["900000", "1200000", "1800000", "3600000", "7200000", "14400000"];
const WORK_LOOP_RETRY_OPTIONS = ["1", "2", "3", "4", "8"];
const WORK_LOOP_COOLDOWN_OPTIONS = ["500", "800", "1000", "1500", "2000"];
const WORK_LOOP_LOW_PRODUCTIVITY_OPTIONS = ["2", "3", "4"];
const WORK_LOOP_FAILURE_OPTIONS = ["2", "3", "4"];
const WORK_LOOP_SAME_SUBPROBLEM_OPTIONS = ["1", "2", "3", "4"];
const WORK_LOOP_HEARTBEAT_OPTIONS = ["1500", "2000", "3000", "5000"];
const VITAL_INFO_PROMPT_MODE_OPTIONS = ["prompt", "warn_only", "off"];
const BOOLEAN_OPTIONS = ["true", "false"];
function nextHigher(options, value) {
    return options.find((v) => Number(v) > value) || options[options.length - 1];
}
function nextLower(options, value) {
    const lower = options.filter((v) => Number(v) < value);
    return lower[lower.length - 1] || options[0];
}
function normalizeTierConfig(draft) {
    if (draft.warnPct >= draft.compactPct)
        draft.compactPct = Number(nextHigher(COMPACT_OPTIONS, draft.warnPct));
    if (draft.compactPct >= draft.hardPct)
        draft.hardPct = Number(nextHigher(HARD_OPTIONS, draft.compactPct));
    if (draft.compactPct >= draft.hardPct)
        draft.compactPct = Number(nextLower(COMPACT_OPTIONS, draft.hardPct));
    if (draft.warnPct >= draft.compactPct)
        draft.warnPct = Number(nextLower(WARN_OPTIONS, draft.compactPct));
}
function rolloverPacketRef(prefix, packet) {
    return String(packet?.packet_ref ||
        packet?.checkpoint_ref ||
        packet?.workpoint_id ||
        packet?.trajectory_id ||
        packet?.mission_packet_ref ||
        packet?.id ||
        `${prefix}:unavailable`);
}
function buildTargetBootstrap(input) {
    const lines = [
        "Focusa rollover target bootstrap (bounded):",
        `source_scope=${input.sourceScope.root_scope.root_path}#${input.sourceScope.continuity_id}`,
        `target_scope=${input.targetScope.root_scope.root_path}#${input.targetScope.continuity_id}`,
        `source_session_id=${input.sourceSessionId}`,
        `target_session_id=${input.targetSessionId}`,
        `checkpoint_ref=${input.checkpointRef}`,
        `workpoint_packet_ref=${input.workpointPacketRef}`,
        `compaction_packet_ref=${input.compactionPacketRef}`,
        `migration_manifest=${input.manifestPath}`,
        "First action: verify Focusa target Workpoint/resume before file/API mutations.",
    ];
    return lines.join("\n").slice(0, 4_000);
}
function renderFocusaContext(data) {
    const { frame, fs } = data;
    const lines = ["# Focusa Context", "", "Rendered live from focusa-pi-bridge current state.", ""];
    if (frame?.title) {
        lines.push(`## Current Focus Frame: ${frame.title}`);
        if (frame?.goal)
            lines.push(`**Goal:** ${frame.goal}`);
        lines.push("");
    }
    const decisions = nonEmptyLines(fs?.decisions);
    if (decisions.length) {
        lines.push("## Active Decisions");
        lines.push(...decisions.map((item) => `- ${item}`));
        lines.push("");
    }
    const constraints = nonEmptyLines(fs?.constraints);
    if (constraints.length) {
        lines.push("## Constraints");
        lines.push(...constraints.map((item) => `- ${item}`));
        lines.push("");
    }
    const currentFocus = String(fs?.current_focus || fs?.current_state || "").trim();
    if (currentFocus) {
        lines.push("## Current Focus");
        lines.push(currentFocus);
        lines.push("");
    }
    const openQuestions = nonEmptyLines(fs?.open_questions);
    if (openQuestions.length) {
        lines.push("## Open Questions");
        lines.push(...openQuestions.map((item) => `- ${item}`));
        lines.push("");
    }
    const nextSteps = nonEmptyLines(fs?.next_steps);
    if (nextSteps.length) {
        lines.push("## Next Steps");
        lines.push(...nextSteps.map((item) => `- ${item}`));
        lines.push("");
    }
    const failures = nonEmptyLines(fs?.failures);
    if (failures.length) {
        lines.push("## Known Failures");
        lines.push(...failures.map((item) => `- ${item}`));
        lines.push("");
    }
    lines.push("---");
    lines.push("Focusa structured context — rendered from live state; follow operator intent first.");
    return lines
        .join("\n")
        .replace(/\n{3,}/g, "\n\n")
        .trim();
}
const sessionInteractionModes = new Map();
const priorInteractionModes = new Map();
export function registerCommands(pi) {
    pi.registerCommand("mission-canvas-mode", {
        description: "Set the durable project interaction mode: canvas, terminal, or headless",
        handler: async (args, ctx) => {
            const requested = String(args || "")
                .trim()
                .toLowerCase();
            const modes = {
                canvas: "canvas-guided",
                "canvas-guided": "canvas-guided",
                terminal: "terminal-guided",
                "terminal-guided": "terminal-guided",
                headless: "headless",
            };
            const mode = modes[requested];
            if (!mode) {
                const current = resolveInteractionMode(getSessionCwd());
                ctx.ui.notify(`Mission Canvas mode: ${current.mode} (source: ${current.source}). Usage: /mission-canvas-mode canvas|terminal|headless`, "info");
                return;
            }
            const saved = saveConfigOverrides(getSessionCwd(), { interactionMode: mode }, "project");
            if (saved.errors.length)
                throw new Error(saved.errors.join("; "));
            refreshMissionCanvasWidget(ctx);
            ctx.ui.notify(`Focusa interaction mode set to ${mode} for this project`, "info");
        },
    });
    pi.registerCommand("mission-canvas-profile", {
        description: "Set the durable Mission Canvas workspace profile and visual variant",
        handler: async (args, ctx) => {
            const [profileArg, variantArg] = String(args || "")
                .trim()
                .toLowerCase()
                .split(/\s+/);
            const profiles = new Set([
                "general",
                "software",
                "legal",
                "markets",
                "research",
                "custom",
            ]);
            const variants = new Set(["default", "high-contrast", "monochrome"]);
            if (!profiles.has(profileArg) ||
                (variantArg && !variants.has(variantArg))) {
                const current = loadConfig(getSessionCwd()).config;
                ctx.ui.notify(`Mission Canvas profile: ${current.missionCanvasWorkspaceProfile} ${current.missionCanvasVisualVariant}. Usage: /mission-canvas-profile general|software|legal|markets|research|custom [default|high-contrast|monochrome]`, "info");
                return;
            }
            const saved = saveConfigOverrides(getSessionCwd(), {
                missionCanvasWorkspaceProfile: profileArg,
                missionCanvasVisualVariant: (variantArg || "default"),
            }, "project");
            if (saved.errors.length)
                throw new Error(saved.errors.join("; "));
            refreshMissionCanvasWidget(ctx);
            ctx.ui.notify(`Mission Canvas profile set to ${saved.config.missionCanvasWorkspaceProfile} · ${saved.config.missionCanvasVisualVariant}`, "info");
        },
    });
    pi.registerCommand("mission-canvas", {
        description: "Open the keyboard-first Focusa Mission Canvas in Pi",
        handler: async (_args, ctx) => {
            const interactionMode = resolveInteractionMode(getSessionCwd());
            if (interactionMode.mode !== "canvas-guided") {
                ctx.ui.notify(`Mission Canvas is disabled by interaction mode: ${interactionMode.mode} (source: ${interactionMode.source})`, "info");
                return;
            }
            const loadModel = async () => {
                const [workpoint, trajectory, workLoop, sessions, silentSessions, surfaces, interviews, closurePackage, artifacts,] = await Promise.all([
                    focusaFetch("/v1/workpoint/resume").catch(() => null),
                    focusaFetch("/v1/trajectory/view").catch(() => null),
                    focusaFetch("/work-loop/status?summary_only=true").catch(() => null),
                    focusaFetch("/v1/session/discover").catch(() => null),
                    focusaFetch("/v1/silent-sessions").catch(() => null),
                    focusaFetch("/v1/mission-canvas/surfaces").catch(() => null),
                    focusaFetch("/v1/interviews/sessions").catch(() => null),
                    focusaFetch("/v1/interviews/closure-package").catch(() => null),
                    focusaFetch("/v1/workspace/artifacts").catch(() => null),
                ]);
                const packet = workpoint?.workpoint ?? workpoint?.resume_packet ?? workpoint ?? {};
                const evidenceRefs = Array.isArray(packet?.verification_records)
                    ? packet.verification_records
                        .slice(0, MAX_MISSION_CANVAS_ROWS)
                        .map((record) => String(record?.evidence_ref ?? record?.result ?? record))
                    : Array.isArray(packet?.evidence_refs)
                        ? packet.evidence_refs.slice(0, MAX_MISSION_CANVAS_ROWS).map(String)
                        : [];
                const projectedSurfaces = projectWorkSurfaces(surfaces);
                const sessionRows = projectSessionInventory(sessions, projectedSurfaces, silentSessions).map(sessionInventoryLabel);
                const surfaceRows = projectedSurfaces.map(workSurfaceLabel);
                const contentionRows = projectedSurfaces
                    .filter((surface) => surface.conflictCount || surface.blockerCount || surface.writerLeaseRef)
                    .map((surface) => `${surface.displayName} · ${surface.conflictCount} conflicts · ${surface.blockerCount} blockers · ${surface.writerLeaseRef || "no writer lease"}`);
                const artifactRows = Array.isArray(artifacts?.artifacts)
                    ? artifacts.artifacts
                        .slice(0, MAX_MISSION_CANVAS_ROWS)
                        .map((artifact) => [artifact?.title ?? artifact?.artifact_id, artifact?.kind, artifact?.evidence_ref]
                        .filter(Boolean)
                        .join(" · "))
                    : [];
                const historyRows = Array.isArray(packet?.verification_records)
                    ? packet.verification_records
                        .slice(0, MAX_MISSION_CANVAS_ROWS)
                        .map((record) => [record?.verified_at ?? record?.created_at, record?.result, record?.evidence_ref]
                        .filter(Boolean)
                        .join(" · "))
                    : [];
                const interviewRows = Array.isArray(interviews?.sessions) ? interviews.sessions : [];
                const activeInterview = interviewRows.find((session) => session?.status === "active") ?? interviewRows[0];
                const presentation = loadConfig(getSessionCwd()).config;
                const workRail = workRailSnapshotFromPacket(workpoint);
                const model = {
                    mission: String(packet?.mission ?? packet?.current_ask ?? "No active Mission Canvas Workpoint"),
                    trajectory: String(trajectory?.short_term_goal ??
                        trajectory?.stg ??
                        trajectory?.long_term_goal ??
                        "No trajectory loaded"),
                    nextAction: String(packet?.next_action ?? packet?.next_slice ?? "Create or resume a canonical Workpoint"),
                    workpointId: String(packet?.workpoint_id ?? ""),
                    workItemId: String(packet?.work_item_id ?? ""),
                    workRailDetails: workRailDetailRows(workRail),
                    projectRoot: String(packet?.project_root ?? getSessionCwd() ?? ""),
                    continuityId: String(packet?.continuity_id ?? getContinuityId() ?? ""),
                    evidenceRefs,
                    blockers: Array.isArray(packet?.blockers)
                        ? packet.blockers.slice(0, MAX_MISSION_CANVAS_ROWS).map(String)
                        : [],
                    sessions: sessionRows,
                    workSurfaces: surfaceRows.length
                        ? surfaceRows
                        : sessionRows.length
                            ? sessionRows
                            : [String(packet?.attachment_id ?? "Current Pi attachment")],
                    workSurfaceDetails: projectedSurfaces.map(workSurfaceDetail),
                    contention: contentionRows,
                    researchArtifacts: artifactRows,
                    history: historyRows,
                    contextStatus: String(trajectory?.current_state ?? packet?.context_status ?? "Context review required"),
                    roleStatus: String(closurePackage?.role_profile?.summary ??
                        activeInterview?.role_summary ??
                        "Role profile not reported"),
                    interviewStatus: String(activeInterview
                        ? `${activeInterview.status ?? "unknown"} · ${activeInterview.session_id ?? "unidentified session"}`
                        : "No durable interview session reported"),
                    specStatus: String(closurePackage?.spec_package?.status ??
                        closurePackage?.status ??
                        packet?.spec_status ??
                        "Spec state not reported"),
                    workLoopStatus: String(workLoop?.status ?? workLoop?.state ?? "Unavailable"),
                    scopeStatus: `${String(workpoint?.status ?? "advisory")} · mode ${interactionMode.mode} (${interactionMode.source})`,
                    workspaceProfile: presentation.missionCanvasWorkspaceProfile,
                    visualVariant: presentation.missionCanvasVisualVariant,
                };
                return model;
            };
            const model = await loadModel();
            await ctx.ui.custom((tui, theme, _kb, done) => new MissionCanvasView(model, theme, () => tui.requestRender(), () => done(undefined), loadModel, (reference) => {
                ctx.ui.setEditorText(reference);
                ctx.ui.notify(`Copied stable Mission Canvas reference: ${reference}`, "info");
            }));
        },
    });
    // /focusa-context (§34.2H runtime render)
    pi.registerCommand("focusa-context", {
        description: "Render current Focusa context inline in the conversation",
        handler: async (_args, ctx) => {
            if (!getFocusaAvailable()) {
                const text = "Focusa offline — no live context available.";
                ctx.ui.notify(text, "warning");
                pi.sendMessage({ customType: "focusa-context", content: text, display: true });
                return;
            }
            let data = await getFocusState();
            if (!data) {
                await ensurePiFrame(ctx.cwd, undefined, "pi-auto-recover");
                data = await getFocusState();
            }
            if (!data) {
                const text = "No active Focusa frame for this Pi session.";
                ctx.ui.notify(text, "info");
                pi.sendMessage({ customType: "focusa-context", content: text, display: true });
                return;
            }
            const rendered = renderFocusaContext(data);
            ctx.ui.notify("Rendered live Focusa context", "info");
            pi.sendMessage({ customType: "focusa-context", content: rendered, display: true });
        },
    });
    pi.registerCommand("focusa-mode", {
        description: "Show or switch Canvas, terminal, or headless interaction mode",
        handler: async (args, ctx) => {
            const attachmentKey = currentAttachmentKey();
            if (!attachmentKey) {
                ctx.ui.notify("Interaction mode unavailable: verified attachment scope required.", "error");
                return;
            }
            const runtime = getAttachmentRuntime(attachmentKey);
            const tokens = String(args || "")
                .trim()
                .split(/\s+/)
                .filter(Boolean);
            const requested = tokens[0] || "status";
            const aliases = {
                canvas: "canvas-guided",
                canvas_guided: "canvas-guided",
                "canvas-guided": "canvas-guided",
                terminal: "terminal-guided",
                terminal_guided: "terminal-guided",
                "terminal-guided": "terminal-guided",
                headless: "headless",
                headless_automation: "headless",
            };
            if (requested === "status") {
                const mode = sessionInteractionModes.get(attachmentKey) || runtime.cfg.interactionMode;
                ctx.ui.notify(`Focusa interaction mode: ${mode}.`, "info");
                return;
            }
            if (requested === "clear") {
                sessionInteractionModes.delete(attachmentKey);
                const prior = priorInteractionModes.get(attachmentKey);
                if (prior)
                    runtime.cfg = { ...runtime.cfg, interactionMode: prior };
                priorInteractionModes.delete(attachmentKey);
                ctx.ui.notify(`Session override cleared; effective mode: ${runtime.cfg.interactionMode}.`, "info");
                return;
            }
            const mode = aliases[requested];
            if (!mode) {
                ctx.ui.notify("Usage: /focusa-mode canvas|terminal|headless|status|clear [--project|--user]", "warning");
                return;
            }
            if (tokens.includes("--project") || tokens.includes("--user")) {
                const scope = tokens.includes("--user") ? "user" : "project";
                try {
                    const saved = saveConfigOverrides(ctx.cwd, { interactionMode: mode }, scope);
                    runtime.cfg = saved.config;
                    sessionInteractionModes.delete(attachmentKey);
                    priorInteractionModes.delete(attachmentKey);
                    ctx.ui.notify(`Focusa ${scope} interaction mode saved: ${mode}.`, "info");
                }
                catch (error) {
                    ctx.ui.notify(`Interaction mode not saved: ${String(error).slice(0, 180)}`, "error");
                }
                return;
            }
            if (!sessionInteractionModes.has(attachmentKey)) {
                priorInteractionModes.set(attachmentKey, runtime.cfg.interactionMode);
            }
            sessionInteractionModes.set(attachmentKey, mode);
            runtime.cfg = { ...runtime.cfg, interactionMode: mode };
            ctx.ui.notify(`Focusa session interaction mode: ${mode}.`, "info");
        },
    });
    // /focusa-settings — native settings UI
    pi.registerCommand("focusa-settings", {
        description: "Open Focusa settings panel",
        handler: async (args, ctx) => {
            const settingsAttachmentKey = currentAttachmentKey();
            if (!settingsAttachmentKey) {
                ctx.ui.notify("Focusa settings unavailable: scoped attachment runtime is missing; no setting was changed.", "error");
                return;
            }
            const settingsRuntime = getAttachmentRuntime(settingsAttachmentKey);
            const simpleProfiles = ["starter", "builder", "hands_off", "audit_safe"];
            const advancedMode = /\badvanced\b/i.test(String(args || ""));
            const otaStatus = await focusaFetch("/update/policy").catch(() => null);
            const effectiveOta = otaStatus?.policy || {};
            let otaEnabled = effectiveOta.enabled === true;
            let otaProfile = effectiveOta.dev_mode_override === true
                ? "dev_auto_all"
                : effectiveOta.mode === "automatic" && effectiveOta.channel === "stable"
                    ? "stable_auto_all"
                    : effectiveOta.mode === "prompt"
                        ? "stable_prompt"
                        : "notify";
            const otaPartsAll = {
                cli: true,
                daemon: true,
                tui: true,
                pi_extension: true,
                menubar: true,
                installer: true,
            };
            const persistOtaPolicy = async () => {
                const profile = otaProfile;
                const result = await focusaFetch("/update/policy", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        enabled: otaEnabled,
                        parts: otaPartsAll,
                        dev_mode: profile === "dev_auto_all",
                        channel: profile === "dev_auto_all" ? "dev" : "stable",
                        mode: profile === "dev_auto_all" || profile === "stable_auto_all"
                            ? "automatic"
                            : profile === "stable_prompt"
                                ? "prompt"
                                : "notify",
                    }),
                });
                const schedulerEnabled = otaEnabled && (profile === "dev_auto_all" || profile === "stable_auto_all");
                const scheduler = result?.status === "completed"
                    ? await focusaFetch("/update/scheduler", {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({
                            enabled: schedulerEnabled,
                            channel: profile === "dev_auto_all" ? "dev" : "stable",
                        }),
                    })
                    : null;
                const allowed = result?.policy?.auto_apply_allowed === true;
                const blockers = Array.isArray(result?.policy?.auto_apply_blocked_until)
                    ? result.policy.auto_apply_blocked_until.join(", ")
                    : "";
                ctx.ui.notify(result?.status === "completed"
                    ? `OTA ${otaEnabled ? "enabled" : "disabled"}: ${profile}; scheduler=${scheduler?.status === "completed" ? (schedulerEnabled ? "on" : "off") : "blocked"}${otaEnabled && !allowed ? ` (blocked: ${blockers || "policy gate"})` : ""}`
                    : `OTA policy update blocked: ${result?.error || result?.failure_class || "daemon unavailable"}`, result?.status === "completed" ? (otaEnabled && !allowed ? "warning" : "info") : "error");
            };
            const draft = {
                contextStatusMode: settingsRuntime.cfg?.contextStatusMode || "actionable",
                interactionMode: settingsRuntime.cfg?.interactionMode || "canvas-guided",
                vitalInfoPromptMode: settingsRuntime.cfg?.vitalInfoPromptMode || "prompt",
                vitalInfoPromptSurfaces: settingsRuntime.cfg?.vitalInfoPromptSurfaces || "project_root,project_verify,workpoint,trajectory",
                operatorStatusBarEnabled: settingsRuntime.cfg?.operatorStatusBarEnabled ?? true,
                operatorStatusVersionEnabled: settingsRuntime.cfg?.operatorStatusVersionEnabled ?? true,
                operatorStatusOtaEnabled: settingsRuntime.cfg?.operatorStatusOtaEnabled ?? true,
                operatorStatusModelUsageEnabled: settingsRuntime.cfg?.operatorStatusModelUsageEnabled ?? true,
                operatorStatusTimeEnabled: settingsRuntime.cfg?.operatorStatusTimeEnabled ?? true,
                operatorStatusDeadlineEnabled: settingsRuntime.cfg?.operatorStatusDeadlineEnabled ?? true,
                operatorStatusPredictionEnabled: settingsRuntime.cfg?.operatorStatusPredictionEnabled ?? true,
                warnPct: settingsRuntime.cfg?.warnPct || 50,
                compactPct: settingsRuntime.cfg?.compactPct || 70,
                hardPct: settingsRuntime.cfg?.hardPct || 85,
                autoCompactionEnabled: settingsRuntime.cfg?.autoCompactionEnabled ?? true,
                autoCompactionTokenCap: settingsRuntime.cfg?.autoCompactionTokenCap ?? 256_000,
                autoCompactionReserveTokens: settingsRuntime.cfg?.autoCompactionReserveTokens ?? 16_384,
                autoCompactionReservePct: settingsRuntime.cfg?.autoCompactionReservePct ?? 10,
                autoCompactionCooldownMs: settingsRuntime.cfg?.autoCompactionCooldownMs ?? 60_000,
                workLoopPreset: settingsRuntime.cfg?.workLoopPreset || "balanced",
                workLoopMaxTurns: settingsRuntime.cfg?.workLoopMaxTurns || 12,
                workLoopMaxWallClockMs: settingsRuntime.cfg?.workLoopMaxWallClockMs || 1_800_000,
                workLoopMaxRetries: settingsRuntime.cfg?.workLoopMaxRetries || 3,
                workLoopCooldownMs: settingsRuntime.cfg?.workLoopCooldownMs || 1_000,
                workLoopAllowDestructiveActions: settingsRuntime.cfg?.workLoopAllowDestructiveActions || false,
                workLoopRequireOperatorForGovernance: settingsRuntime.cfg?.workLoopRequireOperatorForGovernance ?? true,
                workLoopRequireOperatorForScopeChange: settingsRuntime.cfg?.workLoopRequireOperatorForScopeChange ?? true,
                workLoopRequireVerificationBeforePersist: settingsRuntime.cfg?.workLoopRequireVerificationBeforePersist ?? true,
                workLoopMaxConsecutiveLowProductivityTurns: settingsRuntime.cfg?.workLoopMaxConsecutiveLowProductivityTurns || 3,
                workLoopMaxConsecutiveFailures: settingsRuntime.cfg?.workLoopMaxConsecutiveFailures || 3,
                workLoopAutoPauseOnOperatorMessage: settingsRuntime.cfg?.workLoopAutoPauseOnOperatorMessage ?? true,
                workLoopRequireExplainableContinueReason: settingsRuntime.cfg?.workLoopRequireExplainableContinueReason ?? true,
                workLoopMaxSameSubproblemRetries: settingsRuntime.cfg?.workLoopMaxSameSubproblemRetries || 2,
                workLoopStatusHeartbeatMs: settingsRuntime.cfg?.workLoopStatusHeartbeatMs || 5_000,
            };
            const applySimpleProfile = (profile) => {
                if (profile === "starter") {
                    draft.workLoopPreset = "conservative";
                    draft.workLoopMaxTurns = 10;
                    draft.workLoopMaxWallClockMs = 1_200_000;
                    draft.workLoopMaxRetries = 2;
                    draft.workLoopCooldownMs = 1_500;
                    draft.workLoopMaxConsecutiveLowProductivityTurns = 2;
                    draft.workLoopMaxConsecutiveFailures = 2;
                    draft.workLoopMaxSameSubproblemRetries = 1;
                    draft.workLoopStatusHeartbeatMs = 3_000;
                    draft.contextStatusMode = "actionable";
                }
                if (profile === "builder") {
                    draft.workLoopPreset = "balanced";
                    draft.workLoopMaxTurns = 24;
                    draft.workLoopMaxWallClockMs = 3_600_000;
                    draft.workLoopMaxRetries = 3;
                    draft.workLoopCooldownMs = 1_000;
                    draft.workLoopMaxConsecutiveLowProductivityTurns = 3;
                    draft.workLoopMaxConsecutiveFailures = 3;
                    draft.workLoopMaxSameSubproblemRetries = 2;
                    draft.workLoopStatusHeartbeatMs = 2_000;
                    draft.contextStatusMode = "actionable";
                }
                if (profile === "hands_off") {
                    draft.workLoopPreset = "push";
                    draft.workLoopMaxTurns = 120;
                    draft.workLoopMaxWallClockMs = 14_400_000;
                    draft.workLoopMaxRetries = 8;
                    draft.workLoopCooldownMs = 800;
                    draft.workLoopMaxConsecutiveLowProductivityTurns = 4;
                    draft.workLoopMaxConsecutiveFailures = 4;
                    draft.workLoopMaxSameSubproblemRetries = 4;
                    draft.workLoopStatusHeartbeatMs = 1_500;
                    draft.contextStatusMode = "actionable";
                }
                if (profile === "audit_safe") {
                    draft.workLoopPreset = "audit";
                    draft.workLoopMaxTurns = 16;
                    draft.workLoopMaxWallClockMs = 3_600_000;
                    draft.workLoopMaxRetries = 2;
                    draft.workLoopCooldownMs = 1_500;
                    draft.workLoopMaxConsecutiveLowProductivityTurns = 2;
                    draft.workLoopMaxConsecutiveFailures = 2;
                    draft.workLoopMaxSameSubproblemRetries = 1;
                    draft.workLoopStatusHeartbeatMs = 3_000;
                    draft.contextStatusMode = "all";
                }
                draft.workLoopAllowDestructiveActions = false;
                draft.workLoopRequireOperatorForGovernance = true;
                draft.workLoopRequireOperatorForScopeChange = true;
                draft.workLoopRequireVerificationBeforePersist = true;
                // Steering should redirect the loop, not freeze it; hard pauses remain policy-gated.
                draft.workLoopAutoPauseOnOperatorMessage = false;
                draft.workLoopRequireExplainableContinueReason = true;
            };
            const inferSimpleProfile = () => {
                const commonProfileMatches = draft.workLoopAllowDestructiveActions === false &&
                    draft.workLoopRequireOperatorForGovernance === true &&
                    draft.workLoopRequireOperatorForScopeChange === true &&
                    draft.workLoopRequireVerificationBeforePersist === true &&
                    draft.workLoopAutoPauseOnOperatorMessage === false &&
                    draft.workLoopRequireExplainableContinueReason === true;
                if (!commonProfileMatches)
                    return "custom";
                const expected = {
                    starter: {
                        workLoopPreset: "conservative",
                        workLoopMaxTurns: 10,
                        workLoopMaxWallClockMs: 1_200_000,
                        workLoopMaxRetries: 2,
                        workLoopCooldownMs: 1_500,
                        workLoopMaxConsecutiveLowProductivityTurns: 2,
                        workLoopMaxConsecutiveFailures: 2,
                        workLoopMaxSameSubproblemRetries: 1,
                        workLoopStatusHeartbeatMs: 3_000,
                        contextStatusMode: "actionable",
                    },
                    builder: {
                        workLoopPreset: "balanced",
                        workLoopMaxTurns: 24,
                        workLoopMaxWallClockMs: 3_600_000,
                        workLoopMaxRetries: 3,
                        workLoopCooldownMs: 1_000,
                        workLoopMaxConsecutiveLowProductivityTurns: 3,
                        workLoopMaxConsecutiveFailures: 3,
                        workLoopMaxSameSubproblemRetries: 2,
                        workLoopStatusHeartbeatMs: 2_000,
                        contextStatusMode: "actionable",
                    },
                    hands_off: {
                        workLoopPreset: "push",
                        workLoopMaxTurns: 120,
                        workLoopMaxWallClockMs: 14_400_000,
                        workLoopMaxRetries: 8,
                        workLoopCooldownMs: 800,
                        workLoopMaxConsecutiveLowProductivityTurns: 4,
                        workLoopMaxConsecutiveFailures: 4,
                        workLoopMaxSameSubproblemRetries: 4,
                        workLoopStatusHeartbeatMs: 1_500,
                        contextStatusMode: "actionable",
                    },
                    audit_safe: {
                        workLoopPreset: "audit",
                        workLoopMaxTurns: 16,
                        workLoopMaxWallClockMs: 3_600_000,
                        workLoopMaxRetries: 2,
                        workLoopCooldownMs: 1_500,
                        workLoopMaxConsecutiveLowProductivityTurns: 2,
                        workLoopMaxConsecutiveFailures: 2,
                        workLoopMaxSameSubproblemRetries: 1,
                        workLoopStatusHeartbeatMs: 3_000,
                        contextStatusMode: "all",
                    },
                };
                return (simpleProfiles.find((profile) => Object.entries(expected[profile]).every(([key, value]) => draft[key] === value)) || "custom");
            };
            let simpleProfile = inferSimpleProfile();
            const persistDraft = () => {
                try {
                    normalizeTierConfig(draft);
                    const saved = saveConfigOverrides(ctx.cwd, draft, "project");
                    settingsRuntime.cfg = saved.config;
                    if (saved.errors.length)
                        ctx.ui.notify(saved.errors.join("\n"), "warning");
                    else
                        ctx.ui.notify(`Saved Focusa settings → ${saved.path}`, "info");
                    return saved.errors.length === 0;
                }
                catch (error) {
                    ctx.ui.notify(`Focusa setting was not saved; prior configuration remains active. ${String(error?.message || error).slice(0, 180)}`, "error");
                    return false;
                }
            };
            const buildSimpleItems = () => [
                {
                    id: "otaEnabled",
                    label: "Automatic OTA updates",
                    currentValue: String(otaEnabled),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "simpleProfile",
                    label: "Quick profile",
                    currentValue: simpleProfile,
                    values: ["starter", "builder", "hands_off", "audit_safe", "custom"],
                },
                {
                    id: "workLoopMaxTurns",
                    label: "How many turns before pause",
                    currentValue: String(draft.workLoopMaxTurns),
                    values: WORK_LOOP_TURN_OPTIONS,
                },
                {
                    id: "workLoopMaxWallClockMs",
                    label: "Max run time (ms)",
                    currentValue: String(draft.workLoopMaxWallClockMs),
                    values: WORK_LOOP_WALL_CLOCK_OPTIONS,
                },
                {
                    id: "interactionMode",
                    label: "Project interaction mode",
                    currentValue: draft.interactionMode,
                    values: ["canvas-guided", "terminal-guided", "headless"],
                },
                {
                    id: "contextStatusMode",
                    label: "Footer hints",
                    currentValue: draft.contextStatusMode,
                    values: ["off", "actionable", "all"],
                },
                {
                    id: "operatorStatusBarEnabled",
                    label: "Operator status bar",
                    currentValue: String(draft.operatorStatusBarEnabled),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "vitalInfoPromptMode",
                    label: "Vital project info prompt",
                    currentValue: draft.vitalInfoPromptMode,
                    values: VITAL_INFO_PROMPT_MODE_OPTIONS,
                },
                {
                    id: "workLoopRequireVerificationBeforePersist",
                    label: "Require verification before done",
                    currentValue: String(draft.workLoopRequireVerificationBeforePersist),
                    values: BOOLEAN_OPTIONS,
                },
            ];
            const buildAdvancedItems = () => [
                {
                    id: "otaEnabled",
                    label: "Automatic OTA updates",
                    currentValue: String(otaEnabled),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "otaProfile",
                    label: "OTA profile (all surfaces)",
                    currentValue: otaProfile,
                    values: ["dev_auto_all", "stable_auto_all", "stable_prompt", "notify"],
                },
                {
                    id: "interactionMode",
                    label: "Project interaction mode",
                    currentValue: draft.interactionMode,
                    values: ["canvas-guided", "terminal-guided", "headless"],
                },
                {
                    id: "contextStatusMode",
                    label: "Footer context badge",
                    currentValue: draft.contextStatusMode,
                    values: ["off", "actionable", "all"],
                },
                ...[
                    ["operatorStatusBarEnabled", "Operator status bar"],
                    ["operatorStatusVersionEnabled", "Status: Focusa version"],
                    ["operatorStatusOtaEnabled", "Status: OTA state"],
                    ["operatorStatusModelUsageEnabled", "Status: model/provider usage"],
                    ["operatorStatusTimeEnabled", "Status: local time"],
                    ["operatorStatusDeadlineEnabled", "Status: deadline"],
                    ["operatorStatusPredictionEnabled", "Status: next prediction"],
                ].map(([id, label]) => ({
                    id,
                    label,
                    currentValue: String(draft[id]),
                    values: BOOLEAN_OPTIONS,
                })),
                {
                    id: "vitalInfoPromptMode",
                    label: "Vital project info prompt",
                    currentValue: draft.vitalInfoPromptMode,
                    values: VITAL_INFO_PROMPT_MODE_OPTIONS,
                },
                {
                    id: "vitalInfoPromptSurfaces",
                    label: "Vital prompt surfaces",
                    currentValue: draft.vitalInfoPromptSurfaces,
                    values: [
                        "project_root,project_verify,workpoint,trajectory",
                        "project_root",
                        "project_root,project_verify",
                        "project_root,workpoint",
                        "project_root,trajectory",
                        "project_root,project_verify,workpoint,trajectory,evidence,active_object,resource_mode",
                    ],
                },
                {
                    id: "warnPct",
                    label: "Warn threshold %",
                    currentValue: String(draft.warnPct),
                    values: WARN_OPTIONS,
                },
                {
                    id: "compactPct",
                    label: "Auto-compact threshold %",
                    currentValue: String(draft.compactPct),
                    values: COMPACT_OPTIONS,
                },
                {
                    id: "autoCompactionEnabled",
                    label: "Proactive auto-compaction",
                    currentValue: String(draft.autoCompactionEnabled),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "autoCompactionTokenCap",
                    label: "Absolute trigger tokens (0 = off)",
                    currentValue: String(draft.autoCompactionTokenCap),
                    values: AUTO_COMPACTION_TOKEN_CAP_OPTIONS,
                },
                {
                    id: "autoCompactionReserveTokens",
                    label: "Minimum reserve tokens",
                    currentValue: String(draft.autoCompactionReserveTokens),
                    values: AUTO_COMPACTION_RESERVE_TOKEN_OPTIONS,
                },
                {
                    id: "autoCompactionReservePct",
                    label: "Reserve percent",
                    currentValue: String(draft.autoCompactionReservePct),
                    values: AUTO_COMPACTION_RESERVE_PCT_OPTIONS,
                },
                {
                    id: "autoCompactionCooldownMs",
                    label: "Auto-compact cooldown ms",
                    currentValue: String(draft.autoCompactionCooldownMs),
                    values: AUTO_COMPACTION_COOLDOWN_OPTIONS,
                },
                {
                    id: "hardPct",
                    label: "Critical threshold %",
                    currentValue: String(draft.hardPct),
                    values: HARD_OPTIONS,
                },
                {
                    id: "workLoopPreset",
                    label: "Work-loop preset",
                    currentValue: draft.workLoopPreset,
                    values: WORK_LOOP_PRESET_OPTIONS,
                },
                {
                    id: "workLoopMaxTurns",
                    label: "Work-loop max turns",
                    currentValue: String(draft.workLoopMaxTurns),
                    values: WORK_LOOP_TURN_OPTIONS,
                },
                {
                    id: "workLoopMaxWallClockMs",
                    label: "Work-loop max wall clock ms",
                    currentValue: String(draft.workLoopMaxWallClockMs),
                    values: WORK_LOOP_WALL_CLOCK_OPTIONS,
                },
                {
                    id: "workLoopMaxRetries",
                    label: "Work-loop retries",
                    currentValue: String(draft.workLoopMaxRetries),
                    values: WORK_LOOP_RETRY_OPTIONS,
                },
                {
                    id: "workLoopCooldownMs",
                    label: "Work-loop cooldown ms",
                    currentValue: String(draft.workLoopCooldownMs),
                    values: WORK_LOOP_COOLDOWN_OPTIONS,
                },
                {
                    id: "workLoopAllowDestructiveActions",
                    label: "Work-loop allow destructive actions",
                    currentValue: String(draft.workLoopAllowDestructiveActions),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "workLoopRequireOperatorForGovernance",
                    label: "Work-loop require operator for governance",
                    currentValue: String(draft.workLoopRequireOperatorForGovernance),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "workLoopRequireOperatorForScopeChange",
                    label: "Work-loop require operator for scope change",
                    currentValue: String(draft.workLoopRequireOperatorForScopeChange),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "workLoopRequireVerificationBeforePersist",
                    label: "Work-loop require verification before persist",
                    currentValue: String(draft.workLoopRequireVerificationBeforePersist),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "workLoopMaxConsecutiveLowProductivityTurns",
                    label: "Work-loop max low-productivity turns",
                    currentValue: String(draft.workLoopMaxConsecutiveLowProductivityTurns),
                    values: WORK_LOOP_LOW_PRODUCTIVITY_OPTIONS,
                },
                {
                    id: "workLoopMaxConsecutiveFailures",
                    label: "Work-loop max consecutive failures",
                    currentValue: String(draft.workLoopMaxConsecutiveFailures),
                    values: WORK_LOOP_FAILURE_OPTIONS,
                },
                {
                    id: "workLoopAutoPauseOnOperatorMessage",
                    label: "Work-loop auto-pause on operator message",
                    currentValue: String(draft.workLoopAutoPauseOnOperatorMessage),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "workLoopRequireExplainableContinueReason",
                    label: "Work-loop require explainable continue reason",
                    currentValue: String(draft.workLoopRequireExplainableContinueReason),
                    values: BOOLEAN_OPTIONS,
                },
                {
                    id: "workLoopMaxSameSubproblemRetries",
                    label: "Work-loop max same-subproblem retries",
                    currentValue: String(draft.workLoopMaxSameSubproblemRetries),
                    values: WORK_LOOP_SAME_SUBPROBLEM_OPTIONS,
                },
                {
                    id: "workLoopStatusHeartbeatMs",
                    label: "Work-loop status heartbeat ms",
                    currentValue: String(draft.workLoopStatusHeartbeatMs),
                    values: WORK_LOOP_HEARTBEAT_OPTIONS,
                },
            ];
            await ctx.ui.custom((_tui, theme, _kb, done) => {
                const container = new Container();
                container.addChild(new Text(theme.fg("accent", theme.bold(advancedMode ? "Focusa Settings (Advanced)" : "🍎 Focusa Quick Setup")), 1, 1));
                if (!advancedMode) {
                    container.addChild(new Text(theme.fg("dim", "Preset-first setup for beginners. Run /focusa-settings advanced for full controls."), 1, 3));
                }
                const displayedItems = advancedMode ? buildAdvancedItems() : buildSimpleItems();
                const syncDisplayedItems = () => {
                    simpleProfile = inferSimpleProfile();
                    const refreshedItems = advancedMode ? buildAdvancedItems() : buildSimpleItems();
                    const currentValues = new Map(refreshedItems.map((item) => [item.id, item.currentValue]));
                    for (const item of displayedItems) {
                        const currentValue = currentValues.get(item.id);
                        if (currentValue !== undefined)
                            item.currentValue = currentValue;
                    }
                };
                const settingsList = new SettingsList(displayedItems, advancedMode ? 8 : 10, getSettingsListTheme(), (id, newValue) => {
                    if (id === "otaEnabled") {
                        const priorEnabled = otaEnabled;
                        otaEnabled = String(newValue) === "true";
                        void persistOtaPolicy().catch((error) => {
                            otaEnabled = priorEnabled;
                            ctx.ui.notify(`OTA setting was not saved; prior value restored. ${String(error?.message || error).slice(0, 180)}`, "error");
                        });
                        return;
                    }
                    if (id === "otaProfile") {
                        const priorProfile = otaProfile;
                        const priorEnabled = otaEnabled;
                        otaProfile = String(newValue);
                        otaEnabled = otaProfile !== "notify";
                        void persistOtaPolicy().catch((error) => {
                            otaProfile = priorProfile;
                            otaEnabled = priorEnabled;
                            ctx.ui.notify(`OTA setting was not saved; prior value restored. ${String(error?.message || error).slice(0, 180)}`, "error");
                        });
                        return;
                    }
                    const priorDraft = { ...draft };
                    if (id === "simpleProfile") {
                        if (newValue === "custom")
                            return;
                        simpleProfile = String(newValue);
                        applySimpleProfile(simpleProfile);
                        if (!persistDraft())
                            Object.assign(draft, priorDraft);
                        syncDisplayedItems();
                        return;
                    }
                    if (id === "interactionMode")
                        draft.interactionMode = String(newValue);
                    if (id === "contextStatusMode")
                        draft.contextStatusMode = String(newValue);
                    if (id === "vitalInfoPromptMode")
                        draft.vitalInfoPromptMode = String(newValue);
                    if (id === "vitalInfoPromptSurfaces")
                        draft.vitalInfoPromptSurfaces = String(newValue);
                    if (id === "operatorStatusBarEnabled")
                        draft.operatorStatusBarEnabled = newValue === "true";
                    if (id === "operatorStatusVersionEnabled")
                        draft.operatorStatusVersionEnabled = newValue === "true";
                    if (id === "operatorStatusOtaEnabled")
                        draft.operatorStatusOtaEnabled = newValue === "true";
                    if (id === "operatorStatusModelUsageEnabled")
                        draft.operatorStatusModelUsageEnabled = newValue === "true";
                    if (id === "operatorStatusTimeEnabled")
                        draft.operatorStatusTimeEnabled = newValue === "true";
                    if (id === "operatorStatusDeadlineEnabled")
                        draft.operatorStatusDeadlineEnabled = newValue === "true";
                    if (id === "operatorStatusPredictionEnabled")
                        draft.operatorStatusPredictionEnabled = newValue === "true";
                    if (id === "warnPct")
                        draft.warnPct = Number(newValue);
                    if (id === "compactPct")
                        draft.compactPct = Number(newValue);
                    if (id === "hardPct")
                        draft.hardPct = Number(newValue);
                    if (id === "autoCompactionEnabled")
                        draft.autoCompactionEnabled = newValue === "true";
                    if (id === "autoCompactionTokenCap")
                        draft.autoCompactionTokenCap = Number(newValue);
                    if (id === "autoCompactionReserveTokens")
                        draft.autoCompactionReserveTokens = Number(newValue);
                    if (id === "autoCompactionReservePct")
                        draft.autoCompactionReservePct = Number(newValue);
                    if (id === "autoCompactionCooldownMs")
                        draft.autoCompactionCooldownMs = Number(newValue);
                    if (id === "workLoopPreset")
                        draft.workLoopPreset = String(newValue);
                    if (id === "workLoopMaxTurns")
                        draft.workLoopMaxTurns = Number(newValue);
                    if (id === "workLoopMaxWallClockMs")
                        draft.workLoopMaxWallClockMs = Number(newValue);
                    if (id === "workLoopMaxRetries")
                        draft.workLoopMaxRetries = Number(newValue);
                    if (id === "workLoopCooldownMs")
                        draft.workLoopCooldownMs = Number(newValue);
                    if (id === "workLoopAllowDestructiveActions")
                        draft.workLoopAllowDestructiveActions = String(newValue) === "true";
                    if (id === "workLoopRequireOperatorForGovernance")
                        draft.workLoopRequireOperatorForGovernance = String(newValue) === "true";
                    if (id === "workLoopRequireOperatorForScopeChange")
                        draft.workLoopRequireOperatorForScopeChange = String(newValue) === "true";
                    if (id === "workLoopRequireVerificationBeforePersist")
                        draft.workLoopRequireVerificationBeforePersist = String(newValue) === "true";
                    if (id === "workLoopMaxConsecutiveLowProductivityTurns")
                        draft.workLoopMaxConsecutiveLowProductivityTurns = Number(newValue);
                    if (id === "workLoopMaxConsecutiveFailures")
                        draft.workLoopMaxConsecutiveFailures = Number(newValue);
                    if (id === "workLoopAutoPauseOnOperatorMessage")
                        draft.workLoopAutoPauseOnOperatorMessage = String(newValue) === "true";
                    if (id === "workLoopRequireExplainableContinueReason")
                        draft.workLoopRequireExplainableContinueReason = String(newValue) === "true";
                    if (id === "workLoopMaxSameSubproblemRetries")
                        draft.workLoopMaxSameSubproblemRetries = Number(newValue);
                    if (id === "workLoopStatusHeartbeatMs")
                        draft.workLoopStatusHeartbeatMs = Number(newValue);
                    if (!persistDraft())
                        Object.assign(draft, priorDraft);
                    syncDisplayedItems();
                }, () => done(undefined), { enableSearch: true });
                container.addChild(settingsList);
                return {
                    render: (w) => container.render(w),
                    invalidate: () => container.invalidate(),
                    handleInput: (data) => settingsList.handleInput?.(data),
                };
            });
        },
    });
    // /focusa-status (§10.3)
    pi.registerCommand("focusa-rollover", {
        description: "Inspect, dry-run, or execute a bounded native-session rollover: /focusa-rollover inspect|dry-run|execute [output-dir]",
        handler: async (args, ctx) => {
            const [modeRaw, outputRaw] = String(args || "inspect")
                .trim()
                .split(/\s+/, 2);
            const mode = modeRaw || "inspect";
            if (!new Set(["inspect", "dry-run", "execute"]).has(mode)) {
                ctx.ui.notify("Usage: /focusa-rollover inspect | dry-run [output-dir] | execute [output-dir]", "warning");
                return;
            }
            const sourcePath = String(ctx.sessionManager.getSessionFile?.() || "").trim();
            if (!sourcePath) {
                ctx.ui.notify("Focusa rollover blocked: native session file unavailable.", "error");
                return;
            }
            const projectRoot = getSessionCwd();
            const continuityId = String(getContinuityId() || "").trim();
            if (!isProjectRootAuthoritySafe(projectRoot) || !continuityId) {
                ctx.ui.notify("Focusa rollover blocked: typed verified project/workstream scope required.", "error");
                return;
            }
            const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
            const pressure = measureNativeSessionPressure({
                adapter: "pi",
                sessionFile: sourcePath,
                entries: ctx.sessionManager.getEntries?.(),
            });
            if (mode === "inspect") {
                ctx.ui.notify(`Focusa rollover inspect: posture=${pressure.posture} bytes=${pressure.session_bytes} next=${pressure.recommended_action}`, pressure.posture === "normal" ? "info" : "warning");
                return;
            }
            const outputDir = outputRaw
                ? resolve(outputRaw)
                : mode === "execute"
                    ? resolve(dirname(sourcePath), `focusa-rollover-${Date.now()}`)
                    : resolve(dirname(sourcePath), "focusa-rollover-dry-run");
            let preparation = null;
            let targetScope = scope;
            let targetSessionId = "";
            if (mode === "execute") {
                await ctx.waitForIdle();
                preparation = await prepareCompactionRollover();
                if (!preparation.ready) {
                    ctx.ui.notify(`Focusa rollover blocked: ${preparation.reason}; Workpoint, Trajectory, and CompactionMissionPacket checkpoints are required.`, "error");
                    return;
                }
                const targetContinuityId = `rollover-${Date.now().toString(36)}`;
                targetScope = buildProjectWorkstreamKey(projectRoot, targetContinuityId);
                targetSessionId = `pi-rollover-${Date.now().toString(36)}`;
            }
            try {
                const manifest = await migrateNativeSessionBounded({
                    source_path: sourcePath,
                    output_dir: outputDir,
                    scope,
                    mode: mode === "execute" ? "execute" : "dry_run",
                });
                if (mode !== "execute") {
                    ctx.ui.notify(`Focusa rollover dry-run: source=${manifest.source.bytes} bytes id=${manifest.migration_id}`, "info");
                    return;
                }
                const sourceSessionId = String(ctx.sessionManager.getSessionFile?.() || getAttachmentRuntime().sessionFrameKey || "");
                const checkpointRef = rolloverPacketRef("checkpoint", preparation?.workpoint_checkpoint);
                const workpointPacketRef = rolloverPacketRef("workpoint", preparation?.workpoint_checkpoint);
                const compactionPacketRef = rolloverPacketRef("compaction", preparation?.compaction_packet);
                await focusaFetch("/project/session-transfer", {
                    method: "POST",
                    body: JSON.stringify({
                        action: "rollover",
                        rollover_action: "migrate",
                        source_scope: scope,
                        target_scope: targetScope,
                        target_continuity_id: targetScope.continuity_id,
                        source_session_id: sourceSessionId,
                        target_session_id: targetSessionId,
                        checkpoint_ref: checkpointRef,
                        workpoint_packet_ref: workpointPacketRef,
                        compaction_packet_ref: compactionPacketRef,
                        migration_manifest_ref: manifest.manifest_path,
                        seal_source: true,
                    }),
                });
                const materializeTargetWorkpoint = await focusaFetch("/workpoint/rollover/target-materialize", {
                    method: "POST",
                    body: JSON.stringify({
                        source_continuity_id: scope.continuity_id,
                        target_continuity_id: targetScope.continuity_id,
                        source_session_id: sourceSessionId,
                        target_session_id: targetSessionId,
                        project_root: scope.root_scope.root_path,
                        checkpoint_ref: checkpointRef,
                        workpoint_packet_ref: workpointPacketRef,
                        compaction_packet_ref: compactionPacketRef,
                    }),
                });
                if (materializeTargetWorkpoint?.status !== "completed") {
                    ctx.ui.notify(`Focusa rollover blocked: target Workpoint materialization failed (` +
                        `status=${materializeTargetWorkpoint?.status || "unknown"}).`, "error");
                    return;
                }
                const bootstrap = buildTargetBootstrap({
                    sourceScope: scope,
                    targetScope,
                    sourceSessionId,
                    targetSessionId,
                    checkpointRef,
                    workpointPacketRef,
                    compactionPacketRef,
                    manifestPath: manifest.manifest_path || manifest.migration_id,
                });
                // Pi 0.81+ exposes typed transactional replacement on ExtensionCommandContext.
                // Never cast an ordinary event context into this authority boundary.
                const newSessionResult = await ctx.newSession({
                    parentSession: sourceSessionId,
                    setup: async (sessionManager) => {
                        sessionManager.appendMessage?.({
                            role: "user",
                            content: [{ type: "text", text: bootstrap }],
                            timestamp: Date.now(),
                        });
                    },
                    withSession: async (replacementCtx) => {
                        try {
                            const verifyResume = await focusaFetch("/workpoint/resume", {
                                method: "POST",
                                body: JSON.stringify({
                                    mode: "compact_prompt",
                                    project_root: targetScope.root_scope.root_path,
                                    continuity_id: targetScope.continuity_id,
                                    session_id: targetSessionId,
                                }),
                            });
                            const targetWorkpointId = materializeTargetWorkpoint?.workpoint_id ||
                                verifyResume?.body?.workpoint_id ||
                                verifyResume?.body?.resume_packet?.workpoint?.workpoint_id ||
                                verifyResume?.body?.resume_packet?.workpoint_id ||
                                "";
                            await focusaFetch("/project/session-transfer", {
                                method: "POST",
                                body: JSON.stringify({
                                    action: "verify_target",
                                    source_scope: scope,
                                    target_scope: targetScope,
                                    source_session_id: sourceSessionId,
                                    target_session_id: targetSessionId,
                                    target_workpoint_id: targetWorkpointId,
                                    target_resume_canonical: verifyResume?.status === "completed" || verifyResume?.canonical === true,
                                    checkpoint_ref: checkpointRef,
                                    workpoint_packet_ref: workpointPacketRef,
                                    compaction_packet_ref: compactionPacketRef,
                                    migration_manifest_ref: manifest.manifest_path,
                                }),
                            });
                            replacementCtx.ui.notify(`Focusa rollover complete: target=${targetScope.continuity_id}`, "info");
                        }
                        catch (error) {
                            replacementCtx.ui.notify(`Focusa rollover target verification failed: ${error instanceof Error ? error.message : String(error)}`, "error");
                        }
                    },
                });
                if (newSessionResult.cancelled)
                    return;
            }
            catch (error) {
                ctx.ui.notify(`Focusa rollover failed safely; source preserved: ${error instanceof Error ? error.message : String(error)}`, "error");
            }
        },
    });
    pi.registerCommand("focusa-rehydrate", {
        description: "Retrieve bounded content for a Focusa ECS or local output handle",
        handler: async (args, ctx) => {
            try {
                const result = await rehydrateHandle(args, {
                    getLocal: getEcsArtifact,
                    fetchRemote: (path, init) => focusaFetch(path, init),
                });
                pi.sendMessage({
                    customType: "focusa-rehydrate",
                    content: result.content,
                    display: true,
                    details: {
                        handle_id: result.handleId,
                        source: result.source,
                        truncated: result.truncated,
                        original_size: result.originalSize ?? null,
                    },
                }, { deliverAs: "nextTurn" });
                ctx.ui.notify(`Focusa handle ${result.handleId} retrieved from ${result.source}${result.truncated ? " (bounded)" : ""}`, "info");
            }
            catch (error) {
                ctx.ui.notify(error instanceof Error ? error.message : String(error), "error");
            }
        },
    });
    pi.registerCommand("focusa-status", {
        description: "Show Focusa integration status",
        handler: async (_args, ctx) => {
            const up = getFocusaAvailable() ? "✅ Connected" : "❌ Offline";
            const frame = getActiveFrameId() ?? "none";
            const wbm = getAttachmentRuntime().wbmEnabled
                ? getAttachmentRuntime().wbmDeep
                    ? "deep"
                    : getAttachmentRuntime().wbmNoCatalogue
                        ? "on (no-catalogue)"
                        : "on"
                : "off";
            const tier = getAttachmentRuntime().currentTier
                ? ` | Tier: ${getAttachmentRuntime().currentTier.toUpperCase()}`
                : "";
            const compactions = getTotalCompactions() ? ` | Compactions: ${getTotalCompactions()}` : "";
            const focusState = await getFocusState();
            const titleLine = getAttachmentRuntime().activeFrameTitle
                ? `\nTitle: ${getAttachmentRuntime().activeFrameTitle}`
                : "";
            const goalLine = getAttachmentRuntime().activeFrameGoal
                ? `\nGoal: ${getAttachmentRuntime().activeFrameGoal}`
                : "";
            const loop = await focusaFetch("/work-loop");
            const replayPayload = (await focusaFetch("/work-loop/replay/closure-bundle")) ||
                (await focusaFetch("/work-loop/replay/closure-evidence"));
            const replayConsumer = replayConsumerSurface(replayPayload);
            const loopLine = loop
                ? `\nLoop: ${loop.enabled ? "on" : "off"} | Status: ${loop.status} | Project: ${loop.project_status} | Tranche: ${loop.tranche_status}`
                : "";
            const whyLine = loop?.last_continue_reason || loop?.last_blocker_reason
                ? `\nWhy: ${loop.last_continue_reason || loop.last_blocker_reason}`
                : "";
            const budgetLine = loop?.budget_remaining
                ? `\nBudget: retries=${loop.budget_remaining.max_retries} remaining_failure_budget=${loop.budget_remaining.remaining_failure_budget}`
                : "";
            const checkpointLine = loop?.last_checkpoint_id ? `\nCheckpoint: ${loop.last_checkpoint_id}` : "";
            const supervisionLine = loop?.transport?.daemon_supervised_session
                ? `\nSupervision: daemon-owned ${loop.transport.daemon_supervised_session.session_id}`
                : "\nSupervision: none";
            const replayLine = `\nReplay: ${replayConsumer.replayStatus} | pair=${replayConsumer.pairLabel} | continuity_gate=${replayConsumer.continuityGate}`;
            const objectiveLine = replayConsumer.nonClosureObjectiveEvents == null
                ? ""
                : `\nObjectives: non_closure=${replayConsumer.nonClosureObjectiveEvents}${replayConsumer.nonClosureObjectiveRate == null ? "" : ` (${(replayConsumer.nonClosureObjectiveRate * 100).toFixed(1)}%)`}`;
            const snapshot = getEffectiveFocusSnapshot(focusState?.fs);
            const missionLine = snapshot.intent ? `\nMission: ${snapshot.intent}` : "";
            const focusLine = snapshot.currentFocus ? `\nFocus: ${snapshot.currentFocus}` : "";
            const updateNotifications = await focusaFetch("/update/notifications");
            const silentSessionDashboard = await focusaFetch("/silent-sessions/dashboard?limit=20");
            const silentSessions = Array.isArray(silentSessionDashboard?.data?.sessions)
                ? silentSessionDashboard.data.sessions
                : Array.isArray(silentSessionDashboard?.sessions)
                    ? silentSessionDashboard.sessions
                    : [];
            const silentAttention = silentSessions.filter((session) => Boolean(session?.attention));
            const silentLine = `\nSilent sessions: ${silentSessions.length} visible | attention=${silentAttention.length} | source=daemon`;
            const staleUpdateParts = Array.isArray(updateNotifications?.stale_parts)
                ? updateNotifications.stale_parts
                : [];
            const updateLine = staleUpdateParts.length
                ? `\nUpdate: ${updateNotifications?.severity || "warning"} | stale=${staleUpdateParts.join(",")}`
                : "";
            ctx.ui.notify(`Focusa: ${up}\nFrame: ${frame}${titleLine}${goalLine}\nWBM: ${wbm}\nTurns: ${getTurnCount()}${tier}${compactions}` +
                loopLine +
                whyLine +
                budgetLine +
                checkpointLine +
                supervisionLine +
                replayLine +
                objectiveLine +
                missionLine +
                focusLine +
                silentLine +
                updateLine +
                `\n` +
                `Decisions: ${snapshot.decisions.length} | Constraints: ${snapshot.constraints.length} | Failures: ${snapshot.failures.length}` +
                (getAttachmentRuntime().cfg
                    ? `\nConfig: warn=${getAttachmentRuntime().cfg.warnPct}% compact=${getAttachmentRuntime().cfg.compactPct}% hard=${getAttachmentRuntime().cfg.hardPct}% | work-loop=${getAttachmentRuntime().cfg.workLoopPreset}`
                    : ""), "info");
        },
    });
    pi.registerCommand("focus-work", {
        description: "Continuous work loop controls: on|off|pause|resume|stop|status|checkpoint|checkpoints",
        handler: async (args, ctx) => {
            const parts = String(args || "")
                .trim()
                .split(/\s+/)
                .filter(Boolean);
            const sub = String(parts[0] || "status").toLowerCase();
            const rest = parts.slice(1).join(" ").trim();
            if (sub === "on") {
                const payload = {
                    preset: getAttachmentRuntime().cfg?.workLoopPreset || "balanced",
                    policy_overrides: {
                        max_turns: getAttachmentRuntime().cfg?.workLoopMaxTurns,
                        max_wall_clock_ms: getAttachmentRuntime().cfg?.workLoopMaxWallClockMs,
                        max_retries: getAttachmentRuntime().cfg?.workLoopMaxRetries,
                        cooldown_ms: getAttachmentRuntime().cfg?.workLoopCooldownMs,
                        allow_destructive_actions: getAttachmentRuntime().cfg?.workLoopAllowDestructiveActions,
                        require_operator_for_governance: getAttachmentRuntime().cfg?.workLoopRequireOperatorForGovernance,
                        require_operator_for_scope_change: getAttachmentRuntime().cfg?.workLoopRequireOperatorForScopeChange,
                        require_verification_before_persist: getAttachmentRuntime().cfg?.workLoopRequireVerificationBeforePersist,
                        max_consecutive_low_productivity_turns: getAttachmentRuntime().cfg?.workLoopMaxConsecutiveLowProductivityTurns,
                        max_consecutive_failures: getAttachmentRuntime().cfg?.workLoopMaxConsecutiveFailures,
                        auto_pause_on_operator_message: getAttachmentRuntime().cfg?.workLoopAutoPauseOnOperatorMessage,
                        require_explainable_continue_reason: getAttachmentRuntime().cfg?.workLoopRequireExplainableContinueReason,
                        max_same_subproblem_retries: getAttachmentRuntime().cfg?.workLoopMaxSameSubproblemRetries,
                        status_heartbeat_ms: getAttachmentRuntime().cfg?.workLoopStatusHeartbeatMs,
                    },
                };
                const res = await focusaFetch("/work-loop/enable", {
                    method: "POST",
                    headers: { "x-focusa-writer-id": `pi-${process.pid}`, "x-focusa-approval": "approved" },
                    body: JSON.stringify(payload),
                });
                ctx.ui.notify(`focus-work on → ${res?.status || res?.ok || "unknown"}`, "info");
                return;
            }
            if (sub === "pause") {
                const res = await focusaFetch("/work-loop/pause", {
                    method: "POST",
                    headers: { "x-focusa-writer-id": `pi-${process.pid}` },
                    body: JSON.stringify({ reason: "operator pause via /focus-work" }),
                });
                ctx.ui.notify(`focus-work pause → ${res?.status || res?.ok || "unknown"}`, "info");
                return;
            }
            if (sub === "resume") {
                const res = await focusaFetch("/work-loop/resume", {
                    method: "POST",
                    headers: { "x-focusa-writer-id": `pi-${process.pid}` },
                    body: JSON.stringify({ reason: "operator resume via /focus-work" }),
                });
                ctx.ui.notify(`focus-work resume → ${res?.status || res?.ok || "unknown"}`, "info");
                return;
            }
            if (sub === "off" || sub === "stop") {
                const res = await focusaFetch("/work-loop/stop", {
                    method: "POST",
                    headers: { "x-focusa-writer-id": `pi-${process.pid}` },
                    body: JSON.stringify({ reason: `operator ${sub} via /focus-work` }),
                });
                ctx.ui.notify(`focus-work ${sub} → ${res?.status || res?.ok || "unknown"}`, "info");
                return;
            }
            if (sub === "checkpoint") {
                const payload = rest
                    ? { summary: `operator checkpoint via /focus-work: ${rest}` }
                    : { summary: "operator checkpoint via /focus-work" };
                const res = await focusaFetch("/work-loop/checkpoint", {
                    method: "POST",
                    headers: { "x-focusa-writer-id": `pi-${process.pid}` },
                    body: JSON.stringify(payload),
                });
                ctx.ui.notify(`focus-work checkpoint → ${res?.checkpoint_id || res?.status || res?.ok || "unknown"}`, "info");
                return;
            }
            if (sub === "checkpoints") {
                const res = await focusaFetch("/work-loop/checkpoints");
                const checkpoints = Array.isArray(res?.checkpoints) ? res.checkpoints : [];
                const lines = checkpoints
                    .slice(0, 5)
                    .map((c) => `- ${c?.id || "(id?)"}: ${c?.summary || "(no summary)"}`);
                ctx.ui.notify(lines.length
                    ? `Recent checkpoints (${checkpoints.length})\n${lines.join("\n")}`
                    : "No checkpoints available", "info");
                return;
            }
            const fs = await getFocusState();
            const loop = await focusaFetch("/work-loop");
            const replayPayload = (await focusaFetch("/work-loop/replay/closure-bundle")) ||
                (await focusaFetch("/work-loop/replay/closure-evidence"));
            const replayConsumer = replayConsumerSurface(replayPayload);
            const snapshot = getEffectiveFocusSnapshot(fs?.fs);
            const mission = snapshot.intent || "(none)";
            const focus = snapshot.currentFocus || "(none)";
            const objectiveSummary = replayConsumer.nonClosureObjectiveEvents == null
                ? "(n/a)"
                : `${replayConsumer.nonClosureObjectiveEvents}${replayConsumer.nonClosureObjectiveRate == null ? "" : ` (${(replayConsumer.nonClosureObjectiveRate * 100).toFixed(1)}%)`}`;
            ctx.ui.notify(loop
                ? `Loop: ${loop.enabled ? "on" : "off"}\nStatus: ${loop.status}\nProject: ${loop.project_status}\nTranche: ${loop.tranche_status}\nReplay: ${replayConsumer.replayStatus} | pair=${replayConsumer.pairLabel} | continuity_gate=${replayConsumer.continuityGate}\nObjectives: non_closure=${objectiveSummary}\nMission: ${mission}\nFocus: ${focus}\nReason: ${loop.last_continue_reason || loop.last_blocker_reason || "(none)"}\nCheckpoint: ${loop.last_checkpoint_id || "(none)"}\nSupervision: ${loop.transport?.daemon_supervised_session?.session_id || "(none)"}\nPreset: ${loop.policy?.preset || getAttachmentRuntime().cfg?.workLoopPreset || "balanced"}`
                : "Loop status unavailable", "info");
        },
    });
    // /focusa-on (§33.5) — re-enable Focusa writes after /focusa-off
    pi.registerCommand("focusa-on", {
        description: "Re-enable Focusa integration and writes",
        handler: async (_args, ctx) => {
            const h = await focusaFetch("/health");
            if (!h?.ok) {
                ctx.ui.notify("❌ Focusa unavailable", "error");
                return;
            }
            const alreadyEnabled = getFocusaAvailable();
            getAttachmentRuntime().focusaAvailable = true;
            const store = getCurrentScopeStore();
            if (store)
                store.focusaAvailable = true;
            getAttachmentRuntime().outageStart = null;
            getAttachmentRuntime().healthBackoffMs = 30_000;
            const status = await focusaFetch("/status").catch(() => null);
            if (status?.session?.status !== "active") {
                await focusaFetch("/session/start", {
                    method: "POST",
                    body: JSON.stringify({
                        adapter_id: "pi",
                        workspace_id: ctx.cwd || getSessionCwd() || "pi-workspace",
                    }),
                }).catch(() => null);
            }
            if (!getActiveFrameId()) {
                await ensurePiFrame(ctx.cwd, undefined, "pi-auto");
            }
            ctx.ui.setStatus("focusa", getAttachmentRuntime().wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
            if (getActiveFrameId())
                await persistAuthoritativeState();
            if (alreadyEnabled && getActiveFrameId()) {
                ctx.ui.notify(`✅ Focusa already enabled — frame ready: ${getActiveFrameId()}`, "info");
            }
            else if (getActiveFrameId()) {
                ctx.ui.notify(`✅ Focusa enabled — frame ready: ${getActiveFrameId()}`, "info");
            }
            else {
                ctx.ui.notify("⚠️ Focusa enabled but no Pi frame could be created", "warning");
            }
        },
    });
    // /focusa-off (§33.5) — stop ALL Focusa writes; keep reads for status only
    pi.registerCommand("focusa-off", {
        description: "Stop all Focusa writes — Focus State local only",
        handler: async (_args, ctx) => {
            if (!getFocusaAvailable()) {
                ctx.ui.notify("Focusa already disabled", "info");
                return;
            }
            getAttachmentRuntime().focusaAvailable = false;
            const store = getCurrentScopeStore();
            if (store)
                store.focusaAvailable = false;
            ctx.ui.setStatus("focusa", "⏸️ Focusa disabled");
            ctx.ui.notify("⚠️ Focusa writes disabled — Focus State local only", "warning");
        },
    });
    // /focusa-reset (§33.5) — clear all Focus State entries in Focusa's DB + push fresh frame
    pi.registerCommand("focusa-reset", {
        description: "Clear Focus State in Focusa + push fresh Pi frame",
        handler: async (_args, ctx) => {
            const clearedSnapshot = getEffectiveFocusSnapshot();
            const cleared = {
                decisions: clearedSnapshot.decisions.length,
                constraints: clearedSnapshot.constraints.length,
                failures: clearedSnapshot.failures.length,
            };
            getAttachmentRuntime().localDecisions = [];
            getAttachmentRuntime().localConstraints = [];
            getAttachmentRuntime().localFailures = [];
            getAttachmentRuntime().lastFocusSnapshot = {
                decisions: [],
                constraints: [],
                failures: [],
                intent: "",
                currentFocus: "",
            };
            setCompilationErrors([]);
            resetFileEditCounts();
            getAttachmentRuntime().cataloguedDecisions = [];
            getAttachmentRuntime().cataloguedFacts = [];
            getAttachmentRuntime().compactResumePending = false;
            getAttachmentRuntime().forkSuggested = false;
            getAttachmentRuntime().currentTier = "";
            const previousFrameId = getActiveFrameId();
            getAttachmentRuntime().activeFrameId = null;
            {
                const store = getCurrentScopeStore();
                if (store)
                    store.activeFrameId = null;
            }
            persistState();
            if (getFocusaAvailable() && previousFrameId) {
                await focusaFetch("/focus/update", {
                    method: "POST",
                    body: JSON.stringify({
                        frame_id: previousFrameId,
                        turn_id: `pi-turn-${getTurnCount() || 0}`,
                        delta: { decisions: [], constraints: [], failures: [], recent_results: [] },
                    }),
                }).catch(() => { });
            }
            if (getFocusaAvailable()) {
                const frameId = await ensurePiFrame(ctx.cwd, undefined, "pi-reset");
                if (frameId) {
                    getAttachmentRuntime().activeFrameId = frameId;
                    await persistAuthoritativeState();
                    ctx.ui.notify(`✅ Focus State reset (cleared D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\nFresh Pi frame: ${frameId}`, "info");
                }
                else {
                    ctx.ui.notify(`✅ Local shadow cleared (D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\n⚠️ Focusa frame clear failed — writes may resume on old frame`, "warning");
                }
            }
            else {
                ctx.ui.notify(`✅ Local shadow cleared (D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\n⚠️ Focusa offline — run /focusa-on to push fresh frame`, "warning");
            }
        },
    });
}
