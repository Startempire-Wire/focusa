import test from 'node:test';
import assert from 'node:assert/strict';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const enabled = process.env.FOCUSA_LIVE_SESSION_ROUTING === '1';

test('native input fidelity and session switches never borrow the previous actor', {
  skip: !enabled, timeout: 120_000,
}, async () => {
  const required = name => {
    assert.ok(process.env[name], `${name} is required for explicit read-only integration scope`);
    return process.env[name];
  };
  const root = required('FOCUSA_LIVE_PROJECT_ROOT');
  const continuity = required('FOCUSA_LIVE_CONTINUITY_ID');
  const expectedWorkpoint = required('FOCUSA_LIVE_WORKPOINT_ID');
  const sdk = resolve(required('FOCUSA_NATIVE_PI_SDK_ROOT'), 'dist/core');
  const source = resolve(dirname(fileURLToPath(import.meta.url)), '../src');
  const evidence = required('FOCUSA_LIVE_SESSION_ROUTING_EVIDENCE');
  const sshConfig = required('FOCUSA_LIVE_SDK_SSH_CONFIG');
  const sshAlias = required('FOCUSA_LIVE_SDK_SSH_ALIAS');
  const started = Date.now();
  const report = {
    schema: 'focusa.native_session_routing_test.v1', status: 'started',
    model_or_agent_worker_dispatch: false, build: false, compiler: false,
    network_mutations: 0, logical_scopes_created: 0, requests: [], intercepted: [],
  };
  const observed = [];
  const allowed = new Map([
    ['/v1/health', ['GET']], ['/v1/agent/prompt', ['GET']],
    ['/v1/agent/utility-card', ['GET']], ['/v1/project/current', ['GET']],
    ['/v1/project/identity', ['GET', 'POST']], ['/v1/project/verify', ['GET', 'POST']],
    ['/v1/project/bootstrap/status', ['GET', 'POST']],
    ['/v1/project/genesis/status', ['GET', 'POST']],
    ['/v1/workpoint/resume', ['POST']], ['/v1/trajectory/view', ['GET']],
  ]);
  const python = `import sys,json,urllib.request,urllib.error
x=json.load(sys.stdin)
r=urllib.request.Request(x['url'],data=x['body'].encode()if x.get('body')else None,headers=x['headers'],method=x['method'])
try:
 with urllib.request.urlopen(r,timeout=8)as y:print(json.dumps({'status':y.status,'body':y.read().decode(),'headers':dict(y.headers)}))
except urllib.error.HTTPError as y:print(json.dumps({'status':y.code,'body':y.read().decode(),'headers':dict(y.headers)}))
`;
  // This guard remains installed for the child process lifetime: delayed lifecycle
  // telemetry must not escape after the test's assertions have finished.
  globalThis.fetch = async (url, init = {}) => {
    const u = new URL(String(url));
    const method = String(init.method || 'GET').toUpperCase();
    if (!['127.0.0.1', 'localhost'].includes(u.hostname) || u.port !== '8787' ||
        !allowed.get(u.pathname)?.includes(method)) {
      report.intercepted.push({ method, path: u.pathname });
      return new Response(JSON.stringify({ canonical: false, status: 'blocked',
        failure_class: 'validation_rejected', error: 'Read-only test refused operation before transport' }), { status: 409 });
    }
    const headers = Object.fromEntries(new Headers(init.headers || {}));
    assert.ok(!headers.authorization && !headers.cookie, 'Credential transport is outside this test');
    report.requests.push({ method, path: u.pathname,
      ...(u.pathname === '/v1/workpoint/resume' ? { ask: init.body ? JSON.parse(init.body).current_ask : null } : {}) });
    const response = JSON.parse(execFileSync('ssh', ['-F', sshConfig, '-o', 'BatchMode=yes',
      '-o', 'ConnectTimeout=5', '-o', 'StrictHostKeyChecking=yes', sshAlias,
      "python3 -c '" + python.replaceAll("'", "'\\''") + "'"], {
      input: JSON.stringify({ url: u.href, method, headers, body: init.body ? String(init.body) : '' }),
      encoding: 'utf8', timeout: 12_000, maxBuffer: 2 * 1024 * 1024,
    }));
    return new Response(response.body, { status: response.status, headers: response.headers });
  };
  try {
    process.chdir(root);
    const { default: factory } = await import(pathToFileURL(source + '/index.ts').href);
    const state = await import(pathToFileURL(source + '/state.ts').href);
    const binding = await import(pathToFileURL(source + '/workpoint-request-binding.ts').href);
    const { createExtensionRuntime, loadExtensionFromFactory } = await import(pathToFileURL(sdk + '/extensions/loader.js').href);
    const { createEventBus } = await import(pathToFileURL(sdk + '/event-bus.js').href);
    const { ExtensionRunner } = await import(pathToFileURL(sdk + '/extensions/runner.js').href);
    const { SessionManager } = await import(pathToFileURL(sdk + '/session-manager.js').href);
    const session = SessionManager.inMemory(root), runtime = createExtensionRuntime();
    const extension = await loadExtensionFromFactory(async pi => {
      await factory(pi);
      // Observe through the production wrapper; do not manufacture an attachment.
      pi.on('session_start', (_event, ctx) => {
        const key = state.currentAttachmentKey();
        observed.push({ native_session_id: ctx.sessionManager.getSessionId(),
          selected_session_id: key?.session_id,
          ask_source_session_id: state.getAttachmentRuntime().currentAsk?.sessionId });
      });
    }, root, createEventBus(), runtime, 'native-session-routing-consumer');
    const runner = new ExtensionRunner([extension], runtime, root, session, undefined);
    const initial = runner.createContext();
    const idle = initial.isIdle(), trusted = initial.isProjectTrusted?.() === true;
    const prohibited = () => { throw Error('Model, agent and control actions are prohibited'); };
    runner.bindCore({ sendMessage: prohibited, sendUserMessage: prohibited,
      appendEntry: (type, data) => session.appendCustomEntry(type, data),
      setSessionName: name => session.appendSessionInfo(name), getSessionName: () => session.getSessionName(),
      setLabel: (id, label) => session.appendLabelChange(id, label),
      getActiveTools: () => [...extension.tools.keys()],
      getAllTools: () => [...extension.tools.values()].map(x => ({ name: x.definition.name, description: x.definition.description })),
      setActiveTools: prohibited, refreshTools: prohibited, getCommands: () => [...extension.commands.values()],
      setModel: prohibited, getThinkingLevel: () => undefined, setThinkingLevel: prohibited,
    }, { getModel: () => undefined, isIdle: () => idle, isProjectTrusted: () => trusted,
      getSignal: () => undefined, abort: prohibited, hasPendingMessages: () => false,
      shutdown: prohibited, getContextUsage: () => undefined, compact: prohibited, getSystemPrompt: () => '' });
    const execute = (name, params) => extension.tools.get(name).definition.execute(name, params, undefined, undefined, runner.createContext());
    const resumeArgs = { project_root: root, continuity_id: continuity, mode: 'compact_prompt' };
    const before = await execute('focusa_workpoint_resume', resumeArgs);
    assert.equal(before.details?.canonical, true);
    assert.equal(before.details?.resume_packet_v2?.canonical, true);
    assert.equal(before.details.resume_packet_v2.workpoint_id, expectedWorkpoint);
    await execute('focusa_project_identity', { cwd: root, project_root: root });
    await runner.emit({ type: 'session_start', reason: 'startup' });
    await execute('focusa_workpoint_resume', resumeArgs);
    const sid = session.getSessionId();
    assert.equal(observed.at(-1)?.selected_session_id, sid);
    const runtimeState = () => {
      const key = state.attachmentRuntimeRegistry.boundSessionAttachment(sid);
      assert.ok(key && key.workstream.root_scope.root_path === root && key.workstream.continuity_id === continuity);
      return state.getAttachmentRuntime(key);
    };
    const ask = 'Inspect the existing Focusa source and compare the instruction handling paths. ' +
      'Preserve the original request and its evidence while checking the existing implementation. '.repeat(14) +
      'ACTIVITY: discussion only. Do not modify files, build, install, deploy, change destinations, or create scopes.';
    await runner.emitInput(ask, undefined, 'interactive');
    assert.equal(runtimeState().currentAsk.text, ask);
    assert.equal(runtimeState().currentTaskLabel.length, 500);
    let captured = binding.captureResumeRequest(runtimeState(), ask);
    await execute('focusa_workpoint_resume', resumeArgs);
    await execute('focusa_workpoint_resume', { ...resumeArgs, current_ask: ask });
    const forwarded = report.requests.filter(x => x.path === '/v1/workpoint/resume').slice(-2);
    assert.equal(forwarded[0].ask, ask);
    assert.equal(forwarded[1].ask, ask);
    const sourceTurn = runtimeState().currentAsk.sourceTurnId;
    const persistence = await import(pathToFileURL(source + '/persistence.ts').href);
    const nativeAnchor = () => session.getEntries().filter(e => e.type === 'custom' &&
      ['focusa-state', 'focusa-wbm-state'].includes(e.customType)).at(-1);
    const anchored = persistence.loadPersistedRecoveryState(nativeAnchor()?.data);
    assert.equal(anchored?.currentAsk?.text, ask, 'Latest journal anchor must resolve to its exact complete request');
    assert.equal(anchored.currentAsk.textIsPreview, false);
    const entriesBeforeRepeat = session.getEntries().length;
    const actualKey = state.attachmentRuntimeRegistry.boundSessionAttachment(sid);
    state.runWithAttachmentRuntime(actualKey, () => { state.persistState(); state.persistState(); });
    assert.equal(session.getEntries().length, entriesBeforeRepeat, 'Semantic deduplication must suppress unchanged anchors');
    report.exact_anchor_resolves_complete_request = true;
    report.unchanged_snapshots_deduplicated = true;
    // Exercise metadata hooks without selecting, authenticating or running a model.
    await runner.emit({ type: 'model_select', model: {
      id: 'native-lifecycle-metadata-only', provider: 'metadata-fixture', contextWindow: 128000,
    }, source: 'restore' });
    assert.equal(runtimeState().currentAsk.text, ask);
    await execute('focusa_workpoint_resume', resumeArgs);
    assert.equal(report.requests.filter(x => x.path === '/v1/workpoint/resume').at(-1).ask, ask);
    const leaf = session.getLeafId();
    assert.ok(leaf, 'Native journal must have a real entry to retain');
    const compactionId = session.appendCompaction('Metadata-only native storage fixture; no model executed.', leaf, 0);
    assert.ok(session.getEntries().some(e => e.id === compactionId && e.type === 'compaction'));
    await runner.emit({ type: 'session_start', reason: 'startup' });
    assert.equal(session.getSessionId(), sid);
    assert.equal(runtimeState().currentAsk?.text, ask, 'Same-session reload after native compaction must retain tail restrictions');
    assert.equal(runtimeState().currentAsk.sourceTurnId, sourceTurn);
    await execute('focusa_workpoint_resume', resumeArgs);
    assert.equal(report.requests.filter(x => x.path === '/v1/workpoint/resume').at(-1).ask, ask);
    report.model_metadata_and_compaction_reload_verified = true;
    report.model_summarization_executed = false;
    captured = binding.captureResumeRequest(runtimeState(), ask);
    assert.equal(binding.isResumeRequestCurrent(captured, runtimeState()), true);
    await runner.emitInput(ask + ' Compare a different handler instead; earlier replies are stale.', undefined, 'interactive');
    assert.equal(binding.isResumeRequestCurrent(captured, runtimeState()), false);
    report.full_input_and_resume_verified = true;
    report.changed_ask_invalidates_old_binding = true;
    // Both IDs come from the actual SDK, not fixture-provided actor strings.
    const previousRuntime = runtimeState();
    const previousAsk = structuredClone(previousRuntime.currentAsk);
    session.newSession();
    const nextSid = session.getSessionId();
    assert.notEqual(nextSid, sid);
    await runner.emit({ type: 'session_start', reason: 'switch' });
    const switched = observed.at(-1);
    report.session_switch = switched;
    assert.equal(switched.native_session_id, nextSid);
    assert.equal(switched.selected_session_id, nextSid, 'Native session switch must not reuse the prior actor');
    assert.notEqual(switched.selected_session_id, sid);
    assert.ok(!switched.ask_source_session_id || switched.ask_source_session_id === nextSid);
    assert.deepEqual(previousRuntime.currentAsk, previousAsk, 'Starting another session must not clear the prior session request');
    report.previous_session_request_preserved = true;
    report.status = 'native_session_actor_routing_and_input_fidelity_verified';
  } catch (error) {
    report.status = 'failed';
    report.error = String(error?.stack || error);
    throw error;
  } finally {
    report.elapsed_seconds = (Date.now() - started) / 1000;
    writeFileSync(evidence, JSON.stringify(report, null, 2));
  }
});
