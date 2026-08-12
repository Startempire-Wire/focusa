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

  // Steering queue: show operations requiring explicit operator confirmation.
  // Follow-up queue: show preview/discovery operations the operator may want to review.
  const isSteering = $derived(contribution.kind === 'steering_queue');
  const items = $derived(projection.operation_bindings.filter((binding) => {
    if (isSteering) return binding.confirmation === 'explicit' && binding.enabled;
    return binding.confirmation === 'preview' || (binding.confirmation === 'confirm' && !binding.enabled);
  }));
  const count = $derived(items.length);
  const visible = $derived(items.slice(0, 3));
  const overflow = $derived(Math.max(0, count - 3));
</script>

<section class="queue" aria-label={contribution.accessibility.label} data-queue-kind={contribution.kind}>
  <div class="queue-copy">
    <strong>{contribution.accessibility.label}</strong>
    <span>{count === 0 ? 'No items pending' : `${count} item${count !== 1 ? 's' : ''}`}</span>
  </div>
  {#if visible.length > 0}
    <div class="queue-actions">
      {#each visible as item (item.operation_id)}
        {#if isSteering && item.enabled}
          <button
            type="button"
            disabled={!onOperation}
            onclick={() => void onOperation?.(item)}
          >{item.display?.label ?? item.operation_id.split('.').at(-1)}</button>
        {:else}
          <span class="preview-chip">{item.display?.label ?? item.operation_id.split('.').at(-1)}</span>
        {/if}
      {/each}
      {#if overflow > 0}
        <span class="overflow-chip">+{overflow} more</span>
      {/if}
    </div>
  {/if}
</section>

<style>
  .queue{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);min-width:0;min-height:58px;padding:var(--space-2) var(--space-3);border:1px solid var(--color-border);border-radius:var(--radius-control);background:var(--color-panel)}
  .queue-copy{display:grid;gap:2px;min-width:0}.queue strong{color:var(--color-text);font:var(--type-label)}
  .queue span,.queue code{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .queue code{color:var(--color-text-secondary)}.queue-actions{display:flex;gap:var(--space-1);flex-shrink:0;align-items:center}
  .preview-chip{padding:2px 6px;border:1px dashed var(--color-border);border-radius:999px;color:var(--color-text-tertiary);font:var(--type-caption);white-space:nowrap}
  .overflow-chip{color:var(--color-text-tertiary);font:var(--type-caption);white-space:nowrap}
  button{border:1px solid var(--color-border);border-radius:999px;padding:3px var(--space-2);background:var(--color-elevated);color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:disabled{opacity:.45;cursor:not-allowed}
</style>
