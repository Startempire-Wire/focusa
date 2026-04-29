import { S } from "./state.js";

function line(value: unknown): string {
  return String(value || "").trim();
}

export function buildFocusaUtilityCard(mode: "system" | "visible" = "system"): string {
  const packet = S.activeWorkpointPacket || {};
  const mission = line(packet.mission || S.currentAsk?.text || S.activeFrameGoal || S.activeFrameTitle);
  const next = line(packet.next_slice || S.lastCompactDecision);
  const projectRoot = line(packet.project_root || S.sessionCwd);
  const status = S.focusaAvailable ? "available" : "offline/degraded";
  const prefix = mode === "visible" ? "# Focusa Utility Card" : "## Focusa Utility Card";
  return [
    prefix,
    `Status: ${status}`,
    mission ? `Mission: ${mission}` : "Mission: use latest operator instruction and active repo/bead context.",
    next ? `Next anchor: ${next}` : "Next anchor: call focusa_workpoint_resume if resuming or uncertain.",
    projectRoot ? `Scope: project_root=${projectRoot}` : "Scope: bind work to current project root; reject cross-project resume packets.",
    "",
    "Use Focusa as agent working memory and governance:",
    "- First when uncertain/degraded: focusa_tool_doctor.",
    "- Before compaction/model switch/fork/risky continuation: focusa_workpoint_checkpoint.",
    "- After compaction/reload/resume: focusa_workpoint_resume; do not trust transcript tail over Workpoint.",
    "- After proof/tests/API/file evidence: focusa_evidence_capture or focusa_workpoint_link_evidence.",
    "- Before risky or uncertain next action: focusa_predict_record; after outcome: focusa_predict_evaluate.",
    "- For learning: focusa_metacog_* tools; for continuous work: focusa_work_loop_* tools.",
    "- For compaction summaries: use related Workpoint/current-ask/frame/local-shadow/session fallbacks, not blank none fields.",
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}
