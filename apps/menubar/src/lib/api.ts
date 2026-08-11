import { diagnosticsStore } from '$lib/stores/diagnostics.svelte';
import { getCurrentAuthToken } from '$lib/stores/pairing.svelte';
import type { ScopeContext } from '$lib/projectContext.svelte';
import type { SemanticPairActionRequest, SemanticPairStatus } from './types/focus-canvas';
import { bindSpec138OperationPath, spec138Operation } from './generated/spec138-operations';

export const DEFAULT_API_URL = 'http://127.0.0.1:8787';
export const SAVED_CONNECTIONS_KEY = 'focusa_saved_connections_v1';
export const HAS_CONNECTED_KEY = 'focusa_has_connected_successfully';
export const PUBLIC_PAIRING_URL_KEY = 'focusa_public_pairing_url';

export interface SavedConnection {
  url: string;
  label: string;
  first_connected_at: string;
  last_connected_at: string;
}

export function normalizeApiUrl(url: string): string {
  return url.trim().replace(/\/$/, '');
}

export function loadSavedConnections(): SavedConnection[] {
  try {
    const raw = localStorage.getItem(SAVED_CONNECTIONS_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter((c) => c?.url).map((c) => ({
      url: normalizeApiUrl(String(c.url)),
      label: String(c.label || c.url),
      first_connected_at: String(c.first_connected_at || c.last_connected_at || new Date().toISOString()),
      last_connected_at: String(c.last_connected_at || c.first_connected_at || new Date().toISOString()),
    })) : [];
  } catch {
    return [];
  }
}

export function saveConnection(url: string, label?: string): SavedConnection[] {
  const normalized = normalizeApiUrl(url);
  const now = new Date().toISOString();
  const current = loadSavedConnections();
  const existing = current.find((c) => c.url === normalized);
  const next: SavedConnection = {
    url: normalized,
    label: label || existing?.label || normalized,
    first_connected_at: existing?.first_connected_at || now,
    last_connected_at: now,
  };
  const merged = [next, ...current.filter((c) => c.url !== normalized)];
  try {
    localStorage.setItem(SAVED_CONNECTIONS_KEY, JSON.stringify(merged));
    localStorage.setItem(HAS_CONNECTED_KEY, 'true');
  } catch {}
  return merged;
}

export function removeSavedConnection(url: string): SavedConnection[] {
  const normalized = normalizeApiUrl(url);
  const next = loadSavedConnections().filter((c) => c.url !== normalized);
  try {
    localStorage.setItem(SAVED_CONNECTIONS_KEY, JSON.stringify(next));
    if (next.length === 0) localStorage.removeItem(HAS_CONNECTED_KEY);
  } catch {}
  return next;
}

export function hasEverConnected(): boolean {
  try {
    return localStorage.getItem(HAS_CONNECTED_KEY) === 'true' || loadSavedConnections().length > 0;
  } catch {
    return false;
  }
}

export interface EpistemicScopeIdentity {
  project_root: string;
  project_id?: string;
  scope_id?: string;
  canonical_name: string;
  fingerprint: string;
}

/** Invoke generated Spec138 operations; only daemon results carry authority. */
export async function requestSpec138Operation(
  operationId: string,
  identity: EpistemicScopeIdentity,
  continuityId: string,
  id?: string,
  event?: unknown,
): Promise<unknown> {
  const descriptor = spec138Operation(operationId);
  if (!descriptor) throw new Error(`Unknown Spec138 operation: ${operationId}`);
  const scopeId = String(identity.scope_id || identity.project_id || '').trim();
  if (!identity.project_root || !identity.canonical_name || !identity.fingerprint || !scopeId || !continuityId.trim()) {
    throw new Error('Canonical Spec138 operation requires complete typed project scope');
  }
  const path = bindSpec138OperationPath(descriptor.path, id);
  const rootScope = {
    scope_kind: 'project', scope_id: scopeId, root_path: identity.project_root,
    canonical_name: identity.canonical_name, fingerprint: identity.fingerprint,
  };
  if (descriptor.method === 'GET') {
    const query = new URLSearchParams({
      scope_kind: 'project', scope_id: scopeId, root_path: identity.project_root,
      canonical_name: identity.canonical_name, fingerprint: identity.fingerprint,
      continuity_id: continuityId,
    });
    return requestJson(`${path}?${query.toString()}`);
  }
  if (!event) throw new Error(`${operationId} requires a typed ScopedAuthorityEvent`);
  return requestJson(path, {
    method: 'POST',
    body: {
      operation_id: descriptor.operation_id,
      scope: { root_scope: rootScope, continuity_id: continuityId },
      event,
    },
  });
}

export interface ApiRequestOptions {
  timeoutMs?: number;
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
}

export interface NormalizedToolResult {
  status?: string;
  canonical?: boolean;
  advisory?: boolean;
  degraded?: boolean;
  stale?: boolean;
  scope?: unknown;
  scope_status?: string;
  scope_source?: string;
  failure_class?: string;
  retry?: unknown;
  side_effects?: unknown;
  evidence_refs?: unknown;
  next_tools?: unknown;
}

export function getApiUrl(): string {
  try {
    // Headless browser mode: the e2e harness injects a daemon URL via
    // window.__FOCUSA_DAEMON_URL__ before app boot. Allows headless tests
    // to point at a test daemon without touching localStorage.
    const w =
      typeof window !== 'undefined'
        ? (window as { __FOCUSA_DAEMON_URL__?: string }).__FOCUSA_DAEMON_URL__
        : undefined;
    if (w && w.length > 0) return w;
    return localStorage.getItem('focusa_api_url') || DEFAULT_API_URL;
  } catch {
    return DEFAULT_API_URL;
  }
}

export function setApiUrl(url: string): void {
  try {
    localStorage.setItem('focusa_api_url', url);
  } catch {}
}

export function summarizeError(error: unknown): string {
  return error instanceof Error ? error.message : 'Network error';
}

export function normalizeToolResult(payload: any): NormalizedToolResult {
  const result = payload?.details?.tool_result_v1 ?? payload?.tool_result_v1 ?? payload;
  return {
    status: result?.status,
    canonical: result?.canonical,
    advisory: result?.advisory ?? result?.advisory_only,
    degraded: result?.degraded,
    stale: result?.stale,
    scope: result?.scope,
    scope_status: result?.scope?.scope_status ?? result?.scope_status,
    scope_source: result?.scope?.scope_source ?? result?.scope_source,
    failure_class: result?.failure_class,
    retry: result?.retry,
    side_effects: result?.side_effects,
    evidence_refs: result?.evidence_refs,
    next_tools: result?.next_tools,
  };
}

export function isDegraded(payload: any): boolean {
  const result = normalizeToolResult(payload);
  return result.degraded === true || result.stale === true || result.canonical === false || result.status === 'pending' || result.status === 'blocked';
}

export async function requestJson<T = any>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  const { timeoutMs = 3000, method = 'GET', body, headers = {} } = options;
  const base = getApiUrl();
  const url = `${base}${path}`;
  // V2: inject the device pairing token as Bearer from the in-memory
  // currentAuthToken (Keychain-backed). The full token is NEVER mirrored to
  // localStorage anymore. The api module is consumed by Svelte components
  // that import { currentAuthToken } from the pairing store.
  const mergedHeaders = { ...headers };
  if (body && typeof body === 'object' && !Array.isArray(body)) {
    const values = body as Record<string, unknown>;
    const key = values.idempotency_key ?? values.idempotencyKey ?? values.request_id ?? values.requestId;
    if (typeof key === 'string' && key.trim() && !mergedHeaders['Idempotency-Key']) {
      mergedHeaders['Idempotency-Key'] = key.trim();
    }
  }
  if (!mergedHeaders['Authorization'] && !mergedHeaders['authorization']) {
    try {
      const tok = getCurrentAuthToken();
      if (tok && typeof tok === 'string' && tok.trim().length > 0) {
        mergedHeaders['Authorization'] = `Bearer ${tok.trim()}`;
      }
    } catch {
      /* ignore */
    }
  }
  try {
    const resp = await fetch(url, {
      method,
      headers: body === undefined ? mergedHeaders : { 'Content-Type': 'application/json', ...mergedHeaders },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: AbortSignal.timeout(timeoutMs),
    });
    const text = await resp.text();
    let data: any = null;
    if (text) {
      try { data = JSON.parse(text); } catch (error) {
        diagnosticsStore.record({ area: 'api', phase: 'json_parse', error_class: 'json_parse', error, url, method, status: resp.status, body: text });
        throw error;
      }
    }
    if (!resp.ok) {
      const errorBody = data?.error && typeof data.error === 'object' ? data.error : null;
      const code = String(errorBody?.code || data?.code || '');
      const message = String(errorBody?.message || data?.error || data?.message || `${path} returned HTTP ${resp.status}`);
      const err = new Error(message);
      (err as any).status = resp.status;
      (err as any).code = code || undefined;
      (err as any).failure_class = code.startsWith('ENTITLEMENT_')
        ? 'entitlement_blocked'
        : data?.failure_class || `http_${resp.status}`;
      (err as any).required_feature = errorBody?.required_feature || null;
      (err as any).limit_bucket = errorBody?.limit_bucket || null;
      (err as any).recovery = errorBody?.recovery || null;
      (err as any).body = data;
      diagnosticsStore.record({ area: 'api', phase: 'http_response', error: err, url, method, status: resp.status, body: data });
      // FOCUSA_FIX-1vfz/67ud: Handle 401 auth failures from daemon.
      if (resp.status === 401) {
        const fullClass = String(data?.failure_class || '');
        if (fullClass === 'pairing_revoked') {
          // Token was valid but the device was revoked on the daemon side.
          // Clear the cached token so subsequent calls start unauthenticated.
          try {
            // @ts-ignore: clearCurrentAuthToken is exported from pairing store
            const { clearCurrentAuthToken } = await import('$lib/stores/pairing.svelte');
            if (typeof clearCurrentAuthToken === 'function') clearCurrentAuthToken();
          } catch { /* best effort */ }
          // Surface a revocation notification to the operator.
          try {
            const { sendNotification } = await import('@tauri-apps/plugin-notification');
            if (typeof sendNotification === 'function') sendNotification({ title: 'Focusa: Pairing Revoked', body: `Your device was revoked from daemon at ${base}. Re-pair from Settings > Pairing.` });
          } catch { /* Tauri notify not available in web preview */ }
        } else if (fullClass === 'token_expired') {
          // Token has expired; clear it and let the operator re-pair.
          try {
            const { clearCurrentAuthToken } = await import('$lib/stores/pairing.svelte');
            if (typeof clearCurrentAuthToken === 'function') clearCurrentAuthToken();
          } catch { /* best effort */ }
          try {
            const { sendNotification } = await import('@tauri-apps/plugin-notification');
            if (typeof sendNotification === 'function') sendNotification({ title: 'Focusa: Token Expired', body: 'Your device pairing token has expired. Re-pair from Settings > Pairing.' });
          } catch { /* Tauri notify not available in web preview */ }
        }
      }
      throw err;
    }
    return data as T;
  } catch (error) {
    if (!(error as any)?.status) {
      diagnosticsStore.record({ area: 'api', phase: 'fetch', error, url, method, context: { timeoutMs } });
    }
    throw error;
  }
}

export async function fetchJson<T = any>(path: string, timeoutMs = 3000): Promise<T> {
  return requestJson<T>(path, { timeoutMs });
}

export async function postJson<T = any>(path: string, body?: unknown, timeoutMs = 3000): Promise<T> {
  return requestJson<T>(path, { method: 'POST', body, timeoutMs });
}

/** Read complete semantic operation truth for the menubar surface. */
export async function fetchSemanticPairStatus(
  projectRoot: string,
  continuityId: string,
): Promise<SemanticPairStatus> {
  const query = new URLSearchParams({ project_root: projectRoot, continuity_id: continuityId });
  const [status, registry] = await Promise.all([
    fetchJson<Omit<SemanticPairStatus, 'operations'>>(`/v1/semantic-integrity/status?${query}`),
    fetchJson<{ items: SemanticPairStatus['operations'] }>(`/v1/semantic-integrity/operations?${query}&limit=100`),
  ]);
  return { ...status, operations: registry.items ?? [] };
}

/** Invoke a visible semantic operation; daemon policy remains authoritative. */
export async function invokeSemanticPairAction<T = unknown>(
  request: SemanticPairActionRequest,
): Promise<T> {
  const id = encodeURIComponent(request.operation_id);
  return postJson<T>(`/v1/semantic-integrity/operations/${id}`, {
    contract: 'focusa.semantic-integrity.operation.v1',
    operation_id: request.operation_id,
    scope: {
      project_root: request.project_root,
      continuity_id: request.continuity_id,
    },
    payload: {
      ...(request.payload ?? {}),
      ...(request.pair_id ? { pair_id: request.pair_id } : {}),
    },
    idempotency_key: request.idempotency_key,
    confirmation: request.confirmation,
  });
}

/**
 * Build a minimal FocusaSessionIdentity for menubar → daemon calls.
 * Fills required fields; uses menubar as resume_source so the API can distinguish
 * menubar-initiated requests from Pi/CLI surfaces.
 * §33 / §42 parity: menubar is a first-class Focusa surface.
 */
export interface MenubarSessionContext {
  projectRoot?: string;
  continuityId?: string;
  sessionId?: string;
}

export function buildFocusaSessionIdentity(ctx: MenubarSessionContext): Record<string, unknown> {
  const now = new Date().toISOString();
  // Generate a stable session incarnation ID for this menubar session window
  const incId = `menubar-${Date.now()}`;
  return {
    session_frame_key: ctx.sessionId || `menubar-${incId}`,
    session_incarnation_id: incId,
    project_root: ctx.projectRoot || '',
    cwd: ctx.projectRoot || '',          // menubar has no cwd concept; use project_root as proxy
    started_at: now,
    resume_source: 'menubar',
    continuity_id: ctx.continuityId || null,
    pi_session_id: ctx.sessionId || null,
    canonical_scope: false,             // menubar scope is advisory, not canonical
  };
}

/**
 * POST helper that auto-injects session_identity into the body.
 * Use this instead of postJson for all Focusa tool calls.
 */
export async function focusaPost<T = any>(
  path: string,
  body: Record<string, unknown>,
  ctx: MenubarSessionContext,
  timeoutMs = 3000,
): Promise<T> {
  return postJson<T>(path, {
    ...body,
    project_root: body.project_root ?? ctx.projectRoot ?? undefined,
    continuity_id: body.continuity_id ?? ctx.continuityId ?? undefined,
    session_id: body.session_id ?? ctx.sessionId ?? undefined,
    session_identity: buildFocusaSessionIdentity(ctx),
  }, timeoutMs);
}
