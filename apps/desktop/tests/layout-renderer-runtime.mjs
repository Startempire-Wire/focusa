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

  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
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

  console.log('Mission Canvas runtime: PASS (layouts, registry controls, renderer gate, projection and draft authority)');
} finally {
  await server.close();
}
