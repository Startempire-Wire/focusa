import type { AttachmentId, WorkSurfaceId } from '$lib/mission-canvas/types';
import type { PiAttachmentStore } from './pi-attachment-store.svelte';
import type { PiNativeCommand, PiTerminalGeometry } from './pi-attachment-contract';
import type { PtyAttachmentIdentity, PtyCommand, PtyTerminalGeometry } from './pty-contract';
import type { PtyHandle } from './pty-handle';

/**
 * PTY-013 — Agent TUI bridge: authentic terminal renderer stream + controls.
 *
 * The bridge adapts the platform-neutral PtyHandle (PTY-003) to the terminal
 * renderer. Every command is issued through the SAME governed Attachment that
 * the store holds; the renderer is a live PTY stream — there is no static
 * static terminal text. Controls (interrupt/resize/…) activate only for the exact
 * current run generation.
 */

export interface PiTerminalOutput {
  attachment_id: AttachmentId;
  work_surface_id: WorkSurfaceId;
  generation: number;
  sequence: number;
  data: string;
}

export interface PiTerminalBridge {
  send(command: PiNativeCommand): Promise<void>;
  subscribeOutput(attachmentId: AttachmentId, listener: (output: PiTerminalOutput) => void): Promise<() => void> | (() => void);
}

function commandIdentity(store: PiAttachmentStore): PtyAttachmentIdentity | undefined {
  const identity = store.identity;
  if (!identity) return undefined;
  return identity as unknown as PtyAttachmentIdentity;
}

/**
 * Build the bridge over a PtyHandle + the module-level store. Output events
 * are accepted ONLY when they identify the exact current Attachment, current
 * run generation, and a monotonic in-range sequence — anything else is
 * rejected and never reaches the terminal surface (PTY-014).
 */
export function createAgentTuiBridge(handle: PtyHandle, store: PiAttachmentStore): PiTerminalBridge {
  return {
    async send(command: PiNativeCommand): Promise<void> {
      const identity = commandIdentity(store);
      if (!identity) return; // fail closed: no exact Attachment, no command
      switch (command.kind) {
        case 'attach':
          await handle.attach(identity, command.geometry as PtyTerminalGeometry);
          break;
        case 'input': {
          const accepted = await handle.write(command.data);
          if (!accepted) store.error('Input rejected: no attached process for the exact Attachment.');
          break;
        }
        case 'resize':
          await handle.resize(command.geometry as PtyTerminalGeometry);
          break;
        case 'interrupt':
          await handle.interrupt();
          break;
        case 'detach':
          await handle.detach();
          break;
        case 'close':
          await handle.close();
          break;
        case 'restart':
          await handle.restart();
          break;
      }
    },

    subscribeOutput(attachmentId, listener) {
      const unsubscribe = handle.onEvent((event) => {
        // PTY-014: a prior Attachment, generation, or sequence can never
        // update the terminal surface.
        const eventAttachmentId = event.attachment_key.attachment_id;
        const currentIdentity = commandIdentity(store);
        const surfaceMatches = currentIdentity !== undefined
          && event.work_surface_id === currentIdentity.work_surface_id;
        if (eventAttachmentId !== attachmentId) return;
        if (event.kind !== 'output') return;
        if (!surfaceMatches) return;
        if (!store.acceptsOutput(event.generation, event.sequence)) return;
        listener({
          attachment_id: eventAttachmentId,
          work_surface_id: event.work_surface_id,
          generation: event.generation,
          sequence: event.sequence,
          data: event.data
        });
      });
      return unsubscribe;
    }
  };
}
