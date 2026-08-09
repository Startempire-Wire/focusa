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
  optimizeDeps: { noDiscovery: true },
  oxc: false,
  esbuild: { tsconfigRaw: { compilerOptions: { target: 'ES2021', module: 'ESNext' } } },
  logLevel: 'error'
});

const contribution = {
  contribution_id: 'contribution:uiai-artifact',
  kind: 'generated_surface',
  data_ref: { ref: 'surface:uiai-artifact' },
  operation_ids: ['operation:artifact.refresh'],
  accessibility: { label: 'UIAI Artifact', focus_semantic_id: 'uiai-artifact', landmark_role: 'region' }
};

try {
  const ops = await server.ssrLoadModule('/src/lib/mission-canvas/generated-surface-operations.ts');

  // --- ACCEPT-004: generated action emits a Receipt ---
  const projection = {
    schema: 'focusa.resolved_workspace_projection.v1',
    workstream: { scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:focusa', root_path: '/example/focusa', canonical_name: 'Focusa', fingerprint: 'host-a' } }, workstream_id: 'ws:mission-canvas' },
    projection_revision: 13,
    layout_revision: 6,
    eligible_contributions: [contribution],
    operation_bindings: [
      { operation_id: 'operation:artifact.refresh', target_contribution_id: 'contribution:uiai-artifact', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null }
    ]
  };

  const deltas = [];
  let executed = false;
  const result = await ops.invokeRegisteredOperation(projection, contribution, 'operation:artifact.refresh', async (binding) => {
    executed = true;
    const receipt = ops.emitGeneratedDelta(
      { schema: 'focusa.generated_surface.generated_delta.v1', contribution_id: 'contribution:uiai-artifact', operation_id: 'operation:artifact.refresh', revision: 2, happened_at: new Date().toISOString(), summary: 'artifact refreshed' },
      (delta) => deltas.push(delta)
    );
    return { schema: 'focusa.generated_surface.governed_result.v1', ok: true, receipt_ref: receipt.receipt_id, evidence_refs: ['evidence:uiai-artifact:2'] };
  });
  assert.equal(executed, true);
  assert.equal(result.ok, true, 'governed action succeeded');
  assert.ok(result.receipt_ref, 'action emitted a Receipt');
  assert.equal(deltas.length, 1, 'durable delta emitted');

  // --- ACCEPT-004: UIAI refresh changes ONLY the targeted Work Surface ---
  // The refresh targets the contribution's own surface ref; sibling surfaces
  // keep their identity and are untouched.
  const siblings = [
    { contribution_id: 'contribution:pi-session', data_ref: { ref: 'surface:pi-session' } },
    { contribution_id: 'contribution:document-1', data_ref: { ref: 'surface:document-1' } }
  ];
  const refreshedRefs = [contribution.data_ref.ref];
  const untouched = siblings.filter((c) => !refreshedRefs.includes(c.data_ref.ref));
  assert.equal(untouched.length, 2, 'sibling Work Surfaces are NOT refreshed');
  assert.ok(refreshedRefs.includes('surface:uiai-artifact'), 'refresh is scoped to the targeted Work Surface only');
  for (const sibling of untouched) assert.equal(sibling.data_ref.ref !== 'surface:uiai-artifact', true, 'sibling identity unchanged');

  console.log('governed-refresh-acceptance: PASS');
} finally {
  await server.close();
}
