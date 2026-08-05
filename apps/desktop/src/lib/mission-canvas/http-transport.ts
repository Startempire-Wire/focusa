import type { MissionCanvasTransport } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import registry from '../../../../../docs/contracts/spec135/mission-canvas-v1/operation-registry.json';

interface OperationDescriptor {
  operation_id: string;
  method: string;
  path: string;
  response_schema_ref: string;
  availability: string;
  requires_idempotency_key: boolean;
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
    private readonly accessToken?: string
  ) {
    this.#baseUrl = baseUrl.replace(/\/$/, '');
    this.#fetch = fetchImplementation;
  }

  async request<T>(operationId: string, input?: unknown): Promise<T> {
    const operation = operations.get(operationId);
    if (!operation || operation.availability !== 'available') {
      throw new MissionCanvasTransportError('operation_unavailable', operationId);
    }
    if (operation.requires_idempotency_key && !hasIdempotencyKey(input)) {
      throw new MissionCanvasTransportError('idempotency_key_required', operationId);
    }

    const method = operation.method.toUpperCase();
    const resolved = resolvePath(operation.path, input, operationId);
    const url = new URL(`${this.#baseUrl}${resolved.path}`);
    const init: RequestInit = {
      method,
      headers: {
        Accept: 'application/json',
        ...(this.accessToken ? { Authorization: `Bearer ${this.accessToken}` } : {})
      }
    };

    if (method === 'GET' || method === 'HEAD') {
      appendQuery(url, resolved.input);
    } else if (resolved.input !== undefined) {
      (init.headers as Record<string, string>)['Content-Type'] = 'application/json';
      init.body = JSON.stringify(resolved.input);
    }

    let response: Response;
    try {
      response = await this.#fetch(url, init);
    } catch (error) {
      throw new MissionCanvasTransportError(
        error instanceof Error ? error.message : 'transport_request_failed',
        operationId
      );
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
    if (key === 'scope' && typeof value === 'object' && !Array.isArray(value)) {
      for (const [scopeKey, scopeValue] of Object.entries(value)) {
        if (scopeValue !== undefined && scopeValue !== null) url.searchParams.set(scopeKey, String(scopeValue));
      }
      continue;
    }
    url.searchParams.set(key, typeof value === 'object' ? JSON.stringify(value) : String(value));
  }
}

function hasIdempotencyKey(input: unknown): boolean {
  return !!input
    && typeof input === 'object'
    && !Array.isArray(input)
    && typeof (input as Record<string, unknown>).idempotency_key === 'string'
    && (input as Record<string, string>).idempotency_key.length > 0;
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
