<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import type { ContributionRendererRegistry } from './contribution-renderers';
  import MissionCanvasRenderer from './MissionCanvasRenderer.svelte';
  import { MissionCanvasProjectionController } from './projection-controller.svelte';
  import type { ExactScope } from './types';

  let {
    scope,
    client,
    registry
  }: {
    scope?: ExactScope;
    client: MissionCanvasClient;
    registry: ContributionRendererRegistry;
  } = $props();

  const controller = new MissionCanvasProjectionController((exactScope) => client.projectionGet({ scope: exactScope }));

  $effect(() => {
    if (scope) void controller.load(scope);
    else controller.clear();
  });
</script>

<div class="desktop-canvas-runtime" data-runtime-state={controller.state.kind}>
  {#if controller.state.kind === 'ready'}
    <MissionCanvasRenderer projection={controller.state.projection} {registry}/>
  {:else if controller.state.kind === 'stale'}
    <div class="state-banner" role="status">{controller.state.reason}</div>
    <MissionCanvasRenderer projection={controller.state.projection} {registry}/>
  {:else if controller.state.kind === 'loading'}
    <div class="state-message" role="status">Loading canonical workspace…</div>
  {:else if controller.state.kind === 'blocked' || controller.state.kind === 'error'}
    <div class="state-message error" role="alert">{controller.state.reason}</div>
  {/if}
</div>

<style>
  .desktop-canvas-runtime{position:relative;display:grid;min-width:0;min-height:0;height:100%}
  .state-banner{position:absolute;z-index:2;inset-block-start:var(--space-2);inset-inline:var(--space-2);padding:var(--space-2) var(--space-3);border:1px solid var(--color-warning);border-radius:var(--radius-control);background:var(--color-raised);color:var(--color-warning);font:var(--type-caption)}
  .state-message{align-self:center;justify-self:center;padding:var(--layout-card-padding);color:var(--color-text-secondary)}
  .state-message.error{max-width:34rem;border:1px solid var(--color-error);border-radius:var(--radius-card);background:var(--color-raised);color:var(--color-error)}
</style>
