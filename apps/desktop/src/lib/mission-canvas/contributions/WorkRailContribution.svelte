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

  function actionable(binding: OperationBinding): boolean {
    return Boolean(
      onOperation
      && binding.enabled
      && binding.authority_ref
      && !binding.disabled_reason_ref
      && binding.confirmation !== 'preview'
    );
  }
</script>

<section
  class="work-rail"
  aria-label={contribution.accessibility.label}
  data-work-rail-ref={contribution.data_ref.ref}
  data-work-rail-revision={contribution.data_ref.revision}
>
  <header>
    <div>
      <strong>Focusa Work Rail</strong>
      {#if contribution.accessibility.description}
        <span>{contribution.accessibility.description}</span>
      {/if}
    </div>
    <span class:stale={contribution.freshness.status === 'stale'} class="freshness">
      {contribution.freshness.status}
    </span>
  </header>

  <div class="rail-row">
    <div class="rail-reference">
      <span>{projection.workstream.scope.scope_key.canonical_name ?? 'Current project'}</span>
      <strong>{projection.focused_semantic_target ? 'Current work focused' : 'Project work available'}</strong>
    </div>
    <span class="revision">Projection r{projection.projection_revision}</span>
  </div>

  {#if bindings.length > 0}
    <div class="rail-actions" aria-label={`${contribution.accessibility.label} actions`}>
      {#each bindings as binding (binding.operation_id)}
        <button
          type="button"
          disabled={!actionable(binding)}
          title={binding.disabled_reason_ref ?? binding.operation_id}
          onclick={() => void onOperation?.(binding)}
        >
          {binding.operation_id.split('.').at(-1)}
        </button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .work-rail{display:grid;gap:var(--space-2);min-width:0;padding:var(--space-3);border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-panel)}
  header,.rail-row,.rail-actions{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);min-width:0}
  header>div,.rail-reference{display:grid;gap:2px;min-width:0}
  strong{color:var(--color-text);font:var(--type-label)}
  header span,.rail-row span,code{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .freshness{flex-shrink:0;padding:2px var(--space-2);border:1px solid var(--color-border);border-radius:999px;color:var(--color-success)}
  .freshness.stale{color:var(--color-warning)}
  .rail-row{padding-block:var(--space-2);border-block:1px solid var(--color-border)}
  .rail-reference code{color:var(--color-text-secondary)}.revision{flex-shrink:0}
  .rail-actions{justify-content:flex-end}
  button{border:1px solid var(--color-border);border-radius:999px;padding:3px var(--space-2);background:var(--color-elevated);color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:disabled{opacity:.45;cursor:not-allowed}
</style>
