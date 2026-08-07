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
  const { trustedGeneratedSurfaceRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/generated-surface-renderer.ts');
  assert.throws(() => trustedGeneratedSurfaceRenderer({ rendererBindingId: '', semanticBindingIds: [], snapshotResolver: async () => [] }));
  const generatedEntry = trustedGeneratedSurfaceRenderer({
    rendererBindingId: 'renderer:fixture-generated@v1',
    semanticBindingIds: ['semantic:fixture-generated'],
    snapshotResolver: async () => []
  });
  assert.equal(generatedEntry.rendererBindingId, 'renderer:fixture-generated@v1');
  assert.deepEqual(generatedEntry.contributionKinds, ['generated_surface']);

  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  const authorityOf = (value) => ({
    workstream: structuredClone(value.workstream),
    continuity_id: value.continuity_id ?? null,
    attachment: structuredClone(value.attachment ?? null),
    workspace_binding_id: value.workspace_binding_id ?? null,
    runtime_object: structuredClone(value.runtime_object ?? null),
    work_surface_id: value.work_surface_id ?? value.focused_work_surface_id ?? null
  });
  const fixtureAuthority = authorityOf(fixture);
  const { validateLayoutIntegrity } = await server.ssrLoadModule('/src/lib/mission-canvas/layout-references.ts');
  assert.deepEqual(validateLayoutIntegrity({
    kind: 'tabs',
    node_id: 'layout:invalid-tabs',
    contribution_ids: ['contribution:a'],
    active_contribution_id: 'contribution:foreign'
  }).map((issue) => issue.code), ['invalid_active_tab']);
  assert.deepEqual(validateLayoutIntegrity({
    kind: 'split',
    node_id: 'layout:duplicate-root',
    direction: 'horizontal',
    ratio: 0.5,
    children: [
      { kind: 'single', node_id: 'layout:duplicate-a', contribution_id: 'contribution:a' },
      { kind: 'single', node_id: 'layout:duplicate-b', contribution_id: 'contribution:a' }
    ]
  }).map((issue) => issue.code), ['duplicate_contribution']);
  const { DEFAULT_CONTRIBUTION_REGISTRY } = await server.ssrLoadModule('/src/lib/mission-canvas/default-contribution-registry.ts');
  for (const contribution of fixture.eligible_contributions) {
    assert.ok(DEFAULT_CONTRIBUTION_REGISTRY.resolve(contribution), `default registry missing ${contribution.renderer_binding_id}`);
  }
  const queueFixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/one-queue-projection.json', import.meta.url), 'utf8'));
  const previewPrompt = queueFixture.eligible_contributions.find((contribution) => contribution.kind === 'prompt_editor');
  assert.equal(previewPrompt.data_ref.kind, 'canvas_draft');
  assert.equal(queueFixture.operation_bindings[0].operation_id, 'focusa.agent_execution.prompt');
  assert.equal(queueFixture.operation_bindings[0].enabled, true);
  for (const contribution of queueFixture.eligible_contributions) {
    assert.ok(DEFAULT_CONTRIBUTION_REGISTRY.resolve(contribution), `default registry missing ${contribution.renderer_binding_id}`);
  }
  const { default: ActivityNavigation } = await server.ssrLoadModule('/src/lib/mission-canvas/ActivityNavigation.svelte');
  const { body: activityNavigation } = render(ActivityNavigation, {
    props: {
      activities: [
        { activity_mode_id: 'overview', display_name: 'Overview', revision: 1, candidate_contribution_ids: [], viability_rule_revision: '1' },
        { activity_mode_id: 'evidence', display_name: 'Evidence', revision: 1, candidate_contribution_ids: [], viability_rule_revision: '1' }
      ],
      activeActivityModeId: 'evidence',
      onSelect: () => undefined
    }
  });
  assert.match(activityNavigation, /aria-label="Activities"/);
  assert.match(activityNavigation, /data-activity-mode-id="overview"/);
  assert.match(activityNavigation, /aria-current="page" data-activity-mode-id="evidence"/);

  const { default: MissionCanvasRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/MissionCanvasRenderer.svelte');
  const { body: productionProjection } = render(MissionCanvasRenderer, { props: { projection: fixture, registry: DEFAULT_CONTRIBUTION_REGISTRY } });
  assert.match(productionProjection, /aria-label="Mission Canvas context"/);
  assert.match(productionProjection, />Profile</);
  assert.match(productionProjection, />Activity</);
  assert.match(productionProjection, /aria-label="Work Surfaces"/);
  assert.match(productionProjection, /data-work-surface-id="surface:pi"/);
  assert.match(productionProjection, /data-work-surface-ref="surface:pi"/);
  assert.doesNotMatch(productionProjection, /Renderer unavailable/);

  const twoQueueFixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/two-queue-projection.json', import.meta.url), 'utf8'));
  assert.deepEqual(
    twoQueueFixture.eligible_contributions.filter(({ kind }) => kind.endsWith('_queue')).map(({ kind }) => kind),
    ['steering_queue', 'follow_up_queue']
  );
  assert.deepEqual(validateLayoutIntegrity(twoQueueFixture.layout_tree), []);
  for (const contribution of twoQueueFixture.eligible_contributions) {
    assert.ok(DEFAULT_CONTRIBUTION_REGISTRY.resolve(contribution), `two-queue registry missing ${contribution.renderer_binding_id}`);
  }
  const { body: twoQueueProjection } = render(MissionCanvasRenderer, { props: { projection: twoQueueFixture, registry: DEFAULT_CONTRIBUTION_REGISTRY } });
  assert.match(twoQueueProjection, /data-work-rail-ref="work-rail:project"/);
  assert.match(twoQueueProjection, /data-queue-kind="steering_queue"/);
  assert.match(twoQueueProjection, /data-queue-kind="follow_up_queue"/);
  assert.match(twoQueueProjection, />Steering Queue</);
  assert.match(twoQueueProjection, />Follow-up Queue</);
  assert.match(twoQueueProjection, /queue-region/);
  assert.doesNotMatch(twoQueueProjection, /Renderer unavailable/);

  const { default: WorkRailContribution } = await server.ssrLoadModule('/src/lib/mission-canvas/contributions/WorkRailContribution.svelte');
  const workRail = {
    ...twoQueueFixture.eligible_contributions.find(({ kind }) => kind === 'steering_queue'),
    contribution_id: 'contribution:work-rail',
    kind: 'work_rail',
    data_ref: { kind: 'work_rail', ref: 'work-rail:project', revision: 4, freshness: 'current' },
    operation_ids: [],
    accessibility: {
      focus_semantic_id: 'semantic:work-rail',
      label: 'Focusa Work Rail',
      description: 'Canonical project work for the focused Work Surface',
      landmark_role: 'region'
    }
  };
  const { body: workRailProjection } = render(WorkRailContribution, { props: { contribution: workRail, projection: twoQueueFixture } });
  assert.match(workRailProjection, /data-work-rail-ref="work-rail:project"/);
  assert.match(workRailProjection, />Focusa Work Rail</);
  assert.match(workRailProjection, /revision 4/);
  assert.doesNotMatch(workRailProjection, /New Workpoint/);
  const requestedUrls = [];
  const { MissionCanvasHttpTransport, MissionCanvasTransportError } = await server.ssrLoadModule('/src/lib/mission-canvas/http-transport.ts');
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787/', async (url) => {
    requestedUrls.push(String(url));
    return new Response(JSON.stringify(fixture), { status: 200, headers: { 'Content-Type': 'application/json' } });
  });
  const transported = await transport.request('focusa.mission_canvas.projection.get', { ...fixtureAuthority });
  assert.equal(transported.projection_digest, fixture.projection_digest);
  assert.match(requestedUrls[0], /^http:\/\/127\.0\.0\.1:8787\/v1\/mission-canvas\/projection\?workstream=/);
  assert.match(requestedUrls[0], /attachment=/);
  await assert.rejects(
    () => transport.request('focusa.mission_canvas.not-generated'),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );
  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.get', { profile_id: 'software', ...fixtureAuthority }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );
  assert.match(requestedUrls.at(-1), /\/v1\/mission-canvas\/profiles\/software\?workstream=/);
  assert.doesNotMatch(requestedUrls.at(-1), /profile_id=/);
  const arrayTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response('[]', { status: 200, headers: { 'Content-Type': 'application/json' } })
  );
  assert.deepEqual(await arrayTransport.request('focusa.mission_canvas.activity.list', { ...fixtureAuthority }), []);

  const { MissionCanvasProjectionController } = await server.ssrLoadModule('/src/lib/mission-canvas/projection-controller.svelte.ts');
  let response = structuredClone(fixture);
  const controller = new MissionCanvasProjectionController(async () => structuredClone(response));
  await controller.load(fixtureAuthority);
  assert.equal(controller.state.kind, 'ready');

  response = structuredClone(fixture);
  response.attachment.session_id = 'foreign-session';
  await controller.load(fixtureAuthority);
  assert.equal(controller.state.kind, 'blocked');
  assert.equal(controller.state.reason, 'projection_scope_mismatch');

  response = structuredClone(fixture);
  await controller.load(fixtureAuthority);
  response.projection_revision -= 1;
  await controller.load(fixtureAuthority);
  assert.equal(controller.state.kind, 'stale');
  assert.equal(controller.state.reason, 'projection_revision_regressed');

  let refreshResolver;
  let refreshResponse = structuredClone(fixture);
  const preservingController = new MissionCanvasProjectionController(async () => {
    if (!refreshResolver) return structuredClone(refreshResponse);
    return new Promise((resolve) => { refreshResolver = resolve; });
  });
  await preservingController.load(fixtureAuthority);
  refreshResolver = () => {};
  const refreshing = preservingController.load(fixtureAuthority);
  assert.equal(preservingController.state.kind, 'refreshing');
  assert.equal(preservingController.state.projection.projection_digest, fixture.projection_digest);
  refreshResolver(structuredClone({ ...fixture, projection_revision: fixture.projection_revision + 1 }));
  await refreshing;
  assert.equal(preservingController.state.kind, 'ready');

  const draftFixture = {
    ...structuredClone(fixtureAuthority),
    content: 'canonical draft',
    content_sha256: `sha256:${'0'.repeat(64)}`,
    draft_id: 'draft:fixture',
    draft_revision: 4,
    idempotency_key: 'fixture-draft-v4',
    owner: 'canvas_prompt_editor',
    recipient_ref: 'recipient:pi',
    sync_state: 'synchronized',
    updated_at: '2026-08-04T00:00:00Z'
  };
  const binding = { ...structuredClone(fixtureAuthority), draftId: draftFixture.draft_id, recipientRef: 'recipient:pi' };
  let draftResponse = structuredClone(draftFixture);
  const { MissionCanvasDraftController } = await server.ssrLoadModule('/src/lib/mission-canvas/draft-controller.svelte.ts');
  const draftController = new MissionCanvasDraftController({
    get: async () => structuredClone(draftResponse),
    sync: async () => structuredClone(draftResponse)
  });
  await draftController.load(binding);
  assert.equal(draftController.state.kind, 'ready');
  draftResponse.attachment.session_id = 'foreign-session';
  await draftController.sync('preserve this local edit');
  assert.equal(draftController.state.kind, 'conflict');
  assert.equal(draftController.state.reason, 'foreign_draft_binding');
  assert.equal(draftController.state.localContent, 'preserve this local edit');

  let synchronizedBody;
  const generatedClient = {
    draftGet: async (input) => {
      assert.equal(input.draft_id, draftFixture.draft_id);
      return structuredClone(draftFixture);
    },
    draftSync: async (body) => {
      synchronizedBody = structuredClone(body);
      return { ...structuredClone(body), draft_revision: body.draft_revision + 1, sync_state: 'synchronized' };
    },
    recipientResolve: async (input) => ({ schema: 'focusa.mission_canvas.recipient_resolution.v1', ...structuredClone(input), routable: true })
  };
  const { GeneratedDraftTransport, resolveRecipient } = await server.ssrLoadModule('/src/lib/mission-canvas/generated-draft-transport.ts');
  const generatedDraftTransport = new GeneratedDraftTransport(generatedClient);
  assert.equal((await generatedDraftTransport.get(binding)).draft_id, draftFixture.draft_id);
  await generatedDraftTransport.sync({
    ...binding,
    baseDraft: draftFixture,
    content: 'canonical prompt',
    expectedDraftRevision: draftFixture.draft_revision,
    idempotencyKey: 'idempotency:prompt-sync'
  });
  assert.equal(synchronizedBody.owner, 'canvas_prompt_editor');
  assert.equal(synchronizedBody.idempotency_key, 'idempotency:prompt-sync');
  assert.match(synchronizedBody.content_sha256, /^[a-f0-9]{64}$/);
  assert.equal((await resolveRecipient(generatedClient, fixtureAuthority, 'recipient:pi')).recipient_ref, 'recipient:pi');

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
    ...structuredClone(fixtureAuthority)
  };
  let eventResponse = [eventFixture];
  let persistedCursor;
  const { MissionCanvasEventClient } = await server.ssrLoadModule('/src/lib/mission-canvas/event-client.ts');
  const eventClient = new MissionCanvasEventClient(
    { eventsStream: async () => structuredClone(eventResponse) },
    fixtureAuthority,
    { load: () => persistedCursor, persist: (_authority, cursor) => { persistedCursor = cursor; } }
  );
  const acceptedEvents = await eventClient.poll();
  assert.equal(acceptedEvents.accepted.length, 1);
  assert.equal(persistedCursor, 'cursor:1');
  const foreignEvent = structuredClone(eventFixture);
  foreignEvent.event_id = 'event:foreign';
  foreignEvent.event_cursor = 'cursor:2';
  foreignEvent.workstream.workstream_id = 'ws:foreign';
  foreignEvent.attachment.workstream.workstream_id = 'ws:foreign';
  eventResponse = [foreignEvent];
  const rejectedEvents = await eventClient.poll();
  assert.equal(rejectedEvents.rejected[0].reason, 'foreign_event_scope');
  assert.equal(persistedCursor, 'cursor:1');

  // Hostile Desktop event-client cases stay on the generated transport path:
  // no local scope repair, cursor inference, or partial authority handoff.
  const immutableScope = structuredClone(fixtureAuthority);
  const immutableInputs = [];
  const immutableClient = new MissionCanvasEventClient(
    { eventsStream: async (input) => { immutableInputs.push(input); return []; } },
    immutableScope,
    { load: () => undefined, persist: () => { throw new Error('empty tail must not persist'); } }
  );
  immutableScope.workstream.workstream_id = 'ws:mutated-after-client-creation';
  await immutableClient.poll();
  assert.equal(immutableInputs[0].workstream.workstream_id, fixtureAuthority.workstream.workstream_id);

  let invalidScopeCalls = 0;
  const invalidScopeClient = new MissionCanvasEventClient(
    { eventsStream: async () => { invalidScopeCalls += 1; return []; } },
    {},
    { load: () => undefined, persist: () => undefined }
  );
  await assert.rejects(() => invalidScopeClient.poll(), /invalid_workstream_scope/);
  assert.equal(invalidScopeCalls, 0, 'invalid Workstream authority must fail before eventsStream');

  let invalidCursorCalls = 0;
  const invalidCursorClient = new MissionCanvasEventClient(
    { eventsStream: async () => { invalidCursorCalls += 1; return []; } },
    fixtureAuthority,
    { load: () => 'event:not-a-number', persist: () => undefined }
  );
  await assert.rejects(() => invalidCursorClient.poll(), /invalid_persisted_cursor/);
  assert.equal(invalidCursorCalls, 0, 'invalid durable cursor must fail before eventsStream');

  const malformedEventPayload = structuredClone(eventFixture);
  delete malformedEventPayload.event_cursor;
  const malformedEventClient = new MissionCanvasEventClient(
    { eventsStream: async () => [malformedEventPayload] },
    fixtureAuthority,
    { load: () => undefined, persist: () => { throw new Error('malformed event must not persist'); } }
  );
  const malformedEvents = await malformedEventClient.poll();
  assert.equal(malformedEvents.accepted.length, 0);
  assert.match(malformedEvents.rejected[0].reason, /invalid_event/);

  const duplicateEventClient = new MissionCanvasEventClient(
    { eventsStream: async () => [structuredClone(eventFixture), structuredClone(eventFixture)] },
    fixtureAuthority,
    { load: () => undefined, persist: (_scope, cursor) => assert.equal(cursor, 'cursor:1') }
  );
  const duplicateEvents = await duplicateEventClient.poll();
  assert.equal(duplicateEvents.accepted.length, 1);
  assert.equal(duplicateEvents.rejected[0].reason, 'duplicate_event');

  const foreignDirectEvent = structuredClone(eventFixture);
  foreignDirectEvent.event_id = 'event:foreign-direct-client';
  foreignDirectEvent.event_cursor = 'cursor:2';
  foreignDirectEvent.workstream.workstream_id = 'ws:foreign';
  foreignDirectEvent.attachment.workstream.workstream_id = 'ws:foreign';
  const foreignEventClient = new MissionCanvasEventClient(
    { eventsStream: async () => [foreignDirectEvent] },
    fixtureAuthority,
    { load: () => undefined, persist: () => { throw new Error('foreign event must not persist'); } }
  );
  const foreignEvents = await foreignEventClient.poll();
  assert.equal(foreignEvents.accepted.length, 0);
  assert.equal(foreignEvents.rejected[0].reason, 'foreign_event_scope');

  const unavailableEventClient = new MissionCanvasEventClient(
    {},
    fixtureAuthority,
    { load: () => undefined, persist: () => undefined }
  );
  await assert.rejects(() => unavailableEventClient.poll(), /operation_unavailable/);

  let persistAttempts = 0;
  let persistFailureInputs = [];
  const persistFailureClient = new MissionCanvasEventClient(
    { eventsStream: async (input) => {
      persistFailureInputs.push(input.after_cursor);
      return persistAttempts === 0 ? [structuredClone(eventFixture)] : [];
    } },
    fixtureAuthority,
    { load: () => undefined, persist: () => { persistAttempts += 1; throw new Error('storage offline'); } }
  );
  await assert.rejects(() => persistFailureClient.poll(), /event_cursor_persist_failed/);
  assert.equal(persistAttempts, 1);
  // A failed cursor write does not advance in-memory state; a retry remains a
  // replay from the old cursor rather than silently skipping an event.
  await persistFailureClient.poll();
  assert.deepEqual(persistFailureInputs, [undefined, undefined]);

  const {
    MissionCanvasInvalidationController,
    event: projectionEventClassifier
  } = await server.ssrLoadModule('/src/lib/mission-canvas/invalidation-controller.ts');
  const projectionRevision = {
    projectionRevision: fixture.projection_revision,
    layoutRevision: fixture.layout_revision,
    durableEventCursor: fixture.durable_event_cursor,
    authority: fixtureAuthority
  };
  const projectionEvent = {
    ...structuredClone(eventFixture),
    event_id: 'event:projection-refresh',
    event_cursor: 'event:42',
    projection_revision: fixture.projection_revision + 1,
    layout_revision: fixture.layout_revision + 1
  };
  const projectionBatch = { accepted: [projectionEvent], rejected: [], cursor: projectionEvent.event_cursor };

  const classified = projectionEventClassifier.classify(projectionEvent, projectionRevision, fixtureAuthority);
  assert.equal(classified.refresh, true);
  assert.equal(classified.reason, 'projection_revision_advanced');
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:stale-cursor', event_cursor: fixture.durable_event_cursor },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'stale_event_cursor'
  );
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:wrong-namespace', event_cursor: 'cursor:42' },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'event_cursor_namespace_mismatch'
  );
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:stale-revision', event_cursor: 'event:43', layout_revision: fixture.layout_revision - 1 },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'layout_revision_stale'
  );
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:routine-pi', event_cursor: 'event:44', event_kind: 'pi_message_updated', projection_revision: fixture.projection_revision + 2, layout_revision: fixture.layout_revision + 2 },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'event_not_projection_relevant'
  );

  const foreignProjectionEvent = structuredClone(projectionEvent);
  foreignProjectionEvent.event_id = 'event:foreign-invalidation';
  foreignProjectionEvent.event_cursor = 'event:45';
  foreignProjectionEvent.workstream.workstream_id = 'ws:foreign-invalidation';
  foreignProjectionEvent.attachment.workstream.workstream_id = 'ws:foreign-invalidation';
  assert.equal(
    projectionEventClassifier.classify(foreignProjectionEvent, projectionRevision, fixtureAuthority).reason,
    'foreign_event_scope'
  );

  const missingAuthorityInvalidation = new MissionCanvasInvalidationController(() => {
    throw new Error('missing authority must not refresh');
  }, 1000);
  assert.equal(
    missingAuthorityInvalidation.coalesce(projectionBatch, {
      projectionRevision: fixture.projection_revision,
      layoutRevision: fixture.layout_revision,
      durableEventCursor: fixture.durable_event_cursor
    }),
    false
  );
  await missingAuthorityInvalidation.flush();
  missingAuthorityInvalidation.dispose();

  // An omitted contribution remains a Core-owned composition decision.  The
  // event may refresh the canonical projection, but Desktop never creates a
  // replacement contribution or layout node.
  const omittedContributionEvent = structuredClone(projectionEvent);
  omittedContributionEvent.event_id = 'event:empty-omission';
  omittedContributionEvent.event_cursor = 'event:46';
  omittedContributionEvent.event_kind = 'contribution_omitted';
  delete omittedContributionEvent.contribution_id;
  assert.equal(
    projectionEventClassifier.classify(omittedContributionEvent, projectionRevision, fixtureAuthority).refresh,
    true
  );

  let reloads = 0;
  const invalidation = new MissionCanvasInvalidationController(() => { reloads += 1; }, 1000);
  assert.equal(invalidation.coalesce(projectionBatch, projectionRevision, fixtureAuthority), true);
  const secondProjectionEvent = {
    ...structuredClone(projectionEvent),
    event_id: 'event:projection-refresh-2',
    event_cursor: 'event:43',
    projection_revision: fixture.projection_revision + 2,
    layout_revision: fixture.layout_revision + 2
  };
  assert.equal(invalidation.coalesce({ accepted: [secondProjectionEvent], rejected: [] }, projectionRevision, fixtureAuthority), true);
  await invalidation.flush();
  assert.equal(reloads, 1, 'event bursts must cause one bounded refresh');
  assert.equal(invalidation.coalesce({ accepted: [{ ...projectionEvent, event_id: 'event:routine-only', event_kind: 'pi_tool_completed', event_cursor: 'event:47' }], rejected: [] }, projectionRevision, fixtureAuthority), false);
  invalidation.dispose();

  let serializedReloads = 0;
  let releaseReload;
  const serialReload = new Promise((resolve) => { releaseReload = resolve; });
  const serializedInvalidation = new MissionCanvasInvalidationController(async () => {
    serializedReloads += 1;
    if (serializedReloads === 1) await serialReload;
  }, 0);
  serializedInvalidation.coalesce(projectionBatch, projectionRevision, fixtureAuthority);
  const firstRefresh = serializedInvalidation.flush();
  serializedInvalidation.coalesce({ accepted: [secondProjectionEvent] }, projectionRevision, fixtureAuthority);
  assert.equal(serializedReloads, 1, 'refreshes must not overlap');
  releaseReload();
  await firstRefresh;
  await serializedInvalidation.flush();
  assert.equal(serializedReloads, 2, 'events received during refresh must be retained');
  serializedInvalidation.dispose();

  let disposedReloads = 0;
  const disposedInvalidation = new MissionCanvasInvalidationController(() => { disposedReloads += 1; }, 0);
  disposedInvalidation.coalesce(projectionBatch, projectionRevision, fixtureAuthority);
  disposedInvalidation.dispose();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(disposedReloads, 0, 'disposed invalidations must not refresh');

  console.log('Mission Canvas runtime: PASS (layout, renderer, transport, projection, draft, event authority and invalidation coalescing)');
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
}
