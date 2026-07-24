import {
  getContinuityId,
  getFocusaAvailable,
  getLastProjectIdentity,
  getLastProjectRootResolution,
  getLastTrajectoryClarity,
  getScopedWorkpointPacket,
  getSessionCwd,
  isProjectRootAuthoritySafe,
  normalizeProjectRoot,
} from "./state.js";

function line(value: unknown): string {
  return String(value || "")
    .replace(/\s+/g, " ")
    .trim();
}

function compact(value: unknown, fallback: string, max = 180): string {
  const text = line(value);
  if (!text) return fallback;
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

/**
 * Prompt-safe cached fallback for the startup card.
 *
 * The async DVS awareness substrate is the preferred visible renderer. This
 * function intentionally performs no daemon I/O so prompt-critical hooks never
 * delay or reject operator input when Focusa is unavailable.
 */
export function buildFocusaUtilityCard(mode: "system" | "visible" = "system"): string {
  const packet = getScopedWorkpointPacket();
  const projectRoot = normalizeProjectRoot(packet?.project_root || getSessionCwd());
  const resolution = getLastProjectRootResolution();
  const identity = getLastProjectIdentity();
  const identityMatches = normalizeProjectRoot(identity?.project_root) === projectRoot;
  const packetMatches = Boolean(packet && normalizeProjectRoot(packet.project_root) === projectRoot);
  const resolutionVerified =
    resolution?.requiresOperatorConfirmation === false && (resolution?.confidenceScore || 0) >= 0.8;
  const scopeVerified =
    isProjectRootAuthoritySafe(projectRoot) &&
    resolution?.requiresOperatorConfirmation !== true &&
    (identityMatches || packetMatches || resolutionVerified);
  const projectName =
    scopeVerified && identityMatches
      ? compact(identity?.canonical_name || identity?.project_id, "project", 60)
      : "unverified project";
  const trajectory = getLastTrajectoryClarity() || {};
  const mission = compact(
    packet?.mission || trajectory.short_term_goal || trajectory.active_gap,
    "follow the newest operator request",
    180
  );
  const next = compact(
    packet?.next_slice ||
      trajectory.recommended_action ||
      (scopeVerified
        ? "continue the newest operator request"
        : "verify project identity before durable project writes"),
    "continue the newest operator request",
    180
  );
  const continuity = scopeVerified ? line(packet?.continuity_id || getContinuityId()) : "unverified";
  const prefix = mode === "visible" ? "# Focusa" : "## Focusa awareness";

  return [
    prefix,
    `Status: ${getFocusaAvailable() ? "available" : "degraded"}`,
    scopeVerified
      ? `Scope: verified · ${projectName} · ${projectRoot} · continuity=${continuity || "not checkpointed"}`
      : "Scope: unverified",
    `Mission: ${mission}`,
    `Next: ${next}`,
    scopeVerified
      ? "Boundary: operator steering leads; scoped mutation tools enforce durable-write authority."
      : "Boundary: conversation and diagnosis continue; durable project writes wait for identity verification.",
  ].join("\n");
}
