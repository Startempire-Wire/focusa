import assert from 'node:assert/strict';
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

function memoryStorage() {
  const map = new Map();
  return {
    getItem: (key) => (map.has(key) ? map.get(key) : null),
    setItem: (key, value) => void map.set(key, String(value)),
    removeItem: (key) => void map.delete(key),
    _map: map
  };
}

try {
  const { MissionCanvasRestorationController, reconcileSnapshot } =
    await server.ssrLoadModule('/src/lib/mission-canvas/restoration-controller.ts');
  const { workstreamAuthorityStorageKey } =
    await server.ssrLoadModule('/src/lib/mission-canvas/exact-scope.ts');

  const authority = {
    workstream: {
      scope: {
        scope_kind: 'project',
        scope_key: {
          scope_kind: 'project',
          scope_id: 'project:focusa',
          root_path: '/example/focusa',
          canonical_name: 'Focusa',
          fingerprint: 'host-a:worktree-main'
        }
      },
      workstream_id: 'ws:mission-canvas'
    },
    continuity_id: 'continuity:mission-canvas',
    attachment: {
      workstream: {
        scope: {
          scope_kind: 'project',
          scope_key: {
            scope_kind: 'project',
            scope_id: 'project:focusa',
            root_path: '/example/focusa',
            canonical_name: 'Focusa',
            fingerprint: 'host-a:worktree-main'
          }
        },
        workstream_id: 'ws:mission-canvas'
      },
      continuity_id: 'continuity:mission-canvas',
      instance_id: 'instance:pi',
      session_id: 'session:pi',
      attachment_id: 'attachment:pi',
      workspace_binding_id: 'workspace:mission-canvas'
    },
    workspace_binding_id: 'workspace:mission-canvas',
    runtime_object: { runtime_kind: 'pi_session', runtime_id: 'session:pi' },
    work_surface_id: 'surface:pi'
  };

  function projection(contributions = [
    { contribution_id: 'contribution:pi', kind: 'focused_work_surface', data_ref: { ref: 'surface:pi' }, accessibility: { label: 'Pi Session', focus_semantic_id: 'pi', landmark_role: 'region' } },
    { contribution_id: 'contribution:inspector', kind: 'canonical_contribution', data_ref: { ref: 'surface:inspector' }, accessibility: { label: 'Inspector', focus_semantic_id: 'inspector', landmark_role: 'region' } }
  ], overrides = {}) {
    return {
      schema: 'focusa.resolved_workspace_projection.v1',
      workstream: structuredClone(authority.workstream),
      projection_revision: 11,
      layout_revision: 5,
      eligible_contributions: contributions,
      ...overrides
    };
  }

  function snapshot(overrides = {}) {
    return {
      workstream: structuredClone(authority.workstream),
      projectionRevision: 11,
      scroll: [
        { locator: { attribute: 'data-contribution-id', value: 'contribution:pi' }, left: 0, top: 120 },
        { locator: { attribute: 'data-contribution-id', value: 'contribution:inspector' }, left: 0, top: 40 }
      ],
      focus: { locator: { attribute: 'data-semantic-object-id', value: 'editor:input' } },
      activeTab: { attribute: 'data-work-surface-id', value: 'surface:pi' },
      ...overrides
    };
  }

  // --- restart restores the same exact Workstream at a fresh revision ---
  {
    const storage = memoryStorage();
    const controller = new MissionCanvasRestorationController(storage);
    controller.persist(authority, 11, snapshot());
    const applied = [];
    const reconciled = controller.apply(authority, projection(), (value) => applied.push(value));
    assert.equal(applied.length, 1, 'restart restores presentation');
    assert.deepEqual(reconciled, [], 'no surfaces missing');
    assert.equal(applied[0].scroll.length, 2);
  }

  // --- stale projection revision never restores (no local snapshot is canonical) ---
  {
    const storage = memoryStorage();
    const controller = new MissionCanvasRestorationController(storage);
    controller.persist(authority, 11, snapshot());
    // canonical projection already advanced past the stored revision
    const applied = [];
    controller.apply(authority, projection([{ contribution_id: 'contribution:pi', kind: 'focused_work_surface', data_ref: { ref: 'surface:pi' }, accessibility: { label: 'Pi', focus_semantic_id: 'pi', landmark_role: 'region' } }], { projection_revision: 5 }), (value) => applied.push(value));
    assert.equal(applied.length, 0, 'stale revision must not restore');
  }

  // --- foreign Workstream never restores ---
  {
    const storage = memoryStorage();
    const controller = new MissionCanvasRestorationController(storage);
    controller.persist(authority, 11, snapshot());
    const foreign = structuredClone(authority);
    foreign.workstream.workstream_id = 'ws:foreign';
    const applied = [];
    controller.apply(foreign, projection(), (value) => applied.push(value));
    assert.equal(applied.length, 0, 'foreign workstream must not restore');
  }

  // --- missing surfaces are reconciled truthfully: dropped and reported ---
  {
    const storage = memoryStorage();
    const controller = new MissionCanvasRestorationController(storage);
    controller.persist(authority, 11, snapshot());
    const shrunken = projection([
      { contribution_id: 'contribution:pi', kind: 'focused_work_surface', data_ref: { ref: 'surface:pi' }, accessibility: { label: 'Pi', focus_semantic_id: 'pi', landmark_role: 'region' } }
    ]);
    const applied = [];
    const reconciled = controller.apply(authority, shrunken, (value) => applied.push(value));
    assert.ok(reconciled.includes('contribution:inspector'), 'missing inspector reported');
    assert.equal(applied.length, 1, 'still restores the surviving surface');
    assert.equal(applied[0].scroll.length, 1, 'dropped surface scroll removed');
    assert.equal(applied[0].scroll[0].locator.value, 'contribution:pi');
  }

  // --- pure reconcile: missing surface dropped + reported; non-surface kept ---
  {
    const candidate = reconcileSnapshot(snapshot(), projection([{ contribution_id: 'contribution:pi', kind: 'focused_work_surface', data_ref: { ref: 'surface:pi' }, accessibility: { label: 'Pi', focus_semantic_id: 'pi', landmark_role: 'region' } }]));
    assert.deepEqual(candidate.reconciledSurfaces, ['contribution:inspector']);
    assert.equal(candidate.snapshot.scroll.length, 1);
    assert.equal(candidate.snapshot.focus?.locator.value, 'editor:input', 'non-surface focus kept');
  }

  // --- clean restart (no storage) restores nothing ---
  {
    const controller = new MissionCanvasRestorationController(memoryStorage());
    const applied = [];
    controller.apply(authority, projection(), (value) => applied.push(value));
    assert.equal(applied.length, 0, 'no stored snapshot means no restoration');
  }

  // --- storage key is exact-Workstream bound ---
  {
    assert.equal(
      workstreamAuthorityStorageKey(authority),
      workstreamAuthorityStorageKey(structuredClone(authority)),
      'key stable for identical authority'
    );
    const foreign = structuredClone(authority);
    foreign.attachment.attachment_id = 'attachment:other';
    assert.notEqual(workstreamAuthorityStorageKey(authority), workstreamAuthorityStorageKey(foreign));
  }

  console.log('restoration-runtime: PASS');
} finally {
  await server.close();
}
