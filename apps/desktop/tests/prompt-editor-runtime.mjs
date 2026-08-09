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

  function draftState() {
    return {
      ...structuredClone(authority),
      attachment: structuredClone(authority.attachment),
      content: '# Draft\nPreserved editor content',
      content_sha256: `sha256:${'e'.repeat(64)}`,
      draft_id: 'draft:prompt',
      draft_revision: 4,
      idempotency_key: 'idem:draft:editor:1',
      owner: 'canvas_prompt_editor',
      recipient_ref: 'surface:pi',
      sync_state: 'synchronized',
      updated_at: new Date().toISOString()
    };
  }

  // The editor builds its binding from the projection: draftId from the
  // contribution ref, recipientRef from the focused Work Surface.
  function editorBinding(overrides = {}) {
    return {
      workstream: structuredClone(authority.workstream),
      continuity_id: 'continuity:mission-canvas',
      attachment: structuredClone(authority.attachment),
      workspace_binding_id: 'workspace:mission-canvas',
      runtime_object: { runtime_kind: 'pi_session', runtime_id: 'session:pi' },
      work_surface_id: 'surface:pi',
      draftId: 'draft:prompt',
      recipientRef: 'surface:pi',
      ...overrides
    };
  }

  // --- UI-020 scenario 1: no recipient -> no actionable send ---
  {
    const transport = { async get() { return draftState(); }, async sync() { return draftState(); } };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(editorBinding({ recipientRef: '' }));
    assert.equal(controller.canSend, false, 'no recipient means no actionable Send control');
  }

  // --- UI-020 scenario 2: send enabled only for a resolved authorized recipient ---
  {
    const transport = { async get() { return draftState(); }, async sync() { return draftState(); } };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(editorBinding());
    assert.equal(controller.canSend, true, 'resolved recipient agreed by the draft enables Send');
    const foreign = new MissionCanvasDraftController(transport);
    await foreign.load(editorBinding({ recipientRef: 'surface:foreign' }));
    assert.equal(foreign.canSend, false, 'recipient not agreed by the draft is not authorized');
  }

  // --- UI-020 scenario 3: profile/activity/surface changes preserve the draft ---
  {
    let getCount = 0;
    const transport = {
      async get() { getCount += 1; return draftState(); },
      async sync() { return draftState(); }
    };
    const controller = new MissionCanvasDraftController(transport);
    await controller.load(editorBinding());
    assert.equal(getCount, 1);
    const surfaceChange = editorBinding({ work_surface_id: 'surface:inspector' });
    await controller.rebind(surfaceChange);
    assert.equal(getCount, 1, 'surface change must not refetch (draft preserved)');
    assert.equal(controller.state.draft.content, '# Draft\nPreserved editor content');
    const activityChange = editorBinding({ work_surface_id: 'surface:rail', workspace_binding_id: 'workspace:rail' });
    await controller.rebind(activityChange);
    assert.equal(getCount, 1, 'activity change must not refetch (draft preserved)');
    assert.equal(controller.state.binding.work_surface_id, 'surface:rail');
    // Workstream change does refetch (fresh draft for the other Workstream)
    const foreign = editorBinding({ workstream: {
      scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:other', root_path: '/other', canonical_name: 'Other', fingerprint: 'host-b' } },
      workstream_id: 'ws:foreign'
    } });
    await controller.rebind(foreign);
    assert.equal(getCount, 2, 'workstream change refetches');
  }

  // --- sameWorkstreamAttachment used by the editor's rebind decision ---
  {
    const a = editorBinding();
    assert.equal(sameWorkstreamAttachment(a, editorBinding({ work_surface_id: 'surface:other' })), true);
    assert.equal(sameWorkstreamAttachment(a, editorBinding({ continuity_id: 'continuity:other' })), false);
  }

  console.log('prompt-editor-runtime: PASS');
} finally {
  await server.close();
}
