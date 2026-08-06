import {
  sameWorkstreamAuthorityContext,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type {
  WorkstreamAuthorityContext,
  WorkstreamKey,
  WorkstreamOperationRequest
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-types.generated';

/**
 * Generated packets accepted at the Desktop boundary.
 *
 * These are transport-owned shapes, not a Desktop-owned identity DTO.  A
 * WorkstreamKey is the smallest exact packet; the other shapes are accepted
 * only when their generated contract contains the same explicit owner.
 */
export type DesktopCanonicalPacket = WorkstreamKey | WorkstreamAuthorityContext | WorkstreamOperationRequest;

export type DesktopContextValue = Readonly<WorkstreamAuthorityContext>;

export type DesktopContextState =
  | Readonly<{ kind: 'unbound' }>
  | Readonly<{ kind: 'bound'; context: DesktopContextValue }>;

export type DesktopContextClearReason = 'initial' | 'cleared' | 'invalid_packet' | 'context_mismatch';

const UNBOUND_STATE: DesktopContextState = Object.freeze({ kind: 'unbound' });

/**
 * A Svelte-compatible, presentation-only binding to one canonical
 * Workstream.  It stores an exact generated identity packet; it never finds
 * one.  The store is deliberately not persisted because persistence and
 * authority belong to Focusa Core.
 */
export class DesktopContext {
  #state: DesktopContextState = UNBOUND_STATE;
  #clearReason: DesktopContextClearReason = 'initial';
  readonly #listeners = new Set<(state: DesktopContextState) => void>();

  constructor(packet?: DesktopCanonicalPacket) {
    if (packet !== undefined) this.fromCanonicalPacket(packet);
  }

  /** Create a bound store only from an explicit generated canonical packet. */
  static fromCanonicalPacket(packet: DesktopCanonicalPacket): DesktopContext {
    return new DesktopContext().fromCanonicalPacket(packet);
  }

  /** Create an unbound store. */
  static clear(): DesktopContext {
    const context = new DesktopContext();
    context.clear();
    return context;
  }

  /**
   * Replace this store with an exact packet.  A different owner or malformed
   * packet clears the store instead of retaining or repairing the old value.
   */
  fromCanonicalPacket(packet: DesktopCanonicalPacket): this {
    const next = canonicalContext(packet);
    if (!next) {
      this.clear('invalid_packet');
      return this;
    }

    const current = this.value;
    if (current && !sameWorkstreamAuthorityContext(current, next)) {
      this.clear('context_mismatch');
      return this;
    }

    this.#clearReason = 'cleared';
    this.#state = Object.freeze({
      kind: 'bound',
      context: freezeDeep(cloneJson(next))
    });
    this.#notify();
    return this;
  }

  /** Clear the current binding and all subordinate identity. */
  clear(reason: Exclude<DesktopContextClearReason, 'initial'> = 'cleared'): void {
    this.#clearReason = reason;
    this.#state = UNBOUND_STATE;
    this.#notify();
  }

  /** Svelte readable-store contract for consumers that need reactive updates. */
  subscribe(listener: (state: DesktopContextState) => void): () => void {
    this.#listeners.add(listener);
    listener(this.#state);
    return () => this.#listeners.delete(listener);
  }

  get state(): DesktopContextState {
    return this.#state;
  }

  get kind(): DesktopContextState['kind'] {
    return this.#state.kind;
  }

  get snapshot(): DesktopContextState {
    return this.#state;
  }

  get value(): DesktopContextValue | undefined {
    return this.#state.kind === 'bound' ? this.#state.context : undefined;
  }

  get context(): DesktopContextValue | undefined {
    return this.value;
  }

  get workstream(): WorkstreamKey | undefined {
    return this.value?.workstream;
  }

  get isBound(): boolean {
    return this.#state.kind === 'bound';
  }

  get clearReason(): DesktopContextClearReason {
    return this.#clearReason;
  }

  #notify(): void {
    for (const listener of this.#listeners) listener(this.#state);
  }
}

/**
 * Read identity only from one of the generated packet shapes.  This is an
 * explicit DTO adaptation, not a resolver: there is no candidate search,
 * repair, or fallback source.
 */
function canonicalContext(packet: unknown): WorkstreamAuthorityContext | undefined {
  if (validGeneratedPacket('WorkstreamKey', packet)) {
    const workstream = packet as WorkstreamKey;
    return {
      workstream,
      continuity_id: null,
      attachment: null,
      workspace_binding_id: null,
      runtime_object: null,
      work_surface_id: null
    };
  }

  if (validGeneratedPacket('WorkstreamAuthorityContext', packet)) {
    return packet as WorkstreamAuthorityContext;
  }

  if (validGeneratedPacket('WorkstreamOperationRequest', packet)) {
    const request = packet as WorkstreamOperationRequest;
    if (!request.authority
      || !request.authority.envelope
      || request.authority.envelope.status !== 'canonical'
      || typeof request.authority.authority_ref !== 'string'
      || request.authority.authority_ref.trim().length === 0
      || typeof request.authority.envelope.why !== 'string'
      || request.authority.envelope.why.trim().length === 0
      || !request.actor
      || typeof request.actor.actor_id !== 'string'
      || request.actor.actor_id.trim().length === 0) {
      return undefined;
    }
    return {
      workstream: request.workstream,
      continuity_id: request.continuity_id ?? request.attachment?.continuity_id ?? null,
      attachment: request.attachment ?? null,
      workspace_binding_id: request.workspace_binding_id ?? request.attachment?.workspace_binding_id ?? null,
      runtime_object: request.runtime_object ?? null,
      work_surface_id: request.work_surface_id ?? null
    };
  }

  return undefined;
}

function validGeneratedPacket(schema: string, packet: unknown): boolean {
  return validateMissionCanvasContract(schema, packet).valid;
}

function cloneJson<T>(value: T): T {
  if (Array.isArray(value)) return value.map((item) => cloneJson(item)) as T;
  if (value && typeof value === 'object') {
    const copy: Record<string, unknown> = {};
    for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
      copy[key] = cloneJson(item);
    }
    return copy as T;
  }
  return value;
}

function freezeDeep<T>(value: T, seen = new WeakSet<object>()): T {
  if (!value || typeof value !== 'object' || seen.has(value as object)) return value;
  seen.add(value as object);
  for (const child of Object.values(value as Record<string, unknown>)) freezeDeep(child, seen);
  return Object.freeze(value);
}

export default DesktopContext;
