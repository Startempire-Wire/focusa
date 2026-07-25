// Scope-only helpers for the Work Loop menubar surface.
// These functions intentionally have no runtime-store dependency so the same
// fail-closed contract can be tested without rendering Svelte components.

export const WORK_LOOP_STATUS_SCHEMA = 'focusa.work_loop_status.v3';
export const WORK_LOOP_TYPED_STATES = Object.freeze([
  'absent',
  'unavailable',
  'stale',
  'unsupported',
  'blocked',
  'exhausted',
  'zero',
  'healthy',
]);

/**
 * Preserve recognized status values and fail closed on incompatible payloads.
 *
 * @param {unknown} schema
 * @param {unknown} state
 */
export function compatibleWorkLoopStatusState(schema, state) {
  const normalized = clean(state);
  return schema === WORK_LOOP_STATUS_SCHEMA && WORK_LOOP_TYPED_STATES.includes(normalized)
    ? normalized
    : 'unsupported';
}

/**
 * Scope a daemon read to one exact project workstream. Missing authority or a
 * non-API path fails closed instead of issuing an unscoped request.
 *
 * @param {unknown} path
 * @param {unknown} projectRoot
 * @param {unknown} continuityId
 * @returns {string | null}
 */
export function projectScopedPath(path, projectRoot, continuityId) {
  const route = clean(path);
  const root = clean(projectRoot);
  const continuity = clean(continuityId);
  if (!route.startsWith('/v1/') || !root || !continuity) return null;

  const url = new URL(route, 'http://focusa.local');
  url.searchParams.set('project_root', root);
  url.searchParams.set('continuity_id', continuity);
  return `${url.pathname}?${url.searchParams.toString()}`;
}

/**
 * Scope prediction reads with the complete typed project identity required by
 * the predictions API. Partial identity fails closed.
 *
 * @param {unknown} path
 * @param {{ project_root?: unknown, project_id?: unknown, scope_id?: unknown, canonical_name?: unknown, fingerprint?: unknown } | null | undefined} identity
 * @param {unknown} continuityId
 * @returns {string | null}
 */
export function predictionScopedPath(path, identity, continuityId) {
  const route = clean(path);
  const rootPath = clean(identity?.project_root);
  const fingerprint = clean(identity?.fingerprint);
  const scopeId = clean(identity?.scope_id) || clean(identity?.project_id);
  const canonicalName = clean(identity?.canonical_name);
  const continuity = clean(continuityId);
  if (!/^\/v1\/predictions\/(?:recent|stats)(?:\?|$)/.test(route) || !rootPath || !fingerprint || !scopeId || !canonicalName || !continuity) {
    return null;
  }

  const url = new URL(route, 'http://focusa.local');
  url.searchParams.set('scope_kind', 'project');
  url.searchParams.set('scope_id', scopeId);
  url.searchParams.set('root_path', rootPath);
  url.searchParams.set('canonical_name', canonicalName);
  url.searchParams.set('fingerprint', fingerprint);
  url.searchParams.set('continuity_id', continuity);
  return `${url.pathname}?${url.searchParams.toString()}`;
}

/**
 * Build the three read-only Work Loop URLs only when both authority keys exist.
 *
 * @param {unknown} projectRoot
 * @param {unknown} continuityId
 * @returns {{ status: string, health: string, checkpoints: string } | null}
 */
export function workLoopScopedPaths(projectRoot, continuityId) {
  const status = projectScopedPath('/v1/work-loop/status?summary_only=true', projectRoot, continuityId);
  const health = projectScopedPath('/v1/work-loop/health', projectRoot, continuityId);
  const checkpoints = projectScopedPath('/v1/work-loop/checkpoints', projectRoot, continuityId);
  if (!status || !health || !checkpoints) return null;
  return { status, health, checkpoints };
}

/**
 * Compare current menubar authority with the loop execution partition and its
 * canonical Workpoint. Missing or mismatched fields are stale by construction.
 *
 * @param {{ project_root?: unknown, continuity_id?: unknown }} current
 * @param {{ project_root?: unknown, continuity_id?: unknown, canonical?: unknown }} loop
 */
export function evaluateWorkLoopAuthority(current, loop) {
  const currentProjectRoot = clean(current?.project_root);
  const currentContinuityId = clean(current?.continuity_id);
  const loopProjectRoot = clean(loop?.project_root);
  const loopContinuityId = clean(loop?.continuity_id);

  let reason = 'matched';
  if (!currentProjectRoot || !currentContinuityId) reason = 'current_scope_unbound';
  else if (!loopProjectRoot || !loopContinuityId) reason = 'loop_scope_unbound';
  else if (currentProjectRoot !== loopProjectRoot) reason = 'project_root_mismatch';
  else if (currentContinuityId !== loopContinuityId) reason = 'continuity_mismatch';
  else if (loop?.canonical !== true) reason = 'workpoint_noncanonical';

  return {
    currentProjectRoot,
    currentContinuityId,
    loopProjectRoot,
    loopContinuityId,
    scopeMatches: reason === 'matched',
    stale: reason !== 'matched',
    reason,
  };
}

/** @param {unknown} value */
function clean(value) {
  return typeof value === 'string' ? value.trim() : '';
}
