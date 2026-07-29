import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { getActiveWorkpointPacket, getContinuityId, getSessionCwd } from "./state.js";

export type CristStage = "Context" | "Role" | "Interview" | "Spec" | "Tasks";
export interface CristStageBinding {
  stage: CristStage;
  readOperation: string;
  mutateOperation: string;
  command: string;
  invalidationKeys: string[];
}
export const CRIST_STAGE_BINDINGS: CristStageBinding[] = [
  { stage: "Context", readOperation: "focusa.context.source.list", mutateOperation: "focusa.context.source.ingest", command: "/focusa-context", invalidationKeys: ["context.sources", "context.claims"] },
  { stage: "Role", readOperation: "focusa.role_profile.list", mutateOperation: "focusa.role_profile.draft", command: "/focusa-role", invalidationKeys: ["workspace.role"] },
  { stage: "Interview", readOperation: "focusa.interview.session.list", mutateOperation: "focusa.interview.session.mutate", command: "/focusa-interview", invalidationKeys: ["workspace.interview"] },
  { stage: "Spec", readOperation: "focusa.spec_workbench.session.list", mutateOperation: "focusa.spec_workbench.session.mutate", command: "/mission-canvas", invalidationKeys: ["workspace.spec"] },
  { stage: "Tasks", readOperation: "focusa.task_plan.list", mutateOperation: "focusa.task_plan.mutate", command: "/focusa-rail", invalidationKeys: ["workspace.tasks", "workpoint.current"] },
];

export function renderCristStage(stage: CristStage, state: unknown): string {
  const binding = CRIST_STAGE_BINDINGS.find((entry) => entry.stage === stage);
  if (!binding) throw new Error(`Unknown C.R.I.S.T. stage: ${stage}`);
  const packet = state && typeof state === "object" ? state as Record<string, unknown> : {};
  return [
    `# C.R.I.S.T. · ${stage}`,
    `Scope: ${getSessionCwd()} · ${getContinuityId()}`,
    `Canonical read: ${binding.readOperation}`,
    `Governed action: ${binding.mutateOperation}`,
    `Open workflow: ${binding.command}`,
    `Live keys: ${binding.invalidationKeys.join(", ")}`,
    `State: ${JSON.stringify(packet).slice(0, 1200)}`,
    "Authority: generated presentation only; canonical reducers remain authoritative",
  ].join("\n");
}

export function registerCristCanvas(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-crist", {
    description: "Open generated C.R.I.S.T. stage UI bound to canonical operations",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const stage = await ctx.ui.select("C.R.I.S.T. stage", CRIST_STAGE_BINDINGS.map((b) => b.stage));
      if (!stage) return;
      pi.sendMessage({
        customType: "focusa-crist-stage",
        content: renderCristStage(stage as CristStage, getActiveWorkpointPacket()),
        display: true,
      });
    },
  });
}
