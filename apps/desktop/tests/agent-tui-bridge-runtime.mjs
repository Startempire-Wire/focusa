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
    label: 'fake',
    adapterKind: 'virtual',
    commands,
    emit(event) {
      for (const listener of listeners) listener(event);
    },
    async attach(nextIdentity, geom) { commands.push(['attach', nextIdentity.attachment_id, geom]); },
    async write(data) { commands.push(['write', data]); return true; },
    async resize(geom) { commands.push(['resize', geom]); return true; },
    async interrupt() { commands.push(['interrupt']); return true; },
    async detach() { commands.push(['detach']); },
    async close() { commands.push(['close']); },
    async restart() { commands.push(['restart']); },
    dispose() { listeners.clear(); },
    onEvent(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    }
  };
}

try {
  const { createAgentTuiBridge } = await server.ssrLoadModule('/src/lib/shell/agent-tui-bridge.ts');
  const { readPiAttachmentStore } = await server.ssrLoadModule('/src/lib/shell/pi-attachment-store.svelte.ts');

  // --- controls: send fails closed when no exact Attachment is bound ---
  {
    const store = readPiAttachmentStore();
    store.detach();
    const handle = fakeHandle();
    const bridge = createAgentTuiBridge(handle, store);
    await bridge.send({ kind: 'input', attachment_id: 'attachment:pi', data: 'echo x\n' });
    assert.deepEqual(handle.commands, [], 'unbound store: no command reaches the handle');
  }

  // --- controls: attach/input/resize/interrupt/restart route through the handle ---
  {
    const store = readPiAttachmentStore();
    store.detach();
    const handle = fakeHandle();
    const bridge = createAgentTuiBridge(handle, store);
    store.bind(identity, geometry);
    await bridge.send({ kind: 'attach', identity, geometry });
    await bridge.send({ kind: 'input', attachment_id: 'attachment:pi', data: 'echo hi\n' });
    await bridge.send({ kind: 'resize', attachment_id: 'attachment:pi', geometry: { ...geometry, columns: 100 } });
    await bridge.send({ kind: 'interrupt', attachment_id: 'attachment:pi' });
    await bridge.send({ kind: 'restart', attachment_id: 'attachment:pi' });
    const kinds = handle.commands.map(([name]) => name);
    assert.deepEqual(kinds, ['attach', 'write', 'resize', 'interrupt', 'restart']);
    const boundGeneration = store.generation;
    assert.ok(boundGeneration >= 1, 'current run generation active');
    assert.equal(store.acceptsOutput(boundGeneration, store.latestSequence), true);
  }

  // --- output stream: prior Attachment/generation/sequence never updates the terminal ---
  {
    const store = readPiAttachmentStore();
    store.detach();
    const handle = fakeHandle();
    const bridge = createAgentTuiBridge(handle, store);
    store.bind(identity, geometry);
    const received = [];
    const unsubscribe = bridge.subscribeOutput('attachment:pi', (output) => received.push(output));
    // current generation, in-range sequence -> accepted
    handle.emit({
      kind: 'output', data: 'pi> ', attachment_key: { ...identity }, work_surface_id: 'surface:pi',
      generation: store.generation, sequence: store.latestSequence
    });
    assert.equal(received.length, 1, 'current generation output accepted');
    assert.equal(received[0].attachment_id, 'attachment:pi');
    // foreign attachment -> dropped
    handle.emit({
      kind: 'output', data: 'evil', attachment_key: { ...identity, attachment_id: 'attachment:other' }, work_surface_id: 'surface:pi',
      generation: store.generation, sequence: store.latestSequence
    });
    // stale generation -> dropped
    handle.emit({
      kind: 'output', data: 'old', attachment_key: { ...identity }, work_surface_id: 'surface:pi',
      generation: store.generation - 1, sequence: store.latestSequence
    });
    // non-monotonic sequence -> dropped
    handle.emit({
      kind: 'output', data: 'future', attachment_key: { ...identity }, work_surface_id: 'surface:pi',
      generation: store.generation, sequence: store.latestSequence + 99
    });
    assert.equal(received.length, 1, 'stale events never reach the terminal surface');
    unsubscribe();
    handle.emit({
      kind: 'output', data: 'after-unsub', attachment_key: { ...identity }, work_surface_id: 'surface:pi',
      generation: store.generation, sequence: store.latestSequence
    });
    assert.equal(received.length, 1, 'unsubscribe stops the stream');
  }

  // --- control gate activates only when attached at the exact current generation ---
  {
    const store = readPiAttachmentStore();
    store.detach();
    assert.equal(store.attached, false, 'unbound: no controls');
    store.bind(identity, geometry);
    const currentGeneration = store.generation;
    assert.equal(
      store.attached && store.acceptsOutput(currentGeneration, store.latestSequence),
      true,
      'attached current generation: controls active'
    );
    assert.equal(
      store.acceptsOutput(currentGeneration + 1, store.latestSequence),
      false,
      'future generation: controls inactive'
    );
    assert.equal(
      store.acceptsOutput(currentGeneration - 1, store.latestSequence),
      false,
      'prior generation: controls inactive'
    );
  }

  console.log('agent-tui-bridge-runtime: PASS');
} finally {
  await server.close();
}
