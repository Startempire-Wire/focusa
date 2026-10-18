import assert from 'node:assert/strict';
import { AsyncLocalStorage } from 'node:async_hooks';
import { loadRegisteredTool } from './helpers/registered-tool-harness.mjs';
import { loadPureSourceModule, loadSourceSection } from './helpers/source-section-harness.mjs';

// Execute production scope projection, current-context accessor, both registered
// callbacks and process adapter. Only the actual child process is a test boundary.
const scope = loadPureSourceModule('scoped-state.ts');
const invocations = new AsyncLocalStorage();
const currentAttachmentKey = loadSourceSection(
  'state.ts', 'export function currentAttachmentKey()',
  '/**\n * Promote one verified Pi session',
  { attachmentRuntimeContext: invocations }, 'currentAttachmentKey'
);
const spawned = [];
let childFailure = false;
let deferChild = false;
const callbacks = [];
const { dispatchBackgroundJob, BackgroundJobToolError } = loadSourceSection(
  'background-job-tools.ts', 'export const BACKGROUND_JOB_DISPATCH_SCHEMA',
  'export function backgroundJobsApiOrigin',
  { execFile(file, args, options, callback) {
    spawned.push({ file, args, options });
    const finish = () => childFailure
      ? callback(Object.assign(new Error('CLI rejected request'), { code: 1 }), '',
          'NORTH_STAR_ADMISSION_BLOCKED: source_recovery_test')
      : callback(null, JSON.stringify({
          schema: 'focusa.background_job_dispatch.v1', status: 'dispatched',
          job_id: `fixture-${spawned.length}`, name: args[args.indexOf('--name') + 1],
          log_path: '/tmp/source-only-dispatch-fixture.log',
        }), '');
    if (deferChild) callbacks.push(finish);
    else finish();
  } }, '{ dispatchBackgroundJob, BackgroundJobToolError }'
);
const dependencies = {
  currentAttachmentKey,
  getAttachmentRuntime: () => ({ sessionCwd: '/root' }),
  projectAttachmentEnvironment: scope.projectAttachmentEnvironment,
  dispatchBackgroundJob, BackgroundJobToolError,
  toolResult: (ok, status, message, data) => ({ ok, status, message, data }),
};
const sections = [['function backgroundJobErrorDetails(', 'function safeErrorText(']];
const single = loadRegisteredTool('focusa_bg_run', dependencies, sections);
const many = loadRegisteredTool('focusa_bg_run_many', dependencies, sections);
const registered = ['a', 'b'].map((id) => scope.registerVerifiedScopeRef({
  scope_kind: 'project', scope_id: `project:test-${id}`, root_path: `/work/project-${id}`,
  canonical_name: `Source fixture ${id}`, fingerprint: `fixture-${id}`,
}));
const keyFor = (i, continuity = `continuity-${i}`) => ({
  workstream: scope.buildProjectWorkstreamKey(registered[i].root_path, continuity),
  instance_id: `fixture-instance-${i}`, session_id: `fixture-session-${i}`,
  attachment_id: `fixture-attachment-${i}`,
});
const keyA = keyFor(0), keyB = keyFor(1);
const runSingle = (key, name = 'source fixture', extra = {}) => invocations.run(
  key, () => single.execute('fixture-call', { name, command: 'printf source-only', cwd: '/tmp/workload', ...extra })
);
const runMany = (key) => invocations.run(key, () => many.execute('fixture-many', {
  jobs: [{ name: 'first', command: 'printf one' }, { name: 'second', command: 'printf two' }],
}));
const saved = process.env.FOCUSA_ATTACHMENT_KEY_V1;
try {
  delete process.env.FOCUSA_ATTACHMENT_KEY_V1;
  let result = await runSingle(keyA);
  assert.equal(result.ok, true);
  // Baseline RED must fail here: an actual registered callback + dispatcher
  // produced no per-child invocation identity, not a missing test dependency.
  assert.deepEqual(JSON.parse(spawned.at(-1).options.env?.FOCUSA_ATTACHMENT_KEY_V1 || 'null'), keyA,
    'registered dispatch must forward its invocation attachment without startup publication');
  assert.equal(spawned.at(-1).options.cwd, '/tmp/workload');
  assert.equal(spawned.at(-1).options.timeout, 10_000);
  assert.equal(spawned.at(-1).options.maxBuffer, 256 * 1024);
  assert.equal(process.env.FOCUSA_ATTACHMENT_KEY_V1, undefined);

  const base = { PATH: '/fixture/bin', UNRELATED: 'environment-canary', FOCUSA_ATTACHMENT_KEY_V1: 'stale' };
  const before = structuredClone(base);
  const projected = scope.projectAttachmentEnvironment(keyA, base);
  assert.deepEqual(base, before, 'projection must not mutate parent environment');
  assert.notEqual(projected, base);
  assert.equal(projected.UNRELATED, 'environment-canary');
  assert.deepEqual(JSON.parse(projected.FOCUSA_ATTACHMENT_KEY_V1), keyA);

  for (const ambient of ['malformed-json', JSON.stringify(keyB)]) {
    process.env.FOCUSA_ATTACHMENT_KEY_V1 = ambient;
    const start = spawned.length;
    await Promise.all([runSingle(keyA, 'concurrent a'), runSingle(keyB, 'concurrent b')]);
    const pair = spawned.slice(start);
    assert.deepEqual(pair.map((x) => JSON.parse(x.options.env.FOCUSA_ATTACHMENT_KEY_V1)), [keyA, keyB]);
    assert.ok(pair[0].options.env !== pair[1].options.env, 'concurrent child snapshots must be independent');
    assert.equal(process.env.FOCUSA_ATTACHMENT_KEY_V1, ambient);
  }
  const startMany = spawned.length;
  result = await runMany(keyA);
  assert.equal(result.ok, true);
  assert.equal(spawned.length - startMany, 2);
  for (const row of spawned.slice(startMany)) {
    assert.deepEqual(JSON.parse(row.options.env.FOCUSA_ATTACHMENT_KEY_V1), keyA);
  }

  const invalid = [undefined, null, {},
    { ...keyA, workstream: { ...keyA.workstream, continuity_id: '' } },
    { ...keyA, workstream: { ...keyA.workstream, continuity_id: 'extension-bootstrap' } },
    { ...keyA, session_id: ' ' }, { ...keyA, instance_id: '' }, { ...keyA, attachment_id: '' },
    ...['scope_id', 'fingerprint', 'root_path', 'scope_kind'].map((field) => ({
      ...keyA, workstream: { ...keyA.workstream, root_scope: {
        ...keyA.workstream.root_scope, [field]: field === 'scope_kind' ? 'host' : 'foreign',
      } },
    })),
  ];
  process.env.FOCUSA_ATTACHMENT_KEY_V1 = JSON.stringify(keyB);
  for (const key of invalid) {
    const count = spawned.length;
    result = await runSingle(key, 'invalid context', { env: projected, attachment: keyB });
    assert.equal(result.ok, false, 'model fields and ambient identity must not rescue invalid context');
    assert.equal(result.data.failure_class, 'attachment_scope_missing');
    assert.equal(spawned.length, count);
    result = await runMany(key);
    assert.equal(result.ok, false);
    assert.equal(result.data.failures.length, 2);
    assert.ok(result.data.failures.every((f) => f.failure_class === 'attachment_scope_missing'));
    assert.equal(spawned.length, count, 'invalid multi-dispatch must spawn nothing');
  }
  scope.clearVerifiedScopeRefsForTest();
  const count = spawned.length;
  result = await runSingle(keyA);
  assert.equal(result.data.failure_class, 'attachment_scope_missing', 'stale registry binding cannot launch');
  assert.equal(spawned.length, count);
  registered.forEach(scope.registerVerifiedScopeRef);

  deferChild = true;
  const mutable = keyFor(0, 'snapshot-continuity');
  const pending = runSingle(mutable, 'snapshot');
  const sent = JSON.parse(spawned.at(-1).options.env.FOCUSA_ATTACHMENT_KEY_V1);
  mutable.session_id = 'later-session';
  assert.notEqual(sent.session_id, mutable.session_id);
  callbacks.shift()();
  await pending;
  deferChild = false;

  childFailure = true;
  result = await runSingle(keyA, 'preserved rejection');
  assert.equal(result.ok, false);
  assert.equal(result.data.failure_class, 'background_job_dispatch_failed');
  assert.match(result.data.message, /NORTH_STAR_ADMISSION_BLOCKED/);
  assert.ok(!JSON.stringify(result).includes('environment-canary'));
  assert.ok(!JSON.stringify(result).includes('fixture-instance-'));
  assert.ok(!JSON.stringify(result).includes('FOCUSA_ATTACHMENT_KEY_V1'));
  await assert.rejects(dispatchBackgroundJob({ name: 'missing', command: 'printf none', cwd: '/tmp', env: null }),
    (error) => error instanceof BackgroundJobToolError && error.failureClass === 'attachment_scope_missing');
  console.log('Spec188 production scope projection, registered single/many consumers, concurrent isolation, fail-closed and rejection propagation passed; no real job or installed proof claimed.');
} finally {
  if (saved === undefined) delete process.env.FOCUSA_ATTACHMENT_KEY_V1;
  else process.env.FOCUSA_ATTACHMENT_KEY_V1 = saved;
}
