<script lang="ts">
  import type { GeneratedSurfaceSnapshotResolver } from '../generated-surface-types';
  import type { OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection } from '../types';

  interface FocusaGeneratedSurfaceElement extends HTMLElement {
    allowedActions: Iterable<string>;
    snapshot: readonly unknown[];
    delta: readonly unknown[];
  }

  let {
    contribution,
    projection,
    snapshotResolver,
    onOperation
  }: {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    snapshotResolver: GeneratedSurfaceSnapshotResolver;
    onOperation?: (binding: OperationBinding) => void | Promise<void>;
  } = $props();

  let host = $state<HTMLDivElement>();
  let state = $state<'loading' | 'ready' | 'error'>('loading');
  let errorMessage = $state('');
  let generation = 0;

  const actionBindings = $derived(projection.operation_bindings.filter((binding) =>
    binding.target_contribution_id === contribution.contribution_id
    && contribution.operation_ids.includes(binding.operation_id)
    && binding.enabled
    && Boolean(binding.authority_ref)
    && !binding.disabled_reason_ref
  ));

  $effect(() => {
    if (!host) return;
    const requestGeneration = ++generation;
    const currentContribution = contribution;
    const currentProjection = projection;
    const currentBindings = [...actionBindings];
    let disposed = false;
    let element: FocusaGeneratedSurfaceElement | undefined;
    let unsubscribeDelta: (() => void) | undefined;
    state = 'loading';
    errorMessage = '';

    function handleOperation(event: CustomEvent<{ name?: string }>): void {
      const name = event.detail?.name;
      if (!name || !onOperation) return;
      const binding = currentBindings.find((candidate) => candidate.operation_id === name);
      if (!binding || binding.confirmation === 'preview') return;
      void onOperation(binding);
    }

    void (async () => {
      try {
        await import('@focusa/a2ui-renderer/rich-host');
        const resolved = await snapshotResolver(currentContribution, currentProjection);
        if (disposed || requestGeneration !== generation) return;
        const source = Array.isArray(resolved) ? { snapshot: resolved } : resolved;
        element = document.createElement('focusa-generated-surface') as FocusaGeneratedSurfaceElement;
        element.setAttribute('aria-label', currentContribution.accessibility.label);
        element.allowedActions = currentBindings.map((binding) => binding.operation_id);
        element.addEventListener('focusa-operation', handleOperation as EventListener);
        host.replaceChildren(element);
        element.snapshot = source.snapshot;
        if (source.subscribeDelta) {
          const stop = await source.subscribeDelta((messages) => {
            if (!disposed && requestGeneration === generation && element) element.delta = messages;
          });
          if (disposed || requestGeneration !== generation) stop();
          else unsubscribeDelta = stop;
        }
        state = 'ready';
      } catch (error) {
        if (disposed || requestGeneration !== generation) return;
        state = 'error';
        errorMessage = error instanceof Error ? error.message : 'Generated surface unavailable.';
      }
    })();

    return () => {
      disposed = true;
      generation += 1;
      unsubscribeDelta?.();
      element?.removeEventListener('focusa-operation', handleOperation as EventListener);
      element?.remove();
    };
  });
</script>

<section class="generated-surface" aria-label={contribution.accessibility.label} aria-busy={state === 'loading'}>
  <div bind:this={host} class="generated-host"></div>
  {#if state === 'loading'}
    <div class="surface-state" role="status">Loading {contribution.accessibility.label}…</div>
  {:else if state === 'error'}
    <div class="surface-state error" role="alert">
      <strong>Generated surface unavailable</strong>
      <span>{errorMessage}</span>
    </div>
  {/if}
</section>

<style>
  .generated-surface,.generated-host{min-width:0;min-height:0;height:100%}
  .generated-surface{position:relative;overflow:auto;border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-panel)}
  .generated-host{display:block}
  .surface-state{position:absolute;inset:0;display:grid;place-content:center;gap:var(--space-2);padding:var(--layout-card-padding-roomy);background:var(--color-panel);color:var(--color-text-secondary);text-align:center}
  .surface-state.error{color:var(--color-error)}
  .surface-state strong{color:var(--color-text)}
</style>
