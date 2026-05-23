import { S, getScopedWorkpointPacket, isProjectRootAuthoritySafe, normalizeProjectRoot } from "./state.js";

function line(value: unknown): string {
  return String(value || "").trim();
}

export function buildFocusaUtilityCard(mode: "system" | "visible" = "system"): string {
  const scopedPacket = getScopedWorkpointPacket();
  const mission = line(scopedPacket?.mission);
  const next = line(scopedPacket?.next_slice);
  const projectRoot = normalizeProjectRoot(scopedPacket?.project_root || S.sessionCwd);
  const continuityId = scopedPacket ? line(scopedPacket?.continuity_id) : line(S.continuityId);
  const status = S.focusaAvailable ? "available" : "offline/degraded";
  const prefix = mode === "visible" ? "# Focusa Utility Card" : "## Focusa Utility Card";
  const safeScope = !!projectRoot && isProjectRootAuthoritySafe(projectRoot);

  if (mode === "visible" && !scopedPacket) {
    return [
      prefix,
      `Status: ${status}`,
      `Scope: ${projectRoot || "unknown"}${safeScope ? "" : " (broad/unsafe — no Workpoint auto-resume)"}`,
      "Scoped Workpoint: none verified; latest operator instruction is authority.",
      "Next: cd into a project or pass project_root; use focusa_workpoint_resume only for real project work.",
    ].join("\n");
  }

  return [
    prefix,
    `Status: ${status}`,
    scopedPacket ? "Scoped Workpoint: verified project_root + continuity_id match." : "Scoped Workpoint: none verified for this logical session; ignore stale scoped carryover.",
    mission ? `Mission: ${mission}` : "Mission: use latest operator instruction only; no scoped Workpoint mission verified.",
    next ? `Next anchor: ${next}` : "Next anchor: call focusa_workpoint_resume with current continuity_id if resuming project work or uncertain.",
    projectRoot ? `Scope: project_root=${projectRoot}${safeScope ? "" : " (broad/unsafe)"}` : "Scope: bind work to current project root; reject cross-project resume packets.",
    scopedPacket && continuityId ? `Continuity: continuity_id=${continuityId}` : "Continuity: no Workpoint continuity verified for this Pi session; require explicit resume/checkpoint before trusting same-root state.",
    "Trajectory: use focusa_trajectory_view for project goals only; never merge sessions without project_root+continuity_id.",
    "",
    "Use Focusa as agent working memory and governance:",
    "- First when uncertain/degraded: focusa_tool_doctor.",
    "- Project start/resume: focusa_trajectory_view, then Workpoint resume/checkpoint as needed.",
    "- Compaction/model switch/fork/risky continuation: focusa_workpoint_checkpoint before; focusa_workpoint_resume after.",
    "- Proof/tests/API/file evidence: focusa_evidence_capture or focusa_workpoint_link_evidence.",
    "- Predictions/learning/continuous work: focusa_predict_*, focusa_metacog_*, focusa_work_loop_*.",
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}
