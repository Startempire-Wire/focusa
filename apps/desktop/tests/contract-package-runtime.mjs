import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

const desktopRoot = fileURLToPath(new URL('../', import.meta.url));
const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));
process.chdir(desktopRoot);
const server = await createServer({
  configFile: fileURLToPath(new URL('../vite.config.ts', import.meta.url)),
  appType: 'custom',
  server: { middlewareMode: true, fs: { allow: [repositoryRoot] } },
  optimizeDeps: { noDiscovery: true },
  oxc: false,
  esbuild: { tsconfigRaw: { compilerOptions: { target: 'ES2021', module: 'ESNext' } } },
  logLevel: 'error'
});

try {
  // --- CONTRACT-007: package re-exports generated artifacts without copying ---
  const pkg = await server.ssrLoadModule('/src/lib/mission-canvas/contract-probe.ts');
  // Load the generated artifact through the app's own canonical import path
  // (the same resolution the rest of the renderer uses).
  const generatedValidators = await server.ssrLoadModule('/src/lib/mission-canvas/contract-probe.ts');
  // Same module instances (identity equality => zero DTO copies).
  assert.equal(pkg.validateMissionCanvasContract, generatedValidators.validateMissionCanvasContract, 'validator re-exported, not copied');
  assert.equal(pkg.ResolvedWorkspaceProjection, undefined, 'type-only re-export (types erased)');
  const indexSource = readFileSync(new URL('../../../packages/mission-canvas-contracts/index.ts', import.meta.url), 'utf8');
  assert.match(indexSource, /export \* from '\.\.\/\.\.\/docs\/contracts\/spec135\/mission-canvas-v1\/typescript\//, 're-export only, no DTO definitions');
  assert.doesNotMatch(indexSource, /export interface AttachmentKey|export type ResolvedWorkspaceProjection/, 'no duplicated type definitions');

  // --- CONTRACT-008: pure traversal utilities never choose eligibility ---
  const traversal = await server.ssrLoadModule('/src/lib/mission-canvas/traversal-probe.ts');
  const projection = {
    schema: 'focusa.resolved_workspace_projection.v1',
    workstream: { scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:focusa', root_path: '/example/focusa', canonical_name: 'Focusa', fingerprint: 'host-a' } }, workstream_id: 'ws:mission-canvas' },
    projection_revision: 7,
    layout_revision: 3,
    eligible_contributions: [
      { contribution_id: 'contribution:pi', kind: 'focused_work_surface', data_ref: { ref: 'surface:pi' }, accessibility: { label: 'Pi Session', focus_semantic_id: 'pi', landmark_role: 'region' } },
      { contribution_id: 'contribution:inspector', kind: 'canonical_contribution', data_ref: { ref: 'surface:inspector' }, accessibility: { label: 'Inspector', focus_semantic_id: 'inspector', landmark_role: 'region' } },
      { contribution_id: 'contribution:steering', kind: 'steering_queue', data_ref: { ref: 'queue:steering' }, accessibility: { label: 'Steering', focus_semantic_id: 'steering', landmark_role: 'region' } }
    ]
  };
  const frozen = Object.freeze({ ...projection, eligible_contributions: Object.freeze(projection.eligible_contributions.map((c) => Object.freeze(c))) });
  const refs = traversal.collectContributionRefs(frozen);
  assert.equal(refs.length, 3);
  assert.equal(refs[0].contribution_id, 'contribution:pi');
  assert.deepEqual(traversal.contributionIdsOfKind(frozen, 'focused_work_surface'), ['contribution:pi']);
  assert.equal(traversal.findContribution(frozen, 'contribution:steering')?.data_ref.ref, 'queue:steering');
  assert.equal(traversal.findContribution(frozen, 'contribution:missing'), undefined);
  assert.deepEqual(traversal.workSurfaceRefs(frozen), ['surface:pi']);
  assert.deepEqual(traversal.contributionOrder(frozen), ['contribution:pi', 'contribution:inspector', 'contribution:steering']);
  assert.deepEqual(frozen.eligible_contributions.length, 3, 'traversal never mutates the canonical projection');

  console.log('contract-package-runtime: PASS');
} finally {
  await server.close();
}
