import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { executeMissionCanvasAction } from "./commands.js";
import { loadConfig, resolveInteractionMode, type MissionCanvasWorkspaceProfile } from "./config.js";
import { refreshActiveMissionCanvasShell } from "./mission-canvas-shell.js";
import { getAttachmentRuntime, getSessionCwd } from "./state.js";

interface MissionCanvasScope {
  project_root: string;
  continuity_id: string;
  session_id: string;
  attachment_id: string;
  working_subpath_id: string | null;
}

const ACTIONS = ["open", "on", "off", "toggle", "status", "set_profile"] as const;
const PROFILES = ["general", "software", "legal", "markets", "research", "custom"] as const;
let activePiContext: ExtensionContext | undefined;
let piEventSequence = 0;

function scopeForContext(ctx: ExtensionContext): MissionCanvasScope {
  const sessionId = String(ctx.sessionManager.getSessionFile?.() || `pi-session-${process.pid}`);
  const runtime = getAttachmentRuntime();
  return {
    project_root: getSessionCwd(),
    continuity_id: runtime.continuityId || "extension-bootstrap",
    session_id: sessionId,
    attachment_id: sessionId,
    working_subpath_id: null,
  };
}

function appendPiEventSafely(
  ctx: ExtensionContext,
  eventKind: "pi_turn_started" | "pi_turn_completed" | "pi_message_updated" | "pi_tool_started" | "pi_tool_completed",
  event: unknown
): void {
  const presentation = loadConfig(getSessionCwd()).config;
  const scope = scopeForContext(ctx);
  const eventId = `pi-event:${process.pid}:${Date.now()}:${++piEventSequence}`;
  const url = `${presentation.focusaApiBaseUrl.replace(/\/$/, "")}/mission-canvas/pi-session/events`;
  void fetch(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-focusa-permissions": "mission_canvas:write",
      ...(presentation.focusaToken ? { authorization: `Bearer ${presentation.focusaToken}` } : {}),
    },
    body: JSON.stringify({
      scope,
      event_id: eventId,
      event_kind: eventKind,
      projection_revision: 0,
      layout_revision: 0,
      payload: boundedPiEvent(event),
      occurred_at: new Date().toISOString(),
    }),
  }).catch(() => undefined);
}

function boundedPiEvent(event: unknown): unknown {
  const serialized = JSON.stringify(event, (_key, value) => {
    if (typeof value === "string" && value.length > 16_384) return `${value.slice(0, 16_384)}…`;
    return value;
  });
  if (!serialized) return null;
  return serialized.length > 32_768
    ? { truncated: true, preview: serialized.slice(0, 32_768) }
    : JSON.parse(serialized);
}

/** Agent-first Mission Canvas control. Uses the exact controller behind /mission-canvas. */
export function registerMissionCanvasTool(pi: ExtensionAPI): void {
  const bindContext = (_event: unknown, ctx: ExtensionContext) => {
    activePiContext = ctx;
  };
  pi.on("session_start", (event, ctx) => {
    bindContext(event, ctx);
    appendPiEventSafely(ctx, "pi_message_updated", {
      event_kind: "mission_canvas_session_restored",
      interaction_mode: resolveInteractionMode(getSessionCwd()).mode,
      current_pi_session: true,
    });
    if (resolveInteractionMode(getSessionCwd()).mode === "canvas-guided" && ctx.hasUI) {
      // Let Pi finish mounting its stock root before replacing it; that root
      // remains alive underneath Canvas and is revealed by the off switch.
      setTimeout(() => void executeMissionCanvasAction("", ctx), 1_000);
    }
  });
  pi.on("before_agent_start", bindContext);
  pi.on("turn_start", (event, ctx) => {
    bindContext(event, ctx);
    appendPiEventSafely(ctx, "pi_turn_started", event);
  });
  pi.on("turn_end", (event, ctx) => {
    appendPiEventSafely(ctx, "pi_turn_completed", event);
    refreshActiveMissionCanvasShell();
  });
  pi.on("message_update", (event, ctx) => {
    appendPiEventSafely(ctx, "pi_message_updated", event);
    refreshActiveMissionCanvasShell();
  });
  pi.on("tool_execution_start", (event, ctx) => {
    appendPiEventSafely(ctx, "pi_tool_started", event);
  });
  pi.on("tool_execution_end", (event, ctx) => {
    appendPiEventSafely(ctx, "pi_tool_completed", event);
    refreshActiveMissionCanvasShell();
  });
  pi.registerTool({
    name: "focusa_mission_canvas",
    label: "Mission Canvas",
    description:
      "Programmatically open or control Mission Canvas. Open routes through the Desktop rich-host handoff and falls back to the Pi compatibility projection when Desktop is unavailable.",
    parameters: Type.Object({
      action: Type.Union(ACTIONS.map((action) => Type.Literal(action)), {
        description: "Mission Canvas operation. open requests a Desktop handoff (with legacy Pi fallback when required).",
      }),
      profile: Type.Optional(
        Type.Union(PROFILES.map((profile) => Type.Literal(profile)), {
          description: "Required for set_profile; selects the workspace cockpit without changing agent state.",
        })
      ),
    }),
    async execute(_toolCallId, params) {
      const ctx = activePiContext;
      if (!ctx) throw new Error("Mission Canvas has no active Pi session context yet");
      if (params.action === "set_profile" && !params.profile) {
        throw new Error(`profile is required; expected one of: ${PROFILES.join(", ")}`);
      }
      if (params.action === "open" && !ctx.hasUI) {
        throw new Error("Mission Canvas GUI requires an interactive Pi UI; use status/on/off in headless mode");
      }
      const command =
        params.action === "open"
          ? ""
          : params.action === "set_profile"
            ? `profile ${String(params.profile) as MissionCanvasWorkspaceProfile}`
            : params.action;
      const cwd = getSessionCwd();
      // Mission Canvas delegates to the shared controller for Desktop handoff and compatibility fallback.
      await executeMissionCanvasAction(command, ctx);
      const effectiveInteraction = resolveInteractionMode(cwd);
      const effectivePresentation = loadConfig(cwd).config;
      appendPiEventSafely(ctx, "pi_message_updated", {
        event_kind: "mission_canvas_lifecycle_receipt",
        action: params.action,
        current_pi_session: true,
      });
      const state = {
        action: params.action,
        canvas_enabled: effectiveInteraction.mode === "canvas-guided",
        interaction_mode: effectiveInteraction.mode,
        mode_source: effectiveInteraction.source,
        workspace_profile: effectivePresentation.missionCanvasWorkspaceProfile,
        visual_variant: effectivePresentation.missionCanvasVisualVariant,
        canonical_runtime_active: true,
        gui: "desktop_or_pi_tui",
        host_scope: "current_pi_session",
      };
      return {
        content: [{ type: "text", text: JSON.stringify(state, null, 2) }],
        details: state,
      };
    },
  });
}
