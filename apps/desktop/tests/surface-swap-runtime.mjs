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

const identity = {
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
  workspace_binding_id: 'workspace:mission-canvas',
  work_surface_id: 'surface:pi'
};

const geometry = { columns: 120, rows: 32, pixelWidth: 960, pixelHeight: 640 };

function fakeHandle() {
  const commands = [];
  let listeners = new Set();
  return {
    commands,
    attachCalls: 0,
    emit(event) { for (const listener of listeners) listener(event); },
    async attach(nextIdentity) { this.attachCalls += 1; commands.push(['attach', nextIdentity.attachment_id]); },
    async write(data) { commands.push(['write', data]); return true; },
    async resize(geom) { commands.push(['resize', geom]); return true; },
    async interrupt() { commands.push(['interrupt']); return true; },
    async detach() { commands.push(['detach']); },
    async close() { commands.push(['close']); },
    async restart() { commands.push(['restart']); },
    dispose() { listeners.clear(); },
    onEvent(listener) { listeners.add(listener); return () => listeners.delete(listener); }
  };
}

try {
  const { createAgentTuiBridge } = await server.ssrLoadModule('/src/lib/shell/agent-tui-bridge.ts');
  const { readPiAttachmentStore } = await server.ssrLoadModule('/src/lib/shell/pi-attachment-store.svelte.ts');

  // --- PTY-015: surface swap twice keeps the SAME Attachment, process
  // generation, transcript stream, and (store-adjacent) draft state ---
  {
    const store = readPiAttachmentStore();
    store.detach();
    const handle = fakeHandle();
    store.bind(identity, geometry);
    const generationAtBind = store.generation;

    // Surface A (Agent TUI) mounts and subscribes.
    const bridgeA = createAgentTuiBridge(handle, store);
    const surfaceA = [];
    const unsubscribeA = bridgeA.subscribeOutput('attachment:pi', (output) => surfaceA.push(output));
    handle.emit({ kind: 'output', data: 'line-a\n', attachment_key: { ...identity }, work_surface_id: 'surface:pi', generation: generationAtBind, sequence: store.latestSequence });
    assert.equal(surfaceA.length, 1);

    // Switch to Mission Canvas (surface A unmounts; the process keeps running).
    unsubscribeA();
    assert.equal(store.generation, generationAtBind, 'view switch does not bump generation');

    // Switch BACK to Agent TUI: same Attachment, same process generation, stream continues.
    const bridgeB = createAgentTuiBridge(handle, store);
    const surfaceB = [];
    const unsubscribeB = bridgeB.subscribeOutput('attachment:pi', (output) => surfaceB.push(output));
    assert.equal(store.identity?.attachment_id, 'attachment:pi');
    assert.equal(store.generation, generationAtBind, 'second mount returns to same process generation');
    handle.emit({ kind: 'output', data: 'line-b\n', attachment_key: { ...identity }, work_surface_id: 'surface:pi', generation: generationAtBind, sequence: store.latestSequence });
    assert.equal(surfaceB.length, 1, 'stream continues on the same generation');
    assert.equal(surfaceB[0].data, 'line-b\n');

    // Third switch: same Attachment, generation, and stream.
    unsubscribeB();
    const bridgeC = createAgentTuiBridge(handle, store);
    const surfaceC = [];
    bridgeC.subscribeOutput('attachment:pi', (output) => surfaceC.push(output));
    assert.equal(store.generation, generationAtBind, 'third mount still same generation');
    assert.equal(handle.attachCalls, 0, 'no duplicate attach command across swaps — same Pi process');
    assert.equal(store.identity?.attachment_id, 'attachment:pi', 'same Attachment across swaps');

    store.detach();
  }

  console.log('surface-swap-runtime: PASS');
} finally {
  await server.close();
}
