import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { loadRegisteredTool } from './helpers/registered-tool-harness.mjs';

const source = process.env.FOCUSA_TEST_TOOLS_SOURCE
  ? readFileSync(process.env.FOCUSA_TEST_TOOLS_SOURCE, 'utf8') : undefined;
const root = '/tmp/focusa-genesis-status-owned-project';
function context(body, ok = true, continuity = 'status-a', requestedRoot = root) {
  const calls = [];
  const state = { ensured: 0 };
  const tool = loadRegisteredTool('focusa_project_genesis', {
    normalizeProjectRoot: value => String(value || '').replace(/\/$/, ''),
    getLastProjectIdentity: () => ({ project_root: root }),
    getSessionCwd: () => root,
    isProjectRootAuthoritySafe: value => value === root || value === requestedRoot,
    getContinuityId: () => continuity,
    ensureContinuityId: () => { state.ensured++; return 'unwanted-new-continuity'; },
    getSessionId: () => 'status-session-a',
    compactApiEcho: value => value,
    focusaFetchDetailed: async (path, options) => {
      calls.push({ path, options });
      return { ok, status: ok ? 200 : 409, body };
    },
  }, [], source);
  return { tool, calls, state };
}
function ready(continuity) {
  return {
    schema: 'focusa.project_genesis.v1', status: 'ready',
    project_identity: { project_root: root, verified: true },
    authority: { scope: root, continuity_id: continuity },
    first_workpoint: { workpoint_id: `${continuity}-workpoint`, canonical: true },
    next_action: `continue ${continuity}`,
  };
}
async function run(body, ok = true) {
  const c = context(body, ok);
  const result = await c.tool.execute('fixture', {
    action: 'status', project_root: root, continuity_id: 'status-a',
  });
  return { ...c, result };
}
const foreign = await run(ready('status-b'));
assert.equal(foreign.result.details.canonical, false,
  'Genesis consumer promoted a foreign ready receipt into current workstream authority');
assert.notEqual(foreign.result.details.status, 'ready');
assert.equal(foreign.result.details.requested_scope.continuity_id, 'status-a');
assert.equal(foreign.result.details.resolved_scope.continuity_id, 'status-b');
assert.notEqual(foreign.result.details.continuity_id, 'status-a');
assert.equal(new URL(foreign.calls[0].path, 'http://fixture').searchParams.get('continuity_id'), 'status-a');
assert.ok(foreign.result.details.next_tools.includes('focusa_workpoint_resume'));
assert.ok(!foreign.result.content[0].text.includes('continue status-b'));

const matched = {
  ...ready('status-a'), canonical: true,
  status_projection_schema: 'focusa.project_genesis_status.v1',
  requested_scope: { project_root: root, continuity_id: 'status-a' },
  resolved_scope: { project_root: root, continuity_id: 'status-a' },
  scope_validation: { status: 'verified', scope_kind: 'project_workstream_record', execution_admission: false },
};
const valid = await run(matched);
assert.equal(valid.result.details.canonical, true);
assert.equal(valid.result.details.continuity_id, 'status-a');
const worktree = `${root}-worktree`;
const normalized = context({
  ...matched,
  requested_scope: { project_root: worktree, continuity_id: 'status-a' },
  scope_validation: { ...matched.scope_validation, canonical_project_root: root },
}, true, 'status-a', worktree);
const normalizedResult = await normalized.tool.execute('fixture', {
  action: 'status', project_root: worktree, continuity_id: 'status-a',
});
assert.equal(normalizedResult.details.canonical, true);
assert.equal(normalizedResult.details.requested_scope.project_root, worktree);
assert.equal(normalizedResult.details.resolved_scope.project_root, root);
assert.equal(normalizedResult.details.execution_admission, false);
for (const body of [
  ready('status-a'),
  { ...matched, canonical: false },
  { ...matched, resolved_scope: undefined },
  { ...matched, project_identity: { project_root: '/foreign', verified: true } },
  { ...matched, project_identity: { project_root: root, verified: false } },
  { ...matched, authority: { scope: root, continuity_id: 'status-b' } },
  { ...matched, resolved_scope: { project_root: '/foreign', continuity_id: 'status-a' } },
  { ...matched, scope_validation: { status: 'unknown' } },
  { ...matched, scope_validation: { ...matched.scope_validation, execution_admission: true } },
  { ...matched, requested_scope: { project_root: '/foreign', continuity_id: 'status-a' } },
  { ...matched, authority: { scope: {}, continuity_id: 'status-a' } },
  { ...matched, status_projection_schema: 'unsupported' },
  { ...matched, status: 'unknown' },
]) {
  assert.equal((await run(body)).result.details.canonical, false,
    'legacy, mismatched or unqualified Genesis status became canonical');
}
assert.equal((await run(matched, false)).result.details.canonical, false);
const unscoped = context(ready('status-b'), true, '');
const history = await unscoped.tool.execute('fixture', { action: 'status', project_root: root });
assert.equal(unscoped.state.ensured, 0, 'read-only Genesis status initialized a new continuity');
assert.equal(history.details.canonical, false);
assert.equal(new URL(unscoped.calls[0].path, 'http://fixture').searchParams.get('continuity_id'), null);
console.log('PASS: actual Genesis consumer preserves requested/resolved identity, refuses foreign/legacy readiness and forwards continuity');
