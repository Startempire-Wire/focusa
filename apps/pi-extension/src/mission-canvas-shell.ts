import type { ExtensionAPI, ExtensionContext, Theme } from "@earendil-works/pi-coding-agent";
import { Input, Key, matchesKey, truncateToWidth, visibleWidth, type Component, type Focusable } from "@earendil-works/pi-tui";
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

export function refreshActiveMissionCanvasShell(): boolean {
  if (!activeShell) return false;
  activeShell.refreshFromEvent();
  return true;
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
 * Pi-native Mission Canvas overlay. It leaves Pi's transcript and editor
 * mounted underneath, owns input only while visible, and always provides a
 * direct Escape/Ctrl+G path back to the normal terminal.
 */
export class MissionCanvasShell implements Component, Focusable {
  private readonly input = new Input();
  private readonly canvas: MissionCanvasView;
  private disposed = false;
  private _focused = false;
  private scrollOffset = 0;
  private canvasRowCount = 0;
  private viewportRows = 1;
  private eventRefreshTimer?: ReturnType<typeof setTimeout>;

  get focused(): boolean {
    return this._focused;
  }

  set focused(value: boolean) {
    this._focused = value;
    this.input.focused = value;
  }

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
    changeVisualVariant: (variant: string) => void,
    private readonly manageSurfaces: () => Promise<void>,
    private readonly disableCanvas: () => Promise<void>
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
      changeWorkspaceProfile,
      changeVisualVariant
    );
    this.focused = true;
    this.input.onSubmit = (value) => {
      const prompt = value.trim();
      if (!prompt) return;
      this.input.setValue("");
      if (prompt === "/mission-canvas off" || prompt === "/canvas off") {
        // Defer until the Input submit stack unwinds, then use the same
        // controller as the slash command and agent tool. This closes the
        // native shell immediately without spending an agent turn.
        queueMicrotask(() => {
          void this.disableCanvas().catch((error) =>
            this.ctx.ui.notify(`Mission Canvas close failed: ${String(error)}`, "error")
          );
        });
        return;
      }
      this.closeShell();
      void this.pi.sendUserMessage(prompt);
    };
  }

  closeShell(): void {
    if (this.disposed) return;
    const draft = this.input.getValue();
    // Pi's custom UI does not guarantee that `done()` synchronously disposes
    // the component. Tear down timers and references before handing control
    // back so a closed Canvas cannot continue requesting terminal redraws.
    this.dispose();
    this.done();
    if (draft) queueMicrotask(() => this.ctx.ui.setEditorText(draft));
  }

  handleInput(data: string): void {
    if (matchesKey(data, Key.ctrl("o"))) {
      this.closeShell();
      queueMicrotask(() => {
        void this.manageSurfaces().catch((error) =>
          this.ctx.ui.notify(`Mission Canvas Work Surfaces failed: ${String(error)}`, "error")
        );
      });
      return;
    }
    if (matchesKey(data, Key.ctrl("up"))) {
      this.scrollOffset = 0;
      this.canvas.handleInput("mode-prev");
      return;
    }
    if (matchesKey(data, Key.ctrl("down"))) {
      this.scrollOffset = 0;
      this.canvas.handleInput("mode-next");
      return;
    }
    if (matchesKey(data, Key.ctrl("left"))) {
      this.scrollOffset = 0;
      this.canvas.handleInput("profile-prev");
      return;
    }
    if (matchesKey(data, Key.ctrl("right"))) {
      this.scrollOffset = 0;
      this.canvas.handleInput("profile-next");
      return;
    }
    if (matchesKey(data, Key.alt("v"))) {
      this.canvas.handleInput("variant-next");
      return;
    }
    if (matchesKey(data, Key.alt("left"))) {
      this.scrollOffset = 0;
      this.canvas.handleInput("surface-prev");
      return;
    }
    if (matchesKey(data, Key.alt("right"))) {
      this.scrollOffset = 0;
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
    if (matchesKey(data, Key.pageUp)) {
      this.scrollOffset = Math.max(0, this.scrollOffset - this.viewportRows);
      this.requestRender();
      return;
    }
    if (matchesKey(data, Key.pageDown)) {
      const maxOffset = Math.max(0, this.canvasRowCount - this.viewportRows);
      this.scrollOffset = Math.min(maxOffset, this.scrollOffset + this.viewportRows);
      this.requestRender();
      return;
    }
    if (matchesKey(data, Key.escape) || matchesKey(data, Key.ctrl("g"))) {
      this.closeShell();
      return;
    }
    this.input.handleInput(data);
    this.requestRender();
  }

  invalidate(): void {
    this.canvas.invalidate();
    this.input.invalidate();
  }

  refreshFromEvent(): void {
    if (this.disposed || this.eventRefreshTimer) return;
    this.eventRefreshTimer = setTimeout(() => {
      this.eventRefreshTimer = undefined;
      if (!this.disposed) this.canvas.handleInput("refresh");
    }, 150);
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    if (this.eventRefreshTimer) clearTimeout(this.eventRefreshTimer);
    this.eventRefreshTimer = undefined;
    this.canvas.dispose();
    if (activeShell === this) activeShell = undefined;
  }

  render(width: number): string[] {
    const safeWidth = Math.max(1, width);
    this.canvas.setConversation(recentConversation(this.ctx));
    const canvasRows = this.canvas.render(safeWidth);
    const inputRows = this.input.render(Math.max(1, safeWidth - 6));
    const availableRows = Math.max(1, this.terminalRows() - inputRows.length - 3);
    this.canvasRowCount = canvasRows.length;
    this.viewportRows = availableRows;
    this.scrollOffset = Math.min(
      this.scrollOffset,
      Math.max(0, this.canvasRowCount - this.viewportRows)
    );
    const visibleCanvasRows = canvasRows.slice(
      this.scrollOffset,
      this.scrollOffset + availableRows
    );
    if (safeWidth < 8) {
      return [
        ...visibleCanvasRows,
        ...inputRows.map((line) => truncateToWidth(line, safeWidth)),
      ];
    }
    const viewportStatus = this.canvasRowCount > this.viewportRows
      ? `Rows ${this.scrollOffset + 1}-${Math.min(this.canvasRowCount, this.scrollOffset + this.viewportRows)}/${this.canvasRowCount}`
      : `Rows 1-${this.canvasRowCount}/${this.canvasRowCount}`;
    const promptLabel = " PROMPT EDITOR · To: Pi · current session · New Workpoint: /focus-work ";
    const promptTop = `┌${promptLabel}${"─".repeat(Math.max(0, safeWidth - visibleWidth(promptLabel) - 2))}┐`;
    return [
      ...visibleCanvasRows,
      this.theme.fg("accent", truncateToWidth(promptTop, safeWidth)),
      ...inputRows.map((line) => `${this.theme.fg("dim", "│ ")}${truncateToWidth(line, safeWidth - 4)}${" ".repeat(Math.max(0, safeWidth - 4 - visibleWidth(line)))}${this.theme.fg("dim", " │")}`),
      this.theme.fg("dim", `└${"─".repeat(Math.max(1, safeWidth - 2))}┘`),
      truncateToWidth(this.theme.fg("muted", `${viewportStatus} · Ctrl+O surfaces · Alt+V theme · Esc/Ctrl+G close · PgUp/PgDn scroll · Enter send · Ctrl+↑/↓ activity · Alt+←/→ surface`), safeWidth),
    ];
  }
}
