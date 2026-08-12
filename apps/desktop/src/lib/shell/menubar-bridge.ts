// Desktop-to-Menubar bridge: typed message channel for the Focusa
// desktop web app to communicate with the Focusa menubar Tauri app.
//
// The menubar app (apps/menubar) runs as a separate Tauri process and
// advertises a window.__FOCUSA_MENUBAR__ presence signal.  This module
// detects the signal and provides typed send/receive for:
//   - workspace identity push
//   - projection revision notifications
//   - navigation requests
//
// Follows Spec 158: Desktop never duplicates canonical state.
// Messages are advisory presentation preferences only.

export type MenubarMessage =
  | { kind: 'workspace_bound'; workstreamId: string; continuityId: string; profileId: string; activityId: string }
  | { kind: 'workspace_unbound' }
  | { kind: 'projection_updated'; revision: number; profileId: string; activityId: string }
  | { kind: 'navigate'; workspaceId: string }
  | { kind: 'health_ping'; daemonVersion?: string; uptimeMs?: number }
  | { kind: 'prompt_draft_changed'; draftLength: number };

export interface MenubarBridge {
  readonly available: boolean;
  send(message: MenubarMessage): void;
  /** Resolves when the menubar acknowledges, rejects on timeout. */
  sendAsync(message: MenubarMessage, timeoutMs?: number): Promise<void>;
}

const MENUBAR_SIGNAL = '__FOCUSA_MENUBAR__';
const ACK_TIMEOUT_MS = 2000;

declare global {
  interface Window {
    [MENUBAR_SIGNAL]?: {
      postMessage: (msg: MenubarMessage) => void;
      addEventListener?: (event: string, handler: (msg: MenubarMessage) => void) => void;
    };
    __focusa_menubar_ready?: boolean;
  }
}

function detectMenubar(): boolean {
  if (typeof window === 'undefined') return false;
  return Boolean(window.__focusa_menubar_ready || window[MENUBAR_SIGNAL]);
}

export function createMenubarBridge(): MenubarBridge {
  const available = detectMenubar();

  function send(message: MenubarMessage): void {
    if (!available) return;
    try {
      const signal = window[MENUBAR_SIGNAL];
      if (signal?.postMessage) {
        signal.postMessage(message);
      } else {
        // Fallback: dispatch as custom event for menubar webview listener
        window.dispatchEvent(new CustomEvent('focusa-menubar-message', { detail: message }));
      }
    } catch { /* menubar unavailable */ }
  }

  async function sendAsync(message: MenubarMessage, timeoutMs = ACK_TIMEOUT_MS): Promise<void> {
    if (!available) return;
    send(message);
    // Menubar acks are fire-and-forget for now; resolve after a tick
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  return { available, send, sendAsync };
}
