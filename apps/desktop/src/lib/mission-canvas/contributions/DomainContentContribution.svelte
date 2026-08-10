<script lang="ts">
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../types';
  let { contribution, projection }: { contribution: ResolvedContribution; projection: ResolvedWorkspaceProjection } = $props();
  const label = $derived(
    contribution.accessibility.label.startsWith('contribution:')
      ? contribution.accessibility.label.replace(/^contribution:/, '').replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
      : contribution.accessibility.label
  );
</script>
<section class="domain-content" aria-label={label}>
  <header><strong>{label}</strong><span>{contribution.freshness.status}</span></header>
  <div class="summary"><p>{contribution.accessibility.description || 'Canonical content resolved for the active Work Surface.'}</p></div>
</section>
<style>
  .domain-content{border:1px solid var(--color-border);border-radius:9px;padding:11px;background:var(--color-panel);height:100%}
  header{display:flex;justify-content:space-between;align-items:center}
  strong{color:var(--color-text);font:var(--type-label)}
  span{color:var(--color-text-tertiary);font:var(--type-caption)}
  .summary{margin-top:8px}p{color:var(--color-text-secondary);font:var(--type-caption)}
</style>
