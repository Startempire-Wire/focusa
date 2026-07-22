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
 * Build the three read-only Work Loop URLs only when both authority keys exist.
 *
 * @param {unknown} projectRoot
 * @param {unknown} continuityId
 * @returns {{ status: string, health: string, checkpoints: string } | null}
 */
export function workLoopScopedPaths(projectRoot, continuityId) {
  const root = clean(projectRoot);
  const continuity = clean(continuityId);
  if (!root || !continuity) return null;

  const query = new URLSearchParams({ project_root: root, continuity_id: continuity }).toString();
  return {
    status: `/v1/work-loop/status?summary_only=true&${query}`,
    health: `/v1/work-loop/health?${query}`,
    checkpoints: `/v1/work-loop/checkpoints?${query}`,
  };
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
