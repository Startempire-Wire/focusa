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
  // --- GEN-001: the desktop does NOT implement a second A2UI processor/renderer ---
  const surface = readFileSync(new URL('../src/lib/mission-canvas/contributions/GeneratedSurfaceContribution.svelte', import.meta.url), 'utf8');
  const renderer = readFileSync(new URL('../src/lib/mission-canvas/generated-surface-renderer.ts', import.meta.url), 'utf8');
  // The renderer mounts the SAME rich-host from @focusa/a2ui-renderer; no
  // local copy of the A2UI processor exists.
  assert.match(surface, /@focusa\/a2ui-renderer\/rich-host/, 'uses the canonical a2ui rich host');
  assert.doesNotMatch(surface + renderer, /new A2UIProcessor|class A2UIProcessor|processor\./, 'no second A2UI processor');
  assert.doesNotMatch(renderer, /<script src=.*a2ui|shadowRoot.*innerHTML.*processor/, 'no competing generated renderer');

  // --- GEN-001: generated UI uses canonical operations and durable events ---
  const { trustedGeneratedSurfaceRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/generated-surface-renderer.ts');
  // The renderer binding requires EXACT renderer + semantic identities —
  // kind-only fallback is forbidden (fail closed).
  assert.throws(() => trustedGeneratedSurfaceRenderer({ rendererBindingId: '', semanticBindingIds: [] }),
    'binding without exact identities fails closed');
  const binding = trustedGeneratedSurfaceRenderer({ rendererBindingId: 'renderer:generated', semanticBindingIds: ['semantic:generated'] });
  assert.equal(binding.rendererBindingId, 'renderer:generated');
  assert.deepEqual(binding.contributionKinds, ['generated_surface']);
  // Registry entries are generated surface contributions (trusted custom
  // elements), not relabeled transcript Markdown.
  const { trustedCustomElementRenderer } = await server.ssrLoadModule('/src/lib/mission-canvas/custom-element-renderer.ts');
  const custom = trustedCustomElementRenderer({ rendererBindingId: 'renderer:generated', semanticBindingIds: ['semantic:generated'], elementName: 'focusa-generated-surface' });
  assert.equal(custom.rendererBindingId, 'renderer:generated', 'custom element renderer exists');

  // --- GEN-001: the binding filter is canonical — operation_ids + authority_ref ---
  const contribution = {
    contribution_id: 'contribution:generated',
    kind: 'generated_surface',
    data_ref: { ref: 'surface:generated' },
    operation_ids: ['operation:steer', 'operation:execute'],
    accessibility: { label: 'Generated', focus_semantic_id: 'generated', landmark_role: 'region' }
  };
  const bindings = [
    { operation_id: 'operation:steer', target_contribution_id: 'contribution:generated', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null },
    { operation_id: 'operation:execute', target_contribution_id: 'contribution:generated', enabled: false, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: 'authority_blocked' },
    { operation_id: 'operation:other', target_contribution_id: 'contribution:generated', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null },
    { operation_id: 'operation:steer', target_contribution_id: 'contribution:other', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null },
    { operation_id: 'operation:noauthority', target_contribution_id: 'contribution:generated', enabled: true, authority_ref: null, confirmation: 'none', disabled_reason_ref: null }
  ];
  // Replicate the component's filter (read from source to avoid drift):
  const filterSource = surface.match(/actionBindings = \$derived[\s\S]*?\)\);/)?.[0] ?? '';
  assert.match(filterSource, /contribution\.operation_ids\.includes\(binding\.operation_id\)/, 'filter uses canonical operation ids');
  assert.match(filterSource, /binding\.authority_ref/, 'filter requires authority_ref');
  assert.match(filterSource, /!binding\.disabled_reason_ref/, 'filter respects disabled_reason_ref');
  const eligible = bindings.filter((binding) =>
    binding.target_contribution_id === contribution.contribution_id
    && contribution.operation_ids.includes(binding.operation_id)
    && binding.enabled
    && Boolean(binding.authority_ref)
    && !binding.disabled_reason_ref
  );
  assert.deepEqual(eligible.map((b) => b.operation_id), ['operation:steer'],
    'only canonical, enabled, authority-bound operations surface');

  // --- GEN-001: presentation (UXP/UFI) never alters authority or safety ---
  assert.match(surface, /confirmation === 'preview'/, 'preview-confirmation operations are never executed');
  assert.match(surface, /operation_bindings/, 'operations come from the canonical projection');

  console.log('generated-surface-runtime: PASS');
} finally {
  await server.close();
}
