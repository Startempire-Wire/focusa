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
  const resolution = S.lastProjectRootResolution;
  const confidence = resolution ? ` confidence=${Math.round(resolution.confidenceScore * 100)}% source=${resolution.source}` : "";
  const needsConfirm = resolution?.requiresOperatorConfirmation === true;

  const friendlyQ = [
    "Friendly Focusa Q (internal orientation, not a blocker):",
    "1. Where am I? project_root + continuity_id → focusa_project_identity / focusa_project_verify.",
    "2. Where are we going? current state, destination, waypoints → focusa_trajectory_view / define_goal / assess.",
    "3. What is the next useful move? mission + active object + next anchor → focusa_workpoint_resume / checkpoint.",
    "4. What proof changes confidence? tests/API/file handles → focusa_active_object_resolve + focusa_evidence_capture/link.",
    "5. What compounds? prediction outcome + reusable lesson → focusa_predict_record + focusa_predict_evaluate + focusa_metacog_*.",
  ];
  const routeHints = [
    "Tool routes: Orient = focusa_project_identity → focusa_trajectory_view → focusa_workpoint_resume; Execute = focusa_active_object_resolve → focusa_workpoint_checkpoint; Prove = focusa_evidence_capture / focusa_workpoint_link_evidence → focusa_trajectory_assess; Learn = focusa_predict_record → focusa_predict_evaluate → focusa_metacog_capture/retrieve; Recover = focusa_tool_doctor → focusa_resource_mode/focusa_traverse/focusa_workpoint_resume.",
    "Focus State tools (scratch/decide/constraint/failure/etc.) are note/decision slots; use them with the project route, not instead of it.",
  ];

  if (mode === "visible" && !scopedPacket) {
    return [
      prefix,
      `Status: ${status}`,
      `Project folder: ${projectRoot || "unknown"}${safeScope ? "" : " (broad/unsafe — no Workpoint auto-resume)"}${confidence}`,
      ...friendlyQ,
      "Project-bound Workpoint: none verified yet; latest operator instruction is the seed, then bind it to folder + trajectory + next anchor.",
      needsConfirm
        ? "Suggested first route: confirm project folder, then run focusa_trajectory_view/define_goal before durable Focusa writes."
        : "Suggested first route: run focusa_trajectory_view for goals/gap, then checkpoint once mission and next action are clear.",
      ...routeHints,
    ].join("\n");
  }

  return [
    prefix,
    `Status: ${status}`,
    scopedPacket ? "Project-bound Workpoint: verified project_root + continuity_id match." : "Project-bound Workpoint: none verified for this logical session; ignore stale carryover from other projects/sessions.",
    !safeScope || needsConfirm ? "Project folder check: confirm the project file folder/container before durable state writes." : "Project root: confirmed project file folder/container; trajectory provides the functional route.",
    mission ? `Mission: ${mission}` : "Mission: use latest operator instruction as seed; bind it to trajectory + Workpoint before long work.",
    next ? `Next anchor: ${next}` : "Next anchor: call focusa_workpoint_resume with current continuity_id if resuming project work or uncertain.",
    projectRoot ? `Project folder: project_root=${projectRoot}${safeScope ? "" : " (broad/unsafe)"}` : "Project folder: bind work to the folder containing project files; reject cross-project resume packets.",
    scopedPacket && continuityId ? `Continuity: continuity_id=${continuityId}` : "Continuity: no Workpoint continuity verified for this Pi session; use resume/checkpoint before trusting same-root state.",
    "Trajectory: use focusa_trajectory_view for project goals/gap; never merge sessions without project_root+continuity_id.",
    "",
    ...friendlyQ,
    ...routeHints,
    "- Compaction/model switch/fork/risky continuation: focusa_workpoint_checkpoint before; focusa_workpoint_resume after.",
    "- Continuous/background work: focusa_work_loop_* after writer-status/preflight; focusa_silent_sessions only when explicitly managing background sessions.",
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}
