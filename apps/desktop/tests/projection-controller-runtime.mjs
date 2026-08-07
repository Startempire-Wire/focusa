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
  const { MissionCanvasProjectionController, rejectStaleRevision, validateProjection } =
    await server.ssrLoadModule('/src/lib/mission-canvas/projection-controller.svelte.ts');
  const {
    isExactWorkSurfaceIdentity,
    projectWorkSurfaces,
    projectWorkSurfacesWithDiagnostics
  } = await server.ssrLoadModule('/src/lib/mission-canvas/types.ts');
  const { exactScopeKey } = await server.ssrLoadModule('/src/lib/mission-canvas/exact-scope.ts');
  const fixture = JSON.parse(
    await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8')
  );
  const authority = authorityOf(fixture);

  const generatedSurfacePayload = {
    schema: 'focusa.mission_canvas_surface_list.v1',
    surfaces: [{
      work_surface_id: fixture.work_surface_id,
      display_name: 'Pi session',
      surface_kind: 'pi_session',
      identity: {
        workstream: structuredClone(fixture.workstream),
        continuity_id: fixture.continuity_id,
        attachment: structuredClone(fixture.attachment),
        runtime_object: structuredClone(fixture.runtime_object),
        work_surface_id: fixture.work_surface_id
      },
      scope: {
        project_root: fixture.workstream.scope.scope_key.root_path,
        continuity_id: fixture.continuity_id,
        workpoint_id: 'workpoint:pi'
      },
      primary_attachment: {
        ...structuredClone(fixture.attachment),
        role: 'active'
      },
      presentation: {
        title: 'Pi session',
        renderer_id: 'renderer:pi-session@v1',
        pinned: true,
        group_id: fixture.continuity_id
      },
      activity: {
        lifecycle_state: 'open',
        semantic_activity: 'coding',
        health: 'healthy',
        unread_event_count: 2,
        pending_approval_count: 1
      },
      isolation: {
        writer_lease_ref: 'lease:pi',
        worktree_ref: 'worktree:pi',
        browser_isolation_class: 'not-applicable'
      }
    }]
  };
  const projectedSurfaces = projectWorkSurfaces(generatedSurfacePayload);
  assert.equal(projectedSurfaces.length, 1);
  assert.equal(projectedSurfaces[0].identity.workstream.workstream_id, 'ws:mission-canvas');
  assert.equal(projectedSurfaces[0].identity.attachment.attachment_id, 'attachment:pi');
  assert.equal(projectedSurfaces[0].identity.attachment.session_id, 'session:pi');
  assert.equal(projectedSurfaces[0].identity.attachment.instance_id, 'instance:pi');
  assert.equal(projectedSurfaces[0].identity.work_surface_id, 'surface:pi');
  assert.equal(projectedSurfaces[0].sessionId, 'session:pi');
  assert.equal(projectedSurfaces[0].instanceId, 'instance:pi');
  assert.equal(isExactWorkSurfaceIdentity(projectedSurfaces[0].identity), true);

  const splitIdentityPayload = structuredClone(generatedSurfacePayload);
  delete splitIdentityPayload.surfaces[0].identity;
  splitIdentityPayload.surfaces[0].workstream = structuredClone(fixture.workstream);
  delete splitIdentityPayload.surfaces[0].primary_attachment.workstream;
  assert.equal(projectWorkSurfaces(splitIdentityPayload).length, 1);
  assert.equal(exactScopeKey(projectedSurfaces[0]), exactScopeKey(projectedSurfaces[0].identity));
  assert.ok(exactScopeKey(projectedSurfaces[0]));

  const legacySurface = {
    work_surface_id: 'surface:legacy',
    project_root: '/example/focusa',
    continuity_id: 'continuity:mission-canvas',
    attachment_id: 'attachment:legacy',
    instance_id: 'instance:legacy',
    session_id: 'session:legacy'
  };
  const legacyResult = projectWorkSurfacesWithDiagnostics({ surfaces: [legacySurface] });
  assert.deepEqual(legacyResult.surfaces, []);
  assert.equal(legacyResult.quarantined[0].reason, 'missing_exact_identity');
  assert.equal(exactScopeKey(legacySurface), undefined);

  const foreignSurface = structuredClone(generatedSurfacePayload);
  foreignSurface.surfaces[0].identity.workstream.workstream_id = 'ws:foreign';
  foreignSurface.surfaces[0].identity.attachment.workstream.workstream_id = 'ws:foreign';
  assert.equal(projectWorkSurfaces(foreignSurface, authority).length, 0);
  assert.equal(projectWorkSurfacesWithDiagnostics(foreignSurface, authority).quarantined[0].reason, 'foreign_scope');

  const missingAttachment = structuredClone(generatedSurfacePayload);
  delete missingAttachment.surfaces[0].identity.attachment;
  assert.equal(projectWorkSurfaces(missingAttachment).length, 0);

  const foreignAttachment = structuredClone(generatedSurfacePayload);
  foreignAttachment.surfaces[0].identity.attachment.workstream.workstream_id = 'ws:foreign';
  assert.equal(projectWorkSurfaces(foreignAttachment).length, 0);
  assert.equal(projectWorkSurfacesWithDiagnostics(foreignAttachment).quarantined[0].reason, 'foreign_attachment_workstream');

  const mismatchedSession = structuredClone(generatedSurfacePayload);
  mismatchedSession.surfaces[0].session_id = 'session:foreign';
  assert.equal(projectWorkSurfaces(mismatchedSession).length, 0);
  assert.equal(projectWorkSurfacesWithDiagnostics(mismatchedSession).quarantined[0].reason, 'identity_mismatch');

  const duplicateSurfaces = structuredClone(generatedSurfacePayload);
  duplicateSurfaces.surfaces.push(structuredClone(duplicateSurfaces.surfaces[0]));
  const duplicateResult = projectWorkSurfacesWithDiagnostics(duplicateSurfaces);
  assert.deepEqual(duplicateResult.surfaces, []);
  assert.equal(duplicateResult.quarantined.length, 2);
  assert.ok(duplicateResult.quarantined.every(({ reason }) => reason === 'duplicate_identity'));

  assert.equal(validateProjection(fixture, authority).valid, true);

  const malformed = structuredClone(fixture);
  delete malformed.workstream;
  assert.equal(validateProjection(malformed, authority).valid, false);

  const foreignProjection = structuredClone(fixture);
  foreignProjection.workstream.workstream_id = 'ws:foreign';
  foreignProjection.attachment.workstream.workstream_id = 'ws:foreign';
  const foreignValidation = validateProjection(foreignProjection, authority);
  assert.equal(foreignValidation.valid, false);
  assert.equal(foreignValidation.failure, 'scope');
  assert.equal(foreignValidation.reason, 'projection_scope_mismatch');

  const foreignContribution = structuredClone(fixture);
  foreignContribution.eligible_contributions[0].authority.workstream.workstream_id = 'ws:foreign';
  foreignContribution.eligible_contributions[0].authority.attachment.workstream.workstream_id = 'ws:foreign';
  const foreignContributionValidation = validateProjection(foreignContribution, authority);
  assert.equal(foreignContributionValidation.valid, false);
  assert.equal(foreignContributionValidation.failure, 'scope');
  assert.equal(foreignContributionValidation.reason, 'foreign_contribution_scope');

  const invalidLayout = structuredClone(fixture);
  invalidLayout.layout_tree = {
    kind: 'tabs',
    node_id: 'layout:hostile-tabs',
    contribution_ids: ['contribution:pi-session'],
    active_contribution_id: 'contribution:not-eligible'
  };
  assert.equal(validateProjection(invalidLayout, authority).valid, false);

  // Core-owned omission is consumed as-is.  The controller never promotes the
  // omitted Work Rail candidate or invents a replacement contribution.
  assert.equal(
    fixture.eligible_contributions.some(({ contribution_id }) => contribution_id === 'contribution:empty-work-rail'),
    false
  );
  assert.equal(
    fixture.omission_diagnostics.some(({ contribution_id, reason }) =>
      contribution_id === 'contribution:empty-work-rail' && reason === 'no_relevant_content'),
    true
  );

  assert.equal(
    rejectStaleRevision(undefined, fixture),
    undefined
  );
  assert.equal(
    rejectStaleRevision(fixture, { ...fixture, projection_revision: fixture.projection_revision - 1 }),
    'projection_revision_regressed'
  );
  assert.equal(
    rejectStaleRevision(fixture, { ...fixture, layout_revision: fixture.layout_revision - 1 }),
    'projection_layout_revision_regressed'
  );
  assert.equal(
    rejectStaleRevision(fixture, { ...fixture, durable_event_cursor: 'event:40' }),
    'projection_cursor_regressed'
  );
  assert.equal(
    rejectStaleRevision(fixture, { ...fixture, durable_event_cursor: 'cursor:40' }),
    'projection_cursor_namespace_mismatch'
  );

  const requests = [];
  const client = {
    projectionGet: async (input) => {
      requests.push(input);
      return structuredClone(fixture);
    }
  };
  const controller = new MissionCanvasProjectionController(client);
  await controller.load(authority);
  assert.equal(controller.state.kind, 'ready');
  assert.equal(requests.length, 1);
  assert.deepEqual(requests[0], authority);
  assert.equal(Object.isFrozen(controller.state.scope), true);
  assert.equal(Object.isFrozen(controller.state.projection), true);
  assert.equal(controller.state.projection.projection_digest, fixture.projection_digest);

  // The generated client handoff and accepted projection are detached from
  // mutable caller/transport objects.
  authority.workstream.workstream_id = 'ws:caller-mutated';
  requests[0].workstream.workstream_id = 'ws:request-mutated';
  assert.equal(controller.state.scope.workstream.workstream_id, 'ws:mission-canvas');
  assert.equal(controller.state.projection.workstream.workstream_id, 'ws:mission-canvas');

  let invalidRequestCalls = 0;
  const invalidRequestController = new MissionCanvasProjectionController({
    projectionGet: async () => {
      invalidRequestCalls += 1;
      return structuredClone(fixture);
    }
  });
  await invalidRequestController.load({});
  assert.equal(invalidRequestCalls, 0);
  assert.equal(invalidRequestController.state.kind, 'blocked');
  assert.match(invalidRequestController.state.reason, /^invalid_workstream_scope:/);

  const foreignController = new MissionCanvasProjectionController({
    projectionGet: async () => structuredClone(foreignProjection)
  });
  await foreignController.load(authorityOf(fixture));
  assert.equal(foreignController.state.kind, 'blocked');
  assert.equal(foreignController.state.reason, 'projection_scope_mismatch');

  const nestedForeignController = new MissionCanvasProjectionController({
    projectionGet: async () => structuredClone(foreignContribution)
  });
  await nestedForeignController.load(authorityOf(fixture));
  assert.equal(nestedForeignController.state.kind, 'blocked');
  assert.equal(nestedForeignController.state.reason, 'foreign_contribution_scope');

  let response = structuredClone(fixture);
  const staleController = new MissionCanvasProjectionController({
    projectionGet: async () => structuredClone(response)
  });
  await staleController.load(authorityOf(fixture));
  response = { ...response, projection_revision: response.projection_revision - 1 };
  await staleController.load(authorityOf(fixture));
  assert.equal(staleController.state.kind, 'stale');
  assert.equal(staleController.state.reason, 'projection_revision_regressed');
  assert.equal(staleController.state.projection.projection_revision, fixture.projection_revision);

  response = { ...fixture, layout_revision: fixture.layout_revision - 1 };
  await staleController.load(authorityOf(fixture));
  assert.equal(staleController.state.kind, 'stale');
  assert.equal(staleController.state.reason, 'projection_layout_revision_regressed');

  response = { ...fixture, durable_event_cursor: 'event:40' };
  await staleController.load(authorityOf(fixture));
  assert.equal(staleController.state.kind, 'stale');
  assert.equal(staleController.state.reason, 'projection_cursor_regressed');

  const transportFailureController = new MissionCanvasProjectionController({
    projectionGet: async () => { throw new Error('foreign_projection_scope'); }
  });
  await transportFailureController.load(authorityOf(fixture));
  assert.equal(transportFailureController.state.kind, 'blocked');
  assert.equal(transportFailureController.state.reason, 'foreign_projection_scope');

  // A response from an older Workstream cannot win a race with a newer exact
  // binding, even if it resolves last.
  const scopeA = authorityOf(fixture);
  const scopeB = replaceWorkstream(authorityOf(fixture), 'ws:second');
  const pending = new Map();
  const racingController = new MissionCanvasProjectionController({
    projectionGet: (input) => new Promise((resolve) => pending.set(input.workstream.workstream_id, resolve))
  });
  const first = racingController.load(scopeA);
  const second = racingController.load(scopeB);
  pending.get('ws:mission-canvas')(structuredClone(fixture));
  await first;
  assert.equal(racingController.state.kind, 'loading');
  const secondProjection = replaceWorkstream(structuredClone(fixture), 'ws:second');
  pending.get('ws:second')(secondProjection);
  await second;
  assert.equal(racingController.state.kind, 'ready');
  assert.equal(racingController.state.scope.workstream.workstream_id, 'ws:second');
  assert.equal(racingController.state.projection.workstream.workstream_id, 'ws:second');

  let resolveCleared;
  const clearedController = new MissionCanvasProjectionController({
    projectionGet: async () => new Promise((resolve) => { resolveCleared = resolve; })
  });
  const loading = clearedController.load(authorityOf(fixture));
  clearedController.clear();
  resolveCleared(structuredClone(fixture));
  await loading;
  assert.deepEqual(clearedController.state, { kind: 'unbound' });

  assert.equal(controller.accept(authorityOf(fixture), malformed), false);
  assert.equal(controller.state.kind, 'stale');

  console.log('Mission Canvas projection controller: PASS (generated projectionGet, exact WorkSurface identity, quarantine, fail-closed mismatch, stale watermarks, race, clear, omission, and hostile response checks)');
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

function replaceWorkstream(value, workstreamId) {
  if (Array.isArray(value)) {
    for (const item of value) replaceWorkstream(item, workstreamId);
    return value;
  }
  if (!value || typeof value !== 'object') return value;
  for (const [key, child] of Object.entries(value)) {
    if (key === 'workstream' && child && typeof child === 'object' && !Array.isArray(child)) {
      child.workstream_id = workstreamId;
    } else {
      replaceWorkstream(child, workstreamId);
    }
  }
  return value;
}
