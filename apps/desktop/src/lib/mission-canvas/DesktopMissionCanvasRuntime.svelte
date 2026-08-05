<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import ActivityNavigation from './ActivityNavigation.svelte';
  import type { ContributionRendererRegistry } from './contribution-renderers';
  import { LocalEventCursorStore, MissionCanvasEventClient } from './event-client';
  import { MissionCanvasInvalidationController } from './invalidation-controller';
  import MissionCanvasRenderer from './MissionCanvasRenderer.svelte';
  import OperationConfirmationDialog from './OperationConfirmationDialog.svelte';
  import { MissionCanvasProjectionController } from './projection-controller.svelte';
  import type { ActivityMode, ExactScope, OperationBinding, WorkspaceProfile } from './types';
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

  const PROFILE_SELECT_OPERATION = 'focusa.mission_canvas.profile.select';
  const ACTIVITY_SELECT_OPERATION = 'focusa.mission_canvas.activity.select';
  const LAYOUT_MUTATE_OPERATION = 'focusa.mission_canvas.layout.mutate';
  const controller = new MissionCanvasProjectionController((exactScope) => client.projectionGet({ scope: exactScope }));
  let activities = $state<ActivityMode[]>([]);
  let profiles = $state<WorkspaceProfile[]>([]);
  let mutationInFlight = $state(false);
  let pendingConfirmation = $state.raw<{ binding: OperationBinding; subjectLabel: string; run: () => Promise<void> }>();
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

  function operationBinding(operationId: string, targetContributionId?: string): OperationBinding | undefined {
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') return undefined;
    return state.projection.operation_bindings.find((binding) =>
      binding.operation_id === operationId
      && binding.enabled
      && !binding.disabled_reason_ref
      && binding.authority_ref.length > 0
      && (!targetContributionId || binding.target_contribution_id === targetContributionId)
    );
  }

  function operationEnabled(operationId: string, targetContributionId?: string): boolean {
    const binding = operationBinding(operationId, targetContributionId);
    return Boolean(binding && binding.confirmation !== 'preview');
  }

  function requestOperation(binding: OperationBinding, subjectLabel: string, run: () => Promise<void>): void {
    if (!binding.confirmation || binding.confirmation === 'none') {
      void run();
      return;
    }
    pendingConfirmation = { binding, subjectLabel, run };
  }

  function confirmOperation(): void {
    const pending = pendingConfirmation;
    pendingConfirmation = undefined;
    if (!pending || pending.binding.confirmation !== 'explicit') return;
    const current = operationBinding(pending.binding.operation_id, pending.binding.target_contribution_id);
    if (!current || current.authority_ref !== pending.binding.authority_ref || current.confirmation !== 'explicit') return;
    void pending.run();
  }

  function selectActivity(activity: ActivityMode): void {
    const binding = operationBinding(ACTIVITY_SELECT_OPERATION);
    if (!binding) return;
    requestOperation(binding, activity.display_name, () => performActivitySelection(activity));
  }

  async function performActivitySelection(activity: ActivityMode): Promise<void> {
    if (!scope || mutationInFlight) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const idempotencyKey = crypto.randomUUID();
    mutationInFlight = true;
    try {
      const projection = await client.activitySelect({
        scope,
        selection_id: activity.activity_mode_id,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: idempotencyKey
      });
      controller.accept(scope, projection);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'activity_selection_failed');
    } finally {
      mutationInFlight = false;
    }
  }

  function selectProfile(profile: WorkspaceProfile): void {
    const binding = operationBinding(PROFILE_SELECT_OPERATION);
    if (!binding) return;
    requestOperation(binding, profile.display_name, () => performProfileSelection(profile));
  }

  async function performProfileSelection(profile: WorkspaceProfile): Promise<void> {
    if (!scope || mutationInFlight) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const idempotencyKey = crypto.randomUUID();
    mutationInFlight = true;
    try {
      const projection = await client.profileSelect({
        scope,
        selection_id: profile.profile_id,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: idempotencyKey
      });
      controller.accept(scope, projection);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'profile_selection_failed');
    } finally {
      mutationInFlight = false;
    }
  }

  function selectTab(contributionId: string): void {
    const binding = operationBinding(LAYOUT_MUTATE_OPERATION, contributionId);
    if (!binding) return;
    const state = controller.state;
    const subjectLabel = state.kind === 'ready' || state.kind === 'refreshing' || state.kind === 'stale'
      ? state.projection.eligible_contributions.find((item) => item.contribution_id === contributionId)?.accessibility.label ?? contributionId
      : contributionId;
    requestOperation(binding, subjectLabel, () => performTabSelection(contributionId));
  }

  async function performTabSelection(contributionId: string): Promise<void> {
    if (!scope || mutationInFlight) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const commandId = crypto.randomUUID();
    mutationInFlight = true;
    try {
      const result = await client.layoutMutate({
        action: 'set_active_tab',
        attachment_id: scope.attachment_id,
        command_id: commandId,
        expected_layout_revision: state.projection.layout_revision,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: commandId,
        scope,
        target_contribution_id: contributionId
      });
      if (!result.accepted) {
        controller.markStale(result.error_ref ?? 'layout_mutation_rejected');
        return;
      }
      if (result.projection_revision < state.projection.projection_revision || result.layout_revision < state.projection.layout_revision) {
        controller.markStale('layout_mutation_revision_regressed');
        return;
      }
      await controller.load(scope);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'tab_selection_failed');
    } finally {
      mutationInFlight = false;
    }
  }

  $effect(() => {
    if (!scope) return;
    const boundScope = scope;
    const events = new MissionCanvasEventClient(client, boundScope, new LocalEventCursorStore());
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

<div
  class="desktop-canvas-runtime"
  class:has-controls={(activities.length > 1 && operationEnabled(ACTIVITY_SELECT_OPERATION)) || (profiles.length > 1 && operationEnabled(PROFILE_SELECT_OPERATION))}
  class:mutation-pending={mutationInFlight}
  aria-busy={mutationInFlight}
  data-runtime-state={controller.state.kind}
>
  {#if (activities.length > 1 && operationEnabled(ACTIVITY_SELECT_OPERATION)) || (profiles.length > 1 && operationEnabled(PROFILE_SELECT_OPERATION))}
    <header class="workspace-controls">
      <WorkspaceProfileSelector profiles={operationEnabled(PROFILE_SELECT_OPERATION) ? profiles : []} activeProfileId={controller.state.kind === 'ready' || controller.state.kind === 'refreshing' || controller.state.kind === 'stale' ? controller.state.projection.workspace_profile_id : ''} onSelect={(profile) => void selectProfile(profile)}/>
      <ActivityNavigation activities={operationEnabled(ACTIVITY_SELECT_OPERATION) ? activities : []} activeActivityModeId={controller.state.kind === 'ready' || controller.state.kind === 'refreshing' || controller.state.kind === 'stale' ? controller.state.projection.activity_mode_id : ''} onSelect={(activity) => void selectActivity(activity)}/>
    </header>
  {/if}
  {#if controller.state.kind === 'ready'}
    <MissionCanvasRenderer projection={controller.state.projection} {registry} onSelectTab={operationEnabled(LAYOUT_MUTATE_OPERATION) ? (id) => void selectTab(id) : undefined}/>
  {:else if controller.state.kind === 'refreshing'}
    <div class="state-banner" role="status">Refreshing canonical workspace…</div>
    <MissionCanvasRenderer projection={controller.state.projection} {registry} onSelectTab={operationEnabled(LAYOUT_MUTATE_OPERATION) ? (id) => void selectTab(id) : undefined}/>
  {:else if controller.state.kind === 'stale'}
    <div class="state-banner" role="status">{controller.state.reason}</div>
    <MissionCanvasRenderer projection={controller.state.projection} {registry} onSelectTab={operationEnabled(LAYOUT_MUTATE_OPERATION) ? (id) => void selectTab(id) : undefined}/>
  {:else if controller.state.kind === 'loading'}
    <div class="state-message" role="status">Loading canonical workspace…</div>
  {:else if controller.state.kind === 'blocked' || controller.state.kind === 'error'}
    <div class="state-message error" role="alert">{controller.state.reason}</div>
  {/if}
  {#if pendingConfirmation}
    <OperationConfirmationDialog
      binding={pendingConfirmation.binding}
      subjectLabel={pendingConfirmation.subjectLabel}
      onConfirm={confirmOperation}
      onCancel={() => (pendingConfirmation = undefined)}
    />
  {/if}
</div>

<style>
  .desktop-canvas-runtime{position:relative;display:grid;grid-template-rows:minmax(0,1fr);min-width:0;min-height:0;height:100%}
  .desktop-canvas-runtime.has-controls{grid-template-rows:auto minmax(0,1fr)}
  .workspace-controls{display:flex;align-items:center;gap:var(--layout-control-gap);min-width:0;border-bottom:1px solid var(--color-border);background:var(--color-panel)}
  .workspace-controls :global(nav){flex:1}
  .workspace-controls :global(label){padding-inline:var(--space-3)}
  .mutation-pending .workspace-controls,.mutation-pending :global([role='tablist']){pointer-events:none;opacity:.72}
  .state-banner{position:absolute;z-index:2;inset-block-start:var(--space-2);inset-inline:var(--space-2);padding:var(--space-2) var(--space-3);border:1px solid var(--color-warning);border-radius:var(--radius-control);background:var(--color-raised);color:var(--color-warning);font:var(--type-caption)}
  .state-message{align-self:center;justify-self:center;padding:var(--layout-card-padding);color:var(--color-text-secondary)}
  .state-message.error{max-width:34rem;border:1px solid var(--color-error);border-radius:var(--radius-card);background:var(--color-raised);color:var(--color-error)}
</style>
