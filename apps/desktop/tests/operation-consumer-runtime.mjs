import assert from 'node:assert/strict';
import { access, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) args.set(process.argv[index], process.argv[index + 1]);
const operationId = args.get('--operation');
const clientMethod = args.get('--client');
assert.ok(operationId, '--operation is required');
assert.ok(clientMethod, '--client is required');

const desktopRoot = fileURLToPath(new URL('..', import.meta.url));
const repositoryRoot = fileURLToPath(new URL('../../..', import.meta.url));
const generatedKitRoot = fileURLToPath(new URL('../.svelte-kit/', import.meta.url));
const generatedKitTsconfig = fileURLToPath(new URL('../.svelte-kit/tsconfig.json', import.meta.url));
let createdKitTsconfig = false;
try {
  await access(generatedKitTsconfig);
} catch {
  await mkdir(generatedKitRoot, { recursive: true });
  await writeFile(generatedKitTsconfig, JSON.stringify({ compilerOptions: { target: 'ES2021', module: 'ESNext', moduleResolution: 'Bundler', allowJs: true } }));
  createdKitTsconfig = true;
}
process.chdir(desktopRoot);
const server = await createServer({
  root: desktopRoot,
  configFile: false,
  appType: 'custom',
  server: { middlewareMode: true, fs: { allow: [repositoryRoot] } },
  optimizeDeps: { disabled: true },
  oxc: false,
  esbuild: { tsconfigRaw: { compilerOptions: { target: 'ES2021', module: 'ESNext' } } },
  logLevel: 'error'
});

try {
  const generatedClientPath = fileURLToPath(new URL('../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated.ts', import.meta.url));
  const { MissionCanvasClient } = await server.ssrLoadModule(generatedClientPath);
  const { MissionCanvasHttpTransport, MissionCanvasTransportError } = await server.ssrLoadModule('/src/lib/mission-canvas/http-transport.ts');
  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  const authority = {
    workstream: structuredClone(fixture.workstream),
    continuity_id: fixture.continuity_id ?? null,
    attachment: structuredClone(fixture.attachment ?? null),
    workspace_binding_id: fixture.workspace_binding_id ?? null,
    runtime_object: structuredClone(fixture.runtime_object ?? null),
    work_surface_id: fixture.work_surface_id ?? fixture.focused_work_surface_id ?? null
  };

  if (operationId === 'focusa.mission_canvas.rich_host.resolve') {
    await exerciseHostResolution({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.domain_pack.install') {
    await exerciseDomainPackInstall({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else {
    throw new Error(`No bounded Desktop consumer fixture for ${operationId} (${clientMethod})`);
  }
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
  if (createdKitTsconfig) await rm(generatedKitTsconfig, { force: true });
}

async function exerciseHostResolution({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const resolution = {
    ...structuredClone(authority),
    interaction_mode: 'canvas-guided',
    selected_renderer: 'focusa_desktop_tauri',
    platform: 'macOS',
    availability: 'available',
    resolution_reason: 'Focusa Desktop Tauri is the primary Mission Canvas host; Pi overlay is compatibility-only',
    asset_version: null,
    asset_digest: null,
    resolver_revision: 'host-resolver:v2',
    diagnostic_ref: null
  };
  let calls = 0;
  let lastRequest;
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      lastRequest = { url: String(url), init };
      return new Response(JSON.stringify(resolution), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const resolved = await client.rich_hostResolve(structuredClone(authority));
  assert.deepEqual(resolved, resolution);
  assert.equal(calls, 1);
  const requestUrl = new URL(lastRequest.url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/rich-host/resolution');
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.equal(lastRequest.init.method, 'GET');
  assert.equal(lastRequest.init.body, undefined);
  assert.equal(lastRequest.init.headers['X-Focusa-Permissions'], 'mission_canvas:host');
  assert.equal(lastRequest.init.headers['X-Focusa-Capabilities'], 'mission_canvas.desktop_tauri');
  assert.equal(lastRequest.init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(lastRequest.init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(resolution);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const foreignClient = new MissionCanvasClient(foreignTransport);
  await assert.rejects(
    () => foreignClient.rich_hostResolve(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_resolution_scope'
  );

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(resolution), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const missingScopeClient = new MissionCanvasClient(missingScopeTransport);
  await assert.rejects(
    () => missingScopeClient.rich_hostResolve({}),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream must fail before HTTP');

  const invalidResponseTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify({ ...resolution, selected_renderer: 'unknown_renderer' }), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const invalidResponseClient = new MissionCanvasClient(invalidResponseTransport);
  await assert.rejects(
    () => invalidResponseClient.rich_hostResolve(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const { workstream: _omittedWorkstream, ...missingResponseScope } = resolution;
  const missingResponseTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify(missingResponseScope), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const missingResponseClient = new MissionCanvasClient(missingResponseTransport);
  await assert.rejects(
    () => missingResponseClient.rich_hostResolve(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:workstream'
  );

  console.log('Mission Canvas operation consumer: PASS (generated client, GET registry path, Desktop host resolution, exact scope, and hostile response checks)');
}

async function exerciseDomainPackInstall({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const pack = {
    pack_id: 'domain.healthcare',
    version: '1.0.0',
    profile: { profile_id: 'domain.healthcare.clinical' },
    activities: [{ activity_mode_id: 'domain.healthcare.review' }],
    registry_entries: []
  };
  const receipt = {
    schema: 'focusa.mission_canvas.domain_pack_install_receipt.v1',
    workstream: structuredClone(authority.workstream),
    installed: true,
    pack_id: pack.pack_id,
    receipt_ref: 'receipt:domain-pack:domain.healthcare:fixture'
  };

  let calls = 0;
  let lastRequest;
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      lastRequest = { url: String(url), init };
      return new Response(JSON.stringify(receipt), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['mission_canvas'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = {
    ...structuredClone(authority),
    pack,
    idempotency_key: 'idempotency:domain-pack:fixture',
    confirmation: 'confirm'
  };
  const installed = await client.domain_packInstall(input);
  assert.deepEqual(installed, receipt);
  assert.equal(calls, 1);
  assert.match(lastRequest.url, /^http:\/\/127\.0\.0\.1:8787\/v1\/mission-canvas\/domain-packs\/install$/);
  assert.equal(lastRequest.init.method, 'POST');
  assert.equal(lastRequest.init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(lastRequest.init.headers['X-Focusa-Capabilities'], 'mission_canvas');
  assert.equal(lastRequest.init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(lastRequest.init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  const body = JSON.parse(lastRequest.init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.confirmation, 'confirm');
  assert.equal(body.idempotency_key, input.idempotency_key);
  assert.deepEqual(body.pack, pack);

  await assert.rejects(
    () => client.domain_packInstall({ ...structuredClone(authority), pack, idempotency_key: 'idempotency:no-confirm' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'explicit_confirmation_required'
  );
  assert.equal(calls, 1, 'missing confirmation must not reach HTTP');

  const foreignTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(receipt);
    foreign.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  });
  const foreignClient = new MissionCanvasClient(foreignTransport);
  await assert.rejects(
    () => foreignClient.domain_packInstall(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_receipt_scope'
  );

  console.log('Mission Canvas operation consumer: PASS (generated client, transport, confirmation, receipt and scope parity)');
}
