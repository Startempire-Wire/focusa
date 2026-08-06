<script lang="ts">
  import type { Snippet } from 'svelte';
  import MissionCanvasContextBar from './MissionCanvasContextBar.svelte';
  import ProjectionLayoutRenderer from './ProjectionLayoutRenderer.svelte';
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from './types';

  let {
    projection,
    renderContribution,
    onSelectTab
  }: {
    projection: ResolvedWorkspaceProjection;
    renderContribution: Snippet<[ResolvedContribution]>;
    onSelectTab?: (contributionId: string) => void;
  } = $props();

  const contributions = $derived(
    new Map(projection.eligible_contributions.map((item) => [item.contribution_id, item]))
  );
</script>

<section
  class="mission-canvas-frame"
  aria-label="Mission Canvas workspace"
  data-profile={projection.workspace_profile_id}
  data-activity={projection.activity_mode_id}
  data-projection-revision={projection.projection_revision}
  data-layout-revision={projection.layout_revision}
>
  <MissionCanvasContextBar {projection}/>
  <div class="projection-region">
    <ProjectionLayoutRenderer node={projection.layout_tree} {contributions} {renderContribution} {onSelectTab}/>
  </div>
</section>

<style>
  .mission-canvas-frame{container:mission-canvas / inline-size;min-width:0;min-height:0;height:100%;display:grid;grid-template-rows:auto minmax(0,1fr);overflow:hidden}
  .projection-region{min-width:0;min-height:0;overflow:auto}
</style>
