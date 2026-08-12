<script lang="ts">
  import type { OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection } from '../types';

  let {
    contribution,
    projection,
    onOperation
  }: {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    onOperation?: (binding: OperationBinding) => void | Promise<void>;
  } = $props();

  // Follow-up queue: show preview/discovery operations or items requiring operator review.
  const isFollowUp = $derived(contribution.kind === 'follow_up_queue');
  const items = $derived(projection.operation_bindings.filter((binding) => {
    if (isFollowUp) return binding.confirmation === 'preview' || (binding.confirmation === 'confirm' && !binding.enabled);
    return false;
  }));
  const count = $derived(items.length);
</script>

<section class="queue" aria-label={contribution.accessibility.label} data-queue-kind={contribution.kind}>
  <div class="queue-copy">
    <strong>{contribution.accessibility.label}</strong>
    <span>{count === 0 ? 'No items pending' : `${count} pending item${count !== 1 ? 's' : ''}`}</span>
  </div>
  {#if items.length > 0}
    <div class="queue-actions">
      {#each items as item (item.operation_id)}
        <button
          type="button"
          disabled={!onOperation || !item.enabled}
          onclick={() => void onOperation?.(item)}
        >{item.display?.label ?? item.operation_id.split('.').at(-1)}</button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .queue{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);min-width:0;min-height:58px;padding:var(--space-2) var(--space-3);border:1px solid var(--color-border);border-radius:var(--radius-control);background:var(--color-panel)}
  .queue-copy{display:grid;gap:2px;min-width:0}.queue strong{color:var(--color-text);font:var(--type-label)}
  .queue span,.queue code{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .queue code{color:var(--color-text-secondary)}.queue-actions{display:flex;gap:var(--space-1);flex-shrink:0}
  button{border:1px solid var(--color-border);border-radius:999px;padding:3px var(--space-2);background:var(--color-elevated);color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:disabled{opacity:.45;cursor:not-allowed}
</style>
