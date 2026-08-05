<script lang="ts">
  import MissionCanvasFrame from '../../src/lib/mission-canvas/MissionCanvasFrame.svelte';
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../../src/lib/mission-canvas/types';
  import populated from './mission-canvas/populated-projection.json';
  import variants from './mission-canvas/layout-variants.json';

  let { variant = 'populated' }: { variant?: keyof typeof variants | 'populated' } = $props();
  const projection = $derived(
    (variant === 'populated' ? populated : variants[variant]) as ResolvedWorkspaceProjection
  );
</script>

{#snippet renderContribution(contribution: ResolvedContribution)}
  <article data-rendered-contribution={contribution.contribution_id}>
    <h2>{contribution.accessibility.label}</h2>
  </article>
{/snippet}

<MissionCanvasFrame {projection} {renderContribution}/>
