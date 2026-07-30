import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { executeMissionCanvasAction } from "./commands.js";
import { loadConfig, resolveInteractionMode, type MissionCanvasWorkspaceProfile } from "./config.js";
import { getSessionCwd } from "./state.js";

const ACTIONS = ["open", "on", "off", "toggle", "status", "set_profile"] as const;
const PROFILES = ["general", "software", "legal", "markets", "research", "custom"] as const;
let activePiContext: ExtensionContext | undefined;

/** Agent-first Mission Canvas control. Uses the exact controller behind /mission-canvas. */
export function registerMissionCanvasTool(pi: ExtensionAPI): void {
  const bindContext = (_event: unknown, ctx: ExtensionContext) => {
    activePiContext = ctx;
  };
  pi.on("session_start", bindContext);
  pi.on("before_agent_start", bindContext);
  pi.on("turn_start", bindContext);

  pi.registerTool({
    name: "focusa_mission_canvas",
    label: "Mission Canvas",
    description:
      "Programmatically open or control the native Pi Mission Canvas. Actions on/off/toggle/set_profile change only its presentation mode; Focusa canonical runtime remains active.",
    parameters: Type.Object({
      action: Type.Union(ACTIONS.map((action) => Type.Literal(action)), {
        description: "Mission Canvas operation. open renders the real native Pi GUI in the current session.",
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
      await executeMissionCanvasAction(command, ctx);
      const cwd = getSessionCwd();
      const interaction = resolveInteractionMode(cwd);
      const presentation = loadConfig(cwd).config;
      const state = {
        action: params.action,
        canvas_enabled: interaction.mode === "canvas-guided",
        interaction_mode: interaction.mode,
        mode_source: interaction.source,
        workspace_profile: presentation.missionCanvasWorkspaceProfile,
        visual_variant: presentation.missionCanvasVisualVariant,
        canonical_runtime_active: true,
        gui: params.action === "open" ? "opened_and_closed_by_operator" : "unchanged",
      };
      return {
        content: [{ type: "text", text: JSON.stringify(state, null, 2) }],
        details: state,
      };
    },
  });
}
