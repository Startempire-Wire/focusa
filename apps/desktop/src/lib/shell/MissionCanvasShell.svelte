<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import DesktopMissionCanvasRuntime from '$lib/mission-canvas/DesktopMissionCanvasRuntime.svelte';
  import MissionCanvasRenderer from '$lib/mission-canvas/MissionCanvasRenderer.svelte';
  import type { ContributionRendererRegistry } from '$lib/mission-canvas/contribution-renderers';
  import type { ExactScope, ResolvedWorkspaceProjection } from '$lib/mission-canvas/types';

  let {
    projection,
    scope,
    client,
    rendererRegistry
  }: {
    projection?: ResolvedWorkspaceProjection;
    scope?: ExactScope;
    client?: MissionCanvasClient;
    rendererRegistry?: ContributionRendererRegistry;
  } = $props();
</script>

{#if scope && client && rendererRegistry}
  <section class="canvas-live" aria-label="Focusa Mission Canvas workspace">
    <DesktopMissionCanvasRuntime {scope} {client} registry={rendererRegistry}/>
  </section>
{:else if projection && rendererRegistry}
  <section class="canvas-live" aria-label="Focusa Mission Canvas workspace">
    <MissionCanvasRenderer {projection} registry={rendererRegistry}/>
  </section>
{:else}
  <section class="canvas-unbound" aria-label="Mission Canvas unbound">
    <strong>Mission Canvas is unbound</strong>
    <span>An exact Workstream and Attachment are required before canonical workspace contributions can render.</span>
  </section>
{/if}

<style>
  .canvas-live,.canvas-unbound{flex:1;min-height:0;min-width:0;overflow:hidden}
  .canvas-unbound{display:grid;place-content:center;gap:var(--space-2);padding:var(--layout-card-padding-roomy);border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-bg);color:var(--color-text);text-align:center}
  .canvas-unbound span{max-width:34rem;color:var(--color-text-secondary)}
</style>
