import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtemp, mkdir, writeFile, access, readFile, utimes } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const advisory = join(repo, 'scripts/release-diagnostics-advisory.mjs');
const legacy = join(repo, 'scripts/release-readiness-evaluation.mjs');
async function run(script, args = [], env = {}) {
  const child = spawn(process.execPath, [script, '--json', ...args], {
    env: { ...process.env, FOCUSA_PROJECT_ROOT: '', FOCUSA_EXPECTED_VERSION: '', ...env },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let stdout = '', stderr = '';
  child.stdout.on('data', b => { stdout += b; });
  child.stderr.on('data', b => { stderr += b; });
  const status = await new Promise((done, reject) => { child.once('error', reject); child.once('close', done); });
  const result = JSON.parse(stdout);
  assert.equal(result.ready, false);
  assert.equal(result.release_ready, false);
  assert.equal(result.advisory, true);
  assert.equal(result.authority, 'advisory_only');
  assert.equal(result.execution_admission, false);
  assert.equal(result.scope.canonical, false);
  assert.equal(stderr, '', 'Valid and invalid invocations must emit typed diagnostics, not uncaught exceptions');
  return { status, result };
}
const check = (result, name) => result.checks.find(c => c.name === name);

test('clean host directory with spaces is typed; ambient suites are never executed', async () => {
  const root = await mkdtemp(join(tmpdir(), 'focusa358 clean host '));
  await mkdir(join(root, 'scripts'));
  await mkdir(join(root, 'tests'));
  const marker = join(root, 'unexpected-execution');
  await writeFile(join(root, 'scripts/audit-distribution-parity.mjs'), `import{writeFileSync}from'node:fs';writeFileSync(${JSON.stringify(marker)},'ran');`);
  await writeFile(join(root, 'tests/convergence_invariants_static_test.sh'), `printf ran > '${marker}'\n`);
  const { status, result } = await run(advisory, ['--project-root', root]);
  assert.equal(status, 0);
  assert.equal(check(result, 'repository_scope').status, 'unavailable');
  assert.equal(check(result, 'disk_observation').status, 'observed');
  assert.equal(check(result, 'daemon_health').status, 'unavailable');
  assert.equal(check(result, 'release_acceptance').status, 'unavailable');
  await assert.rejects(access(marker));
  const noGit = await run(advisory, ['--project-root', root], { PATH: '' });
  assert.equal(noGit.status, 0);
  assert.equal(check(noGit.result, 'repository_scope').status, 'unavailable');
});

test('missing project and disk paths produce a declared envelope, not df exceptions', async () => {
  const root = await mkdtemp(join(tmpdir(), 'focusa358 missing '));
  const { status, result } = await run(legacy, ['--project-root', join(root, 'absent'), '--disk-path', join(root, 'absent-disk')]);
  assert.equal(status, 0);
  assert.equal(result.schema, 'focusa.release_readiness_evaluation.v1');
  assert.equal(check(result, 'repository_scope').status, 'unavailable');
  assert.equal(check(result, 'disk_observation').status, 'unavailable');
  assert.equal(result.exit_semantics, 'diagnostic_completion_not_release_acceptance');
});

test('healthy, failed, malformed and wrong-version HTTP observations cannot grant readiness', async t => {
  const server = createServer((req, res) => {
    if (req.url === '/failed') { res.writeHead(503); res.end('unavailable'); return; }
    if (req.url === '/malformed') { res.end('{'); return; }
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ ok: true, version: '0.0.0-fixture', ready: true, gate_chain_green: true }));
  });
  await new Promise(done => server.listen(0, '127.0.0.1', done));
  t.after(() => new Promise(done => server.close(done)));
  const endpoint = `http://127.0.0.1:${server.address().port}`;
  const args = ['--project-root', repo, '--expected-version', '0.0.0-fixture'];
  const healthy = await run(advisory, [...args, '--daemon-url', endpoint + '/healthy']);
  assert.equal(healthy.status, 0);
  for (const name of ['repository_scope', 'disk_observation', 'daemon_health', 'runtime_version']) {
    assert.equal(check(healthy.result, name).status, 'observed');
  }
  assert.equal(check(healthy.result, 'release_acceptance').status, 'unavailable');
  const failed = await run(advisory, [...args, '--daemon-url', endpoint + '/failed']);
  assert.equal(check(failed.result, 'daemon_health').http_status, 503);
  const absent = await run(advisory, [...args, '--daemon-url', 'http://127.0.0.1:0/health']);
  assert.equal(check(absent.result, 'daemon_health').status, 'unavailable');
  const malformed = await run(advisory, [...args, '--daemon-url', endpoint + '/malformed']);
  assert.equal(check(malformed.result, 'daemon_health').error_code, 'INVALID_HEALTH_JSON');
  const wrong = await run(legacy, ['--project-root', repo, '--daemon-url', endpoint + '/healthy', '--expected-version', 'old-version']);
  assert.equal(check(wrong.result, 'runtime_version').status, 'error');
});

test('stale or foreign log inputs are rejected; legacy output always remains ready:false', async () => {
  const root = await mkdtemp(join(tmpdir(), 'focusa358 stale '));
  const stale = join(root, 'focusa-consolidation-gates6.log');
  await writeFile(stale, 'foreign-commit old-run GATES-GREEN\n');
  await utimes(stale, new Date('2000-01-01'), new Date('2000-01-01'));
  const ignored = await run(legacy, ['--project-root', root], { TMPDIR: root });
  assert.equal(ignored.status, 0);
  assert.equal(ignored.result.evidence_policy, 'no_gate_or_extension_evidence_evaluated');
  for (const script of [legacy, advisory]) {
    const rejected = await run(script, ['--evidence', stale]);
    assert.equal(rejected.status, 2);
    assert.equal(check(rejected.result, 'diagnostic_invocation').status, 'error');
  }
});

test('invalid endpoint is typed and informational CI does not suppress failures', async () => {
  const { status, result } = await run(advisory, ['--daemon-url', 'not-a-url']);
  assert.equal(status, 0);
  assert.equal(check(result, 'daemon_health').error_code, 'INVALID_ENDPOINT');
  const ci = await readFile(join(repo, '.github/workflows/ci.yml'), 'utf8');
  const step = ci.slice(ci.indexOf('      - name: Release diagnostics (#358'), ci.indexOf('\n\n', ci.indexOf('      - name: Release diagnostics (#358')));
  assert.ok(step.includes('advisory only; never release acceptance'));
  assert.ok(step.includes('node --test tests/358-release-diagnostics-advisory.test.mjs'));
  assert.ok(step.includes('node scripts/release-diagnostics-advisory.mjs --advisory --json'));
  assert.ok(!step.includes('|| true') && !step.includes('continue-on-error'));
});
