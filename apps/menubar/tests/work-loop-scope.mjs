import assert from 'node:assert/strict';
import {
  WORK_LOOP_STATUS_SCHEMA,
  WORK_LOOP_TYPED_STATES,
  compatibleWorkLoopStatusState,
  evaluateWorkLoopAuthority,
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
