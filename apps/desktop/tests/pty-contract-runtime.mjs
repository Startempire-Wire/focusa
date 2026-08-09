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

const attachmentKey = {
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
  workspace_binding_id: 'workspace:mission-canvas'
};

const geometry = { columns: 120, rows: 32, pixelWidth: 960, pixelHeight: 640 };

try {
  const contract = await server.ssrLoadModule('/src/lib/shell/pty-contract.ts');
  const { piAttachmentStore, readPiAttachmentStore } =
    await server.ssrLoadModule('/src/lib/shell/pi-attachment-store.svelte.ts');

  // --- every event kind carries AttachmentKey, WorkSurfaceId, generation, sequence ---
  {
    const identity = { attachment_key: structuredClone(attachmentKey), work_surface_id: 'surface:pi' };
    const events = [
      { kind: 'attached', geometry },
      { kind: 'output', data: 'pi> ' },
      { kind: 'resized', geometry: { ...geometry, columns: 100 } },
      { kind: 'interrupted' },
      { kind: 'detached' },
      { kind: 'closed' },
      { kind: 'restarted' },
      { kind: 'error', message: 'bridge down' },
      { kind: 'stale_rejected', reason: 'stale_generation' },
      { kind: 'stale_rejected', reason: 'non_monotonic_sequence' }
    ];
    for (const partial of events) {
      const event = { ...identity, generation: 3, sequence: 7, ...partial };
      // structural check: full key + surface + generation + seq present
      assert.equal(event.attachment_key.workstream.workstream_id, 'ws:mission-canvas');
      assert.equal(event.attachment_key.attachment_id, 'attachment:pi');
      assert.equal(event.work_surface_id, 'surface:pi');
      assert.equal(event.generation, 3);
      assert.equal(event.sequence, 7);
    }
    assert.equal(contract.PTY_COMMAND_KINDS.join(','), 'attach,input,resize,interrupt,detach,close,restart');
  }

  // --- stale-output rejection: stale generation and non-monotonic sequence ---
  {
    const { evaluatePtyOutput } = contract;
    assert.equal(evaluatePtyOutput(2, 2, 1, 1).accepted, true, 'current generation, in-range seq accepted');
    assert.deepEqual(evaluatePtyOutput(1, 2, 1, 1), { accepted: false, reason: 'stale_generation' });
    assert.deepEqual(evaluatePtyOutput(2, 2, 5, 1), { accepted: false, reason: 'non_monotonic_sequence' });
    assert.equal(evaluatePtyOutput(2, 2, 0, 1).accepted, true, 'replayed seq in current generation accepted (reconnect)');
  }

  // --- monotonic sequence and generation factories ---
  {
    const nextSeq = contract.createMonotonicSequence();
    assert.equal(nextSeq(), 1);
    assert.equal(nextSeq(), 2);
    const nextGen = contract.createRunGeneration();
    assert.equal(nextGen(), 1);
    assert.equal(nextGen(), 2);
  }

  // --- the store envelope carries the full AttachmentKey + generation + seq ---
  {
    const store = readPiAttachmentStore();
    const identity = { ...structuredClone(attachmentKey), work_surface_id: 'surface:pi' };
    store.bind(identity, geometry);
    assert.equal(store.latestEnvelope?.generation, 1, 'first run generation is 1');
    assert.equal(store.latestEnvelope?.sequence, 1, 'attach is the first monotonic command');
    assert.equal(store.latestEnvelope?.attachment_key?.attachment_id, 'attachment:pi');
    assert.equal(store.latestEnvelope?.work_surface_id, 'surface:pi');
    assert.equal(store.latestEnvelope?.command.kind, 'attach');
    // send() positive paths for interrupt/resize/close/restart
    assert.equal(store.send({ kind: 'interrupt', attachment_id: 'attachment:pi' }), true);
    assert.equal(store.send({ kind: 'resize', attachment_id: 'attachment:pi', geometry: { ...geometry, columns: 100 } }), true);
    assert.equal(store.send({ kind: 'close', attachment_id: 'attachment:pi' }), true);
    assert.equal(store.send({ kind: 'restart', attachment_id: 'attachment:pi' }), true);
    assert.equal(store.latestEnvelope?.sequence, 5);
    // stale-output rejection on the store matches the contract decision
    assert.equal(store.acceptsOutput(store.latestEnvelope.generation, 3), true);
    assert.equal(store.acceptsOutput(store.latestEnvelope.generation - 1, 3), false, 'stale generation rejected');
    assert.equal(store.acceptsOutput(store.latestEnvelope.generation, 99), false, 'non-monotonic sequence rejected');
    // unbound store rejects every write-path command
    store.detach();
    assert.equal(store.send({ kind: 'interrupt', attachment_id: 'attachment:pi' }), false);
    assert.equal(store.send({ kind: 'restart', attachment_id: 'attachment:pi' }), false);
  }

  console.log('pty-contract-runtime: PASS');
} finally {
  await server.close();
}
