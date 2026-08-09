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

const contribution = {
  contribution_id: 'contribution:genesis-context',
  kind: 'generated_surface',
  data_ref: { ref: 'surface:genesis-context' },
  operation_ids: ['operation:genesis.context.advance'],
  accessibility: { label: 'Genesis Context', focus_semantic_id: 'genesis-context', landmark_role: 'region' }
};

function projection(bindings) {
  return {
    schema: 'focusa.resolved_workspace_projection.v1',
    workstream: { scope: { scope_kind: 'project', scope_key: { scope_kind: 'project', scope_id: 'project:focusa', root_path: '/example/focusa', canonical_name: 'Focusa', fingerprint: 'host-a' } }, workstream_id: 'ws:mission-canvas' },
    projection_revision: 11,
    layout_revision: 4,
    eligible_contributions: [contribution],
    operation_bindings: bindings
  };
}

try {
  const ops = await server.ssrLoadModule('/src/lib/mission-canvas/generated-surface-operations.ts');
  const stages = await server.ssrLoadModule('/src/lib/mission-canvas/generated-stage-surfaces.ts');

  // --- GEN-002: component cannot call an unregistered operation ---
  {
    const verdict = ops.resolveOperationPermission(projection([]), contribution, 'operation:unregistered');
    assert.deepEqual(verdict, { permitted: false, reason: 'unregistered_operation' });
    const result = await ops.invokeRegisteredOperation(projection([]), contribution, 'operation:unregistered', async () => {
      throw new Error('executor must never run for unregistered operations');
    });
    assert.equal(result.ok, false);
    assert.equal(result.error_ref, 'permission_denied:unregistered_operation');
  }

  // --- GEN-002: cannot bypass the permission projection ---
  {
    const disabled = projection([{ operation_id: 'operation:genesis.context.advance', target_contribution_id: 'contribution:genesis-context', enabled: false, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: 'permission_denied' }]);
    assert.deepEqual(ops.resolveOperationPermission(disabled, contribution, 'operation:genesis.context.advance'), { permitted: false, reason: 'disabled_by_projection' });
    const noAuthority = projection([{ operation_id: 'operation:genesis.context.advance', target_contribution_id: 'contribution:genesis-context', enabled: true, authority_ref: null, confirmation: 'none', disabled_reason_ref: null }]);
    assert.deepEqual(ops.resolveOperationPermission(noAuthority, contribution, 'operation:genesis.context.advance'), { permitted: false, reason: 'missing_authority_ref' });
    const wrongContribution = projection([{ operation_id: 'operation:genesis.context.advance', target_contribution_id: 'contribution:other', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null }]);
    assert.deepEqual(ops.resolveOperationPermission(wrongContribution, contribution, 'operation:genesis.context.advance'), { permitted: false, reason: 'unregistered_operation' });
  }

  // --- GEN-002: governed Result/Evidence/Receipt + generated delta ---
  {
    const permitted = projection([{ operation_id: 'operation:genesis.context.advance', target_contribution_id: 'contribution:genesis-context', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null }]);
    let ran = false;
    const result = await ops.invokeRegisteredOperation(permitted, contribution, 'operation:genesis.context.advance', async (binding) => {
      ran = true;
      assert.equal(binding.authority_ref, 'authority:exact');
      return { schema: 'focusa.generated_surface.governed_result.v1', ok: true, evidence_refs: ['evidence:genesis-context-1'] };
    });
    assert.equal(ran, true);
    assert.equal(result.ok, true);
    const deltas = [];
    const receipt = ops.emitGeneratedDelta(
      { schema: 'focusa.generated_surface.generated_delta.v1', contribution_id: 'contribution:genesis-context', operation_id: 'operation:genesis.context.advance', revision: 1, happened_at: new Date().toISOString(), summary: 'context advanced' },
      (delta) => deltas.push(delta)
    );
    assert.equal(deltas.length, 1, 'durable event emitted (not relabeled transcript)');
    assert.equal(receipt.accepted, true);
    assert.equal(receipt.schema, 'focusa.generated_surface.governed_receipt.v1');
  }

  // --- GEN-003: five resumable generated stage surfaces ---
  {
    assert.deepEqual(stages.CRIST_STAGE_ORDER, ['context', 'role', 'interview', 'spec', 'tasks']);
    assert.equal(stages.CRIST_STAGE_SURFACES.length, 5);
    for (const stage of stages.CRIST_STAGE_SURFACES) {
      assert.equal(stage.resumable, true, `${stage.stage_id}: resumable`);
      assert.equal(stage.terminal_fallback_truth, 'canonical_projection', `${stage.stage_id}: terminal fallback truth is the canonical projection`);
      assert.ok(stage.primary_action_operation.startsWith('operation:genesis.'), `${stage.stage_id}: primary action is canonical`);
      assert.ok(stage.autosave_draft_id.startsWith('draft:genesis:'), `${stage.stage_id}: autosave/resume draft`);
      assert.deepEqual(stage.recovery_states, ['error', 'retry', 'resume'], `${stage.stage_id}: recovery states`);
      assert.equal(stage.next_stages.length <= 1, true, `${stage.stage_id}: canonical ordering`);
    }
    // chain: context→role→interview→spec→tasks
    assert.deepEqual(stages.CRIST_STAGE_SURFACES[0].next_stages, ['role']);
    assert.deepEqual(stages.CRIST_STAGE_SURFACES[3].next_stages, ['tasks']);
    assert.deepEqual(stages.CRIST_STAGE_SURFACES[4].next_stages, []);
    // primary action resolves ONLY through the canonical permission projection
    const bindings = [{ operation_id: 'operation:genesis.context.advance', target_contribution_id: 'contribution:genesis-context', enabled: true, authority_ref: 'authority:exact', confirmation: 'none', disabled_reason_ref: null }];
    const action = stages.stagePrimaryAction(stages.CRIST_STAGE_SURFACES[0], bindings, 'contribution:genesis-context');
    assert.equal(action?.operation_id, 'operation:genesis.context.advance');
    assert.equal(stages.stagePrimaryAction(stages.CRIST_STAGE_SURFACES[0], [], 'contribution:genesis-context'), undefined, 'no unregistered primary action');
    assert.throws(() => stages.cristStageSurface('mock'), 'unknown stages are never invented');
  }

  console.log('generated-operations-runtime: PASS');
} finally {
  await server.close();
}
