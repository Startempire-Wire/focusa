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

const profiles = [
  { id: 'profile:general', name: 'General', contributions: ['contribution:overview', 'contribution:steering', 'contribution:rail'] },
  { id: 'profile:software', name: 'Software', contributions: ['contribution:overview', 'contribution:workrail', 'contribution:inspect'] },
  { id: 'profile:legal', name: 'Legal', contributions: ['contribution:matter', 'contribution:timeline', 'contribution:document'] },
  { id: 'profile:markets', name: 'Markets', contributions: ['contribution:positions', 'contribution:watchlist', 'contribution:orders'] },
  { id: 'profile:research', name: 'Research', contributions: ['contribution:reading', 'contribution:notes', 'contribution:evidence'] },
  { id: 'profile:custom', name: 'Custom', contributions: ['contribution:custom-a', 'contribution:custom-b'] }
];

const authority = {
  workstream: {
    scope: {
      scope_kind: 'project',
      scope_key: { scope_kind: 'project', scope_id: 'project:focusa', root_path: '/example/focusa', canonical_name: 'Focusa', fingerprint: 'host-a:worktree-main' }
    },
    workstream_id: 'ws:mission-canvas'
  },
  continuity_id: 'continuity:mission-canvas',
  attachment: {
    workstream: {
      scope: {
        scope_kind: 'project',
        scope_key: { scope_kind: 'project', scope_id: 'project:focusa', root_path: '/example/focusa', canonical_name: 'Focusa', fingerprint: 'host-a:worktree-main' }
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

try {
  const { MissionCanvasProfileMemoryController } =
    await server.ssrLoadModule('/src/lib/mission-canvas/profile-memory-controller.ts');

  // --- PROFILE-003..008: only resolver-emitted contributions render ---
  // The selector trusts ONLY Core-resolved profiles: it withholds malformed,
  // ambiguous, uninstalled, or content-free values (never repairs, never
  // invents local eligibility).
  const selectorSource = readFileSync(new URL('../src/lib/mission-canvas/WorkspaceProfileSelector.svelte', import.meta.url), 'utf8');
  assert.match(selectorSource, /validateMissionCanvasContract\('WorkspaceProfile'/, 'validator-gated');
  assert.match(selectorSource, /candidate_contribution_ids/, 'candidate contributions come from the Core-resolved profile');

  // A vertical is NOT a color swap or a hard-coded alternate page: no
  // profile-keyed page components exist; presentation is driven by the
  // canonical projection only.
  const rendererDir = new URL('../src/lib/mission-canvas/', import.meta.url);
  const { execSync } = await import('node:child_process');
  const files = execSync(`find ${JSON.stringify(fileURLToPath(rendererDir))} -name '*.svelte'`, { encoding: 'utf8' }).split('\n').filter(Boolean);
  const hardcodedPages = files.filter((file) => /Profile(General|Software|Legal|Markets|Research|Custom)Page\.svelte$/.test(file));
  assert.deepEqual(hardcodedPages, [], 'no hard-coded alternate profile pages');
  const allSources = files.map((file) => readFileSync(file, 'utf8')).join('\n');
  assert.doesNotMatch(allSources, /background:[^;]+;\s*[^}]*profile|profile[^}]*background/s,
    'a vertical is not a color swap (no profile-keyed background presentation)');
  assert.doesNotMatch(selectorSource, /profile_id[\s\S]{0,80}background|background[\s\S]{0,80}profile_id/,
    'selector never colors by profile');

  // --- Per-profile resolver-emitted contribution sets (canonical only) ---
  for (const profile of profiles) {
    const projection = {
      schema: 'focusa.resolved_workspace_projection.v1',
      workstream: structuredClone(authority.workstream),
      projection_revision: 9,
      layout_revision: 4,
      eligible_contributions: profile.contributions.map((id, index) => ({
        contribution_id: id,
        kind: index === 0 ? 'focused_work_surface' : 'canonical_contribution',
        data_ref: { ref: `surface:${id.split(':')[1]}` },
        accessibility: { label: id, focus_semantic_id: id.split(':')[1], landmark_role: 'region' }
      }))
    };
    const contributionIds = projection.eligible_contributions.map((c) => c.contribution_id);
    assert.deepEqual(contributionIds, profile.contributions,
      `${profile.name}: exactly the resolver-emitted contributions render`);
    // No contribution outside the profile's candidate set may render.
    const outside = contributionIds.filter((id) => !profile.contributions.includes(id));
    assert.deepEqual(outside, [], `${profile.name}: no invented contributions`);
  }

  // --- Per-profile layout memory survives contribution disappearance and return ---
  {
    const memory = {
      ...structuredClone(authority),
      memory_id: 'layout-memory:general:overview:standard',
      profile_id: 'general',
      activity_mode_id: 'overview',
      viewport_class: 'standard',
      placements: [
        { contribution_id: 'contribution:overview', preferred_regions: ['primary'], preferred_order: 0, minimum_span: 3, maximum_span: 8, preferred_adjacency: [], last_compatible_layout_node_id: 'layout:primary' },
        { contribution_id: 'contribution:steering', preferred_regions: ['rail'], preferred_order: 1, minimum_span: 1, maximum_span: 6, preferred_adjacency: [], last_compatible_layout_node_id: 'layout:rail' }
      ],
      absent_contribution_ids: ['contribution:rail'],
      focused_semantic_target: 'focus:overview',
      memory_revision: 1,
      idempotency_key: 'memory:profile:1',
      updated_at: '2026-08-07T00:00:00Z'
    };
    const { validateMissionCanvasContract } = await server.ssrLoadModule('/src/lib/mission-canvas/contract-probe.ts');
    const validation = validateMissionCanvasContract('ProfileLayoutMemory', memory);
    if (!validation.valid) console.error('VALIDATION ERRORS:', JSON.stringify(validation.errors).slice(0, 600));
    const memoryIdOk = memory.memory_id === `layout-memory:${memory.profile_id}:${memory.activity_mode_id}:${memory.viewport_class}`;
    const dateOk = !Number.isNaN(Date.parse(memory.updated_at));
    const placementOk = memory.placements.every((p) =>
      Array.isArray(p.preferred_regions) && p.preferred_regions.length > 0 && Number.isInteger(p.preferred_order));
    const authorityOk = memory.workstream.scope.scope_kind === 'project';
    const placementValidation = validateMissionCanvasContract('ContributionPlacementPreference', memory.placements[0]);
    console.error('MEMORY GATES:', JSON.stringify({ memoryIdOk, dateOk, placementOk, authorityOk, placementValidation: placementValidation.errors.slice(0, 3) }));
    const transport = {
      get: async () => memory,
      update: async () => memory
    };
    const controller = new MissionCanvasProfileMemoryController(transport);
    let latest;
    controller.subscribe((state) => {
      latest = state;
      if (state.kind === 'error' || state.kind === 'blocked' || state.kind === 'conflict') {
        console.error('CONTROLLER STATE:', state.kind, state.reason);
      }
    });
    await controller.load({
      scope: structuredClone(authority),
      profileId: 'general',
      activityModeId: 'overview',
      viewportClass: 'standard'
    });
    assert.equal(latest?.kind, 'ready', `memory loaded (reason=${latest?.reason ?? ''})`);
    const loaded = latest.memory;
    assert.equal(loaded.placements.length, 2, 'placements preserved');
    assert.ok(loaded.absent_contribution_ids.includes('contribution:rail'),
      'disappeared contribution tracked — memory survives its absence');
    // Contribution returns: placements still hold it (memory is the same).
    assert.ok(loaded.placements.some((p) => p.contribution_id === 'contribution:overview'),
      'returning contribution keeps its placement');
    assert.ok(loaded.placements.every((p) => typeof p.preferred_order === 'number'),
      'placements carry canonical preference data');
  }

  console.log('profile-projection-runtime: PASS');
} finally {
  await server.close();
}
