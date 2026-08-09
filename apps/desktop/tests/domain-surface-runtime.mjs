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

function projection(contributions) {
  return {
    schema: 'focusa.resolved_workspace_projection.v1',
    workstream: { scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:focusa', root_path: '/example/focusa', canonical_name: 'Focusa', fingerprint: 'host-a' } }, workstream_id: 'ws:mission-canvas' },
    projection_revision: 12,
    layout_revision: 5,
    eligible_contributions: contributions
  };
}

const contribution = (ref, kind = 'domain_surface') => ({
  contribution_id: `contribution:${ref.replace(/[:/]/g, '-')}`,
  kind,
  data_ref: { ref },
  accessibility: { label: ref, focus_semantic_id: ref, landmark_role: 'region' }
});

try {
  const surfaces = await server.ssrLoadModule('/src/lib/mission-canvas/domain-surface-contributions.ts');

  // --- DOMAIN-001..012: only eligible (resolver-emitted) contributions render ---
  for (const descriptor of surfaces.DOMAIN_SURFACE_DESCRIPTORS) {
    // Resolver emitted it -> eligible.
    const emitted = projection([contribution(`${descriptor.ref_prefix}pi`), contribution('surface:overview:mission')]);
    assert.equal(surfaces.domainSurfaceEligible(emitted, descriptor.domain_surface_id), true,
      `${descriptor.domain_surface_id}: renders when Core emits it`);
    // Not emitted -> never renders (no fixed dashboard, no local inference).
    const absent = projection([contribution('surface:overview:mission')]);
    const other = descriptor.domain_surface_id === 'overview' ? 'surface:context:pi' : 'surface:overview:pi';
    assert.equal(surfaces.domainSurfaceEligible(projection([contribution(other)]), descriptor.domain_surface_id), false,
      `${descriptor.domain_surface_id}: never renders without resolver emission`);
    void absent;
  }

  // --- DOMAIN-001: no fixed mission-status dashboard is encoded ---
  {
    const files = ['MissionCanvasRenderer.svelte', 'ProjectionLayoutRenderer.svelte', 'MissionCanvasFrame.svelte'];
    for (const file of files) {
      const source = readFileSync(new URL(`../src/lib/mission-canvas/${file}`, import.meta.url), 'utf8');
      assert.doesNotMatch(source, /domainSurfaceEligible|DOMAIN_SURFACE_DESCRIPTORS/, `${file} does not embed domain eligibility`);
    }
    // A projection that emits ONLY overview renders exactly the emitted set.
    const overviewOnly = projection([
      contribution('surface:overview:mission'),
      contribution('surface:overview:steering', 'steering_queue')
    ]);
    assert.deepEqual(overviewOnly.eligible_contributions.map((c) => c.data_ref.ref),
      ['surface:overview:mission', 'surface:overview:steering'],
      'overview renders exactly the resolver-emitted contributions');
  }

  // --- DOMAIN-002..006: provenance/governance/resumability via canonical ops ---
  {
    // Context/Role/Interview/Spec/Tasks are canonical domain surfaces; the
    // generated-surface operation gate (GEN-002) governs their actions.
    const { resolveOperationPermission } = await server.ssrLoadModule('/src/lib/mission-canvas/generated-surface-operations.ts');
    const contributionId = 'contribution:surface-context-pi';
    const proj = projection([contribution('surface:context:pi')]);
    proj.operation_bindings = [
      { operation_id: 'operation:genesis.context.advance', target_contribution_id: contributionId, enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null }
    ];
    const verdict = resolveOperationPermission(proj, proj.eligible_contributions[0], 'operation:genesis.context.advance');
    assert.equal(verdict.permitted, true, 'governed role surface action resolves through the permission projection');
  }

  // --- DOMAIN-007: multiplexed runtime inventory (one contribution per runtime ref) ---
  {
    const proj = projection([
      contribution('surface:runtime-inventory:pi'),
      contribution('surface:runtime-inventory:uiai'),
      contribution('surface:runtime-inventory:silent-session')
    ]);
    const refs = surfaces.runtimeInventoryRefs(proj);
    assert.equal(refs.length, 3, 'one inventory contribution per emitted runtime ref');
    assert.equal(refs[1], 'surface:runtime-inventory:uiai');
  }

  // --- DOMAIN-008..010: document/research/proof contributions ---
  {
    const proj = projection([
      contribution('surface:document:contracts'),
      contribution('surface:research:web'),
      contribution('surface:proof:evidence')
    ]);
    assert.equal(surfaces.domainSurfaceEligible(proj, 'document'), true);
    assert.equal(surfaces.domainSurfaceEligible(proj, 'research'), true);
    assert.equal(surfaces.domainSurfaceEligible(proj, 'proof'), true);
    assert.equal(surfaces.domainSurfaceEligible(proj, 'history'), false, 'unemitted surface absent');
  }

  // --- DOMAIN-011: bounded Workstream history ---
  {
    const proj = projection([contribution('surface:history:workstream')]);
    assert.equal(surfaces.domainSurfaceEligible(proj, 'history'), true);
    // History contributions never surface outside resolver emission.
    assert.equal(surfaces.domainSurfaceEligible(projection([]), 'history'), false);
  }

  // --- DOMAIN-012: explicit management flow is separate from the daily workspace ---
  {
    const daily = projection([contribution('surface:overview:mission'), contribution('surface:management:settings')]);
    assert.equal(surfaces.domainSurfaceEligible(daily, 'management'), true);
    assert.equal(surfaces.managementFlowEligible(daily), false, 'management never merges into the daily workspace');
    const managementOnly = projection([contribution('surface:management:settings')]);
    assert.equal(surfaces.managementFlowEligible(managementOnly), true, 'management flow is explicit and separate');
    assert.ok(!surfaces.MANAGEMENT_DOMAIN_SURFACES.includes('overview'), 'daily surfaces are never in the management set');
    assert.ok(!surfaces.DAILY_DOMAIN_SURFACES.includes('management'), 'management is never in the daily set');
  }

  console.log('domain-surface-runtime: PASS');
} finally {
  await server.close();
}
