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

  // Show all operations targeting this contribution, not just those
  // in operation_ids (which the old daemon returns empty).
  const bindings = $derived(projection.operation_bindings.filter((binding) =>
    binding.target_contribution_id === contribution.contribution_id
  ));
  const liveBindings = $derived(bindings.filter(b => b.enabled && b.confirmation !== 'preview'));
  const previewBindings = $derived(bindings.filter(b => !b.enabled || b.confirmation === 'preview'));

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

  {#if liveBindings.length > 0}
    <div class="rail-actions" aria-label={`${contribution.accessibility.label} live actions`}>
      {#each liveBindings as binding (binding.operation_id)}
        <button
          type="button"
          disabled={!actionable(binding)}
          title={binding.display?.label ?? binding.operation_id}
          onclick={() => void onOperation?.(binding)}
        >
          {binding.display?.label ?? binding.operation_id.split('.').at(-1)}
        </button>
      {/each}
    </div>
  {/if}
  {#if previewBindings.length > 0}
    <div class="rail-preview" aria-label={`${contribution.accessibility.label} preview items`}>
      <span class="preview-label">Upcoming</span>
      {#each previewBindings as binding (binding.operation_id)}
        <span class="preview-chip" title={binding.display?.label ?? binding.operation_id}>
          {binding.display?.label ?? binding.operation_id.split('.').at(-1)}
        </span>
      {/each}
    </div>
  {/if}
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
  .rail-preview{display:flex;align-items:center;gap:4px;padding:4px 0;flex-wrap:wrap}
  .preview-label{color:var(--color-text-tertiary);font:10px/1.5 ui-monospace,SFMono-Regular,monospace;text-transform:uppercase;letter-spacing:.05em}
  .preview-chip{padding:1px 5px;border:1px dashed var(--color-border);border-radius:3px;color:var(--color-text-tertiary);font:10px/1.4 ui-monospace,SFMono-Regular,monospace}
  button{border:1px solid var(--color-border);border-radius:999px;padding:3px var(--space-2);background:var(--color-elevated);color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:disabled{opacity:.45;cursor:not-allowed}
</style>
