import type { ExtensionAPI, ExtensionContext, Theme } from "@earendil-works/pi-coding-agent";
import { Input, Key, matchesKey, truncateToWidth, visibleWidth, type Component } from "@earendil-works/pi-tui";
import { MissionCanvasView, type MissionCanvasModel } from "./mission-canvas-view.js";

let activeShell: MissionCanvasShell | undefined;

export function closeActiveMissionCanvasShell(): boolean {
  if (!activeShell) return false;
  activeShell.closeShell();
  return true;
}

export function hasActiveMissionCanvasShell(): boolean {
  return Boolean(activeShell);
}

function contentText(value: unknown): string {
  if (typeof value === "string") return value;
  if (Array.isArray(value)) {
    return value
      .map((part) =>
        typeof part === "string"
          ? part
          : part && typeof part === "object" && "text" in part
            ? String((part as { text?: unknown }).text ?? "")
            : ""
      )
      .filter(Boolean)
      .join(" ");
  }
  return "";
}

function recentConversation(ctx: ExtensionContext): string[] {
  const entries = ctx.sessionManager.getEntries?.() ?? [];
  const rows: string[] = [];
  for (const entry of entries.slice(-16) as any[]) {
    const message = entry?.message ?? entry;
    const role = String(message?.role ?? entry?.type ?? "event").toUpperCase();
    const body = contentText(message?.content ?? entry?.content).replace(/\s+/g, " ").trim();
    if (!body || !["USER", "ASSISTANT", "TOOL", "TOOLRESULT"].some((kind) => role.startsWith(kind))) continue;
    rows.push(`${role.padEnd(9)} ${body}`);
  }
  return rows.slice(-5);
}

/**
 * Authoritative Pi-native Mission Canvas. It replaces only Pi's visible root
 * while mounted; the same ExtensionContext, SessionManager, model stream,
 * tools, editor target, and history remain active in the current terminal.
 */
export class MissionCanvasShell implements Component {
  private readonly input = new Input();
  private readonly canvas: MissionCanvasView;
  private readonly refreshTimer: ReturnType<typeof setInterval>;
  private disposed = false;

  constructor(
    model: MissionCanvasModel,
    private readonly theme: Theme,
    private readonly requestRender: () => void,
    private readonly terminalRows: () => number,
    private readonly done: () => void,
    private readonly reload: () => Promise<MissionCanvasModel>,
    private readonly pi: ExtensionAPI,
    private readonly ctx: ExtensionContext,
    copyReference: (reference: string) => void,
    changeWorkspaceProfile: (profile: string) => void,
    private readonly disableCanvas: () => void
  ) {
    activeShell?.closeShell();
    activeShell = this;
    this.canvas = new MissionCanvasView(
      model,
      theme,
      requestRender,
      () => {},
      reload,
      copyReference,
      changeWorkspaceProfile
    );
    this.ctx.ui.setTitle("Focusa Mission Canvas");
    this.ctx.ui.setFooter((_tui, footerTheme) => ({
      render: (width: number) => [
        truncateToWidth(
          footerTheme.fg(
            "accent",
            `FOCUSA MISSION CANVAS · CURRENT PI SESSION · ${this.ctx.model?.id ?? "model unavailable"} · /mission-canvas off`
          ),
          Math.max(1, width)
        ),
      ],
      invalidate() {},
    }));
    this.input.focused = true;
    this.input.onSubmit = (value) => {
      const prompt = value.trim();
      if (!prompt) return;
      this.input.setValue("");
      if (prompt === "/mission-canvas off" || prompt === "/canvas off") {
        // Route through the agent-first tool queue. Closing a custom shell from
        // its own editor-submit stack is re-entrant in Pi; the tool transition
        // runs after input dispatch and safely reveals the stock root.
        void this.pi.sendUserMessage(
          "Use the focusa_mission_canvas tool with action off now. Do not call any other tool."
        );
        return;
      }
      void this.pi.sendUserMessage(prompt);
      this.requestRender();
    };
    this.refreshTimer = setInterval(() => this.requestRender(), 250);
  }

  closeShell(): void {
    if (this.disposed) return;
    const draft = this.input.getValue();
    this.done();
    if (draft) queueMicrotask(() => this.ctx.ui.setEditorText(draft));
  }

  handleInput(data: string): void {
    if (matchesKey(data, Key.ctrl("up"))) {
      this.canvas.handleInput("mode-prev");
      return;
    }
    if (matchesKey(data, Key.ctrl("down"))) {
      this.canvas.handleInput("mode-next");
      return;
    }
    if (matchesKey(data, Key.ctrl("left"))) {
      this.canvas.handleInput("profile-prev");
      return;
    }
    if (matchesKey(data, Key.ctrl("right"))) {
      this.canvas.handleInput("profile-next");
      return;
    }
    if (matchesKey(data, Key.alt("left"))) {
      this.canvas.handleInput("surface-prev");
      return;
    }
    if (matchesKey(data, Key.alt("right"))) {
      this.canvas.handleInput("surface-next");
      return;
    }
    if (matchesKey(data, Key.ctrl("r"))) {
      this.canvas.handleInput("refresh");
      return;
    }
    if (matchesKey(data, Key.ctrl("y"))) {
      this.canvas.handleInput("copy");
      return;
    }
    if (matchesKey(data, Key.escape)) {
      this.input.setValue("");
      this.requestRender();
      return;
    }
    this.input.handleInput(data);
    this.requestRender();
  }

  invalidate(): void {
    this.canvas.invalidate();
    this.input.invalidate();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    clearInterval(this.refreshTimer);
    this.canvas.dispose();
    this.ctx.ui.setFooter(undefined);
    this.ctx.ui.setTitle("Pi");
    if (activeShell === this) activeShell = undefined;
  }

  render(width: number): string[] {
    const safeWidth = Math.max(40, width);
    this.canvas.setConversation(recentConversation(this.ctx));
    const canvasRows = this.canvas.render(safeWidth);
    const inputRows = this.input.render(Math.max(1, safeWidth - 6));
    const fixedRows = canvasRows.length + inputRows.length + 4;
    const fillRows = Math.max(0, this.terminalRows() - fixedRows - 1);
    const canvasFill = `\x1b[48;2;8;13;20m${" ".repeat(safeWidth)}\x1b[0m`;
    const promptLabel = " PROMPT EDITOR · To: Pi · current session · New Workpoint: /focus-work ";
    const promptTop = `┌${promptLabel}${"─".repeat(Math.max(0, safeWidth - visibleWidth(promptLabel) - 2))}┐`;
    return [
      ...canvasRows,
      ...Array.from({ length: fillRows }, () => canvasFill),
      this.theme.fg("accent", truncateToWidth(promptTop, safeWidth)),
      ...inputRows.map((line) => `${this.theme.fg("dim", "│ ")}${truncateToWidth(line, safeWidth - 4)}${" ".repeat(Math.max(0, safeWidth - 4 - visibleWidth(line)))}${this.theme.fg("dim", " │")}`),
      this.theme.fg("dim", `└${"─".repeat(Math.max(1, safeWidth - 2))}┘`),
      truncateToWidth(this.theme.fg("muted", "Enter send · Ctrl+↑/↓ activity · Ctrl+←/→ workspace · Alt+←/→ Work Surface · /mission-canvas off restores stock Pi"), safeWidth),
    ];
  }
}
