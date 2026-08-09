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

  if (operationId === 'focusa.mission_canvas.projection.get') {
    await exerciseProjectionGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.profile.list') {
    await exerciseProfileList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.profile.get') {
    await exerciseProfileGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.activity.list') {
    await exerciseActivityList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.registry.list') {
    await exerciseRegistryList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.activity.select') {
    await exerciseActivitySelect({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority, fixture });
  } else if (operationId === 'focusa.mission_canvas.layout_memory.get') {
    await exerciseLayoutMemoryGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.layout_memory.update') {
    await exerciseLayoutMemoryUpdate({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.layout.mutate') {
    await exerciseLayoutMutation({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.profile.select') {
    await exerciseProfileSelect({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority, fixture });
  } else if (operationId === 'focusa.mission_canvas.projection.resolve') {
    await exerciseProjectionResolve({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.rich_host.resolve') {
    await exerciseHostResolution({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.rich_host.launch') {
    await exerciseHostLaunch({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.rich_host.focus') {
    await exerciseHostFocus({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.rich_host.hide') {
    await exerciseHostHide({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.rich_host.close') {
    await exerciseHostClose({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.domain_pack.install') {
    await exerciseDomainPackInstall({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.draft.get') {
    await exerciseDraftGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.draft.sync') {
    await exerciseDraftSync({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.recomposition.evidence.get') {
    await exerciseRecompositionEvidenceGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.recipient.resolve') {
    await exerciseRecipientResolve({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.recomposition.receipt.get') {
    await exerciseRecompositionReceiptGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.recomposition.diagnostics.list') {
    await exerciseRecompositionDiagnosticsList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority });
  } else if (operationId === 'focusa.mission_canvas.events.stream') {
    await exerciseEventsStream({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority, server });
  } else {
    throw new Error(`No bounded Desktop consumer fixture for ${operationId} (${clientMethod})`);
  }
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
  if (createdKitTsconfig) await rm(generatedKitTsconfig, { force: true });
}

async function exerciseLayoutMutation({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  assert.ok(authority.attachment, 'layout mutation fixture requires exact attachment authority');
  const input = {
    ...structuredClone(authority),
    action: 'focus',
    attachment: structuredClone(authority.attachment),
    command_id: 'command:layout:focus',
    expected_layout_revision: 7,
    expected_projection_revision: 11,
    idempotency_key: 'idem:layout:focus',
    target_contribution_id: 'contribution:primary'
  };
  const canonical = {
    accepted: true,
    attachment: structuredClone(authority.attachment),
    command_id: input.command_id,
    continuity_id: authority.continuity_id,
    error_ref: null,
    event_cursor: 'event:layout:12',
    evidence_ref: 'evidence:layout:12',
    layout_revision: 8,
    projection_digest: `sha256:${'a'.repeat(64)}`,
    projection_revision: 12,
    receipt_ref: 'receipt:layout:12',
    runtime_object: structuredClone(authority.runtime_object),
    work_surface_id: authority.work_surface_id,
    workspace_binding_id: authority.workspace_binding_id,
    workstream: structuredClone(authority.workstream)
  };
  let responseBody = structuredClone(canonical);
  const transport = new MissionCanvasHttpTransport('http://focusa.test', async () => new Response(
    JSON.stringify(responseBody),
    { status: 200, headers: { 'content-type': 'application/json' } }
  ));
  const client = new MissionCanvasClient(transport);
  const result = await client.layoutMutate(structuredClone(input));
  assert.equal(result.command_id, input.command_id);
  assert.equal(result.layout_revision, 8);
  assert.equal(result.projection_revision, 12);

  responseBody = structuredClone(canonical);
  responseBody.workstream.workstream_id = 'ws:foreign';
  await assert.rejects(
    () => client.layoutMutate({ ...structuredClone(input), command_id: 'command:layout:foreign' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.includes('scope_mismatch')
  );

  responseBody = { ...structuredClone(canonical), projection_revision: 11, layout_revision: 7 };
  await assert.rejects(
    () => client.layoutMutate({ ...structuredClone(input), command_id: 'command:layout:stale' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.includes('stale_layout_mutation')
  );
}

async function exerciseProfileSelectCoreDirectHostile({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  // profile.select returns the direct Core-owned ResolvedWorkspaceProjection;
  // the receipt is carried by its exact Workstream-scoped receipt_refs.
  let calls = 0;
  const requests = [];
  let response = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  response.workspace_profile_id = 'research';
  response.workspace_profile_revision = 3;
  response.projection_revision = 13;
  response.layout_revision = 6;
  response.durable_event_cursor = 'event:42';
  response.evidence_refs = ['recomposition-evidence:profile-select'];
  response.receipt_refs = ['recomposition-receipt:profile-select'];

  const input = {
    ...structuredClone(authority),
    selection_id: 'research',
    expected_projection_revision: 12,
    idempotency_key: 'idempotency:profile-select'
  };
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const selected = await client.profileSelect(structuredClone(input));
  assert.deepEqual(selected, response);
  assert.deepEqual(selected.workstream, authority.workstream);
  assert.deepEqual(selected.attachment, authority.attachment);
  assert.equal('projection' in selected, false, 'profileSelect must not adopt a route wrapper');
  assert.deepEqual(selected.receipt_refs, ['recomposition-receipt:profile-select']);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/profiles/select');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'profile.select has no invented operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  assert.equal(requests[0].init.headers['If-Match'], '12');
  assert.equal(requests[0].init.headers['Idempotency-Key'], input.idempotency_key);
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.selection_id, input.selection_id);
  assert.equal(body.expected_projection_revision, input.expected_projection_revision);
  assert.equal(body.idempotency_key, input.idempotency_key);
  assert.equal('layout_tree' in body, false, 'Desktop must not compose the selected profile locally');

  // Core-owned omission remains truthful: an empty contribution has a
  // diagnostic and never appears in the returned layout.
  assert.equal(response.eligible_contributions.some(({ contribution_id }) => contribution_id === 'contribution:empty-work-rail'), false);
  assert.equal(response.omission_diagnostics.some(({ contribution_id, reason }) => contribution_id === 'contribution:empty-work-rail' && reason === 'no_relevant_content'), true);
  assert.equal(JSON.stringify(response.layout_tree).includes('contribution:empty-work-rail'), false);

  let missingSelectionCalls = 0;
  const missingSelectionTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingSelectionCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingSelectionTransport).profileSelect({
      ...structuredClone(authority),
      expected_projection_revision: 12,
      idempotency_key: 'idempotency:missing-selection'
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:missing:selection_id'
  );
  assert.equal(missingSelectionCalls, 0, 'missing selected profile must fail before HTTP');

  let missingIdempotencyCalls = calls;
  const missingIdempotency = { ...structuredClone(input) };
  delete missingIdempotency.idempotency_key;
  await assert.rejects(
    () => client.profileSelect(missingIdempotency),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing idempotency must fail before HTTP');

  const missingRevision = { ...structuredClone(input) };
  delete missingRevision.expected_projection_revision;
  await assert.rejects(
    () => client.profileSelect(missingRevision),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'if_match_revision_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing If-Match must fail before HTTP');

  const foreignTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(response);
    foreign.workstream.workstream_id = 'ws:foreign';
    foreign.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], [], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_projection_scope'
  );

  const foreignContributionTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreignContribution = structuredClone(response);
    foreignContribution.eligible_contributions[0].authority.workstream.workstream_id = 'ws:foreign';
    foreignContribution.eligible_contributions[0].authority.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreignContribution), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], [], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignContributionTransport).profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_contribution_scope'
  );

  const wrapperTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ projection: response, receipt: { workstream: authority.workstream } }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrapperTransport).profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  response.projection_revision = 12;
  response.durable_event_cursor = 'event:41';
  await assert.rejects(
    () => client.profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_revision'
  );
  response.projection_revision = 14;
  response.layout_revision = 5;
  response.durable_event_cursor = 'event:43';
  await assert.rejects(
    () => client.profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_layout_revision'
  );
  response.layout_revision = 6;
  response.durable_event_cursor = 'event:41';
  await assert.rejects(
    () => client.profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_cursor'
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).profileSelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.unavailable', structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated profileSelect, exact Workstream POST, Core-owned direct projection and receipt, omission, foreign authority, missing selection/If-Match/idempotency, stale revision/layout/cursor, permission, and hostile response checks)');
}

async function exerciseProfileGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const profile = {
    profile_id: 'research',
    revision: 4,
    display_name: 'Research',
    candidate_contribution_ids: ['contribution:research', 'contribution:evidence'],
    density: 'compact',
    terminology_registry_ref: 'registry:terminology:research',
    renderer_registry_ref: 'registry:renderer:builtin',
    domain_semantic_binding_registry_ref: 'registry:semantics:research',
    viability_rule_revision: 'profile-viability:v2',
    installed: true
  };
  let response = structuredClone(profile);
  let calls = 0;
  const requests = [];
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = { ...structuredClone(authority), profile_id: 'research' };
  const fetched = await client.profileGet(structuredClone(input));
  assert.deepEqual(fetched, profile);
  assert.equal(fetched.profile_id, input.profile_id);
  assert.equal('profile' in fetched, false, 'profileGet must return the direct generated WorkspaceProfile DTO');
  assert.equal('workstream' in fetched, false, 'profile DTO must not invent response authority fields');
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/profiles/research');
  assert.equal(requestUrl.searchParams.has('profile_id'), false, 'path identity must not leak into the query');
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('attachment')), authority.attachment);
  assert.equal(requestUrl.searchParams.get('continuity_id'), authority.continuity_id);
  assert.equal(requestUrl.searchParams.get('workspace_binding_id'), authority.workspace_binding_id);
  assert.equal(requestUrl.searchParams.get('runtime_object'), JSON.stringify(authority.runtime_object));
  assert.equal(requestUrl.searchParams.get('work_surface_id'), authority.work_surface_id);
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'profile.get must not mint an operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  let missingParameterCalls = 0;
  const missingParameterTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingParameterCalls += 1;
      return new Response(JSON.stringify(profile), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingParameterTransport).profileGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:missing:profile_id'
  );
  assert.equal(missingParameterCalls, 0, 'missing profile path identity must fail before HTTP');
  await assert.rejects(
    () => new MissionCanvasClient(missingParameterTransport).profileGet({ ...structuredClone(authority), profile_id: {} }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:missing:profile_id'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingParameterTransport).profileGet({ ...structuredClone(authority), profile_id: '   ' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:missing:profile_id'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingParameterTransport).profileGet({ ...structuredClone(input), layout_tree: { kind: 'single' } }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:unknown:layout_tree'
  );
  assert.equal(missingParameterCalls, 0, 'client-local layout input must fail before profile GET');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(profile), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).profileGet({ profile_id: 'research' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before profile GET');

  response = { ...structuredClone(profile), display_name: undefined };
  delete response.display_name;
  await assert.rejects(
    () => client.profileGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:display_name'
  );

  const wrappedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ profile }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrappedTransport).profileGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const foreignScopeTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ ...profile, workstream: authority.workstream }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignScopeTransport).profileGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.includes('unknown:workstream')
  );

  const mismatchedProfileTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ ...profile, profile_id: 'other' }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(mismatchedProfileTransport).profileGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:profile_id_mismatch'
  );

  const notFoundTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({
      schema: 'focusa.tool_result.v1',
      status: 'blocked',
      error: { code: 'profile_not_found', message: 'The requested workspace profile is not in the canonical registry' }
    }), { status: 404 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(notFoundTransport).profileGet({ ...structuredClone(authority), profile_id: 'missing' }),
    (error) => error instanceof MissionCanvasTransportError
      && error.message === 'transport_response_failed'
      && error.status === 404
      && error.response?.error?.code === 'profile_not_found'
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).profileGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).profileGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.unavailable', structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated profileGet, exact Workstream GET/path metadata, direct WorkspaceProfile, typed not-found, missing path/authority, permission, mismatched profile, malformed/wrapped/foreign response, and unavailable operation checks)');
}

async function exerciseProfileList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const profile = (profileId, displayName, contributionIds) => ({
    profile_id: profileId,
    revision: 1,
    display_name: displayName,
    candidate_contribution_ids: contributionIds,
    density: 'standard',
    terminology_registry_ref: `registry:terminology:${profileId}`,
    renderer_registry_ref: 'registry:renderer:builtin',
    domain_semantic_binding_registry_ref: null,
    viability_rule_revision: 'profile-viability:v1',
    installed: true
  });
  const meaningfulProfiles = [
    profile('software', 'Software Engineering', ['contribution:pi-session', 'contribution:tasks']),
    profile('research', 'Research', ['contribution:research'])
  ];
  let calls = 0;
  const requests = [];
  let response = structuredClone(meaningfulProfiles);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const listed = await client.profileList(structuredClone(authority));
  assert.deepEqual(listed, meaningfulProfiles);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/profiles');
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('attachment')), authority.attachment);
  assert.equal(requestUrl.searchParams.get('continuity_id'), authority.continuity_id);
  assert.equal(requestUrl.searchParams.get('workspace_binding_id'), authority.workspace_binding_id);
  assert.equal(requestUrl.searchParams.get('runtime_object'), JSON.stringify(authority.runtime_object));
  assert.equal(requestUrl.searchParams.get('work_surface_id'), authority.work_surface_id);
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'profile.list must not mint an operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  // The Core response is authoritative. Desktop accepts only the generated
  // meaningful eligible profile list and never invents an unavailable option.
  response = [];
  assert.deepEqual(await client.profileList(structuredClone(authority)), [], 'empty profile list must stay empty');

  const wrappedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ profiles: meaningfulProfiles }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrappedTransport).profileList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:expected array'
  );

  const malformedProfile = structuredClone(meaningfulProfiles[0]);
  delete malformedProfile.candidate_contribution_ids;
  const malformedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([malformedProfile]), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(malformedTransport).profileList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:0:')
  );

  const foreignProfile = { ...structuredClone(meaningfulProfiles[0]), workstream: { workstream_id: 'ws:foreign' } };
  const foreignPayloadTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([foreignProfile]), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignPayloadTransport).profileList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:0:unknown:workstream')
  );

  const foreignScopeTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_identity_mismatch' } }), { status: 409 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignScopeTransport).profileList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 409
  );

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(meaningfulProfiles), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).profileList({}),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).profileList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).profileList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.unavailable', structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated profileList, exact Workstream GET, meaningful eligible profile list, empty profile list, foreign scope, missing authority, permission, unavailable operation, and hostile response checks)');
}

async function exerciseActivityList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const activity = (activityModeId, displayName, candidateContributionIds, revision = 1) => ({
    activity_mode_id: activityModeId,
    revision,
    display_name: displayName,
    candidate_contribution_ids: candidateContributionIds,
    terminology_overrides_ref: `registry:terminology:activity:${activityModeId}`,
    viability_rule_revision: 'activity-viability:v1'
  });
  const registeredActivities = [
    activity('overview', 'Overview', ['contribution:pi-session', 'contribution:project-overview']),
    activity('sessions', 'Sessions', ['contribution:pi-session'])
  ];
  let calls = 0;
  const requests = [];
  let response = structuredClone(registeredActivities);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const listed = await client.activityList(structuredClone(authority));
  assert.deepEqual(listed, registeredActivities);
  assert.equal(calls, 1);
  assert.equal('activities' in listed, false, 'activityList must return direct generated ActivityMode[]');
  assert.equal('workstream' in listed[0], false, 'activity metadata must not invent response authority fields');

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/activities');
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('attachment')), authority.attachment);
  assert.equal(requestUrl.searchParams.get('continuity_id'), authority.continuity_id);
  assert.equal(requestUrl.searchParams.get('workspace_binding_id'), authority.workspace_binding_id);
  assert.equal(requestUrl.searchParams.get('runtime_object'), JSON.stringify(authority.runtime_object));
  assert.equal(requestUrl.searchParams.get('work_surface_id'), authority.work_surface_id);
  assert.equal(requestUrl.searchParams.get('after_cursor'), null, 'activity.list has no stale cursor input');
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'activity.list must not invent an operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  // Core-owned ineligible/empty activity output remains empty. Desktop never
  // creates a navigation choice merely because a registry ID is known.
  response = [];
  assert.deepEqual(await client.activityList(structuredClone(authority)), [], 'empty activity list must remain empty');

  const wrappedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ activities: registeredActivities }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrappedTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:expected array'
  );

  const malformedActivity = structuredClone(registeredActivities[0]);
  delete malformedActivity.candidate_contribution_ids;
  const malformedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([malformedActivity]), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(malformedTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:0:')
  );

  // A response cannot turn an activity DTO into a foreign scoped object. The
  // generated validator rejects the invented field rather than adopting it.
  const foreignActivity = { ...structuredClone(registeredActivities[0]), workstream: { workstream_id: 'ws:foreign' } };
  const foreignPayloadTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([foreignActivity]), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignPayloadTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:0:unknown:workstream')
  );

  const foreignScopeTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_identity_mismatch' } }), { status: 409 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignScopeTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 409
  );

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(registeredActivities), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).activityList({}),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before activity.list HTTP');

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  }, undefined, 30_000, ['mission_canvas:read'], []);
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  const unavailableCapabilityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Capabilities'], undefined, 'no activity capability may be minted locally');
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(unavailableCapabilityTransport).activityList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.activity.unavailable', structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated activityList, exact Workstream GET, direct registered ActivityMode[], empty activity list, foreign scope, missing authority, permission, unavailable capability/operation, and hostile response checks)');
}

async function exerciseRegistryList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const registryEntry = (entryId, kind) => ({
    enabled: true,
    entry_id: entryId,
    payload_ref: `registry-entry:${entryId}@1`,
    registry_kind: kind,
    revision: 1,
    schema_ref: 'registry-entry.schema.json',
    required_capabilities: [],
    required_permissions: []
  });
  const registryKind = 'PanelRegistry';
  const entries = [
    registryEntry('panel:focusa-inspector', registryKind),
    registryEntry('panel:work-rail', registryKind)
  ];
  let calls = 0;
  const requests = [];
  let response = structuredClone(entries);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const listed = await client.registryList({
    ...structuredClone(authority),
    registry_kind: registryKind
  });
  assert.deepEqual(listed, entries);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, `http://127.0.0.1:8787/v1/mission-canvas/registries/${registryKind}`);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('attachment')), authority.attachment);
  assert.equal(requestUrl.searchParams.get('continuity_id'), authority.continuity_id);
  assert.equal(requestUrl.searchParams.get('workspace_binding_id'), authority.workspace_binding_id);
  assert.equal(requestUrl.searchParams.get('runtime_object'), JSON.stringify(authority.runtime_object));
  assert.equal(requestUrl.searchParams.get('work_surface_id'), authority.work_surface_id);
  assert.equal(requestUrl.searchParams.get('registry_kind'), null);
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'registry.list must not mint an operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  // empty registry output is still a valid generated response when no matching
  // entries exist for the requested kind.
  response = [];
  assert.deepEqual(await client.registryList({
    ...structuredClone(authority),
    registry_kind: registryKind
  }), []);

  const missingPathTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify(response), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingPathTransport).registryList(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'path_parameter_required:registry_kind'
  );

  const wrappedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ entries: entries }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrappedTransport).registryList({
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:expected array'
  );

  const malformedEntry = structuredClone(entries[0]);
  delete malformedEntry.registry_kind;
  const malformedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([malformedEntry]), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(malformedTransport).registryList({
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:0:missing:registry_kind'
  );

  const foreignPayload = structuredClone(entries[0]);
  foreignPayload.workstream = { workstream_id: 'ws:foreign' };
  const foreignPayloadTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([foreignPayload]), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignPayloadTransport).registryList({
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:0:unknown:workstream')
  );

  const foreignScopeTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_identity_mismatch' } }), { status: 409 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignScopeTransport).registryList({
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 409
  );

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(entries), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).registryList({
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before registryList HTTP');

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).registryList({
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).registryList({
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.registry.unavailable', {
      ...structuredClone(authority),
      registry_kind: registryKind
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  const unknownKindTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'registry_kind_unknown' } }), { status: 422 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(unknownKindTransport).registryList({
      ...structuredClone(authority),
      registry_kind: 'UnknownRegistryKind'
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  console.log('Mission Canvas operation consumer: PASS (generated registryList, exact Workstream GET, path param handling, direct RegistryEntry[], empty registry list, foreign scope, missing authority, missing path, malformed/hostile response checks)');
}

async function exerciseActivitySelect({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority, fixture }) {
  // activity.select is a generated mutation whose direct response remains the
  // Core-resolved projection. Desktop must not turn the selected mode into a
  // local candidate/layout decision or adopt a route wrapper.
  let calls = 0;
  const requests = [];
  let response = structuredClone(fixture);
  response.activity_mode_id = 'tasks';
  response.activity_mode_revision = fixture.activity_mode_revision + 1;
  response.projection_revision = fixture.projection_revision + 1;
  response.layout_revision = fixture.layout_revision + 1;
  response.durable_event_cursor = 'event:42';
  response.evidence_refs = ['recomposition-evidence:activity-select'];
  response.receipt_refs = ['recomposition-receipt:activity-select'];
  const input = {
    ...structuredClone(authority),
    selection_id: 'tasks',
    expected_projection_revision: fixture.projection_revision,
    idempotency_key: 'idempotency:activity-select'
  };
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const selected = await client.activitySelect(structuredClone(input));
  assert.deepEqual(selected, response);
  assert.equal(selected.activity_mode_id, 'tasks');
  assert.deepEqual(selected.workstream, authority.workstream);
  assert.deepEqual(selected.attachment, authority.attachment);
  assert.equal('projection' in selected, false, 'activitySelect must not adopt a route wrapper');
  assert.deepEqual(selected.receipt_refs, ['recomposition-receipt:activity-select']);
  assert.equal(calls, 1);
  assert.equal(JSON.stringify(selected.layout_tree).includes('contribution:empty-work-rail'), false, 'canonical layout must omit empty contributions recursively');
  assert.equal(selected.omission_diagnostics.some(({ contribution_id, reason }) => contribution_id === 'contribution:empty-work-rail' && reason === 'no_relevant_content'), true);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/activities/select');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'activity.select must not mint an operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  assert.equal(requests[0].init.headers['If-Match'], String(input.expected_projection_revision));
  assert.equal(requests[0].init.headers['Idempotency-Key'], input.idempotency_key);
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.selection_id, input.selection_id);
  assert.equal(body.expected_projection_revision, input.expected_projection_revision);
  assert.equal(body.idempotency_key, input.idempotency_key);
  assert.equal('candidate_contribution_ids' in body, false, 'Desktop must not own the canonical candidate registry');
  assert.equal('eligible_contributions' in body, false, 'Desktop must not send a projection as activity input');
  assert.equal('layout_tree' in body, false, 'Desktop must not compose activity layout locally');

  let validationCalls = 0;
  const validationTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      validationCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(validationTransport).activitySelect({
      ...structuredClone(input),
      selection_id: undefined
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:missing:selection_id'
  );
  await assert.rejects(
    () => new MissionCanvasClient(validationTransport).activitySelect({
      ...structuredClone(input),
      layout_tree: { kind: 'single', contribution_id: 'invented' }
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:unknown:layout_tree'
  );
  assert.equal(validationCalls, 0, 'missing selection and local composition input must fail before HTTP');

  const missingIdempotency = structuredClone(input);
  delete missingIdempotency.idempotency_key;
  await assert.rejects(
    () => client.activitySelect(missingIdempotency),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  const missingRevision = structuredClone(input);
  delete missingRevision.expected_projection_revision;
  await assert.rejects(
    () => client.activitySelect(missingRevision),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'if_match_revision_required'
  );
  assert.equal(calls, 1, 'missing mutation controls must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).activitySelect({
      selection_id: 'tasks',
      expected_projection_revision: fixture.projection_revision,
      idempotency_key: 'idempotency:missing-scope'
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before activitySelect HTTP');

  const foreignTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(response);
    foreign.workstream.workstream_id = 'ws:foreign';
    foreign.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], [], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_projection_scope'
  );

  const foreignContributionTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreignContribution = structuredClone(response);
    foreignContribution.eligible_contributions[0].authority.workstream.workstream_id = 'ws:foreign';
    foreignContribution.eligible_contributions[0].authority.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreignContribution), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], [], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignContributionTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_contribution_scope'
  );

  const malformedContribution = structuredClone(response);
  malformedContribution.eligible_contributions = [null];
  const malformedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(malformedContribution), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(malformedTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:0:missing_contribution'
  );

  const wrapperTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ projection: response, receipt: { workstream: authority.workstream } }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrapperTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const missingResponseScope = structuredClone(response);
  delete missingResponseScope.workstream;
  const missingResponseScopeTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(missingResponseScope), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingResponseScopeTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const missingContributionAuthority = structuredClone(response);
  missingContributionAuthority.eligible_contributions[0].authority = {};
  const missingContributionAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(missingContributionAuthority), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingContributionAuthorityTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:0:invalid:workstream')
  );

  response.projection_revision = fixture.projection_revision;
  response.layout_revision = fixture.layout_revision + 1;
  response.durable_event_cursor = 'event:41';
  await assert.rejects(
    () => client.activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_revision'
  );
  response.projection_revision = fixture.projection_revision + 2;
  response.layout_revision = fixture.layout_revision;
  response.durable_event_cursor = 'event:43';
  await assert.rejects(
    () => client.activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_layout_revision'
  );
  response.layout_revision = fixture.layout_revision + 1;
  response.durable_event_cursor = 'event:41';
  await assert.rejects(
    () => client.activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_cursor'
  );

  const capabilityDeniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(capabilityDeniedTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).activitySelect(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.activity.unavailable', structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated activitySelect, exact Workstream POST, Core-owned direct recomposition, trusted recursive projection data, empty omission, foreign authority, missing selection/If-Match/idempotency, stale revision/layout/cursor, capability/permission/authority, and hostile response checks)');
}

async function exerciseLayoutMemoryGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const memory = {
    ...structuredClone(authority),
    memory_id: 'layout-memory:software:overview:standard',
    profile_id: 'software',
    activity_mode_id: 'overview',
    viewport_class: 'standard',
    placements: [{
      contribution_id: 'contribution:pi-session',
      preferred_regions: ['primary'],
      preferred_order: 0,
      minimum_span: 3,
      maximum_span: 8,
      preferred_adjacency: [],
      last_compatible_layout_node_id: 'layout:primary'
    }],
    absent_contribution_ids: ['contribution:empty-work-rail'],
    focused_semantic_target: 'focus:pi-session',
    memory_revision: 4,
    idempotency_key: 'memory:fixture',
    updated_at: '2026-08-07T00:00:00Z'
  };
  const input = {
    ...structuredClone(authority),
    profile_id: memory.profile_id,
    activity_mode_id: memory.activity_mode_id,
    viewport_class: memory.viewport_class
  };
  let calls = 0;
  const requests = [];
  let response = structuredClone(memory);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const fetched = await client.layout_memoryGet(structuredClone(input));
  assert.deepEqual(fetched, memory);
  assert.deepEqual(fetched.workstream, authority.workstream);
  assert.deepEqual(fetched.attachment, authority.attachment);
  assert.equal('document' in fetched, false, 'layout_memoryGet must not adopt a StoredDocument envelope');
  assert.equal('layout_tree' in fetched, false, 'layout memory must not become a client-local projection');
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/layout-memory');
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('attachment')), authority.attachment);
  assert.equal(requestUrl.searchParams.get('continuity_id'), authority.continuity_id);
  assert.equal(requestUrl.searchParams.get('workspace_binding_id'), authority.workspace_binding_id);
  assert.equal(requestUrl.searchParams.get('runtime_object'), JSON.stringify(authority.runtime_object));
  assert.equal(requestUrl.searchParams.get('work_surface_id'), authority.work_surface_id);
  assert.equal(requestUrl.searchParams.get('profile_id'), memory.profile_id);
  assert.equal(requestUrl.searchParams.get('activity_mode_id'), memory.activity_mode_id);
  assert.equal(requestUrl.searchParams.get('viewport_class'), memory.viewport_class);
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'layout memory GET must not mint a capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  for (const field of ['profile_id', 'activity_mode_id', 'viewport_class']) {
    const missing = structuredClone(input);
    delete missing[field];
    await assert.rejects(
      () => client.layout_memoryGet(missing),
      (error) => error instanceof MissionCanvasTransportError && error.message === `invalid_request:missing:${field}`
    );
  }
  const localComposition = { ...structuredClone(input), layout_tree: { kind: 'single', contribution_id: 'invented' } };
  await assert.rejects(
    () => client.layout_memoryGet(localComposition),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:unknown:layout_tree'
  );
  assert.equal(calls, 1, 'invalid selectors and local composition must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(memory), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).layout_memoryGet({
      profile_id: memory.profile_id,
      activity_mode_id: memory.activity_mode_id,
      viewport_class: memory.viewport_class
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before layout memory HTTP');

  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(memory);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_profile_memory_scope'
  );

  const foreignAttachmentTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(memory);
      foreign.attachment.attachment_id = 'attachment:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignAttachmentTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_profile_memory_scope'
  );

  const foreignProfileTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreignProfile = structuredClone(memory);
    foreignProfile.profile_id = 'research';
    foreignProfile.memory_id = 'layout-memory:research:overview:standard';
    return new Response(JSON.stringify(foreignProfile), { status: 200 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(foreignProfileTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_profile_memory_profile_id'
  );

  const wrappedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ memory, document: { payload: memory } }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrappedTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const malformedPlacement = structuredClone(memory);
  malformedPlacement.placements[0].minimum_span = 0;
  const malformedPlacementTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(malformedPlacement), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(malformedPlacementTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:placements:0:content'
  );

  response = structuredClone(memory);
  response.memory_revision = 3;
  await assert.rejects(
    () => client.layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_profile_memory_revision'
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const unavailableCapabilityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(unavailableCapabilityTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).layout_memoryGet(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.layout_memory.unavailable', structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated layout_memoryGet, exact Workstream/profile GET, direct ProfileLayoutMemory, no local composition, foreign authority/profile, stale memory revision, missing selectors/authority, capability/permission denial, and hostile response checks)');
}

async function exerciseLayoutMemoryUpdate({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  // The mutation consumes the generated ProfileLayoutMemory representation,
  // persists only semantic placement preference, and returns the direct Core
  // RecompositionReceipt. Desktop never receives or reconstructs a layout tree.
  const memory = {
    ...structuredClone(authority),
    memory_id: 'layout-memory:software:overview:standard',
    profile_id: 'software',
    activity_mode_id: 'overview',
    viewport_class: 'standard',
    placements: [{
      contribution_id: 'contribution:pi-session',
      preferred_regions: ['primary'],
      preferred_order: 0,
      minimum_span: 3,
      maximum_span: 8,
      preferred_adjacency: [],
      last_compatible_layout_node_id: 'layout:primary'
    }],
    // An omitted contribution remains durable memory only. It is not a
    // request to render a placeholder or reserve a geometry slot.
    absent_contribution_ids: ['contribution:empty-work-rail'],
    focused_semantic_target: 'focus:pi-session',
    memory_revision: 4,
    idempotency_key: 'idempotency:layout-memory:update',
    updated_at: '2026-08-07T00:00:00Z'
  };
  const input = structuredClone(memory);
  const digest = `sha256:${'a'.repeat(64)}`;
  let response = {
    ...structuredClone(authority),
    receipt_id: 'recomposition-receipt:layout-memory:5',
    accepted: true,
    projection_revision: 5,
    layout_revision: 5,
    projection_digest: digest,
    event_cursor: 'event:42',
    evidence_id: 'recomposition-evidence:layout-memory:5',
    idempotency_key: input.idempotency_key,
    issued_at: '2026-08-07T00:00:01Z'
  };
  let calls = 0;
  const requests = [];
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const receipt = await client.layout_memoryUpdate(structuredClone(input));
  assert.deepEqual(receipt, response);
  assert.equal(receipt.accepted, true);
  assert.deepEqual(receipt.workstream, authority.workstream);
  assert.deepEqual(receipt.attachment, authority.attachment);
  assert.equal('receipt' in receipt, false, 'layout_memoryUpdate must not adopt a route wrapper');
  assert.equal('memory' in receipt, false, 'layout_memoryUpdate returns Receipt, not a memory wrapper');
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/layout-memory');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], undefined, 'update has no invented operation capability');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  assert.equal(requests[0].init.headers['If-Match'], String(input.memory_revision));
  assert.equal(requests[0].init.headers['Idempotency-Key'], input.idempotency_key);
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.deepEqual(body.absent_contribution_ids, ['contribution:empty-work-rail']);
  assert.equal(body.memory_revision, input.memory_revision);
  assert.equal(body.idempotency_key, input.idempotency_key);
  assert.equal('layout_tree' in body, false, 'Desktop must not compose layout locally');
  assert.equal('eligible_contributions' in body, false, 'Desktop must not own eligibility');
  assert.equal('document_id' in body, false, 'generated input is not a persistence envelope');
  assert.equal('payload' in body, false, 'generated input is not a route-local document wrapper');

  const missingIdempotency = structuredClone(input);
  delete missingIdempotency.idempotency_key;
  await assert.rejects(
    () => client.layout_memoryUpdate(missingIdempotency),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, 1, 'missing idempotency must fail before HTTP');

  const missingRevision = structuredClone(input);
  delete missingRevision.memory_revision;
  await assert.rejects(
    () => client.layout_memoryUpdate(missingRevision),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'if_match_revision_required'
  );
  assert.equal(calls, 1, 'missing If-Match revision must fail before HTTP');

  const localComposition = { ...structuredClone(input), layout_tree: { kind: 'single', contribution_id: 'invented' } };
  await assert.rejects(
    () => client.layout_memoryUpdate(localComposition),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:unknown:layout_tree'
  );
  const malformedPlacement = structuredClone(input);
  malformedPlacement.placements[0].minimum_span = 0;
  await assert.rejects(
    () => client.layout_memoryUpdate(malformedPlacement),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).layout_memoryUpdate({
      memory_id: input.memory_id,
      profile_id: input.profile_id,
      activity_mode_id: input.activity_mode_id,
      viewport_class: input.viewport_class,
      memory_revision: input.memory_revision,
      idempotency_key: 'idempotency:missing-scope'
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  const foreignReceiptTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(response);
    foreign.workstream.workstream_id = 'ws:foreign';
    foreign.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], [], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignReceiptTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_layout_memory_receipt_scope'
  );

  const foreignAttachmentTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(response);
    foreign.attachment.attachment_id = 'attachment:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], [], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignAttachmentTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_layout_memory_receipt_scope'
  );

  const mismatchedIdempotencyTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    return new Response(JSON.stringify({ ...structuredClone(response), idempotency_key: 'idempotency:other' }), { status: 200 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(mismatchedIdempotencyTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:idempotency_key_mismatch'
  );

  const wrappedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ receipt: response }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrappedTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const notAcceptedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ ...structuredClone(response), accepted: false }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(notAcceptedTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:receipt_not_accepted'
  );

  const wrongRevisionTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ ...structuredClone(response), accepted: true, projection_revision: 9, layout_revision: 9, event_cursor: 'event:49' }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrongRevisionTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:layout_memory_revision_mismatch'
  );

  response = { ...structuredClone(response), projection_revision: 4, layout_revision: 4, event_cursor: 'event:41' };
  await assert.rejects(
    () => client.layout_memoryUpdate({ ...structuredClone(input), memory_revision: 3 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_layout_memory_revision'
  );
  response = { ...structuredClone(response), projection_revision: 6, layout_revision: 6, event_cursor: 'event:41' };
  await assert.rejects(
    () => client.layout_memoryUpdate({ ...structuredClone(input), memory_revision: 5 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_layout_memory_cursor'
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const unavailableCapabilityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Capabilities'], undefined, 'Desktop must not mint an update capability');
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(unavailableCapabilityTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
    assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).layout_memoryUpdate(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.layout_memory.unavailable', structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated layout_memoryUpdate, exact Workstream POST, direct RecompositionReceipt, semantic placement persistence, Idempotency-Key/If-Match, foreign receipt authority, stale memory revision/cursor, empty/ineligible memory, no local composition, permission/capability, and hostile response checks)');
}

async function exerciseProfileSelect({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority, fixture }) {
  let response = structuredClone(fixture);
  response.projection_revision = fixture.projection_revision + 1;
  response.layout_revision = fixture.layout_revision + 1;
  response.durable_event_cursor = 'event:42';
  const requests = [];
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (url, init) => {
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['focusa.mission_canvas.profile.select'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = {
    ...structuredClone(authority),
    selection_id: 'research',
    expected_projection_revision: fixture.projection_revision,
    idempotency_key: 'idempotency:profile-select-consumer'
  };
  const selected = await client.profileSelect(input);
  assert.equal(selected.projection_revision, fixture.projection_revision + 1);
  const request = requests[0];
  assert.equal(new URL(request.url).pathname, '/v1/mission-canvas/profiles/select');
  assert.equal(request.init.method, 'POST');
  assert.equal(request.init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(request.init.headers['X-Focusa-Capabilities'], 'focusa.mission_canvas.profile.select');
  assert.equal(request.init.headers['If-Match'], String(fixture.projection_revision));
  assert.equal(request.init.headers['Idempotency-Key'], input.idempotency_key);
  assert.equal(JSON.parse(request.init.body).selection_id, 'research');
  assert.deepEqual(JSON.parse(request.init.body).workstream, authority.workstream);

  response = structuredClone(fixture);
  response.projection_revision = fixture.projection_revision;
  response.layout_revision = fixture.layout_revision + 2;
  response.durable_event_cursor = 'event:43';
  await assert.rejects(
    () => client.profileSelect(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_revision'
  );

  response = structuredClone(fixture);
  response.projection_revision = fixture.projection_revision + 2;
  response.layout_revision = fixture.layout_revision + 2;
  response.durable_event_cursor = 'event:41';
  await assert.rejects(
    () => client.profileSelect(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_cursor'
  );

  response = structuredClone(fixture);
  response.projection_revision = fixture.projection_revision + 3;
  response.layout_revision = fixture.layout_revision + 3;
  response.durable_event_cursor = 'event:44';
  response.workstream.workstream_id = 'ws:foreign-profile-select';
  response.attachment.workstream.workstream_id = 'ws:foreign-profile-select';
  await assert.rejects(
    () => client.profileSelect(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_projection_scope'
  );

  let calls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      calls += 1;
      return new Response(JSON.stringify(fixture), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['focusa.mission_canvas.profile.select'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).profileSelect({
      selection_id: 'research',
      expected_projection_revision: fixture.projection_revision,
      idempotency_key: 'idempotency:missing-profile-scope'
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(calls, 0, 'missing Workstream authority must fail before profileSelect HTTP');

  const deniedTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Permissions'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked' }), { status: 403 });
    }
  );
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).profileSelect(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.select.unavailable', structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated profileSelect, exact Workstream POST, idempotency/concurrency, foreign scope, stale revision/cursor, missing authority, permission, and unavailable operation checks)');
}

async function exerciseProjectionGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  let calls = 0;
  const requests = [];
  let response = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.projection.get'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  // projectionGet returns the generated ResolvedWorkspaceProjection DTO;
  // no Desktop-owned projection shape or composition resolver is introduced.
  const projection = await client.projectionGet(structuredClone(authority));
  assert.deepEqual(projection, response);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/projection');
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('workstream')), authority.workstream);
  assert.deepEqual(JSON.parse(requestUrl.searchParams.get('attachment')), authority.attachment);
  assert.equal(requestUrl.searchParams.get('continuity_id'), authority.continuity_id);
  assert.equal(requestUrl.searchParams.get('workspace_binding_id'), authority.workspace_binding_id);
  assert.equal(requestUrl.searchParams.get('runtime_object'), JSON.stringify(authority.runtime_object));
  assert.equal(requestUrl.searchParams.get('work_surface_id'), authority.work_surface_id);
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], 'mission_canvas.projection.get');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');

  // Eligibility and composition remain Core-owned: the empty exact Workstream
  // Work Rail is omitted and diagnosed; Desktop never invents a replacement contribution.
  assert.equal(projection.eligible_contributions.some(({ contribution_id }) => contribution_id === 'contribution:empty-work-rail'), false);
  assert.equal(projection.omission_diagnostics.some(({ contribution_id, reason }) => contribution_id === 'contribution:empty-work-rail' && reason === 'no_relevant_content'), true);
  assert.equal(JSON.stringify(projection.layout_tree).includes('contribution:empty-work-rail'), false);

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.projection.get'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).projectionGet({}),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  const foreignTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(response);
    foreign.workstream.workstream_id = 'ws:foreign';
    foreign.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:read'], ['mission_canvas.projection.get'], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_projection_scope'
  );

  const foreignContributionTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreignContribution = structuredClone(response);
    foreignContribution.eligible_contributions[0].authority.workstream.workstream_id = 'ws:foreign';
    foreignContribution.eligible_contributions[0].authority.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreignContribution), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:read'], ['mission_canvas.projection.get'], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignContributionTransport).projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_contribution_scope'
  );

  const missingResponseScope = structuredClone(response);
  delete missingResponseScope.workstream;
  const invalidResponseTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(missingResponseScope), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidResponseTransport).projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  const malformedContributionResponse = structuredClone(response);
  malformedContributionResponse.eligible_contributions = [null];
  const malformedContributionTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(malformedContributionResponse), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(malformedContributionTransport).projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:0:missing_contribution'
  );

  // A stale projection is not adopted even when the server response remains
  // structurally valid. The durable cursor is an independent watermark.
  response.projection_revision -= 1;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_revision'
  );
  response.projection_revision += 2;
  await assert.rejects(
    () => client.projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_cursor'
  );
  response.projection_revision += 1;
  response.layout_revision -= 2;
  response.durable_event_cursor = 'event:42';
  await assert.rejects(
    () => client.projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_layout_revision'
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).projectionGet(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  await assert.rejects(
    () => transport.request('focusa.mission_canvas.projection.unavailable', structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );

  console.log('Mission Canvas operation consumer: PASS (generated projectionGet, exact Workstream GET, Core-owned omission, foreign authority, stale revision/layout/cursor, permission, unavailable operation, and hostile response checks)');
}

async function exerciseProjectionResolve({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  // The generated ContributionEligibilityContext is request-only; Core owns
  // candidate eligibility, composition, layout, evidence, and receipt output.
  let calls = 0;
  const requests = [];
  let response = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  response.evidence_refs = ['evidence:projection-resolve'];
  response.receipt_refs = ['receipt:projection-resolve'];
  const input = {
    ...structuredClone(authority),
    workspace_profile_id: response.workspace_profile_id,
    workspace_profile_revision: response.workspace_profile_revision,
    activity_mode_id: response.activity_mode_id,
    activity_mode_revision: response.activity_mode_revision,
    focused_work_surface_id: response.focused_work_surface_id,
    open_work_surface_ids: [response.focused_work_surface_id],
    pinned_work_surface_ids: [],
    canonical_read_model_revision: response.canonical_read_model_revision,
    available_operations: ['focusa.agent_execution.prompt'],
    capabilities: ['mission_canvas.projection.resolve'],
    permissions: ['mission_canvas:write'],
    viewport: {
      class: 'standard',
      css_width: 1440,
      css_height: 900,
      device_pixel_ratio: 2,
      platform: 'macOS'
    },
    project_constraint_refs: [],
    user_preference_ref: null,
    resolver_rule_revision: 'adaptive-composition:v1',
    observed_at: '2026-07-30T12:00:00Z',
    previous_projection_revision: response.projection_revision - 1,
    previous_layout_revision: response.layout_revision - 1,
    event_cursor: 'event:40',
    idempotency_key: 'idempotency:projection-resolve'
  };

  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['mission_canvas.projection.resolve'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const projection = await client.projectionResolve(structuredClone(input));
  assert.deepEqual(projection, response);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/projection/resolve');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], 'mission_canvas.projection.resolve');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  assert.equal(requests[0].init.headers['If-Match'], '11');
  assert.equal(requests[0].init.headers['Idempotency-Key'], input.idempotency_key);
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.workspace_profile_id, input.workspace_profile_id);
  assert.equal(body.activity_mode_id, input.activity_mode_id);
  assert.deepEqual(body.viewport, input.viewport);
  assert.equal('candidates' in body, false, 'Desktop must not own the candidate registry');
  assert.equal('layout_tree' in body, false, 'Desktop must not compose a layout');
  assert.equal('idempotency_key' in body, false, 'idempotency is transport metadata, not a generated context field');
  assert.equal('previous_projection_revision' in body, false, 'If-Match is transport metadata, not a generated context field');
  assert.equal('event_cursor' in body, false, 'Core derives the durable cursor from exact state');
  assert.equal(projection.receipt_refs.length > 0, true, 'the direct projection carries the Core receipt reference');
  assert.equal(projection.eligible_contributions.some(({ contribution_id }) => contribution_id === 'contribution:empty-work-rail'), false);
  assert.equal(projection.omission_diagnostics.some(({ contribution_id, reason }) => contribution_id === 'contribution:empty-work-rail' && reason === 'no_relevant_content'), true);
  assert.equal(JSON.stringify(projection.layout_tree).includes('contribution:empty-work-rail'), false);

  let missingIdempotencyCalls = calls;
  const missingIdempotency = { ...structuredClone(input) };
  delete missingIdempotency.idempotency_key;
  await assert.rejects(
    () => client.projectionResolve(missingIdempotency),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing idempotency must fail before HTTP');

  const missingRevision = { ...structuredClone(input) };
  delete missingRevision.previous_projection_revision;
  await assert.rejects(
    () => client.projectionResolve(missingRevision),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'if_match_revision_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing If-Match revision must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['mission_canvas.projection.resolve'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).projectionResolve({ idempotency_key: 'idempotency:missing-scope', previous_projection_revision: 0 }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  const foreignTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreign = structuredClone(response);
    foreign.workstream.workstream_id = 'ws:foreign';
    foreign.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreign), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], ['mission_canvas.projection.resolve'], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_projection_scope'
  );

  const foreignContributionTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () => {
    const foreignContribution = structuredClone(response);
    foreignContribution.eligible_contributions[0].authority.workstream.workstream_id = 'ws:foreign';
    foreignContribution.eligible_contributions[0].authority.attachment.workstream.workstream_id = 'ws:foreign';
    return new Response(JSON.stringify(foreignContribution), { status: 200 });
  }, undefined, 30_000, ['mission_canvas:write'], ['mission_canvas.projection.resolve'], 'actor:desktop', 'authority:desktop');
  await assert.rejects(
    () => new MissionCanvasClient(foreignContributionTransport).projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_contribution_scope'
  );

  const wrapperTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify({ schema: 'focusa.mission_canvas.resolve_result.v1', projection: response }), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrapperTransport).projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  response.projection_revision = 11;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_revision'
  );
  response.projection_revision = 13;
  response.layout_revision = 4;
  response.durable_event_cursor = 'event:43';
  await assert.rejects(
    () => client.projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_layout_revision'
  );
  response.layout_revision = 6;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_cursor'
  );

  const capabilityDeniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(capabilityDeniedTransport).projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const deniedTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async (_url, init) => {
    assert.equal(init.headers['X-Focusa-Permissions'], undefined);
    assert.equal(init.headers['If-Match'], '11');
    return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
  });
  await assert.rejects(
    () => new MissionCanvasClient(deniedTransport).projectionResolve(structuredClone(input)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  console.log('Mission Canvas operation consumer: PASS (generated projectionResolve, Core-owned direct projection, exact Workstream, If-Match/idempotency, omission, foreign authority, stale revision/layout/cursor, capability, permission, and hostile response checks)');
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

async function exerciseHostLaunch({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const rendererResolution = {
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
  const lifecycle = {
    ...structuredClone(authority),
    host_instance_id: 'rich-host:desktop:mission-canvas',
    renderer_resolution: rendererResolution,
    state: 'visible',
    focused: true,
    process_id: null,
    window_id: 'window:mission-canvas',
    pi_draft_ref: null,
    canvas_draft_ref: null,
    last_error_ref: null,
    durable_event_cursor: 'event:41',
    lifecycle_revision: 1,
    updated_at: '2026-08-07T00:00:00Z'
  };

  let calls = 0;
  const requests = [];
  let response = structuredClone(lifecycle);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = {
    ...structuredClone(authority),
    idempotency_key: 'idempotency:rich-host:launch'
  };
  const launched = await client.rich_hostLaunch(input);
  assert.deepEqual(launched, lifecycle);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/rich-host/launch');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:host');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], 'mission_canvas.desktop_tauri');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.continuity_id, authority.continuity_id);
  assert.equal(body.workspace_binding_id, authority.workspace_binding_id);
  assert.deepEqual(body.runtime_object, authority.runtime_object);
  assert.equal(body.work_surface_id, authority.work_surface_id);
  assert.equal(body.idempotency_key, input.idempotency_key);
  assert.equal('document_id' in body, false, 'launch must use the generated command, not a document envelope');
  assert.equal('payload' in body, false, 'launch must not expose route-local lifecycle payloads');
  assert.equal('pi_session_id' in body, false, 'launch must not fork or replace the Pi session');

  let missingIdempotencyCalls = calls;
  await assert.rejects(
    () => client.rich_hostLaunch(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing idempotency must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).rich_hostLaunch({ idempotency_key: 'idempotency:missing-scope' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  // A foreign lifecycle response is rejected at the generated transport
  // boundary; it cannot be adopted from another Workstream or attachment.
  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(lifecycle);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_lifecycle_scope'
  );

  const foreignRendererTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(lifecycle);
      foreign.renderer_resolution.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignRendererTransport).rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_lifecycle_scope'
  );

  const missingRendererScope = structuredClone(lifecycle);
  delete missingRendererScope.renderer_resolution.workstream;
  const missingRendererScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify(missingRendererScope), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingRendererScopeTransport).rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:renderer_resolution_workstream'
  );

  const invalidHostId = structuredClone(lifecycle);
  invalidHostId.host_instance_id = 'host-invented';
  const invalidHostTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(invalidHostId), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidHostTransport).rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:host_instance_id'
  );

  // A structurally valid but stale lifecycle response is not adopted.
  response = structuredClone(lifecycle);
  response.lifecycle_revision = 0;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_revision'
  );
  response.lifecycle_revision = 2;
  await assert.rejects(
    () => client.rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_cursor'
  );

  const invalidState = structuredClone(lifecycle);
  invalidState.state = 'not-a-lifecycle-state';
  const invalidStateTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(invalidState), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidStateTransport).rich_hostLaunch(input),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  console.log('Mission Canvas operation consumer: PASS (generated rich_hostLaunch, exact Workstream POST, idempotency, Desktop presentation, foreign lifecycle, stale lifecycle, no Pi fork, and hostile response checks)');
}

async function exerciseHostFocus({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const rendererResolution = {
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
  const lifecycle = {
    ...structuredClone(authority),
    host_instance_id: 'rich-host:desktop:mission-canvas',
    renderer_resolution: rendererResolution,
    state: 'focused',
    focused: true,
    process_id: null,
    window_id: 'window:mission-canvas',
    pi_draft_ref: 'draft:pi',
    canvas_draft_ref: 'draft:canvas',
    last_error_ref: null,
    durable_event_cursor: 'event:41',
    lifecycle_revision: 2,
    updated_at: '2026-08-07T00:00:00Z'
  };

  let calls = 0;
  const requests = [];
  let response = structuredClone(lifecycle);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = {
    ...structuredClone(authority),
    idempotency_key: 'idempotency:rich-host:focus'
  };
  const focused = await client.rich_hostFocus(input);
  assert.deepEqual(focused, lifecycle);
  assert.equal(focused.state, 'focused');
  assert.equal(focused.focused, true);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/rich-host/focus');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:host');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], 'mission_canvas.desktop_tauri');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.continuity_id, authority.continuity_id);
  assert.equal(body.workspace_binding_id, authority.workspace_binding_id);
  assert.deepEqual(body.runtime_object, authority.runtime_object);
  assert.equal(body.work_surface_id, authority.work_surface_id);
  assert.equal(body.idempotency_key, input.idempotency_key);
  for (const field of ['activity_mode_id', 'workspace_profile_id', 'projection_revision', 'layout_revision', 'state', 'renderer_resolution']) {
    assert.equal(field in body, false, `focus must not mutate canonical activity or compose locally: ${field}`);
  }
  assert.equal('eligible_contributions' in focused, false, 'focus must not invent composition output');

  const callsBeforeMissingIdempotency = calls;
  await assert.rejects(
    () => client.rich_hostFocus(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, callsBeforeMissingIdempotency, 'missing idempotency must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).rich_hostFocus({ idempotency_key: 'idempotency:missing-scope' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  // A foreign focus response cannot be adopted from another Workstream.
  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(lifecycle);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_lifecycle_scope'
  );

  const foreignRendererTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreignRenderer = structuredClone(lifecycle);
      foreignRenderer.renderer_resolution.selected_renderer = 'focusa_pi_rich_window';
      return new Response(JSON.stringify(foreignRenderer), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignRendererTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:focus_renderer'
  );

  const missingRendererScope = structuredClone(lifecycle);
  delete missingRendererScope.renderer_resolution.workstream;
  const missingRendererScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify(missingRendererScope), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingRendererScopeTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:renderer_resolution_workstream'
  );

  const wrongState = structuredClone(lifecycle);
  wrongState.state = 'visible';
  wrongState.focused = false;
  const wrongStateTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(wrongState), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrongStateTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:focus_state'
  );

  const invalidHostId = structuredClone(lifecycle);
  invalidHostId.host_instance_id = 'host-invented';
  const invalidHostTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(invalidHostId), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidHostTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:host_instance_id'
  );

  // A structurally valid but stale focus response is not adopted.
  response = structuredClone(lifecycle);
  response.lifecycle_revision = 1;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_revision'
  );
  response.lifecycle_revision = 3;
  await assert.rejects(
    () => client.rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_cursor'
  );

  const noCapabilityTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(noCapabilityTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const noPermissionTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Permissions'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
    },
    undefined,
    30_000,
    [],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(noPermissionTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
      assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri']
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).rich_hostFocus(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  console.log('Mission Canvas operation consumer: PASS (generated rich_hostFocus, exact Workstream POST, existing Desktop focus, canonical activity preservation, foreign authority/renderer, missing authority, capability/permission denial, stale lifecycle, and hostile response checks)');
}

async function exerciseRecompositionDiagnosticsList({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const now = new Date().toISOString();
  const diagnostics = [
    { contribution_id: 'contribution:omitted', reason: 'capability_not_present', rule_revision: 'resolver:v2', projection_revision: 11, canonical_input_refs: [], observed_at: now },
    { contribution_id: 'contribution:ineligible', reason: 'not_authorized', rule_revision: 'resolver:v2', projection_revision: 11, canonical_input_refs: [], observed_at: now }
  ];
  let responseBody = structuredClone(diagnostics);
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  const client = new MissionCanvasClient(transport);
  const result = await client.recompositionDiagnosticsList({ ...structuredClone(authority), revision: 11 });
  assert.equal(result.length, 2);
  assert.equal(result[0].contribution_id, 'contribution:omitted');

  const foreignTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response('not-an-array', { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).recompositionDiagnosticsList({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message.includes('invalid_response')
  );

  responseBody = [{ contribution_id: '', reason: 'not_authorized', rule_revision: 'resolver:v2', projection_revision: 11, canonical_input_refs: [], observed_at: now }];
  const badRowTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(badRowTransport).recompositionDiagnosticsList({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message.includes('invalid_response')
  );

  const acceptedCustomReason = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify([{ contribution_id: 'contribution:omitted', reason: 'custom_omission', rule_revision: 'resolver:v2', projection_revision: 11, canonical_input_refs: [], observed_at: now }]), { status: 200 })
  );
  const accepted = await new MissionCanvasClient(acceptedCustomReason).recompositionDiagnosticsList({ ...structuredClone(authority), revision: 11 });
  assert.equal(accepted[0].contribution_id, 'contribution:omitted');
}


async function exerciseRecompositionReceiptGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  assert.ok(authority.attachment, 'recomposition.receipt.get fixture requires exact attachment authority');
  const receipt = {
    ...structuredClone(authority),
    attachment: structuredClone(authority.attachment),
    accepted: true,
    error_ref: null,
    event_cursor: 'event:11',
    evidence_id: 'recomposition-evidence:key:11',
    idempotency_key: 'idem:resolve:11',
    issued_at: new Date().toISOString(),
    layout_revision: 5,
    projection_digest: `sha256:${'r'.repeat(64)}`,
    projection_revision: 11,
    receipt_id: 'recomposition-receipt:key:11'
  };
  let responseBody = structuredClone(receipt);
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  const client = new MissionCanvasClient(transport);
  const result = await client.recompositionReceiptGet({ ...structuredClone(authority), revision: 11 });
  assert.equal(result.receipt_id, receipt.receipt_id);
  assert.equal(result.projection_revision, 11);
  assert.equal(result.accepted, true);

  responseBody = structuredClone(receipt);
  responseBody.workstream.workstream_id = 'ws:foreign';
  if (responseBody.attachment) responseBody.attachment.workstream.workstream_id = 'ws:foreign';
  await assert.rejects(
    () => client.recompositionReceiptGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:scope_mismatch'
  );

  responseBody = { ...structuredClone(receipt), receipt_id: '' };
  await assert.rejects(
    () => client.recompositionReceiptGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:receipt_id'
  );

  responseBody = { ...structuredClone(receipt), projection_revision: -1 };
  await assert.rejects(
    () => client.recompositionReceiptGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:projection_revision'
  );

  responseBody = { ...structuredClone(receipt), accepted: false };
  await assert.rejects(
    () => client.recompositionReceiptGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:not_accepted'
  );
}

async function exerciseRecipientResolve({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  assert.ok(authority.attachment, 'recipient.resolve fixture requires exact attachment authority');
  const resolution = {
    schema: 'focusa.mission_canvas.recipient_resolution.v1',
    ...structuredClone(authority),
    attachment: structuredClone(authority.attachment),
    recipient_ref: `recipient:session:${authority.attachment.session_id}`,
    routable: true
  };
  let responseBody = structuredClone(resolution);
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  const client = new MissionCanvasClient(transport);
  const result = await client.recipientResolve({ ...structuredClone(authority), recipient_ref: resolution.recipient_ref });
  assert.equal(result.routable, true);
  assert.equal(result.schema, 'focusa.mission_canvas.recipient_resolution.v1');

  responseBody = structuredClone(resolution);
  responseBody.workstream.workstream_id = 'ws:foreign';
  if (responseBody.attachment) responseBody.attachment.workstream.workstream_id = 'ws:foreign';
  await assert.rejects(
    () => client.recipientResolve({ ...structuredClone(authority), recipient_ref: resolution.recipient_ref }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:scope_mismatch'
  );

  responseBody = { ...structuredClone(resolution), schema: 'invented' };
  await assert.rejects(
    () => client.recipientResolve({ ...structuredClone(authority), recipient_ref: resolution.recipient_ref }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:schema'
  );

  responseBody = { ...structuredClone(resolution), recipient_ref: '' };
  await assert.rejects(
    () => client.recipientResolve({ ...structuredClone(authority), recipient_ref: '' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:recipient_ref'
  );

  responseBody = { ...structuredClone(resolution), routable: false };
  await assert.rejects(
    () => client.recipientResolve({ ...structuredClone(authority), recipient_ref: resolution.recipient_ref }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:not_routable'
  );
}

async function exerciseRecompositionEvidenceGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  assert.ok(authority.attachment, 'recomposition.evidence.get fixture requires exact attachment authority');
  const evidence = {
    ...structuredClone(authority),
    attachment: structuredClone(authority.attachment),
    candidate_contribution_ids: ['contribution:primary', 'contribution:queue'],
    diagnostic_refs: [],
    eligibility_decisions: [],
    evidence_id: 'recomposition-evidence:key:11',
    input_projection_digest: null,
    layout_decision_refs: [],
    observed_at: new Date().toISOString(),
    output_projection_digest: `sha256:${'e'.repeat(64)}`,
    rule_revision: 'resolver:v2',
    trigger: 'explicit_resolve'
  };
  let responseBody = structuredClone(evidence);
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  const client = new MissionCanvasClient(transport);
  const result = await client.recompositionEvidenceGet({ ...structuredClone(authority), revision: 11 });
  assert.equal(result.evidence_id, evidence.evidence_id);
  assert.equal(result.trigger, 'explicit_resolve');
  assert.ok(result.candidate_contribution_ids.length >= 2);

  responseBody = structuredClone(evidence);
  responseBody.workstream.workstream_id = 'ws:foreign';
  if (responseBody.attachment) responseBody.attachment.workstream.workstream_id = 'ws:foreign';
  await assert.rejects(
    () => client.recompositionEvidenceGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:scope_mismatch'
  );

  responseBody = { ...structuredClone(evidence), evidence_id: '' };
  await assert.rejects(
    () => client.recompositionEvidenceGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:evidence_id'
  );

  responseBody = { ...structuredClone(evidence), output_projection_digest: 'not-a-digest' };
  await assert.rejects(
    () => client.recompositionEvidenceGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:output_projection_digest'
  );

  responseBody = { ...structuredClone(evidence), trigger: 'invented' };
  await assert.rejects(
    () => client.recompositionEvidenceGet({ ...structuredClone(authority), revision: 11 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:trigger'
  );
}

async function exerciseDraftSync({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  assert.ok(authority.attachment, 'draft.sync fixture requires exact attachment authority');
  const draft = {
    ...structuredClone(authority),
    attachment: structuredClone(authority.attachment),
    content: '# Draft\nCanonical draft content',
    content_sha256: `sha256:${'d'.repeat(64)}`,
    draft_id: 'draft:prompt',
    draft_revision: 4,
    idempotency_key: 'idem:draft:sync:1',
    owner: 'canvas_prompt_editor',
    recipient_ref: 'recipient:pi-session',
    selection_start: 2,
    selection_end: 5,
    sync_state: 'synchronized',
    updated_at: new Date().toISOString()
  };
  let responseBody = structuredClone(draft);
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  const client = new MissionCanvasClient(transport);
  const result = await client.draftSync(structuredClone(draft));
  assert.equal(result.draft_id, 'draft:prompt');
  assert.equal(result.draft_revision, 4);
  assert.equal(result.sync_state, 'synchronized');
  assert.equal(result.idempotency_key, 'idem:draft:sync:1');

  responseBody = structuredClone(draft);
  responseBody.workstream.workstream_id = 'ws:foreign';
  if (responseBody.attachment) responseBody.attachment.workstream.workstream_id = 'ws:foreign';
  await assert.rejects(
    () => client.draftSync(structuredClone(draft)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:scope_mismatch'
  );

  responseBody = { ...structuredClone(draft), draft_id: '' };
  await assert.rejects(
    () => client.draftSync({ ...structuredClone(draft), draft_id: '' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:draft_id'
  );

  await assert.rejects(
    () => client.draftSync({ ...structuredClone(draft), draft_revision: -1 }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'if_match_revision_required'
  );

  responseBody = { ...structuredClone(draft), draft_revision: -1 };
  await assert.rejects(
    () => client.draftSync(structuredClone(draft)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:draft_revision'
  );

  await assert.rejects(
    () => client.draftSync({ ...structuredClone(draft), sync_state: 'invented' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_request:invalid:sync_state'
  );

  responseBody = { ...structuredClone(draft), sync_state: 'invented' };
  await assert.rejects(
    () => client.draftSync(structuredClone(draft)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:sync_state'
  );
}

async function exerciseDraftGet({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  assert.ok(authority.attachment, 'draft.get fixture requires exact attachment authority');
  const draft = {
    ...structuredClone(authority),
    attachment: structuredClone(authority.attachment),
    content: '# Draft\nCanonical draft content',
    content_sha256: `sha256:${'d'.repeat(64)}`,
    draft_id: 'draft:prompt',
    draft_revision: 4,
    idempotency_key: 'idem:draft:1',
    owner: 'canvas_prompt_editor',
    recipient_ref: 'recipient:pi-session',
    selection_start: 2,
    selection_end: 5,
    sync_state: 'synchronized',
    updated_at: new Date().toISOString()
  };
  let responseBody = structuredClone(draft);
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(responseBody), { status: 200 })
  );
  const client = new MissionCanvasClient(transport);
  const result = await client.draftGet({ ...structuredClone(authority), draft_id: 'draft:prompt' });
  assert.equal(result.draft_id, 'draft:prompt');
  assert.equal(result.draft_revision, 4);
  assert.equal(result.sync_state, 'synchronized');

  responseBody = structuredClone(draft);
  responseBody.workstream.workstream_id = 'ws:foreign';
  if (responseBody.attachment) responseBody.attachment.workstream.workstream_id = 'ws:foreign';
  await assert.rejects(
    () => client.draftGet({ ...structuredClone(authority), draft_id: 'draft:foreign' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:scope_mismatch'
  );

  responseBody = { ...structuredClone(draft), draft_id: '' };
  await assert.rejects(
    () => client.draftGet({ ...structuredClone(authority), draft_id: 'draft:missing-id' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:draft_id'
  );

  responseBody = { ...structuredClone(draft), draft_revision: -1 };
  await assert.rejects(
    () => client.draftGet({ ...structuredClone(authority), draft_id: 'draft:bad-rev' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:draft_revision'
  );

  responseBody = { ...structuredClone(draft), sync_state: 'invented' };
  await assert.rejects(
    () => client.draftGet({ ...structuredClone(authority), draft_id: 'draft:bad-sync' }),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:invalid:sync_state'
  );
}

async function exerciseHostClose({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const rendererResolution = {
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
  const lifecycle = {
    ...structuredClone(authority),
    host_instance_id: 'rich-host:desktop:mission-canvas',
    renderer_resolution: rendererResolution,
    state: 'closing',
    focused: false,
    process_id: null,
    window_id: 'window:mission-canvas',
    pi_draft_ref: 'draft:pi',
    canvas_draft_ref: 'draft:canvas',
    last_error_ref: null,
    durable_event_cursor: 'event:41',
    lifecycle_revision: 2,
    updated_at: '2026-08-07T00:00:00Z'
  };

  let calls = 0;
  const requests = [];
  let response = structuredClone(lifecycle);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = {
    confirmation: 'confirm',
    ...structuredClone(authority),
    idempotency_key: 'idempotency:rich-host:close'
  };
  const hidden = await client.rich_hostClose(input);
  assert.deepEqual(hidden, lifecycle);
  assert.equal(hidden.state, 'closing');
  assert.equal(hidden.focused, false);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/rich-host/close');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:host');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], 'mission_canvas.desktop_tauri');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.continuity_id, authority.continuity_id);
  assert.equal(body.workspace_binding_id, authority.workspace_binding_id);
  assert.deepEqual(body.runtime_object, authority.runtime_object);
  assert.equal(body.work_surface_id, authority.work_surface_id);
  assert.equal(body.idempotency_key, input.idempotency_key);
  for (const field of ['activity_mode_id', 'workspace_profile_id', 'projection_revision', 'layout_revision', 'state', 'renderer_resolution']) {
    assert.equal(field in body, false, `hide must not invent local composition output: ${field}`);
  }
  assert.equal('eligible_contributions' in hidden, false, 'hide must not invent composition output');

  let missingIdempotencyCalls = calls;
  await assert.rejects(
    () => client.rich_hostClose(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing idempotency must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).rich_hostClose({ idempotency_key: 'idempotency:missing-scope' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  // A foreign hide response cannot be adopted from another Workstream.
  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(lifecycle);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_lifecycle_scope'
  );

  const foreignRendererTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreignRenderer = structuredClone(lifecycle);
      foreignRenderer.renderer_resolution.selected_renderer = 'focusa_pi_rich_window';
      return new Response(JSON.stringify(foreignRenderer), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignRendererTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:close_renderer'
  );

  const missingRendererScope = structuredClone(lifecycle);
  delete missingRendererScope.renderer_resolution.workstream;
  const missingRendererScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify(missingRendererScope), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingRendererScopeTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:renderer_resolution_workstream'
  );

  const wrongState = structuredClone(lifecycle);
  wrongState.state = 'visible';
  wrongState.focused = true;
  const wrongStateTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(wrongState), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrongStateTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:close_state'
  );

  const invalidHostId = structuredClone(lifecycle);
  invalidHostId.host_instance_id = 'host-invented';
  const invalidHostTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(invalidHostId), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidHostTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:host_instance_id'
  );

  // A structurally valid but stale hide response is not adopted.
  response = structuredClone(lifecycle);
  response.lifecycle_revision = 1;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_revision'
  );
  response.lifecycle_revision = 3;
  await assert.rejects(
    () => client.rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_cursor'
  );

  const noCapabilityTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(noCapabilityTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const noPermissionTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Permissions'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
    },
    undefined,
    30_000,
    [],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(noPermissionTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
      assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri']
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).rich_hostClose(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  console.log('Mission Canvas operation consumer: PASS (generated rich_hostClose, exact Workstream POST, presentation hidden without changing runtime/surface/drafts, foreign authority, missing authority, capability/permission denial, stale lifecycle, and hostile response checks)');
}


async function exerciseHostHide({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority }) {
  const rendererResolution = {
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
  const lifecycle = {
    ...structuredClone(authority),
    host_instance_id: 'rich-host:desktop:mission-canvas',
    renderer_resolution: rendererResolution,
    state: 'hidden',
    focused: false,
    process_id: null,
    window_id: 'window:mission-canvas',
    pi_draft_ref: 'draft:pi',
    canvas_draft_ref: 'draft:canvas',
    last_error_ref: null,
    durable_event_cursor: 'event:41',
    lifecycle_revision: 2,
    updated_at: '2026-08-07T00:00:00Z'
  };

  let calls = 0;
  const requests = [];
  let response = structuredClone(lifecycle);
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const input = {
    ...structuredClone(authority),
    idempotency_key: 'idempotency:rich-host:hide'
  };
  const hidden = await client.rich_hostHide(input);
  assert.deepEqual(hidden, lifecycle);
  assert.equal(hidden.state, 'hidden');
  assert.equal(hidden.focused, false);
  assert.equal(calls, 1);

  const requestUrl = new URL(requests[0].url);
  assert.equal(`${requestUrl.origin}${requestUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/rich-host/hide');
  assert.equal(requests[0].init.method, 'POST');
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:host');
  assert.equal(requests[0].init.headers['X-Focusa-Capabilities'], 'mission_canvas.desktop_tauri');
  assert.equal(requests[0].init.headers['X-Focusa-Actor-Id'], 'actor:desktop');
  assert.equal(requests[0].init.headers['X-Focusa-Authority-Ref'], 'authority:desktop');
  const body = JSON.parse(requests[0].init.body);
  assert.deepEqual(body.workstream, authority.workstream);
  assert.deepEqual(body.attachment, authority.attachment);
  assert.equal(body.continuity_id, authority.continuity_id);
  assert.equal(body.workspace_binding_id, authority.workspace_binding_id);
  assert.deepEqual(body.runtime_object, authority.runtime_object);
  assert.equal(body.work_surface_id, authority.work_surface_id);
  assert.equal(body.idempotency_key, input.idempotency_key);
  for (const field of ['activity_mode_id', 'workspace_profile_id', 'projection_revision', 'layout_revision', 'state', 'renderer_resolution']) {
    assert.equal(field in body, false, `hide must not invent local composition output: ${field}`);
  }
  assert.equal('eligible_contributions' in hidden, false, 'hide must not invent composition output');

  let missingIdempotencyCalls = calls;
  await assert.rejects(
    () => client.rich_hostHide(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'idempotency_key_required'
  );
  assert.equal(calls, missingIdempotencyCalls, 'missing idempotency must fail before HTTP');

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response(JSON.stringify(response), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).rich_hostHide({ idempotency_key: 'idempotency:missing-scope' }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream authority must fail before HTTP');

  // A foreign hide response cannot be adopted from another Workstream.
  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = structuredClone(lifecycle);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.workstream.workstream_id = 'ws:foreign';
      foreign.renderer_resolution.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify(foreign), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_lifecycle_scope'
  );

  const foreignRendererTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreignRenderer = structuredClone(lifecycle);
      foreignRenderer.renderer_resolution.selected_renderer = 'focusa_pi_rich_window';
      return new Response(JSON.stringify(foreignRenderer), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignRendererTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:hide_renderer'
  );

  const missingRendererScope = structuredClone(lifecycle);
  delete missingRendererScope.renderer_resolution.workstream;
  const missingRendererScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify(missingRendererScope), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingRendererScopeTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:missing:renderer_resolution_workstream'
  );

  const wrongState = structuredClone(lifecycle);
  wrongState.state = 'visible';
  wrongState.focused = true;
  const wrongStateTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(wrongState), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(wrongStateTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:hide_state'
  );

  const invalidHostId = structuredClone(lifecycle);
  invalidHostId.host_instance_id = 'host-invented';
  const invalidHostTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response(JSON.stringify(invalidHostId), { status: 200 })
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidHostTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'invalid_response:host_instance_id'
  );

  // A structurally valid but stale hide response is not adopted.
  response = structuredClone(lifecycle);
  response.lifecycle_revision = 1;
  response.durable_event_cursor = 'event:40';
  await assert.rejects(
    () => client.rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_revision'
  );
  response.lifecycle_revision = 3;
  await assert.rejects(
    () => client.rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_lifecycle_cursor'
  );

  const noCapabilityTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Capabilities'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'capability_unavailable' } }), { status: 403 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(noCapabilityTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const noPermissionTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Permissions'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'permission_denied' } }), { status: 403 });
    },
    undefined,
    30_000,
    [],
    ['mission_canvas.desktop_tauri'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(noPermissionTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 403
  );

  const missingAuthorityTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (_url, init) => {
      assert.equal(init.headers['X-Focusa-Actor-Id'], undefined);
      assert.equal(init.headers['X-Focusa-Authority-Ref'], undefined);
      return new Response(JSON.stringify({ schema: 'focusa.tool_result.v1', status: 'blocked', error: { code: 'workstream_context_invalid' } }), { status: 422 });
    },
    undefined,
    30_000,
    ['mission_canvas:host'],
    ['mission_canvas.desktop_tauri']
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingAuthorityTransport).rich_hostHide(input),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'transport_response_failed' && error.status === 422
  );

  console.log('Mission Canvas operation consumer: PASS (generated rich_hostHide, exact Workstream POST, presentation hidden without changing runtime/surface/drafts, foreign authority, missing authority, capability/permission denial, stale lifecycle, and hostile response checks)');
}

async function exerciseEventsStream({ MissionCanvasClient, MissionCanvasHttpTransport, MissionCanvasTransportError, authority, server }) {
  const event = (sequence, overrides = {}) => ({
    event_id: `projection-event:stream:${sequence}`,
    event_kind: 'projection_resolved',
    ...structuredClone(authority),
    projection_revision: sequence,
    layout_revision: sequence,
    event_cursor: `event:${sequence}`,
    occurred_at: `2026-08-06T00:00:0${sequence}Z`,
    payload_ref: `mission-canvas:composition-event:${sequence}`,
    evidence_refs: [],
    receipt_refs: [],
    ...overrides
  });

  let calls = 0;
  const requests = [];
  let response = [event(1), event(2)];
  const transport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787/',
    async (url, init) => {
      calls += 1;
      requests.push({ url: String(url), init });
      return new Response(JSON.stringify(response), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.events.stream'],
    'actor:desktop',
    'authority:desktop'
  );
  const client = new MissionCanvasClient(transport);
  const first = await client.eventsStream(structuredClone(authority));
  assert.deepEqual(first, [event(1), event(2)]);
  assert.equal(calls, 1);
  const firstUrl = new URL(requests[0].url);
  assert.equal(`${firstUrl.origin}${firstUrl.pathname}`, 'http://127.0.0.1:8787/v1/mission-canvas/events');
  assert.deepEqual(JSON.parse(firstUrl.searchParams.get('workstream')), authority.workstream);
  assert.equal(firstUrl.searchParams.get('after_cursor'), null);
  assert.equal(requests[0].init.method, 'GET');
  assert.equal(requests[0].init.body, undefined);
  assert.equal(requests[0].init.headers['X-Focusa-Permissions'], 'mission_canvas:read');

  // The Desktop event client performs the replay-then-tail cursor handshake
  // over the generated method, while retaining exact Workstream authority.
  const { MissionCanvasEventClient } = await server.ssrLoadModule('/src/lib/mission-canvas/event-client.ts');
  const cursorWrites = [];
  const cursorStore = {
    load: () => undefined,
    persist: (_scope, cursor) => cursorWrites.push(cursor)
  };
  const replayThenTail = [];
  let tailResponse = [event(1), event(2)];
  let tailCalls = 0;
  const generatedStreamClient = {
    eventsStream: async (input) => {
      replayThenTail.push(input.after_cursor);
      const next = tailResponse;
      tailResponse = tailCalls === 0
        ? [event(3)]
        : [event(4, { projection_revision: 1, layout_revision: 1 })];
      tailCalls += 1;
      return structuredClone(next);
    }
  };
  const eventClient = new MissionCanvasEventClient(generatedStreamClient, authority, cursorStore);
  const replay = await eventClient.poll();
  assert.equal(replay.accepted.length, 2);
  assert.deepEqual(replayThenTail, [undefined]);
  assert.equal(cursorWrites.at(-1), 'event:2');
  const tail = await eventClient.poll();
  assert.equal(tail.accepted.length, 1);
  assert.equal(tail.accepted[0].event_cursor, 'event:3');
  assert.deepEqual(replayThenTail, [undefined, 'event:2']);
  assert.equal(cursorWrites.at(-1), 'event:3');
  const regressed = await eventClient.poll();
  assert.equal(regressed.accepted.length, 0);
  assert.equal(regressed.rejected[0].reason, 'projection_revision_regressed');
  assert.equal(cursorWrites.at(-1), 'event:3');

  // A foreign response is rejected at the generated transport boundary, not
  // adopted from a tab, latest record, or caller-provided project path.
  const foreignTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      const foreign = event(4);
      foreign.workstream.workstream_id = 'ws:foreign';
      foreign.attachment.workstream.workstream_id = 'ws:foreign';
      return new Response(JSON.stringify([foreign]), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.events.stream'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(foreignTransport).eventsStream(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_event_scope'
  );

  let missingScopeCalls = 0;
  const missingScopeTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      missingScopeCalls += 1;
      return new Response('[]', { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.events.stream'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingScopeTransport).eventsStream({}),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(missingScopeCalls, 0, 'missing Workstream must fail before HTTP');

  // Empty replay is a valid tail state; the consumer must not invent a panel,
  // event, or contribution when the exact Workstream has no events.
  const emptyTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response('[]', { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.events.stream'],
    'actor:desktop',
    'authority:desktop'
  );
  assert.deepEqual(await new MissionCanvasClient(emptyTransport).eventsStream(structuredClone(authority)), []);

  const staleStreamClient = {
    eventsStream: async () => [event(5, { event_cursor: 'event:2', projection_revision: 5, layout_revision: 5 })]
  };
  const staleEventClient = new MissionCanvasEventClient(staleStreamClient, authority, {
    load: () => 'event:3',
    persist: () => { throw new Error('stale cursor must not be persisted'); }
  });
  const stale = await staleEventClient.poll();
  assert.equal(stale.accepted.length, 0);
  assert.equal(stale.rejected[0].reason, 'event_cursor_regressed');

  const invalidEvent = event(6);
  delete invalidEvent.event_cursor;
  const invalidResponseTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => new Response(JSON.stringify([invalidEvent]), { status: 200 }),
    undefined,
    30_000,
    ['mission_canvas:read'],
    ['mission_canvas.events.stream'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(invalidResponseTransport).eventsStream(structuredClone(authority)),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );

  console.log('Mission Canvas operation consumer: PASS (generated client, exact Workstream replay/tail cursor, foreign scope, missing authority, stale cursor, and empty-tail hostile checks)');
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
