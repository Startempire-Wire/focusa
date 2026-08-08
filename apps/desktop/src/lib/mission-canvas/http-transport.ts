import type { MissionCanvasOperationInput, MissionCanvasTransport } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import {
  sameWorkstreamAuthorityContext,
  sameWorkstreamKey,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import {
  authorityFromEvent,
  authorityFromProjection,
  sameWorkstreamAuthority,
  workstreamAuthorityStorageKey
} from './exact-scope';
import type {
  ProjectionLifecycleEvent,
  ResolvedWorkspaceProjection,
  WorkstreamAuthorityContext
} from './types';
import registry from '../../../../../docs/contracts/spec135/mission-canvas-v1/operation-registry.json';

interface OperationDescriptor {
  operation_id: string;
  method: string;
  path: string;
  request_schema_ref: string;
  response_schema_ref: string;
  availability: string;
  requires_idempotency_key: boolean;
  requires_if_match_revision: boolean;
  confirmation: string;
  receipt_required: boolean;
}

interface ProjectionCursor {
  kind: string;
  value: number;
}

interface ProjectionWatermark {
  projectionRevision: number;
  layoutRevision: number;
  cursor?: ProjectionCursor;
}

interface HostLifecycleWatermark {
  lifecycleRevision: number;
  cursor?: ProjectionCursor;
}

interface LayoutMemoryWatermark {
  memoryRevision: number;
  cursor?: ProjectionCursor;
}

export class MissionCanvasTransportError extends Error {
  constructor(
    message: string,
    readonly operationId: string,
    readonly status?: number,
    readonly response?: unknown
  ) {
    super(message);
    this.name = 'MissionCanvasTransportError';
  }
}

const operations = new Map(
  (registry.operations as OperationDescriptor[]).map((operation) => [operation.operation_id, operation])
);

export class MissionCanvasHttpTransport implements MissionCanvasTransport {
  readonly #baseUrl: string;
  readonly #fetch: typeof fetch;
  readonly #projectionWatermarks = new Map<string, ProjectionWatermark>();
  readonly #hostLifecycleWatermarks = new Map<string, HostLifecycleWatermark>();
  readonly #layoutMemoryWatermarks = new Map<string, LayoutMemoryWatermark>();

  constructor(
    baseUrl: string,
    fetchImplementation: typeof fetch = fetch,
    private readonly accessToken?: string,
    private readonly timeoutMs = 30_000,
    private readonly permissionScopes: readonly string[] = [],
    private readonly capabilityRefs: readonly string[] = [],
    private readonly actorId?: string,
    private readonly authorityRef?: string
  ) {
    this.#baseUrl = baseUrl.replace(/\/$/, '');
    this.#fetch = fetchImplementation;
  }

  async request<T>(operationId: string, input: MissionCanvasOperationInput): Promise<T> {
    const operation = operations.get(operationId);
    if (!operation || operation.availability !== 'available') {
      throw new MissionCanvasTransportError('operation_unavailable', operationId);
    }
    const authority = authorityFromInput(input);
    const authorityValidation = validateMissionCanvasContract('WorkstreamAuthorityContext', authority);
    if (!authorityValidation.valid) {
      throw new MissionCanvasTransportError(
        `invalid_workstream_identity:${authorityValidation.errors.join(',')}`,
        operationId
      );
    }
    if (operation.requires_idempotency_key && !hasIdempotencyKey(input)) {
      throw new MissionCanvasTransportError('idempotency_key_required', operationId);
    }
    const ifMatchRevision = readIfMatchRevision(input, operationId);
    if (operation.requires_if_match_revision && ifMatchRevision === undefined) {
      throw new MissionCanvasTransportError('if_match_revision_required', operationId);
    }
    if (operation.confirmation === 'explicit' && !hasConfirmation(input)) {
      throw new MissionCanvasTransportError('explicit_confirmation_required', operationId);
    }

    const method = operation.method.toUpperCase();
    const requestInput = transportRequestInput(operation, input);
    const requestValidation = validateOperationRequest(
      operationId,
      operation.request_schema_ref,
      requestInput
    );
    if (!requestValidation.valid) {
      throw new MissionCanvasTransportError(
        `invalid_request:${requestValidation.errors.join(',')}`,
        operationId
      );
    }
    if (operationId === 'focusa.mission_canvas.layout_memory.update') {
      const watermark = validateProfileLayoutMemoryResponse(
        operationId,
        0,
        requestInput,
        authority,
        requestInput
      );
      if (watermark.memoryRevision === 0) {
        throw new MissionCanvasTransportError(
          'invalid_request:memory_revision',
          operationId
        );
      }
    }
    const resolved = resolvePath(operation.path, requestInput, operationId);
    const url = new URL(`${this.#baseUrl}${resolved.path}`);
    const init: RequestInit = {
      method,
      headers: {
        Accept: 'application/json',
        ...(this.accessToken ? { Authorization: `Bearer ${this.accessToken}` } : {}),
        ...(this.permissionScopes.length > 0 ? { 'X-Focusa-Permissions': this.permissionScopes.join(',') } : {}),
        ...(this.capabilityRefs.length > 0 ? { 'X-Focusa-Capabilities': this.capabilityRefs.join(',') } : {}),
        ...(this.actorId ? { 'X-Focusa-Actor-Id': this.actorId } : {}),
        ...(this.authorityRef ? { 'X-Focusa-Authority-Ref': this.authorityRef } : {}),
        ...(ifMatchRevision !== undefined ? { 'If-Match': ifMatchRevision } : {}),
        ...(hasIdempotencyKey(input)
          ? { 'Idempotency-Key': (input as Record<string, string>).idempotency_key }
          : {})
      }
    };

    if (method === 'GET' || method === 'HEAD') {
      appendQuery(url, resolved.input);
    } else if (resolved.input !== undefined) {
      (init.headers as Record<string, string>)['Content-Type'] = 'application/json';
      init.body = JSON.stringify(resolved.input);
    }

    const abort = new AbortController();
    const timeout = setTimeout(() => abort.abort(), this.timeoutMs);
    init.signal = abort.signal;
    let response: Response;
    try {
      response = await this.#fetch(url, init);
    } catch (error) {
      throw new MissionCanvasTransportError(
        error instanceof Error ? error.message : 'transport_request_failed',
        operationId
      );
    } finally {
      clearTimeout(timeout);
    }

    const value = await readResponse(response);
    if (!response.ok) {
      throw new MissionCanvasTransportError('transport_response_failed', operationId, response.status, value);
    }

    const validation = validateResponse(operation.response_schema_ref, value);
    if (!validation.valid) {
      throw new MissionCanvasTransportError(
        `invalid_response:${validation.errors.join(',')}`,
        operationId,
        response.status,
        value
      );
    }
    if (operationId === 'focusa.mission_canvas.profile.get') {
      const expectedProfileId = (requestInput as Record<string, unknown>).profile_id;
      const returnedProfileId = (value as Record<string, unknown>).profile_id;
      if (returnedProfileId !== expectedProfileId) {
        throw new MissionCanvasTransportError(
          'invalid_response:profile_id_mismatch',
          operationId,
          response.status,
          value
        );
      }
    }
    if (operationId === 'focusa.mission_canvas.layout_memory.get') {
      const watermark = validateProfileLayoutMemoryResponse(
        operationId,
        response.status,
        value,
        authority,
        requestInput
      );
      const requestRecord = requestInput as Record<string, unknown>;
      const memoryKey = [
        workstreamAuthorityStorageKey(authority),
        requestRecord.profile_id,
        requestRecord.activity_mode_id,
        requestRecord.viewport_class
      ].join('|');
      const previous = this.#layoutMemoryWatermarks.get(memoryKey);
      if (previous && watermark.memoryRevision < previous.memoryRevision) {
        throw new MissionCanvasTransportError(
          'stale_profile_memory_revision',
          operationId,
          response.status,
          value
        );
      }
      this.#layoutMemoryWatermarks.set(memoryKey, watermark);
    }
    if (operationId === 'focusa.mission_canvas.layout_memory.update') {
      const receipt = validateLayoutMemoryUpdateReceipt(
        operationId,
        response.status,
        value,
        authority,
        requestInput
      );
      const requestRecord = requestInput as Record<string, unknown>;
      const memoryKey = [
        workstreamAuthorityStorageKey(authority),
        requestRecord.profile_id,
        requestRecord.activity_mode_id,
        requestRecord.viewport_class
      ].join('|');
      const previous = this.#layoutMemoryWatermarks.get(memoryKey);
      if (previous && receipt.memoryRevision < previous.memoryRevision) {
        throw new MissionCanvasTransportError(
          'stale_layout_memory_revision',
          operationId,
          response.status,
          value
        );
      }
      if (
        previous?.cursor !== undefined
        && receipt.cursor !== undefined
        && previous.cursor.kind === receipt.cursor.kind
        && receipt.cursor.value < previous.cursor.value
      ) {
        throw new MissionCanvasTransportError(
          'stale_layout_memory_cursor',
          operationId,
          response.status,
          value
        );
      }
      this.#layoutMemoryWatermarks.set(memoryKey, receipt);
    }
    if (operationId === 'focusa.mission_canvas.layout.mutate') {
      validateLayoutMutationResult(operationId, response.status, value, authority, requestInput);
    }
    if (operation.response_schema_ref === 'ResolvedWorkspaceProjection') {
      const watermark = validateProjectionResponse(
        operationId,
        response.status,
        value,
        authority
      );
      const key = workstreamAuthorityStorageKey(authority);
      const previous = this.#projectionWatermarks.get(key);
      if (previous && watermark.projectionRevision < previous.projectionRevision) {
        throw new MissionCanvasTransportError(
          'stale_projection_revision',
          operationId,
          response.status,
          value
        );
      }
      if (previous && watermark.layoutRevision < previous.layoutRevision) {
        throw new MissionCanvasTransportError(
          'stale_projection_layout_revision',
          operationId,
          response.status,
          value
        );
      }
      if (
        previous?.cursor !== undefined
        && watermark.cursor !== undefined
        && previous.cursor.kind === watermark.cursor.kind
        && watermark.cursor.value < previous.cursor.value
      ) {
        throw new MissionCanvasTransportError(
          'stale_projection_cursor',
          operationId,
          response.status,
          value
        );
      }
      this.#projectionWatermarks.set(key, watermark);
    }
    if (operation.response_schema_ref === 'DomainPackInstallReceipt'
      && !sameWorkstreamKey((value as { workstream?: unknown }).workstream, authority.workstream)) {
      throw new MissionCanvasTransportError(
        'foreign_receipt_scope',
        operationId,
        response.status,
        value
      );
    }
    if (operation.response_schema_ref === 'ProjectionLifecycleEvent[]') {
      for (const [index, event] of (value as ProjectionLifecycleEvent[]).entries()) {
        const eventAuthority = authorityFromEvent(event);
        const eventAuthorityValidation = validateMissionCanvasContract(
          'WorkstreamAuthorityContext',
          eventAuthority
        );
        if (!eventAuthorityValidation.valid) {
          throw new MissionCanvasTransportError(
            `invalid_response:${index}:${eventAuthorityValidation.errors.join(',')}`,
            operationId,
            response.status,
            value
          );
        }
        if (!sameWorkstreamAuthority(eventAuthority, authority)) {
          throw new MissionCanvasTransportError(
            'foreign_event_scope',
            operationId,
            response.status,
            value
          );
        }
      }
    }
    if (operation.response_schema_ref === 'HostRendererResolution') {
      const responseAuthority = authorityFromResolution(value);
      if (!responseAuthority) {
        throw new MissionCanvasTransportError(
          'invalid_response:missing:workstream',
          operationId,
          response.status,
          value
        );
      }
      if (!sameWorkstreamKey(responseAuthority.workstream, authority.workstream)) {
        throw new MissionCanvasTransportError(
          'foreign_resolution_scope',
          operationId,
          response.status,
          value
        );
      }
      const responseAuthorityValidation = validateMissionCanvasContract(
        'WorkstreamAuthorityContext',
        responseAuthority
      );
      if (!responseAuthorityValidation.valid) {
        throw new MissionCanvasTransportError(
          `invalid_response:${responseAuthorityValidation.errors.join(',')}`,
          operationId,
          response.status,
          value
        );
      }
      if (!sameWorkstreamAuthorityContext(responseAuthority, authority)) {
        throw new MissionCanvasTransportError(
          'foreign_resolution_scope',
          operationId,
          response.status,
          value
        );
      }
    }
    if (operation.response_schema_ref === 'HostLifecycleState') {
      const watermark = validateLifecycleResponse(
        operationId,
        response.status,
        value,
        authority
      );
      if (operationId === 'focusa.mission_canvas.rich_host.focus') {
        validateHostFocusResponse(operationId, response.status, value);
      }
      if (operationId === 'focusa.mission_canvas.rich_host.hide') {
        validateHostHideResponse(operationId, response.status, value);
      }
      if (operationId === 'focusa.mission_canvas.rich_host.close') {
        validateHostCloseResponse(operationId, response.status, value);
      }
      const key = workstreamAuthorityStorageKey(authority);
      const previous = this.#hostLifecycleWatermarks.get(key);
      if (previous && watermark.lifecycleRevision < previous.lifecycleRevision) {
        throw new MissionCanvasTransportError(
          'stale_lifecycle_revision',
          operationId,
          response.status,
          value
        );
      }
      if (
        previous?.cursor !== undefined
        && watermark.cursor !== undefined
        && previous.cursor.kind === watermark.cursor.kind
        && watermark.cursor.value < previous.cursor.value
      ) {
        throw new MissionCanvasTransportError(
          'stale_lifecycle_cursor',
          operationId,
          response.status,
          value
        );
      }
      this.#hostLifecycleWatermarks.set(key, watermark);
    }
    return value as T;
  }
}

function validateOperationRequest(
  operationId: string,
  schemaRef: string,
  value: unknown
): { valid: boolean; errors: string[] } {
  const validation = validateMissionCanvasContract(schemaRef, value);
  if (!validation.valid) return validation;
  if (operationId === 'focusa.mission_canvas.profile.get') {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return { valid: false, errors: ['expected object'] };
    }
    const requestObject = value as Record<string, unknown>;
    const profileId = requestObject.profile_id;
    if (typeof profileId !== 'string' || profileId.trim().length === 0) {
      return { valid: false, errors: ['missing:profile_id'] };
    }
    const allowedFields = new Set([
      'profile_id',
      'workstream',
      'continuity_id',
      'attachment',
      'workspace_binding_id',
      'runtime_object',
      'work_surface_id'
    ]);
    const unknownFields = Object.keys(requestObject)
      .filter((field) => !allowedFields.has(field))
      .map((field) => `unknown:${field}`);
    if (unknownFields.length > 0) return { valid: false, errors: unknownFields };
    return validation;
  }
  if (operationId === 'focusa.mission_canvas.layout_memory.get') {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return { valid: false, errors: ['expected object'] };
    }
    const requestObject = value as Record<string, unknown>;
    for (const field of ['profile_id', 'activity_mode_id', 'viewport_class']) {
      if (typeof requestObject[field] !== 'string' || requestObject[field].trim().length === 0) {
        return { valid: false, errors: [`missing:${field}`] };
      }
    }
    const allowedFields = new Set([
      'profile_id',
      'activity_mode_id',
      'viewport_class',
      'workstream',
      'continuity_id',
      'attachment',
      'workspace_binding_id',
      'runtime_object',
      'work_surface_id'
    ]);
    const unknownFields = Object.keys(requestObject)
      .filter((field) => !allowedFields.has(field))
      .map((field) => `unknown:${field}`);
    if (unknownFields.length > 0) return { valid: false, errors: unknownFields };
    if (!['minimum', 'compact', 'standard', 'productive', 'wide', 'reference_capture']
      .includes(requestObject.viewport_class as string)) {
      return { valid: false, errors: ['invalid:viewport_class'] };
    }
    return validation;
  }
  if (operationId === 'focusa.mission_canvas.layout_memory.update') {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return { valid: false, errors: ['expected object'] };
    }
    const requestObject = value as Record<string, unknown>;
    const memoryRevision = requestObject.memory_revision;
    if (typeof memoryRevision !== 'number'
      || !Number.isSafeInteger(memoryRevision)
      || memoryRevision < 1) {
      return { valid: false, errors: ['missing:memory_revision'] };
    }
    if (typeof requestObject.idempotency_key !== 'string'
      || requestObject.idempotency_key.trim().length === 0) {
      return { valid: false, errors: ['missing:idempotency_key'] };
    }
    return validation;
  }
  if (operationId === 'focusa.mission_canvas.layout.mutate') {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return { valid: false, errors: ['expected object'] };
    }
    const command = value as Record<string, unknown>;
    if (typeof command.command_id !== 'string' || command.command_id.trim().length === 0) {
      return { valid: false, errors: ['missing:command_id'] };
    }
    if (typeof command.idempotency_key !== 'string' || command.idempotency_key.trim().length === 0) {
      return { valid: false, errors: ['missing:idempotency_key'] };
    }
    if (!Number.isSafeInteger(command.expected_projection_revision)
      || (command.expected_projection_revision as number) < 0
      || !Number.isSafeInteger(command.expected_layout_revision)
      || (command.expected_layout_revision as number) < 0) {
      return { valid: false, errors: ['invalid:expected_revision'] };
    }
    return validation;
  }
  if (operationId !== 'focusa.mission_canvas.profile.select'
    && operationId !== 'focusa.mission_canvas.activity.select') return validation;
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return { valid: false, errors: ['expected object'] };
  }
  const requestObject = value as Record<string, unknown>;
  const selectionId = requestObject.selection_id;
  if (typeof selectionId !== 'string' || selectionId.trim().length === 0) {
    return { valid: false, errors: ['missing:selection_id'] };
  }
  const expectedRevision = requestObject.expected_projection_revision;
  if (typeof expectedRevision !== 'number'
    || !Number.isSafeInteger(expectedRevision)
    || expectedRevision < 0) {
    return { valid: false, errors: ['missing:expected_projection_revision'] };
  }
  if (requestObject.event_cursor !== undefined
    && (typeof requestObject.event_cursor !== 'string' || requestObject.event_cursor.trim().length === 0)) {
    return { valid: false, errors: ['invalid:event_cursor'] };
  }
  const allowedFields = new Set([
    'selection_id',
    'expected_projection_revision',
    'idempotency_key',
    'event_cursor',
    'workstream',
    'continuity_id',
    'attachment',
    'workspace_binding_id',
    'runtime_object',
    'work_surface_id'
  ]);
  const unknownFields = Object.keys(requestObject)
    .filter((field) => !allowedFields.has(field))
    .map((field) => `unknown:${field}`);
  if (unknownFields.length > 0) return { valid: false, errors: unknownFields };
  return validation;
}

function validateProfileLayoutMemoryResponse(
  operationId: string,
  status: number,
  value: unknown,
  expectedAuthority: WorkstreamAuthorityContext,
  requestInput: unknown
): LayoutMemoryWatermark {
  const memory = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  if (!memory) {
    throw new MissionCanvasTransportError(
      'invalid_response:expected_object',
      operationId,
      status,
      value
    );
  }

  const responseAuthority = authorityFromRecord(memory);
  const authorityValidation = validateMissionCanvasContract(
    'WorkstreamAuthorityContext',
    responseAuthority
  );
  if (!authorityValidation.valid) {
    throw new MissionCanvasTransportError(
      `invalid_response:${authorityValidation.errors.join(',')}`,
      operationId,
      status,
      value
    );
  }
  if (!sameWorkstreamAuthorityContext(responseAuthority, expectedAuthority)) {
    throw new MissionCanvasTransportError(
      'foreign_profile_memory_scope',
      operationId,
      status,
      value
    );
  }

  const request = requestInput as Record<string, unknown>;
  for (const field of ['profile_id', 'activity_mode_id', 'viewport_class']) {
    if (memory[field] !== request[field]) {
      const mismatchCode = field === 'profile_id'
        ? 'foreign_profile_memory_profile_id'
        : field === 'activity_mode_id'
          ? 'foreign_profile_memory_activity_mode_id'
          : 'foreign_profile_memory_viewport_class';
      throw new MissionCanvasTransportError(
        mismatchCode,
        operationId,
        status,
        value
      );
    }
  }
  const profileId = memory.profile_id;
  const activityModeId = memory.activity_mode_id;
  const viewportClass = memory.viewport_class;
  if (typeof profileId !== 'string' || typeof activityModeId !== 'string' || typeof viewportClass !== 'string') {
    throw new MissionCanvasTransportError(
      'invalid_response:profile_memory_identity',
      operationId,
      status,
      value
    );
  }
  const memoryId = memory.memory_id;
  const expectedMemoryId = `layout-memory:${profileId}:${activityModeId}:${viewportClass}`;
  if (typeof memoryId !== 'string' || !/^layout-memory:[a-z0-9._:-]+$/.test(memoryId)) {
    throw new MissionCanvasTransportError(
      'invalid_response:memory_id',
      operationId,
      status,
      value
    );
  }
  if (memoryId !== expectedMemoryId) {
    throw new MissionCanvasTransportError(
      'invalid_response:memory_id_mismatch',
      operationId,
      status,
      value
    );
  }

  const memoryRevision = memory.memory_revision;
  if (typeof memoryRevision !== 'number'
    || !Number.isSafeInteger(memoryRevision)
    || memoryRevision < 0) {
    throw new MissionCanvasTransportError(
      'invalid_response:memory_revision',
      operationId,
      status,
      value
    );
  }
  if (typeof memory.idempotency_key !== 'string' || memory.idempotency_key.trim().length === 0) {
    throw new MissionCanvasTransportError(
      'invalid_response:idempotency_key',
      operationId,
      status,
      value
    );
  }
  if (typeof memory.updated_at !== 'string' || Number.isNaN(Date.parse(memory.updated_at))) {
    throw new MissionCanvasTransportError(
      'invalid_response:updated_at',
      operationId,
      status,
      value
    );
  }

  const placements = memory.placements;
  if (!Array.isArray(placements)) {
    throw new MissionCanvasTransportError(
      'invalid_response:placements',
      operationId,
      status,
      value
    );
  }
  const placementIds = new Set<string>();
  const contributionIdPattern = /^contribution:[a-z0-9][a-z0-9._:-]{0,159}$/;
  const regionKinds = new Set([
    'primary',
    'secondary',
    'inspector',
    'rail',
    'queue',
    'composer',
    'navigation',
    'overlay'
  ]);
  for (const [index, placement] of placements.entries()) {
    const placementValidation = validateMissionCanvasContract(
      'ContributionPlacementPreference',
      placement
    );
    if (!placementValidation.valid) {
      throw new MissionCanvasTransportError(
        `invalid_response:placements:${index}:${placementValidation.errors.join(',')}`,
        operationId,
        status,
        value
      );
    }
    if (!placement || typeof placement !== 'object' || Array.isArray(placement)) {
      throw new MissionCanvasTransportError(
        `invalid_response:placements:${index}:expected_object`,
        operationId,
        status,
        value
      );
    }
    const placementRecord = placement as Record<string, unknown>;
    const contributionId = placementRecord.contribution_id;
    const regions = placementRecord.preferred_regions;
    const minimumSpan = placementRecord.minimum_span;
    const maximumSpan = placementRecord.maximum_span;
    const preferredOrder = placementRecord.preferred_order;
    const adjacency = placementRecord.preferred_adjacency;
    const lastCompatibleNode = placementRecord.last_compatible_layout_node_id;
    if (typeof contributionId !== 'string'
      || !contributionIdPattern.test(contributionId)
      || placementIds.has(contributionId)
      || !Array.isArray(regions)
      || regions.length === 0
      || new Set(regions).size !== regions.length
      || regions.some((region) => typeof region !== 'string' || !regionKinds.has(region))
      || (adjacency !== undefined
        && (!Array.isArray(adjacency)
          || new Set(adjacency).size !== adjacency.length
          || adjacency.some((id) => typeof id !== 'string' || !contributionIdPattern.test(id))))
      || (lastCompatibleNode !== undefined
        && lastCompatibleNode !== null
        && (typeof lastCompatibleNode !== 'string' || lastCompatibleNode.trim().length === 0))
      || typeof preferredOrder !== 'number'
      || !Number.isSafeInteger(preferredOrder)
      || preferredOrder < 0
      || typeof minimumSpan !== 'number'
      || !Number.isSafeInteger(minimumSpan)
      || minimumSpan < 1
      || minimumSpan > 12
      || typeof maximumSpan !== 'number'
      || !Number.isSafeInteger(maximumSpan)
      || maximumSpan < minimumSpan
      || maximumSpan > 12) {
      throw new MissionCanvasTransportError(
        `invalid_response:placements:${index}:content`,
        operationId,
        status,
        value
      );
    }
    placementIds.add(contributionId);
  }

  const absent = memory.absent_contribution_ids;
  if (!Array.isArray(absent)) {
    throw new MissionCanvasTransportError(
      'invalid_response:absent_contribution_ids',
      operationId,
      status,
      value
    );
  }
  const absentIds = new Set<string>();
  for (const contributionId of absent) {
    if (typeof contributionId !== 'string'
      || !contributionIdPattern.test(contributionId)
      || absentIds.has(contributionId)) {
      throw new MissionCanvasTransportError(
        'invalid_response:absent_contribution_ids:content',
        operationId,
        status,
        value
      );
    }
    absentIds.add(contributionId);
  }
  if ([...placementIds].some((contributionId) => absentIds.has(contributionId))) {
    throw new MissionCanvasTransportError(
      'invalid_response:memory_partition_overlap',
      operationId,
      status,
      value
    );
  }

  return { memoryRevision };
}

function validateLayoutMutationResult(
  operationId: string,
  status: number,
  value: unknown,
  expectedAuthority: WorkstreamAuthorityContext,
  requestInput: unknown
): void {
  const result = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  const request = requestInput && typeof requestInput === 'object' && !Array.isArray(requestInput)
    ? requestInput as Record<string, unknown>
    : undefined;
  if (!result || !request) {
    throw new MissionCanvasTransportError('invalid_response:expected_layout_mutation_result', operationId, status, value);
  }
  const responseAuthority = authorityFromRecord(result);
  const authorityValidation = validateMissionCanvasContract('WorkstreamAuthorityContext', responseAuthority);
  if (!authorityValidation.valid || !sameWorkstreamAuthorityContext(responseAuthority, expectedAuthority)) {
    throw new MissionCanvasTransportError('invalid_response:scope_mismatch', operationId, status, value);
  }
  if (result.command_id !== request.command_id) {
    throw new MissionCanvasTransportError('invalid_response:command_mismatch', operationId, status, value);
  }
  const projectionRevision = result.projection_revision;
  const layoutRevision = result.layout_revision;
  const expectedProjectionRevision = request.expected_projection_revision;
  const expectedLayoutRevision = request.expected_layout_revision;
  if (!Number.isSafeInteger(projectionRevision)
    || !Number.isSafeInteger(layoutRevision)
    || !Number.isSafeInteger(expectedProjectionRevision)
    || !Number.isSafeInteger(expectedLayoutRevision)
    || (projectionRevision as number) <= (expectedProjectionRevision as number)
    || (layoutRevision as number) <= (expectedLayoutRevision as number)) {
    throw new MissionCanvasTransportError('invalid_response:stale_layout_mutation', operationId, status, value);
  }
}

function validateLayoutMemoryUpdateReceipt(
  operationId: string,
  status: number,
  value: unknown,
  expectedAuthority: WorkstreamAuthorityContext,
  requestInput: unknown
): LayoutMemoryWatermark {
  const receipt = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  if (!receipt) {
    throw new MissionCanvasTransportError(
      'invalid_response:expected_receipt',
      operationId,
      status,
      value
    );
  }
  const responseAuthority = authorityFromRecord(receipt);
  const authorityValidation = validateMissionCanvasContract(
    'WorkstreamAuthorityContext',
    responseAuthority
  );
  if (!authorityValidation.valid || !sameWorkstreamAuthorityContext(responseAuthority, expectedAuthority)) {
    throw new MissionCanvasTransportError(
      'foreign_layout_memory_receipt_scope',
      operationId,
      status,
      value
    );
  }
  const request = requestInput as Record<string, unknown>;
  if (receipt.idempotency_key !== request.idempotency_key) {
    throw new MissionCanvasTransportError(
      'invalid_response:idempotency_key_mismatch',
      operationId,
      status,
      value
    );
  }
  if (receipt.accepted !== true) {
    throw new MissionCanvasTransportError(
      'invalid_response:receipt_not_accepted',
      operationId,
      status,
      value
    );
  }
  const projectionRevision = receipt.projection_revision;
  const layoutRevision = receipt.layout_revision;
  if (typeof projectionRevision !== 'number'
    || !Number.isSafeInteger(projectionRevision)
    || projectionRevision < 1
    || typeof layoutRevision !== 'number'
    || !Number.isSafeInteger(layoutRevision)
    || layoutRevision < 1) {
    throw new MissionCanvasTransportError(
      'invalid_response:layout_memory_revision',
      operationId,
      status,
      value
    );
  }
  const expectedRevision = request.memory_revision;
  if (typeof expectedRevision !== 'number'
    || !Number.isSafeInteger(expectedRevision)
    || expectedRevision < 0
    || layoutRevision !== expectedRevision + 1
    || projectionRevision !== layoutRevision) {
    throw new MissionCanvasTransportError(
      'invalid_response:layout_memory_revision_mismatch',
      operationId,
      status,
      value
    );
  }
  if (typeof receipt.receipt_id !== 'string' || receipt.receipt_id.trim().length === 0
    || typeof receipt.evidence_id !== 'string' || receipt.evidence_id.trim().length === 0
    || typeof receipt.projection_digest !== 'string' || !/^sha256:[a-f0-9]{64}$/.test(receipt.projection_digest)
    || typeof receipt.issued_at !== 'string' || Number.isNaN(Date.parse(receipt.issued_at))) {
    throw new MissionCanvasTransportError(
      'invalid_response:receipt_proof',
      operationId,
      status,
      value
    );
  }
  const eventCursor = receipt.event_cursor;
  if (typeof eventCursor !== 'string' || !eventCursor.trim()) {
    throw new MissionCanvasTransportError(
      'invalid_response:layout_memory_cursor',
      operationId,
      status,
      value
    );
  }
  const cursor = parseProjectionCursor(eventCursor);
  if (!cursor) {
    throw new MissionCanvasTransportError(
      'invalid_response:layout_memory_cursor',
      operationId,
      status,
      value
    );
  }
  return { memoryRevision: layoutRevision, cursor };
}

function validateResponse(schemaRef: string, value: unknown): { valid: boolean; errors: string[] } {
  if (!schemaRef.endsWith('[]')) return validateMissionCanvasContract(schemaRef, value);
  if (!Array.isArray(value)) return { valid: false, errors: ['expected array'] };
  const itemSchema = schemaRef.slice(0, -2);
  const errors = value.flatMap((item, index) =>
    validateMissionCanvasContract(itemSchema, item).errors.map((error) => `${index}:${error}`)
  );
  return { valid: errors.length === 0, errors };
}

function transportRequestInput(operation: OperationDescriptor, input: MissionCanvasOperationInput): MissionCanvasOperationInput {
  if (operation.request_schema_ref !== 'ContributionEligibilityContext') return input;
  const {
    idempotency_key: _idempotencyKey,
    if_match_revision: _ifMatchRevision,
    expected_revision: _expectedRevision,
    expected_projection_revision: _expectedProjectionRevision,
    previous_projection_revision: _previousProjectionRevision,
    previous_layout_revision: _previousLayoutRevision,
    event_cursor: _eventCursor,
    causation_id: _causationId,
    ...generatedContext
  } = input as Record<string, unknown>;
  return generatedContext as MissionCanvasOperationInput;
}

function resolvePath(path: string, input: unknown, operationId: string): { path: string; input: unknown } {
  const record = input && typeof input === 'object' && !Array.isArray(input)
    ? { ...(input as Record<string, unknown>) }
    : undefined;
  const resolvedPath = path.replace(/\{([^}]+)\}/g, (_match, name: string) => {
    const value = record?.[name];
    if (value === undefined || value === null || typeof value === 'object') {
      throw new MissionCanvasTransportError(`path_parameter_required:${name}`, operationId);
    }
    delete record?.[name];
    return encodeURIComponent(String(value));
  });
  return { path: resolvedPath, input: record ?? input };
}

function appendQuery(url: URL, input: unknown): void {
  if (!input || typeof input !== 'object' || Array.isArray(input)) return;
  for (const [key, value] of Object.entries(input)) {
    if (value === undefined || value === null) continue;
    url.searchParams.set(key, typeof value === 'object' ? JSON.stringify(value) : String(value));
  }
}

function authorityFromInput(input: unknown): WorkstreamAuthorityContext {
  if (!input || typeof input !== 'object' || Array.isArray(input)) {
    return { workstream: undefined as never };
  }
  return authorityFromRecord(input as Record<string, unknown>);
}

function authorityFromRecord(value: Record<string, unknown>): WorkstreamAuthorityContext {
  return {
    workstream: value.workstream as WorkstreamAuthorityContext['workstream'],
    continuity_id: (value.continuity_id as WorkstreamAuthorityContext['continuity_id']) ?? null,
    attachment: (value.attachment as WorkstreamAuthorityContext['attachment']) ?? null,
    workspace_binding_id: (value.workspace_binding_id as WorkstreamAuthorityContext['workspace_binding_id']) ?? null,
    runtime_object: (value.runtime_object as WorkstreamAuthorityContext['runtime_object']) ?? null,
    work_surface_id: (value.work_surface_id as WorkstreamAuthorityContext['work_surface_id']) ?? null
  };
}

function authorityFromResolution(value: unknown): WorkstreamAuthorityContext | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined;
  const response = value as Record<string, unknown>;
  if (!('workstream' in response)) return undefined;
  return authorityFromRecord(response);
}

function authorityFromLifecycleState(value: unknown): WorkstreamAuthorityContext | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined;
  const state = value as Record<string, unknown>;
  if (!('workstream' in state)) return undefined;
  return authorityFromRecord(state);
}

function validateLifecycleResponse(
  operationId: string,
  status: number,
  value: unknown,
  expectedAuthority: WorkstreamAuthorityContext
): HostLifecycleWatermark {
  const lifecycle = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  if (!lifecycle) {
    throw new MissionCanvasTransportError(
      'invalid_response:expected_object',
      operationId,
      status,
      value
    );
  }
  const responseAuthority = authorityFromLifecycleState(value);
  if (!responseAuthority) {
    throw new MissionCanvasTransportError(
      'invalid_response:missing:workstream',
      operationId,
      status,
      value
    );
  }
  if (!sameWorkstreamKey(responseAuthority.workstream, expectedAuthority.workstream)) {
    throw new MissionCanvasTransportError(
      'foreign_lifecycle_scope',
      operationId,
      status,
      value
    );
  }
  const responseAuthorityValidation = validateMissionCanvasContract(
    'WorkstreamAuthorityContext',
    responseAuthority
  );
  if (!responseAuthorityValidation.valid || !sameWorkstreamAuthorityContext(responseAuthority, expectedAuthority)) {
    throw new MissionCanvasTransportError(
      'foreign_lifecycle_scope',
      operationId,
      status,
      value
    );
  }

  const rendererAuthority = authorityFromResolution(lifecycle?.renderer_resolution);
  if (!rendererAuthority) {
    throw new MissionCanvasTransportError(
      'invalid_response:missing:renderer_resolution_workstream',
      operationId,
      status,
      value
    );
  }
  const rendererAuthorityValidation = validateMissionCanvasContract(
    'WorkstreamAuthorityContext',
    rendererAuthority
  );
  if (!rendererAuthorityValidation.valid || !sameWorkstreamAuthorityContext(rendererAuthority, responseAuthority)) {
    throw new MissionCanvasTransportError(
      'foreign_lifecycle_scope',
      operationId,
      status,
      value
    );
  }
  if (typeof lifecycle?.host_instance_id !== 'string' || !/^rich-host:[a-z0-9._:-]+$/.test(lifecycle.host_instance_id)) {
    throw new MissionCanvasTransportError(
      'invalid_response:host_instance_id',
      operationId,
      status,
      value
    );
  }
  const lifecycleRevision = lifecycle.lifecycle_revision;
  if (typeof lifecycleRevision !== 'number' || !Number.isSafeInteger(lifecycleRevision) || lifecycleRevision < 0) {
    throw new MissionCanvasTransportError(
      'invalid_response:lifecycle_revision',
      operationId,
      status,
      value
    );
  }
  if (typeof lifecycle.durable_event_cursor !== 'string' || lifecycle.durable_event_cursor.trim().length === 0) {
    throw new MissionCanvasTransportError(
      'invalid_response:durable_event_cursor',
      operationId,
      status,
      value
    );
  }
  const cursor = parseProjectionCursor(lifecycle.durable_event_cursor);
  if (!cursor) {
    throw new MissionCanvasTransportError(
      'invalid_response:lifecycle_cursor',
      operationId,
      status,
      value
    );
  }
  return {
    lifecycleRevision,
    cursor
  };
}

function validateHostFocusResponse(
  operationId: string,
  status: number,
  value: unknown
): void {
  const lifecycle = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  if (!lifecycle || lifecycle.state !== 'focused' || lifecycle.focused !== true) {
    throw new MissionCanvasTransportError(
      'invalid_response:focus_state',
      operationId,
      status,
      value
    );
  }
  const renderer = lifecycle.renderer_resolution;
  if (!renderer || typeof renderer !== 'object' || Array.isArray(renderer)
    || (renderer as Record<string, unknown>).selected_renderer !== 'focusa_desktop_tauri') {
    throw new MissionCanvasTransportError(
      'invalid_response:focus_renderer',
      operationId,
      status,
      value
    );
  }
}

function validateHostHideResponse(
  operationId: string,
  status: number,
  value: unknown
): void {
  const lifecycle = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  if (!lifecycle || lifecycle.state !== 'hidden' || lifecycle.focused !== false) {
    throw new MissionCanvasTransportError(
      'invalid_response:hide_state',
      operationId,
      status,
      value
    );
  }
  const renderer = lifecycle.renderer_resolution;
  if (!renderer || typeof renderer !== 'object' || Array.isArray(renderer)
    || (renderer as Record<string, unknown>).selected_renderer !== 'focusa_desktop_tauri') {
    throw new MissionCanvasTransportError(
      'invalid_response:hide_renderer',
      operationId,
      status,
      value
    );
  }
}

function validateHostCloseResponse(
  operationId: string,
  status: number,
  value: unknown
): void {
  const lifecycle = value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
  if (!lifecycle || lifecycle.state !== 'closing' || lifecycle.focused !== false) {
    throw new MissionCanvasTransportError(
      'invalid_response:close_state',
      operationId,
      status,
      value
    );
  }
  const renderer = lifecycle.renderer_resolution;
  if (!renderer || typeof renderer !== 'object' || Array.isArray(renderer)
    || (renderer as Record<string, unknown>).selected_renderer !== 'focusa_desktop_tauri') {
    throw new MissionCanvasTransportError(
      'invalid_response:close_renderer',
      operationId,
      status,
      value
    );
  }
}

function validateProjectionResponse(
  operationId: string,
  status: number,
  value: unknown,
  expectedAuthority: WorkstreamAuthorityContext
): ProjectionWatermark {
  const projection = value as ResolvedWorkspaceProjection;
  const responseAuthority = authorityFromProjection(projection);
  const authorityValidation = validateMissionCanvasContract(
    'WorkstreamAuthorityContext',
    responseAuthority
  );
  if (!authorityValidation.valid) {
    throw new MissionCanvasTransportError(
      `invalid_response:${authorityValidation.errors.join(',')}`,
      operationId,
      status,
      value
    );
  }
  if (!sameWorkstreamAuthority(responseAuthority, expectedAuthority)) {
    throw new MissionCanvasTransportError(
      'foreign_projection_scope',
      operationId,
      status,
      value
    );
  }
  if (
    projection.focused_work_surface_id !== null
    && (!projection.attachment
      || projection.work_surface_id !== projection.focused_work_surface_id)
  ) {
    throw new MissionCanvasTransportError(
      'invalid_response:focused_work_surface_authority',
      operationId,
      status,
      value
    );
  }

  if (!Array.isArray(projection.eligible_contributions)) {
    throw new MissionCanvasTransportError(
      'invalid_response:eligible_contributions',
      operationId,
      status,
      value
    );
  }
  for (const [index, contribution] of projection.eligible_contributions.entries()) {
    if (!contribution || typeof contribution !== 'object' || Array.isArray(contribution)) {
      throw new MissionCanvasTransportError(
        `invalid_response:${index}:missing_contribution`,
        operationId,
        status,
        value
      );
    }
    const contributionValue = (contribution as unknown as Record<string, unknown>).authority;
    if (!contributionValue || typeof contributionValue !== 'object' || Array.isArray(contributionValue)) {
      throw new MissionCanvasTransportError(
        `invalid_response:${index}:missing_contribution_authority`,
        operationId,
        status,
        value
      );
    }
    const contributionAuthority = authorityFromRecord(contributionValue as Record<string, unknown>);
    const contributionValidation = validateMissionCanvasContract(
      'WorkstreamAuthorityContext',
      contributionAuthority
    );
    if (!contributionValidation.valid) {
      throw new MissionCanvasTransportError(
        `invalid_response:${index}:${contributionValidation.errors.join(',')}`,
        operationId,
        status,
        value
      );
    }
    if (!sameWorkstreamAuthority(contributionAuthority, responseAuthority)) {
      throw new MissionCanvasTransportError(
        'foreign_contribution_scope',
        operationId,
        status,
        value
      );
    }
  }

  if (
    !Number.isSafeInteger(projection.projection_revision)
    || projection.projection_revision < 0
    || !Number.isSafeInteger(projection.layout_revision)
    || projection.layout_revision < 0
    || typeof projection.durable_event_cursor !== 'string'
    || projection.durable_event_cursor.trim().length === 0
  ) {
    throw new MissionCanvasTransportError(
      'invalid_response:projection_watermark',
      operationId,
      status,
      value
    );
  }
  const cursor = parseProjectionCursor(projection.durable_event_cursor);
  if (!cursor) {
    throw new MissionCanvasTransportError(
      'invalid_response:projection_cursor',
      operationId,
      status,
      value
    );
  }
  return {
    projectionRevision: projection.projection_revision,
    layoutRevision: projection.layout_revision,
    cursor
  };
}

function parseProjectionCursor(cursor: string): ProjectionCursor | undefined {
  const normalized = cursor.trim();
  const prefixed = /^(event|cursor|mission-canvas):([0-9]+)$/.exec(normalized);
  const match = prefixed ?? /^([0-9]+)$/.exec(normalized);
  if (!match) return undefined;
  const kind = prefixed ? prefixed[1] : 'opaque-numeric';
  const value = Number(prefixed ? prefixed[2] : match[1]);
  return Number.isSafeInteger(value) ? { kind, value } : undefined;
}

function hasIdempotencyKey(input: unknown): boolean {
  return !!input
    && typeof input === 'object'
    && !Array.isArray(input)
    && typeof (input as Record<string, unknown>).idempotency_key === 'string'
    && (input as Record<string, string>).idempotency_key.trim().length > 0;
}

function readIfMatchRevision(input: unknown, operationId?: string): string | undefined {
  if (!input || typeof input !== 'object' || Array.isArray(input)) return undefined;
  const record = input as Record<string, unknown>;
  if (operationId === 'focusa.mission_canvas.layout_memory.update') {
    const memoryRevision = record.memory_revision;
    if (typeof memoryRevision === 'number'
      && Number.isSafeInteger(memoryRevision)
      && memoryRevision >= 0) {
      // ProfileLayoutMemory carries the current representation revision; the
      // generated mutation uses it as the optimistic If-Match watermark and
      // Core advances the persisted preference atomically.
      return String(memoryRevision);
    }
  }
  for (const field of [
    'if_match_revision',
    'expected_revision',
    'expected_projection_revision',
    'previous_projection_revision'
  ]) {
    const value = record[field];
    if (typeof value === 'number' && Number.isSafeInteger(value) && value >= 0) return String(value);
    if (typeof value === 'string' && /^(?:0|[1-9][0-9]*)$/.test(value.trim())) {
      const normalized = value.trim();
      const numeric = Number(normalized);
      if (Number.isSafeInteger(numeric)) return normalized;
    }
  }
  return undefined;
}

function hasConfirmation(input: unknown): boolean {
  if (!input || typeof input !== 'object' || Array.isArray(input)) return false;
  const value = (input as Record<string, unknown>).confirmation;
  return value === 'confirm';
}

async function readResponse(response: Response): Promise<unknown> {
  if (response.status === 204) return {};
  const text = await response.text();
  if (!text) return {};
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}
