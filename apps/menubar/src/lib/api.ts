export const DEFAULT_API_URL = 'http://127.0.0.1:8787';

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
  const resp = await fetch(`${base}${path}`, {
    method,
    headers: body === undefined ? headers : { 'Content-Type': 'application/json', ...headers },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(timeoutMs),
  });
  if (!resp.ok) {
    const data = await resp.json().catch(() => ({}));
    throw new Error(data?.error || data?.message || `${path} returned HTTP ${resp.status}`);
  }
  return await resp.json() as T;
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
