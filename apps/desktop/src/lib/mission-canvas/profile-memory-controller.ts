import type {
  MissionCanvasClient,
  MissionCanvasOperationInput
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import {
  sameWorkstreamAuthorityContext,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type {
  ProfileLayoutMemory,
  WorkstreamAuthorityContext
} from './types';
import type { RecompositionReceipt } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-types.generated';

export type ProfileMemoryViewportClass = ProfileLayoutMemory['viewport_class'];

export interface ProfileMemoryBinding {
  scope: WorkstreamAuthorityContext;
  profileId: string;
  activityModeId: string;
  viewportClass: ProfileMemoryViewportClass;
}

/**
 * The controller consumes generated operation results through this narrow
 * adapter.  `get` and `update` intentionally return canonical semantic memory,
 * not a layout tree or a renderer model.  `updateWithReceipt` is optional so
 * test/integration adapters can retain the generated Receipt cursor without
 * changing the durable-memory surface used by the controller.
 */
export interface ProfileMemoryTransport {
  get(binding: ProfileMemoryBinding): Promise<ProfileLayoutMemory>;
  update(memory: ProfileLayoutMemory): Promise<ProfileLayoutMemory>;
  updateWithReceipt?: (memory: ProfileLayoutMemory) => Promise<ProfileMemoryUpdateResult>;
}

export interface ProfileMemoryUpdateResult {
  memory: ProfileLayoutMemory;
  receipt?: RecompositionReceipt;
}

export type GeneratedProfileMemoryClient = Pick<
  MissionCanvasClient,
  'layout_memoryGet' | 'layout_memoryUpdate'
>;

export type ProfileMemoryState =
  | { kind: 'unbound' }
  | { kind: 'blocked'; binding?: ProfileMemoryBinding; reason: string }
  | { kind: 'loading'; binding: ProfileMemoryBinding }
  | { kind: 'ready'; binding: ProfileMemoryBinding; memory: ProfileLayoutMemory }
  | { kind: 'saving'; binding: ProfileMemoryBinding; memory: ProfileLayoutMemory }
  | {
      kind: 'conflict';
      binding: ProfileMemoryBinding;
      memory: ProfileLayoutMemory;
      pendingMemory?: ProfileLayoutMemory;
      reason: string;
    }
  | { kind: 'error'; binding?: ProfileMemoryBinding; reason: string };

export type ProfileMemoryStateListener = (state: ProfileMemoryState) => void;

const VIEWPORT_CLASSES = new Set<ProfileMemoryViewportClass>([
  'minimum',
  'compact',
  'standard',
  'productive',
  'wide',
  'reference_capture'
]);

const REGION_KINDS = new Set([
  'primary',
  'secondary',
  'inspector',
  'rail',
  'queue',
  'composer',
  'navigation',
  'overlay'
]);

const CONTRIBUTION_ID = /^contribution:[a-z0-9][a-z0-9._:-]{0,159}$/;

/**
 * Adapt the generated read operation without deriving a profile, activity,
 * viewport, or scope from a projection or a local tab.  The generated client
 * remains responsible for operation metadata, route/path, DTO validation,
 * permission, and transport watermark checks.
 */
export async function layoutMemoryGet(
  client: GeneratedProfileMemoryClient,
  binding: ProfileMemoryBinding
): Promise<ProfileLayoutMemory> {
  const normalizedBinding = normalizeBinding(binding);
  if (!normalizedBinding) throw controllerError('invalid_workstream_authority');

  const value = await client.layout_memoryGet({
    ...cloneJson(normalizedBinding.scope),
    profile_id: normalizedBinding.profileId,
    activity_mode_id: normalizedBinding.activityModeId,
    viewport_class: normalizedBinding.viewportClass
  } as MissionCanvasOperationInput);
  assertMemoryMatches(value, normalizedBinding);
  return cloneJson(value);
}

/**
 * Adapt the generated mutation operation.  Core returns a direct
 * RecompositionReceipt, so the controller does not pretend that the receipt is
 * a layout-memory response.  It is checked against the exact submitted
 * semantic memory and its next revision before a transport can materialize the
 * accepted memory for the controller state.
 */
export async function layoutMemoryUpdate(
  client: GeneratedProfileMemoryClient,
  memory: ProfileLayoutMemory
): Promise<RecompositionReceipt> {
  const normalizedMemory = normalizeMemory(memory);
  if (!normalizedMemory) throw controllerError('invalid_profile_memory');

  const receipt = await client.layout_memoryUpdate(
    cloneJson(normalizedMemory) as MissionCanvasOperationInput
  );
  assertReceiptMatches(receipt, normalizedMemory);
  return cloneJson(receipt);
}

/**
 * Generated transport adapter used by the Desktop Mission Canvas runtime.
 * There is no local layout resolver here: an accepted Receipt advances only
 * the submitted semantic memory revision and timestamp; contribution
 * eligibility and canonical geometry remain Core-owned.
 */
export class GeneratedProfileMemoryTransport implements ProfileMemoryTransport {
  constructor(private readonly client: GeneratedProfileMemoryClient) {}

  get(binding: ProfileMemoryBinding): Promise<ProfileLayoutMemory> {
    return layoutMemoryGet(this.client, binding);
  }

  async update(memory: ProfileLayoutMemory): Promise<ProfileLayoutMemory> {
    return (await this.updateWithReceipt(memory)).memory;
  }

  async updateWithReceipt(memory: ProfileLayoutMemory): Promise<ProfileMemoryUpdateResult> {
    const normalizedMemory = normalizeMemory(memory);
    if (!normalizedMemory) throw controllerError('invalid_profile_memory');
    const receipt = await layoutMemoryUpdate(this.client, normalizedMemory);
    const next = materializeAcceptedMemory(normalizedMemory, receipt);
    assertMemoryMatches(next, {
      scope: authorityFromMemory(normalizedMemory),
      profileId: normalizedMemory.profile_id,
      activityModeId: normalizedMemory.activity_mode_id,
      viewportClass: normalizedMemory.viewport_class
    });
    return { memory: next, receipt };
  }
}

export function createProfileMemoryTransport(
  client: GeneratedProfileMemoryClient
): GeneratedProfileMemoryTransport {
  return new GeneratedProfileMemoryTransport(client);
}

export class ProfileMemoryControllerError extends Error {
  constructor(readonly code: string) {
    super(code);
    this.name = 'ProfileMemoryControllerError';
  }
}

function controllerError(code: string): ProfileMemoryControllerError {
  return new ProfileMemoryControllerError(code);
}

/**
 * A bounded, non-authoritative controller for one exact profile/activity/
 * viewport memory binding.  It never filters contributions, calculates a
 * layout, reserves absent geometry, or substitutes a missing response.
 */
export class MissionCanvasProfileMemoryController {
  state: ProfileMemoryState = { kind: 'unbound' };
  #generation = 0;
  #listeners = new Set<ProfileMemoryStateListener>();
  #watermarks = new Map<string, ProfileMemoryWatermark>();
  readonly #transport: ProfileMemoryTransport;

  constructor(
    transport: ProfileMemoryTransport | GeneratedProfileMemoryClient
  ) {
    this.#transport = isGeneratedClient(transport)
      ? new GeneratedProfileMemoryTransport(transport)
      : transport;
  }

  subscribe(listener: ProfileMemoryStateListener): () => void {
    this.#listeners.add(listener);
    listener(this.state);
    return () => this.#listeners.delete(listener);
  }

  async load(binding: ProfileMemoryBinding): Promise<void> {
    const normalizedBinding = normalizeBinding(binding);
    if (!normalizedBinding) {
      this.setState({ kind: 'blocked', reason: 'invalid_workstream_authority' });
      return;
    }

    const generation = ++this.#generation;
    const prior = this.memoryForBinding(normalizedBinding);
    this.setState({ kind: 'loading', binding: normalizedBinding });
    try {
      const memory = await this.#transport.get(normalizedBinding);
      if (generation !== this.#generation) return;
      const normalizedMemory = normalizeMemory(memory);
      if (!normalizedMemory) {
        this.setState({ kind: 'error', binding: normalizedBinding, reason: 'invalid_profile_memory' });
        return;
      }
      if (!matches(normalizedMemory, normalizedBinding)) {
        this.setState({ kind: 'error', binding: normalizedBinding, reason: 'foreign_profile_memory' });
        return;
      }

      const key = profileMemoryKey(normalizedBinding);
      const previousWatermark = this.#watermarks.get(key);
      if (previousWatermark && normalizedMemory.memory_revision < previousWatermark.memoryRevision) {
        this.setConflictOrError(
          normalizedBinding,
          prior,
          'stale_profile_memory_revision'
        );
        return;
      }
      this.#watermarks.set(key, {
        memoryRevision: normalizedMemory.memory_revision,
        cursor: previousWatermark?.cursor
      });
      this.setState({
        kind: 'ready',
        binding: normalizedBinding,
        memory: freezeMemory(normalizedMemory)
      });
    } catch (error) {
      if (generation !== this.#generation) return;
      this.setConflictOrError(normalizedBinding, prior, errorMessage(error));
    }
  }

  async update(memory: ProfileLayoutMemory): Promise<void> {
    const current = this.state;
    if (current.kind !== 'ready' && current.kind !== 'conflict') return;

    const binding = current.binding;
    const prior = current.memory;
    const normalizedMemory = normalizeMemory(memory);
    if (!normalizedMemory || !matches(normalizedMemory, binding)) {
      this.setState({
        kind: 'conflict',
        binding,
        memory: prior,
        reason: normalizedMemory ? 'foreign_profile_memory' : 'invalid_profile_memory'
      });
      return;
    }
    if (normalizedMemory.memory_revision !== prior.memory_revision) {
      this.setState({
        kind: 'conflict',
        binding,
        memory: prior,
        pendingMemory: freezeMemory(normalizedMemory),
        reason: 'profile_memory_revision_conflict'
      });
      return;
    }

    const generation = ++this.#generation;
    const pendingMemory = freezeMemory(normalizedMemory);
    this.setState({ kind: 'saving', binding, memory: pendingMemory });
    try {
      const result = this.#transport.updateWithReceipt
        ? await this.#transport.updateWithReceipt(normalizedMemory)
        : { memory: await this.#transport.update(normalizedMemory) };
      if (generation !== this.#generation) return;

      const next = normalizeMemory(result.memory);
      if (!next || !matches(next, binding)) {
        this.setState({
          kind: 'conflict',
          binding,
          memory: prior,
          pendingMemory,
          reason: 'foreign_profile_memory'
        });
        return;
      }
      const expectedRevision = prior.memory_revision + 1;
      if (next.memory_revision !== expectedRevision) {
        this.setState({
          kind: 'conflict',
          binding,
          memory: prior,
          pendingMemory,
          reason: next.memory_revision < expectedRevision
            ? 'stale_profile_memory_revision'
            : 'profile_memory_revision_ambiguous'
        });
        return;
      }
      const receipt = result.receipt;
      if (receipt) {
        assertReceiptMatches(receipt, normalizedMemory);
        const key = profileMemoryKey(binding);
        const priorWatermark = this.#watermarks.get(key);
        const cursor = parseCursor(receipt.event_cursor);
        if (!cursor) {
          this.setState({
            kind: 'conflict',
            binding,
            memory: prior,
            pendingMemory,
            reason: 'invalid_profile_memory_cursor'
          });
          return;
        }
        if (priorWatermark?.cursor
          && (priorWatermark.cursor.kind !== cursor.kind
            || cursor.value < priorWatermark.cursor.value)) {
          this.setState({
            kind: 'conflict',
            binding,
            memory: prior,
            pendingMemory,
            reason: priorWatermark.cursor.kind !== cursor.kind
              ? 'profile_memory_cursor_namespace_mismatch'
              : 'stale_profile_memory_cursor'
          });
          return;
        }
        this.#watermarks.set(key, { memoryRevision: next.memory_revision, cursor });
      } else {
        this.#watermarks.set(profileMemoryKey(binding), {
          memoryRevision: next.memory_revision,
          cursor: this.#watermarks.get(profileMemoryKey(binding))?.cursor
        });
      }
      this.setState({ kind: 'ready', binding, memory: freezeMemory(next) });
    } catch (error) {
      if (generation !== this.#generation) return;
      this.setState({
        kind: 'conflict',
        binding,
        memory: prior,
        pendingMemory,
        reason: errorMessage(error)
      });
    }
  }

  clear(): void {
    this.#generation += 1;
    this.setState({ kind: 'unbound' });
  }

  private memoryForBinding(binding: ProfileMemoryBinding): ProfileLayoutMemory | undefined {
    const current = this.state;
    if ((current.kind !== 'ready' && current.kind !== 'saving' && current.kind !== 'conflict')
      || !sameBinding(current.binding, binding)) {
      return undefined;
    }
    return current.memory;
  }

  private setConflictOrError(
    binding: ProfileMemoryBinding,
    prior: ProfileLayoutMemory | undefined,
    reason: string
  ): void {
    if (prior && isConflictReason(reason)) {
      this.setState({ kind: 'conflict', binding, memory: prior, reason });
    } else {
      this.setState({ kind: 'error', binding, reason });
    }
  }

  private setState(state: ProfileMemoryState): void {
    this.state = state;
    for (const listener of this.#listeners) listener(this.state);
  }
}

type ProfileMemoryWatermark = {
  memoryRevision: number;
  cursor?: ProfileMemoryCursor;
};

type ProfileMemoryCursor = { kind: string; value: number };

function isGeneratedClient(
  value: ProfileMemoryTransport | GeneratedProfileMemoryClient
): value is GeneratedProfileMemoryClient {
  return typeof (value as GeneratedProfileMemoryClient).layout_memoryGet === 'function'
    && typeof (value as GeneratedProfileMemoryClient).layout_memoryUpdate === 'function';
}

function normalizeBinding(value: unknown): ProfileMemoryBinding | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined;
  const candidate = value as Record<string, unknown>;
  const scope = candidate.scope;
  if (!validateAuthority(scope)) return undefined;
  if (typeof candidate.profileId !== 'string' || candidate.profileId.trim() === '') return undefined;
  if (typeof candidate.activityModeId !== 'string' || candidate.activityModeId.trim() === '') return undefined;
  if (!isViewportClass(candidate.viewportClass)) return undefined;
  return {
    scope: cloneJson(scope as WorkstreamAuthorityContext),
    profileId: candidate.profileId,
    activityModeId: candidate.activityModeId,
    viewportClass: candidate.viewportClass
  };
}

function normalizeMemory(value: unknown): ProfileLayoutMemory | undefined {
  const structural = validateMissionCanvasContract('ProfileLayoutMemory', value);
  if (!structural.valid || !value || typeof value !== 'object' || Array.isArray(value)) return undefined;
  const memory = value as ProfileLayoutMemory;
  if (!validateAuthority(memory)) return undefined;
  if (typeof memory.profile_id !== 'string' || memory.profile_id.trim() === '') return undefined;
  if (typeof memory.activity_mode_id !== 'string' || memory.activity_mode_id.trim() === '') return undefined;
  if (!isViewportClass(memory.viewport_class)) return undefined;
  if (memory.memory_id !== `layout-memory:${memory.profile_id}:${memory.activity_mode_id}:${memory.viewport_class}`) return undefined;
  if (!Number.isSafeInteger(memory.memory_revision) || memory.memory_revision < 0) return undefined;
  if (typeof memory.idempotency_key !== 'string' || memory.idempotency_key.trim() === '') return undefined;
  if (typeof memory.updated_at !== 'string' || Number.isNaN(Date.parse(memory.updated_at))) return undefined;
  if (memory.focused_semantic_target !== undefined
    && memory.focused_semantic_target !== null
    && (typeof memory.focused_semantic_target !== 'string' || memory.focused_semantic_target.trim() === '')) return undefined;
  if (!Array.isArray(memory.placements) || !Array.isArray(memory.absent_contribution_ids)) return undefined;

  const placementIds = new Set<string>();
  for (const placement of memory.placements) {
    const validation = validateMissionCanvasContract('ContributionPlacementPreference', placement);
    if (!validation.valid || !placement || typeof placement !== 'object' || Array.isArray(placement)) return undefined;
    const value = placement as Record<string, unknown>;
    const contributionId = value.contribution_id;
    const regions = value.preferred_regions;
    const adjacency = value.preferred_adjacency;
    if (typeof contributionId !== 'string' || !CONTRIBUTION_ID.test(contributionId) || placementIds.has(contributionId)) return undefined;
    if (!Array.isArray(regions) || regions.length === 0 || new Set(regions).size !== regions.length
      || regions.some((region) => typeof region !== 'string' || !REGION_KINDS.has(region))) return undefined;
    if (typeof value.preferred_order !== 'number' || !Number.isSafeInteger(value.preferred_order) || value.preferred_order < 0) return undefined;
    if (typeof value.minimum_span !== 'number' || !Number.isSafeInteger(value.minimum_span) || value.minimum_span < 1 || value.minimum_span > 12) return undefined;
    if (typeof value.maximum_span !== 'number' || !Number.isSafeInteger(value.maximum_span) || value.maximum_span < value.minimum_span || value.maximum_span > 12) return undefined;
    if (adjacency !== undefined
      && (!Array.isArray(adjacency)
        || new Set(adjacency).size !== adjacency.length
        || adjacency.some((id) => typeof id !== 'string' || !CONTRIBUTION_ID.test(id)))) return undefined;
    if (value.last_compatible_layout_node_id !== undefined
      && value.last_compatible_layout_node_id !== null
      && (typeof value.last_compatible_layout_node_id !== 'string' || value.last_compatible_layout_node_id.trim() === '')) return undefined;
    placementIds.add(contributionId);
  }

  const absentIds = new Set<string>();
  for (const contributionId of memory.absent_contribution_ids) {
    if (typeof contributionId !== 'string' || !CONTRIBUTION_ID.test(contributionId) || absentIds.has(contributionId)) return undefined;
    absentIds.add(contributionId);
  }
  if ([...placementIds].some((contributionId) => absentIds.has(contributionId))) return undefined;
  return cloneJson(memory);
}

function validateAuthority(value: unknown): boolean {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const authority = value as Record<string, unknown>;
  const authorityValue = {
    workstream: authority.workstream,
    continuity_id: authority.continuity_id ?? null,
    attachment: authority.attachment ?? null,
    workspace_binding_id: authority.workspace_binding_id ?? null,
    runtime_object: authority.runtime_object ?? null,
    work_surface_id: authority.work_surface_id ?? null
  };
  const structural = validateMissionCanvasContract('WorkstreamAuthorityContext', authorityValue);
  if (!structural.valid) return false;
  const workstream = authority.workstream;
  if (!workstream || typeof workstream !== 'object' || Array.isArray(workstream)) return false;
  const workstreamRecord = workstream as Record<string, unknown>;
  if (typeof workstreamRecord.workstream_id !== 'string' || workstreamRecord.workstream_id.trim() === '') return false;
  const scope = workstreamRecord.scope;
  if (!scope || typeof scope !== 'object' || Array.isArray(scope)) return false;
  const scopeRecord = scope as Record<string, unknown>;
  if (scopeRecord.scope_kind !== 'project' && scopeRecord.scope_kind !== 'host') return false;
  const scopeKey = scopeRecord.scope_key;
  if (!scopeKey || typeof scopeKey !== 'object' || Array.isArray(scopeKey)) return false;
  const scopeKeyRecord = scopeKey as Record<string, unknown>;
  for (const field of ['scope_id', 'root_path', 'canonical_name', 'fingerprint']) {
    if (typeof scopeKeyRecord[field] !== 'string' || scopeKeyRecord[field].trim() === '') return false;
  }
  if (scopeKeyRecord.scope_kind !== scopeRecord.scope_kind) return false;
  const attachment = authority.attachment;
  if (attachment !== undefined && attachment !== null) {
    if (!validateMissionCanvasContract('AttachmentKey', attachment)) return false;
    const attachmentRecord = attachment as Record<string, unknown>;
    if (!sameWorkstreamAuthorityContext(
      { workstream },
      { workstream: attachmentRecord.workstream }
    )) return false;
    for (const field of ['instance_id', 'session_id', 'attachment_id', 'workspace_binding_id']) {
      if (typeof attachmentRecord[field] !== 'string' || attachmentRecord[field].trim() === '') return false;
    }
    if (authority.continuity_id != null && authority.continuity_id !== attachmentRecord.continuity_id) return false;
    if (authority.workspace_binding_id != null && authority.workspace_binding_id !== attachmentRecord.workspace_binding_id) return false;
  }
  if (authority.work_surface_id != null && (authority.attachment == null || typeof authority.work_surface_id !== 'string' || authority.work_surface_id.trim() === '')) return false;
  return true;
}

function authorityFromMemory(memory: ProfileLayoutMemory): WorkstreamAuthorityContext {
  return {
    workstream: memory.workstream,
    continuity_id: memory.continuity_id ?? null,
    attachment: memory.attachment ?? null,
    workspace_binding_id: memory.workspace_binding_id ?? null,
    runtime_object: memory.runtime_object ?? null,
    work_surface_id: memory.work_surface_id ?? null
  };
}

function assertMemoryMatches(memory: unknown, binding: ProfileMemoryBinding): asserts memory is ProfileLayoutMemory {
  const normalizedMemory = normalizeMemory(memory);
  if (!normalizedMemory) throw controllerError('invalid_profile_memory');
  if (!matches(normalizedMemory, binding)) throw controllerError('foreign_profile_memory');
}

function assertReceiptMatches(
  receipt: unknown,
  memory: ProfileLayoutMemory
): asserts receipt is RecompositionReceipt {
  const structural = validateMissionCanvasContract('RecompositionReceipt', receipt);
  if (!structural.valid || !receipt || typeof receipt !== 'object' || Array.isArray(receipt)) {
    throw controllerError('invalid_profile_memory_receipt');
  }
  const value = receipt as RecompositionReceipt;
  if (!validateAuthority(value)) throw controllerError('invalid_profile_memory_receipt_authority');
  if (!sameWorkstreamAuthorityContext(value, memory)) throw controllerError('foreign_profile_memory_receipt');
  if (value.accepted !== true) throw controllerError('profile_memory_receipt_not_accepted');
  if (value.idempotency_key !== memory.idempotency_key) throw controllerError('profile_memory_receipt_idempotency_mismatch');
  if (!Number.isSafeInteger(value.projection_revision)
    || !Number.isSafeInteger(value.layout_revision)
    || value.projection_revision < 1
    || value.layout_revision !== memory.memory_revision + 1
    || value.projection_revision !== value.layout_revision) {
    throw controllerError('profile_memory_receipt_revision_mismatch');
  }
  if (typeof value.receipt_id !== 'string' || value.receipt_id.trim() === ''
    || typeof value.evidence_id !== 'string' || value.evidence_id.trim() === ''
    || typeof value.projection_digest !== 'string' || !/^sha256:[a-f0-9]{64}$/.test(value.projection_digest)
    || typeof value.issued_at !== 'string' || Number.isNaN(Date.parse(value.issued_at))) {
    throw controllerError('invalid_profile_memory_receipt');
  }
  if (!parseCursor(value.event_cursor)) throw controllerError('invalid_profile_memory_cursor');
}

function materializeAcceptedMemory(
  memory: ProfileLayoutMemory,
  receipt: RecompositionReceipt
): ProfileLayoutMemory {
  return {
    ...cloneJson(memory),
    memory_revision: receipt.layout_revision,
    updated_at: receipt.issued_at
  };
}

function matches(memory: ProfileLayoutMemory, binding: ProfileMemoryBinding): boolean {
  return sameBinding({
    scope: memory,
    profileId: memory.profile_id,
    activityModeId: memory.activity_mode_id,
    viewportClass: memory.viewport_class
  }, binding);
}

function sameBinding(left: ProfileMemoryBinding, right: ProfileMemoryBinding): boolean {
  return sameWorkstreamAuthorityContext(left.scope, right.scope)
    && left.profileId === right.profileId
    && left.activityModeId === right.activityModeId
    && left.viewportClass === right.viewportClass;
}

function profileMemoryKey(binding: ProfileMemoryBinding): string {
  return JSON.stringify([
    binding.scope,
    binding.profileId,
    binding.activityModeId,
    binding.viewportClass
  ]);
}

function isViewportClass(value: unknown): value is ProfileMemoryViewportClass {
  return typeof value === 'string' && VIEWPORT_CLASSES.has(value as ProfileMemoryViewportClass);
}

function parseCursor(value: unknown): ProfileMemoryCursor | undefined {
  if (typeof value !== 'string') return undefined;
  const normalized = value.trim();
  const prefixed = /^(event|cursor|mission-canvas):([0-9]+)$/.exec(normalized);
  const plain = prefixed ?? /^([0-9]+)$/.exec(normalized);
  if (!plain) return undefined;
  const kind = prefixed ? prefixed[1] : 'opaque-numeric';
  const number = Number(prefixed ? prefixed[2] : plain[1]);
  return Number.isSafeInteger(number) ? { kind, value: number } : undefined;
}

function isConflictReason(reason: string): boolean {
  return reason.startsWith('foreign_')
    || reason.startsWith('stale_')
    || reason.includes('revision')
    || reason.includes('cursor')
    || reason.includes('scope');
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'profile_memory_operation_failed';
}

function freezeMemory(memory: ProfileLayoutMemory): ProfileLayoutMemory {
  return deepFreeze(cloneJson(memory));
}

function cloneJson<T>(value: T): T {
  if (typeof globalThis.structuredClone === 'function') {
    try {
      return globalThis.structuredClone(value);
    } catch {
      // Svelte 5 $state proxies are not structured-cloneable.
      return JSON.parse(JSON.stringify(value)) as T;
    }
  }
  return JSON.parse(JSON.stringify(value)) as T;
}

function deepFreeze<T>(value: T, seen = new WeakSet<object>()): T {
  if (!value || typeof value !== 'object' || seen.has(value as object)) return value;
  seen.add(value as object);
  for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child, seen);
  return Object.freeze(value);
}
