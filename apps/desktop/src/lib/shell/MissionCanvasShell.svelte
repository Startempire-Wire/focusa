<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import DesktopMissionCanvasRuntime from '$lib/mission-canvas/DesktopMissionCanvasRuntime.svelte';
  import { DEFAULT_CONTRIBUTION_REGISTRY } from '$lib/mission-canvas/default-contribution-registry';
  import MissionCanvasRenderer from '$lib/mission-canvas/MissionCanvasRenderer.svelte';
  import GlobalPromptEditor from './GlobalPromptEditor.svelte';
  import type { ContributionRendererRegistry } from '$lib/mission-canvas/contribution-renderers';
  import type { OperationBinding, ResolvedWorkspaceProjection, WorkstreamAuthorityContext } from '$lib/mission-canvas/types';

  let {
    projection,
    authority,
    client,
    rendererRegistry = DEFAULT_CONTRIBUTION_REGISTRY,
    executeContributionOperation
  }: {
    projection?: ResolvedWorkspaceProjection;
    authority?: WorkstreamAuthorityContext;
    client?: MissionCanvasClient;
    rendererRegistry?: ContributionRendererRegistry;
    executeContributionOperation?: (binding: OperationBinding, projection: ResolvedWorkspaceProjection) => Promise<void>;
  } = $props();
</script>

{#if authority && client && rendererRegistry}
  <section class="canvas-live" aria-label="Focusa Mission Canvas workspace">
    <DesktopMissionCanvasRuntime {authority} {client} registry={rendererRegistry} {executeContributionOperation}/>
    <GlobalPromptEditor {authority} {client} />
  </section>
{:else if projection && rendererRegistry}
  <section class="canvas-live" aria-label="Focusa Mission Canvas workspace">
    <MissionCanvasRenderer {projection} registry={rendererRegistry}/>
  </section>
{:else}
  <section class="canvas-unbound" aria-label="Mission Canvas unbound">
    <strong>Mission Canvas is unbound</strong>
    <span>A canonical Workstream is required before workspace contributions can render; runtime Attachment identity gates attached interactions.</span>
  </section>
{/if}

<style>
  .canvas-live,.canvas-unbound{flex:1;min-height:0;min-width:0;overflow:hidden}
  .canvas-live{display:flex;flex-direction:column}
  .canvas-unbound{display:grid;place-content:center;gap:var(--space-2);padding:var(--layout-card-padding-roomy);border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-bg);color:var(--color-text);text-align:center}
  .canvas-unbound span{max-width:34rem;color:var(--color-text-secondary)}
</style>
