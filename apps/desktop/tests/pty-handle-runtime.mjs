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

try {
  const { createPtyHandle, detectPtyAdapter, isPtyCommandKind, isPtyEventKind } =
    await server.ssrLoadModule('/src/lib/shell/pty-handle.ts');

  // --- factory picks the honest virtual adapter outside Tauri ---
  {
    assert.equal(detectPtyAdapter(), 'virtual', 'no Tauri internals in preview');
    const handle = await createPtyHandle({ adapter: 'virtual' });
    assert.equal(handle.adapterKind, 'virtual');
    assert.equal(handle.label, 'virtual-pty (preview)');
  }

  // --- full command surface: attach/write/resize/interrupt/detach/close/restart ---
  {
    for (const kind of ['attach', 'input', 'resize', 'interrupt', 'detach', 'close', 'restart']) {
      assert.equal(isPtyCommandKind(kind), true, kind);
    }
    assert.equal(isPtyCommandKind('pipe'), false, 'ordinary pipes are not part of the command surface');
    for (const kind of ['attached', 'output', 'resized', 'interrupted', 'detached', 'closed', 'restarted', 'error', 'stale_rejected']) {
      assert.equal(isPtyEventKind(kind), true, kind);
    }
  }

  // --- virtual handle lifecycle: every event carries full identity + generation + seq ---
  {
    const handle = await createPtyHandle({ adapter: 'virtual' });
    const events = [];
    handle.onEvent((event) => events.push(event));
    await handle.attach(identity, geometry);
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, 'attached');
    assert.equal(events[0].attachment_key.workstream.workstream_id, 'ws:mission-canvas');
    assert.equal(events[0].attachment_key.attachment_id, 'attachment:pi');
    assert.equal(events[0].work_surface_id, 'surface:pi');
    assert.equal(events[0].generation, 1, 'attach starts generation 1');
    assert.equal(events[0].sequence, 1, 'attach is first monotonic event');

    assert.equal(await handle.write('echo hi\n'), false, 'virtual handle never invents input acceptance');
    assert.equal(await handle.resize({ ...geometry, columns: 100 }), true);
    assert.equal(await handle.interrupt(), true);
    assert.equal(events[1].kind, 'resized');
    assert.equal(events[2].kind, 'interrupted');
    assert.equal(events[2].generation, 1);
    assert.equal(events[2].sequence, 3, 'sequence stays monotonic');

    await handle.restart();
    assert.equal(events[3].kind, 'restarted');
    assert.equal(events[3].sequence, 4);

    await handle.detach();
    assert.equal(events[4].kind, 'detached');
    assert.equal(await handle.interrupt(), false, 'detached handle rejects commands');
    assert.equal(await handle.resize(geometry), false);

    await handle.attach(identity, geometry);
    assert.equal(events[5].generation, 2, 're-attach bumps run generation');
    assert.equal(events[5].sequence, 1, 'new generation restarts the sequence');

    await handle.close();
    assert.equal(events[6].kind, 'closed');
    handle.dispose();
    assert.equal(events.length, 7, 'dispose stops emitting');
  }

  // --- native adapter fails closed when the Cargo runtime command is missing ---
  {
    const { createTauriPtyHandle } = await server.ssrLoadModule('/src/lib/shell/tauri-pty-adapter.ts');
    const events = [];
    const invoke = async () => { throw new Error('command focusa_pty_attach not found'); };
    const handle = createTauriPtyHandle({ invoke });
    handle.onEvent((event) => events.push(event));
    await handle.attach(identity, geometry);
    assert.equal(events[0].kind, 'error', 'missing native command surfaces a typed error');
    assert.equal(events[0].message, 'command focusa_pty_attach not found');
    assert.equal(events[0].attachment_key.attachment_id, 'attachment:pi');
    assert.equal(events[0].generation, 1);
    assert.equal(events[0].sequence, 1);
    assert.equal(await handle.write('x'), false, 'no attached process, no input path');
  }

  // --- native adapter accepts a command result and emits attached ---
  {
    const { createTauriPtyHandle } = await server.ssrLoadModule('/src/lib/shell/tauri-pty-adapter.ts');
    const events = [];
    const handle = createTauriPtyHandle({ invoke: async (command) => (command.endsWith('_attach') ? { ok: true } : { ok: true }) });
    handle.onEvent((event) => events.push(event));
    await handle.attach(identity, geometry);
    assert.equal(events[0].kind, 'attached');
    assert.equal(await handle.resize(geometry), true);
    assert.equal(events[1].kind, 'resized');
  }

  console.log('pty-handle-runtime: PASS');
} finally {
  await server.close();
}
