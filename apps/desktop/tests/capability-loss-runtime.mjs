import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

const desktopRoot = fileURLToPath(new URL('../', import.meta.url));
const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));
process.chdir(desktopRoot);
const server = await createServer({
  configFile: fileURLToPath(new URL('../vite.config.ts', import.meta.url)),
  appType: 'custom',
  server: { middlewareMode: true, fs: { allow: [repositoryRoot] } },
  optimizeDeps: { disabled: true },
  oxc: false,
  esbuild: { tsconfigRaw: { compilerOptions: { target: 'ES2021', module: 'ESNext' } } },
  logLevel: 'error'
});

try {
  const { CapabilityLossController } = await server.ssrLoadModule('/src/lib/mission-canvas/capability-loss-controller.ts');
  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  fixture.projection_revision = 10;
  fixture.layout_revision = 5;
  assert.ok(fixture.eligible_contributions.length > 1);
  const removed = fixture.eligible_contributions.at(-1).contribution_id;
  const recomposed = structuredClone(fixture);
  recomposed.projection_revision = 11;
  recomposed.layout_revision = 6;
  recomposed.eligible_contributions = recomposed.eligible_contributions.filter((item) => item.contribution_id !== removed);
  const authority = authorityOf(fixture);
  let refreshes = 0;
  const controller = new CapabilityLossController(async () => { refreshes += 1; return structuredClone(recomposed); });
  const event = capabilityEvent(fixture, authority);

  const loss = await controller.handle([event], fixture, authority);
  assert.equal(refreshes, 1);
  assert.deepEqual(loss.notification.affectedContributionIds, [removed]);
  assert.equal(loss.projection.eligible_contributions.some((item) => item.contribution_id === removed), false);
  assert.ok(loss.projection.eligible_contributions.length > 0, 'safe eligible content remains');

  const foreign = structuredClone(event);
  foreign.workstream.workstream_id = 'ws:foreign';
  assert.equal(await controller.handle([foreign], fixture, authority), undefined);
  assert.equal(refreshes, 1);

  const stale = structuredClone(event);
  stale.projection_revision = 9;
  assert.equal(await controller.handle([stale], fixture, authority), undefined);
  assert.equal(refreshes, 1);

  const restoredProjection = structuredClone(fixture);
  restoredProjection.projection_revision = 12;
  restoredProjection.layout_revision = 7;
  const returnController = new CapabilityLossController(async () => restoredProjection);
  const returnEvent = capabilityEvent(recomposed, authority);
  returnEvent.projection_revision = 12;
  returnEvent.layout_revision = 7;
  const returned = await returnController.handle([returnEvent], recomposed, authority);
  assert.equal(returned.notification, undefined);
  assert.deepEqual(returned.restoredContributionIds, [removed]);

  assert.equal(await controller.handle([], fixture, authority), undefined);
  console.log('Mission Canvas capability loss: PASS (recompose, dead control removal, safe content, return, foreign and stale cases)');
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
}

function authorityOf(projection) {
  return {
    workstream: structuredClone(projection.workstream),
    continuity_id: projection.continuity_id ?? null,
    attachment: structuredClone(projection.attachment ?? null),
    workspace_binding_id: projection.workspace_binding_id ?? null,
    runtime_object: structuredClone(projection.runtime_object ?? null),
    work_surface_id: projection.work_surface_id ?? projection.focused_work_surface_id ?? null
  };
}

function capabilityEvent(projection, authority) {
  return {
    ...structuredClone(authority),
    event_cursor: 'event:capability:11',
    event_id: 'event:capability:changed',
    event_kind: 'capability_changed',
    evidence_refs: [],
    layout_revision: projection.layout_revision,
    occurred_at: new Date().toISOString(),
    payload_ref: 'capability:changed',
    projection_revision: projection.projection_revision,
    receipt_refs: []
  };
}
