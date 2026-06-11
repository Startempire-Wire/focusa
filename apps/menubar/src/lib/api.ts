import { diagnosticsStore } from '$lib/stores/diagnostics.svelte';

export const DEFAULT_API_URL = 'http://127.0.0.1:8787';
export const SAVED_CONNECTIONS_KEY = 'focusa_saved_connections_v1';
export const HAS_CONNECTED_KEY = 'focusa_has_connected_successfully';

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
  try {
    const resp = await fetch(url, {
      method,
      headers: body === undefined ? headers : { 'Content-Type': 'application/json', ...headers },
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
      const err = new Error(data?.error || data?.message || `${path} returned HTTP ${resp.status}`);
      (err as any).status = resp.status;
      (err as any).failure_class = data?.failure_class || `http_${resp.status}`;
      (err as any).body = data;
      diagnosticsStore.record({ area: 'api', phase: 'http_response', error: err, url, method, status: resp.status, body: data });
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
    workspace_id: ctx.continuityId || ctx.projectRoot || 'menubar',
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
