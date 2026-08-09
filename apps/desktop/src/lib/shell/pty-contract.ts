import type { AttachmentKey, RuntimeObjectRef, WorkSurfaceId } from '$lib/mission-canvas/types';

/**
 * PTY-002 — typed command/event contract for the governed Pi Attachment.
 *
 * The Agent TUI and the Mission Canvas share ONE Pi session and Attachment.
 * Every command carries the exact AttachmentKey it targets; every event
 * identifies the AttachmentKey, the WorkSurfaceId it surfaced on, the run
 * generation, and a monotonic sequence. Ordinary child-process pipes are
 * never used — the native PTY bridge (PTY-003+) exchanges these typed
 * commands/events through the PiAttachmentStore singleton.
 */

export type PtyRunGeneration = number;
export type PtyMonotonicSequence = number;

export interface PtyAttachmentIdentity extends AttachmentKey {
  work_surface_id: WorkSurfaceId;
  runtime_object?: RuntimeObjectRef | null;
}

export interface PtyTerminalGeometry {
  columns: number;
  rows: number;
  pixelWidth: number;
  pixelHeight: number;
}

/** Full command surface of the governed Pi Attachment. */
export type PtyCommand =
  | { kind: 'attach'; identity: PtyAttachmentIdentity; geometry: PtyTerminalGeometry }
  | { kind: 'input'; attachment_key: PtyAttachmentIdentity; work_surface_id: WorkSurfaceId; data: string }
  | { kind: 'resize'; attachment_key: PtyAttachmentIdentity; work_surface_id: WorkSurfaceId; geometry: PtyTerminalGeometry }
  | { kind: 'interrupt'; attachment_key: PtyAttachmentIdentity; work_surface_id: WorkSurfaceId }
  | { kind: 'detach'; attachment_key: PtyAttachmentIdentity; work_surface_id: WorkSurfaceId }
  | { kind: 'close'; attachment_key: PtyAttachmentIdentity; work_surface_id: WorkSurfaceId }
  | { kind: 'restart'; attachment_key: PtyAttachmentIdentity; work_surface_id: WorkSurfaceId };

export const PTY_COMMAND_KINDS = ['attach', 'input', 'resize', 'interrupt', 'detach', 'close', 'restart'] as const;

interface PtyEventBase {
  attachment_key: PtyAttachmentIdentity;
  work_surface_id: WorkSurfaceId;
  generation: PtyRunGeneration;
  sequence: PtyMonotonicSequence;
}

/** Every PTY event identifies AttachmentKey, WorkSurfaceId, generation, sequence. */
export type PtyEvent =
  | (PtyEventBase & { kind: 'attached'; geometry: PtyTerminalGeometry })
  | (PtyEventBase & { kind: 'output'; data: string })
  | (PtyEventBase & { kind: 'resized'; geometry: PtyTerminalGeometry })
  | (PtyEventBase & { kind: 'interrupted' })
  | (PtyEventBase & { kind: 'detached' })
  | (PtyEventBase & { kind: 'closed' })
  | (PtyEventBase & { kind: 'restarted' })
  | (PtyEventBase & { kind: 'error'; message: string })
  | (PtyEventBase & { kind: 'stale_rejected'; reason: 'stale_generation' | 'non_monotonic_sequence' });

export type PtyEventKind = PtyEvent['kind'];

export interface PtyOutputDecision {
  accepted: boolean;
  reason?: 'stale_generation' | 'non_monotonic_sequence';
}

/**
 * Stale-output rejection (contract-level, used by the store and the bridge):
 * output is accepted only for the CURRENT run generation and with a sequence
 * that continues the current monotonic run (equal to the last emitted, or the
 * next one). Output from an earlier generation or a replayed sequence is
 * rejected with a typed reason.
 */
export function evaluatePtyOutput(
  generation: PtyRunGeneration,
  latestGeneration: PtyRunGeneration,
  sequence: PtyMonotonicSequence,
  latestSequence: PtyMonotonicSequence
): PtyOutputDecision {
  if (generation !== latestGeneration) {
    return { accepted: false, reason: 'stale_generation' };
  }
  // Output for the current generation may be replayed up to the latest known
  // sequence (reconnect re-sync); anything beyond it is non-monotonic.
  if (sequence < 0 || sequence > latestSequence) {
    return { accepted: false, reason: 'non_monotonic_sequence' };
  }
  return { accepted: true };
}

/** Monotonic sequence factory; each call yields the next strictly increasing value. */
export function createMonotonicSequence(start = 0): () => PtyMonotonicSequence {
  let value = start;
  return () => {
    value += 1;
    return value;
  };
}

/** Run generation factory; each call yields a strictly increasing generation. */
export function createRunGeneration(start = 0): () => PtyRunGeneration {
  let value = start;
  return () => {
    value += 1;
    return value;
  };
}
