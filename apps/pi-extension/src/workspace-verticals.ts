import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { getActiveWorkpointPacket, getContinuityId, getSessionCwd } from "./state.js";

export type WorkspaceVertical = "General" | "Software" | "Legal" | "Markets" | "Research" | "Custom";

export interface CanonicalArtifactProjection {
  artifactId: string;
  artifactKind: string;
  title: string;
  beforeRef: string;
  afterRef: string;
  evidenceRefs: string[];
  projectRoot: string;
  continuityId: string;
  sessionOrigin: string;
  freshness: string;
  authority: string;
  summary: string;
  changes: string[];
}

interface VerticalProfile {
  profile: WorkspaceVertical;
  identity: string;
  variants: string[];
  primaryView: string;
  safetyLens: string;
}

export const VERTICAL_PROFILES: Record<WorkspaceVertical, VerticalProfile> = {
  General: {
    profile: "General",
    identity: "neutral Focusa living-field",
    variants: ["balanced", "quiet", "high-contrast"],
    primaryView: "Mission, Workpoint, Evidence, CRIST stages, sessions, next safe action",
    safetyLens: "No domain-specific assumptions",
  },
  Software: {
    profile: "Software",
    identity: "cobalt/electric-violet technical grid",
    variants: ["dense-grid", "diff-focus", "test-focus"],
    primaryView: "Unified code diff with file, branch, test, and change markers",
    safetyLens: "Security, performance, compatibility, and proof indicators remain visible",
  },
  Legal: {
    profile: "Legal",
    identity: "oxblood/brass document workspace",
    variants: ["redline", "citation", "timeline"],
    primaryView: "Side-by-side redline with clause, citation, authority, and date anchors",
    safetyLens: "Legal assistance never implies filing, sending, or approval authority",
  },
  Markets: {
    profile: "Markets",
    identity: "midnight/amber signal workspace",
    variants: ["thesis", "risk", "catalyst"],
    primaryView: "Thesis revision with changed assumptions, catalysts, and invalidation rules",
    safetyLens: "Research role never grants trading or execution authority",
  },
  Research: {
    profile: "Research",
    identity: "indigo/teal notebook graph",
    variants: ["claim-delta", "source-graph", "reading-queue"],
    primaryView: "Claim delta with evidence changes, contradictions, and provenance trails",
    safetyLens: "Multiple sources remain visible without merging origin",
  },
  Custom: {
    profile: "Custom",
    identity: "registered profile composition",
    variants: ["operator-composed", "compact", "expanded"],
    primaryView: "Approved registered artifact cards and domain-pack composition",
    safetyLens: "Custom presentation cannot invent canonical state or authority",
  },
};

const selectedProfiles = new Map<string, { profile: WorkspaceVertical; variant: string }>();

export function artifactInvariant(artifact: CanonicalArtifactProjection): string[] {
  return [
    `Artifact: ${artifact.artifactId}`,
    `Kind: ${artifact.artifactKind}`,
    `Scope: ${artifact.projectRoot} · ${artifact.continuityId}`,
    `Before: ${artifact.beforeRef || "none"}`,
    `After: ${artifact.afterRef || "none"}`,
    `Evidence: ${artifact.evidenceRefs.length ? artifact.evidenceRefs.join(", ") : "none"}`,
    `Session origin: ${artifact.sessionOrigin || "unknown"}`,
    `Freshness: ${artifact.freshness || "unknown"}`,
    `Authority: ${artifact.authority || "presentation-only"}`,
  ];
}

function profileBody(artifact: CanonicalArtifactProjection, profile: WorkspaceVertical): string[] {
  const changes = artifact.changes.length ? artifact.changes : [artifact.summary];
  switch (profile) {
    case "Software":
      return ["## Unified diff", ...changes.map((change) => `+ ${change}`), "Tests: evidence-linked only"];
    case "Legal":
      return ["## Side-by-side redline", `BEFORE │ ${artifact.beforeRef || "none"}`, `AFTER  │ ${artifact.afterRef || "none"}`, ...changes.map((change) => `Clause delta: ${change}`)];
    case "Markets":
      return ["## Thesis revision", ...changes.map((change) => `Changed assumption: ${change}`), "Execution: operator-controlled"];
    case "Research":
      return ["## Claim delta", ...changes.map((change) => `Claim/evidence change: ${change}`), `Provenance: ${artifact.evidenceRefs.join(", ") || "unverified"}`];
    case "Custom":
      return ["## Registered custom projection", ...changes.map((change) => `Card: ${change}`), "Unregistered semantics: suppressed"];
    default:
      return ["## General artifact card", artifact.summary, ...changes.map((change) => `- ${change}`)];
  }
}

export function renderArtifactProjection(
  artifact: CanonicalArtifactProjection,
  profile: WorkspaceVertical,
  variant: string
): string {
  const descriptor = VERTICAL_PROFILES[profile];
  if (!descriptor.variants.includes(variant)) {
    throw new Error(`Variant ${variant} is not registered for ${profile}`);
  }
  return [
    `# ${profile} Artifact Projection — ${artifact.title}`,
    "",
    `Visual identity: ${descriptor.identity} · Variant: ${variant}`,
    `Primary view: ${descriptor.primaryView}`,
    `Safety lens: ${descriptor.safetyLens}`,
    "",
    ...artifactInvariant(artifact),
    "",
    ...profileBody(artifact, profile),
    "",
    `Open artifact: ${artifact.artifactId}`,
  ].join("\n");
}

function activeArtifact(): CanonicalArtifactProjection {
  const packet = getActiveWorkpointPacket();
  const workpoint = packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet;
  const id = String(workpoint?.workpoint_id || packet?.workpoint_id || "workpoint:unavailable");
  const evidence = Array.isArray(workpoint?.verification_records)
    ? workpoint.verification_records.map((record: any) => record.evidence_ref).filter(Boolean)
    : Array.isArray(packet?.evidence_refs)
      ? packet.evidence_refs
      : [];
  return {
    artifactId: id,
    artifactKind: "workpoint",
    title: String(workpoint?.mission || packet?.mission || "Active Workpoint"),
    beforeRef: String(packet?.previous_workpoint_ref || ""),
    afterRef: id,
    evidenceRefs: evidence,
    projectRoot: getSessionCwd(),
    continuityId: getContinuityId(),
    sessionOrigin: String(workpoint?.session_id || packet?.session_id || "pi"),
    freshness: String(packet?.updated_at || "live projection"),
    authority: "presentation-only; canonical Workpoint and evidence reducers remain authoritative",
    summary: String(workpoint?.next_slice || packet?.next_action || "No next action reported"),
    changes: Array.isArray(packet?.recent_changes) ? packet.recent_changes : [],
  };
}

export function registerWorkspaceVerticals(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-profile", {
    description: "Select a Mission Canvas workspace vertical and independent visual variant",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const profile = (await ctx.ui.select(
        "Workspace vertical",
        Object.keys(VERTICAL_PROFILES)
      )) as WorkspaceVertical | undefined;
      if (!profile) return;
      const variant = await ctx.ui.select("Visual variant", VERTICAL_PROFILES[profile].variants);
      if (!variant) return;
      const scopeKey = `${getSessionCwd()}\u0000${getContinuityId()}`;
      selectedProfiles.set(scopeKey, { profile, variant });
      pi.sendMessage({
        customType: "focusa-artifact-projection",
        content: renderArtifactProjection(activeArtifact(), profile, variant),
        display: true,
      });
    },
  });

  pi.registerCommand("focusa-artifact", {
    description: "Render the active canonical artifact through the selected vertical",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const scopeKey = `${getSessionCwd()}\u0000${getContinuityId()}`;
      const selected = selectedProfiles.get(scopeKey) || {
        profile: "General" as WorkspaceVertical,
        variant: VERTICAL_PROFILES.General.variants[0],
      };
      pi.sendMessage({
        customType: "focusa-artifact-projection",
        content: renderArtifactProjection(activeArtifact(), selected.profile, selected.variant),
        display: true,
      });
    },
  });
}
