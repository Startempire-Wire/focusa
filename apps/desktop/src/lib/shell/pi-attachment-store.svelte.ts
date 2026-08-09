import {
  UNBOUND_PI_ATTACHMENT,
  hasExactPiAttachment,
  type PiAttachmentIdentity,
  type PiAttachmentProjection,
  type PiNativeCommand,
  type PiTerminalGeometry
} from './pi-attachment-contract';
import { evaluatePtyOutput, type PtyCommand, type PtyRunGeneration } from './pty-contract';

/**
 * PTY-001 frontend Pi attachment state machine.
 *
 * The store is a module-level singleton so Mission Canvas and the Agent TUI
 * observe the SAME Pi session and Attachment, and so the attachment identity
 * and command surface survive view switches (Pi remains alive while Mission
 * Canvas is visible). Every transition carries the run generation and a
 * monotonic sequence; output from a stale generation is rejected.
 */

export interface PiAttachmentEnvelope {
  schema: 'focusa.desktop.pi_attachment_envelope.v1';
  attachment_key: PiAttachmentIdentity | null;
  attachment_id: PiAttachmentIdentity['attachment_id'] | null;
  work_surface_id: PiAttachmentIdentity['work_surface_id'] | null;
  generation: number;
  sequence: number;
  command: PtyCommand;
}

export class PiAttachmentStore {
  state = $state<PiAttachmentProjection>(UNBOUND_PI_ATTACHMENT);
  #generation = 0;
  #sequence = 0;

  /** Latest dispatched command envelope; the terminal bridge replays it. */
  latestEnvelope = $state<PiAttachmentEnvelope | null>(null);

  /** True while a native runtime is attached with an exact identity. */
  get attached(): boolean {
    return this.state.state === 'attached' && hasExactPiAttachment(this.state);
  }

  get identity(): PiAttachmentIdentity | undefined {
    return this.attached ? (this.state as PiAttachmentProjection & { identity: PiAttachmentIdentity }).identity : undefined;
  }

  bind(identity: PiAttachmentIdentity, geometry: PiTerminalGeometry): void {
    this.#generation += 1;
    this.#sequence = 0;
    this.state = {
      state: 'attached',
      identity,
      runtimeLabel: `Pi · ${identity.session_id}`,
      detail: 'Exact Workstream Attachment bound. Native terminal input is enabled.',
      canWrite: true,
      canSteer: true,
      canInterrupt: true
    };
    this.dispatch({ kind: 'attach', identity, geometry });
  }

  binding(geometry: PiTerminalGeometry): void {
    this.state = {
      state: 'binding',
      runtimeLabel: 'Binding Pi runtime…',
      detail: 'Resolving the exact Workstream Attachment before native input is enabled.',
      canWrite: false,
      canSteer: false,
      canInterrupt: false
    };
  }

  disconnect(reason: string): void {
    this.state = {
      state: 'disconnected',
      runtimeLabel: 'Pi runtime disconnected',
      detail: reason,
      canWrite: false,
      canSteer: false,
      canInterrupt: false
    };
  }

  error(reason: string): void {
    this.state = {
      state: 'error',
      runtimeLabel: 'Pi runtime error',
      detail: reason,
      canWrite: false,
      canSteer: false,
      canInterrupt: false
    };
  }

  detach(): void {
    const identity = this.identity;
    if (identity) this.dispatch({ kind: 'detach', attachment_id: identity.attachment_id });
    this.#generation += 1;
    this.state = UNBOUND_PI_ATTACHMENT;
  }

  /** Interrupt, resize, close, or restart the attached runtime. */
  send(command: Extract<PiNativeCommand, { kind: 'interrupt' | 'resize' | 'close' | 'restart' }>): boolean {
    if (!this.attached) return false;
    this.dispatch(command as PiNativeCommand);
    return true;
  }

  /** Current run generation; controls activate only for this generation. */
  get generation(): number {
    return this.#generation;
  }

  /** Latest dispatched monotonic sequence of the current run. */
  get latestSequence(): number {
    return this.#sequence;
  }

  /**
   * Stale-output rejection: output produced for an earlier run generation is
   * dropped. The terminal must pass the generation it was bound with.
   */
  acceptsOutput(generation: number, sequence: number): boolean {
    return evaluatePtyOutput(generation as PtyRunGeneration, this.#generation, sequence, this.#sequence).accepted;
  }

  private toPtyCommand(command: PiNativeCommand, identity: PiAttachmentIdentity | undefined): PtyCommand {
    if (!identity) return command as unknown as PtyCommand;
    if (command.kind === 'attach') {
      return { kind: 'attach', identity, geometry: command.geometry };
    }
    const base = { attachment_key: identity, work_surface_id: identity.work_surface_id };
    switch (command.kind) {
      case 'input':
        return { ...base, kind: 'input', data: command.data };
      case 'resize':
        return { ...base, kind: 'resize', geometry: command.geometry };
      case 'interrupt':
        return { ...base, kind: 'interrupt' };
      case 'detach':
        return { ...base, kind: 'detach' };
      case 'close':
        return { ...base, kind: 'close' };
      case 'restart':
        return { ...base, kind: 'restart' };
    }
  }

  private dispatch(command: PiNativeCommand): void {
    const identity = this.identity;
    this.#sequence += 1;
    this.latestEnvelope = {
      schema: 'focusa.desktop.pi_attachment_envelope.v1',
      attachment_key: identity ?? null,
      attachment_id: identity?.attachment_id ?? null,
      work_surface_id: identity?.work_surface_id ?? null,
      generation: this.#generation,
      sequence: this.#sequence,
      command: this.toPtyCommand(command, identity)
    };
  }
}

/** Module-level singleton: one Pi session + Attachment for both surfaces. */
export const piAttachmentStore = new PiAttachmentStore();

export function readPiAttachmentStore(): PiAttachmentStore {
  return piAttachmentStore;
}
