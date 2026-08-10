import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

const server = await createServer({
  appType: 'custom',
  server: { middlewareMode: true },
  logLevel: 'error'
});

try {
  const { render } = await server.ssrLoadModule('svelte/server');
  const { default: Harness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasLayoutHarness.svelte');
  const { default: SingleLayoutHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasSingleLayoutHarness.svelte');
  const { default: RegistryControlsHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasRegistryControlsHarness.svelte');
  const { default: TrustedRegistryHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasTrustedRegistryHarness.svelte');
  const { default: WorkspaceProfileSelector } = await server.ssrLoadModule('/src/lib/mission-canvas/WorkspaceProfileSelector.svelte');
  const generatedClientPath = fileURLToPath(new URL('../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated.ts', import.meta.url));
  const { MissionCanvasClient } = await server.ssrLoadModule(generatedClientPath);
  const { CanonicalEventHistory } = await server.ssrLoadModule('/src/lib/mission-canvas/canonical-event-history.ts');
  const { default: CustomElementHarness } = await server.ssrLoadModule('/tests/fixtures/MissionCanvasCustomElementHarness.svelte');

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

  const profile = (profileId, displayName, candidateContributionIds, installed = true) => ({
    profile_id: profileId,
    revision: 1,
    display_name: displayName,
    candidate_contribution_ids: candidateContributionIds,
    density: 'standard',
    terminology_registry_ref: `registry:terminology:${profileId}`,
    renderer_registry_ref: 'registry:renderer:builtin',
    domain_semantic_binding_registry_ref: `registry:semantics:${profileId}`,
    viability_rule_revision: 'profile-viability:v1',
    installed
  });
  const eligibleProfiles = [
    profile('general', 'General', ['contribution:pi-session']),
    profile('software', 'Software Engineering', ['contribution:pi-session', 'contribution:tasks']),
    profile('legal', 'Legal', ['contribution:document']),
    profile('markets', 'Markets', ['contribution:market-overview']),
    profile('research', 'Research', ['contribution:research']),
    profile('custom', 'Custom', ['contribution:pi-session'])
  ];
  const { body: profileSelector } = render(WorkspaceProfileSelector, {
    props: {
      profiles: eligibleProfiles,
      activeProfileId: 'software',
      onSelect: () => undefined
    }
  });
  assert.match(profileSelector, /data-profile-selector="eligible"/);
  for (const displayName of ['General', 'Software Engineering', 'Legal', 'Markets', 'Research', 'Custom']) {
    assert.match(profileSelector, new RegExp(`>${displayName}<`));
  }
  assert.doesNotMatch(profileSelector, /Unavailable|Degraded|Unsupported|No profiles/);

  const { body: ineligibleProfiles } = render(WorkspaceProfileSelector, {
    props: {
      profiles: [
        profile('software', 'Software Engineering', ['contribution:pi-session']),
        profile('research', 'Research', ['contribution:research']),
        profile('empty', 'Empty', []),
        profile('disabled', 'Disabled', ['contribution:pi-session'], false)
      ],
      activeProfileId: 'software',
      onSelect: () => undefined
    }
  });
  assert.match(ineligibleProfiles, />Software Engineering</);
  assert.doesNotMatch(ineligibleProfiles, />Empty</);
  assert.doesNotMatch(ineligibleProfiles, />Disabled</);

  const { body: invalidProfiles } = render(WorkspaceProfileSelector, {
    props: {
      profiles: [
        profile('software', 'Software Engineering', ['contribution:pi-session']),
        (() => {
          const malformed = profile('foreign', 'Foreign', ['contribution:pi-session']);
          malformed.candidate_contribution_ids = null;
          return malformed;
        })()
      ],
      activeProfileId: 'software',
      onSelect: () => undefined
    }
  });
  assert.doesNotMatch(invalidProfiles, /<select/);

  const { body: duplicateProfiles } = render(WorkspaceProfileSelector, {
    props: {
      profiles: [
        profile('software', 'Software Engineering', ['contribution:pi-session']),
        profile('software', 'Foreign duplicate', ['contribution:tasks'])
      ],
      activeProfileId: 'software',
      onSelect: () => undefined
    }
  });
  assert.doesNotMatch(duplicateProfiles, /<select/);

  const { body: staleActiveProfile } = render(WorkspaceProfileSelector, {
    props: {
      profiles: eligibleProfiles,
      activeProfileId: 'foreign',
      onSelect: () => undefined
    }
  });
  assert.doesNotMatch(staleActiveProfile, /<select/);

  const { body: blockedRegistry } = render(TrustedRegistryHarness);
  assert.match(blockedRegistry, /role="alert"/);
  assert.match(blockedRegistry, /data-unavailable-renderer="renderer:focusa-inspector@v1"/);
  assert.doesNotMatch(blockedRegistry, /data-trusted-renderer=/);
  const { body: completeRegistry } = render(TrustedRegistryHarness, { props: { complete: true } });
  assert.doesNotMatch(completeRegistry, /role="alert"/);
  assert.match(completeRegistry, /data-trusted-renderer="renderer:pi-session@v1"/);
  assert.match(completeRegistry, /data-trusted-renderer="renderer:focusa-inspector@v1"/);

  // UI-005 hostile registry cases: only exact generated binding identities may
  // select a trusted component. No malformed contribution, semantic/kind
  // mismatch, or renderer-owned prop can become a fallback renderer.
  const {
    ContributionRendererRegistry,
    resolveContributionRenderer
  } = await server.ssrLoadModule('/src/lib/mission-canvas/contribution-renderers.ts');
  const { default: ResolvedContributionRenderer } = await server.ssrLoadModule('/tests/fixtures/ResolvedContributionHarnessRenderer.svelte');
  const rendererFixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  const piContribution = rendererFixture.eligible_contributions[0];
  const hostileRegistry = new ContributionRendererRegistry([{
    rendererBindingId: 'renderer:pi-session@v1',
    semanticBindingIds: ['semantic:pi-session'],
    contributionKinds: ['focused_work_surface'],
    component: ResolvedContributionRenderer
  }]);
  const resolvedPi = resolveContributionRenderer(hostileRegistry, piContribution);
  assert.equal(resolvedPi.component, ResolvedContributionRenderer);
  assert.equal(hostileRegistry.resolveContributionRenderer(piContribution).component, ResolvedContributionRenderer);
  assert.equal(hostileRegistry.has('renderer:missing@v1'), false);
  assert.equal(hostileRegistry.resolve(null), undefined);

  // UI-006 hostile single-node cases exercise the real recursive renderer
  // boundary, not only MissionCanvasRenderer's aggregate blocked state. A
  // single node may create one host exactly when its keyed contribution and
  // trusted binding both match; malformed or omitted inputs create no layout
  // wrapper, so no parent geometry can reserve a gap.
  const singleProjection = structuredClone(rendererFixture);
  singleProjection.layout_tree = {
    node_id: 'layout:single-hostile-test',
    kind: 'single',
    contribution_id: piContribution.contribution_id
  };
  const { body: singleHost } = render(SingleLayoutHarness, {
    props: { projection: singleProjection, registry: hostileRegistry }
  });
  assert.equal((singleHost.match(/data-rendered-contribution=/g) ?? []).length, 1);
  assert.match(singleHost, /class="layout-single[^\"]*"[^>]*data-contribution-id="contribution:pi-session"/);

  const missingSingle = structuredClone(singleProjection);
  missingSingle.layout_tree.contribution_id = 'contribution:omitted';
  const { body: omittedSingle } = render(SingleLayoutHarness, {
    props: { projection: missingSingle, registry: hostileRegistry }
  });
  assert.doesNotMatch(omittedSingle, /layout-single|data-rendered-contribution=|contribution:omitted/);

  const foreignKeyedContribution = structuredClone(singleProjection);
  foreignKeyedContribution.eligible_contributions[0].contribution_id = 'contribution:foreign';
  const { body: mismatchedSingle } = render(SingleLayoutHarness, {
    props: { projection: foreignKeyedContribution, registry: hostileRegistry }
  });
  assert.doesNotMatch(mismatchedSingle, /layout-single|data-rendered-contribution=/);

  const unknownSingle = structuredClone(singleProjection);
  unknownSingle.eligible_contributions[0].renderer_binding_id = 'renderer:untrusted@9';
  const { body: unknownSingleBody } = render(SingleLayoutHarness, {
    props: { projection: unknownSingle, registry: hostileRegistry }
  });
  assert.doesNotMatch(unknownSingleBody, /layout-single|data-rendered-contribution=/);

  const semanticMismatchSingle = structuredClone(singleProjection);
  semanticMismatchSingle.eligible_contributions[0].semantic_binding_id = 'semantic:foreign';
  const { body: semanticMismatchSingleBody } = render(SingleLayoutHarness, {
    props: { projection: semanticMismatchSingle, registry: hostileRegistry }
  });
  assert.doesNotMatch(semanticMismatchSingleBody, /layout-single|data-rendered-contribution=/);

  const missingAuthoritySingle = structuredClone(singleProjection);
  delete missingAuthoritySingle.eligible_contributions[0].authority;
  const { body: missingAuthoritySingleBody } = render(SingleLayoutHarness, {
    props: { projection: missingAuthoritySingle, registry: hostileRegistry }
  });
  assert.doesNotMatch(missingAuthoritySingleBody, /layout-single|data-rendered-contribution=/);

  // UI-010 renders the canonical Tabs node as an all-or-nothing
  // presentation.  The button list is the exact Core contribution order and
  // the active child is the canonical active_contribution_id; the renderer
  // never keeps a client-local tab index or repairs an omitted child.
  const tabRegistry = new ContributionRendererRegistry([
    {
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      contributionKinds: ['focused_work_surface'],
      component: ResolvedContributionRenderer
    },
    {
      rendererBindingId: 'renderer:focusa-inspector@v1',
      semanticBindingIds: ['semantic:focusa-inspector'],
      contributionKinds: ['inspector'],
      component: ResolvedContributionRenderer
    }
  ]);
  const tabsProjection = structuredClone(rendererFixture);
  tabsProjection.layout_tree = {
    node_id: 'layout:ui-010-tabs',
    kind: 'tabs',
    active_contribution_id: 'contribution:pi-session',
    contribution_ids: [
      'contribution:pi-session',
      'contribution:focusa-inspector'
    ]
  };
  const { body: tabsBody } = render(SingleLayoutHarness, {
    props: { projection: tabsProjection, registry: tabRegistry }
  });
  assert.match(tabsBody, /class="layout-tabs[^\"]*with-strip/);
  assert.match(tabsBody, /data-layout-node="layout:ui-010-tabs"/);
  assert.equal((tabsBody.match(/role="tab"/g) ?? []).length, 2);
  assert.equal((tabsBody.match(/disabled/g) ?? []).length, 2, 'tabs without the generated layout mutation callback remain inert');
  assert.match(tabsBody, /role="tab"[^>]*aria-selected="true"/);
  assert.match(tabsBody, /role="tab"[^>]*aria-selected="false"/);
  assert.equal((tabsBody.match(/data-rendered-contribution=/g) ?? []).length, 1);
  assert.match(tabsBody, /data-rendered-contribution="contribution:pi-session"/);
  assert.doesNotMatch(tabsBody, /data-rendered-contribution="contribution:focusa-inspector"/);
  assert.match(tabsBody, /aria-controls="panel-layout:ui-010-tabs-contribution:pi-session"/);
  assert.match(tabsBody, /id="panel-layout:ui-010-tabs-contribution:pi-session"/);

  // The same node is also exercised through the production renderer/frame
  // chain, not only the focused recursive harness.
  const { default: ProductionMissionCanvasRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/MissionCanvasRenderer.svelte');
  const { body: productionTabsBody } = render(ProductionMissionCanvasRenderer, {
    props: { projection: tabsProjection, registry: tabRegistry }
  });
  assert.match(productionTabsBody, /data-layout-node="layout:ui-010-tabs"/);
  assert.match(productionTabsBody, /data-contribution-id="contribution:pi-session"/);
  assert.doesNotMatch(productionTabsBody, /data-rendered-contribution="contribution:focusa-inspector"/);

  const hostileTabs = (mutate) => {
    const candidate = structuredClone(tabsProjection);
    mutate(candidate.layout_tree, candidate);
    return render(SingleLayoutHarness, { props: { projection: candidate, registry: tabRegistry } }).body;
  };
  const tabsMustFailClosed = /layout-tabs|tab-list|role="tab"|tabpanel|data-rendered-contribution=/;
  for (const [name, mutate] of [
    ['omitted tab contribution', (node) => { node.contribution_ids[1] = 'contribution:omitted'; }],
    ['omitted active contribution', (node) => { node.active_contribution_id = 'contribution:omitted'; }],
    ['foreign keyed contribution', (_node, projection) => { projection.eligible_contributions[1].contribution_id = 'contribution:foreign'; }],
    ['unknown renderer binding', (_node, projection) => { projection.eligible_contributions[1].renderer_binding_id = 'renderer:untrusted@9'; }],
    ['semantic binding mismatch', (_node, projection) => { projection.eligible_contributions[1].semantic_binding_id = 'semantic:foreign'; }],
    ['missing contribution authority', (_node, projection) => { delete projection.eligible_contributions[1].authority; }],
    ['duplicate tab contribution', (node) => { node.contribution_ids[1] = node.contribution_ids[0]; }],
    ['duplicate node field', (node) => { node.reserved_panel = 'substitute'; }]
  ]) {
    const body = hostileTabs(mutate);
    assert.doesNotMatch(body, tabsMustFailClosed, `${name} must fail closed before tab geometry`);
  }

  const nestedOmission = structuredClone(singleProjection);
  nestedOmission.layout_tree = {
    node_id: 'layout:split-with-omission',
    kind: 'split',
    orientation: 'horizontal',
    ratio: 0.5,
    children: [
      { node_id: 'layout:present', kind: 'single', contribution_id: piContribution.contribution_id },
      { node_id: 'layout:omitted', kind: 'single', contribution_id: 'contribution:omitted' }
    ]
  };
  const { body: nestedOmissionBody } = render(SingleLayoutHarness, {
    props: { projection: nestedOmission, registry: hostileRegistry }
  });
  assert.doesNotMatch(nestedOmissionBody, /layout-split|split-child|data-rendered-contribution=/);

  // UI-007 exercises the production recursive split path with the canonical
  // orientation, ratio, and child order. The renderer consumes the Core tree;
  // it does not create a local ratio or a substitute child.
  const splitRegistry = new ContributionRendererRegistry([
    {
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      contributionKinds: ['focused_work_surface'],
      component: ResolvedContributionRenderer
    },
    {
      rendererBindingId: 'renderer:focusa-inspector@v1',
      semanticBindingIds: ['semantic:focusa-inspector'],
      contributionKinds: ['inspector'],
      component: ResolvedContributionRenderer
    }
  ]);
  const splitProjection = structuredClone(rendererFixture);
  splitProjection.layout_tree = {
    node_id: 'layout:ui-007-split',
    kind: 'split',
    orientation: 'vertical',
    ratio: 0.68,
    children: [
      { node_id: 'layout:ui-007-first', kind: 'single', contribution_id: 'contribution:pi-session' },
      { node_id: 'layout:ui-007-second', kind: 'single', contribution_id: 'contribution:focusa-inspector' }
    ]
  };
  const { body: splitBody } = render(SingleLayoutHarness, {
    props: { projection: splitProjection, registry: splitRegistry }
  });
  assert.match(splitBody, /class="layout-split[^"]*"/);
  assert.match(splitBody, /data-layout-orientation="vertical"/);
  assert.match(splitBody, /data-split-ratio="0.68"/);
  assert.equal((splitBody.match(/class="split-child/g) ?? []).length, 2);
  const firstSplitContribution = splitBody.indexOf('data-rendered-contribution="contribution:pi-session"');
  const secondSplitContribution = splitBody.indexOf('data-rendered-contribution="contribution:focusa-inspector"');
  assert.ok(firstSplitContribution >= 0 && firstSplitContribution < secondSplitContribution, 'split child order must remain canonical');

  const hostileSplit = (mutate) => {
    const candidate = structuredClone(splitProjection);
    mutate(candidate.layout_tree, candidate);
    return render(SingleLayoutHarness, { props: { projection: candidate, registry: splitRegistry } }).body;
  };
  for (const [name, mutate] of [
    ['unknown orientation', (node) => { node.orientation = 'diagonal'; }],
    ['ratio below Core minimum', (node) => { node.ratio = 0.09; }],
    ['ratio above Core maximum', (node) => { node.ratio = 0.91; }],
    ['three children', (node) => { node.children.push(structuredClone(node.children[0])); }],
    ['unknown node field', (node) => { node.direction = 'horizontal'; }],
    ['duplicate contribution', (node) => { node.children[1].contribution_id = node.children[0].contribution_id; }],
    ['duplicate node identity', (node) => { node.children[1].node_id = node.children[0].node_id; }],
    ['unknown renderer binding', (_node, projection) => { projection.eligible_contributions[1].renderer_binding_id = 'renderer:untrusted@9'; }],
    ['semantic binding mismatch', (_node, projection) => { projection.eligible_contributions[1].semantic_binding_id = 'semantic:foreign'; }],
    ['missing contribution authority', (_node, projection) => { delete projection.eligible_contributions[1].authority; }]
  ]) {
    const body = hostileSplit(mutate);
    assert.doesNotMatch(body, /layout-split|split-child|data-rendered-contribution=/, `${name} must fail closed before geometry`);
  }

  const sharedChildProjection = structuredClone(splitProjection);
  const sharedChild = sharedChildProjection.layout_tree.children[0];
  sharedChildProjection.layout_tree.children[1] = sharedChild;
  const { body: sharedChildBody } = render(SingleLayoutHarness, {
    props: { projection: sharedChildProjection, registry: splitRegistry }
  });
  assert.doesNotMatch(sharedChildBody, /layout-split|split-child|data-rendered-contribution=/, 'shared child identity must not duplicate geometry');

  // UI-008 exercises the production stack path with more than one ordered
  // child and a nested stack.  The registry deliberately uses a binding that
  // is not present in the default registry for the nested inspector so a
  // recursive call that drops the trusted registry cannot render a false
  // success.
  const stackRegistry = new ContributionRendererRegistry([
    {
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      contributionKinds: ['focused_work_surface'],
      component: ResolvedContributionRenderer
    },
    {
      rendererBindingId: 'renderer:ui-008-inspector@v1',
      semanticBindingIds: ['semantic:focusa-inspector'],
      contributionKinds: ['inspector'],
      component: ResolvedContributionRenderer
    }
  ]);
  const stackProjection = structuredClone(rendererFixture);
  stackProjection.eligible_contributions[1].renderer_binding_id = 'renderer:ui-008-inspector@v1';
  stackProjection.layout_tree = {
    node_id: 'layout:ui-008-stack',
    kind: 'stack',
    gap_token: 'cluster',
    children: [
      { node_id: 'layout:ui-008-first', kind: 'single', contribution_id: 'contribution:pi-session' },
      { node_id: 'layout:ui-008-second', kind: 'single', contribution_id: 'contribution:focusa-inspector' }
    ]
  };
  const { body: stackBody } = render(SingleLayoutHarness, {
    props: { projection: stackProjection, registry: stackRegistry }
  });
  assert.match(stackBody, /class="layout-stack[^"]*"/);
  assert.match(stackBody, /data-layout-node="layout:ui-008-stack"/);
  assert.match(stackBody, /data-gap-token="cluster"/);
  assert.equal((stackBody.match(/data-stack-index=/g) ?? []).length, 2);
  assert.equal((stackBody.match(/class="stack-child/g) ?? []).length, 2);
  const firstStackContribution = stackBody.indexOf('data-rendered-contribution="contribution:pi-session"');
  const secondStackContribution = stackBody.indexOf('data-rendered-contribution="contribution:focusa-inspector"');
  assert.ok(firstStackContribution >= 0 && firstStackContribution < secondStackContribution, 'stack child order must remain canonical');

  const nestedStackProjection = structuredClone(stackProjection);
  nestedStackProjection.layout_tree = {
    node_id: 'layout:ui-008-outer-stack',
    kind: 'stack',
    children: [
      { node_id: 'layout:ui-008-outer-first', kind: 'single', contribution_id: 'contribution:pi-session' },
      {
        node_id: 'layout:ui-008-inner-stack',
        kind: 'stack',
        gap_token: 'tight',
        children: [
          { node_id: 'layout:ui-008-inner-only', kind: 'single', contribution_id: 'contribution:focusa-inspector' }
        ]
      }
    ]
  };
  const { body: nestedStackBody } = render(SingleLayoutHarness, {
    props: { projection: nestedStackProjection, registry: stackRegistry }
  });
  assert.equal((nestedStackBody.match(/class="layout-stack[^"]*"/g) ?? []).length, 2, 'nested stacks must render recursively');
  assert.equal((nestedStackBody.match(/data-stack-index=/g) ?? []).length, 3);
  assert.match(nestedStackBody, /data-layout-node="layout:ui-008-inner-stack"[^>]*data-gap-token="tight"/);
  const outerFirst = nestedStackBody.indexOf('data-rendered-contribution="contribution:pi-session"');
  const innerOnly = nestedStackBody.indexOf('data-rendered-contribution="contribution:focusa-inspector"');
  assert.ok(outerFirst >= 0 && outerFirst < innerOnly, 'nested stack order must remain canonical');

  const hostileStack = (mutate) => {
    const candidate = structuredClone(stackProjection);
    mutate(candidate.layout_tree, candidate);
    return render(SingleLayoutHarness, { props: { projection: candidate, registry: stackRegistry } }).body;
  };
  const stackMustFailClosed = /layout-stack|stack-child|layout-single|data-rendered-contribution=/;
  for (const [name, mutate] of [
    ['empty child list', (node) => { node.children = []; }],
    ['omitted child contribution', (node) => { node.children[1].contribution_id = 'contribution:omitted'; }],
    ['foreign keyed contribution', (_node, projection) => { projection.eligible_contributions[0].contribution_id = 'contribution:foreign'; }],
    ['unknown renderer binding', (_node, projection) => { projection.eligible_contributions[1].renderer_binding_id = 'renderer:untrusted@9'; }],
    ['semantic binding mismatch', (_node, projection) => { projection.eligible_contributions[1].semantic_binding_id = 'semantic:foreign'; }],
    ['missing contribution authority', (_node, projection) => { delete projection.eligible_contributions[1].authority; }],
    ['duplicate contribution', (node) => { node.children[1].contribution_id = node.children[0].contribution_id; }],
    ['duplicate node identity', (node) => { node.children[1].node_id = node.children[0].node_id; }],
    ['invalid gap token', (node) => { node.gap_token = { attacker: 'no-css-token' }; }],
    ['empty gap token', (node) => { node.gap_token = ''; }],
    ['sparse children', (node) => {
      const sparse = new Array(2);
      sparse[0] = node.children[0];
      node.children = sparse;
    }],
    ['non-index array member', (node) => { node.children.extra = 'foreign geometry'; }],
    ['unknown stack field', (node) => { node.reserved_panel = 'substitute'; }]
  ]) {
    const body = hostileStack(mutate);
    assert.doesNotMatch(body, stackMustFailClosed, `${name} must fail closed before stack geometry`);
  }

  // UI-009 exercises the production recursive grid path. The inspector uses
  // a binding that exists only in this trusted registry, proving every grid
  // child receives the registry rather than falling back to the default map.
  const gridRegistry = new ContributionRendererRegistry([
    {
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      contributionKinds: ['focused_work_surface'],
      component: ResolvedContributionRenderer
    },
    {
      rendererBindingId: 'renderer:ui-009-inspector@v1',
      semanticBindingIds: ['semantic:focusa-inspector'],
      contributionKinds: ['inspector'],
      component: ResolvedContributionRenderer
    }
  ]);
  const gridProjection = structuredClone(rendererFixture);
  gridProjection.eligible_contributions[1].renderer_binding_id = 'renderer:ui-009-inspector@v1';
  gridProjection.layout_tree = {
    node_id: 'layout:ui-009-grid',
    kind: 'grid',
    columns: 2,
    gap_token: 'cluster',
    children: [
      { node_id: 'layout:ui-009-first', kind: 'single', contribution_id: 'contribution:pi-session' },
      { node_id: 'layout:ui-009-second', kind: 'single', contribution_id: 'contribution:focusa-inspector' }
    ]
  };
  const { body: gridBody } = render(SingleLayoutHarness, {
    props: { projection: gridProjection, registry: gridRegistry }
  });
  assert.match(gridBody, /class="layout-grid[^"]*"/);
  assert.match(gridBody, /data-layout-node="layout:ui-009-grid"/);
  assert.match(gridBody, /data-layout-columns="2"/);
  assert.match(gridBody, /data-gap-token="cluster"/);
  assert.match(gridBody, /style="--layout-columns:2"/);
  assert.equal((gridBody.match(/class="grid-child/g) ?? []).length, 2);
  assert.equal((gridBody.match(/data-grid-index=/g) ?? []).length, 2);
  const firstGridContribution = gridBody.indexOf('data-rendered-contribution="contribution:pi-session"');
  const secondGridContribution = gridBody.indexOf('data-rendered-contribution="contribution:focusa-inspector"');
  assert.ok(firstGridContribution >= 0 && firstGridContribution < secondGridContribution, 'grid child order must remain canonical');

  const nestedGridProjection = structuredClone(gridProjection);
  nestedGridProjection.layout_tree = {
    node_id: 'layout:ui-009-outer-grid',
    kind: 'grid',
    columns: 2,
    children: [
      { node_id: 'layout:ui-009-outer-first', kind: 'single', contribution_id: 'contribution:pi-session' },
      {
        node_id: 'layout:ui-009-inner-grid',
        kind: 'grid',
        columns: 1,
        gap_token: 'tight',
        children: [
          { node_id: 'layout:ui-009-inner-only', kind: 'single', contribution_id: 'contribution:focusa-inspector' }
        ]
      }
    ]
  };
  const { body: nestedGridBody } = render(SingleLayoutHarness, {
    props: { projection: nestedGridProjection, registry: gridRegistry }
  });
  assert.equal((nestedGridBody.match(/class="layout-grid[^"]*"/g) ?? []).length, 2, 'nested grids must render recursively');
  assert.equal((nestedGridBody.match(/class="grid-child/g) ?? []).length, 3);
  assert.match(nestedGridBody, /data-layout-node="layout:ui-009-inner-grid"[^>]*data-layout-columns="1"/);
  assert.match(nestedGridBody, /data-layout-node="layout:ui-009-inner-grid"[^>]*data-gap-token="tight"/);
  const nestedGridFirst = nestedGridBody.indexOf('data-rendered-contribution="contribution:pi-session"');
  const nestedGridSecond = nestedGridBody.indexOf('data-rendered-contribution="contribution:focusa-inspector"');
  assert.ok(nestedGridFirst >= 0 && nestedGridFirst < nestedGridSecond, 'nested grid order must remain canonical');

  const hostileGrid = (mutate) => {
    const candidate = structuredClone(gridProjection);
    mutate(candidate.layout_tree, candidate);
    return render(SingleLayoutHarness, { props: { projection: candidate, registry: gridRegistry } }).body;
  };
  const gridMustFailClosed = /layout-grid|grid-child|layout-single|data-rendered-contribution=/;
  for (const [name, mutate] of [
    ['empty child list', (node) => { node.children = []; }],
    ['omitted child contribution', (node) => { node.children[1].contribution_id = 'contribution:omitted'; }],
    ['foreign keyed contribution', (_node, projection) => { projection.eligible_contributions[0].contribution_id = 'contribution:foreign'; }],
    ['unknown renderer binding', (_node, projection) => { projection.eligible_contributions[1].renderer_binding_id = 'renderer:untrusted@9'; }],
    ['semantic binding mismatch', (_node, projection) => { projection.eligible_contributions[1].semantic_binding_id = 'semantic:foreign'; }],
    ['missing contribution authority', (_node, projection) => { delete projection.eligible_contributions[1].authority; }],
    ['duplicate contribution', (node) => { node.children[1].contribution_id = node.children[0].contribution_id; }],
    ['duplicate node identity', (node) => { node.children[1].node_id = node.children[0].node_id; }],
    ['zero columns', (node) => { node.columns = 0; }],
    ['too many columns', (node) => { node.columns = 13; }],
    ['fractional columns', (node) => { node.columns = 1.5; }],
    ['string columns', (node) => { node.columns = '2'; }],
    ['non-finite columns', (node) => { node.columns = Number.POSITIVE_INFINITY; }],
    ['invalid gap token', (node) => { node.gap_token = { attacker: 'no-css-token' }; }],
    ['empty gap token', (node) => { node.gap_token = ''; }],
    ['sparse children', (node) => {
      const sparse = new Array(2);
      sparse[0] = node.children[0];
      node.children = sparse;
    }],
    ['non-index array member', (node) => { node.children.extra = 'foreign geometry'; }],
    ['shared child identity', (node) => { node.children[1] = node.children[0]; }],
    ['unknown grid field', (node) => { node.reserved_panel = 'substitute'; }],
    ['unknown node kind', (node) => { node.kind = 'masonry'; }]
  ]) {
    const body = hostileGrid(mutate);
    assert.doesNotMatch(body, gridMustFailClosed, `${name} must fail closed before grid geometry`);
  }

  const unknownBinding = structuredClone(piContribution);
  unknownBinding.renderer_binding_id = 'renderer:untrusted@9';
  assert.deepEqual(hostileRegistry.resolveWithDiagnostic(unknownBinding), {
    status: 'blocked',
    diagnostic: {
      reason: 'unknown_renderer_binding',
      contributionId: piContribution.contribution_id,
      rendererBindingId: 'renderer:untrusted@9',
      semanticBindingId: piContribution.semantic_binding_id
    }
  });

  const semanticMismatch = structuredClone(piContribution);
  semanticMismatch.semantic_binding_id = 'semantic:foreign';
  assert.equal(hostileRegistry.resolve(semanticMismatch), undefined);
  assert.equal(hostileRegistry.resolveWithDiagnostic(semanticMismatch).diagnostic.reason, 'semantic_binding_mismatch');

  const kindMismatch = structuredClone(piContribution);
  kindMismatch.kind = 'inspector';
  assert.equal(hostileRegistry.resolve(kindMismatch), undefined);
  assert.equal(hostileRegistry.resolveWithDiagnostic(kindMismatch).diagnostic.reason, 'contribution_kind_mismatch');

  const missingAuthority = structuredClone(piContribution);
  delete missingAuthority.authority;
  assert.equal(hostileRegistry.resolve(missingAuthority), undefined);
  assert.equal(hostileRegistry.resolveWithDiagnostic(missingAuthority).diagnostic.reason, 'invalid_contribution');

  const boundButUnknownProjection = structuredClone(rendererFixture);
  boundButUnknownProjection.eligible_contributions[0].renderer_binding_id = 'renderer:untrusted@9';
  const { default: MissionCanvasRendererComponent } = await server.ssrLoadModule('/src/lib/mission-canvas/MissionCanvasRenderer.svelte');
  const { body: blockedUnknownRenderer } = render(MissionCanvasRendererComponent, {
    props: { projection: boundButUnknownProjection, registry: hostileRegistry }
  });
  assert.match(blockedUnknownRenderer, /data-unavailable-renderer="renderer:untrusted@9"/);
  assert.match(blockedUnknownRenderer, /data-renderer-resolution-reason="unknown_renderer_binding"/);
  assert.doesNotMatch(blockedUnknownRenderer, /data-trusted-renderer=/);

  const semanticBindings = ['semantic:pi-session'];
  const rendererProps = { nested: { owner: 'trusted registry' } };
  const immutableRegistry = new ContributionRendererRegistry([{
    rendererBindingId: 'renderer:immutable@1',
    semanticBindingIds: semanticBindings,
    contributionKinds: ['focused_work_surface'],
    component: ResolvedContributionRenderer,
    componentProps: rendererProps
  }]);
  semanticBindings[0] = 'semantic:foreign';
  rendererProps.nested.owner = 'caller mutation';
  const immutableContribution = structuredClone(piContribution);
  immutableContribution.renderer_binding_id = 'renderer:immutable@1';
  const immutableResolved = immutableRegistry.resolve(immutableContribution);
  assert.equal(immutableResolved.componentProps.nested.owner, 'trusted registry');
  assert.throws(() => new ContributionRendererRegistry([{
    rendererBindingId: 'renderer:unsafe-props@1',
    component: ResolvedContributionRenderer,
    componentProps: { contribution: 'caller-controlled' }
  }]), /cannot override contribution/);
  assert.throws(() => new ContributionRendererRegistry([{
    rendererBindingId: 'renderer:not-a-component@1',
    component: {}
  }]), /Invalid trusted renderer component/);
  assert.throws(() => new ContributionRendererRegistry([
    { rendererBindingId: 'renderer:duplicate@1', component: ResolvedContributionRenderer },
    { rendererBindingId: 'renderer:duplicate@1', component: ResolvedContributionRenderer }
  ]), /duplicate renderer binding/);

  const { body: customElements } = render(CustomElementHarness);
  assert.match(customElements, /<fixture-pi-session[^>]*data-contribution-id="contribution:pi-session"/);
  assert.match(customElements, /<fixture-focusa-inspector[^>]*data-contribution-id="contribution:focusa-inspector"/);
  const { trustedGeneratedSurfaceRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/generated-surface-renderer.ts');
  assert.throws(() => trustedGeneratedSurfaceRenderer({ rendererBindingId: '', semanticBindingIds: [], snapshotResolver: async () => [] }));
  const generatedEntry = trustedGeneratedSurfaceRenderer({
    rendererBindingId: 'renderer:fixture-generated@v1',
    semanticBindingIds: ['semantic:fixture-generated'],
    snapshotResolver: async () => []
  });
  assert.equal(generatedEntry.rendererBindingId, 'renderer:fixture-generated@v1');
  assert.deepEqual(generatedEntry.contributionKinds, ['generated_surface']);

  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  const authorityOf = (value) => ({
    workstream: structuredClone(value.workstream),
    continuity_id: value.continuity_id ?? null,
    attachment: structuredClone(value.attachment ?? null),
    workspace_binding_id: value.workspace_binding_id ?? null,
    runtime_object: structuredClone(value.runtime_object ?? null),
    work_surface_id: value.work_surface_id ?? value.focused_work_surface_id ?? null
  });
  const fixtureAuthority = authorityOf(fixture);
  const { validateLayoutIntegrity } = await server.ssrLoadModule('/src/lib/mission-canvas/layout-references.ts');
  assert.deepEqual(validateLayoutIntegrity({
    kind: 'tabs',
    node_id: 'layout:invalid-tabs',
    contribution_ids: ['contribution:a'],
    active_contribution_id: 'contribution:foreign'
  }).map((issue) => issue.code), ['invalid_active_tab']);
  assert.deepEqual(validateLayoutIntegrity({
    kind: 'split',
    node_id: 'layout:duplicate-root',
    direction: 'horizontal',
    ratio: 0.5,
    children: [
      { kind: 'single', node_id: 'layout:duplicate-a', contribution_id: 'contribution:a' },
      { kind: 'single', node_id: 'layout:duplicate-b', contribution_id: 'contribution:a' }
    ]
  }).map((issue) => issue.code), ['duplicate_contribution']);
  const { DEFAULT_CONTRIBUTION_REGISTRY } = await server.ssrLoadModule('/src/lib/mission-canvas/default-contribution-registry.ts');
  for (const contribution of fixture.eligible_contributions) {
    assert.ok(DEFAULT_CONTRIBUTION_REGISTRY.resolve(contribution), `default registry missing ${contribution.renderer_binding_id}`);
  }
  const queueFixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/one-queue-projection.json', import.meta.url), 'utf8'));
  const previewPrompt = queueFixture.eligible_contributions.find((contribution) => contribution.kind === 'prompt_editor');
  assert.equal(previewPrompt.data_ref.kind, 'canvas_draft');
  assert.equal(queueFixture.operation_bindings[0].operation_id, 'focusa.agent_execution.prompt');
  assert.equal(queueFixture.operation_bindings[0].enabled, true);
  for (const contribution of queueFixture.eligible_contributions) {
    assert.ok(DEFAULT_CONTRIBUTION_REGISTRY.resolve(contribution), `default registry missing ${contribution.renderer_binding_id}`);
  }

  const historyContribution = {
    ...structuredClone(fixture.eligible_contributions[0]),
    contribution_id: 'contribution:history',
    kind: 'generated_surface',
    semantic_binding_id: 'semantic:history',
    renderer_binding_id: 'renderer:history@v1',
    data_ref: {
      kind: 'history',
      ref: 'history:workspace',
      revision: fixture.projection_revision,
      freshness: 'current'
    },
    accessibility: {
      label: 'Workspace History',
      description: 'Canonical cursor-scoped projection event history',
      landmark_role: 'region',
      focus_semantic_id: 'focus:history',
      keyboard_operation_ids: []
    },
    operation_ids: []
  };
  const historyProjection = {
    ...structuredClone(fixture),
    eligible_contributions: [historyContribution],
    operation_bindings: [],
    layout_tree: {
      node_id: 'layout:history',
      kind: 'single',
      contribution_id: historyContribution.contribution_id
    }
  };
  assert.ok(
    DEFAULT_CONTRIBUTION_REGISTRY.resolve(historyContribution),
    `default registry missing ${historyContribution.renderer_binding_id}`
  );

  const historyEvent = {
    ...fixtureAuthority,
    event_cursor: 'event:90',
    event_id: 'event:history-1',
    event_kind: 'projection_rehydrated',
    evidence_refs: ['evidence:history-1'],
    layout_revision: fixture.layout_revision + 1,
    occurred_at: '2026-08-08T00:00:01Z',
    payload_ref: 'history:payload:1',
    projection_revision: fixture.projection_revision + 1,
    receipt_refs: ['receipt:history-1']
  };
  const foreignHistoryEvent = {
    ...historyEvent,
    event_id: 'event:history-foreign',
    event_cursor: 'event:91',
    workstream: { ...structuredClone(fixtureAuthority.workstream), workstream_id: 'ws:foreign' },
    attachment: {
      ...structuredClone(fixtureAuthority.attachment),
      workstream: { ...structuredClone(fixtureAuthority.workstream), workstream_id: 'ws:foreign' }
    }
  };
  const staleHistoryEvent = {
    ...historyEvent,
    event_id: 'event:history-stale',
    event_cursor: 'event:40'
  };
  const malformedHistoryEvent = {
    ...historyEvent,
    event_id: 'event:history-malformed',
    event_cursor: 'cursor:not-a-number'
  };
  const historyRender = CanonicalEventHistory.render({
    projection: historyProjection,
    authority: fixtureAuthority,
    events: [historyEvent, foreignHistoryEvent, staleHistoryEvent, malformedHistoryEvent],
    maxRows: 1
  });
  assert.equal(historyRender.rows.length, 1);
  assert.equal(historyRender.rows[0].event_id, 'event:history-1');
  assert.equal(historyRender.rejected.some(({ reason }) => reason === 'foreign_event_scope'), true);
  assert.equal(historyRender.rejected.some(({ reason }) => reason === 'event_cursor_stale'), true);
  assert.equal(historyRender.rejected.some(({ reason }) => reason === 'invalid_event_cursor'), true);

  const staleRevisionHistoryEvent = {
    ...historyEvent,
    event_id: 'event:history-stale-revision',
    event_cursor: 'event:92',
    projection_revision: fixture.projection_revision - 1
  };
  const staleRevisionRender = CanonicalEventHistory.render({
    projection: historyProjection,
    authority: fixtureAuthority,
    events: [staleRevisionHistoryEvent]
  });
  assert.equal(staleRevisionRender.rows.length, 0);
  assert.equal(staleRevisionRender.rejected.some(({ reason }) => reason === 'projection_revision_stale'), true);

  const invalidAuthorityRender = CanonicalEventHistory.render({
    projection: historyProjection,
    authority: { workstream: {} },
    events: [historyEvent]
  });
  assert.equal(invalidAuthorityRender.rows.length, 0);
  assert.equal(invalidAuthorityRender.rejected[0].reason.startsWith('invalid_projection_authority'), true);

  const blockedHistoryContributionBinding = {
    ...historyContribution,
    contribution_id: 'contribution:history-missing',
    renderer_binding_id: 'renderer:history@v999'
  };
  const blockedResolution = DEFAULT_CONTRIBUTION_REGISTRY.resolveWithDiagnostic(blockedHistoryContributionBinding);
  assert.equal(blockedResolution.status, 'blocked');

  assert.equal(
    CanonicalEventHistory.render({
      projection: historyProjection,
      authority: fixtureAuthority,
      events: [historyEvent],
      maxRows: 1
    }).rows[0].event_id,
    'event:history-1'
  );
  assert.equal(CanonicalEventHistory.render(historyProjection, [historyEvent], fixtureAuthority, 1).rows[0].event_id, 'event:history-1');

  const malformedProjectionCursorRender = CanonicalEventHistory.render({
    ...historyProjection,
    durable_event_cursor: 'cursor:bad'
  },
  [historyEvent],
  fixtureAuthority,
  1);
  assert.equal(malformedProjectionCursorRender.rejected.some(({ reason }) => reason === 'invalid_projection_cursor'), true);

  const { default: ActivityNavigation } = await server.ssrLoadModule('/src/lib/mission-canvas/ActivityNavigation.svelte');
  const { body: activityNavigation } = render(ActivityNavigation, {
    props: {
      activities: [
        { activity_mode_id: 'overview', display_name: 'Overview', revision: 1, candidate_contribution_ids: [], viability_rule_revision: '1' },
        { activity_mode_id: 'evidence', display_name: 'Evidence', revision: 1, candidate_contribution_ids: [], viability_rule_revision: '1' }
      ],
      activeActivityModeId: 'evidence',
      onSelect: () => undefined
    }
  });
  assert.match(activityNavigation, /aria-label="Activities"/);
  assert.match(activityNavigation, /data-activity-mode-id="overview"/);
  assert.match(activityNavigation, /aria-current="page" data-activity-mode-id="evidence"/);

  const { default: MissionCanvasRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/MissionCanvasRenderer.svelte');
  const { body: productionProjection } = render(MissionCanvasRenderer, { props: { projection: fixture, registry: DEFAULT_CONTRIBUTION_REGISTRY } });
  assert.match(productionProjection, /aria-label="Mission Canvas context"/);
  assert.match(productionProjection, />Workspace</);
  assert.match(productionProjection, />Activity</);
  assert.match(productionProjection, /aria-label="Work Surfaces"/);
  assert.match(productionProjection, /data-work-surface-id="surface:pi"/);
  assert.match(productionProjection, />Pi Session</);
  assert.match(productionProjection, /data-layout-node="layout:root"/);
  assert.match(productionProjection, /data-layout-orientation="horizontal"/);
  assert.match(productionProjection, /data-split-ratio="0.7"/);
  assert.equal((productionProjection.match(/class="split-child/g) ?? []).length, 2);
  assert.doesNotMatch(productionProjection, /Renderer unavailable/);

  const { body: historyProductionProjection } = render(MissionCanvasRenderer, {
    props: {
      projection: historyProjection,
      registry: DEFAULT_CONTRIBUTION_REGISTRY
    }
  });
  assert.match(historyProductionProjection, /data-contribution-id="contribution:history"/);
  assert.match(historyProductionProjection, /Workspace History/);
  assert.doesNotMatch(historyProductionProjection, /Renderer unavailable/);

  const { default: CanonicalEventHistoryContribution } = await server.ssrLoadModule('/src/lib/mission-canvas/contributions/CanonicalEventHistoryContribution.svelte');
  const { body: blockedHistoryContribution } = render(CanonicalEventHistoryContribution, {
    props: {
      contribution: historyContribution,
      projection: historyProjection
    }
  });
  assert.match(blockedHistoryContribution, /data-history-status="blocked"/);
  assert.match(blockedHistoryContribution, /History stream is not available\./);

  // The real MissionCanvasRenderer path must preserve the canonical split
  // without creating a client-local resize control or mutating projection
  // state during render. Any layout mutation remains an explicit generated
  // operation owned by DesktopMissionCanvasRuntime.
  assert.doesNotMatch(productionProjection, /<input[^>]+type="range"|draggable="true"/);

  const twoQueueFixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/two-queue-projection.json', import.meta.url), 'utf8'));
  assert.deepEqual(
    twoQueueFixture.eligible_contributions.filter(({ kind }) => kind.endsWith('_queue')).map(({ kind }) => kind),
    ['steering_queue', 'follow_up_queue']
  );
  assert.deepEqual(validateLayoutIntegrity(twoQueueFixture.layout_tree), []);
  for (const contribution of twoQueueFixture.eligible_contributions) {
    assert.ok(DEFAULT_CONTRIBUTION_REGISTRY.resolve(contribution), `two-queue registry missing ${contribution.renderer_binding_id}`);
  }
  const { body: twoQueueProjection } = render(MissionCanvasRenderer, { props: { projection: twoQueueFixture, registry: DEFAULT_CONTRIBUTION_REGISTRY } });
  assert.match(twoQueueProjection, /data-work-rail-ref="work-rail:project"/);
  assert.match(twoQueueProjection, /data-queue-kind="steering_queue"/);
  assert.match(twoQueueProjection, /data-queue-kind="follow_up_queue"/);
  assert.match(twoQueueProjection, />Steering Queue</);
  assert.match(twoQueueProjection, />Follow-up Queue</);
  assert.match(twoQueueProjection, /queue-region/);
  assert.doesNotMatch(twoQueueProjection, /Renderer unavailable/);

  const { default: WorkRailContribution } = await server.ssrLoadModule('/src/lib/mission-canvas/contributions/WorkRailContribution.svelte');
  const workRail = {
    ...twoQueueFixture.eligible_contributions.find(({ kind }) => kind === 'steering_queue'),
    contribution_id: 'contribution:work-rail',
    kind: 'work_rail',
    data_ref: { kind: 'work_rail', ref: 'work-rail:project', revision: 4, freshness: 'current' },
    operation_ids: [],
    accessibility: {
      focus_semantic_id: 'semantic:work-rail',
      label: 'Focusa Work Rail',
      description: 'Canonical project work for the focused Work Surface',
      landmark_role: 'region'
    }
  };
  const { body: workRailProjection } = render(WorkRailContribution, { props: { contribution: workRail, projection: twoQueueFixture } });
  assert.match(workRailProjection, /data-work-rail-ref="work-rail:project"/);
  assert.match(workRailProjection, />Focusa Work Rail</);
  assert.match(workRailProjection, /data-work-rail-revision="4"/);
  assert.match(workRailProjection, /Projection r12/);
  assert.doesNotMatch(workRailProjection, /New Workpoint/);
  const requestedUrls = [];
  const { MissionCanvasHttpTransport, MissionCanvasTransportError } = await server.ssrLoadModule('/src/lib/mission-canvas/http-transport.ts');
  const transport = new MissionCanvasHttpTransport('http://127.0.0.1:8787/', async (url) => {
    requestedUrls.push(String(url));
    return new Response(JSON.stringify(fixture), { status: 200, headers: { 'Content-Type': 'application/json' } });
  });
  const transported = await transport.request('focusa.mission_canvas.projection.get', { ...fixtureAuthority });
  assert.equal(transported.projection_digest, fixture.projection_digest);
  assert.match(requestedUrls[0], /^http:\/\/127\.0\.0\.1:8787\/v1\/mission-canvas\/projection\?workstream=/);
  assert.match(requestedUrls[0], /attachment=/);
  await assert.rejects(
    () => transport.request('focusa.mission_canvas.not-generated'),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'operation_unavailable'
  );
  await assert.rejects(
    () => transport.request('focusa.mission_canvas.profile.get', { profile_id: 'software', ...fixtureAuthority }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_response:')
  );
  assert.match(requestedUrls.at(-1), /\/v1\/mission-canvas\/profiles\/software\?workstream=/);
  assert.doesNotMatch(requestedUrls.at(-1), /profile_id=/);
  const arrayTransport = new MissionCanvasHttpTransport('http://127.0.0.1:8787', async () =>
    new Response('[]', { status: 200, headers: { 'Content-Type': 'application/json' } })
  );
  assert.deepEqual(await arrayTransport.request('focusa.mission_canvas.activity.list', { ...fixtureAuthority }), []);

  // Profile selection remains a generated, exact-Workstream mutation. The
  // Desktop selector never calls a route or invents a revision; the transport
  // owns operation metadata, optimistic concurrency, and fail-closed watermarks.
  const profileSelectRequests = [];
  let profileSelectResponse = structuredClone(fixture);
  profileSelectResponse.projection_revision = fixture.projection_revision + 1;
  profileSelectResponse.layout_revision = fixture.layout_revision + 1;
  profileSelectResponse.durable_event_cursor = 'event:42';
  const profileSelectTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (url, init) => {
      profileSelectRequests.push({ url: String(url), init });
      return new Response(JSON.stringify(profileSelectResponse), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['focusa.mission_canvas.profile.select'],
    'actor:desktop',
    'authority:desktop'
  );
  const profileSelectInput = {
    ...structuredClone(fixtureAuthority),
    selection_id: 'research',
    expected_projection_revision: fixture.projection_revision,
    idempotency_key: 'idempotency:profile-select'
  };
  const profileSelectClient = new MissionCanvasClient(profileSelectTransport);
  const selectedProjection = await profileSelectClient.profileSelect(profileSelectInput);
  assert.equal(selectedProjection.projection_revision, fixture.projection_revision + 1);
  const profileSelectRequest = profileSelectRequests[0];
  assert.equal(new URL(profileSelectRequest.url).pathname, '/v1/mission-canvas/profiles/select');
  assert.equal(profileSelectRequest.init.method, 'POST');
  assert.equal(profileSelectRequest.init.headers['X-Focusa-Permissions'], 'mission_canvas:write');
  assert.equal(profileSelectRequest.init.headers['X-Focusa-Capabilities'], 'focusa.mission_canvas.profile.select');
  assert.equal(profileSelectRequest.init.headers['If-Match'], String(fixture.projection_revision));
  assert.equal(profileSelectRequest.init.headers['Idempotency-Key'], profileSelectInput.idempotency_key);
  assert.equal(JSON.parse(profileSelectRequest.init.body).selection_id, 'research');
  assert.deepEqual(JSON.parse(profileSelectRequest.init.body).workstream, fixtureAuthority.workstream);

  profileSelectResponse = structuredClone(fixture);
  profileSelectResponse.projection_revision = fixture.projection_revision;
  profileSelectResponse.layout_revision = fixture.layout_revision + 2;
  profileSelectResponse.durable_event_cursor = 'event:43';
  await assert.rejects(
    () => profileSelectClient.profileSelect(profileSelectInput),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_revision'
  );

  profileSelectResponse = structuredClone(fixture);
  profileSelectResponse.projection_revision = fixture.projection_revision + 2;
  profileSelectResponse.layout_revision = fixture.layout_revision + 2;
  profileSelectResponse.durable_event_cursor = 'event:41';
  await assert.rejects(
    () => profileSelectClient.profileSelect(profileSelectInput),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'stale_projection_cursor'
  );

  profileSelectResponse = structuredClone(fixture);
  profileSelectResponse.projection_revision = fixture.projection_revision + 3;
  profileSelectResponse.layout_revision = fixture.layout_revision + 3;
  profileSelectResponse.durable_event_cursor = 'event:44';
  profileSelectResponse.workstream.workstream_id = 'ws:foreign-profile-select';
  profileSelectResponse.attachment.workstream.workstream_id = 'ws:foreign-profile-select';
  await assert.rejects(
    () => profileSelectClient.profileSelect(profileSelectInput),
    (error) => error instanceof MissionCanvasTransportError && error.message === 'foreign_projection_scope'
  );

  let profileSelectCalls = 0;
  const missingProfileSelectAuthority = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async () => {
      profileSelectCalls += 1;
      return new Response(JSON.stringify(fixture), { status: 200 });
    },
    undefined,
    30_000,
    ['mission_canvas:write'],
    ['focusa.mission_canvas.profile.select'],
    'actor:desktop',
    'authority:desktop'
  );
  await assert.rejects(
    () => new MissionCanvasClient(missingProfileSelectAuthority).profileSelect({
      selection_id: 'research',
      expected_projection_revision: fixture.projection_revision,
      idempotency_key: 'idempotency:missing-authority'
    }),
    (error) => error instanceof MissionCanvasTransportError && error.message.startsWith('invalid_workstream_identity:')
  );
  assert.equal(profileSelectCalls, 0, 'profile selection without Workstream authority must fail before HTTP');

  const { MissionCanvasProjectionController } = await server.ssrLoadModule('/src/lib/mission-canvas/projection-controller.svelte.ts');
  let response = structuredClone(fixture);
  const controller = new MissionCanvasProjectionController(async () => structuredClone(response));
  await controller.load(fixtureAuthority);
  assert.equal(controller.state.kind, 'ready');

  response = structuredClone(fixture);
  response.attachment.session_id = 'foreign-session';
  await controller.load(fixtureAuthority);
  assert.equal(controller.state.kind, 'blocked');
  assert.equal(controller.state.reason, 'projection_scope_mismatch');

  response = structuredClone(fixture);
  await controller.load(fixtureAuthority);
  response.projection_revision -= 1;
  await controller.load(fixtureAuthority);
  assert.equal(controller.state.kind, 'stale');
  assert.equal(controller.state.reason, 'projection_revision_regressed');

  let refreshResolver;
  let refreshResponse = structuredClone(fixture);
  const preservingController = new MissionCanvasProjectionController(async () => {
    if (!refreshResolver) return structuredClone(refreshResponse);
    return new Promise((resolve) => { refreshResolver = resolve; });
  });
  await preservingController.load(fixtureAuthority);
  refreshResolver = () => {};
  const refreshing = preservingController.load(fixtureAuthority);
  assert.equal(preservingController.state.kind, 'refreshing');
  assert.equal(preservingController.state.projection.projection_digest, fixture.projection_digest);
  refreshResolver(structuredClone({ ...fixture, projection_revision: fixture.projection_revision + 1 }));
  await refreshing;
  assert.equal(preservingController.state.kind, 'ready');

  // PROFILE-002 exercises the bounded profile-memory runtime source. The
  // controller receives exact generated DTOs and persists semantic placement
  // preferences through the generated operation client; it never constructs a
  // layout tree or reserves geometry for an absent contribution.
  const {
    GeneratedProfileMemoryTransport,
    MissionCanvasProfileMemoryController
  } = await server.ssrLoadModule('/src/lib/mission-canvas/profile-memory-controller.ts');
  const memoryBinding = {
    scope: structuredClone(fixtureAuthority),
    profileId: 'software',
    activityModeId: 'overview',
    viewportClass: 'standard'
  };
  const profileMemory = (revision = 4, overrides = {}) => ({
    ...structuredClone(fixtureAuthority),
    memory_id: 'layout-memory:software:overview:standard',
    profile_id: 'software',
    activity_mode_id: 'overview',
    viewport_class: 'standard',
    placements: [{
      contribution_id: 'contribution:pi-session',
      preferred_regions: ['primary'],
      preferred_order: 0,
      minimum_span: 3,
      maximum_span: 8,
      preferred_adjacency: [],
      last_compatible_layout_node_id: 'layout:primary'
    }],
    absent_contribution_ids: ['contribution:empty-work-rail'],
    focused_semantic_target: 'focus:pi-session',
    memory_revision: revision,
    idempotency_key: `memory:profile:${revision}`,
    updated_at: '2026-08-07T00:00:00Z',
    ...structuredClone(overrides)
  });
  const profileMemoryReceipt = (revision = 5, overrides = {}) => ({
    ...structuredClone(fixtureAuthority),
    receipt_id: `recomposition-receipt:layout-memory:${revision}`,
    accepted: true,
    projection_revision: revision,
    layout_revision: revision,
    projection_digest: `sha256:${'a'.repeat(64)}`,
    event_cursor: `event:${revision + 40}`,
    evidence_id: `recomposition-evidence:layout-memory:${revision}`,
    idempotency_key: 'idempotency:profile-memory',
    issued_at: '2026-08-07T00:00:01Z',
    ...structuredClone(overrides)
  });
  const canonicalMemory = profileMemory();
  const profileMemoryRequests = [];
  const profileMemoryHttpTransport = new MissionCanvasHttpTransport(
    'http://127.0.0.1:8787',
    async (url, init) => {
      profileMemoryRequests.push({ url: String(url), init });
      return init.method === 'GET'
        ? new Response(JSON.stringify(canonicalMemory), { status: 200, headers: { 'Content-Type': 'application/json' } })
        : new Response(JSON.stringify(profileMemoryReceipt(5)), { status: 200, headers: { 'Content-Type': 'application/json' } });
    },
    undefined,
    30_000,
    ['mission_canvas:read', 'mission_canvas:write'],
    [],
    'actor:desktop',
    'authority:desktop'
  );
  const profileMemoryClient = new MissionCanvasClient(profileMemoryHttpTransport);
  const generatedMemoryTransport = new GeneratedProfileMemoryTransport(profileMemoryClient);
  const profileMemoryController = new MissionCanvasProfileMemoryController(generatedMemoryTransport);
  await profileMemoryController.load(memoryBinding);
  assert.equal(profileMemoryController.state.kind, 'ready');
  assert.equal(profileMemoryController.state.memory.memory_revision, 4);
  assert.deepEqual(profileMemoryController.state.memory.absent_contribution_ids, ['contribution:empty-work-rail']);
  assert.equal('layout_tree' in profileMemoryController.state.memory, false);

  await profileMemoryController.update({
    ...structuredClone(profileMemoryController.state.memory),
    idempotency_key: 'idempotency:profile-memory'
  });
  assert.equal(profileMemoryController.state.kind, 'ready');
  assert.equal(profileMemoryController.state.memory.memory_revision, 5);
  assert.deepEqual(
    profileMemoryController.state.memory.absent_contribution_ids,
    ['contribution:empty-work-rail'],
    'disappearing optional contributions retain semantic return memory'
  );
  assert.equal('layout_tree' in profileMemoryController.state.memory, false);
  assert.equal(new URL(profileMemoryRequests[0].url).pathname, '/v1/mission-canvas/layout-memory');
  assert.equal(profileMemoryRequests[0].init.method, 'GET');
  assert.equal(new URL(profileMemoryRequests[1].url).pathname, '/v1/mission-canvas/layout-memory');
  assert.equal(profileMemoryRequests[1].init.method, 'POST');
  const profileMemoryBody = JSON.parse(profileMemoryRequests[1].init.body);
  assert.deepEqual(profileMemoryBody.workstream, fixtureAuthority.workstream);
  assert.deepEqual(profileMemoryBody.attachment, fixtureAuthority.attachment);
  assert.equal(profileMemoryBody.memory_revision, 4);
  assert.deepEqual(profileMemoryBody.absent_contribution_ids, ['contribution:empty-work-rail']);
  assert.equal('layout_tree' in profileMemoryBody, false);
  assert.equal('eligible_contributions' in profileMemoryBody, false);

  // Foreign authority, profile identity, and missing authority never fall
  // back to the last local profile or to a project/continuity approximation.
  const foreignMemory = profileMemory(4, {
    workstream: { ...structuredClone(fixtureAuthority.workstream), workstream_id: 'ws:foreign' },
    attachment: {
      ...structuredClone(fixtureAuthority.attachment),
      workstream: { ...structuredClone(fixtureAuthority.workstream), workstream_id: 'ws:foreign' }
    }
  });
  const foreignController = new MissionCanvasProfileMemoryController({
    get: async () => foreignMemory,
    update: async (value) => value
  });
  await foreignController.load(memoryBinding);
  assert.equal(foreignController.state.kind, 'error');
  assert.equal(foreignController.state.reason, 'foreign_profile_memory');

  const foreignProfileController = new MissionCanvasProfileMemoryController({
    get: async () => profileMemory(4, {
      profile_id: 'research',
      memory_id: 'layout-memory:research:overview:standard'
    }),
    update: async (value) => value
  });
  await foreignProfileController.load(memoryBinding);
  assert.equal(foreignProfileController.state.kind, 'error');
  assert.equal(foreignProfileController.state.reason, 'foreign_profile_memory');

  let missingAuthorityCalls = 0;
  const missingAuthorityController = new MissionCanvasProfileMemoryController({
    get: async () => {
      missingAuthorityCalls += 1;
      return profileMemory();
    },
    update: async (value) => value
  });
  await missingAuthorityController.load({
    ...memoryBinding,
    scope: undefined
  });
  assert.equal(missingAuthorityController.state.kind, 'blocked');
  assert.equal(missingAuthorityController.state.reason, 'invalid_workstream_authority');
  assert.equal(missingAuthorityCalls, 0);

  // A stale read is retained as a conflict only when a canonical prior memory
  // exists. The regressed response is never adopted.
  let staleRead = false;
  const staleController = new MissionCanvasProfileMemoryController({
    get: async () => staleRead ? profileMemory(3) : profileMemory(4),
    update: async (value) => value
  });
  await staleController.load(memoryBinding);
  staleRead = true;
  await staleController.load(memoryBinding);
  assert.equal(staleController.state.kind, 'conflict');
  assert.equal(staleController.state.reason, 'stale_profile_memory_revision');
  assert.equal(staleController.state.memory.memory_revision, 4);

  // Invalid update scope and stale update results fail closed before any
  // semantic preference can replace the last canonical memory.
  let foreignUpdateCalls = 0;
  const updateController = new MissionCanvasProfileMemoryController({
    get: async () => profileMemory(4),
    update: async (value) => {
      foreignUpdateCalls += 1;
      return value;
    }
  });
  await updateController.load(memoryBinding);
  await updateController.update({
    ...profileMemory(4),
    workstream: { ...structuredClone(fixtureAuthority.workstream), workstream_id: 'ws:foreign' },
    attachment: {
      ...structuredClone(fixtureAuthority.attachment),
      workstream: { ...structuredClone(fixtureAuthority.workstream), workstream_id: 'ws:foreign' }
    }
  });
  assert.equal(updateController.state.kind, 'conflict');
  assert.equal(updateController.state.reason, 'foreign_profile_memory');
  assert.equal(updateController.state.memory.memory_revision, 4);
  assert.equal(foreignUpdateCalls, 0);

  const staleUpdateController = new MissionCanvasProfileMemoryController({
    get: async () => profileMemory(4),
    update: async () => profileMemory(3)
  });
  await staleUpdateController.load(memoryBinding);
  await staleUpdateController.update({ ...profileMemory(4), idempotency_key: 'idempotency:stale-update' });
  assert.equal(staleUpdateController.state.kind, 'conflict');
  assert.equal(staleUpdateController.state.reason, 'stale_profile_memory_revision');
  assert.equal(staleUpdateController.state.memory.memory_revision, 4);

  let cursorUpdateCount = 0;
  const cursorController = new MissionCanvasProfileMemoryController({
    get: async () => profileMemory(4),
    update: async (value) => ({ ...structuredClone(value), memory_revision: value.memory_revision + 1 }),
    updateWithReceipt: async (value) => {
      const nextRevision = value.memory_revision + 1;
      const nextMemory = {
        ...structuredClone(value),
        memory_revision: nextRevision,
        updated_at: '2026-08-07T00:00:02Z'
      };
      const receipt = profileMemoryReceipt(nextRevision, {
        idempotency_key: value.idempotency_key,
        event_cursor: cursorUpdateCount++ === 0 ? 'event:45' : 'event:44'
      });
      return { memory: nextMemory, receipt };
    }
  });
  await cursorController.load(memoryBinding);
  await cursorController.update({ ...profileMemory(4), idempotency_key: 'idempotency:cursor-first' });
  assert.equal(cursorController.state.kind, 'ready');
  await cursorController.update({
    ...structuredClone(cursorController.state.memory),
    idempotency_key: 'idempotency:cursor-stale'
  });
  assert.equal(cursorController.state.kind, 'conflict');
  assert.equal(cursorController.state.reason, 'stale_profile_memory_cursor');
  assert.equal(cursorController.state.memory.memory_revision, 5);

  const unavailableController = new MissionCanvasProfileMemoryController({
    get: async () => { throw new Error('operation_unavailable'); },
    update: async (value) => value
  });
  await unavailableController.load(memoryBinding);
  assert.equal(unavailableController.state.kind, 'error');
  assert.equal(unavailableController.state.reason, 'operation_unavailable');
  assert.equal(unavailableController.state.memory, undefined);

  // Older in-flight responses cannot overwrite a newer exact binding.
  let releaseOldLoad;
  const raceController = new MissionCanvasProfileMemoryController({
    get: async (binding) => binding.profileId === 'software'
      ? new Promise((resolve) => { releaseOldLoad = resolve; })
      : profileMemory(7, {
        profile_id: 'research',
        memory_id: 'layout-memory:research:overview:standard'
      }),
    update: async (value) => value
  });
  const oldLoad = raceController.load(memoryBinding);
  await raceController.load({ ...memoryBinding, profileId: 'research' });
  assert.equal(raceController.state.kind, 'ready');
  assert.equal(raceController.state.binding.profileId, 'research');
  releaseOldLoad(profileMemory(8));
  await oldLoad;
  assert.equal(raceController.state.kind, 'ready');
  assert.equal(raceController.state.binding.profileId, 'research');

  const draftFixture = {
    ...structuredClone(fixtureAuthority),
    content: 'canonical draft',
    content_sha256: `sha256:${'0'.repeat(64)}`,
    draft_id: 'draft:fixture',
    draft_revision: 4,
    idempotency_key: 'fixture-draft-v4',
    owner: 'canvas_prompt_editor',
    recipient_ref: 'recipient:pi',
    sync_state: 'synchronized',
    updated_at: '2026-08-04T00:00:00Z'
  };
  const binding = { ...structuredClone(fixtureAuthority), draftId: draftFixture.draft_id, recipientRef: 'recipient:pi' };
  let draftResponse = structuredClone(draftFixture);
  const { MissionCanvasDraftController } = await server.ssrLoadModule('/src/lib/mission-canvas/draft-controller.svelte.ts');
  const draftController = new MissionCanvasDraftController({
    get: async () => structuredClone(draftResponse),
    sync: async () => structuredClone(draftResponse)
  });
  await draftController.load(binding);
  assert.equal(draftController.state.kind, 'ready');
  draftResponse.attachment.session_id = 'foreign-session';
  await draftController.sync('preserve this local edit');
  assert.equal(draftController.state.kind, 'conflict');
  assert.equal(draftController.state.reason, 'foreign_draft_binding');
  assert.equal(draftController.state.localContent, 'preserve this local edit');

  let synchronizedBody;
  const generatedClient = {
    draftGet: async (input) => {
      assert.equal(input.draft_id, draftFixture.draft_id);
      return structuredClone(draftFixture);
    },
    draftSync: async (body) => {
      synchronizedBody = structuredClone(body);
      return { ...structuredClone(body), draft_revision: body.draft_revision + 1, sync_state: 'synchronized' };
    },
    recipientResolve: async (input) => ({ schema: 'focusa.mission_canvas.recipient_resolution.v1', ...structuredClone(input), routable: true })
  };
  const { GeneratedDraftTransport, resolveRecipient } = await server.ssrLoadModule('/src/lib/mission-canvas/generated-draft-transport.ts');
  const generatedDraftTransport = new GeneratedDraftTransport(generatedClient);
  assert.equal((await generatedDraftTransport.get(binding)).draft_id, draftFixture.draft_id);
  await generatedDraftTransport.sync({
    ...binding,
    baseDraft: draftFixture,
    content: 'canonical prompt',
    expectedDraftRevision: draftFixture.draft_revision,
    idempotencyKey: 'idempotency:prompt-sync'
  });
  assert.equal(synchronizedBody.owner, 'canvas_prompt_editor');
  assert.equal(synchronizedBody.idempotency_key, 'idempotency:prompt-sync');
  assert.match(synchronizedBody.content_sha256, /^[a-f0-9]{64}$/);
  assert.equal((await resolveRecipient(generatedClient, fixtureAuthority, 'recipient:pi')).recipient_ref, 'recipient:pi');

  const eventFixture = {
    event_cursor: 'cursor:1',
    event_id: 'event:1',
    event_kind: 'capability_changed',
    evidence_refs: [],
    layout_revision: fixture.layout_revision + 1,
    occurred_at: '2026-08-04T00:00:01Z',
    payload_ref: 'fixture:event:1',
    projection_revision: fixture.projection_revision + 1,
    receipt_refs: [],
    ...structuredClone(fixtureAuthority)
  };
  let eventResponse = [eventFixture];
  let persistedCursor;
  const { MissionCanvasEventClient } = await server.ssrLoadModule('/src/lib/mission-canvas/event-client.ts');
  const eventClient = new MissionCanvasEventClient(
    { eventsStream: async () => structuredClone(eventResponse) },
    fixtureAuthority,
    { load: () => persistedCursor, persist: (_authority, cursor) => { persistedCursor = cursor; } }
  );
  const acceptedEvents = await eventClient.poll();
  assert.equal(acceptedEvents.accepted.length, 1);
  assert.equal(persistedCursor, 'cursor:1');
  const foreignEvent = structuredClone(eventFixture);
  foreignEvent.event_id = 'event:foreign';
  foreignEvent.event_cursor = 'cursor:2';
  foreignEvent.workstream.workstream_id = 'ws:foreign';
  foreignEvent.attachment.workstream.workstream_id = 'ws:foreign';
  eventResponse = [foreignEvent];
  const rejectedEvents = await eventClient.poll();
  assert.equal(rejectedEvents.rejected[0].reason, 'foreign_event_scope');
  assert.equal(persistedCursor, 'cursor:1');

  // Hostile Desktop event-client cases stay on the generated transport path:
  // no local scope repair, cursor inference, or partial authority handoff.
  const immutableScope = structuredClone(fixtureAuthority);
  const immutableInputs = [];
  const immutableClient = new MissionCanvasEventClient(
    { eventsStream: async (input) => { immutableInputs.push(input); return []; } },
    immutableScope,
    { load: () => undefined, persist: () => { throw new Error('empty tail must not persist'); } }
  );
  immutableScope.workstream.workstream_id = 'ws:mutated-after-client-creation';
  await immutableClient.poll();
  assert.equal(immutableInputs[0].workstream.workstream_id, fixtureAuthority.workstream.workstream_id);

  let invalidScopeCalls = 0;
  const invalidScopeClient = new MissionCanvasEventClient(
    { eventsStream: async () => { invalidScopeCalls += 1; return []; } },
    {},
    { load: () => undefined, persist: () => undefined }
  );
  await assert.rejects(() => invalidScopeClient.poll(), /invalid_workstream_scope/);
  assert.equal(invalidScopeCalls, 0, 'invalid Workstream authority must fail before eventsStream');

  let invalidCursorCalls = 0;
  const invalidCursorClient = new MissionCanvasEventClient(
    { eventsStream: async () => { invalidCursorCalls += 1; return []; } },
    fixtureAuthority,
    { load: () => 'event:not-a-number', persist: () => undefined }
  );
  await assert.rejects(() => invalidCursorClient.poll(), /invalid_persisted_cursor/);
  assert.equal(invalidCursorCalls, 0, 'invalid durable cursor must fail before eventsStream');

  const malformedEventPayload = structuredClone(eventFixture);
  delete malformedEventPayload.event_cursor;
  const malformedEventClient = new MissionCanvasEventClient(
    { eventsStream: async () => [malformedEventPayload] },
    fixtureAuthority,
    { load: () => undefined, persist: () => { throw new Error('malformed event must not persist'); } }
  );
  const malformedEvents = await malformedEventClient.poll();
  assert.equal(malformedEvents.accepted.length, 0);
  assert.match(malformedEvents.rejected[0].reason, /invalid_event/);

  const duplicateEventClient = new MissionCanvasEventClient(
    { eventsStream: async () => [structuredClone(eventFixture), structuredClone(eventFixture)] },
    fixtureAuthority,
    { load: () => undefined, persist: (_scope, cursor) => assert.equal(cursor, 'cursor:1') }
  );
  const duplicateEvents = await duplicateEventClient.poll();
  assert.equal(duplicateEvents.accepted.length, 1);
  assert.equal(duplicateEvents.rejected[0].reason, 'duplicate_event');

  const foreignDirectEvent = structuredClone(eventFixture);
  foreignDirectEvent.event_id = 'event:foreign-direct-client';
  foreignDirectEvent.event_cursor = 'cursor:2';
  foreignDirectEvent.workstream.workstream_id = 'ws:foreign';
  foreignDirectEvent.attachment.workstream.workstream_id = 'ws:foreign';
  const foreignEventClient = new MissionCanvasEventClient(
    { eventsStream: async () => [foreignDirectEvent] },
    fixtureAuthority,
    { load: () => undefined, persist: () => { throw new Error('foreign event must not persist'); } }
  );
  const foreignEvents = await foreignEventClient.poll();
  assert.equal(foreignEvents.accepted.length, 0);
  assert.equal(foreignEvents.rejected[0].reason, 'foreign_event_scope');

  const unavailableEventClient = new MissionCanvasEventClient(
    {},
    fixtureAuthority,
    { load: () => undefined, persist: () => undefined }
  );
  await assert.rejects(() => unavailableEventClient.poll(), /operation_unavailable/);

  let persistAttempts = 0;
  let persistFailureInputs = [];
  const persistFailureClient = new MissionCanvasEventClient(
    { eventsStream: async (input) => {
      persistFailureInputs.push(input.after_cursor);
      return persistAttempts === 0 ? [structuredClone(eventFixture)] : [];
    } },
    fixtureAuthority,
    { load: () => undefined, persist: () => { persistAttempts += 1; throw new Error('storage offline'); } }
  );
  await assert.rejects(() => persistFailureClient.poll(), /event_cursor_persist_failed/);
  assert.equal(persistAttempts, 1);
  // A failed cursor write does not advance in-memory state; a retry remains a
  // replay from the old cursor rather than silently skipping an event.
  await persistFailureClient.poll();
  assert.deepEqual(persistFailureInputs, [undefined, undefined]);

  const {
    MissionCanvasInvalidationController,
    event: projectionEventClassifier
  } = await server.ssrLoadModule('/src/lib/mission-canvas/invalidation-controller.ts');
  const projectionRevision = {
    projectionRevision: fixture.projection_revision,
    layoutRevision: fixture.layout_revision,
    durableEventCursor: fixture.durable_event_cursor,
    authority: fixtureAuthority
  };
  const projectionEvent = {
    ...structuredClone(eventFixture),
    event_id: 'event:projection-refresh',
    event_cursor: 'event:42',
    projection_revision: fixture.projection_revision + 1,
    layout_revision: fixture.layout_revision + 1
  };
  const projectionBatch = { accepted: [projectionEvent], rejected: [], cursor: projectionEvent.event_cursor };

  const classified = projectionEventClassifier.classify(projectionEvent, projectionRevision, fixtureAuthority);
  assert.equal(classified.refresh, true);
  assert.equal(classified.reason, 'projection_revision_advanced');
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:stale-cursor', event_cursor: fixture.durable_event_cursor },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'stale_event_cursor'
  );
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:wrong-namespace', event_cursor: 'cursor:42' },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'event_cursor_namespace_mismatch'
  );
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:stale-revision', event_cursor: 'event:43', layout_revision: fixture.layout_revision - 1 },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'layout_revision_stale'
  );
  assert.equal(
    projectionEventClassifier.classify(
      { ...projectionEvent, event_id: 'event:routine-pi', event_cursor: 'event:44', event_kind: 'pi_message_updated', projection_revision: fixture.projection_revision + 2, layout_revision: fixture.layout_revision + 2 },
      projectionRevision,
      fixtureAuthority
    ).reason,
    'event_not_projection_relevant'
  );

  const foreignProjectionEvent = structuredClone(projectionEvent);
  foreignProjectionEvent.event_id = 'event:foreign-invalidation';
  foreignProjectionEvent.event_cursor = 'event:45';
  foreignProjectionEvent.workstream.workstream_id = 'ws:foreign-invalidation';
  foreignProjectionEvent.attachment.workstream.workstream_id = 'ws:foreign-invalidation';
  assert.equal(
    projectionEventClassifier.classify(foreignProjectionEvent, projectionRevision, fixtureAuthority).reason,
    'foreign_event_scope'
  );

  const missingAuthorityInvalidation = new MissionCanvasInvalidationController(() => {
    throw new Error('missing authority must not refresh');
  }, 1000);
  assert.equal(
    missingAuthorityInvalidation.coalesce(projectionBatch, {
      projectionRevision: fixture.projection_revision,
      layoutRevision: fixture.layout_revision,
      durableEventCursor: fixture.durable_event_cursor
    }),
    false
  );
  await missingAuthorityInvalidation.flush();
  missingAuthorityInvalidation.dispose();

  // An omitted contribution remains a Core-owned composition decision.  The
  // event may refresh the canonical projection, but Desktop never creates a
  // replacement contribution or layout node.
  const omittedContributionEvent = structuredClone(projectionEvent);
  omittedContributionEvent.event_id = 'event:empty-omission';
  omittedContributionEvent.event_cursor = 'event:46';
  omittedContributionEvent.event_kind = 'contribution_omitted';
  delete omittedContributionEvent.contribution_id;
  assert.equal(
    projectionEventClassifier.classify(omittedContributionEvent, projectionRevision, fixtureAuthority).refresh,
    true
  );

  let reloads = 0;
  const invalidation = new MissionCanvasInvalidationController(() => { reloads += 1; }, 1000);
  assert.equal(invalidation.coalesce(projectionBatch, projectionRevision, fixtureAuthority), true);
  const secondProjectionEvent = {
    ...structuredClone(projectionEvent),
    event_id: 'event:projection-refresh-2',
    event_cursor: 'event:43',
    projection_revision: fixture.projection_revision + 2,
    layout_revision: fixture.layout_revision + 2
  };
  assert.equal(invalidation.coalesce({ accepted: [secondProjectionEvent], rejected: [] }, projectionRevision, fixtureAuthority), true);
  await invalidation.flush();
  assert.equal(reloads, 1, 'event bursts must cause one bounded refresh');
  assert.equal(invalidation.coalesce({ accepted: [{ ...projectionEvent, event_id: 'event:routine-only', event_kind: 'pi_tool_completed', event_cursor: 'event:47' }], rejected: [] }, projectionRevision, fixtureAuthority), false);
  invalidation.dispose();

  let serializedReloads = 0;
  let releaseReload;
  const serialReload = new Promise((resolve) => { releaseReload = resolve; });
  const serializedInvalidation = new MissionCanvasInvalidationController(async () => {
    serializedReloads += 1;
    if (serializedReloads === 1) await serialReload;
  }, 0);
  serializedInvalidation.coalesce(projectionBatch, projectionRevision, fixtureAuthority);
  const firstRefresh = serializedInvalidation.flush();
  serializedInvalidation.coalesce({ accepted: [secondProjectionEvent] }, projectionRevision, fixtureAuthority);
  assert.equal(serializedReloads, 1, 'refreshes must not overlap');
  releaseReload();
  await firstRefresh;
  await serializedInvalidation.flush();
  assert.equal(serializedReloads, 2, 'events received during refresh must be retained');
  serializedInvalidation.dispose();

  let disposedReloads = 0;
  const disposedInvalidation = new MissionCanvasInvalidationController(() => { disposedReloads += 1; }, 0);
  disposedInvalidation.coalesce(projectionBatch, projectionRevision, fixtureAuthority);
  disposedInvalidation.dispose();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(disposedReloads, 0, 'disposed invalidations must not refresh');

  const {
    WorkSurfaceInventory,
    SessionAttachmentIdentity,
    default: SessionInventoryContribution
  } = await server.ssrLoadModule('/src/lib/mission-canvas/contributions/SessionInventoryContribution.svelte');
  const inventorySource = structuredClone(fixture.eligible_contributions.find(({ kind }) => kind === 'focused_work_surface'));
  inventorySource.operation_ids = ['focusa.mission_canvas.rich_host.focus'];
  const inventoryContribution = {
    ...structuredClone(inventorySource),
    contribution_id: 'contribution:silent-sessions',
    kind: 'inspector',
    semantic_binding_id: 'semantic:silent-sessions',
    renderer_binding_id: 'renderer:silent-sessions@v1',
    data_ref: { kind: 'session_inventory', ref: 'session-inventory:canonical', revision: fixture.projection_revision },
    accessibility: {
      label: 'Multiplexed Runtime Inventory',
      description: 'Canonical sessions and attachments',
      landmark_role: 'region',
      focus_semantic_id: 'semantic:silent-sessions'
    }
  };
  const inventoryProjection = {
    ...structuredClone(fixture),
    eligible_contributions: [inventoryContribution, inventorySource],
    operation_bindings: [{
      operation_id: 'focusa.mission_canvas.rich_host.focus',
      target_contribution_id: inventorySource.contribution_id,
      enabled: true,
      authority_ref: 'authority:pi-session',
      confirmation: 'none',
      disabled_reason_ref: null
    }]
  };
  const exactInventorySurface = {
    identity: {
      workstream: structuredClone(fixture.workstream),
      continuity_id: fixture.continuity_id,
      attachment: structuredClone(fixture.attachment),
      runtime_object: structuredClone(fixture.runtime_object),
      work_surface_id: fixture.work_surface_id
    },
    workSurfaceId: fixture.work_surface_id,
    displayName: 'Active Pi session',
    kind: 'pi_session',
    projectRoot: '/example/focusa',
    continuityId: fixture.continuity_id,
    workpointId: 'workpoint:fixture',
    workItemRef: 'work:fixture',
    instanceId: fixture.attachment.instance_id,
    sessionId: fixture.attachment.session_id,
    attachmentId: fixture.attachment.attachment_id,
    role: 'active',
    rendererId: 'renderer:pi-session@v1',
    pinned: true,
    groupId: 'group:fixture',
    splitGroupId: '',
    lifecycleState: 'open',
    semanticActivity: 'coding',
    health: 'healthy',
    unreadEventCount: 2,
    pendingApprovalCount: 1,
    conflictCount: 0,
    blockerCount: 0,
    writerLeaseRef: 'lease:fixture',
    worktreeRef: 'worktree:fixture',
    browserIsolationClass: 'not-applicable'
  };
  const resolvedInventoryRenderer = DEFAULT_CONTRIBUTION_REGISTRY.resolve(inventoryContribution);
  assert.ok(resolvedInventoryRenderer, 'session inventory renderer must be registry-owned');
  assert.equal(typeof WorkSurfaceInventory?.render, 'function', 'session inventory render seam exists');
  assert.equal(typeof SessionAttachmentIdentity?.render, 'function', 'session attachment identity render seam exists');
  assert.equal(
    WorkSurfaceInventory.render(inventoryProjection, inventoryContribution, [exactInventorySurface])[0]?.inventoryScope,
    'local',
    'local inventory is labeled local when exact identity is available'
  );
  assert.equal(
    SessionAttachmentIdentity.render(inventoryContribution.authority, inventoryProjection).inventoryScope,
    'local',
    'session attachment identity render reports local for exact authority'
  );
  const { body: inventoryBody } = render(SessionInventoryContribution, {
    props: {
      contribution: inventoryContribution,
      projection: inventoryProjection,
      workSurfaces: [exactInventorySurface],
      onOperation: () => undefined
    }
  });
  assert.match(inventoryBody, /data-session-inventory="session-inventory:canonical"/);
  assert.match(inventoryBody, /data-workstream-id="ws:mission-canvas"/);
  assert.match(inventoryBody, /data-attachment-id="attachment:pi"/);
  assert.match(inventoryBody, /data-session-id="session:pi"/);
  assert.match(inventoryBody, /data-instance-id="instance:pi"/);
  assert.match(inventoryBody, /data-work-surface-id="surface:pi"/);
  assert.match(inventoryBody, /data-bindable="true"/);
  assert.match(inventoryBody, /<button[^>]*>focus<\/button>/);
  assert.match(inventoryBody, /data-session-inventory-mode="local"/);
  assert.match(inventoryBody, /Visual focus is local and not canonical activity\./);

  const legacyInventorySurface = {
    ...structuredClone(exactInventorySurface),
    identity: undefined,
    workSurfaceId: 'surface:legacy',
    projectRoot: '/example/focusa',
    continuityId: fixture.continuity_id,
    instanceId: 'instance:legacy',
    sessionId: 'session:legacy',
    attachmentId: 'attachment:legacy'
  };
  const { body: legacyInventoryBody } = render(SessionInventoryContribution, {
    props: {
      contribution: inventoryContribution,
      projection: inventoryProjection,
      workSurfaces: [legacyInventorySurface]
    }
  });
  assert.match(legacyInventoryBody, /data-row-state="compatibility"/);
  assert.match(legacyInventoryBody, /data-bindable="false"/);
  assert.match(legacyInventoryBody, /Compatibility data only/);
  assert.match(legacyInventoryBody, /data-session-inventory-mode="aggregate"/);
  assert.match(legacyInventoryBody, /aggregate inventory/);
  assert.doesNotMatch(legacyInventoryBody, /data-workstream-id=/);
  assert.doesNotMatch(legacyInventoryBody, /data-attachment-id=/);
  assert.doesNotMatch(legacyInventoryBody, /<button/);

  const foreignInventorySurface = structuredClone(exactInventorySurface);
  foreignInventorySurface.identity.workstream.workstream_id = 'ws:foreign';
  foreignInventorySurface.identity.attachment.workstream.workstream_id = 'ws:foreign';
  const { body: foreignInventoryBody } = render(SessionInventoryContribution, {
    props: {
      contribution: inventoryContribution,
      projection: inventoryProjection,
      workSurfaces: [foreignInventorySurface]
    }
  });
  assert.doesNotMatch(foreignInventoryBody, /data-session-inventory-row|ws:foreign|session:pi/);

  const duplicateInventorySurface = structuredClone(exactInventorySurface);
  const { body: duplicateInventoryBody } = render(SessionInventoryContribution, {
    props: {
      contribution: inventoryContribution,
      projection: inventoryProjection,
      workSurfaces: [exactInventorySurface, duplicateInventorySurface],
      onOperation: () => undefined
    }
  });
  assert.match(duplicateInventoryBody, /data-row-state="quarantined"/);
  assert.match(duplicateInventoryBody, /data-quarantine-reason="duplicate_identity"/);
  assert.match(duplicateInventoryBody, /data-bindable="false"/);
  assert.doesNotMatch(duplicateInventoryBody, /<button/);

  const staleProjection = structuredClone(inventoryProjection);
  staleProjection.eligible_contributions[1].freshness.status = 'stale';
  const { body: staleInventoryBody } = render(SessionInventoryContribution, {
    props: {
      contribution: staleProjection.eligible_contributions[0],
      projection: staleProjection,
      workSurfaces: [exactInventorySurface],
      onOperation: () => undefined
    }
  });
  assert.match(staleInventoryBody, /data-bindable="false"/);
  assert.doesNotMatch(staleInventoryBody, /<button/);

  const staleWatermarkProjection = structuredClone(inventoryProjection);
  staleWatermarkProjection.projection_revision = -1;
  const { body: staleWatermarkBody } = render(SessionInventoryContribution, {
    props: {
      contribution: staleWatermarkProjection.eligible_contributions[0],
      projection: staleWatermarkProjection,
      workSurfaces: [exactInventorySurface],
      onOperation: () => undefined
    }
  });
  assert.match(staleWatermarkBody, /data-row-state="quarantined"/);
  assert.match(staleWatermarkBody, /data-quarantine-reason="stale_revision_or_cursor"/);
  assert.doesNotMatch(staleWatermarkBody, /<button/);

  const aggregateInventoryContribution = {
    ...structuredClone(inventoryContribution),
    authority: {
      ...structuredClone(inventoryContribution.authority),
      attachment: null,
      work_surface_id: null
    }
  };
  const aggregateInventoryProjection = {
    ...structuredClone(inventoryProjection),
    attachment: null,
    work_surface_id: null,
    focused_work_surface_id: null,
    eligible_contributions: [aggregateInventoryContribution],
    operation_bindings: []
  };
  const { body: aggregateInventoryBody } = render(SessionInventoryContribution, {
    props: {
      contribution: aggregateInventoryContribution,
      projection: aggregateInventoryProjection
    }
  });
  assert.match(aggregateInventoryBody, /data-row-state="compatibility"/);
  assert.match(aggregateInventoryBody, /data-bindable="false"/);
  assert.match(aggregateInventoryBody, /data-session-inventory-mode="aggregate"/);
  assert.doesNotMatch(aggregateInventoryBody, /data-attachment-id=/);
  assert.doesNotMatch(aggregateInventoryBody, /<button/);
  assert.equal(
    SessionAttachmentIdentity.render(aggregateInventoryContribution.authority, aggregateInventoryProjection).inventoryScope,
    'aggregate',
    'aggregate authority renders as aggregate inventory mode'
  );

  const emptyContribution = {
    ...structuredClone(aggregateInventoryContribution),
    contribution_id: 'contribution:empty-session-inventory',
    semantic_binding_id: 'semantic:empty-session-inventory',
    renderer_binding_id: 'renderer:other@v1',
    data_ref: { kind: 'empty', ref: 'empty:session-inventory', revision: 1 }
  };
  const emptyProjection = { ...structuredClone(aggregateInventoryProjection), eligible_contributions: [emptyContribution] };
  const { body: emptyInventoryBody } = render(SessionInventoryContribution, {
    props: { contribution: emptyContribution, projection: emptyProjection }
  });
  assert.doesNotMatch(emptyInventoryBody, /data-session-inventory|data-session-inventory-row/);

  console.log('Mission Canvas runtime: PASS (layout, renderer, generated profile-memory controller, transport, projection, draft, event authority, invalidation coalescing, and hostile session inventory identity cases)');
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
}
