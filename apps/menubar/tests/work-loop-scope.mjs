import assert from 'node:assert/strict';
import {
  WORK_LOOP_STATUS_SCHEMA,
  WORK_LOOP_TYPED_STATES,
  compatibleWorkLoopStatusState,
  evaluateWorkLoopAuthority,
  predictionScopedPath,
  projectScopedPath,
  workLoopScopedPaths,
} from '../src/lib/workLoopScope.js';

const root = '/tmp/focusa-project';
const continuity = 'workloop-completion';
const paths = workLoopScopedPaths(root, continuity);
assert.ok(paths, 'complete authority must produce scoped Work Loop paths');
for (const path of Object.values(paths)) {
  const url = new URL(path, 'http://127.0.0.1:8787');
  assert.equal(url.searchParams.get('project_root'), root);
  assert.equal(url.searchParams.get('continuity_id'), continuity);
}
assert.equal(workLoopScopedPaths(root, ''), null);
assert.equal(workLoopScopedPaths('', continuity), null);

for (const path of [
  '/v1/metacognition/status',
  '/v1/metacognition/evaluations/recent?limit=5',
  '/v1/focus/snapshots/recent?limit=5',
]) {
  const scoped = projectScopedPath(path, root, continuity);
  assert.ok(scoped, `${path} must accept exact project authority`);
  const url = new URL(scoped, 'http://127.0.0.1:8787');
  assert.equal(url.searchParams.get('project_root'), root);
  assert.equal(url.searchParams.get('continuity_id'), continuity);
}
assert.equal(projectScopedPath('/v1/metacognition/status', root, ''), null);
assert.equal(projectScopedPath('/health', root, continuity), null);

const identity = {
  project_root: root,
  project_id: 'focusa',
  canonical_name: 'Focusa',
  fingerprint: 'project-fnv1a64:1234',
};
for (const path of ['/v1/predictions/recent?limit=5', '/v1/predictions/stats']) {
  const scoped = predictionScopedPath(path, identity, continuity);
  assert.ok(scoped, `${path} must accept complete typed identity`);
  const url = new URL(scoped, 'http://127.0.0.1:8787');
  assert.equal(url.searchParams.get('scope_kind'), 'project');
  assert.equal(url.searchParams.get('scope_id'), 'focusa');
  assert.equal(url.searchParams.get('root_path'), root);
  assert.equal(url.searchParams.get('canonical_name'), 'Focusa');
  assert.equal(url.searchParams.get('fingerprint'), identity.fingerprint);
  assert.equal(url.searchParams.get('continuity_id'), continuity);
}
assert.equal(predictionScopedPath('/v1/predictions/stats', { ...identity, fingerprint: '' }, continuity), null);
assert.equal(predictionScopedPath('/v1/metacognition/status', identity, continuity), null);

for (const state of WORK_LOOP_TYPED_STATES) {
  assert.equal(compatibleWorkLoopStatusState(WORK_LOOP_STATUS_SCHEMA, state), state);
}
assert.equal(compatibleWorkLoopStatusState('focusa.work_loop_status.v999', 'healthy'), 'unsupported');
assert.equal(compatibleWorkLoopStatusState(WORK_LOOP_STATUS_SCHEMA, 'maybe'), 'unsupported');
assert.equal(compatibleWorkLoopStatusState(WORK_LOOP_STATUS_SCHEMA, ''), 'unsupported');

const current = { project_root: root, continuity_id: continuity };
assert.deepEqual(
  evaluateWorkLoopAuthority(current, {
    project_root: root,
    continuity_id: continuity,
    canonical: true,
  }),
  {
    currentProjectRoot: root,
    currentContinuityId: continuity,
    loopProjectRoot: root,
    loopContinuityId: continuity,
    scopeMatches: true,
    stale: false,
    reason: 'matched',
  },
);
assert.equal(
  evaluateWorkLoopAuthority(current, {
    project_root: root,
    continuity_id: 'pi-turn-76',
    canonical: true,
  }).reason,
  'continuity_mismatch',
);
assert.equal(
  evaluateWorkLoopAuthority(current, {
    project_root: '/tmp/other',
    continuity_id: continuity,
    canonical: true,
  }).reason,
  'project_root_mismatch',
);
assert.equal(
  evaluateWorkLoopAuthority(current, {
    project_root: root,
    continuity_id: continuity,
    canonical: false,
  }).reason,
  'workpoint_noncanonical',
);
assert.equal(
  evaluateWorkLoopAuthority({ project_root: root, continuity_id: '' }, {
    project_root: root,
    continuity_id: continuity,
    canonical: true,
  }).reason,
  'current_scope_unbound',
);

console.log('work-loop menubar scope contract: ok');
