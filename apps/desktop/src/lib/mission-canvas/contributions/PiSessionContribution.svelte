<script lang="ts">
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../types';
  let { contribution, projection }: { contribution: ResolvedContribution; projection: ResolvedWorkspaceProjection } = $props();
  const attached = $derived(Boolean(projection.attachment));
  const actionCount = $derived(projection.operation_bindings.filter((binding) => binding.target_contribution_id === contribution.contribution_id && binding.enabled).length);
</script>
<section class="pi-session" aria-label={contribution.accessibility.label}>
  <header><div><i></i><strong>Pi Session</strong></div><span>{attached ? 'Attached · live' : 'Unavailable'}</span></header>
  <div class="session-body">
    <article><span>Runtime</span><strong>{projection.runtime_object?.runtime_kind?.replace(/_/g, ' ') ?? 'Pi session'}</strong></article>
    <article><span>Work Surface</span><strong>{projection.work_surface_id ? 'Focused' : 'Not selected'}</strong></article>
    <article><span>Governed actions</span><strong>{actionCount} available</strong></article>
  </div>
  <footer>Terminal interaction remains in the authentic Agent TUI Work Surface.</footer>
</section>
<style>
  .pi-session{display:grid;grid-template-rows:auto 1fr auto;height:100%;min-height:150px;border:1px solid var(--color-border);border-radius:9px;background:var(--color-panel);overflow:hidden}header{display:flex;align-items:center;justify-content:space-between;padding:9px 11px;border-bottom:1px solid var(--color-border)}header div{display:flex;align-items:center;gap:7px}header i{width:9px;height:9px;border-radius:50%;background:var(--color-accent)}header strong{font:var(--type-label)}header span{color:var(--color-success);font:var(--type-caption)}.session-body{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:7px;padding:10px}.session-body article{display:grid;align-content:center;gap:5px;padding:9px;border:1px solid var(--color-border);border-radius:7px;background:var(--color-bg)}article span,footer{color:var(--color-text-tertiary);font:var(--type-caption)}article strong{color:var(--color-text-secondary);font:var(--type-label);text-transform:capitalize}footer{padding:7px 11px;border-top:1px solid var(--color-border)}
</style>
