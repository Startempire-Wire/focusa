import { S, getScopedWorkpointPacket, normalizeProjectRoot } from "./state.js";

function line(value: unknown): string {
  return String(value || "").trim();
}

export function buildFocusaUtilityCard(mode: "system" | "visible" = "system"): string {
  const scopedPacket = getScopedWorkpointPacket();
  const mission = line(scopedPacket?.mission);
  const next = line(scopedPacket?.next_slice);
  const projectRoot = normalizeProjectRoot(S.sessionCwd || scopedPacket?.project_root);
  const continuityId = line(S.continuityId || scopedPacket?.continuity_id);
  const status = S.focusaAvailable ? "available" : "offline/degraded";
  const prefix = mode === "visible" ? "# Focusa Utility Card" : "## Focusa Utility Card";
  return [
    prefix,
    `Status: ${status}`,
    scopedPacket ? "Scoped Workpoint: verified project_root + continuity_id match." : "Scoped Workpoint: none verified for this logical session; ignore stale scoped carryover.",
    mission ? `Mission: ${mission}` : "Mission: use latest operator instruction only; no scoped Workpoint mission verified.",
    next ? `Next anchor: ${next}` : "Next anchor: call focusa_workpoint_resume with current continuity_id if resuming or uncertain.",
    projectRoot ? `Scope: project_root=${projectRoot}` : "Scope: bind work to current project root; reject cross-project resume packets.",
    continuityId ? `Continuity: continuity_id=${continuityId}` : "Continuity: require a stable continuity_id before trusting same-root session state.",
    "Trajectory: use focusa_trajectory_view for high/mid/low goals and advisory similarity; never merge same-high-level sessions without project_root+continuity_id.",
    "",
    "Use Focusa as agent working memory and governance:",
    "- First when uncertain/degraded: focusa_tool_doctor.",
    "- On project start/resume or unclear next action: focusa_trajectory_view, then Workpoint resume/checkpoint as needed.",
    "- Before compaction/model switch/fork/risky continuation: focusa_workpoint_checkpoint.",
    "- After compaction/reload/resume: focusa_workpoint_resume; do not trust transcript tail over Workpoint.",
    "- After proof/tests/API/file evidence: focusa_evidence_capture or focusa_workpoint_link_evidence.",
    "- Before risky or uncertain next action: focusa_predict_record; after outcome: focusa_predict_evaluate.",
    "- For learning: focusa_metacog_* tools; for continuous work: focusa_work_loop_* tools.",
    "- For compaction summaries: use related Workpoint/current-ask/frame/local-shadow/session fallbacks, not blank none fields.",
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}
