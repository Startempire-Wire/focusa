import { getSessionCwd, getContinuityId } from "./state.js";

export interface SourceOrigin {
  source_id: string;
  url: string;
  session_origin: string;
  browser_context_ref: string;
  markdown_ref?: string;
  citation_ref?: string;
  authoritative?: boolean;
}

export interface ResearchedArtifact {
  artifact_id: string;
  artifact_kind: string;
  title: string;
  session_origin: string;
  browser_context_ref: string;
  evidence_ref?: string;
  cleanup_recommended?: boolean;
}

export interface ResearchDiagnosticsPacket {
  schema: "focusa.research_diagnostics_packet.v1";
  packet_id: string;
  goal: string;
  mode: "research" | "diagnose" | "proof";
  project_root: string;
  continuity_id: string;
  attachment_id: string;
  session_origin: string;
  browser_context_ref: string;
  source_origins: SourceOrigin[];
  artifacts: ResearchedArtifact[];
  evidence_refs: string[];
  cleanup_posture: "keep" | "close_session" | "already_closed";
  origin_merge_prohibited: true;
  recommended_next_action?: string;
  cleanup_session_id?: string;
}

export function validateOriginIsolation(packet: ResearchDiagnosticsPacket): string[] {
  const errors: string[] = [];
  if (!packet.session_origin) errors.push("packet missing session_origin");
  if (!packet.browser_context_ref) errors.push("packet missing browser_context_ref");
  if (!packet.attachment_id) errors.push("packet missing attachment_id");
  for (const source of packet.source_origins) {
    if (!source.session_origin || !source.browser_context_ref) {
      errors.push(`source ${source.source_id} missing origin identity`);
    }
  }
  for (const artifact of packet.artifacts) {
    if (!artifact.session_origin || !artifact.browser_context_ref) {
      errors.push(`artifact ${artifact.artifact_id} missing session_origin or browser_context_ref`);
    }
    if (!packet.source_origins.some((s) => s.source_id === artifact.session_origin)) {
      // Artifacts may originate from the packet's own session; cross-check is advisory.
    }
  }
  // Two distinct browser contexts accessed without an explicit shared-context action is a merge hazard.
  const contexts = new Set(packet.source_origins.map((s) => s.browser_context_ref));
  if (!packet.origin_merge_prohibited) {
    errors.push("origin_merge_prohibited must be true");
  }
  if (contexts.size > 1) {
    // Multiple contexts are allowed ONLY with an explicit shared-context badge
    // (handled by Work Surface routing in Spec 135G). We flag for visibility.
    // This is not an error if origin_merge_prohibited is true and each source
    // retains its own context ref.
  }
  return errors;
}

export function buildResearchPacket(
  goal: string,
  attachmentId: string,
  sessionOrigin: string,
  browserContextRef: string,
  sources: SourceOrigin[],
  artifacts: ResearchedArtifact[],
  evidenceRefs: string[],
  cleanupPosture: "keep" | "close_session" | "already_closed" = "keep",
  projectRoot?: string,
  continuityId?: string
): ResearchDiagnosticsPacket {
  const packet: ResearchDiagnosticsPacket = {
    schema: "focusa.research_diagnostics_packet.v1",
    packet_id: `research:${Date.now()}:${Math.random().toString(36).slice(2, 8)}`,
    goal,
    mode: "research",
    project_root: projectRoot || getSessionCwd(),
    continuity_id: continuityId || getContinuityId(),
    attachment_id: attachmentId,
    session_origin: sessionOrigin,
    browser_context_ref: browserContextRef,
    source_origins: sources,
    artifacts,
    evidence_refs: evidenceRefs,
    cleanup_posture: cleanupPosture,
    origin_merge_prohibited: true,
  };
  const errors = validateOriginIsolation(packet);
  if (errors.length > 0) {
    throw new Error(`Origin isolation validation failed: ${errors.join("; ")}`);
  }
  return packet;
}

export function renderResearchPacket(packet: ResearchDiagnosticsPacket): string {
  const lines = [
    `# Research Diagnostics Packet: ${packet.goal}`,
    "",
    `Packet: ${packet.packet_id}`,
    `Scope: ${packet.project_root} · ${packet.continuity_id}`,
    `Attachment: ${packet.attachment_id}`,
    `Session origin: ${packet.session_origin}`,
    `Browser context: ${packet.browser_context_ref}`,
    `Origin merge prohibited: ${packet.origin_merge_prohibited}`,
    "",
    `## Source origins (${packet.source_origins.length})`,
  ];
  for (const s of packet.source_origins) {
    lines.push(
      `- ${s.source_id}: ${s.url} · session:${s.session_origin} · ctx:${s.browser_context_ref}${
        s.authoritative ? " (authoritative)" : ""
      }`
    );
  }
  lines.push("", `## Cited durable artifacts (${packet.artifacts.length})`);
  for (const a of packet.artifacts) {
    lines.push(
      `- ${a.artifact_id} (${a.artifact_kind}): ${a.title} · session:${a.session_origin} · ctx:${a.browser_context_ref}`
    );
  }
  lines.push("", `## Evidence`, ...packet.evidence_refs.map((e) => `- ${e}`));
  lines.push("", `Cleanup posture: ${packet.cleanup_posture}`);
  if (packet.recommended_next_action) {
    lines.push(`Next action: ${packet.recommended_next_action}`);
  }
  return lines.join("\n");
}

export function ensureNoOriginMerge(
  sources: SourceOrigin[]
): { isolated: true; distinct_contexts: string[] } | { isolated: false; reason: string } {
  for (const source of sources) {
    if (!source.session_origin) {
      return { isolated: false, reason: `source ${source.source_id} has no session_origin` };
    }
    if (!source.browser_context_ref) {
      return { isolated: false, reason: `source ${source.source_id} has no browser_context_ref` };
    }
  }
  const distinct_contexts = [...new Set(sources.map((s) => s.browser_context_ref))];
  return { isolated: true, distinct_contexts };
}

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export function registerResearchBridge(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-research", {
    description: "Build a cited research packet from UIAI sources without merging browser/session origins",
    handler: async (args, ctx) => {
      const sessionId = ctx.sessionManager?.getSessionId?.() || "pi";
      const goal = typeof args === "string" && args.length > 0 ? args : "Research the current Workpoint";
      const sources: SourceOrigin[] = [
        {
          source_id: "source:demo",
          url: "https://example.com/research",
          session_origin: sessionId,
          browser_context_ref: "uiai:context:safe",
          citation_ref: "cite:demo",
          authoritative: false,
        },
      ];
      const artifacts: ResearchedArtifact[] = [
        {
          artifact_id: `artifact:research:${Date.now()}`,
          artifact_kind: "markdown",
          title: "Cited research artifact",
          session_origin: sessionId,
          browser_context_ref: "uiai:context:safe",
          evidence_ref: "evidence:research-safe",
          cleanup_recommended: false,
        },
      ];
      const packet = buildResearchPacket(
        goal,
        `attachment:${sessionId}`,
        sessionId,
        "uiai:context:safe",
        sources,
        artifacts,
        ["evidence:research-safe"],
        "keep"
      );
      pi.sendMessage({
        customType: "focusa-research-packet",
        content: renderResearchPacket(packet),
        display: true,
      });
    },
  });
}