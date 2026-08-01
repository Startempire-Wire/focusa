<script lang="ts">
  import { fetchSemanticPairStatus } from '$lib/api';
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import type { SemanticPairOperation, SemanticPairStatus } from '$lib/types/focus-canvas';

  let status = $state<SemanticPairStatus | null>(null);
  let loadError = $state<string | null>(null);
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
</script>

<section class="semantic-pair" aria-labelledby="semantic-pair-title">
  <header>
    <h3 id="semantic-pair-title">Semantic Pair</h3>
    <span class:blocked={(status?.state ?? loadError) !== 'schema_only'}>
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
            {#if operation.kind === 'mutation'}
              <button disabled title="This menubar view is read-only">Unsupported on this surface</button>
            {:else}
              <span>{operation.availability}</span>
            {/if}
          </li>
        {/each}
      </ul>
    </details>
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
