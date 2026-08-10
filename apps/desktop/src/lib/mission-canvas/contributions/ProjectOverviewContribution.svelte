<script lang="ts">
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../types';
  let { contribution, projection }: { contribution: ResolvedContribution; projection: ResolvedWorkspaceProjection } = $props();
  const project = $derived(projection.workstream.scope.scope_key.canonical_name ?? 'Project');
  const surfaceCount = $derived(projection.eligible_contributions.filter((item) => item.kind === 'focused_work_surface').length);
  const enabledActions = $derived(projection.operation_bindings.filter((item) => item.enabled).length);
</script>
<section class="project-overview" aria-label={contribution.accessibility.label}>
  <header><div><span>Overview</span><h2>{project} · Project Home</h2></div><em>Live</em></header>
  <div class="summary-grid">
    <article><i class="purple"></i><span>Mission context</span><strong>{projection.focused_semantic_target ? 'Focused' : 'Available'}</strong></article>
    <article><i class="green"></i><span>Active workspace</span><strong>{projection.workspace_profile_id}</strong></article>
    <article><i class="blue"></i><span>Work Surfaces</span><strong>{surfaceCount} available</strong></article>
    <article><i class="green"></i><span>Governed actions</span><strong>{enabledActions} enabled</strong></article>
  </div>
  <div class="focus-panel">
    <span>Current activity</span>
    <strong>{projection.activity_mode_id.replace(/[-_]+/g, ' ')}</strong>
    <small>Composition resolved by Core · layout r{projection.layout_revision}</small>
  </div>
</section>
<style>
  .project-overview{display:grid;grid-template-rows:auto auto minmax(90px,1fr);gap:8px;height:100%;padding:11px;border:1px solid var(--color-border);border-radius:9px;background:var(--color-panel)}
  header{display:flex;align-items:center;justify-content:space-between}header div{display:flex;align-items:baseline;gap:8px}header span{color:var(--color-text-tertiary);font:var(--type-caption)}h2{margin:0;color:var(--color-text);font-size:16px}em{color:var(--color-success);font:var(--type-caption);font-style:normal}
  .summary-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px}.summary-grid article{display:grid;grid-template-columns:auto 1fr;gap:2px 7px;padding:8px;border:1px solid var(--color-border);border-radius:7px;background:var(--color-raised)}article i{grid-row:1/3;width:10px;height:10px;margin-top:2px;border-radius:50%}.purple{background:var(--color-accent)}.green{background:var(--color-success)}.blue{background:#4fa8ff}article span{color:var(--color-text-secondary);font:var(--type-caption)}article strong{color:var(--color-text-tertiary);font:var(--type-caption);font-weight:500}
  .focus-panel{display:grid;align-content:center;gap:5px;padding:12px;border:1px solid var(--color-border);border-radius:7px;background:var(--color-bg)}.focus-panel span{color:var(--color-text-tertiary);font:var(--type-overline);text-transform:uppercase}.focus-panel strong{color:var(--color-text);font-size:15px;text-transform:capitalize}.focus-panel small{color:var(--color-text-tertiary)}
  @container mission-canvas (max-width:800px){.summary-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>
