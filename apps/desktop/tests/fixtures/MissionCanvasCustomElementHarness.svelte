<script lang="ts">
  import MissionCanvasRenderer from '../../src/lib/mission-canvas/MissionCanvasRenderer.svelte';
  import { ContributionRendererRegistry } from '../../src/lib/mission-canvas/contribution-renderers';
  import { trustedCustomElementRenderer } from '../../src/lib/mission-canvas/custom-element-renderer';
  import type { ResolvedWorkspaceProjection } from '../../src/lib/mission-canvas/types';
  import projection from './mission-canvas/populated-projection.json';

  const registry = new ContributionRendererRegistry([
    trustedCustomElementRenderer({
      rendererBindingId: 'renderer:pi-session@v1',
      semanticBindingIds: ['semantic:pi-session'],
      elementName: 'fixture-pi-session'
    }),
    trustedCustomElementRenderer({
      rendererBindingId: 'renderer:focusa-inspector@v1',
      semanticBindingIds: ['semantic:focusa-inspector'],
      elementName: 'fixture-focusa-inspector'
    })
  ]);
</script>

<MissionCanvasRenderer projection={projection as ResolvedWorkspaceProjection} {registry}/>
