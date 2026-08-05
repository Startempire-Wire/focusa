<script lang="ts">
  import MissionCanvasRenderer from '../../src/lib/mission-canvas/MissionCanvasRenderer.svelte';
  import { ContributionRendererRegistry } from '../../src/lib/mission-canvas/contribution-renderers';
  import type { ResolvedWorkspaceProjection } from '../../src/lib/mission-canvas/types';
  import projection from './mission-canvas/populated-projection.json';
  import Renderer from './ResolvedContributionHarnessRenderer.svelte';

  let { complete = false }: { complete?: boolean } = $props();

  const entries = $derived([
    {
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      component: Renderer
    },
    ...(complete ? [{
      rendererBindingId: 'renderer:focusa-inspector@v1',
      semanticBindingIds: ['semantic:focusa-inspector'],
      component: Renderer
    }] : [])
  ]);
  const registry = $derived(new ContributionRendererRegistry(entries));
</script>

<MissionCanvasRenderer projection={projection as ResolvedWorkspaceProjection} {registry}/>
