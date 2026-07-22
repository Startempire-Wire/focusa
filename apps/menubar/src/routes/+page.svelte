<script lang="ts">
  import { fetchJson, focusaPost, hasEverConnected } from '$lib/api';
  import { getProjectContext } from '$lib/projectContext.svelte';
  import { workLoopScopedPaths } from '$lib/workLoopScope.js';
  import { focusStore } from '$lib/stores/focus.svelte';
  import { gateStore } from '$lib/stores/gate.svelte';
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import FocusView from '$lib/components/FocusView.svelte';
  import GatePanel from '$lib/components/GatePanel.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import FirstRunWizard from '$lib/components/FirstRunWizard.svelte';
  import MissionCanvasView from '$lib/components/MissionCanvasView.svelte';
  import TrajectoryPeek from '$lib/components/TrajectoryPeek.svelte';
  import WorkpointPeek from '$lib/components/WorkpointPeek.svelte';
  import ProofPeek from '$lib/components/ProofPeek.svelte';
  import WorkLoopPeek from '$lib/components/WorkLoopPeek.svelte';
  import { onMount } from 'svelte';

  import SyncPanel from '$lib/components/SyncPanel.svelte';
  import PairingPanel from '$lib/components/PairingPanel.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';

  type Tab = 'focus' | 'mission-canvas' | 'trajectory' | 'workpoint' | 'proof' | 'workloop' | 'gate' | 'sync' | 'pair' | 'settings';
  let activeTab = $state<Tab>('focus');
  let everConnected = $state(false);

  let pollTimer: ReturnType<typeof setInterval> | undefined;

  async function safe<T>(load: () => Promise<T>): Promise<T | null> {
    try {
      return await load();
    } catch {
      return null;
    }
  }

  async function poll() {
    try {
      const state = await fetchJson('/v1/state/dump', 5000);
      everConnected = true;
      const projectIdentityRaw = await safe(() => fetchJson('/v1/project/identity'));
      const activeWorkpointId = state?.workpoint?.active_workpoint_id;
      const activeWorkpointRecord = state?.workpoint?.active ?? state?.workpoint?.records?.find?.((record: any) => record?.workpoint_id === activeWorkpointId) ?? null;
      const projectIdentityRecord = projectIdentityRaw?.project_identity ?? projectIdentityRaw ?? {};
      // Spec104 MEN-02: derive typed scope from one source of truth (projectContext.svelte.ts).
      const typedScope = getProjectContext({
        projectIdentity: projectIdentityRecord,
        workpointResume: state?.workpointResume,
        workpoint: activeWorkpointRecord ?? state?.workpoint,
      });
      const projectRoot = typedScope.projectRoot || null;
      const continuityId = typedScope.continuityId || null;
      const projectIdentity = {
        ...projectIdentityRecord,
        project_root: projectRoot,
        continuity_id: continuityId,
        raw: projectIdentityRaw,
      };
      const scopedParams = new URLSearchParams();
      if (projectRoot) scopedParams.set('project_root', projectRoot);
      if (continuityId) scopedParams.set('continuity_id', continuityId);
      const scopedQuery = scopedParams.toString();
      const scopedSuffix = scopedQuery ? `&${scopedQuery}` : '';
      const scopedPathSuffix = scopedQuery ? `?${scopedQuery}` : '';
      const workLoopPaths = workLoopScopedPaths(projectRoot, continuityId);
      const [health, doctor, contracts, focusFrame, trajectory, workpoint, workpointResume, workLoop, workLoopHealth, workLoopCheckpoints, memoryTelemetry, events, tokenBudget, cacheMetadata, predictionsRecent, predictionsStats, metacogStatus, metacogEvaluations, snapshotsRecent, lineageHead, releaseProof, updateNotifications] = await Promise.all([
        safe(() => fetchJson('/v1/health')),
        safe(() => fetchJson('/v1/doctor', 5000)),
        safe(() => fetchJson('/v1/ontology/tool-contracts')),
        safe(() => scopedQuery ? fetchJson(`/v1/focus/frame/current?${scopedQuery}`) : Promise.resolve(null)),
        safe(() => fetchJson(`/v1/trajectory/view?mode=summary${scopedSuffix}`)),
        safe(() => fetchJson(`/v1/workpoint/current${scopedPathSuffix}`)),
        safe(() => focusaPost('/v1/workpoint/resume', scopedQuery ? { project_root: projectRoot, continuity_id: continuityId } : {}, { projectRoot: projectRoot || undefined, continuityId: continuityId || undefined }, 5000)),
        safe(() => workLoopPaths ? fetchJson(workLoopPaths.status) : Promise.resolve(null)),
        safe(() => workLoopPaths ? fetchJson(workLoopPaths.health) : Promise.resolve(null)),
        safe(() => workLoopPaths ? fetchJson(workLoopPaths.checkpoints) : Promise.resolve(null)),
        safe(() => fetchJson('/v1/telemetry/memory')),
        safe(() => fetchJson('/v1/events/recent?limit=5')),
        safe(() => fetchJson('/v1/telemetry/token-budget/status?limit=5')),
        safe(() => fetchJson('/v1/telemetry/cache-metadata/status?limit=5')),
        safe(() => fetchJson('/v1/predictions/recent?limit=5')),
        safe(() => fetchJson('/v1/predictions/stats')),
        safe(() => fetchJson('/v1/metacognition/status')),
        safe(() => fetchJson('/v1/metacognition/evaluations/recent?limit=5')),
        safe(() => fetchJson('/v1/focus/snapshots/recent?limit=5')),
        safe(() => fetchJson('/v1/lineage/head')),
        safe(() => fetchJson('/v1/release/proof/status')),
        safe(() => fetchJson('/v1/update/notifications')),
      ]);
      const workpointPacket = workpointResume?.resume_packet ?? workpointResume?.packet ?? null;
      const normalizedWorkpointResume = workpointPacket
        ? { ...workpointResume, ...workpointPacket, resume_packet: workpointPacket }
        : workpointResume;
      const trajectoryRecord = trajectory?.trajectory ?? trajectory ?? {};
      const trajectoryLadder = trajectoryRecord?.trajectory_ladder ?? {};
      const normalizedTrajectory = {
        ...trajectory,
        ...trajectoryRecord,
        hlt: trajectory?.hlt ?? trajectoryRecord?.hlt ?? trajectoryRecord?.long_term_goal ?? trajectoryLadder?.hlt,
        mlg: trajectory?.mlg ?? trajectoryRecord?.mlg ?? trajectoryRecord?.mid_level_goal ?? trajectoryLadder?.mlg,
        stg: trajectory?.stg ?? trajectoryRecord?.stg ?? trajectoryRecord?.short_term_goal ?? trajectoryLadder?.stg,
        continuity_id: trajectory?.continuity_id ?? continuityId,
        project_identity: trajectory?.project_identity ?? { project_root: projectRoot },
      };
      const normalizedSession = {
        ...state?.session,
        project_root: projectRoot,
        continuity_id: continuityId,
      };
      focusStore.update(state);
      gateStore.update(state.focus_gate);
      runtimeStore.update({
        health,
        doctor,
        projectIdentity,
        focusFrame,
        trajectory: normalizedTrajectory,
        workpoint,
        workpointResume: normalizedWorkpointResume,
        session: normalizedSession,
        workLoop,
        workLoopHealth,
        workLoopCheckpoints,
        memoryTelemetry,
        ontologyContractsVersion: contracts?.version ?? null,
        ontologyContractsCount: Array.isArray(contracts?.contracts) ? contracts.contracts.length : 0,
        recentEventCount: Array.isArray(events?.events) ? events.events.length : 0,
        tokenBudget,
        cacheMetadata,
        predictionsRecent,
        predictionsStats,
        metacogStatus,
        metacogEvaluations,
        snapshotsRecent,
        lineageHead,
        releaseProof: releaseProof ?? {
          status: 'manual_proof_required',
          summary: 'Release-proof endpoint unavailable; run focusa release prove --tag <tag> before publish.',
        },
        updateNotifications,
      });
    } catch (e: any) {
      const msg = e?.message || 'Failed to connect';
      focusStore.setError(msg);
      runtimeStore.setError(msg);
    }
  }

  onMount(() => {
    everConnected = hasEverConnected();
    const onSaved = () => {
      everConnected = true;
      focusStore.setConnecting();
      void poll();
      if (!pollTimer) pollTimer = setInterval(poll, 2000);
    };
    window.addEventListener('focusa-connection-saved', onSaved);
    if (everConnected) {
      focusStore.setConnecting();
      poll(); // immediate first poll only after a saved/successful connection exists
      pollTimer = setInterval(poll, 2000);
    }
    return () => {
      window.removeEventListener('focusa-connection-saved', onSaved);
      if (pollTimer) clearInterval(pollTimer);
    };
  });
</script>

<!-- Header bar -->
<header class="header">
  <div class="header-left">
    <div class="status-dot" class:connected={focusStore.connected === 'connected'} class:error={focusStore.connected === 'error'}></div>
    <span class="header-title">Focusa</span>
    {#if (runtimeStore.snapshot.updateNotifications?.stale_parts?.length ?? 0) > 0}
      <span class="badge" title={runtimeStore.snapshot.updateNotifications?.message ?? 'Focusa update available'}>
        Update {runtimeStore.snapshot.updateNotifications.stale_parts.length}
      </span>
    {/if}
  </div>
  {#if everConnected || focusStore.connected === 'connected'}
  <nav class="tabs" aria-label="Focusa peeks">
    <button class="tab primary" class:active={activeTab === 'focus'} aria-pressed={activeTab === 'focus'} title="Focus bubble" onclick={() => activeTab = 'focus'}>
      <span class="tab-mark">◌</span><span>Focus</span>
    </button>
    <button class="tab primary" class:active={activeTab === 'mission-canvas'} aria-pressed={activeTab === 'mission-canvas'} title="Mission Canvas" onclick={() => activeTab = 'mission-canvas'}>
      <span class="tab-mark">◇</span><span>Now</span>
    </button>
    <button class="tab" class:active={activeTab === 'trajectory'} aria-pressed={activeTab === 'trajectory'} title="Trajectory" onclick={() => activeTab = 'trajectory'}>
      <span class="tab-mark">↗</span><span>Path</span>
    </button>
    <button class="tab" class:active={activeTab === 'workpoint'} aria-pressed={activeTab === 'workpoint'} title="Workpoint" onclick={() => activeTab = 'workpoint'}>
      <span class="tab-mark">□</span><span>WP</span>
    </button>
    <button class="tab" class:active={activeTab === 'proof'} aria-pressed={activeTab === 'proof'} title="Proof" onclick={() => activeTab = 'proof'}>
      <span class="tab-mark">✓</span><span>Proof</span>
    </button>
    <button class="tab" class:active={activeTab === 'workloop'} aria-pressed={activeTab === 'workloop'} title="Work Loop" onclick={() => activeTab = 'workloop'}>
      <span class="tab-mark">∞</span><span>Loop</span>
    </button>
    <button class="tab quiet" class:active={activeTab === 'gate'} aria-pressed={activeTab === 'gate'} title="Focus Gate" onclick={() => activeTab = 'gate'}>
      <span class="tab-mark">⌁</span><span>Gate</span>
      {#if gateStore.surfacedCount > 0}
        <span class="badge">{gateStore.surfacedCount}</span>
      {/if}
    </button>
    <button class="tab quiet" class:active={activeTab === 'sync'} aria-pressed={activeTab === 'sync'} title="Sync" onclick={() => activeTab = 'sync'}>
      <span class="tab-mark">⇄</span><span>Sync</span>
    </button>
    <button class="tab quiet" class:active={activeTab === 'pair'} aria-pressed={activeTab === 'pair'} title="Device Pairing" onclick={() => activeTab = 'pair'}>
      <span class="tab-mark">⎘</span><span>Pair</span>
    </button>
    <button class="tab icon-only quiet" class:active={activeTab === 'settings'} aria-pressed={activeTab === 'settings'} title="Settings" aria-label="Settings" onclick={() => activeTab = 'settings'}>
      ⚙
    </button>
  </nav>
  {/if}
</header>

<!-- Content -->
<main class="content">
  {#if !everConnected && focusStore.connected !== 'connected'}
    <FirstRunWizard />
  {:else if activeTab === 'focus'}
    <FocusView />
  {:else if activeTab === 'mission-canvas'}
    <MissionCanvasView />
  {:else if activeTab === 'trajectory'}
    <TrajectoryPeek />
  {:else if activeTab === 'workpoint'}
    <WorkpointPeek />
  {:else if activeTab === 'proof'}
    <ProofPeek />
  {:else if activeTab === 'workloop'}
    <WorkLoopPeek />
  {:else if activeTab === 'gate'}
    <GatePanel />
  {:else if activeTab === 'sync'}
    <SyncPanel />
  {:else if activeTab === 'pair'}
    <PairingPanel host="operator-host" />
  {:else if activeTab === 'settings'}
    <Settings />
  {/if}
</main>

<!-- Global toast notifications for action buttons -->
<ToastContainer />

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    background: var(--bg-panel);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fg-tertiary);
    transition: background var(--dur-normal) var(--ease);
    flex-shrink: 0;
  }

  .status-dot.connected { background: var(--green); }
  .status-dot.error { background: var(--red); }

  .header-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--fg-secondary);
    letter-spacing: 0.3px;
  }

  .tabs {
    display: flex;
    align-items: center;
    gap: 3px;
    max-width: 245px;
    overflow-x: auto;
    scrollbar-width: none;
    padding: 2px;
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    border-radius: var(--r-full);
    background: color-mix(in srgb, var(--bg-elevated) 65%, transparent);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
  }
  .tabs::-webkit-scrollbar { display: none; }

  .tab {
    font-family: var(--font);
    font-size: 10px;
    font-weight: 600;
    color: var(--fg-tertiary);
    background: transparent;
    border: 1px solid transparent;
    padding: 4px 7px;
    border-radius: var(--r-full);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
    flex: 0 0 auto;
  }

  .tab-mark {
    color: var(--fg-tertiary);
    font-size: 10px;
    line-height: 1;
  }

  .tab.quiet:not(.active) span:not(.badge) {
    opacity: 0.82;
  }

  .tab.icon-only {
    width: 24px;
    justify-content: center;
    padding-inline: 0;
  }

  .tab:hover {
    color: var(--fg-secondary);
    background: var(--bg-hover);
    transform: translateY(-1px);
  }

  .tab.active {
    color: var(--fg);
    background: color-mix(in srgb, var(--accent) 16%, var(--bg-panel));
    border-color: color-mix(in srgb, var(--accent) 35%, var(--border));
  }

  .tab.active .tab-mark {
    color: var(--accent);
  }

  .badge {
    font-size: 9px;
    font-weight: 700;
    min-width: 15px;
    height: 15px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent);
    color: white;
    border-radius: var(--r-full);
    padding: 0 4px;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

</style>
