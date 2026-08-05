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

  console.log('Mission Canvas recursive layout runtime: PASS (6 variants, omission, scope and stale rejection)');
} finally {
  await server.close();
}
