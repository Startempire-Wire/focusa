<script lang="ts">
  import ProjectionLayoutRenderer from '../../src/lib/mission-canvas/ProjectionLayoutRenderer.svelte';
  import { DEFAULT_CONTRIBUTION_REGISTRY } from '../../src/lib/mission-canvas/default-contribution-registry';
  import type { ContributionRendererRegistry } from '../../src/lib/mission-canvas/contribution-renderers';
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../../src/lib/mission-canvas/types';

  let {
    projection,
    registry = DEFAULT_CONTRIBUTION_REGISTRY
  }: {
    projection: ResolvedWorkspaceProjection;
    registry?: ContributionRendererRegistry;
  } = $props();

  const contributions = $derived(
    new Map(projection.eligible_contributions.map((item) => [item.contribution_id, item]))
  );
</script>

{#snippet renderContribution(contribution: ResolvedContribution)}
  <article data-rendered-contribution={contribution.contribution_id}>
    <h2>{contribution.accessibility.label}</h2>
  </article>
{/snippet}

<ProjectionLayoutRenderer
  node={projection.layout_tree}
  {contributions}
  {registry}
  {renderContribution}
/>
