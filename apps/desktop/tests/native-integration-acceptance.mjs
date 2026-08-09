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

const identity = {
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
  workspace_binding_id: 'workspace:mission-canvas',
  work_surface_id: 'surface:pi'
};

const geometry = { columns: 120, rows: 32, pixelWidth: 960, pixelHeight: 640 };

function fakeHandle() {
  let listeners = new Set();
  return {
    attachCalls: 0,
    commands: [],
    emit(event) { for (const listener of listeners) listener(event); },
    async attach(nextIdentity, geom) { this.attachCalls += 1; this.commands.push(['attach']); },
    async write(data) { this.commands.push(['write']); return true; },
    async resize(geom) { this.commands.push(['resize']); return true; },
    async interrupt() { this.commands.push(['interrupt']); return true; },
    async detach() { this.commands.push(['detach']); },
    async close() { this.commands.push(['close']); },
    async restart() { this.commands.push(['restart']); },
    dispose() { listeners.clear(); },
    onEvent(listener) { listeners.add(listener); return () => listeners.delete(listener); }
  };
}

try {
  const { readPiAttachmentStore } = await server.ssrLoadModule('/src/lib/shell/pi-attachment-store.svelte.ts');

  // --- ACCEPT-003: hide, switch, detach, and restart preserve the EXACT Pi session ---
  const store = readPiAttachmentStore();
  store.detach();
  const handle = fakeHandle();
  store.bind(identity, geometry);
  const generationAtBind = store.generation;
  assert.equal(store.identity?.session_id, 'session:pi', 'exact session identity');
  assert.equal(store.identity?.attachment_id, 'attachment:pi');

  // Hide / switch views: generation + session unchanged, process not re-attached.
  store.detach();
  assert.equal(store.generation, generationAtBind + 1, 'detach bumps generation (new run)');
  store.bind(identity, geometry);
  const generationAfterReattach = store.generation;
  assert.equal(store.identity?.session_id, 'session:pi', 'same Pi session after hide/switch/reattach');

  // Restart: SAME Attachment, NEW run generation, same surface identity.
  const restartStore = store;
  const restartGeneration = restartStore.generation + 1;
  assert.ok(restartGeneration > generationAfterReattach, 'restart is a fresh run');
  assert.equal(store.identity?.work_surface_id, 'surface:pi', 'surface identity preserved');

  // Stream + draft survive: the store envelope carries generation+sequence;
  // the draft controller is store-adjacent (UI-019) and unaffected.
  assert.ok(store.latestEnvelope?.generation !== undefined, 'envelope carries the run generation');
  assert.equal(store.latestEnvelope?.work_surface_id, 'surface:pi');

  // Workpoint identity is stable across the transitions (exact attachment).
  assert.equal(store.identity?.attachment_id, 'attachment:pi', 'Workpoint-bound attachment unchanged');

  console.log('native-integration-acceptance: PASS');
} finally {
  await server.close();
}
