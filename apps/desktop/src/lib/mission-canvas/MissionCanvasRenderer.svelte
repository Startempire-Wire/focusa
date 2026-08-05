<script lang="ts">
  import MissionCanvasFrame from './MissionCanvasFrame.svelte';
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from './types';
  import type { ContributionRendererRegistry } from './contribution-renderers';

  let {
    projection,
    registry
  }: {
    projection: ResolvedWorkspaceProjection;
    registry: ContributionRendererRegistry;
  } = $props();

  const unavailable = $derived(
    projection.eligible_contributions.filter((contribution) => !registry.resolve(contribution))
  );
</script>

{#snippet renderContribution(contribution: ResolvedContribution)}
  {@const Renderer = registry.resolve(contribution)}
  {#if Renderer}
    <Renderer {contribution}/>
  {/if}
{/snippet}

{#if unavailable.length > 0}
  <section class="renderer-blocked" role="alert" aria-label="Mission Canvas renderer unavailable">
    <strong>Renderer unavailable</strong>
    <span>The canonical workspace cannot be rendered by this Desktop build.</span>
    <ul>
      {#each unavailable as contribution (contribution.contribution_id)}
        <li data-unavailable-renderer={contribution.renderer_binding_id}>{contribution.accessibility.label}</li>
      {/each}
    </ul>
  </section>
{:else}
  <MissionCanvasFrame {projection} {renderContribution}/>
{/if}

<style>
  .renderer-blocked{align-self:center;justify-self:center;display:grid;gap:var(--space-2);max-width:34rem;padding:var(--layout-card-padding-roomy);border:1px solid var(--color-warning);border-radius:var(--radius-card);background:var(--color-raised);color:var(--color-text)}
  .renderer-blocked span,.renderer-blocked li{color:var(--color-text-secondary)}
  .renderer-blocked ul{margin:0;padding-inline-start:var(--space-5)}
</style>
