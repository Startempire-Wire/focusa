import { normalizeDaemonOrigin } from './validation.mjs';

const ENVELOPE_SCHEMA = 'focusa.silent_session_api_envelope.v1';
const APPROVAL_SCHEMA = 'focusa.silent_session_approval_response.v1';
const APPROVED_ACTIONS = new Set(['start', 'steer', 'cancel']);
const ACTIONS = new Set(['start', 'pause', 'resume', 'steer', 'cancel']);

export class OrchestrationError extends Error {
  constructor(kind, message, status = null, details = null) { super(message); this.name = 'OrchestrationError'; this.kind = kind; this.status = status; this.details = details; }
}
export class StaleTargetError extends OrchestrationError {
  constructor(details = null) { super('stale_target', 'exact target is stale; owner reconfirmation is required', 409, details); this.name = 'StaleTargetError'; }
}

function exactTarget(target) {
  if (!target || typeof target.session_id !== 'string' || !target.session_id || typeof target.run_id !== 'string' || !target.run_id || !Number.isSafeInteger(target.generation) || target.generation < 1) {
    throw new TypeError('exact session_id, run_id and positive generation are required');
  }
  return Object.freeze({ session_id: target.session_id, run_id: target.run_id, generation: target.generation });
}
function headers(token, permission, json = false) {
  if (typeof token !== 'string' || !token) throw new TypeError('paired-device token is required');
  return { accept: 'application/json', authorization: `Bearer ${token}`, 'x-focusa-permissions': permission, ...(json ? { 'content-type': 'application/json' } : {}) };
}
async function request(path, { baseUrl, token, fetchImpl, method = 'GET', body, signal, permission = 'write:*' }) {
  const response = await fetchImpl(new URL(path, normalizeDaemonOrigin(baseUrl)), { method, signal, headers: headers(token, permission, body !== undefined), ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
  let envelope;
  try { envelope = await response.json(); } catch { throw new OrchestrationError('invalid_envelope', 'daemon response is not JSON', response.status); }
  if (envelope?.schema !== ENVELOPE_SCHEMA) throw new OrchestrationError('unsupported', 'daemon envelope schema is unsupported', response.status);
  if (envelope.stale === true || ['stale_target','stale_generation','exact_target_mismatch'].includes(envelope.failure_class)) throw new StaleTargetError(envelope);
  if (!response.ok || envelope.ok !== true) {
    const kind = response.status === 401 ? 'unauthenticated' : response.status === 403 ? (envelope.failure_class?.startsWith('approval') ? 'approval_required' : 'forbidden') : response.status === 409 ? 'conflict' : 'rejected';
    throw new OrchestrationError(kind, envelope.status ?? 'orchestration request rejected', response.status, envelope);
  }
  return envelope;
}

export async function refreshExactTarget(target, options) {
  const exact = exactTarget(target);
  const query = new URLSearchParams({ run_id: exact.run_id, generation: String(exact.generation) });
  const envelope = await request(`/v1/silent-sessions/${encodeURIComponent(exact.session_id)}/status?${query}`, { ...options, permission: 'read:*' });
  const session = envelope.data?.session, run = envelope.data?.run;
  if (session?.id !== exact.session_id || run?.id !== exact.run_id || run?.generation !== exact.generation) {
    throw new StaleTargetError(envelope);
  }
  return Object.freeze({ target: exact, session: Object.freeze({ ...session }), run: Object.freeze({ ...run }), temporal_context: envelope.data.temporal_context ?? null });
}

function actionPayload(action, payload) {
  if (action === 'steer') {
    if (typeof payload?.instruction !== 'string' || !payload.instruction.trim()) throw new TypeError('steer instruction is required');
    return Object.freeze({ instruction: payload.instruction.trim() });
  }
  if (payload != null && Object.keys(payload).length) throw new TypeError(`${action} does not accept a payload`);
  return null;
}
function stable(value) {
  if (Array.isArray(value)) return `[${value.map(stable).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stable(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}
async function persistIntent(store, record) {
  if (!store?.load || !store?.persist) throw new TypeError('durable idempotency store is required');
  const existing = await store.load(record.idempotency_key);
  if (existing && stable(existing) !== stable(record)) throw new OrchestrationError('idempotency_conflict', 'idempotency key conflicts with a changed action', 409);
  if (!existing) await store.persist(Object.freeze(record));
  const committed = await store.load(record.idempotency_key);
  if (!committed || stable(committed) !== stable(record)) throw new OrchestrationError('storage_failure', 'action idempotency intent was not durably committed');
}

async function issueApproval(action, target, payload, key, options) {
  const envelope = await request(`/v1/silent-sessions/${encodeURIComponent(target.session_id)}/approvals`, { ...options, method: 'POST', body: {
    schema: 'focusa.silent_session_approval_request.v1', action, run_id: target.run_id, generation: target.generation,
    idempotency_key: key, risk_acknowledged: true, payload,
  } });
  const approval = envelope.data?.approval;
  if (approval?.schema !== APPROVAL_SCHEMA || approval.session_id !== target.session_id || approval.run_id !== target.run_id || approval.generation !== target.generation || approval.action !== action || !approval.approval_id || !approval.action_idempotency_key) {
    throw new OrchestrationError('invalid_approval', 'approval response lacks the exact action target');
  }
  if (Date.parse(approval.expires_at) <= Date.now()) throw new OrchestrationError('approval_expired', 'approval expired before action submission', 403);
  return Object.freeze({ ...approval });
}

export async function orchestrateAction({ action, target, payload = null, idempotency_key, idempotencyStore, requestOptions }) {
  if (!ACTIONS.has(action)) throw new TypeError(`unsupported orchestration action: ${action}`);
  if (typeof idempotency_key !== 'string' || !idempotency_key || idempotency_key.length > 200) throw new TypeError('bounded idempotency_key is required');
  const exact = exactTarget(target), exactPayload = actionPayload(action, payload);
  const intent = { idempotency_key, action, target: exact, payload: exactPayload };
  await persistIntent(idempotencyStore, intent);
  await refreshExactTarget(exact, requestOptions);

  let approval = null, actionKey = idempotency_key;
  if (APPROVED_ACTIONS.has(action)) {
    const approvalKey = `${idempotency_key}:approval`;
    await persistIntent(idempotencyStore, { idempotency_key: approvalKey, action: `approve:${action}`, target: exact, payload: exactPayload });
    approval = await issueApproval(action, exact, exactPayload, approvalKey, requestOptions);
    actionKey = approval.action_idempotency_key;
    await persistIntent(idempotencyStore, { idempotency_key: actionKey, action, target: exact, payload: exactPayload, approval_id: approval.approval_id });
  }
  const route = action === 'steer' ? 'steer' : action;
  const body = { run_id: exact.run_id, generation: exact.generation, idempotency_key: actionKey,
    ...(approval ? { approval_id: approval.approval_id } : {}), ...(exactPayload ?? {}) };
  let mutation;
  try {
    mutation = await request(`/v1/silent-sessions/${encodeURIComponent(exact.session_id)}/${route}`, { ...requestOptions, method: 'POST', body });
  } catch (error) {
    if (error instanceof StaleTargetError) {
      let refreshed = null;
      try { refreshed = await refreshExactTarget(exact, requestOptions); } catch (refreshError) { refreshed = refreshError.details ?? null; }
      throw new StaleTargetError(refreshed);
    }
    throw error;
  }
  const canonical = await refreshExactTarget(exact, requestOptions);
  return Object.freeze({ action, target: exact, approval, mutation_status: mutation.status, canonical });
}
