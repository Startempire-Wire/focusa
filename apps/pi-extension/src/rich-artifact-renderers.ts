export type ArtifactKind =
  | "image"
  | "markdown"
  | "dataset"
  | "diff"
  | "browser_snapshot"
  | "diagnostics"
  | "chart"
  | "document"
  | "media"
  | "fpv_session";

export interface RichArtifactDescriptor {
  schema: "focusa.workspace_artifact_descriptor.v1";
  artifact_id: string;
  artifact_kind: ArtifactKind;
  title: string;
  before_ref: string;
  after_ref: string;
  evidence_refs: string[];
  summary?: string;
  changes?: string[];
  citations?: { citation_ref: string; source_origin: string }[];
  provenance?: { source_kind: string; harvested_at?: string };
  project_root: string;
  continuity_id: string;
  session_origin: string;
  freshness: string;
  authority: string;
  vertical_dispatch?: string;
  render_safe: boolean;
  redacted?: boolean;
  artifact_handle?: string;
  external_open_ref?: string;
}

interface KindRenderer {
  kind: ArtifactKind;
  primary: string;
  fallback: string;
  render: (d: RichArtifactDescriptor) => string[];
}

const KINDS: Record<ArtifactKind, KindRenderer> = {
  image: { kind: "image", primary: "image viewer with zoom, metadata, source, and evidence", fallback: "artifact card + Open action", render: rfrFallback },
  markdown: { kind: "markdown", primary: "cited research/document reader", fallback: "bounded text + handle", render: textFallback },
  dataset: { kind: "dataset", primary: "sortable/filterable table", fallback: "schema/row summary + download/open", render: summaryFallback },
  diff: { kind: "diff", primary: "workspace-specific change viewer", fallback: "unified text diff", render: diffFallback },
  browser_snapshot: { kind: "browser_snapshot", primary: "structured accessibility tree and refs", fallback: "bounded JSON/text tree", render: summaryFallback },
  diagnostics: { kind: "diagnostics", primary: "console/network/error inspector", fallback: "summarized findings + refs", render: diagnosticsFallback },
  chart: { kind: "chart", primary: "interactive chart where supported", fallback: "table and static summary", render: summaryFallback },
  document: { kind: "document", primary: "document/PDF reader", fallback: "extracted text + source page refs", render: textFallback },
  media: { kind: "media", primary: "bounded media viewer", fallback: "metadata + external/open action", render: rfrFallback },
  fpv_session: { kind: "fpv_session", primary: "live UIAI FPV Work Surface/share", fallback: "session status + share/open action", render: fpvFallback },
};

export const RICH_ARTIFACT_RENDERERS: Record<ArtifactKind, KindRenderer> = KINDS;

function invariantLines(d: RichArtifactDescriptor): string[] {
  if (!d.render_safe) {
    return [
      `Artifact: ${d.artifact_id}`,
      "RENDER_BLOCKED: render_safe is false; fallback required",
    ];
  }
  const citations = d.citations && d.citations.length > 0
    ? d.citations.map((c) => `${c.citation_ref}@${c.source_origin}`).join(", ")
    : "none";
  const provenance = d.provenance
    ? `${d.provenance.source_kind}${d.provenance.harvested_at ? `@${d.provenance.harvested_at}` : ""}`
    : "unknown";
  return [
    `Artifact: ${d.artifact_id}`,
    `Kind: ${d.artifact_kind}`,
    `Scope: ${d.project_root} · ${d.continuity_id}`,
    `Before: ${d.before_ref || "none"}`,
    `After: ${d.after_ref || "none"}`,
    `Evidence: ${d.evidence_refs.length ? d.evidence_refs.join(", ") : "none"}`,
    `Session origin: ${d.session_origin}`,
    `Freshness: ${d.freshness}`,
    `Authority: ${d.authority}`,
    `Citations: ${citations}`,
    `Provenance: ${provenance}`,
  ];
}

function rfrFallback(d: RichArtifactDescriptor): string[] {
  return ["## Artifact card + Open action",
    `Open: ${d.external_open_ref || d.artifact_handle || d.artifact_id}`,
    ...invariantLines(d)];
}

function textFallback(d: RichArtifactDescriptor): string[] {
  return ["## Bounded text + handle",
    d.summary || d.title,
    `Handle: ${d.artifact_handle || d.artifact_id}`,
    ...invariantLines(d)];
}

function summaryFallback(d: RichArtifactDescriptor): string[] {
  return ["## Summary + refs",
    d.summary || d.title,
    ...(d.changes || []).map((c) => `- ${c}`),
    ...invariantLines(d)];
}

function diffFallback(d: RichArtifactDescriptor): string[] {
  const changes = (d.changes && d.changes.length > 0) ? d.changes : [d.summary || "unified text diff"];
  return ["## Unified diff",
    `BEFORE: ${d.before_ref || "none"}`,
    `AFTER: ${d.after_ref || "none"}`,
    ...changes.map((c) => `+ ${c}`),
    ...invariantLines(d)];
}

function diagnosticsFallback(d: RichArtifactDescriptor): string[] {
  return ["## Summarized findings + refs",
    d.summary || "No findings",
    `Refs: ${d.evidence_refs.join(", ") || "none"}`,
    ...invariantLines(d)];
}

function fpvFallback(d: RichArtifactDescriptor): string[] {
  return ["## Session status + share/open action",
    `FPV session: ${d.artifact_id}`,
    `Open: ${d.external_open_ref || d.session_origin}`,
    ...invariantLines(d)];
}

export function renderRichArtifact(
  d: RichArtifactDescriptor,
  mode: "primary" | "fallback" = "fallback",
  vertical: string = "General"
): string {
  const renderer = KINDS[d.artifact_kind];
  if (!renderer) {
    throw new Error(`Unknown artifact kind: ${d.artifact_kind}`);
  }
  const lines = [
    `# ${vertical} Artifact: ${d.title}`,
    "",
    `Kind: ${d.artifact_kind}`,
    `Primary renderer: ${renderer.primary}`,
    `Required fallback: ${renderer.fallback}`,
    `Render mode: ${mode}`,
    ``,
    ...renderer.render(d),
  ];
  // No client may silently discard an artifact — fallback always included.
  if (mode === "primary") {
    lines.push("", `Fallback if unavailable: ${renderer.fallback}`);
  }
  return lines.join("\n");
}

export function fallbackSafeRender(d: RichArtifactDescriptor, vertical: string = "General"): string {
  return renderRichArtifact(d, "fallback", vertical);
}

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export function registerRichArtifactRenderers(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-artifact-render", {
    description: "Render a rich Workspace Artifact descriptor through its required fallback renderer",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const kind = await ctx.ui.select("Artifact kind", Object.keys(RICH_ARTIFACT_RENDERERS));
      if (!kind) return;
      const sample: RichArtifactDescriptor = {
        schema: "focusa.workspace_artifact_descriptor.v1",
        artifact_id: `artifact:${kind}:${Date.now()}`,
        artifact_kind: kind as ArtifactKind,
        title: `Canonical ${kind} artifact`,
        before_ref: "ref:before",
        after_ref: "ref:after",
        evidence_refs: ["evidence:safe-render"],
        summary: `Safe presentation of a ${kind} artifact from canonical refs`,
        project_root: "/project/canvas",
        continuity_id: "continuity:canvas",
        session_origin: "pi",
        freshness: "live",
        authority: "presentation-only; canonical reducers retain authority",
        render_safe: true,
        provenance: { source_kind: "uiai", harvested_at: new Date().toISOString() },
      };
      pi.sendMessage({
        customType: "focusa-rich-artifact",
        content: fallbackSafeRender(sample),
        display: true,
      });
    },
  });
}