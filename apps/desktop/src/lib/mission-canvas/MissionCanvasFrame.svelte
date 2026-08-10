<script lang="ts">
  import type { Snippet } from 'svelte';
  import ActualWorkSurfaceStrip from './ActualWorkSurfaceStrip.svelte';
  import MissionCanvasContextBar from './MissionCanvasContextBar.svelte';
  import ProjectionLayoutRenderer from './ProjectionLayoutRenderer.svelte';
  import { DEFAULT_CONTRIBUTION_REGISTRY } from './default-contribution-registry';
  import type { ContributionRendererRegistry } from './contribution-renderers';
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from './types';

  let {
    projection,
    renderContribution,
    onSelectTab,
    registry = DEFAULT_CONTRIBUTION_REGISTRY
  }: {
    projection: ResolvedWorkspaceProjection;
    renderContribution: Snippet<[ResolvedContribution]>;
    onSelectTab?: (contributionId: string) => void;
    registry?: ContributionRendererRegistry;
  } = $props();

  const contributions = $derived(
    new Map(projection.eligible_contributions.map((item) => [item.contribution_id, item]))
  );
  const hasWorkSurfaces = $derived(
    projection.eligible_contributions.some((item) => item.kind === 'focused_work_surface')
  );
</script>

<section
  class="mission-canvas-frame"
  class:with-work-surfaces={hasWorkSurfaces}
  aria-label="Mission Canvas workspace"
  data-profile={projection.workspace_profile_id}
  data-activity={projection.activity_mode_id}
  data-projection-revision={projection.projection_revision}
  data-layout-revision={projection.layout_revision}
>
  <MissionCanvasContextBar {projection}/>
  <ActualWorkSurfaceStrip {projection} onSelect={onSelectTab}/>
  <div class="projection-region">
    <ProjectionLayoutRenderer node={projection.layout_tree} {contributions} {renderContribution} {onSelectTab} {registry}/>
  </div>
</section>

<style>
  .mission-canvas-frame{container:mission-canvas / inline-size;min-width:0;min-height:0;height:100%;display:grid;grid-template-rows:auto minmax(0,1fr);gap:5px;padding:6px;overflow:hidden;background:var(--color-bg)}
  .mission-canvas-frame.with-work-surfaces{grid-template-rows:auto auto minmax(0,1fr)}
  .projection-region{min-width:0;min-height:0;overflow:auto}
</style>
