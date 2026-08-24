import { saveConnection, forgetConnection } from './storage.mjs';
import { normalizeDaemonOrigin, requestDaemonOriginPermission } from './validation.mjs';

function requireFetch(fetchImpl) {
  if (typeof fetchImpl !== 'function') throw new Error('fetch implementation is required');
  return fetchImpl;
}
async function jsonResponse(response, operation) {
  let payload;
  try { payload = await response.json(); } catch { throw new Error(`${operation} returned non-JSON status ${response.status}`); }
  if (!response.ok) {
    const error = new Error(`${operation} failed: ${payload.failure_class ?? response.status}`);
    error.status = response.status; error.failure_class = payload.failure_class ?? null; error.payload = payload; throw error;
  }
  return payload;
}

export async function startPairing(input, deps = {}) {
  const baseUrl = normalizeDaemonOrigin(input.base_url);
  const chromeApi = deps.chromeApi ?? globalThis.chrome;
  const fetchImpl = requireFetch(deps.fetchImpl ?? globalThis.fetch);
  const granted = await requestDaemonOriginPermission(baseUrl, chromeApi);
  if (!granted) return Object.freeze({ state: 'permission_denied', base_url: baseUrl });
  const response = await fetchImpl(`${baseUrl}/v1/device/pair/start`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      device_name: input.device_name || 'Focusa Workforce Chrome',
      platform: 'focusa-workforce-chrome', daemon_base_url: baseUrl, scopes: ['read', 'write'],
    }),
  });
  const payload = await jsonResponse(response, 'pair_start');
  if (!payload.code || !payload.device_id || !payload.expires_at) throw new Error('pair_start response is incomplete');
  return Object.freeze({
    state: 'awaiting_approval', base_url: baseUrl, label: input.label || 'Focusa daemon',
    code: payload.code, device_id: payload.device_id, scopes: Object.freeze([...payload.scopes]),
    expires_at: payload.expires_at, operator_command: payload.operator_handoff?.command ?? null,
  });
}

export async function pollPairing(pairing, deps = {}) {
  if (pairing?.state !== 'awaiting_approval') throw new TypeError('awaiting_approval pairing state is required');
  const fetchImpl = requireFetch(deps.fetchImpl ?? globalThis.fetch);
  const response = await fetchImpl(`${pairing.base_url}/v1/device/pair/status?code=${encodeURIComponent(pairing.code)}`);
  const payload = await jsonResponse(response, 'pair_status');
  if (payload.status === 'pending') return pairing;
  if (payload.status === 'expired') return Object.freeze({ ...pairing, state: 'expired' });
  if (payload.status === 'revoked') return Object.freeze({ ...pairing, state: 'revoked' });
  if (payload.status === 'consumed' || (payload.token_present && !payload.token)) {
    return Object.freeze({ ...pairing, state: 'token_consumed_repair_required' });
  }
  if (payload.status !== 'completed' || typeof payload.token !== 'string' || !payload.token) {
    throw new Error(`pair_status returned unsupported state ${payload.status}`);
  }
  const now = (deps.now ?? (() => new Date()))().toISOString();
  const record = await saveConnection({
    schema: 'focusa.workforce_connection.v1', connection_id: pairing.device_id, label: pairing.label,
    base_url: pairing.base_url, device_id: pairing.device_id, token: payload.token,
    granted_scopes: payload.scopes, last_cursor: null, created_at: now, last_connected_at: now,
  }, deps.chromeApi ?? globalThis.chrome);
  return Object.freeze({ state: 'paired', connection: record });
}

export async function forgetPairedConnection(connectionId, chromeApi = globalThis.chrome) {
  return forgetConnection(connectionId, chromeApi);
}
