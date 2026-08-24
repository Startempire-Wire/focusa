import { renderOrientationMission } from './orientation.mjs';
import { normalizeDaemonOrigin } from './validation.mjs';

const ENVELOPE_SCHEMA = 'focusa.silent_session_api_envelope.v1';
const encoder = new TextEncoder();

export class SessionCreateError extends Error {
  constructor(kind, message, status = null) { super(message); this.name = 'SessionCreateError'; this.kind = kind; this.status = status; }
}
export class SessionCreateConflictError extends SessionCreateError {
  constructor(message = 'idempotency key conflicts with a changed session request') { super('idempotency_conflict', message, 409); this.name = 'SessionCreateConflictError'; }
}

function required(value, field, max = 4096) {
  if (typeof value !== 'string' || !value.trim() || encoder.encode(value.trim()).byteLength > max) throw new TypeError(`${field} is required and bounded`);
  return value.trim();
}

export function buildSafeSessionConfig({ packet, display_name, provider, model, auth_profile_ref }) {
  const mission = renderOrientationMission(packet);
  const role = required(packet.role_profile_ref, 'role_profile_ref', 512);
  return Object.freeze({
    schema: 'focusa.silent_session_config.v1',
    identity: Object.freeze({ display_name: required(display_name, 'display_name', 200), project_root: required(packet.project_root, 'project_root'),
      continuity_id: required(packet.continuity_id, 'continuity_id', 512), work_item_ref: packet.work_item_ref,
      mission, agent_identity_ref: required(packet.agent_identity_ref, 'agent_identity_ref', 512), role_profile_ref: role }),
    harness: Object.freeze({ kind: 'pi', adapter_version: '1', native_resume_policy: 'prefer' }),
    model: Object.freeze({ provider: required(provider, 'provider', 128), model: required(model, 'model', 256), thinking: null,
      selection_policy: 'exact', fallback_policy: 'disabled', allowed_fallbacks: Object.freeze([]),
      auth_profile_ref: required(auth_profile_ref, 'auth_profile_ref', 512), require_entitlement_preflight: true,
      require_runtime_model_confirmation: true }),
    workspace: Object.freeze({ strategy: 'read_only_shared', source_root: packet.project_root, worktree_root: null, base_ref: null,
      branch_name: null, integration_policy: 'manual' }),
    bootstrap: Object.freeze({ target_profile: 'rules_and_context', packet_mode: 'session_start', verification_required: true }),
    supervision: Object.freeze({ restart_policy: 'on_failure', max_process_restarts: 3, max_transport_retries: 5,
      retry_backoff_seconds: 2, soft_pause_timeout_seconds: 30, graceful_stop_timeout_seconds: 30,
      checkpoint_interval_seconds: 300, checkpoint_event_interval: 250, waiting_input_timeout_seconds: 900,
      silent_output_warning_seconds: 300 }),
    resources: Object.freeze({ priority: 0, max_wall_clock_seconds: 1800, max_cpu_percent: null, max_memory_bytes: null,
      max_pids: null, max_disk_bytes: null, max_output_bytes: 16 * 1024 * 1024, max_tokens: null, max_cost_usd: null, max_turns: 12 }),
    output: Object.freeze({ persist_stdout: true, persist_stderr: true, persist_semantic_events: true, chunk_max_bytes: 1048576,
      chunk_max_seconds: 60, redaction_profile_ref: 'default', operator_projection_budget: 4096,
      raw_retention_policy_ref: 'raw-default' }),
    governance: Object.freeze({ context_authority_required: true, risky_mutation_preflight_required: true,
      destructive_actions_allowed: false, writer_lease_required: true, completion_receipt_required: true,
      evidence_policy_ref: 'required', policy_locks: Object.freeze([]) }),
    notifications: Object.freeze({ waiting_input: true, blocked: true, failed: true, completed: true, model_mismatch: true,
      budget_pressure: true, channels: Object.freeze([]) }),
    retention: Object.freeze({ policy_ref: 'default', evidence_hold: false }),
  });
}

async function post(path, body, { baseUrl, token, fetchImpl = globalThis.fetch, signal }) {
  const response = await fetchImpl(new URL(path, normalizeDaemonOrigin(baseUrl)), {
    method: 'POST', signal, headers: { accept: 'application/json', 'content-type': 'application/json',
      authorization: `Bearer ${required(token, 'token')}`, 'x-focusa-permissions': 'write:*' }, body: JSON.stringify(body),
  });
  let envelope = null;
  try { envelope = await response.json(); } catch { throw new SessionCreateError('invalid_envelope', 'daemon response is not JSON', response.status); }
  if (envelope?.schema !== ENVELOPE_SCHEMA) throw new SessionCreateError('unsupported', 'daemon envelope schema is unsupported', response.status);
  if (!response.ok || envelope.ok !== true) {
    if (response.status === 409 || envelope.failure_class === 'idempotency_key_reused') throw new SessionCreateConflictError();
    const kind = response.status === 401 ? 'unauthenticated' : response.status === 403 ? 'forbidden' : response.status >= 500 ? 'degraded' : 'rejected';
    throw new SessionCreateError(kind, envelope.status ?? 'request rejected', response.status);
  }
  return envelope;
}

export async function preflightSafeSession(config, options) {
  const envelope = await post('/v1/silent-sessions/preflight', { config, layers: [] }, options);
  const data = envelope.data;
  if (envelope.status !== 'preflight_ok' || data?.validation?.valid !== true || typeof data.redacted_config_hash !== 'string' || !data.redacted_config_hash) {
    throw new SessionCreateError('invalid_preflight', 'daemon preflight did not return a valid redacted config hash');
  }
  return Object.freeze({ config, redacted_config_hash: data.redacted_config_hash, config_digest: await digest(config) });
}

function stable(value) {
  if (Array.isArray(value)) return `[${value.map(stable).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stable(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}
async function digest(value) {
  const bytes = await crypto.subtle.digest('SHA-256', encoder.encode(stable(value)));
  return [...new Uint8Array(bytes)].map((item) => item.toString(16).padStart(2, '0')).join('');
}

export async function createPreflightedSession({ preflight, idempotency_key, idempotencyStore, requestOptions }) {
  if (!preflight?.redacted_config_hash || !preflight?.config || !preflight?.config_digest) throw new TypeError('successful preflight is required');
  if (await digest(preflight.config) !== preflight.config_digest) throw new SessionCreateConflictError('configuration changed after preflight');
  const key = required(idempotency_key, 'idempotency_key', 200);
  if (!idempotencyStore?.load || !idempotencyStore?.persist) throw new TypeError('durable idempotency store is required');
  const request = { config: preflight.config, layers: [], idempotency_key: key };
  const request_digest = await digest(request);
  const existing = await idempotencyStore.load(key);
  if (existing && existing.request_digest !== request_digest) throw new SessionCreateConflictError();
  if (!existing) {
    await idempotencyStore.persist(Object.freeze({ idempotency_key: key, request_digest, redacted_config_hash: preflight.redacted_config_hash }));
    const committed = await idempotencyStore.load(key);
    if (committed?.request_digest !== request_digest) throw new SessionCreateError('storage_failure', 'idempotency key was not durably committed');
  }
  let envelope;
  try { envelope = await post('/v1/silent-sessions', request, requestOptions); }
  catch (error) {
    if (error instanceof SessionCreateError && !['invalid_envelope','unsupported','degraded'].includes(error.kind)) throw error;
    envelope = await post('/v1/silent-sessions', request, requestOptions);
  }
  const data = envelope.data;
  if (data?.redacted_config_hash && data.redacted_config_hash !== preflight.redacted_config_hash) throw new SessionCreateConflictError('created config hash differs from approved preflight');
  if (!data?.session?.id || !(data.run?.id || data.run_id)) throw new SessionCreateError('invalid_envelope', 'create response lacks exact session/run target');
  return Object.freeze({ session: data.session, run: data.run ?? Object.freeze({ id: data.run_id, generation: data.run_generation }),
    redacted_config_hash: data.redacted_config_hash ?? preflight.redacted_config_hash, idempotent_replay: data.idempotent_replay === true });
}
