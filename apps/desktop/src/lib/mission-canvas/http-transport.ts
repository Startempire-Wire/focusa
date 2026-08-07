import type { MissionCanvasOperationInput, MissionCanvasTransport } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import {
  sameWorkstreamAuthorityContext,
  sameWorkstreamKey,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import { authorityFromEvent, sameWorkstreamAuthority } from './exact-scope';
import type { ProjectionLifecycleEvent, WorkstreamAuthorityContext } from './types';
import registry from '../../../../../docs/contracts/spec135/mission-canvas-v1/operation-registry.json';

interface OperationDescriptor {
  operation_id: string;
  method: string;
  path: string;
  response_schema_ref: string;
  availability: string;
  requires_idempotency_key: boolean;
  confirmation: string;
  receipt_required: boolean;
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
    if (operation.confirmation === 'explicit' && !hasConfirmation(input)) {
      throw new MissionCanvasTransportError('explicit_confirmation_required', operationId);
    }

    const method = operation.method.toUpperCase();
    const resolved = resolvePath(operation.path, input, operationId);
    const url = new URL(`${this.#baseUrl}${resolved.path}`);
    const init: RequestInit = {
      method,
      headers: {
        Accept: 'application/json',
        ...(this.accessToken ? { Authorization: `Bearer ${this.accessToken}` } : {}),
        ...(this.permissionScopes.length > 0 ? { 'X-Focusa-Permissions': this.permissionScopes.join(',') } : {}),
        ...(this.capabilityRefs.length > 0 ? { 'X-Focusa-Capabilities': this.capabilityRefs.join(',') } : {}),
        ...(this.actorId ? { 'X-Focusa-Actor-Id': this.actorId } : {}),
        ...(this.authorityRef ? { 'X-Focusa-Authority-Ref': this.authorityRef } : {})
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
    return value as T;
  }
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
  const value = input as Record<string, unknown>;
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
  return {
    workstream: response.workstream as WorkstreamAuthorityContext['workstream'],
    continuity_id: (response.continuity_id as WorkstreamAuthorityContext['continuity_id']) ?? null,
    attachment: (response.attachment as WorkstreamAuthorityContext['attachment']) ?? null,
    workspace_binding_id: (response.workspace_binding_id as WorkstreamAuthorityContext['workspace_binding_id']) ?? null,
    runtime_object: (response.runtime_object as WorkstreamAuthorityContext['runtime_object']) ?? null,
    work_surface_id: (response.work_surface_id as WorkstreamAuthorityContext['work_surface_id']) ?? null
  };
}

function hasIdempotencyKey(input: unknown): boolean {
  return !!input
    && typeof input === 'object'
    && !Array.isArray(input)
    && typeof (input as Record<string, unknown>).idempotency_key === 'string'
    && (input as Record<string, string>).idempotency_key.trim().length > 0;
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
