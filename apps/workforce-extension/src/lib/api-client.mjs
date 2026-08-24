import { normalizeDaemonOrigin } from './validation.mjs';

const MAX_RESPONSE_BYTES = 2 * 1024 * 1024;

export class ProjectionRequestError extends Error {
  constructor(kind, message, status = null) {
    super(message);
    this.name = 'ProjectionRequestError';
    this.kind = kind;
    this.status = status;
  }
}

const endpointContracts = Object.freeze({
  health: Object.freeze({ path: '/v1/health', schema: 'focusa.health.v1' }),
  work_loop: Object.freeze({ path: '/v1/work-loop/status?summary_only=true', schema: 'focusa.work_loop_status.v3' }),
  roster: Object.freeze({ path: '/v1/silent-sessions', schema: 'focusa.silent_session_api_envelope.v1' }),
});

function classifyStatus(status) {
  if (status === 401) return 'unauthenticated';
  if (status === 403) return 'forbidden';
  if (status === 404 || status === 405 || status === 501) return 'unsupported';
  return status >= 500 ? 'degraded' : 'request_rejected';
}

export async function fetchProjection(kind, { baseUrl, token, fetchImpl = globalThis.fetch, signal } = {}) {
  const contract = endpointContracts[kind];
  if (!contract) throw new TypeError(`unsupported projection kind: ${kind}`);
  const origin = normalizeDaemonOrigin(baseUrl);
  if (typeof token !== 'string' || !token) throw new TypeError('paired-device token is required');
  const response = await fetchImpl(new URL(contract.path, origin), {
    method: 'GET', signal,
    headers: { accept: 'application/json', authorization: `Bearer ${token}`, 'x-focusa-permissions': 'read:*' },
  });
  if (!response.ok) {
    throw new ProjectionRequestError(classifyStatus(response.status), `${kind} projection unavailable`, response.status);
  }
  const text = await response.text();
  if (new TextEncoder().encode(text).byteLength > MAX_RESPONSE_BYTES) {
    throw new ProjectionRequestError('invalid_envelope', `${kind} projection exceeds response bound`, response.status);
  }
  let body;
  try { body = JSON.parse(text); } catch { throw new ProjectionRequestError('invalid_envelope', `${kind} projection is not JSON`, response.status); }
  if (!body || body.schema !== contract.schema) {
    throw new ProjectionRequestError('unsupported', `${kind} projection schema is unsupported`, response.status);
  }
  if (body.degraded === true) throw new ProjectionRequestError('degraded', `${kind} projection is degraded`, response.status);
  return Object.freeze(body);
}

export const fetchHealth = (options) => fetchProjection('health', options);
export const fetchWorkLoop = (options) => fetchProjection('work_loop', options);
export const fetchRoster = (options) => fetchProjection('roster', options);
