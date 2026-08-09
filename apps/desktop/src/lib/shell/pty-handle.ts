import type { PtyCommand, PtyEvent, PtyEventKind, PtyTerminalGeometry } from './pty-contract';
import type { PtyAttachmentIdentity } from './pty-contract';

/**
 * PTY-003 — platform-neutral PTY handle.
 *
 * One handle is created per governed Pi Attachment. The Agent TUI and the
 * Mission Canvas surface the SAME handle (and therefore the same Pi session
 * and Attachment) through the module-level PiAttachmentStore. The handle
 * speaks the typed PtyCommand/PtyEvent contract from PTY-002; every event
 * carries the AttachmentKey, WorkSurfaceId, run generation, and monotonic
 * sequence. A real PTY library backs the native adapter (Cargo side); the
 * browser fallback is explicitly virtual — no ordinary child-process pipe
 * implementation exists anywhere in the renderer.
 */

export interface PtyHandleOptions {
  /** Native adapter toggle; defaults to Tauri internals detection. */
  adapter?: 'tauri' | 'virtual';
  /** Tauri invoke function, injected to keep the renderer shell-agnostic. */
  invoke?: (command: string, args: Record<string, unknown>) => Promise<unknown>;
}

export interface PtyHandle {
  readonly label: string;
  readonly adapterKind: 'tauri' | 'virtual';
  /** Attach the persistent Pi process for the exact Attachment. */
  attach(identity: PtyAttachmentIdentity, geometry: PtyTerminalGeometry): Promise<void>;
  write(data: string): Promise<boolean>;
  resize(geometry: PtyTerminalGeometry): Promise<boolean>;
  interrupt(): Promise<boolean>;
  detach(): Promise<void>;
  close(): Promise<void>;
  restart(): Promise<void>;
  dispose(): void;
  onEvent(listener: (event: PtyEvent) => void): () => void;
}

export function isPtyCommandKind(value: string): value is PtyCommand['kind'] {
  return value === 'attach' || value === 'input' || value === 'resize' || value === 'interrupt'
    || value === 'detach' || value === 'close' || value === 'restart';
}

export function isPtyEventKind(value: string): value is PtyEventKind {
  return value === 'attached' || value === 'output' || value === 'resized' || value === 'interrupted'
    || value === 'detached' || value === 'closed' || value === 'restarted' || value === 'error'
    || value === 'stale_rejected';
}

export function detectPtyAdapter(): 'tauri' | 'virtual' {
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) return 'tauri';
  return 'virtual';
}

/**
 * Factory: returns the native adapter when Tauri internals are present,
 * otherwise the honest virtual fallback (preview/SSR). Never falls back to an
 * ordinary pipe.
 */
export async function createPtyHandle(options: PtyHandleOptions = {}): Promise<PtyHandle> {
  const adapter = options.adapter ?? detectPtyAdapter();
  if (adapter === 'tauri') {
    const { createTauriPtyHandle } = await import('./tauri-pty-adapter');
    return createTauriPtyHandle(options);
  }
  const { createVirtualPtyHandle } = await import('./virtual-pty-handle');
  return createVirtualPtyHandle();
}
