<script lang="ts">
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import type { ResolvedContribution } from '../types';

  let { contribution }: { contribution: ResolvedContribution } = $props();
  const tone = $derived(contribution.freshness.status === 'current' ? 'ready' : contribution.freshness.status === 'stale' ? 'watch' : 'neutral');
</script>

<section class="canonical-contribution" aria-label={contribution.accessibility.label} data-contribution-kind={contribution.kind}>
  <header>
    <div>
      <strong>{contribution.accessibility.label}</strong>
      {#if contribution.accessibility.description}<span>{contribution.accessibility.description}</span>{/if}
    </div>
    <StatusBadge {tone} label={contribution.freshness.status}/>
  </header>
  <div class="reference" aria-label="Canonical content reference">
    <span>{contribution.data_ref.kind}</span>
    <code>{contribution.data_ref.ref}</code>
  </div>
</section>

<style>
  .canonical-contribution{display:grid;grid-template-rows:auto minmax(0,1fr);height:100%;min-height:0;border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-panel);overflow:hidden}
  header{display:flex;align-items:flex-start;justify-content:space-between;gap:var(--space-3);padding:var(--space-3) var(--space-4);border-bottom:1px solid var(--color-border)}
  header div{display:grid;gap:var(--space-1);min-width:0}
  header strong{color:var(--color-text);font:var(--type-label)}
  header span{color:var(--color-text-tertiary);font:var(--type-caption)}
  .reference{align-self:center;justify-self:center;display:grid;justify-items:center;gap:var(--space-2);max-width:100%;padding:var(--layout-card-padding-roomy);color:var(--color-text-tertiary)}
  .reference span{font:var(--type-caption);text-transform:uppercase;letter-spacing:.08em}
  code{max-width:100%;overflow:hidden;color:var(--color-text-secondary);font:var(--type-code);text-overflow:ellipsis;white-space:nowrap}
</style>
