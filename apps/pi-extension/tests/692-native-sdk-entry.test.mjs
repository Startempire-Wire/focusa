import test from 'node:test';
import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import { existsSync, realpathSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';

const sourceRoot = fileURLToPath(new URL('../src/', import.meta.url));
const sdkRoot = process.env.FOCUSA_NATIVE_PI_SDK_ROOT;
let registeredNativeEntry;

// Resolve the real source modules with native type erasure, never a SDK facade.
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (/^(typescript|esbuild|tsx|ts-node)(\/|$)/.test(specifier)) {
      throw new Error('Compiler/transpiler packages are prohibited in this native SDK proof');
    }
    try {
      return nextResolve(specifier, context);
    } catch (error) {
      if (specifier.endsWith('.js') && context.parentURL?.startsWith('file:')) {
        const candidate = new URL(specifier.slice(0, -3) + '.ts', context.parentURL);
        const path = fileURLToPath(candidate);
        if (path.startsWith(sourceRoot) && existsSync(path)) {
          return nextResolve(candidate.href, context);
        }
      }
      throw error;
    }
  },
});

test('full Focusa entry registers through the actual native Pi SDK without compilation', {
  skip: !sdkRoot && 'Requires an explicitly selected existing Pi SDK; never installs one',
}, async () => {
  const sdk = realpathSync(sdkRoot);
  const originalFetch = globalThis.fetch;
  let networkRequests = 0;
  globalThis.fetch = async () => {
    networkRequests += 1;
    throw new Error('Registration cannot initialize scope, contact a provider, or call a daemon');
  };
  try {
    const { default: factory } = await import(new URL('../src/index.ts', import.meta.url));
    const { FOCUSA_TOOL_CONTRACTS } = await import(new URL('../src/tool-contracts.ts', import.meta.url));
    const { createExtensionRuntime, loadExtensionFromFactory } = await import(
      pathToFileURL(`${sdk}/dist/core/extensions/loader.js`).href
    );
    const { createEventBus } = await import(pathToFileURL(`${sdk}/dist/core/event-bus.js`).href);
    const runtime = createExtensionRuntime();
    const extension = await loadExtensionFromFactory(
      factory, process.cwd(), createEventBus(), runtime, 'actual-focusa-entry'
    );
    registeredNativeEntry = { extension, runtime };
    assert.deepEqual(
      [...extension.tools.keys()].sort(),
      FOCUSA_TOOL_CONTRACTS.map(contract => contract.name).sort()
    );
    assert.equal(networkRequests, 0);
    assert.equal(typeof extension.tools.get('focusa_north_star_gate').definition.execute, 'function');
    // Registration integration only: this does not establish a live attachment,
    // execution authority, model efficacy, or completion of issue692.
  } finally {
    globalThis.fetch = originalFetch;
  }
});

// Opt-in real-service consumer proof. This never creates a project/workstream,
// invokes a model, dispatches work, installs a package, or treats READY as a grant.
test('fresh native SDK resume and readiness agree with the real daemon', {
  skip: process.env.FOCUSA_LIVE_SDK_CONSUMER !== '1',
}, async () => {
  const { execFileSync } = await import('node:child_process');
  const { writeFileSync } = await import('node:fs');
  const root = process.env.FOCUSA_LIVE_PROJECT_ROOT;
  const continuity = process.env.FOCUSA_LIVE_CONTINUITY_ID;
  const evidence = process.env.FOCUSA_LIVE_SDK_EVIDENCE_DIR;
  const sshConfig = process.env.FOCUSA_LIVE_SDK_SSH_CONFIG;
  const sshAlias = process.env.FOCUSA_LIVE_SDK_SSH_ALIAS;
  assert(root?.startsWith('/') && !['/', '/root', '/tmp'].includes(root));
  assert(continuity && evidence && sshConfig && sshAlias && sdkRoot);
  const sdk = realpathSync(sdkRoot);
  const { ExtensionRunner } = await import(pathToFileURL(`${sdk}/dist/core/extensions/runner.js`).href);
  const { SessionManager } = await import(pathToFileURL(`${sdk}/dist/core/session-manager.js`).href);
  const requests = [];
  const allowed = new Map([
    ['/v1/health', ['GET']],
    ['/v1/project/identity', ['GET', 'POST']],
    ['/v1/project/verify', ['GET', 'POST']],
    ['/v1/project/north-star-gate', ['GET']],
    ['/v1/trajectory/view', ['GET']],
    ['/v1/workpoint/resume', ['POST']], // Existing canonical read operation.
  ]);
  const python = `import sys,json,urllib.request,urllib.error
x=json.load(sys.stdin)
r=urllib.request.Request(x['url'],data=x['body'].encode()if x.get('body')else None,headers=x['headers'],method=x['method'])
try:
 with urllib.request.urlopen(r,timeout=6)as p:out={'status':p.status,'headers':dict(p.headers),'body':p.read().decode()}
except urllib.error.HTTPError as e:out={'status':e.code,'headers':dict(e.headers),'body':e.read().decode()}
print(json.dumps(out))
`;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (url, init = {}) => {
    const target = new URL(String(url));
    const method = String(init.method || 'GET').toUpperCase();
    assert(['127.0.0.1', 'localhost'].includes(target.hostname) && target.port === '8787');
    assert(allowed.get(target.pathname)?.includes(method), `Non-read operation prohibited: ${method}`);
    const headers = Object.fromEntries(new Headers(init.headers || {}));
    assert(!headers.authorization && !headers.cookie, 'Credential-bearing requests prohibited');
    requests.push({ path: target.pathname, method, query: Object.fromEntries(target.searchParams) });
    const response = JSON.parse(execFileSync('ssh', [
      '-F', sshConfig, '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=5',
      '-o', 'StrictHostKeyChecking=yes', sshAlias,
      "python3 -c '" + python.replaceAll("'", "'\\''") + "'",
    ], {
      input: JSON.stringify({ url: target.href, method, headers, body: init.body ? String(init.body) : '' }),
      encoding: 'utf8', timeout: 12000, maxBuffer: 2 * 1024 * 1024,
    }));
    return new Response(response.body, { status: response.status, headers: response.headers });
  };
  try {
    // One real package activation per discovery root: preserve the duplicate guard.
    assert(registeredNativeEntry, 'The actual native entry must register successfully first');
    const { extension, runtime } = registeredNativeEntry;
    const session = SessionManager.inMemory(root);
    const runner = new ExtensionRunner([extension], runtime, root, session, undefined);
    const ctx = runner.createContext();
    const execute = (name, params) => extension.tools.get(name).definition.execute(name, params, undefined, undefined, ctx);
    const text = result => (result.content || []).filter(c => c.type === 'text').map(c => c.text).join('\n');
    const unbound = await execute('focusa_north_star_gate', { trigger: 'native_sdk_unbound_negative', projection: 'full' });
    assert(text(unbound).includes('NORTH STAR BLOCKED'));
    assert(!text(unbound).includes('NORTH STAR READY'));
    // Only the actual registered resume owner restores existing read context.
    const resume = await execute('focusa_workpoint_resume', { project_root: root, continuity_id: continuity, mode: 'compact_prompt' });
    writeFileSync(`${evidence}/native-resume-result.json`, JSON.stringify(resume, null, 2) + '\n');
    const identity = await execute('focusa_project_identity', { cwd: root, project_root: root });
    writeFileSync(`${evidence}/native-identity-result.json`, JSON.stringify(identity, null, 2) + '\n');
    const trigger = 'native_sdk_full_read_consumer';
    const result = await execute('focusa_north_star_gate', { trigger, projection: 'full' });
    writeFileSync(`${evidence}/native-gate-result.json`, JSON.stringify(result, null, 2) + '\n');
    assert(text(result).includes('NORTH STAR READY'));
    assert.equal(result.details.canonical, true);
    const call = requests.find(r => r.path === '/v1/project/north-star-gate' && r.query.trigger === trigger);
    assert(call, 'Native callback discarded the explicit trigger');
    assert.equal(call.query.projection, 'full');
    assert.equal(call.query.project_root, root);
    assert.equal(call.query.continuity_id, continuity);
    writeFileSync(`${evidence}/native-requests.json`, JSON.stringify(requests, null, 2) + '\n');
    // A readiness read is not mutation/dispatch admission or a model-efficacy result.
  } finally {
    globalThis.fetch = originalFetch;
  }
});
