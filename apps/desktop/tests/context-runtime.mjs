import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createServer } from 'vite';

const server = await createServer({
  appType: 'custom',
  server: { middlewareMode: true },
  logLevel: 'error'
});

try {
  const { DesktopContext } = await server.ssrLoadModule('/src/lib/mission-canvas/context.ts');
  const projection = JSON.parse(
    await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8')
  );
  const authority = {
    workstream: structuredClone(projection.workstream),
    continuity_id: projection.continuity_id,
    attachment: structuredClone(projection.attachment),
    workspace_binding_id: projection.workspace_binding_id,
    runtime_object: structuredClone(projection.runtime_object),
    work_surface_id: projection.work_surface_id
  };

  const bound = DesktopContext.fromCanonicalPacket(authority);
  assert.equal(bound.state.kind, 'bound');
  assert.equal(bound.kind, 'bound');
  assert.equal(bound.isBound, true);
  assert.equal(bound.value.workstream.workstream_id, 'ws:mission-canvas');
  assert.equal(bound.workstream.scope.scope_key.root_path, '/example/focusa');
  assert.deepEqual(bound.value, authority);

  // The store owns a frozen copy, not a mutable reference to the transport packet.
  assert.equal(Object.isFrozen(bound.state), true);
  assert.equal(Object.isFrozen(bound.value), true);
  assert.equal(Object.isFrozen(bound.value.workstream), true);
  authority.workstream.workstream_id = 'ws:mutated-after-bind';
  assert.equal(bound.value.workstream.workstream_id, 'ws:mission-canvas');
  assert.throws(() => {
    bound.value.workstream.workstream_id = 'ws:local-mutation';
  }, TypeError);

  // A WorkstreamKey is the smallest valid canonical packet. No subordinate
  // identity is manufactured when the packet does not contain one.
  const identityOnly = DesktopContext.fromCanonicalPacket(projection.workstream);
  assert.equal(identityOnly.state.kind, 'bound');
  assert.equal(identityOnly.value.continuity_id, null);
  assert.equal(identityOnly.value.attachment, null);
  assert.equal(identityOnly.value.workstream.workstream_id, 'ws:mission-canvas');

  // A generated operation request is accepted only with canonical authority.
  const operationPacket = {
    schema: 'focusa.workstream_operation_request.v1',
    workstream: structuredClone(projection.workstream),
    actor: { actor_type: 'desktop', actor_id: 'desktop:test' },
    authority: {
      authority_ref: 'authority:test',
      envelope: { status: 'canonical', why: 'verified Desktop Workstream packet' }
    },
    command_id: 'focusa.mission_canvas.projection.get',
    input: {}
  };
  const operationContext = DesktopContext.fromCanonicalPacket(operationPacket);
  assert.equal(operationContext.state.kind, 'bound');
  assert.equal(operationContext.value.workstream.workstream_id, 'ws:mission-canvas');

  // A different explicit Workstream clears an already-bound store. It is not
  // repaired from the previous context or from subordinate identifiers.
  const switched = DesktopContext.fromCanonicalPacket({
    workstream: structuredClone(projection.workstream),
    continuity_id: projection.continuity_id,
    attachment: structuredClone(projection.attachment),
    workspace_binding_id: projection.workspace_binding_id,
    runtime_object: structuredClone(projection.runtime_object),
    work_surface_id: projection.work_surface_id
  });
  const foreign = structuredClone(switched.value);
  foreign.workstream.workstream_id = 'ws:foreign';
  foreign.attachment.workstream.workstream_id = 'ws:foreign';
  switched.fromCanonicalPacket(foreign);
  assert.deepEqual(switched.state, { kind: 'unbound' });
  assert.equal(switched.clearReason, 'context_mismatch');

  const invalidPackets = [
    { continuity_id: projection.continuity_id },
    { project_root: '/example/focusa', workstream_id: 'ws:local' },
    { current_tab: 'mission-canvas', remembered_workspace: 'software' },
    {
      ...operationPacket,
      authority: { authority_ref: 'authority:blocked', envelope: { status: 'blocked', why: 'not verified' } }
    },
    { ...operationPacket, authority: undefined }
  ];
  for (const packet of invalidPackets) {
    const invalid = DesktopContext.fromCanonicalPacket(packet);
    assert.deepEqual(invalid.state, { kind: 'unbound' });
    assert.equal(invalid.kind, 'unbound');
  }

  const events = [];
  const subscribed = new DesktopContext();
  const unsubscribe = subscribed.subscribe((state) => events.push(state.kind));
  subscribed.fromCanonicalPacket(bound.value);
  subscribed.clear();
  unsubscribe();
  subscribed.fromCanonicalPacket(bound.value);
  assert.deepEqual(events, ['unbound', 'bound', 'unbound']);

  const cleared = DesktopContext.clear();
  assert.deepEqual(cleared.state, { kind: 'unbound' });

  console.log('Desktop Workstream context: PASS (canonical identity, mismatch clear, immutability, and fail-closed cases)');
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
}
