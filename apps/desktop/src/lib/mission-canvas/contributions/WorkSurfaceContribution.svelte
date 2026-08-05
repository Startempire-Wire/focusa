<script lang="ts">
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import type { OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection } from '../types';

  const LIFECYCLE_LABELS: Readonly<Record<string, string>> = {
    'focusa.mission_canvas.rich_host.launch': 'Launch',
    'focusa.mission_canvas.rich_host.focus': 'Focus',
    'focusa.mission_canvas.rich_host.hide': 'Hide',
    'focusa.mission_canvas.rich_host.close': 'Close'
  };

  let {
    contribution,
    projection,
    onOperation
  }: {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    onOperation?: (binding: OperationBinding) => void | Promise<void>;
  } = $props();

  const tone = $derived(contribution.freshness.status === 'current' ? 'ready' : contribution.freshness.status === 'stale' ? 'watch' : 'neutral');
  const lifecycleBindings = $derived(projection.operation_bindings.filter((binding) =>
    binding.target_contribution_id === contribution.contribution_id
    && contribution.operation_ids.includes(binding.operation_id)
    && binding.operation_id in LIFECYCLE_LABELS
  ));

  function actionable(binding: OperationBinding): boolean {
    return Boolean(onOperation && binding.enabled && binding.authority_ref && !binding.disabled_reason_ref && binding.confirmation !== 'preview');
  }
</script>

<section class="work-surface" aria-label={contribution.accessibility.label} data-work-surface-ref={contribution.data_ref.ref}>
  <header>
    <div class="identity">
      <strong>{contribution.accessibility.label}</strong>
      {#if contribution.accessibility.description}<span>{contribution.accessibility.description}</span>{/if}
    </div>
    <div class="actions">
      <StatusBadge {tone} label={contribution.freshness.status}/>
      {#each lifecycleBindings as binding (binding.operation_id)}
        <button
          type="button"
          disabled={!actionable(binding)}
          aria-label={`${LIFECYCLE_LABELS[binding.operation_id]} ${contribution.accessibility.label}`}
          onclick={() => void onOperation?.(binding)}
        >{LIFECYCLE_LABELS[binding.operation_id]}</button>
      {/each}
    </div>
  </header>
  <div class="artifact-reference">
    <span>{contribution.data_ref.kind}</span>
    <code>{contribution.data_ref.ref}</code>
    <small>revision {contribution.data_ref.revision}</small>
  </div>
</section>

<style>
  .work-surface{display:grid;grid-template-rows:auto minmax(0,1fr);height:100%;min-height:0;border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-panel);overflow:hidden}
  header,.actions{display:flex;align-items:center;gap:var(--space-2)}
  header{justify-content:space-between;padding:var(--space-3) var(--space-4);border-bottom:1px solid var(--color-border)}
  .identity{display:grid;gap:var(--space-1);min-width:0}
  .identity strong{color:var(--color-text);font:var(--type-label)}
  .identity span{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .actions{flex-shrink:0}
  button{border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-1) var(--space-2);background:transparent;color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:disabled{cursor:not-allowed;opacity:.45}
  .artifact-reference{align-self:center;justify-self:center;display:grid;justify-items:center;gap:var(--space-2);max-width:100%;padding:var(--layout-card-padding-roomy);color:var(--color-text-tertiary)}
  .artifact-reference span{font:var(--type-caption);letter-spacing:.08em;text-transform:uppercase}
  code{max-width:100%;overflow:hidden;color:var(--color-text-secondary);font:var(--type-code);text-overflow:ellipsis;white-space:nowrap}
  small{font:var(--type-caption)}
</style>
