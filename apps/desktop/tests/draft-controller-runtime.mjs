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
  const { MissionCanvasDraftController, sameWorkstreamAttachment } =
    await server.ssrLoadModule('/src/lib/mission-canvas/draft-controller.svelte.ts');

  const authority = {
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
    attachment: {
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
    },
    workspace_binding_id: 'workspace:mission-canvas',
    runtime_object: { runtime_kind: 'pi_session', runtime_id: 'session:pi' },
    work_surface_id: 'surface:pi'
  };

  function draftState(overrides = {}) {
    return {
      ...structuredClone(authority),
      attachment: structuredClone(authority.attachment),
      content: '# Draft\nPreserved content',
      content_sha256: `sha256:${'d'.repeat(64)}`,
      draft_id: 'draft:prompt',
      draft_revision: 4,
      idempotency_key: 'idem:draft:1',
      owner: 'canvas_prompt_editor',
      recipient_ref: 'recipient:pi-session',
      selection_start: 2,
      selection_end: 5,
      sync_state: 'synchronized',
      updated_at: new Date().toISOString(),
      ...overrides
    };
  }

  function binding(overrides = {}) {
    return {
      ...structuredClone(authority),
      attachment: structuredClone(authority.attachment),
      draftId: 'draft:prompt',
      recipientRef: 'recipient:pi-session',
      ...overrides
    };
  }

  // --- Scenario A: presentation-only change preserves the draft ---
  {
    const gets = [];
    const transport = {
      async get(binding) { gets.push(binding); return draftState(); },
      async sync() { throw new Error('sync not expected'); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(binding({ work_surface_id: 'surface:pi' }));
    assert.equal(controller.state.kind, 'ready');
    const surfaceChanged = binding({ work_surface_id: 'surface:inspector' });
    await controller.rebind(surfaceChanged);
    assert.equal(controller.state.kind, 'ready');
    assert.equal(controller.state.binding.work_surface_id, 'surface:inspector');
    assert.equal(controller.state.draft.content, '# Draft\nPreserved content');
    assert.equal(gets.length, 1, 'surface change must not refetch the draft');
  }

  // --- Scenario B: profile/activity change preserves the draft ---
  {
    const gets = [];
    const transport = {
      async get(binding) { gets.push(binding); return draftState(); },
      async sync() { throw new Error('sync not expected'); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(binding());
    const activityChanged = binding({ work_surface_id: 'surface:rail', workspace_binding_id: 'workspace:rail' });
    await controller.rebind(activityChanged);
    assert.equal(controller.state.kind, 'ready');
    assert.equal(controller.state.draft.draft_id, 'draft:prompt');
    assert.equal(gets.length, 1);
  }

  // --- Scenario C: Workstream-level change reloads (does not preserve) ---
  {
    let getCount = 0;
    const transport = {
      async get() { getCount += 1; return draftState({ draft_revision: 5 }); },
      async sync() { throw new Error('sync not expected'); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(binding());
    assert.equal(getCount, 1);
    const foreign = binding({ workstream: {
      scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:other', root_path: '/other', canonical_name: 'Other', fingerprint: 'host-b' } },
      workstream_id: 'ws:foreign'
    } });
    await controller.rebind(foreign);
    assert.equal(getCount, 2, 'workstream change must refetch');
  }

  // --- Scenario D: foreign draft binding rejected on load ---
  {
    const transport = {
      async get() { return draftState({ recipient_ref: 'recipient:other' }); },
      async sync() { throw new Error('sync not expected'); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(binding());
    assert.equal(controller.state.kind, 'error');
    assert.equal(controller.state.reason, 'foreign_draft_binding');
  }

  // --- Scenario E: foreign/regressed sync revision rejected ---
  {
    const transport = {
      async get() { return draftState(); },
      async sync() { return draftState({ draft_revision: 3 }); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(binding());
    await controller.sync('newer content');
    assert.equal(controller.state.kind, 'conflict');
    assert.equal(controller.state.reason, 'draft_revision_regressed');
  }

  // --- Scenario F: transport conflict surfaces as conflict state ---
  {
    const transport = {
      async get() { return draftState(); },
      async sync() { return draftState({ sync_state: 'conflict', conflict_ref: 'conflict:concurrent' }); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(binding());
    await controller.sync('contending content');
    assert.equal(controller.state.kind, 'conflict');
    assert.equal(controller.state.reason, 'conflict:concurrent');
  }

  // --- Scenario G: no send control without a resolved authorized recipient ---
  {
    const transport = { async get() { return draftState(); }, async sync() { return draftState(); } };
    const unbound = new MissionCanvasDraftController(transport);
    assert.equal(unbound.canSend, false, 'unbound has no send control');

    const emptyRecipient = new MissionCanvasDraftController(transport);
    await emptyRecipient.load(binding({ recipientRef: '' }));
    assert.equal(emptyRecipient.canSend, false, 'empty recipient has no send control');

    const mismatched = new MissionCanvasDraftController(transport);
    await mismatched.load(binding({ recipientRef: 'recipient:other' }));
    assert.equal(mismatched.canSend, false, 'recipient not agreed by the draft has no send control');

    const authorized = new MissionCanvasDraftController(transport);
    await authorized.load(binding());
    assert.equal(authorized.canSend, true, 'resolved authorized recipient enables send');
  }

  // --- Scenario H: sameWorkstreamAttachment semantics ---
  {
    const a = binding();
    const surfaceChanged = binding({ work_surface_id: 'surface:rail' });
    assert.equal(sameWorkstreamAttachment(a, surfaceChanged), true);
    const foreignWorkstream = binding({ workstream: {
      scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:other', root_path: '/other', canonical_name: 'Other', fingerprint: 'host-b' } },
      workstream_id: 'ws:foreign'
    } });
    assert.equal(sameWorkstreamAttachment(a, foreignWorkstream), false);
    const foreignContinuity = binding({ continuity_id: 'continuity:other' });
    assert.equal(sameWorkstreamAttachment(a, foreignContinuity), false);
  }

  console.log('draft-controller-runtime: PASS');
} finally {
  await server.close();
}
