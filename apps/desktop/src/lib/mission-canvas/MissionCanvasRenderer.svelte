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
</script>

{#snippet renderContribution(contribution: ResolvedContribution)}
  {@const Renderer = registry.resolve(contribution)}
  {#if Renderer}
    <Renderer {contribution}/>
  {/if}
{/snippet}

<MissionCanvasFrame {projection} {renderContribution}/>
