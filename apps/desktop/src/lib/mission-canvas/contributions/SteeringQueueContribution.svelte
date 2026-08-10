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

  const bindings = $derived(projection.operation_bindings.filter((binding) =>
    binding.target_contribution_id === contribution.contribution_id
    && contribution.operation_ids.includes(binding.operation_id)
  ));
</script>

<section class="queue" aria-label={contribution.accessibility.label} data-queue-kind={contribution.kind}>
  <div class="queue-copy">
    <strong>{contribution.kind === 'steering_queue' ? 'Steering Queue' : 'Follow-up Queue'}</strong>
    <span>{contribution.kind === 'steering_queue' ? 'Delivered at next safe active-turn boundary' : 'Delivered after current agent run completes'}</span>
  </div>
  {#if bindings.length > 0}
    <div class="queue-actions">
      {#each bindings as binding (binding.operation_id)}
        <button
          type="button"
          disabled={!onOperation || !binding.enabled || !binding.authority_ref || Boolean(binding.disabled_reason_ref) || binding.confirmation === 'preview'}
          onclick={() => void onOperation?.(binding)}
        >{binding.operation_id.split('.').at(-1)}</button>
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
