<script lang="ts">
  import { fetchSemanticPairStatus, invokeSemanticPairAction } from '$lib/api';
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import type { SemanticPairOperation, SemanticPairStatus } from '$lib/types/focus-canvas';

  let status = $state<SemanticPairStatus | null>(null);
  let loadError = $state<string | null>(null);
  let selected = $state<SemanticPairOperation | null>(null);
  let payloadText = $state('{"pair_id":""}');
  let mutationConfirmed = $state(false);
  let invokeResult = $state<string | null>(null);
  let s = $derived(runtimeStore.snapshot as Record<string, any>);
  let scope = $derived.by(() => {
    const identity = (s.projectIdentity ?? {}) as Record<string, unknown>;
    const workpoint = (s.workpointResume ?? s.workpoint ?? {}) as Record<string, unknown>;
    const workpointScope = workpoint.scope && typeof workpoint.scope === 'object'
      ? workpoint.scope as Record<string, unknown> : {};
    return {
      projectRoot: String(identity.project_root ?? identity.projectRoot ?? workpointScope.project_root ?? ''),
      continuityId: String(identity.continuity_id ?? identity.continuityId ?? workpointScope.continuity_id ?? ''),
    };
  });

  $effect(() => {
    const { projectRoot, continuityId } = scope;
    if (!projectRoot || !continuityId) {
      status = null;
      loadError = 'schema_only';
      return;
    }
    let active = true;
    fetchSemanticPairStatus(projectRoot, continuityId)
      .then((value) => { if (active) { status = value; loadError = null; } })
      .catch(() => { if (active) { status = null; loadError = 'degraded'; } });
    return () => { active = false; };
  });

  const label = (operation: SemanticPairOperation) => operation.operation_id.replaceAll('_', ' ');

  async function executeSelected() {
    if (!selected) return;
    if (selected.kind === 'mutation' && !mutationConfirmed) {
      invokeResult = 'Operator confirmation is required for mutation.';
      return;
    }
    try {
      const payload = JSON.parse(payloadText) as Record<string, unknown>;
      const pairId = typeof payload.pair_id === 'string' ? payload.pair_id : undefined;
      invokeResult = 'Executing…';
      const result = await invokeSemanticPairAction<Record<string, unknown>>({
        operation_id: selected.operation_id,
        project_root: scope.projectRoot,
        continuity_id: scope.continuityId,
        pair_id: pairId,
        payload,
        idempotency_key: selected.kind === 'mutation' ? crypto.randomUUID() : undefined,
        confirmation: selected.kind === 'mutation' ? 'operator_confirmed' : undefined,
      });
      invokeResult = JSON.stringify(result);
      status = await fetchSemanticPairStatus(scope.projectRoot, scope.continuityId);
    } catch (error) {
      invokeResult = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<section class="semantic-pair" aria-labelledby="semantic-pair-title">
  <header>
    <h3 id="semantic-pair-title">Semantic Pair</h3>
    <span class:blocked={(status?.state ?? loadError) !== 'supported'}>
      {status?.state ?? loadError ?? 'schema_only'}
    </span>
  </header>
  <p aria-live="polite">
    {status ? `${status.operations.length} semantic operations visible.` : 'Semantic portfolio is not currently readable.'}
  </p>
  {#if status}
    <details>
      <summary>Operations and capabilities</summary>
      <ul>
        {#each status.operations as operation (operation.operation_id)}
          <li>
            <code>{label(operation)}</code>
            <span>{operation.kind}</span>
            <span>{operation.availability}</span>
            <button
              disabled={operation.availability !== 'supported'}
              onclick={() => { selected = operation; mutationConfirmed = false; invokeResult = null; }}
            >Invoke</button>
          </li>
        {/each}
      </ul>
    </details>
    {#if selected}
      <form onsubmit={(event) => { event.preventDefault(); void executeSelected(); }}>
        <strong>{selected.operation_id}</strong>
        <label>
          Typed operation payload
          <textarea bind:value={payloadText} rows="5" spellcheck="false"></textarea>
        </label>
        {#if selected.kind === 'mutation'}
          <label><input type="checkbox" bind:checked={mutationConfirmed} /> Confirm mutation</label>
        {/if}
        <button type="submit">Execute governed operation</button>
        <button type="button" onclick={() => { selected = null; }}>Cancel</button>
      </form>
      {#if invokeResult}<pre aria-live="polite">{invokeResult}</pre>{/if}
    {/if}
  {/if}
</section>

<style>
  .semantic-pair { border: 1px solid var(--border, #475569); border-radius: .5rem; padding: .75rem; }
  header, li { display: flex; align-items: center; justify-content: space-between; gap: .5rem; }
  h3, p { margin: 0 0 .5rem; }
  ul { display: grid; gap: .35rem; padding: 0; list-style: none; }
  code { overflow-wrap: anywhere; }
  .blocked { color: var(--warning, #b45309); }
  button:disabled { cursor: not-allowed; opacity: .75; }
</style>
