// Actual native SDK + canonical daemon reads; never starts/resumes a loop or worker.
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const enabled = process.env.FOCUSA_LIVE_LOOP_REPORTING === '1';
test('actual native loop reporting distinguishes unverified lease and route preview', { skip: !enabled }, async () => {
  const root = process.env.FOCUSA_LIVE_PROJECT_ROOT;
  const continuity = process.env.FOCUSA_LIVE_CONTINUITY_ID;
  const sdk = process.env.FOCUSA_NATIVE_PI_SDK_ROOT + '/dist/core';
  const expected = JSON.parse(readFileSync(process.env.FOCUSA_LIVE_LOOP_EXPECTED_SCOPE, 'utf8'));
  const evidence = process.env.FOCUSA_LIVE_LOOP_EVIDENCE;
  assert.ok(root && continuity && evidence && expected.no_execution_grant === true);
  process.env.FOCUSA_PROJECT_ROOT = root;
  process.env.FOCUSA_CONTINUITY_ID = continuity;
  const requests = [];
  let verifiedLoopReads = 0;
  const relay = `import sys,json,urllib.request,urllib.error
x=json.load(sys.stdin)
r=urllib.request.Request(x['url'],data=x['body'].encode()if x.get('body')else None,headers=x['headers'],method=x['method'])
try:
 with urllib.request.urlopen(r,timeout=6)as p:out={'status':p.status,'headers':dict(p.headers),'body':p.read().decode()}
except urllib.error.HTTPError as e:out={'status':e.code,'headers':dict(e.headers),'body':e.read().decode()}
print(json.dumps(out))`;
  const allowed = new Map([
    ['/v1/health', ['GET']], ['/v1/project/identity', ['GET','POST']],
    ['/v1/project/verify', ['GET','POST']], ['/v1/workpoint/resume', ['POST']],
    ['/v1/trajectory/view', ['GET']], ['/v1/work-loop/status', ['GET']],
  ]);
  globalThis.fetch = async (url, init = {}) => {
    const u = new URL(String(url));
    const method = String(init.method || 'GET').toUpperCase();
    assert.ok(['127.0.0.1','localhost'].includes(u.hostname) && u.port === '8787');
    assert.ok(allowed.get(u.pathname)?.includes(method), `Refuse unexpected operation: ${method} ${u.pathname}`);
    const headers = Object.fromEntries(new Headers(init.headers || {}));
    assert.ok(!headers.authorization && !headers.cookie, 'No credential-bearing diagnostic transport');
    requests.push({ method, path: u.pathname });
    const raw = execFileSync('ssh', ['-F', process.env.FOCUSA_LIVE_SDK_SSH_CONFIG,
      '-o','BatchMode=yes','-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes',
      process.env.FOCUSA_LIVE_SDK_SSH_ALIAS, "python3 -c '" + relay.replaceAll("'", "'\\''") + "'"],
      { input: JSON.stringify({ url: u.href, method, headers, body: init.body ? String(init.body) : '' }),
        encoding: 'utf8', timeout: 12000, maxBuffer: 2 * 1024 * 1024 });
    const response = JSON.parse(raw);
    if (u.pathname === '/v1/work-loop/status' && response.status === 200) {
      const body = JSON.parse(response.body);
      assert.deepEqual(body.execution_partition?.project_root_key, expected.project_root_key, 'Actual project selection must match independently read scope');
      assert.deepEqual(body.execution_partition?.workstream_key, expected.workstream_key, 'Actual workstream selection must match independently read scope');
      assert.equal(body.active_writer, null, 'Live test is limited to the already-observed unclaimed loop');
      verifiedLoopReads++;
    }
    return new Response(response.body, { status: response.status, headers: response.headers });
  };
  const { default: factory } = await import(new URL('../src/index.ts', import.meta.url));
  const { createExtensionRuntime, loadExtensionFromFactory } = await import(pathToFileURL(sdk + '/extensions/loader.js').href);
  const { createEventBus } = await import(pathToFileURL(sdk + '/event-bus.js').href);
  const { ExtensionRunner } = await import(pathToFileURL(sdk + '/extensions/runner.js').href);
  const { SessionManager } = await import(pathToFileURL(sdk + '/session-manager.js').href);
  const session = SessionManager.inMemory(root), runtime = createExtensionRuntime();
  const extension = await loadExtensionFromFactory(factory, root, createEventBus(), runtime, 'actual-loop-reporting-consumer');
  const runner = new ExtensionRunner([extension], runtime, root, session, undefined);
  const ctx = runner.createContext();
  const execute = (name, params) => extension.tools.get(name).definition.execute(name, params, undefined, undefined, ctx);
  const resumePacket = await execute('focusa_workpoint_resume', { project_root: root, continuity_id: continuity, mode: 'compact_prompt' });
  assert.equal(resumePacket.details?.canonical, true, 'Normal canonical read recovery must succeed');
  await execute('focusa_project_identity', { cwd: root, project_root: root });
  const writer = await execute('focusa_work_loop_writer_status', {});
  assert.equal(writer.details.writer_observation_verified, true);
  assert.equal(writer.details.active_writer, 'none');
  const preview = await execute('focusa_work_loop_control', { action: 'pause', preflight: true });
  assert.match(preview.content[0].text, /route preview only/);
  assert.equal(preview.details.request_sent, false);
  assert.equal(preview.details.canonical, false);
  assert.equal(preview.details.execution_admission_checked, false);
  const resume = await execute('focusa_work_loop_control', { action: 'resume', renew_budget: false });
  assert.equal(resume.details.ok, false);
  assert.equal(resume.details.status, 'degraded');
  assert.equal(resume.details.failure_class, 'writer_lease_unverified');
  assert.equal(resume.details.request_sent, false);
  assert.equal(resume.details.canonical, false);
  assert.match(resume.content[0].text, /resume request not sent/);
  assert.doesNotMatch(resume.content[0].text, /blocked|owned by another writer/);
  assert.ok(verifiedLoopReads >= 2);
  assert.ok(requests.filter(x => x.path.startsWith('/v1/work-loop/')).every(x => x.method === 'GET'));
  writeFileSync(evidence, JSON.stringify({ status: 'actual_native_loop_reporting_verified',
    scope_verified: true, verified_loop_reads: verifiedLoopReads, requests,
    writer_text: writer.content[0].text, preview_text: preview.content[0].text,
    resume_text: resume.content[0].text, control_request_sent: false,
    budget_renewed: false, model_or_worker_dispatch: false, model_behavior_measured: false,
    installed: false, issue693_closed: false }, null, 2));
});
