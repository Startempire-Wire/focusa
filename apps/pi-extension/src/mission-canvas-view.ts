import type { Theme } from "@earendil-works/pi-coding-agent";
import {
  truncateToWidth,
  visibleWidth,
  wrapTextWithAnsi,
  type Component,
} from "@earendil-works/pi-tui";
import {
  accessibilityPreferences,
  responsiveCanvasMode,
  surfaceCapacity,
  virtualWindow,
} from "./mission-canvas-accessibility.js";

export type MissionCanvasActivity =
  | "Overview"
  | "Context"
  | "Role"
  | "Interview"
  | "Spec"
  | "Tasks / Work"
  | "Sessions"
  | "Documents"
  | "Research"
  | "Evidence"
  | "History"
  | "Controls";

export interface MissionCanvasModel {
  mission: string;
  trajectory: string;
  nextAction: string;
  workpointId: string;
  workItemId: string;
  workRailDetails: string[];
  projectRoot: string;
  continuityId: string;
  evidenceRefs: string[];
  blockers: string[];
  sessions: string[];
  workSurfaces: string[];
  workSurfaceDetails: string[][];
  contention: string[];
  researchArtifacts: string[];
  history: string[];
  contextStatus: string;
  roleStatus: string;
  interviewStatus: string;
  specStatus: string;
  workLoopStatus: string;
  scopeStatus: string;
  workspaceProfile: string;
  visualVariant: string;
  steeringQueue?: string[];
  followUpQueue?: string[];
}

interface Contribution {
  id: string;
  title: string;
  lines: string[];
  tone: Tone;
  priority: number;
}

type Tone = "purple" | "green" | "blue" | "amber" | "red" | "cyan";
type RGB = readonly [number, number, number];

const ACTIVITIES: MissionCanvasActivity[] = [
  "Overview",
  "Context",
  "Role",
  "Interview",
  "Spec",
  "Tasks / Work",
  "Sessions",
  "Documents",
  "Research",
  "Evidence",
  "History",
  "Controls",
];

const PROFILES = [
  { id: "general", label: "General", defaultActivity: "Overview" },
  { id: "software", label: "Software Engineering", defaultActivity: "Tasks / Work" },
  { id: "legal", label: "Legal", defaultActivity: "Documents" },
  { id: "markets", label: "Markets", defaultActivity: "Overview" },
  { id: "research", label: "Research", defaultActivity: "Research" },
  { id: "custom", label: "Custom", defaultActivity: "Overview" },
] as const;

const COLORS = {
  canvas: [8, 13, 20] as RGB,
  panel: [13, 21, 32] as RGB,
  panelRaised: [17, 27, 40] as RGB,
  border: [35, 50, 70] as RGB,
  text: [231, 237, 245] as RGB,
  muted: [139, 153, 172] as RGB,
  purple: [143, 92, 246] as RGB,
  green: [67, 211, 131] as RGB,
  blue: [73, 163, 255] as RGB,
  amber: [244, 184, 75] as RGB,
  red: [255, 99, 112] as RGB,
  cyan: [52, 207, 226] as RGB,
};

const MAX_ROWS = 8;
const EMPTY_TRUTH = /^(?:unavailable|unknown|none|no (?:active|canonical|durable|evidence|history|research|session|spec|trajectory|work)|.*not (?:available|loaded|reported|found|defined))\b/i;
const PLACEHOLDER_TRUTH = /(?:adapter-unavailable|\bunbound\b|\bunknown\b|\bno-workpoint\b|:\s*none\b|0\/0\s*$)/i;

function clean(value: unknown): string {
  return String(value ?? "")
    .replace(/\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))/g, "")
    .replace(/[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function meaningful(value: unknown): boolean {
  const text = clean(value);
  return Boolean(text) && !EMPTY_TRUTH.test(text) && !PLACEHOLDER_TRUTH.test(text);
}

const NON_PRINTABLE_ASCII = /[^\x20-\x7e]/;
const ANSI_SEQUENCE = /\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))/g;
const WIDE_UNICODE = /[\u1100-\u115f\u2e80-\ua4cf\uac00-\ud7a3\uf900-\ufaff\ufe10-\ufe6f\uff00-\uff60\uffe0-\uffe6]|[\u{1f300}-\u{1faff}]/u;

function visibleWidthFast(text: string): number {
  const plain = text.replace(ANSI_SEQUENCE, "");
  if (!NON_PRINTABLE_ASCII.test(plain)) return plain.length;
  return WIDE_UNICODE.test(plain) ? visibleWidth(text) : Array.from(plain).length;
}

function truncateFast(text: string, width: number): string {
  const safeWidth = Math.max(1, width);
  return visibleWidthFast(text) <= safeWidth ? text : truncateToWidth(text, safeWidth);
}

function wrapFast(text: string, width: number): string[] {
  const safeWidth = Math.max(1, width);
  if (NON_PRINTABLE_ASCII.test(text) || text.length <= safeWidth) {
    return NON_PRINTABLE_ASCII.test(text) ? wrapTextWithAnsi(text, safeWidth) : [text];
  }

  const lines: string[] = [];
  let remaining = text;
  while (remaining.length > safeWidth) {
    let breakAt = remaining.lastIndexOf(" ", safeWidth);
    if (breakAt <= 0) breakAt = safeWidth;
    lines.push(remaining.slice(0, breakAt));
    remaining = remaining.slice(breakAt).trimStart();
  }
  lines.push(remaining);
  return lines;
}

function useful(values: readonly unknown[] | undefined): string[] {
  const result: string[] = [];
  for (const value of values ?? []) {
    const text = clean(value);
    if (!text || EMPTY_TRUTH.test(text) || PLACEHOLDER_TRUTH.test(text)) continue;
    result.push(text);
    if (result.length >= MAX_ROWS) break;
  }
  return result;
}

function fg(rgb: RGB): string {
  return `\x1b[38;2;${rgb[0]};${rgb[1]};${rgb[2]}m`;
}

function bg(rgb: RGB): string {
  return `\x1b[48;2;${rgb[0]};${rgb[1]};${rgb[2]}m`;
}

function paint(text: string, color: RGB = COLORS.text, background?: RGB, bold = false): string {
  return `${background ? bg(background) : ""}${fg(color)}${bold ? "\x1b[1m" : ""}${text}\x1b[0m`;
}

function filled(text: string, width: number, color: RGB, background: RGB, bold = false): string {
  const clipped = truncateFast(text, width);
  const usedWidth = visibleWidthFast(clipped);
  const padding = " ".repeat(Math.max(0, width - usedWidth));
  return `${bg(background)}${fg(color)}${bold ? "\x1b[1m" : ""}${clipped}${padding}\x1b[0m`;
}

function toneColor(tone: Tone): RGB {
  return COLORS[tone];
}

function projectName(root: string): string {
  const parts = root.split(/[\\/]/).filter(Boolean);
  return parts.at(-1) || "Project";
}

function labelForProfile(id: string): string {
  return PROFILES.find((profile) => profile.id === id)?.label ?? "General";
}

function contribution(id: string, title: string, values: unknown[], tone: Tone, priority: number): Contribution | undefined {
  const lines = useful(values);
  return lines.length ? { id, title, lines, tone, priority } : undefined;
}

function graphLines(profile: string): string[] {
  if (profile === "research") return ["Security ── Auth refactor ── Reliability", "    ╲          │          ╱", "   Auditability ─────── UX impact"];
  if (profile === "markets") return ["Reliability thesis  ▁▂▂▄▅▆▆█", "Bull 62%  ·  Base 28%  ·  Bear 10%"];
  return ["User repo ── Auth flow ── Hasher", "     ╲         │         ╱", "      Errors ─ Evidence"];
}

function resolveContributions(model: MissionCanvasModel, activity: MissionCanvasActivity, transcript: string[]): Contribution[] {
  const result: Array<Contribution | undefined> = [];
  const surface = useful(model.workSurfaceDetails[0]);
  const evidence = useful(model.evidenceRefs);
  const research = useful(model.researchArtifacts);
  const sessions = useful(model.sessions);
  const history = useful(model.history);
  const blockers = useful(model.blockers);
  const work = useful(model.workRailDetails);
  const steering = useful(model.steeringQueue);
  const followUp = useful(model.followUpQueue);
  const profile = model.workspaceProfile;

  switch (activity) {
    case "Overview":
      result.push(
        contribution("mission", "Mission Status", [model.mission, model.trajectory], "purple", 100),
        contribution("focus", "Today’s Focus", [model.nextAction], "green", 95),
        contribution("work", "Active Work", [model.workpointId, model.workItemId, ...work], "blue", 90),
        contribution("proof", "Evidence Posture", [`${evidence.length} evidence refs`, ...evidence], "green", 85),
        contribution("surface", profile === "markets" ? "Active Thesis" : "Focused Work Surface", profile === "markets" ? graphLines(profile) : surface, "purple", 80),
        contribution("transcript", "Pi Transcript · live", transcript, "blue", 75)
      );
      break;
    case "Context":
      result.push(
        contribution("facts", "Canonical Facts", [model.contextStatus, model.projectRoot, model.continuityId, "Generated action · /focusa-context"], "purple", 100),
        contribution("graph", profile === "research" ? "Claim / Source Graph" : "Semantic Graph", graphLines(profile), "blue", 90),
        contribution("freshness", "Freshness", [model.scopeStatus, model.workLoopStatus], "green", 80),
        contribution("conflicts", "Conflicts", [...blockers, ...model.contention], "red", 75)
      );
      break;
    case "Role":
      result.push(contribution("role", "Role Profile", [model.roleStatus, "Generated action · /focusa-role"], "purple", 100), contribution("authority", "Authority", evidence, "green", 80));
      break;
    case "Interview":
      result.push(contribution("interview", "Grill Interview", [model.interviewStatus, "Generated action · /focusa-interview"], "purple", 100), contribution("questions", "Open Decisions", blockers, "amber", 80));
      break;
    case "Spec":
      result.push(contribution("spec", "Governed Specification", [model.specStatus, model.nextAction, "Generated action · /focusa-crist"], "purple", 100), contribution("proof", "Acceptance Evidence", evidence, "green", 80));
      break;
    case "Tasks / Work":
      result.push(
        contribution("surface", profile === "legal" ? "Current Matter" : "Focused Work Surface", surface, "purple", 100),
        contribution("transcript", "Pi Transcript · live", transcript, "blue", 95),
        contribution("workpoint", meaningful(model.workpointId) ? "Current Workpoint" : "Next Step", meaningful(model.workpointId) ? [model.workpointId, model.nextAction] : [model.nextAction], "purple", 90),
        contribution("rail", "Work Rail", work.length ? [...work, "Generated action · /focusa-rail"] : [], "blue", 85),
        contribution("proof", "Evidence / Authority", evidence, "green", 80),
        contribution("blockers", "Contention", blockers, "red", 75)
      );
      break;
    case "Sessions":
      result.push(contribution("sessions", "Multiplexed Runtime Inventory", sessions, "green", 100), contribution("surface", "Attachment / Isolation", model.workSurfaces, "blue", 90), contribution("history", "Session Activity", history, "purple", 80));
      break;
    case "Documents":
      result.push(contribution("document", profile === "legal" ? "Requirements · Redline" : "Documents", [...research, ...surface], "purple", 100), contribution("authority", "Authorities / Sources", evidence, "blue", 85), contribution("deadlines", "Deadlines / Evidence", blockers, "amber", 75));
      break;
    case "Research":
      result.push(contribution("sources", "Source Matrix", research, "green", 100), contribution("graph", "Claim / Source Graph", graphLines(profile), "blue", 90), contribution("synthesis", "Synthesis", [model.contextStatus, model.trajectory], "purple", 80), contribution("contradictions", "Contradictions", blockers, "red", 75));
      break;
    case "Evidence":
      result.push(contribution("matrix", "Evidence Matrix", evidence, "green", 100), contribution("readiness", "Proof Readiness", [model.specStatus, model.nextAction], "purple", 90), contribution("gaps", "Gaps", blockers, "amber", 80), contribution("history", "Receipts / Promotion", history, "blue", 70));
      break;
    case "History":
      result.push(contribution("history", "Receipt-backed History", history, "purple", 100), contribution("evidence", "Provenance", evidence, "green", 80));
      break;
    case "Controls":
      result.push(contribution("controls", "Governed Controls", ["Canvas on/off · /mission-canvas", "Work Surfaces · /focusa-surfaces (open, focus, pin, group, split, compare)", "New Workpoint · /focus-work", "Workspace profile · project preference", "Preview/commit for authority-changing actions"], "purple", 100));
      break;
  }

  if (steering.length) result.push({ id: "steering", title: "Steering Queue", lines: steering, tone: "purple", priority: 65 });
  if (followUp.length) result.push({ id: "follow-up", title: "Follow-up Queue", lines: followUp, tone: "blue", priority: 60 });
  return result.filter((item): item is Contribution => Boolean(item)).sort((a, b) => b.priority - a.priority);
}

/** Pi-native authoritative Mission Canvas. Canonical state remains external. */
export class MissionCanvasView implements Component {
  private activity: MissionCanvasActivity = "Overview";
  private selectedSurface = 0;
  private conversation: string[] = [];
  private refreshing = false;
  private renderCache?: { width: number; signature: string; lines: string[] };
  private readonly layoutMemory = new Map<string, MissionCanvasActivity>();
  private readonly refreshTimer: ReturnType<typeof setInterval>;

  constructor(
    private model: MissionCanvasModel,
    private readonly _theme: Theme,
    private readonly requestRender: () => void,
    private readonly close: () => void,
    private readonly reload: () => Promise<MissionCanvasModel>,
    private readonly copyReference: (reference: string) => void,
    private readonly changeWorkspaceProfile?: (profile: string) => void
  ) {
    this.activity = (PROFILES.find((profile) => profile.id === model.workspaceProfile)?.defaultActivity ?? "Overview") as MissionCanvasActivity;
    this.refreshTimer = setInterval(() => void this.refresh(), 5_000);
  }

  setConversation(rows: string[]): void {
    const next = rows.map(clean).filter(Boolean).slice(-8);
    if (next.join("\n") === this.conversation.join("\n")) return;
    this.conversation = next;
    this.invalidate();
  }

  invalidate(): void {
    this.renderCache = undefined;
  }

  dispose(): void {
    clearInterval(this.refreshTimer);
  }

  handleInput(data: string): void {
    if (data === "mode-prev") this.selectActivity(-1);
    else if (data === "mode-next") this.selectActivity(1);
    else if (data === "surface-prev") this.selectSurface(-1);
    else if (data === "surface-next") this.selectSurface(1);
    else if (data === "profile-prev") this.selectProfile(-1);
    else if (data === "profile-next") this.selectProfile(1);
    else if (data === "refresh") void this.refresh();
    else if (data === "copy") this.copyReference(this.model.workpointId || this.model.workItemId || this.model.continuityId);
    else if (data === "close") this.close();
  }

  private selectActivity(direction: number): void {
    const current = ACTIVITIES.indexOf(this.activity);
    this.activity = ACTIVITIES[(current + direction + ACTIVITIES.length) % ACTIVITIES.length];
    this.layoutMemory.set(this.model.workspaceProfile, this.activity);
    this.invalidate();
    this.requestRender();
  }

  private selectSurface(direction: number): void {
    const count = Math.max(1, this.model.workSurfaces.length);
    this.selectedSurface = (this.selectedSurface + direction + count) % count;
    this.invalidate();
    this.requestRender();
  }

  private selectProfile(direction: number): void {
    this.layoutMemory.set(this.model.workspaceProfile, this.activity);
    const current = Math.max(0, PROFILES.findIndex((profile) => profile.id === this.model.workspaceProfile));
    const next = PROFILES[(current + direction + PROFILES.length) % PROFILES.length];
    this.model = { ...this.model, workspaceProfile: next.id };
    this.activity = this.layoutMemory.get(next.id) ?? next.defaultActivity;
    this.changeWorkspaceProfile?.(next.id);
    this.invalidate();
    this.requestRender();
  }

  render(width: number): string[] {
    const safeWidth = Math.max(40, width);
    const signature = `${this.activity}|${this.model.workspaceProfile}|${this.selectedSurface}|${this.refreshing}|${this.conversation.join("|")}`;
    if (this.renderCache?.width === safeWidth && this.renderCache.signature === signature) return this.renderCache.lines;

    const rows = [
      ...this.renderTopBar(safeWidth),
      ...this.renderSurfaceStrip(safeWidth),
      ...this.renderWorkspace(safeWidth),
    ];
    this.renderCache = { width: safeWidth, signature, lines: rows };
    return rows;
  }

  private renderTopBar(width: number): string[] {
    const project = projectName(this.model.projectRoot);
    const profile = labelForProfile(this.model.workspaceProfile);
    const session = meaningful(this.model.continuityId) ? this.model.continuityId : "current Pi session";
    const context = `  F  Project  ${project}   Workstream  ${clean(this.model.workItemId) || "Current"}   Workspace  ${profile}`;
    const state = `  ● Session Active   Pi · ${truncateFast(session, Math.max(12, width - 46))}   ${this.activity}   Canvas ●`;
    return [
      filled(context, width, COLORS.text, COLORS.canvas, true),
      filled(state, width, COLORS.muted, COLORS.canvas),
    ];
  }

  private renderSurfaceStrip(width: number): string[] {
    const surfaces = this.model.workSurfaces.length ? this.model.workSurfaces : ["Pi · current session"];
    const window = virtualWindow(surfaces, this.selectedSurface, surfaceCapacity(width));
    const labels = window.values.map((surface, offset) => {
      const selected = window.start + offset === this.selectedSurface;
      return selected ? paint(` ${clean(surface)} `, COLORS.text, COLORS.purple, true) : paint(` ${clean(surface)} `, COLORS.muted, COLORS.panel);
    });
    return [filled(` ${labels.join(" ")}`, width, COLORS.muted, COLORS.canvas)];
  }

  private renderWorkspace(width: number): string[] {
    const mode = responsiveCanvasMode(width);
    const preferences = accessibilityPreferences();
    const narrow = mode === "narrow" || width < 72;
    const railWidth = narrow ? 0 : Math.min(18, Math.max(14, Math.floor(width * 0.16)));
    const mainWidth = Math.max(30, width - railWidth - (railWidth ? 1 : 0));
    const contributions = resolveContributions(this.model, this.activity, this.conversation);
    const main = this.renderContributionGrid(contributions, mainWidth);

    if (narrow) {
      const activityTabs = ACTIVITIES.map((activity) => activity === this.activity ? paint(` ${activity} `, COLORS.text, COLORS.purple, true) : paint(` ${activity} `, COLORS.muted, COLORS.panel)).join(" ");
      return [filled(` ${truncateFast(activityTabs, width - 1)}`, width, COLORS.muted, COLORS.canvas), ...main.map((line) => filled(line, width, COLORS.text, COLORS.canvas)), filled(`  Ctrl+↑/↓ mode · Ctrl+←/→ profile · Alt+←/→ surface · ${preferences.highContrast ? "high contrast" : "adaptive color"} · ${preferences.reducedMotion ? "reduced motion" : "state transitions"}`, width, COLORS.muted, COLORS.canvas)];
    }

    const rail = ACTIVITIES.map((activity) => activity === this.activity ? filled(`  ◆ ${activity}`, railWidth, COLORS.text, COLORS.purple, true) : filled(`  ◇ ${activity}`, railWidth, COLORS.muted, COLORS.panel));
    const height = Math.max(rail.length, main.length);
    const rows = Array.from({ length: height }, (_, index) => {
      const left = rail[index] ?? filled("", railWidth, COLORS.muted, COLORS.panel);
      const right = main[index] ?? filled("", mainWidth, COLORS.text, COLORS.canvas);
      return `${left}${filled(" ", 1, COLORS.muted, COLORS.canvas)}${right}`;
    });
    rows.push(filled(`  Ctrl+↑/↓ activity · Ctrl+←/→ profile · Alt+←/→ Work Surface · ${preferences.highContrast ? "high contrast" : "adaptive color"} · ${preferences.reducedMotion ? "reduced motion" : "state transitions"}`, width, COLORS.muted, COLORS.canvas));
    return rows;
  }

  private renderContributionGrid(contributions: Contribution[], width: number): string[] {
    if (!contributions.length) return [];
    const columns = width >= 76 ? 2 : 1;
    const gap = columns === 2 ? 1 : 0;
    const columnWidth = columns === 2 ? Math.floor((width - gap) / 2) : width;
    const cards = contributions.map((item) => this.renderCard(item, columnWidth));
    if (columns === 1) return cards.flatMap((card, index) => index ? [filled("", width, COLORS.text, COLORS.canvas), ...card] : card);

    const rows: string[] = [];
    for (let index = 0; index < cards.length; index += 2) {
      const left = cards[index];
      const right = cards[index + 1] ?? [];
      const height = Math.max(left.length, right.length);
      for (let row = 0; row < height; row++) {
        rows.push(`${left[row] ?? filled("", columnWidth, COLORS.text, COLORS.canvas)}${filled(" ", gap, COLORS.text, COLORS.canvas)}${right[row] ?? filled("", columnWidth, COLORS.text, COLORS.canvas)}`);
      }
      if (index + 2 < cards.length) rows.push(filled("", width, COLORS.text, COLORS.canvas));
    }
    return rows;
  }

  private renderCard(item: Contribution, width: number): string[] {
    const color = toneColor(item.tone);
    const inner = Math.max(10, width - 2);
    const title = ` ${item.title} `;
    const top = `${paint("┌", color, COLORS.panel)}${paint(truncateFast(`${title}${"─".repeat(inner)}`, inner), color, COLORS.panel, true)}${paint("┐", color, COLORS.panel)}`;
    const body = item.lines.flatMap((line) => wrapFast(line, Math.max(1, inner - 2))).slice(0, MAX_ROWS).map((line, index) => {
      const marker = index === 0 ? "● " : "  ";
      return `${paint("│", COLORS.border, COLORS.panel)}${filled(` ${marker}${line}`, inner, index === 0 ? color : COLORS.text, COLORS.panel)}${paint("│", COLORS.border, COLORS.panel)}`;
    });
    const bottom = `${paint("└", COLORS.border, COLORS.panel)}${paint("─".repeat(inner), COLORS.border, COLORS.panel)}${paint("┘", COLORS.border, COLORS.panel)}`;
    return [top, ...body, bottom];
  }

  private async refresh(): Promise<void> {
    if (this.refreshing) return;
    this.refreshing = true;
    this.invalidate();
    this.requestRender();
    try {
      const profile = this.model.workspaceProfile;
      this.model = await this.reload();
      this.model.workspaceProfile = profile;
      this.selectedSurface = Math.min(this.selectedSurface, Math.max(0, this.model.workSurfaces.length - 1));
    } finally {
      this.refreshing = false;
      this.invalidate();
      this.requestRender();
    }
  }
}
