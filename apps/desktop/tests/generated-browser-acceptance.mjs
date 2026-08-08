import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

const command = parseCommand();
if (!command || command.case !== 'uiai-artifact') {
  console.error('Usage: node apps/desktop/tests/generated-browser-acceptance.mjs --case uiai-artifact');
  process.exit(1);
}

const desktopRoot = fileURLToPath(new URL('../', import.meta.url));
process.chdir(desktopRoot);
const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));

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
  const { ContributionRendererRegistry } = await server.ssrLoadModule('/src/lib/mission-canvas/contribution-renderers.ts');
  const { validateProjection, rejectStaleRevision } = await server.ssrLoadModule('/src/lib/mission-canvas/projection-controller.svelte.ts');
  const { collectLayoutContributionIds, validateLayoutIntegrity } = await server.ssrLoadModule('/src/lib/mission-canvas/layout-references.ts');
  const {
    BrowserArtifactRef,
    UIAISessionRef,
    ArtifactRenderer,
    parseBrowserArtifactDescriptor,
    browserSessionContextFromDescriptor
  } = await server.ssrLoadModule('/src/lib/mission-canvas/browser-artifact.ts');

  const variants = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/layout-variants.json', import.meta.url), 'utf8'));
  const rawDescriptor = await readFile(new URL('./fixtures/mission-canvas/browser-artifact-descriptor.json', import.meta.url), 'utf8');
  const fixtureDescriptor = JSON.parse(rawDescriptor);

  assert.equal(BrowserArtifactRef.validate(fixtureDescriptor), true);
  assert.equal(UIAISessionRef.validate(browserSessionContextFromDescriptor(fixtureDescriptor)), true);

  const serializedDescriptor = JSON.stringify(fixtureDescriptor);
  const parsedDescriptor = parseBrowserArtifactDescriptor(serializedDescriptor);
  assert.ok(Boolean(parsedDescriptor), 'browser artifact descriptor fixture must parse');

  const artifactLines = ArtifactRenderer.render(parsedDescriptor);
  assert.ok(artifactLines.includes(`Artifact: ${fixtureDescriptor.artifact_id}`));
  assert.ok(artifactLines.includes('Execution control: Desktop is metadata-only; browser actions remain in UIAI Engine.'));

  const projection = makeBrowserArtifactProjection(variants.single, fixtureDescriptor);
  const authority = authorityOf(projection);
  const validation = validateProjection(projection, authority);
  assert.equal(validation.valid, true, 'baseline browser projection must validate');

  const registryText = await readFile(new URL('../src/lib/mission-canvas/default-contribution-registry.ts', import.meta.url), 'utf8');
  assert.equal(registryText.includes('renderer:artifact:browser_snapshot@v1'), true, 'expected browser snapshot renderer binding is wired in the host registry file');

  const browserContribution = projection.eligible_contributions[0];
  const hostRegistry = new ContributionRendererRegistry([
    {
      rendererBindingId: 'renderer:artifact:browser_snapshot@v1',
      contributionKinds: ['focused_work_surface', 'generated_surface'],
      component: () => null
    }
  ]);
  const hostResolution = hostRegistry.resolveContributionRenderer(browserContribution);
  assert.equal(hostResolution === undefined ? 'blocked' : 'resolved', 'resolved', 'local browser contribution should resolve against the browser renderer binding');

  const unavailableRegistry = new ContributionRendererRegistry([
    {
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      contributionKinds: ['focused_work_surface'],
      component: () => null
    }
  ]);
  const unavailableResolution = unavailableRegistry.resolveWithDiagnostic(browserContribution);
  assert.equal(unavailableResolution.status, 'blocked');
  assert.equal(unavailableResolution.diagnostic.reason, 'unknown_renderer_binding');

  const foreign = structuredClone(projection);
  foreign.workstream.workstream_id = 'ws:foreign';
  if (foreign.attachment) foreign.attachment.workstream.workstream_id = 'ws:foreign';
  const foreignValidation = validateProjection(foreign, authority);
  assert.equal(foreignValidation.valid, false);
  assert.equal(foreignValidation.failure, 'scope');

  const missingAuthority = structuredClone(projection);
  delete missingAuthority.workstream;
  const missingAuthorityValidation = validateProjection(missingAuthority, authority);
  assert.equal(missingAuthorityValidation.valid, false);

  const staleProjection = structuredClone(projection);
  staleProjection.projection_revision = projection.projection_revision - 1;
  assert.equal(rejectStaleRevision(projection, staleProjection), 'projection_revision_regressed');

  const staleLayoutProjection = structuredClone(projection);
  staleLayoutProjection.layout_revision = projection.layout_revision - 1;
  assert.equal(rejectStaleRevision(projection, staleLayoutProjection), 'projection_layout_revision_regressed');

  const staleCursorProjection = structuredClone(projection);
  staleCursorProjection.durable_event_cursor = 'event:39';
  assert.equal(rejectStaleRevision(projection, staleCursorProjection), 'projection_cursor_regressed');

  const mismatchedCursorProjection = structuredClone(projection);
  mismatchedCursorProjection.durable_event_cursor = 'cursor:41';
  assert.equal(rejectStaleRevision(projection, mismatchedCursorProjection), 'projection_cursor_namespace_mismatch');

  const missingLayoutProjection = structuredClone(projection);
  missingLayoutProjection.layout_tree = {
    node_id: 'layout:uiai-browser-artifact-missing',
    kind: 'single',
    contribution_id: 'contribution:missing'
  };
  const missingLayoutIds = collectLayoutContributionIds(missingLayoutProjection.layout_tree);
  const eligibleContributionIds = new Set(missingLayoutProjection.eligible_contributions.map((contribution) => contribution.contribution_id));
  const unresolvedLayoutContribution = [...missingLayoutIds].find((id) => !eligibleContributionIds.has(id));
  assert.equal(unresolvedLayoutContribution, 'contribution:missing');
  assert.equal(validateLayoutIntegrity(projection.layout_tree).length, 0, 'baseline layout integrity is clean');

  const unplacedProjection = structuredClone(projection);
  unplacedProjection.eligible_contributions.push({
    ...browserContribution,
    contribution_id: 'contribution:unplaced-browser-artifact',
    data_ref: {
      ...browserContribution.data_ref,
      ref: JSON.stringify({
        ...fixtureDescriptor,
        artifact_id: 'artifact:uiai:browser:unplaced',
        artifact_kind: 'browser_snapshot'
      })
    }
  });
  unplacedProjection.candidate_contribution_ids = [
    browserContribution.contribution_id,
    'contribution:unplaced-browser-artifact'
  ];
  const unplacedIds = collectLayoutContributionIds(unplacedProjection.layout_tree);
  const unplacedEligible = new Set(unplacedProjection.eligible_contributions.map((item) => item.contribution_id));
  const unplacedOnly = [...unplacedProjection.eligible_contributions]
    .map((item) => item.contribution_id)
    .filter((id) => !unplacedIds.has(id));
  assert.equal(unplacedOnly.includes('contribution:unplaced-browser-artifact'), true);

  const invalidContributionAuth = structuredClone(projection);
  invalidContributionAuth.eligible_contributions[0].authority.workstream.workstream_id = 'ws:foreign';
  invalidContributionAuth.workstream.workstream_id = 'ws:foreign';
  if (invalidContributionAuth.attachment) invalidContributionAuth.attachment.workstream.workstream_id = 'ws:foreign';
  const invalidContributionValidation = validateProjection(invalidContributionAuth, authority);
  assert.equal(invalidContributionValidation.valid, false);
  assert.equal(invalidContributionValidation.failure, 'scope');

  console.log('Mission Canvas browser artifact acceptance: PASS (uiai-artifact)');
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) {
    await new Promise((resolve) => server.httpServer.close(resolve));
  }
}

function makeBrowserArtifactProjection(baseProjection, descriptor) {
  const projection = structuredClone(baseProjection);
  const contributionTemplate = structuredClone(projection.eligible_contributions.at(0));
  const contribution = {
    ...contributionTemplate,
    contribution_id: 'contribution:uiai-browser-artifact',
    kind: 'focused_work_surface',
    semantic_binding_id: 'semantic:uiai-browser-artifact',
    renderer_binding_id: 'renderer:artifact:browser_snapshot@v1',
    data_ref: {
      kind: 'workspace_artifact',
      ref: JSON.stringify(descriptor),
      revision: 14,
      freshness: 'current'
    },
    accessibility: {
      ...contributionTemplate.accessibility,
      label: 'UIAI Browser Snapshot',
      description: 'Exact browser session/context/artifact projection for isolated UIAI context',
      focus_semantic_id: 'focus:uiai-browser-artifact'
    },
    operation_ids: ['focusa.mission_canvas.rich_host.launch']
  };

  projection.eligible_contributions = [contribution];
  projection.candidate_contribution_ids = [contribution.contribution_id];
  projection.layout_tree = {
    node_id: 'layout:uiai-browser-artifact',
    kind: 'single',
    contribution_id: contribution.contribution_id
  };
  projection.focused_semantic_target = 'focus:uiai-browser-artifact';

  return projection;
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

function parseCommand() {
  const args = process.argv.slice(2);
  const caseIndex = args.indexOf('--case');
  if (caseIndex < 0) return {};
  return {
    case: args[caseIndex + 1]
  };
}
