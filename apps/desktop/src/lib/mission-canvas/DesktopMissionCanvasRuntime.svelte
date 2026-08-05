<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import ActivityNavigation from './ActivityNavigation.svelte';
  import type { ContributionRendererRegistry } from './contribution-renderers';
  import { MissionCanvasEventClient, SessionEventCursorStore } from './event-client';
  import { MissionCanvasInvalidationController } from './invalidation-controller';
  import MissionCanvasRenderer from './MissionCanvasRenderer.svelte';
  import { MissionCanvasProjectionController } from './projection-controller.svelte';
  import type { ActivityMode, ExactScope, WorkspaceProfile } from './types';
  import WorkspaceProfileSelector from './WorkspaceProfileSelector.svelte';

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
  let activities = $state<ActivityMode[]>([]);
  let profiles = $state<WorkspaceProfile[]>([]);
  let controlsGeneration = 0;

  $effect(() => {
    if (!scope) {
      controller.clear();
      activities = [];
      profiles = [];
      return;
    }
    const exactScope = scope;
    const generation = ++controlsGeneration;
    void controller.load(exactScope);
    void Promise.all([
      client.activityList({ scope: exactScope }),
      client.profileList({ scope: exactScope })
    ]).then(([nextActivities, nextProfiles]) => {
      if (generation !== controlsGeneration) return;
      activities = nextActivities;
      profiles = nextProfiles.filter((profile) => profile.installed);
    }).catch(() => {
      if (generation !== controlsGeneration) return;
      activities = [];
      profiles = [];
    });
    return () => { controlsGeneration += 1; };
  });

  async function selectActivity(activity: ActivityMode): Promise<void> {
    if (!scope) return;
    try {
      const projection = await client.activitySelect({ scope, activity_mode_id: activity.activity_mode_id });
      controller.accept(scope, projection);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'activity_selection_failed');
    }
  }

  async function selectProfile(profile: WorkspaceProfile): Promise<void> {
    if (!scope) return;
    try {
      const projection = await client.profileSelect({ scope, profile_id: profile.profile_id });
      controller.accept(scope, projection);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'profile_selection_failed');
    }
  }

  async function selectTab(contributionId: string): Promise<void> {
    if (!scope) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const commandId = crypto.randomUUID();
    try {
      await client.layoutMutate({
        action: 'set_active_tab',
        attachment_id: scope.attachment_id,
        command_id: commandId,
        expected_layout_revision: state.projection.layout_revision,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: commandId,
        scope,
        target_contribution_id: contributionId
      });
      await controller.load(scope);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'tab_selection_failed');
    }
  }

  $effect(() => {
    if (!scope) return;
    const boundScope = scope;
    const events = new MissionCanvasEventClient(client, boundScope, new SessionEventCursorStore());
    const invalidations = new MissionCanvasInvalidationController(() => controller.load(boundScope));
    const unsubscribe = events.subscribe((batch) => {
      const state = controller.state;
      if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') return;
      invalidations.enqueue(batch, {
        projectionRevision: state.projection.projection_revision,
        layoutRevision: state.projection.layout_revision
      });
    });
    events.start();
    return () => {
      unsubscribe();
      events.stop();
      invalidations.dispose();
    };
  });
</script>

<div class="desktop-canvas-runtime" data-runtime-state={controller.state.kind}>
  {#if activities.length > 1 || profiles.length > 1}
    <header class="workspace-controls">
      <WorkspaceProfileSelector {profiles} activeProfileId={controller.state.kind === 'ready' || controller.state.kind === 'refreshing' || controller.state.kind === 'stale' ? controller.state.projection.workspace_profile_id : ''} onSelect={(profile) => void selectProfile(profile)}/>
      <ActivityNavigation {activities} activeActivityModeId={controller.state.kind === 'ready' || controller.state.kind === 'refreshing' || controller.state.kind === 'stale' ? controller.state.projection.activity_mode_id : ''} onSelect={(activity) => void selectActivity(activity)}/>
    </header>
  {/if}
  {#if controller.state.kind === 'ready'}
    <MissionCanvasRenderer projection={controller.state.projection} {registry} onSelectTab={(id) => void selectTab(id)}/>
  {:else if controller.state.kind === 'refreshing'}
    <div class="state-banner" role="status">Refreshing canonical workspace…</div>
    <MissionCanvasRenderer projection={controller.state.projection} {registry} onSelectTab={(id) => void selectTab(id)}/>
  {:else if controller.state.kind === 'stale'}
    <div class="state-banner" role="status">{controller.state.reason}</div>
    <MissionCanvasRenderer projection={controller.state.projection} {registry} onSelectTab={(id) => void selectTab(id)}/>
  {:else if controller.state.kind === 'loading'}
    <div class="state-message" role="status">Loading canonical workspace…</div>
  {:else if controller.state.kind === 'blocked' || controller.state.kind === 'error'}
    <div class="state-message error" role="alert">{controller.state.reason}</div>
  {/if}
</div>

<style>
  .desktop-canvas-runtime{position:relative;display:grid;grid-template-rows:auto minmax(0,1fr);min-width:0;min-height:0;height:100%}
  .workspace-controls{display:flex;align-items:center;gap:var(--layout-control-gap);min-width:0;border-bottom:1px solid var(--color-border);background:var(--color-panel)}
  .workspace-controls :global(nav){flex:1}
  .workspace-controls :global(label){padding-inline:var(--space-3)}
  .state-banner{position:absolute;z-index:2;inset-block-start:var(--space-2);inset-inline:var(--space-2);padding:var(--space-2) var(--space-3);border:1px solid var(--color-warning);border-radius:var(--radius-control);background:var(--color-raised);color:var(--color-warning);font:var(--type-caption)}
  .state-message{align-self:center;justify-self:center;padding:var(--layout-card-padding);color:var(--color-text-secondary)}
  .state-message.error{max-width:34rem;border:1px solid var(--color-error);border-radius:var(--radius-card);background:var(--color-raised);color:var(--color-error)}
</style>
