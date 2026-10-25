import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';

const enabled = process.env.FOCUSA_NATIVE_HLT_RESTORE === '1';
test('actual SDK lifecycle and UI select preserve scoped accepted-history restore boundaries', {
  skip: !enabled, timeout: 90_000,
}, async () => {
  const sdkRoot = process.env.FOCUSA_NATIVE_PI_SDK_ROOT;
  const evidence = process.env.FOCUSA_NATIVE_HLT_EVIDENCE;
  const root = process.env.FOCUSA_LIVE_PROJECT_ROOT;
  const sshConfig = process.env.FOCUSA_LIVE_SDK_SSH_CONFIG;
  const sshAlias = process.env.FOCUSA_LIVE_SDK_SSH_ALIAS;
  assert.ok(sdkRoot && evidence && root && sshConfig && sshAlias, 'explicit SDK, verified project, read relay and evidence paths are required');
  const sandbox = mkdtempSync(join(tmpdir(), 'focusa-native-684-'));
  const home = join(sandbox, 'home');
  mkdirSync(home);
  process.env.HOME = home;
  process.env.XDG_CONFIG_HOME = join(home, '.config');
  process.env.XDG_STATE_HOME = join(home, '.local/state');
  const source = resolve(dirname(fileURLToPath(import.meta.url)), '../src');
  const sdk = resolve(sdkRoot, 'dist/core');
  const report = { schema: 'focusa.native_hlt_restore_test.v1', status: 'started',
    build: false, compiler: false, model_or_agent_worker_dispatch: false,
    network_mutations: 0, backend_or_installed_restoration_verified: false,
    live_logical_scopes_created: 0, canonical_restore_accepted: false, cases: [], requests: [] };
  let scenario, state, runner, session, selections, notices, definitions;
  const accepted = 'Restore only the previously accepted same-scope purpose while preserving customer records. '.repeat(12);
  const acceptedEvent = '00000000-0000-4000-8000-000000000684';
  const forbidden = () => { throw Error('Model, agent and core control actions are prohibited'); };
  globalThis.fetch = async (url, init = {}) => {
    const u = new URL(url instanceof Request ? url.url : String(url));
    const method = String(init.method || (url instanceof Request ? url.method : 'GET')).toUpperCase();
    const path = u.pathname.replace(/^\/v1(?=\/)/, '');
    report.requests.push({ method, path, native_session_id: session?.getSessionId(),
      selected_session_id: state?.currentAttachmentKey()?.session_id,
      continuity: state?.getContinuityId(), runtime_continuity: state?.getAttachmentRuntime()?.continuityId,
      attached_continuity: state?.currentAttachmentKey()?.workstream.continuity_id,
      query_continuity: u.searchParams.get('continuity_id') });
    const response = (body, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
    if (path === '/session/start' && method === 'POST') {
      const request = JSON.parse(init.body);
      return response({ status: 'completed', canonical: false, fixture_only: true, execution_admission: false,
        session: { ...request, id: session.getSessionId(), session_id: session.getSessionId(), status: 'active' } });
    }
    if (path === '/trajectory/define-goal' && method === 'POST') {
      definitions.push(JSON.parse(init.body));
      return response({ status: 'blocked', canonical: false, code: 'NATIVE_FIXTURE_MUTATION_INTERCEPTED', execution_admission: false });
    }
    // An active ambient session is not this SDK-created fixture actor. Do not
    // borrow its state: the production session initializer runs, but its POST
    // remains intercepted and cannot create any daemon record.
    if (path === '/status' && method === 'GET') return response({ status: 'completed', session: { status: 'idle' }, canonical: false });
    const readMethods = new Map([
      ['/health', ['GET']], ['/status', ['GET']], ['/agent/prompt', ['GET']],
      ['/agent/utility-card', ['GET']], ['/project/current', ['GET']],
      ['/project/identity', ['GET', 'POST']], ['/project/verify', ['GET', 'POST']],
      ['/project/bootstrap/status', ['GET', 'POST']], ['/project/genesis/status', ['GET', 'POST']],
      ['/workpoint/resume', ['POST']],
    ]);
    if (['127.0.0.1', 'localhost'].includes(u.hostname) && u.port === '8787' && readMethods.get(path)?.includes(method)) {
      const python = `import sys,json,urllib.request,urllib.error
x=json.load(sys.stdin)
r=urllib.request.Request(x['url'],data=x['body'].encode()if x.get('body')else None,headers=x['headers'],method=x['method'])
try:
 with urllib.request.urlopen(r,timeout=8)as y:print(json.dumps({'status':y.status,'body':y.read().decode(),'headers':dict(y.headers)}))
except urllib.error.HTTPError as y:print(json.dumps({'status':y.code,'body':y.read().decode(),'headers':dict(y.headers)}))
`;
      const result = JSON.parse(execFileSync('ssh', ['-F', sshConfig, '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=5', '-o', 'StrictHostKeyChecking=yes', sshAlias, 'python3 -c ' + "'" + python.replaceAll("'", "'\\''") + "'"], {
        input: JSON.stringify({ url: u.href, method, headers: Object.fromEntries(new Headers(init.headers || (url instanceof Request ? url.headers : undefined))), body: typeof init.body === 'string' ? init.body : '' }),
        encoding: 'utf8', timeout: 15_000,
      }));
      return new Response(result.body, { status: result.status, headers: result.headers });
    }
    if (method !== 'GET') return response({ status: 'blocked', canonical: false, execution_admission: false, code: 'NATIVE_FIXTURE_MUTATION_INTERCEPTED' });
    if (path === '/hlt/history') {
      const continuity = u.searchParams.get('continuity_id');
      assert.equal(u.searchParams.get('project_root'), root);
      if (scenario === 'steering') await runner.emitInput('The operator now chooses a different purpose; do not restore the prior goal.', undefined, 'interactive');
      if (scenario === 'unavailable') return response({ status: 'error', canonical: false }, 503);
      const entry = { event_id: acceptedEvent, project_root: root, continuity_id: continuity, new_hlt: accepted, source: 'operator' };
      const history = { status: 'completed', project_root: root, continuity_id: continuity,
        entries_order: 'newest_first', latest_valid_for_continuity: accepted, entries: [entry] };
      if (scenario === 'empty') history.entries = [];
      if (scenario === 'foreign') entry.project_root = root + '-foreign';
      if (scenario === 'superseded') history.latest_valid_for_continuity = 'A newer accepted purpose supersedes this entry.';
      return response(history);
    }
    if (path === '/trajectory/view') return response({ status: 'not_found', canonical: false, project_root: root,
      continuity_id: u.searchParams.get('continuity_id'), session_id: session.getSessionId(), recommended_action: 'define_goal' });
    return response({ status: 'not_found', canonical: false, execution_admission: false }, 404);
  };
  try {
    const { default: factory } = await import(pathToFileURL(source + '/index.ts').href);
    state = await import(pathToFileURL(source + '/state.ts').href);
    const { createExtensionRuntime, loadExtensionFromFactory } = await import(pathToFileURL(sdk + '/extensions/loader.js').href);
    const { createEventBus } = await import(pathToFileURL(sdk + '/event-bus.js').href);
    const { ExtensionRunner } = await import(pathToFileURL(sdk + '/extensions/runner.js').href);
    const { SessionManager } = await import(pathToFileURL(sdk + '/session-manager.js').href);
      session = SessionManager.inMemory(root);
      const runtime = createExtensionRuntime();
      const observed = [];
      const extension = await loadExtensionFromFactory(async pi => {
        await factory(pi);
        pi.on('input', (_event, ctx) => {
          const active = state.getAttachmentRuntime();
          active.cfg.vitalInfoPromptMode = 'prompt';
          active.cfg.vitalInfoPromptSurfaces = 'trajectory';
          active.vitalInfoPrompted = {};
          observed.push({ native: ctx.sessionManager.getSessionId(), selected: state.currentAttachmentKey()?.session_id });
        });
      }, root, createEventBus(), runtime, 'native-684-ui-consumer');
      runner = new ExtensionRunner([extension], runtime, root, session, undefined);
      const initial = runner.createContext();
      const idle = initial.isIdle(), trusted = initial.isProjectTrusted?.() === true;
      runner.bindCore({ sendMessage: forbidden, sendUserMessage: forbidden,
        appendEntry: (type, data) => session.appendCustomEntry(type, data),
        setSessionName: name => session.appendSessionInfo(name), getSessionName: () => session.getSessionName(),
        setLabel: (id, label) => session.appendLabelChange(id, label),
        getActiveTools: () => [...extension.tools.keys()],
        getAllTools: () => [...extension.tools.values()].map(x => ({ name: x.definition.name, description: x.definition.description })),
        setActiveTools: forbidden, refreshTools: forbidden, getCommands: () => [...extension.commands.values()],
        setModel: forbidden, getThinkingLevel: () => undefined, setThinkingLevel: forbidden,
      }, { getModel: () => undefined, isIdle: () => idle, isProjectTrusted: () => trusted,
        getSignal: () => undefined, abort: forbidden, hasPendingMessages: () => false,
        shutdown: forbidden, getContextUsage: () => undefined, compact: forbidden, getSystemPrompt: () => '' });
      runner.setUIContext({
        ...initial.ui,
        select: async (title, options) => {
          selections.push({ title, options, native_session_id: session.getSessionId(),
            selected_session_id: state.currentAttachmentKey()?.session_id,
            continuity: state.getContinuityId(), runtime_continuity: state.getAttachmentRuntime().continuityId });
          if (!title.includes('trajectory is not set') || scenario === 'cancel') return undefined;
          return options.find(option => option.startsWith(scenario === 'skip' ? 'F)' : 'D)'));
        },
        confirm: async () => false, input: async () => undefined, editor: async () => undefined,
        notify: (message, level) => notices.push({ message, level }),
        setStatus() {}, setWidget() {}, setTitle() {}, setEditorText() {},
        getEditorText: () => '', custom: forbidden,
      }, 'interactive');
    const requested = process.env.FOCUSA_NATIVE_HLT_SCENARIO;
    assert.ok(['accepted', 'unavailable', 'empty', 'skip', 'cancel', 'foreign', 'superseded', 'steering'].includes(requested), 'one explicit isolated scenario is required');
    for (scenario of [requested]) {
      selections = []; notices = []; definitions = []; observed.length = 0;
      await runner.emitInput('Exercise the previous accepted purpose through the native UI; do not alter live goals.', undefined, 'interactive');
      await runner.emit({ type: 'session_start', reason: 'startup' });
      assert.ok(observed.length > 0, 'SDK input context was not observed');
      assert.ok(observed.every(x => x.native === session.getSessionId() && x.selected === x.native), 'native context borrowed another actor');
      const item = { scenario, selections: [...selections], notices: [...notices], definitions: [...definitions], native_session_id: session.getSessionId() };
      report.cases.push(item);
      assert.equal(selections.filter(x => x.title.includes('trajectory is not set')).length, 1, 'NATIVE_UI_NOT_REACHED: lifecycle setup must not count as a product restore failure');
      if (scenario === 'accepted') {
        assert.equal(definitions.length, 1);
        assert.equal(definitions[0].long_term_goal, accepted);
        assert.equal(definitions[0].desired_end_state, accepted);
        assert.equal(definitions[0].goal_source, 'previous_valid_fallback');
        const historyRequest = report.requests.filter(x => x.path === '/hlt/history').at(-1);
        assert.ok(historyRequest.query_continuity);
        assert.equal(historyRequest.continuity, '', 'native fixture must reproduce missing mutable projection continuity');
        assert.equal(historyRequest.query_continuity, historyRequest.attached_continuity);
        assert.equal(definitions[0].continuity_id, historyRequest.query_continuity);
        assert.equal(definitions[0].session_id, session.getSessionId());
        assert.ok(!notices.some(x => x.message.includes('trajectory defined')), 'intercepted rejection is not backend restoration');
        assert.deepEqual(definitions[0].supersession_evidence_refs, [`hlt-history-event:${acceptedEvent}`]);
      } else {
        assert.equal(definitions.length, 0, `${scenario} must not attempt goal adoption`);
        if (scenario === 'unavailable') assert.ok(notices.some(x => x.message.includes('previous goal could not be read')));
        if (scenario === 'empty') assert.ok(notices.some(x => x.message.includes('No previous accepted goal')));
        if (scenario === 'foreign' || scenario === 'superseded') assert.ok(notices.some(x => x.message.includes('does not match')));
      }
    }
    await runner.emit({ type: 'session_shutdown' });
    assert.equal(report.requests.filter(x => /\/(work-loop\/control|workpoint\/checkpoint|project\/genesis\/commit)$/.test(x.path)).length, 0);
    report.status = 'actual_sdk_restore_ui_request_mapping_and_negative_boundaries_verified';
  } catch (error) {
    report.status = 'failed'; report.error = String(error?.stack || error);
    report.last_selection = selections; report.last_notices = notices; report.last_definitions = definitions;
    try { await runner?.emit({ type: 'session_shutdown' }); }
    catch (cleanupError) { report.fixture_shutdown_error = String(cleanupError); }
    throw error;
  } finally {
    writeFileSync(evidence, JSON.stringify(report, null, 2) + '\n');
  }
});
