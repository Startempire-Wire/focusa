import type { ExtensionAPI, ExtensionContext, Theme } from "@earendil-works/pi-coding-agent";
import { Input, Key, matchesKey, truncateToWidth, wrapTextWithAnsi, type Component } from "@earendil-works/pi-tui";
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
 * Terminal compatibility projection for Mission Canvas state. While mounted
 * through ctx.ui.custom(), it temporarily replaces the visible stock Pi TUI,
 * but it remains a pi_terminal_projection—not the Focusa rich GUI. The same
 * ExtensionContext, SessionManager, model stream, tools, and history remain active.
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
    this.ctx.ui.setTitle("Focusa Mission Canvas · Terminal Projection");
    this.ctx.ui.setFooter((_tui, footerTheme) => ({
      render: (width: number) => [
        truncateToWidth(
          footerTheme.fg(
            "accent",
            `FOCUSA TERMINAL CANVAS PROJECTION · SAME SESSION · ${this.ctx.model?.id ?? "model unavailable"} · /mission-canvas off`
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
    this.done();
  }

  handleInput(data: string): void {
    if (matchesKey(data, Key.ctrl("left"))) {
      this.canvas.handleInput("[");
      return;
    }
    if (matchesKey(data, Key.ctrl("right"))) {
      this.canvas.handleInput("]");
      return;
    }
    if (matchesKey(data, Key.alt("left")) || matchesKey(data, Key.alt("right"))) {
      this.canvas.handleInput(data);
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
    const safeWidth = Math.max(20, width);
    const canvasRows = this.canvas
      .render(safeWidth)
      .map((line) => line.replace("Esc close", "Esc clear input"));
    const stream = recentConversation(this.ctx).flatMap((line) =>
      wrapTextWithAnsi(line, safeWidth - 4).map((part) => `│ ${truncateToWidth(part, safeWidth - 4)}`)
    );
    const streamRows = stream.length ? stream : ["│ No conversation messages yet"];
    const inputRows = this.input.render(Math.max(1, safeWidth - 7));
    return [
      ...canvasRows,
      "",
      this.theme.fg("accent", "AGENT STREAM · SAME SESSION"),
      ...streamRows,
      this.theme.fg("dim", `└${"─".repeat(Math.max(1, safeWidth - 1))}`),
      "",
      ...inputRows.map((line, index) =>
        index === 0
          ? `${this.theme.fg("accent", "YOU ")}${line}`
          : `${" ".repeat(6)}${line}`
      ),
      this.theme.fg(
        "muted",
        "Enter send · Ctrl+←/→ workspace · Alt+←/→ Work Surface · /mission-canvas off restores stock Pi"
      ),
    ];
  }
}
