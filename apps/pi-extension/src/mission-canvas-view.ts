import type { Theme } from "@earendil-works/pi-coding-agent";
import {
  Key,
  matchesKey,
  truncateToWidth,
  visibleWidth,
  wrapTextWithAnsi,
  type Component,
} from "@earendil-works/pi-tui";
import {
  accessibilityPreferences,
  accessibleStateLabel,
  focusRestorationLabel,
  responsiveCanvasMode,
  surfaceCapacity,
  virtualWindow,
} from "./mission-canvas-accessibility.js";

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

const WORKSPACE_PROFILES = [
  { id: "general", label: "General" },
  { id: "software", label: "Software Engineering" },
  { id: "legal", label: "Legal" },
  { id: "markets", label: "Markets" },
  { id: "research", label: "Research" },
  { id: "custom", label: "Custom" },
] as const;

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
  private renderCache?: { width: number; lines: string[] };
  private readonly refreshTimer: ReturnType<typeof setInterval>;

  constructor(
    private model: MissionCanvasModel,
    private readonly theme: Theme,
    private readonly requestRender: () => void,
    private readonly close: () => void,
    private readonly reload: () => Promise<MissionCanvasModel>,
    private readonly copyReference: (reference: string) => void,
    private readonly changeWorkspaceProfile?: (profile: string) => void
  ) {
    // Bounded reconnect/degraded fallback; canonical event projection remains authoritative.
    this.refreshTimer = setInterval(() => void this.refresh(), 5_000);
  }

  invalidate(): void {
    this.renderCache = undefined;
  }

  dispose(): void {
    clearInterval(this.refreshTimer);
  }

  handleInput(data: string): void {
    this.renderCache = undefined;
    const key = data.toLowerCase();
    if (key === "r") {
      void this.refresh();
      return;
    }
    if (key === "y") {
      this.copyReference(this.model.workpointId || this.model.workItemId || this.model.continuityId);
      return;
    }
    if (data === "[" || data === "]") {
      const current = Math.max(
        0,
        WORKSPACE_PROFILES.findIndex((profile) => profile.id === this.model.workspaceProfile)
      );
      const direction = data === "]" ? 1 : -1;
      const next = WORKSPACE_PROFILES[(current + direction + WORKSPACE_PROFILES.length) % WORKSPACE_PROFILES.length];
      this.model = { ...this.model, workspaceProfile: next.id };
      this.changeWorkspaceProfile?.(next.id);
      this.requestRender();
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
    const safeWidth = Math.max(1, width);
    if (this.renderCache?.width === safeWidth) return this.renderCache.lines;

    const panel = PANELS[this.selected];
    const mode = responsiveCanvasMode(width);
    const preferences = accessibilityPreferences();
    const workspaceSelector = WORKSPACE_PROFILES.map((profile) =>
      profile.id === this.model.workspaceProfile
        ? this.theme.fg("accent", `● ${profile.label}`)
        : this.theme.fg("dim", `○ ${profile.label}`)
    ).join("  ");
    const navigationHelp =
      mode === "narrow"
        ? "←/→ panel · [/] workspace · Esc close"
        : "N/W/S/P/H/C panels · [/] workspace · Y copy ref · R refresh · Esc close";
    const header = [
      this.theme.fg("accent", this.theme.bold("FOCUSA WORKSPACE SYSTEM · MISSION CANVAS")),
      this.theme.fg("muted", "One Runtime. Many Workspaces. Same mission, Workpoint, evidence, and history."),
      `${this.theme.fg("accent", "WORKSPACE")}  ${workspaceSelector}`,
      this.theme.fg(
        "muted",
        `${accessibleStateLabel("pi", this.refreshing ? "refreshing" : "live", "attachment-scoped")} · ${this.model.scopeStatus} · ${text(this.model.projectRoot)} · layout:${mode} · contrast:${preferences.highContrast ? "high" : "theme"} · motion:${preferences.reducedMotion ? "reduced" : "state-change-only"} · ${focusRestorationLabel(preferences)}`
      ),
      this.surfaceStrip(width),
      "",
      this.theme.fg("accent", "CURRENT WORKSPACE COCKPIT"),
      PANELS.map((name, index) =>
        index === this.selected
          ? this.theme.fg("accent", `[${index + 1} ${name}]`)
          : this.theme.fg("dim", `${index + 1} ${name}`)
      ).join("  "),
      "",
    ];
    const body = mode === "desktop" ? this.dashboardLines(panel, safeWidth) : this.panelLines(panel);
    const wrap = (lines: string[]) =>
      lines.flatMap((line) =>
        wrapTextWithAnsi(line, safeWidth).map((part) => truncateToWidth(part, safeWidth))
      );
    const rendered =
      mode === "desktop"
        ? [
            // Dashboard cards already wrap their content; avoid a second ANSI wrap pass.
            ...wrap(header),
            ...body.map((line) => truncateToWidth(line, safeWidth)),
            "",
            ...wrap([this.theme.fg("muted", navigationHelp)]),
          ]
        : wrap([...header, ...body, "", this.theme.fg("muted", navigationHelp)]);
    this.renderCache = { width: safeWidth, lines: rendered };
    return rendered;
  }

  private dashboardLines(panel: MissionCanvasPanel, width: number): string[] {
    const gap = 2;
    const columnWidth = Math.max(28, Math.floor((width - gap) / 2));
    const panelRows = this.panelLines(panel);
    const dashboardRows = panelRows.slice(0, 12);
    if (panelRows.length > dashboardRows.length) {
      dashboardRows.push(this.theme.fg("dim", `… ${panelRows.length - dashboardRows.length} more; open the focused panel`));
    }
    const current = this.card(
      `CURRENT ${panel.toUpperCase()}`,
      dashboardRows,
      columnWidth
    );
    const next = this.card(
      "NEXT UP",
      [
        this.theme.fg("accent", text(this.model.nextAction)),
        `Task  ${text(this.model.workItemId)}`,
        `Loop  ${text(this.model.workLoopStatus)}`,
        `Proof ${this.model.evidenceRefs.length} evidence reference(s)`,
        `Surface ${text(this.model.workSurfaces[this.selectedSurface], "Current Pi attachment")}`,
      ],
      columnWidth
    );
    const top = this.joinCards(current, next, columnWidth, gap);
    const changes = this.card(
      "WHAT CHANGES",
      [
        `✓ Workspace layout: ${text(this.model.workspaceProfile)}`,
        `✓ Terminology and focused panel: ${panel}`,
        `✓ Work Surface emphasis`,
        `✓ Visual variant: ${text(this.model.visualVariant)}`,
      ],
      columnWidth
    );
    const stable = this.card(
      "WHAT STAYS THE SAME",
      [
        `✓ Mission: ${text(this.model.mission)}`,
        `✓ Workpoint: ${text(this.model.workpointId)}`,
        `✓ Session: ${text(this.model.continuityId)}`,
        `✓ Canonical evidence and history`,
      ],
      columnWidth
    );
    return [...top, "", ...this.joinCards(changes, stable, columnWidth, gap)];
  }

  private card(title: string, body: string[], width: number): string[] {
    const inner = Math.max(8, width - 2);
    const titleText = ` ${title} `;
    const top = `┌${titleText}${"─".repeat(Math.max(0, inner - titleText.length))}┐`;
    const rows = body.flatMap((line) => wrapTextWithAnsi(line, Math.max(1, inner - 2)));
    const rendered = rows.map((line) => {
      const clipped = truncateToWidth(line, Math.max(1, inner - 2));
      const padding = " ".repeat(Math.max(0, inner - 2 - visibleWidth(clipped)));
      return `│ ${clipped}${padding} │`;
    });
    return [this.theme.fg("accent", top), ...rendered, this.theme.fg("dim", `└${"─".repeat(inner)}┘`)];
  }

  private joinCards(left: string[], right: string[], columnWidth: number, gap: number): string[] {
    const height = Math.max(left.length, right.length);
    const blank = " ".repeat(columnWidth);
    return Array.from({ length: height }, (_, index) => {
      const leftLine = left[index] ?? blank;
      const leftPadding = " ".repeat(Math.max(0, columnWidth - visibleWidth(leftLine)));
      return `${leftLine}${leftPadding}${" ".repeat(gap)}${right[index] ?? blank}`;
    });
  }

  private async refresh(): Promise<void> {
    if (this.refreshing) return;
    this.refreshing = true;
    this.renderCache = undefined;
    this.requestRender();
    try {
      this.model = await this.reload();
      this.selectedSurface = Math.min(this.selectedSurface, Math.max(0, this.model.workSurfaces.length - 1));
    } finally {
      this.refreshing = false;
      this.renderCache = undefined;
      this.requestRender();
    }
  }

  private surfaceStrip(width: number): string {
    const surfaces = this.model.workSurfaces.length ? this.model.workSurfaces : ["Current Pi attachment"];
    const window = virtualWindow(surfaces, this.selectedSurface, surfaceCapacity(width));
    const { start, values: visible } = window;
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
