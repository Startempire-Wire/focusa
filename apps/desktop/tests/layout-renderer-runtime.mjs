import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createServer } from 'vite';

const server = await createServer({
  appType: 'custom',
  server: { middlewareMode: true },
  logLevel: 'error'
});

try {
  const { render } = await server.ssrLoadModule('svelte/server');
  const { default: Harness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasLayoutHarness.svelte');
  const { default: RegistryControlsHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasRegistryControlsHarness.svelte');
  const { default: TrustedRegistryHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasTrustedRegistryHarness.svelte');
  const { default: CustomElementHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasCustomElementHarness.svelte');
  const expectations = {
    single: ['layout-single'],
    split: ['layout-split'],
    stack: ['layout-stack'],
    grid: ['layout-grid'],
    tabs: ['layout-tabs', 'role="tablist"'],
    inspector: ['layout-inspector', '<aside']
  };

  for (const [variant, markers] of Object.entries(expectations)) {
    const { body } = render(Harness, { props: { variant } });
    for (const marker of markers) assert.match(body, new RegExp(marker), `${variant} missing ${marker}`);
    assert.doesNotMatch(body, /undefined|null/, `${variant} rendered unresolved data`);
  }

  const { body: populated } = render(Harness, { props: { variant: 'populated' } });
  assert.match(populated, /data-rendered-contribution="contribution:pi-session"/);
  assert.match(populated, /data-rendered-contribution="contribution:focusa-inspector"/);
  assert.doesNotMatch(populated, /contribution:empty-work-rail/);

  const { body: controls } = render(RegistryControlsHarness);
  assert.match(controls, /data-activity-mode-id="activity:overview"/);
  assert.match(controls, /data-activity-mode-id="activity:tasks"/);
  assert.match(controls, /aria-current="page"[^>]*data-activity-mode-id="activity:tasks"/);
  assert.match(controls, /<option value="profile:software"[^>]*selected="">/);
  const { body: emptyControls } = render(RegistryControlsHarness, { props: { empty: true } });
  assert.doesNotMatch(emptyControls, /<nav|<select/);

  const { body: blockedRegistry } = render(TrustedRegistryHarness);
  assert.match(blockedRegistry, /role="alert"/);
  assert.match(blockedRegistry, /data-unavailable-renderer="renderer:focusa-inspector@v1"/);
  assert.doesNotMatch(blockedRegistry, /data-trusted-renderer=/);
  const { body: completeRegistry } = render(TrustedRegistryHarness, { props: { complete: true } });
  assert.doesNotMatch(completeRegistry, /role="alert"/);
  assert.match(completeRegistry, /data-trusted-renderer="renderer:pi-session@v1"/);
  assert.match(completeRegistry, /data-trusted-renderer="renderer:focusa-inspector@v1"/);
  const { body: customElements } = render(CustomElementHarness);
  assert.match(customElements, /<fixture-pi-session[^>]*data-contribution-id="contribution:pi-session"/);
  assert.match(customElements, /<fixture-focusa-inspector[^>]*data-contribution-id="contribution:focusa-inspector"/);

  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  const requestedUrls = [];
  const { MissionCanvasHttpTransport, MissionCanvasTransportError } = await server.ssrLoadModule('/src/lib/mission-canvas/http-transport.ts');
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787/', async (url) => {
    requestedUrls.push(String(url));
    return new Response(JSON.stringify(fixture), { status: 200, headers: { 'Content-Type': 'application/json' } });
  });
  const transported = await transport.request('focusa.mission_canvas.projection.get', { scope: fixture.scope });
  assert.equal(transported.projection_digest, fixture.projection_digest);
  assert.match(requestedUrls[0], /^http:\/\/127\.0\.0\.1:8787\/v1\/mission-canvas\/projection\?project_root=/);
  assert.match(requestedUrls[0], /attachment_id=/);
  await assert.rejects(
    () => transport.request('focusa.mission_canvas.not-generated'),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );
  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.get', { profile_id: 'software', scope: fixture.scope }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );
  assert.match(requestedUrls.at(-1), /\/v1\/mission-canvas\/profiles\/software\?project_root=/);
  assert.doesNotMatch(requestedUrls.at(-1), /profile_id=/);
  const arrayTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response('[]', { status: 200, headers: { 'Content-Type': 'application/json' } })
  );
  assert.deepEqual(await arrayTransport.request('focusa.mission_canvas.activity.list', { scope: fixture.scope }), []);

  const { MissionCanvasProjectionController } = await server.ssrLoadModule('/src/lib/mission-canvas/projection-controller.svelte.ts');
  let response = structuredClone(fixture);
  const controller = new MissionCanvasProjectionController(async () => structuredClone(response));
  await controller.load(fixture.scope);
  assert.equal(controller.state.kind, 'ready');

  response = structuredClone(fixture);
  response.scope.session_id = 'foreign-session';
  await controller.load(fixture.scope);
  assert.equal(controller.state.kind, 'blocked');
  assert.equal(controller.state.reason, 'projection_scope_mismatch');

  response = structuredClone(fixture);
  await controller.load(fixture.scope);
  response.projection_revision -= 1;
  await controller.load(fixture.scope);
  assert.equal(controller.state.kind, 'stale');
  assert.equal(controller.state.reason, 'projection_revision_regressed');

  let refreshResolver;
  let refreshResponse = structuredClone(fixture);
  const preservingController = new MissionCanvasProjectionController(async () => {
    if (!refreshResolver) return structuredClone(refreshResponse);
    return new Promise((resolve) => { refreshResolver = resolve; });
  });
  await preservingController.load(fixture.scope);
  refreshResolver = () => {};
  const refreshing = preservingController.load(fixture.scope);
  assert.equal(preservingController.state.kind, 'refreshing');
  assert.equal(preservingController.state.projection.projection_digest, fixture.projection_digest);
  refreshResolver(structuredClone({ ...fixture, projection_revision: fixture.projection_revision + 1 }));
  await refreshing;
  assert.equal(preservingController.state.kind, 'ready');

  const draftFixture = {
    attachment_id: fixture.scope.attachment_id,
    content: 'canonical draft',
    content_sha256: `sha256:${'0'.repeat(64)}`,
    draft_id: 'draft:fixture',
    draft_revision: 4,
    idempotency_key: 'fixture-draft-v4',
    owner: 'canvas_prompt_editor',
    recipient_ref: 'recipient:pi',
    scope: structuredClone(fixture.scope),
    sync_state: 'synchronized',
    updated_at: '2026-08-04T00:00:00Z'
  };
  const binding = { scope: fixture.scope, attachmentId: fixture.scope.attachment_id, recipientRef: 'recipient:pi' };
  let draftResponse = structuredClone(draftFixture);
  const { MissionCanvasDraftController } = await server.ssrLoadModule('/src/lib/mission-canvas/draft-controller.svelte.ts');
  const draftController = new MissionCanvasDraftController({
    get: async () => structuredClone(draftResponse),
    sync: async () => structuredClone(draftResponse)
  });
  await draftController.load(binding);
  assert.equal(draftController.state.kind, 'ready');
  draftResponse.scope.session_id = 'foreign-session';
  await draftController.sync('preserve this local edit');
  assert.equal(draftController.state.kind, 'conflict');
  assert.equal(draftController.state.reason, 'foreign_draft_binding');
  assert.equal(draftController.state.localContent, 'preserve this local edit');

  const eventFixture = {
    event_cursor: 'cursor:1',
    event_id: 'event:1',
    event_kind: 'capability_changed',
    evidence_refs: [],
    layout_revision: fixture.layout_revision + 1,
    occurred_at: '2026-08-04T00:00:01Z',
    payload_ref: 'fixture:event:1',
    projection_revision: fixture.projection_revision + 1,
    receipt_refs: [],
    scope: structuredClone(fixture.scope)
  };
  let eventResponse = [eventFixture];
  let persistedCursor;
  const { MissionCanvasEventClient } = await server.ssrLoadModule('/src/lib/mission-canvas/event-client.ts');
  const eventClient = new MissionCanvasEventClient(
    { eventsStream: async () => structuredClone(eventResponse) },
    fixture.scope,
    { load: () => persistedCursor, persist: (_scope, cursor) => { persistedCursor = cursor; } }
  );
  const acceptedEvents = await eventClient.poll();
  assert.equal(acceptedEvents.accepted.length, 1);
  assert.equal(persistedCursor, 'cursor:1');
  eventResponse = [{ ...eventFixture, event_id: 'event:foreign', event_cursor: 'cursor:2', scope: { ...fixture.scope, session_id: 'foreign' } }];
  const rejectedEvents = await eventClient.poll();
  assert.equal(rejectedEvents.rejected[0].reason, 'foreign_event_scope');
  assert.equal(persistedCursor, 'cursor:1');

  let reloads = 0;
  const { MissionCanvasInvalidationController } = await server.ssrLoadModule('/src/lib/mission-canvas/invalidation-controller.ts');
  const invalidations = new MissionCanvasInvalidationController(() => { reloads += 1; }, 1000);
  assert.equal(invalidations.enqueue(acceptedEvents, { projectionRevision: fixture.projection_revision, layoutRevision: fixture.layout_revision }), true);
  invalidations.enqueue(acceptedEvents, { projectionRevision: fixture.projection_revision, layoutRevision: fixture.layout_revision });
  await invalidations.flush();
  assert.equal(reloads, 1);

  console.log('Mission Canvas runtime: PASS (layout, renderer, transport, projection, draft and event authority)');
} finally {
  await server.close();
}
