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

try {
  const { readPiAttachmentStore, PiAttachmentStore } =
    await server.ssrLoadModule('/src/lib/shell/pi-attachment-store.svelte.ts');

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
    work_surface_id: 'surface:pi',
    runtime_object: { runtime_kind: 'pi_session', runtime_id: 'session:pi' }
  };
  const geometry = { columns: 120, rows: 40, pixelWidth: 960, pixelHeight: 480 };

  // --- singleton identity across surfaces ---
  {
    const first = readPiAttachmentStore();
    const second = readPiAttachmentStore();
    assert.equal(first, second, 'both surfaces must observe the same singleton store');
    assert.equal(first.state.state, 'unbound');
  }

  // --- bind transitions to attached with exact identity ---
  {
    const store = new PiAttachmentStore();
    store.binding(geometry);
    assert.equal(store.state.state, 'binding');
    store.bind(identity, geometry);
    assert.equal(store.state.state, 'attached');
    assert.equal(store.attached, true);
    assert.equal(store.identity?.attachment_id, 'attachment:pi');
    assert.equal(store.state.canWrite, true);
    assert.equal(store.state.canInterrupt, true);
    assert.equal(store.latestEnvelope?.command.kind, 'attach');
    assert.equal(store.latestEnvelope?.generation, 1);
    assert.equal(store.latestEnvelope?.sequence, 1);
    assert.equal(store.latestEnvelope?.attachment_id, 'attachment:pi');
    assert.equal(store.latestEnvelope?.work_surface_id, 'surface:pi');
  }

  // --- monotonic sequence + generation guard on commands ---
  {
    const store = new PiAttachmentStore();
    store.bind(identity, geometry);
    assert.equal(store.send({ kind: 'resize', attachment_id: 'attachment:pi', geometry: { ...geometry, columns: 100 } }), true);
    assert.equal(store.send({ kind: 'interrupt', attachment_id: 'attachment:pi' }), true);
    assert.equal(store.latestEnvelope?.sequence, 3, 'attach + resize + interrupt');
    assert.equal(store.latestEnvelope?.generation, 1);
    // commands after detach are refused
    store.detach();
    assert.equal(store.state.state, 'unbound');
    assert.equal(store.send({ kind: 'interrupt', attachment_id: 'attachment:pi' }), false);
    assert.equal(store.send({ kind: 'restart', attachment_id: 'attachment:pi' }), false);
  }

  // --- stale-output rejection across generations ---
  {
    const store = new PiAttachmentStore();
    store.bind(identity, geometry);
    const generation = store.latestEnvelope?.generation ?? 1;
    assert.equal(store.acceptsOutput(generation, 1), true);
    assert.equal(store.acceptsOutput(generation - 1, 1), false, 'previous generation output rejected');
    assert.equal(store.acceptsOutput(generation + 1, 1), false, 'future generation output rejected');
    assert.equal(store.acceptsOutput(generation, 99), false, 'out-of-sequence output rejected');
    store.detach();
    store.bind({ ...identity, session_id: 'session:pi-2' }, geometry);
    assert.equal(store.latestEnvelope?.generation, 3, 'rebind advances the generation');
  }

  // --- disconnect / error transitions ---
  {
    const store = new PiAttachmentStore();
    store.bind(identity, geometry);
    store.disconnect('runtime exited');
    assert.equal(store.state.state, 'disconnected');
    assert.equal(store.state.canWrite, false);
    const second = new PiAttachmentStore();
    second.error('pty spawn failed');
    assert.equal(second.state.state, 'error');
    assert.equal(second.state.canInterrupt, false);
  }

  console.log('pi-attachment-store-runtime: PASS');
} finally {
  await server.close();
}
