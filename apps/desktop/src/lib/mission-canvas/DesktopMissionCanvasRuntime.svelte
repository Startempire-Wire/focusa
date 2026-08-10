<script lang="ts">
  import { tick, untrack } from 'svelte';
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import ActivityNavigation from './ActivityNavigation.svelte';
  import { CapabilityLossController } from './capability-loss-controller';
  import type { ContributionRendererRegistry } from './contribution-renderers';
  import { LocalEventCursorStore, MissionCanvasEventClient } from './event-client';
  import { MissionCanvasInvalidationController } from './invalidation-controller';
  import MissionCanvasRenderer from './MissionCanvasRenderer.svelte';
  import OperationConfirmationDialog from './OperationConfirmationDialog.svelte';
  import { capturePresentationState, restoreIfStillPresent, type PresentationStateSnapshot } from './presentation-state';
  import { MissionCanvasRestorationController } from './restoration-controller';
  import { MissionCanvasProjectionController } from './projection-controller.svelte';
  import type { ActivityMode, OperationBinding, ResolvedWorkspaceProjection, WorkstreamAuthorityContext, WorkspaceProfile } from './types';
  import WorkspaceProfileSelector from './WorkspaceProfileSelector.svelte';

  let {
    authority,
    client,
    registry,
    executeContributionOperation
  }: {
    authority?: WorkstreamAuthorityContext;
    client: MissionCanvasClient;
    registry: ContributionRendererRegistry;
    executeContributionOperation?: (binding: OperationBinding, projection: ResolvedWorkspaceProjection) => Promise<void>;
  } = $props();

  const PROFILE_SELECT_OPERATION = 'focusa.mission_canvas.profile.select';
  const ACTIVITY_SELECT_OPERATION = 'focusa.mission_canvas.activity.select';
  const LAYOUT_MUTATE_OPERATION = 'focusa.mission_canvas.layout.mutate';
  const controller = new MissionCanvasProjectionController((boundAuthority) => client.projectionGet({ ...boundAuthority }));
  const restoration = typeof localStorage !== 'undefined'
    ? new MissionCanvasRestorationController(localStorage)
    : undefined;
  let activities = $state<ActivityMode[]>([]);
  let profiles = $state<WorkspaceProfile[]>([]);
  let mutationInFlight = $state(false);
  let pendingConfirmation = $state.raw<{ binding: OperationBinding; subjectLabel: string; run: () => Promise<void> }>();
  let capabilityNotice = $state<string | undefined>();
  let presentationRoot = $state<HTMLElement | undefined>();
  let controlsGeneration = 0;

  $effect(() => {
    pendingConfirmation = undefined;
    if (!authority) {
      controller.clear();
      activities = [];
      profiles = [];
      return;
    }
    const boundAuthority = authority;
    const generation = ++controlsGeneration;
    // The presentation snapshot reads controller.state + presentationRoot; those
    // are DOM/state snapshots, not effect dependencies. Reading them tracked
    // made this effect depend on the very controller.state that load() writes,
    // causing an infinite loading->ready->loading re-run loop in the browser
    // (invisible to SSR where $effect never runs).
    const snapshot = untrack(() => captureCurrentPresentation());
    // controller.load() synchronously reads this.state (projectionForScope) and
    // then writes a new loading/refreshing state object; if those reads were
    // tracked, every state write would re-run this effect and restart the load
    // forever. The effect must depend only on the binding props, not on the
    // controller's own lifecycle state.
    const loadPromise = untrack(() => controller.load(boundAuthority));
    void loadPromise.then(() => {
      const loaded = controller.state;
      const projection = loaded.kind === 'ready' || loaded.kind === 'refreshing' || loaded.kind === 'stale'
        ? loaded.projection
        : undefined;
      // Restart/reconnect: a persisted snapshot is advisory and only restores
      // presentation for surfaces still present in the canonical projection.
      if (restoration && projection) {
        restoration.apply(boundAuthority, projection, (candidate) => {
          if (candidate) restorePresentation(candidate);
        });
        const presentation = captureCurrentPresentation();
        if (presentation) {
          restoration.persist(boundAuthority, projection.projection_revision, presentation);
        }
      }
      restorePresentation(snapshot);
    });
    void Promise.all([
      client.activityList({ ...boundAuthority }),
      client.profileList({ ...boundAuthority })
    ]).then(([nextActivities, nextProfiles]) => {
      if (generation !== controlsGeneration) return;
      // `profileList` is the generated Core-owned eligibility result. Keep its
      // ordering and DTOs intact; the selector only applies renderer-boundary
      // fail-closed checks and never recomputes meaningful composition.
      activities = nextActivities;
      profiles = nextProfiles;
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

  function invokeContributionOperation(binding: OperationBinding): void {
    if (!executeContributionOperation || binding.confirmation === 'preview') return;
    const current = operationBinding(binding.operation_id, binding.target_contribution_id);
    if (!current || current.authority_ref !== binding.authority_ref) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') return;
    const contribution = state.projection.eligible_contributions.find((item) => item.contribution_id === binding.target_contribution_id);
    if (!contribution) return;
    requestOperation(current, contribution.accessibility.label, async () => {
      if (mutationInFlight) return;
      const latest = operationBinding(current.operation_id, current.target_contribution_id);
      const latestState = controller.state;
      if (!latest || latest.authority_ref !== current.authority_ref) return;
      if (latestState.kind !== 'ready' && latestState.kind !== 'refreshing' && latestState.kind !== 'stale') return;
      const snapshot = captureCurrentPresentation();
      mutationInFlight = true;
      try {
        await executeContributionOperation(latest, latestState.projection);
        if (authority) {
          await controller.load(authority);
          await restorePresentation(snapshot);
        }
      } catch (error) {
        controller.markStale(error instanceof Error ? error.message : 'contribution_operation_failed');
      } finally {
        mutationInFlight = false;
      }
    });
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
    if (!authority || mutationInFlight) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const snapshot = captureCurrentPresentation();
    const idempotencyKey = crypto.randomUUID();
    mutationInFlight = true;
    try {
      const projection = await client.activitySelect({
        ...authority,
        selection_id: activity.activity_mode_id,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: idempotencyKey
      });
      controller.accept(authority, projection);
      await restorePresentation(snapshot);
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
    if (!authority || mutationInFlight) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const snapshot = captureCurrentPresentation();
    const idempotencyKey = crypto.randomUUID();
    mutationInFlight = true;
    try {
      const projection = await client.profileSelect({
        ...authority,
        selection_id: profile.profile_id,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: idempotencyKey
      });
      controller.accept(authority, projection);
      await restorePresentation(snapshot);
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
    if (!authority || !authority.attachment || mutationInFlight) return;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'stale') return;
    const snapshot = captureCurrentPresentation();
    const commandId = crypto.randomUUID();
    mutationInFlight = true;
    try {
      const result = await client.layoutMutate({
        ...authority,
        action: 'set_active_tab',
        attachment: authority.attachment,
        command_id: commandId,
        expected_layout_revision: state.projection.layout_revision,
        expected_projection_revision: state.projection.projection_revision,
        idempotency_key: commandId,
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
      await controller.load(authority);
      await restorePresentation(snapshot);
    } catch (error) {
      controller.markStale(error instanceof Error ? error.message : 'tab_selection_failed');
    } finally {
      mutationInFlight = false;
    }
  }

  function captureCurrentPresentation(): PresentationStateSnapshot | undefined {
    if (!presentationRoot) return undefined;
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') return undefined;
    return capturePresentationState(presentationRoot, state.projection);
  }

  async function restorePresentation(snapshot: PresentationStateSnapshot | undefined): Promise<void> {
    if (!snapshot || !presentationRoot) return;
    await tick();
    const state = controller.state;
    if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') return;
    restoreIfStillPresent(presentationRoot, snapshot, state.projection);
  }

  $effect(() => {
    if (!authority) return;
    const boundAuthority = authority;
    const events = new MissionCanvasEventClient(client, boundAuthority, new LocalEventCursorStore());
    const refreshProjection = async (): Promise<ResolvedWorkspaceProjection> => {
      const snapshot = captureCurrentPresentation();
      await controller.refresh(boundAuthority);
      await restorePresentation(snapshot);
      const state = controller.state;
      if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') {
        throw new Error('projection_refresh_unavailable');
      }
      return state.projection;
    };
    const invalidation = new MissionCanvasInvalidationController(async () => { await refreshProjection(); });
    const capabilityLoss = new CapabilityLossController(refreshProjection);
    let noticeTimer: ReturnType<typeof setTimeout> | undefined;
    const unsubscribe = events.subscribe((batch) => {
      const state = controller.state;
      if (state.kind !== 'ready' && state.kind !== 'refreshing' && state.kind !== 'stale') return;
      const capabilityEvents = batch.accepted.filter((event) => event.event_kind === 'capability_changed');
      if (capabilityEvents.length > 0) {
        void capabilityLoss.handle(capabilityEvents, state.projection, boundAuthority).then((result) => {
          if (!result) return;
          capabilityNotice = result.notification?.message;
          if (noticeTimer) clearTimeout(noticeTimer);
          if (capabilityNotice) noticeTimer = setTimeout(() => { capabilityNotice = undefined; }, 5000);
        });
      }
      const remaining = batch.accepted.filter((event) => event.event_kind !== 'capability_changed');
      if (remaining.length > 0) {
        invalidation.coalesce({ ...batch, accepted: remaining }, {
          projectionRevision: state.projection.projection_revision,
          layoutRevision: state.projection.layout_revision,
          durableEventCursor: state.projection.durable_event_cursor,
          authority: boundAuthority
        }, boundAuthority);
      }
    });
    events.start();
    return () => {
      unsubscribe();
      events.stop();
      capabilityLoss.cancel();
      if (noticeTimer) clearTimeout(noticeTimer);
      invalidation.dispose();
    };
  });
</script>

<div
  bind:this={presentationRoot}
  class="desktop-canvas-runtime"
  data-presentation-root="true"
  class:has-controls={profiles.length > 1 && operationEnabled(PROFILE_SELECT_OPERATION)}
  class:mutation-pending={mutationInFlight}
  aria-busy={mutationInFlight}
  data-runtime-state={controller.state.kind}
>
  {#if capabilityNotice}
    <p class="capability-notice" role="status">{capabilityNotice}</p>
  {/if}
  {#if profiles.length > 1 && operationEnabled(PROFILE_SELECT_OPERATION)}
    <header class="workspace-controls">
      <WorkspaceProfileSelector profiles={profiles} activeProfileId={controller.state.kind === 'ready' || controller.state.kind === 'refreshing' || controller.state.kind === 'stale' ? controller.state.projection.workspace_profile_id : ''} onSelect={(profile) => void selectProfile(profile)}/>
    </header>
  {/if}
  <div class="workspace-body" class:has-activity-rail={activities.length > 1 && operationEnabled(ACTIVITY_SELECT_OPERATION)}>
    {#if activities.length > 1 && operationEnabled(ACTIVITY_SELECT_OPERATION)}
      <aside class="activity-rail">
        <ActivityNavigation activities={activities} activeActivityModeId={controller.state.kind === 'ready' || controller.state.kind === 'refreshing' || controller.state.kind === 'stale' ? controller.state.projection.activity_mode_id : ''} onSelect={(activity) => void selectActivity(activity)}/>
      </aside>
    {/if}
    <div class="canvas-stage">
      {#if controller.state.kind === 'ready'}
        <MissionCanvasRenderer projection={controller.state.projection} {registry} {client} onOperation={executeContributionOperation ? invokeContributionOperation : undefined} onSelectTab={operationEnabled(LAYOUT_MUTATE_OPERATION) ? (id) => void selectTab(id) : undefined}/>
      {:else if controller.state.kind === 'refreshing'}
        <div class="state-banner" role="status">Refreshing canonical workspace…</div>
        <MissionCanvasRenderer projection={controller.state.projection} {registry} {client} onOperation={executeContributionOperation ? invokeContributionOperation : undefined} onSelectTab={operationEnabled(LAYOUT_MUTATE_OPERATION) ? (id) => void selectTab(id) : undefined}/>
      {:else if controller.state.kind === 'stale'}
        <div class="state-banner" role="status">{controller.state.reason}</div>
        <MissionCanvasRenderer projection={controller.state.projection} {registry} {client} onOperation={executeContributionOperation ? invokeContributionOperation : undefined} onSelectTab={operationEnabled(LAYOUT_MUTATE_OPERATION) ? (id) => void selectTab(id) : undefined}/>
      {:else if controller.state.kind === 'loading'}
        <div class="state-message" role="status">Loading canonical workspace…</div>
      {:else if controller.state.kind === 'blocked' || controller.state.kind === 'error'}
        <div class="state-message error" role="alert">{controller.state.reason}</div>
      {/if}
    </div>
  </div>
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
  .desktop-canvas-runtime{container:mission-canvas / inline-size;position:relative;display:grid;grid-template-rows:minmax(0,1fr);min-width:0;min-height:0;height:100%}
  .desktop-canvas-runtime.has-controls{grid-template-rows:auto minmax(0,1fr)}
  .workspace-controls{display:flex;align-items:center;justify-content:flex-end;gap:var(--layout-control-gap);min-width:0;border-bottom:1px solid var(--color-border);background:var(--color-panel)}
  .workspace-controls :global(label){padding-inline:var(--space-3)}
  .workspace-body{display:grid;grid-template-columns:minmax(0,1fr);min-width:0;min-height:0;overflow:hidden}
  .workspace-body.has-activity-rail{grid-template-columns:minmax(132px,168px) minmax(0,1fr)}
  .activity-rail,.canvas-stage{position:relative;min-width:0;min-height:0;overflow:hidden}
  .canvas-stage{display:grid}
  @container mission-canvas (max-width:820px){.workspace-body.has-activity-rail{grid-template-columns:minmax(0,1fr);grid-template-rows:auto minmax(0,1fr)}}
  .mutation-pending .workspace-controls,.mutation-pending :global([role='tablist']){pointer-events:none;opacity:.72}
  .capability-notice,.state-banner{position:absolute;z-index:2;inset-block-start:var(--space-2);inset-inline:var(--space-2);padding:var(--space-2) var(--space-3);border:1px solid var(--color-warning);border-radius:var(--radius-control);background:var(--color-raised);color:var(--color-warning);font:var(--type-caption)}
  .capability-notice{inset-inline-start:auto;max-width:min(30rem,calc(100% - var(--space-4)));margin:0}
  .state-message{align-self:center;justify-self:center;padding:var(--layout-card-padding);color:var(--color-text-secondary)}
  .state-message.error{max-width:34rem;border:1px solid var(--color-error);border-radius:var(--radius-card);background:var(--color-raised);color:var(--color-error)}
</style>
