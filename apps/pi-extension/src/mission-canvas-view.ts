import type { Theme } from "@earendil-works/pi-coding-agent";
import { Key, matchesKey, truncateToWidth, wrapTextWithAnsi, type Component } from "@earendil-works/pi-tui";

export type MissionCanvasPanel =
  | "Now"
  | "Work"
  | "Sessions"
  | "Contention"
  | "Proof"
  | "Research"
  | "History"
  | "Context"
  | "Role"
  | "Interview"
  | "Spec"
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
}

const PANELS: MissionCanvasPanel[] = [
  "Now",
  "Work",
  "Sessions",
  "Contention",
  "Proof",
  "Research",
  "History",
  "Context",
  "Role",
  "Interview",
  "Spec",
  "Controls",
];

function text(value: unknown, fallback = "Unavailable"): string {
  const clean = String(value ?? "")
    .replace(/\s+/g, " ")
    .trim();
  return clean || fallback;
}

const MAX_VISIBLE_ROWS = 40;

function rows(values: string[], empty: string): string[] {
  if (!values.length) return [`  ${empty}`];
  const visible = values.slice(0, MAX_VISIBLE_ROWS).map((value) => `  • ${text(value)}`);
  if (values.length > MAX_VISIBLE_ROWS) {
    visible.push(`  … ${values.length - MAX_VISIBLE_ROWS} more rows; refine the focused projection`);
  }
  return visible;
}

/** Keyboard-first, Pi-native Mission Canvas. Canonical state remains external. */
export class MissionCanvasView implements Component {
  private selected = 0;
  private selectedSurface = 0;
  private refreshing = false;
  private readonly refreshTimer: ReturnType<typeof setInterval>;

  constructor(
    private model: MissionCanvasModel,
    private readonly theme: Theme,
    private readonly requestRender: () => void,
    private readonly close: () => void,
    private readonly reload: () => Promise<MissionCanvasModel>,
    private readonly copyReference: (reference: string) => void
  ) {
    // Bounded reconnect/degraded fallback; canonical event projection remains authoritative.
    this.refreshTimer = setInterval(() => void this.refresh(), 5_000);
  }

  invalidate(): void {}

  dispose(): void {
    clearInterval(this.refreshTimer);
  }

  handleInput(data: string): void {
    const key = data.toLowerCase();
    if (key === "r") {
      void this.refresh();
      return;
    }
    if (key === "y") {
      this.copyReference(this.model.workpointId || this.model.workItemId || this.model.continuityId);
      return;
    }
    const panelKeys: Partial<Record<string, MissionCanvasPanel>> = {
      n: "Now",
      w: "Work",
      s: "Sessions",
      p: "Proof",
      e: "Proof",
      h: "History",
      c: "Controls",
    };
    if (panelKeys[key]) {
      this.selected = PANELS.indexOf(panelKeys[key]!);
      this.requestRender();
      return;
    }
    if (matchesKey(data, Key.escape) || matchesKey(data, Key.ctrl("c"))) {
      this.close();
      return;
    }
    if (matchesKey(data, Key.alt("left"))) {
      const count = Math.max(1, this.model.workSurfaces.length);
      this.selectedSurface = (this.selectedSurface - 1 + count) % count;
      this.requestRender();
      return;
    }
    if (matchesKey(data, Key.alt("right"))) {
      const count = Math.max(1, this.model.workSurfaces.length);
      this.selectedSurface = (this.selectedSurface + 1) % count;
      this.requestRender();
      return;
    }
    if (matchesKey(data, Key.left) || matchesKey(data, Key.shift("tab"))) {
      this.selected = (this.selected - 1 + PANELS.length) % PANELS.length;
      this.requestRender();
      return;
    }
    if (matchesKey(data, Key.right) || matchesKey(data, Key.tab)) {
      this.selected = (this.selected + 1) % PANELS.length;
      this.requestRender();
      return;
    }
    const number = Number(data);
    if (Number.isInteger(number) && number >= 1 && number <= PANELS.length) {
      this.selected = number - 1;
      this.requestRender();
    }
  }

  render(width: number): string[] {
    const panel = PANELS[this.selected];
    const lines = [
      this.theme.fg(
        "accent",
        this.theme.bold(
          `FOCUSA MISSION CANVAS · ${text(this.model.workspaceProfile).toUpperCase()} · ${text(this.model.visualVariant)}`
        )
      ),
      this.theme.fg(
        "muted",
        `${this.model.scopeStatus} · ${text(this.model.projectRoot)} · ${this.refreshing ? "refreshing" : "live"} · N/W/S/P/H/C panels · Y copy ref · R refresh · Esc close · ←/→ panel · Alt+←/→ surface`
      ),
      this.surfaceStrip(),
      "",
      PANELS.map((name, index) =>
        index === this.selected
          ? this.theme.fg("accent", `[${index + 1} ${name}]`)
          : this.theme.fg("dim", `${index + 1} ${name}`)
      ).join("  "),
      "",
      ...this.panelLines(panel),
    ];
    return lines.flatMap((line) =>
      wrapTextWithAnsi(line, Math.max(1, width)).map((part) => truncateToWidth(part, Math.max(1, width)))
    );
  }

  private async refresh(): Promise<void> {
    if (this.refreshing) return;
    this.refreshing = true;
    this.requestRender();
    try {
      this.model = await this.reload();
      this.selectedSurface = Math.min(this.selectedSurface, Math.max(0, this.model.workSurfaces.length - 1));
    } finally {
      this.refreshing = false;
      this.requestRender();
    }
  }

  private surfaceStrip(): string {
    const surfaces = this.model.workSurfaces.length ? this.model.workSurfaces : ["Current Pi attachment"];
    const start = Math.max(0, Math.min(this.selectedSurface - 3, surfaces.length - 8));
    const visible = surfaces.slice(start, start + 8);
    const labels = visible.map((surface, offset) => {
      const index = start + offset;
      return index === this.selectedSurface
        ? this.theme.fg("accent", `[${text(surface)}]`)
        : this.theme.fg("dim", text(surface));
    });
    if (start > 0) labels.unshift(this.theme.fg("dim", `…${start}`));
    if (start + visible.length < surfaces.length) {
      labels.push(this.theme.fg("dim", `…${surfaces.length - start - visible.length}`));
    }
    return `${this.theme.fg("accent", "WORK SURFACES")}  ${labels.join("  ")}`;
  }

  private panelLines(panel: MissionCanvasPanel): string[] {
    switch (panel) {
      case "Now":
        return this.section(
          "MISSION",
          this.model.mission,
          "TRAJECTORY",
          this.model.trajectory,
          "NEXT SAFE ACTION",
          this.model.nextAction
        );
      case "Work":
        return [
          this.heading("FOCUSED WORK SURFACE"),
          ...rows(
            this.model.workSurfaceDetails[this.selectedSurface] ?? [],
            "Current attachment has no projected surface detail"
          ),
          this.heading("WORK RAIL"),
          ...rows(this.model.workRailDetails, "No canonical Work Rail item detail"),
          `  Loop: ${text(this.model.workLoopStatus)}`,
          this.heading("BLOCKERS"),
          ...rows(this.model.blockers, "No blockers reported"),
        ];
      case "Sessions":
        return [
          this.heading("ACTIVE SESSIONS AND ATTACHMENTS"),
          ...rows(this.model.sessions, "No session inventory available"),
        ];
      case "Contention":
        return [
          this.heading("CONTENTION · PROPOSALS · WRITER LEASES"),
          ...rows(this.model.contention, "No contention reported"),
        ];
      case "Proof":
        return [
          this.heading("EVIDENCE AND RECEIPTS"),
          ...rows(this.model.evidenceRefs, "No evidence linked"),
        ];
      case "Research":
        return [
          this.heading("RESEARCH · SOURCES · RICH ARTIFACTS"),
          ...rows(this.model.researchArtifacts, "No research artifacts projected"),
        ];
      case "History":
        return [
          this.heading("RECEIPT-BACKED HISTORY"),
          ...rows(this.model.history, "No durable history projected"),
        ];
      case "Context":
        return this.section("C · CONTEXT", this.model.contextStatus, "CONTINUITY", this.model.continuityId);
      case "Role":
        return this.section("R · ROLE", this.model.roleStatus);
      case "Interview":
        return this.section("I · INTERVIEW", this.model.interviewStatus);
      case "Spec":
        return this.section("S · SPEC", this.model.specStatus);
      case "Controls":
        return [
          this.heading("CONTROLS"),
          "  /mission-canvas-mode canvas|terminal|headless",
          "  /focus-work to bind or resume focused work",
          "  /focusa-status for daemon and attachment status",
          "  Mutations remain preview/commit governed; this view never changes authority.",
        ];
    }
  }

  private heading(value: string): string {
    return this.theme.fg("accent", this.theme.bold(value));
  }

  private section(...pairs: string[]): string[] {
    const lines: string[] = [];
    for (let index = 0; index < pairs.length; index += 2) {
      lines.push(this.heading(pairs[index]), `  ${text(pairs[index + 1])}`);
    }
    return lines;
  }
}
