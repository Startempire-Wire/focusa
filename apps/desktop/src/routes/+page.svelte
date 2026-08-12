<script lang="ts">
  import { onMount } from 'svelte';
  import { FOCUSA_DESKTOP_WORKSPACES, workspaceById } from '$lib/shell/workspace-manifest';
  import { readDaemonHealth, supportsMissionCanvasApi, type DaemonReadStatus } from '$lib/shell/daemon-health';
  import MissionCanvasShell from '$lib/shell/MissionCanvasShell.svelte';
  import AgentTuiSurface from '$lib/shell/AgentTuiSurface.svelte';
  import ContextControlPanel from '$lib/shell/ContextControlPanel.svelte';
  import { readSidebarPreferences, saveSidebarPreferences, type DesktopSidebarMode } from '$lib/shell/sidebar-preferences';
  import Icon, { type IconName } from '$lib/ui/Icon.svelte';
  import IconButton from '$lib/ui/IconButton.svelte';
  import StatePanel from '$lib/ui/StatePanel.svelte';
  import ThinkingOrb from '$lib/ui/ThinkingOrb.svelte';
  import MotionControl from '$lib/ui/MotionControl.svelte';
  import CommandPalette from '$lib/ui/CommandPalette.svelte';
  import { installMotionPreference, scene, setMotionPreference } from '$lib/ui/motion';
  import type { PresentationCommand } from '$lib/shell/command-manifest';
  import type { LiveCanvasBinding } from '$lib/mission-canvas/live-canvas-bridge';
  import { resolveLiveCanvasBinding } from '$lib/mission-canvas/live-canvas-bridge';
  import type { ResolvedWorkspaceProjection } from '$lib/mission-canvas/types';
  import { createMenubarBridge } from '$lib/shell/menubar-bridge';

  const sidebarGroups: ReadonlyArray<{ id: string; label: string; workspaceIds: readonly string[] }> = [
    { id: 'orient', label: 'Orient', workspaceIds: ['mission-canvas', 'mission-deck'] },
    { id: 'work', label: 'Work', workspaceIds: ['pi-work-surface', 'crist', 'context-role', 'workpoints', 'trajectory'] },
    { id: 'records', label: 'Records', workspaceIds: ['sessions', 'contention', 'evidence', 'documents', 'research'] },
    { id: 'system', label: 'System', workspaceIds: ['agent-runtime'] }
  ];
  const workspaceIcons: Record<string, IconName> = {
    'mission-deck': 'deck', 'mission-canvas': 'canvas', 'pi-work-surface': 'terminal', crist: 'crist',
    'context-role': 'context', workpoints: 'target', trajectory: 'route', sessions: 'sessions',
    contention: 'approvals', evidence: 'evidence', documents: 'documents', research: 'research', 'agent-runtime': 'runtime'
  };

  let activeWorkspaceId = $state('mission-deck');
  let uiMode = $state<'tui' | 'canvas'>('canvas');
  let shellMode = $state<'browser preview' | 'native desktop'>('browser preview');
  let daemon = $state<DaemonReadStatus>({
    kind: 'checking',
    label: 'Checking daemon',
    detail: 'Reading infrastructure health only.'
  });
  let activeWorkspace = $derived(workspaceById(activeWorkspaceId));
  let developmentProjection = $state<ResolvedWorkspaceProjection>();
  let liveBinding = $state<LiveCanvasBinding>();
  let liveBindingState = $state<'idle' | 'checking' | 'live' | 'fixture'>('idle');
  let daemonOrbState = $derived<'idle' | 'loading' | 'error'>(daemon.kind === 'checking' ? 'loading' : daemon.kind === 'unavailable' ? 'error' : 'idle');
  let missionCanvasApiAvailable = $derived(supportsMissionCanvasApi('version' in daemon ? daemon.version : undefined));
  let sidebarMode = $state<DesktopSidebarMode>('expanded');
  let sidebarWidth = $state(248);
  let collapsedSidebarGroups = $state<string[]>([]);
  let commandOpen = $state(false);
  let workNotes = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('focusa-desktop-notes') ?? '') : '');
  let contextOpen = $state(false);
  let sidebarResizeStart: { x: number; width: number } | null = null;
  const menubar = createMenubarBridge();

  // Push workspace binding to menubar when live binding changes
  $effect(() => {
    const lb = liveBinding;
    if (lb && menubar.available) {
      menubar.send({
        kind: 'workspace_bound',
        workstreamId: lb.authority.workstream.workstream_id,
        continuityId: lb.authority.continuity_id ?? '',
        profileId: lb.projection.workspace_profile_id,
        activityId: lb.projection.activity_mode_id
      });
    } else if (menubar.available) {
      menubar.send({ kind: 'workspace_unbound' });
    }
  });

  // Push daemon health to menubar
  $effect(() => {
    if (daemon.version && menubar.available) {
      menubar.send({ kind: 'health_ping', daemonVersion: daemon.version, uptimeMs: daemon.uptimeMs });
    }
  });

  async function refreshDaemon(): Promise<void> {
    daemon = { kind: 'checking', label: 'Checking daemon', detail: 'Reading infrastructure health only.' };
    daemon = await readDaemonHealth();
  }

  function persistSidebar(): void {
    saveSidebarPreferences({ schema: 'focusa.desktop.sidebar_preferences.v1', mode: sidebarMode, widthPx: sidebarWidth, collapsedGroups: collapsedSidebarGroups });
  }

  function setSidebarMode(mode: DesktopSidebarMode): void { sidebarMode = mode; persistSidebar(); }
  function toggleSidebarGroup(group: string): void {
    collapsedSidebarGroups = collapsedSidebarGroups.includes(group) ? collapsedSidebarGroups.filter((item) => item !== group) : [...collapsedSidebarGroups, group];
    persistSidebar();
  }
  function executePresentationCommand(command: PresentationCommand): void {
    const action = command.action;
    if (action.kind === 'navigate-workspace') { activeWorkspaceId = action.workspaceId; uiMode = 'canvas'; }
    if (action.kind === 'set-interface') uiMode = action.interfaceMode;
    if (action.kind === 'set-sidebar') setSidebarMode(action.sidebarMode);
    if (action.kind === 'set-motion') setMotionPreference(action.motionMode);
  }

  function resizeSidebar(event: PointerEvent): void {
    if (!sidebarResizeStart) return;
    sidebarWidth = Math.min(320, Math.max(208, sidebarResizeStart.width + sidebarResizeStart.x - event.clientX));
  }
  function endSidebarResize(): void {
    sidebarResizeStart = null;
    persistSidebar();
    window.removeEventListener('pointermove', resizeSidebar);
  }
  function beginSidebarResize(event: PointerEvent): void {
    if (sidebarMode !== 'expanded') return;
    event.preventDefault();
    sidebarResizeStart = { x: event.clientX, width: sidebarWidth };
    window.addEventListener('pointermove', resizeSidebar);
    window.addEventListener('pointerup', endSidebarResize, { once: true });
  }

  $effect(() => {
    const workspaceId = activeWorkspace.id;
    const previewHost = shellMode === 'browser preview';
    let cancelled = false;
    developmentProjection = undefined;
    liveBinding = undefined;
    liveBindingState = 'idle';
    if (import.meta.env.DEV && previewHost) {
      void import('$lib/mission-canvas/development-preview').then((module) => {
        if (!cancelled) developmentProjection = module.developmentProjection(workspaceId);
      });
    }
    if (previewHost && missionCanvasApiAvailable && workspaceId === 'mission-canvas') {
      liveBindingState = 'checking';
      void resolveLiveCanvasBinding().then((result) => {
        if (cancelled) return;
        if (result.kind === 'live' && result.binding) {
          liveBinding = result.binding;
          liveBindingState = 'live';
        } else {
          liveBinding = undefined;
          liveBindingState = 'fixture';
        }
      });
    }
    return () => { cancelled = true; };
  });

  onMount(() => {
    shellMode = '__TAURI_INTERNALS__' in window ? 'native desktop' : 'browser preview';
    const stopMotionPreference = installMotionPreference();
    const preferences = readSidebarPreferences();
    sidebarMode = preferences.mode;
    sidebarWidth = preferences.widthPx;
    // Clutter guard: unless the operator persisted an explicit collapse set,
    // default to collapsing every group except the active workspace's group.
    collapsedSidebarGroups = preferences.collapsedGroups.length > 0
      ? preferences.collapsedGroups
      : sidebarGroups.filter((group) => !group.workspaceIds.includes(activeWorkspaceId)).map((group) => group.id);
    const onKeyDown = (event: KeyboardEvent) => {
      const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement || event.target instanceof HTMLSelectElement;
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') { event.preventDefault(); commandOpen = true; }
      if (event.key === 'Escape') contextOpen = false;
      if (!typing && event.key === '[' && !event.metaKey && !event.ctrlKey) {
        event.preventDefault();
        setSidebarMode(sidebarMode === 'expanded' ? 'compact' : 'expanded');
      }
    };
    window.addEventListener('keydown', onKeyDown);
    void refreshDaemon();
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('pointermove', resizeSidebar);
      stopMotionPreference();
    };
  });
</script>

<svelte:head>
  <title>Focusa Desktop</title>
</svelte:head>

<div class="desktop-shell" class:tui-mode={uiMode === 'tui'} class:canvas-mode={uiMode === 'canvas'} class:sidebar-compact={sidebarMode === 'compact'} style={`--sidebar-width:${sidebarWidth}px`}>
  <header class="titlebar">
    <div class="brand-lockup" aria-label="Focusa Desktop">
      <span class="brand-mark" aria-hidden="true">F</span>
      <div>
        <strong>Focusa Desktop</strong>
        <span>Workstream workspace</span>
      </div>
    </div>
    <div class="titlebar-status">
      <div class="ui-mode-switch" role="group" aria-label="Switch between Mission Canvas and Agent TUI">
        <i class:tui={uiMode === 'tui'} aria-hidden="true"></i>
        <button
          type="button"
          class:active={uiMode === 'canvas'}
          aria-pressed={uiMode === 'canvas'}
          onclick={() => (uiMode = 'canvas')}
        >Mission Canvas</button>
        <button
          type="button"
          class:active={uiMode === 'tui'}
          aria-pressed={uiMode === 'tui'}
          onclick={() => (uiMode = 'tui')}
        >Agent TUI</button>
      </div>
      <span class="mode-chip">{shellMode}</span>
      <span class:unavailable={daemon.kind === 'unavailable'} class:connected={daemon.kind === 'read-only'} class="daemon-chip">
        <span class="status-dot" aria-hidden="true"></span>{daemon.label}
      </span>
    </div>
  </header>

  {#if uiMode === 'canvas'}
  <aside class="sidebar" class:compact={sidebarMode === 'compact'} aria-label="Focusa workspaces">
    <div class="sidebar-heading">
      <span>Workspaces</span>
      <IconButton
        icon="panel-left"
        label={`${sidebarMode === 'compact' ? 'Expand' : 'Collapse'} sidebar ([)`}
        pressed={sidebarMode === 'compact'}
        onclick={() => setSidebarMode(sidebarMode === 'compact' ? 'expanded' : 'compact')}
      />
    </div>
    <button class="scope-card" type="button" aria-label="Context Control" aria-expanded={contextOpen} title="Context Control" onclick={() => (contextOpen = !contextOpen)}>
      <span class="scope-icon" aria-hidden="true"><Icon name="scope" size={18} /></span>
      <span class="scope-copy"><span class="eyebrow">Context Control</span><strong>{liveBinding ? 'Bound' : 'Unbound'}</strong><small>{liveBinding?.authority?.attachment?.attachment_id ?? 'No exact Attachment selected.'}</small></span>
    </button>
    <ContextControlPanel bind:open={contextOpen} {daemon} authority={liveBinding?.authority} onResolve={() => (activeWorkspaceId = 'mission-canvas')}/>
    <button class="find-button" type="button" aria-label="Find or do" onclick={() => (commandOpen = true)}><Icon name="search" size={16}/><span>Find or do</span><kbd>⌘K</kbd></button>
    <div class="workspace-groups">
      {#each sidebarGroups as group}
        <section class="workspace-group" aria-label={`${group.label} workspaces`}>
          <button class="group-heading" type="button" aria-expanded={!collapsedSidebarGroups.includes(group.id)} onclick={() => toggleSidebarGroup(group.id)}>
            <span>{group.label}</span><Icon name={collapsedSidebarGroups.includes(group.id) ? 'chevron-right' : 'chevron-down'} size={14} />
          </button>
          {#if sidebarMode === 'compact' || !collapsedSidebarGroups.includes(group.id)}
            <nav aria-label={`${group.label} workspaces`}>
              {#each FOCUSA_DESKTOP_WORKSPACES.filter((workspace) => group.workspaceIds.includes(workspace.id)) as workspace}
                <button
                  type="button"
                  class:active={workspace.id === activeWorkspaceId}
                  aria-label={workspace.label}
                  title={workspace.label}
                  aria-current={workspace.id === activeWorkspaceId ? 'page' : undefined}
                  onclick={() => (activeWorkspaceId = workspace.id)}
                >
                  <span class="workspace-icon" aria-hidden="true"><Icon name={workspaceIcons[workspace.id] ?? 'sparkles'} /></span>
                  <span class="workspace-label">{workspace.shortLabel}</span>
                  {#if workspace.availability === 'planned'}<small>M{workspace.milestone}</small>{:else}<small>live</small>{/if}
                </button>
              {/each}
            </nav>
          {/if}
        </section>
      {/each}
    </div>
    {#if sidebarMode === 'expanded'}<MotionControl />{/if}
    {#if menubar.available}
      <div class="menubar-status" aria-label="Menubar active">
        <span class="dot"></span>Menubar
      </div>
    {/if}
    {#if sidebarMode === 'expanded'}<button class="sidebar-resize-handle" type="button" aria-label="Resize sidebar" onpointerdown={beginSidebarResize}></button>{/if}
  </aside>
  {/if}

  <main>
    {#key `${uiMode}:${activeWorkspaceId}`}
    <div class="view-scene" in:scene={{ duration: 220, y: 5 }} out:scene={{ duration: 110, y: 2 }}>
    {#if uiMode === 'tui'}
      <AgentTuiSurface />
    {:else if activeWorkspace.id === 'mission-canvas' || developmentProjection}
      {#if liveBinding}
        <MissionCanvasShell authority={liveBinding.authority} client={liveBinding.client} />
      {:else}
        {#if developmentProjection}
          <div class="development-fixture-label" role="status">Schema fixture · noncanonical development preview</div>
          {#if liveBindingState === 'checking'}
            <p class="infra-gap-notice" role="note">Connecting to the local daemon for a live canonical projection…</p>
          {:else if daemon.kind === 'read-only' && !missionCanvasApiAvailable}
            <p class="infra-gap-notice" role="note">
              Mission Canvas API requires Focusa daemon 0.9.143+ (running {daemon.version}). The canvas stays
              read-only until the daemon is upgraded; no authority is invented.
            </p>
          {/if}
        {/if}
        <MissionCanvasShell projection={developmentProjection} />
      {/if}
    {:else}
      <section class="workspace-heading">
        <div>
          <span class="eyebrow">Focusa Desktop workspace</span>
          <h1>{activeWorkspace.label}</h1>
          <p>{activeWorkspace.description}</p>
        </div>
        <span class:planned={activeWorkspace.availability === 'planned'} class="availability">
          {activeWorkspace.availability === 'shell' ? 'Available' : `Planned milestone ${activeWorkspace.milestone}%`}
        </span>
      </section>

    {#if activeWorkspace.id === 'mission-deck'}
      <section class="mission-grid" aria-label="Mission Deck shell">
        <article class="hero-card">
          <span class="eyebrow">Canonical authority</span>
          {#if liveBinding}
            <h2>Workstream bound</h2>
            <p>
              {liveBinding.projection.workspace_profile_id} · {liveBinding.projection.activity_mode_id} · revision {liveBinding.projection.projection_revision}
            </p>
            <div class="identity-chain" aria-label="Resolved canonical identity chain">
              <span class="resolved">{liveBinding.authority.workstream.scope.scope_key.scope_id ?? 'Scope'}</span><i><Icon name="chevron-right" size={14}/></i><span class="resolved">{liveBinding.authority.workstream.workstream_id}</span><i><Icon name="chevron-right" size={14}/></i><span class="resolved">{liveBinding.authority.continuity_id ?? '—'}</span><i><Icon name="chevron-right" size={14}/></i><span class="resolved">{liveBinding.authority.attachment?.attachment_id ?? '—'}</span>
            </div>
            <button class="secondary-button" type="button" onclick={() => (activeWorkspaceId = 'mission-canvas')}>Open Mission Canvas</button>
          {:else}
            <h2>No Workstream attached</h2>
            <p>
              Desktop will not derive authority from the current tab, local directory, last project,
              latest record, or daemon-global selection.
            </p>
            <div class="identity-chain" aria-label="Required canonical identity chain">
              <span>ScopeRef</span><i><Icon name="chevron-right" size={14}/></i><span>WorkstreamId</span><i><Icon name="chevron-right" size={14}/></i><span>ContinuityId</span><i><Icon name="chevron-right" size={14}/></i><span>AttachmentKey</span>
            </div>
          {/if}
        </article>
        <article class="status-card">
          <div class="card-heading">
            <div>
              <span class="eyebrow">Infrastructure</span>
              <h2>{daemon.label}</h2>
            </div>
            <ThinkingOrb state={daemonOrbState} size={28} label="Daemon infrastructure" />
          </div>
          <p>{daemon.detail}</p>
          {#if daemon.kind === 'read-only' && daemon.version}
            <div class="daemon-stats" aria-label="Daemon infrastructure stats">
              <span>v{daemon.version}</span>
              {#if daemon.uptimeMs !== undefined}<span>{Math.round(daemon.uptimeMs / 1000 / 60 / 60)}h uptime</span>{/if}
              {#if daemon.snapshotMb !== undefined}<span>{daemon.snapshotMb} MB store</span>{/if}
              {#if daemon.failuresTotal !== undefined && daemon.failuresTotal === 0}<span class="clean">0 failures</span>{/if}
            </div>
          {/if}
          <button class="secondary-button" type="button" onclick={refreshDaemon} disabled={daemon.kind === 'checking'}>
            {daemon.kind === 'checking' ? 'Checking…' : 'Refresh health'}
          </button>
        </article>
        <article class="principle-card">
          <span class="eyebrow">Runtime boundary</span>
          <h2>Presentation without duplication</h2>
          <ul>
            <li><Icon name="check" size={14}/>Infrastructure health only</li>
            <li><Icon name="check" size={14}/>No canonical cognitive cache</li>
            <li><Icon name="check" size={14}/>No implicit active Workstream</li>
            <li><Icon name="check" size={14}/>No domain mutation</li>
          </ul>
        </article>
        <article class="notes-card">
          <span class="eyebrow">Work notes</span>
          <h2>Quick capture</h2>
          <p>Local scratchpad — never synced, never canonical.</p>
          <textarea class="notes-area" placeholder="Jot down a thought, task, or reminder…" aria-label="Work notes scratchpad" rows={4} bind:value={workNotes} oninput={() => { try { localStorage.setItem('focusa-desktop-notes', workNotes); } catch { /* quota */ } }}></textarea>
          <small>Stored in localStorage. Cleared when you clear browser data.</small>
        </article>
      </section>
    {:else if activeWorkspace.id === 'agent-runtime'}
      <section class="empty-state">
        <span class="eyebrow">Infrastructure plane</span>
        <h2>{daemon.label}</h2>
        <p>{daemon.detail}</p>
        <button class="secondary-button" type="button" onclick={refreshDaemon} disabled={daemon.kind === 'checking'}>
          {daemon.kind === 'checking' ? 'Checking…' : 'Refresh health'}
        </button>
      </section>
    {:else if activeWorkspace.id === 'sessions'}
      <section class="empty-state" aria-label="Sessions workspace">
        <span class="eyebrow">Canonical Sessions</span>
        <h2>Workstream Attachments</h2>
        {#if liveBinding}
          <p>{liveBinding.authority.attachment?.attachment_id ?? 'No attachment'} · {liveBinding.authority.continuity_id ?? 'No continuity'}</p>
          <div class="identity-chain" aria-label="Attachment details">
            <span class="resolved">{liveBinding.authority.attachment?.kind ?? 'runtime'}</span><i>/</i>
            <span class="resolved">{liveBinding.authority.attachment?.instance_id ?? 'no instance'}</span><i>/</i>
            <span class="resolved">{liveBinding.authority.attachment?.session_id ?? 'no session'}</span>
          </div>
          <p class="field-note">Full session inventory requires the <code>focusa.mission_canvas.session.inventory</code> daemon endpoint.</p>
        {:else}
          <p>Open Mission Canvas to bind a Workstream first.</p>
        {/if}
      </section>
    {:else if activeWorkspace.id === 'context-role' || activeWorkspace.id === 'workpoints' || activeWorkspace.id === 'trajectory' || activeWorkspace.id === 'evidence'}
      <section class="empty-state" aria-label={`${activeWorkspace.label} workspace`}>
        <span class="eyebrow">{activeWorkspace.label} workspace</span>
        <h2>{activeWorkspace.description}</h2>
        {#if liveBinding}
          <div class="identity-chain" aria-label="Current binding">
            <span class="resolved">{liveBinding.authority.workstream.workstream_id}</span><i>·</i>
            <span>r{liveBinding.projection.projection_revision}</span><i>·</i>
            <span>{liveBinding.projection.workspace_profile_id}/{liveBinding.projection.activity_mode_id}</span>
          </div>
          {#if activeWorkspace.id === 'context-role'}
            <p>The current role is derived from the Workstream profile and activity mode. Full C.R.I.S.T. Context/Role/Interview/Spec/Tasks projection requires the <code>focusa.mission_canvas.generated_surface.resolve</code> daemon endpoint.</p>
          {:else if activeWorkspace.id === 'workpoints'}
            <p>Workpoint state and verification posture require the <code>focusa.mission_canvas.workpoint.list</code> endpoint. The Work Rail contribution already renders live workpoint data from the projection.</p>
          {:else if activeWorkspace.id === 'trajectory'}
            <p>Tactical Workstream Trajectory (HLT/MLG/STG/waypoints) requires the <code>focusa.mission_canvas.trajectory.view</code> endpoint. Open Mission Canvas for live trajectory contributions.</p>
          {:else if activeWorkspace.id === 'evidence'}
            <p>Evidence, Receipts, and closure proof require the <code>focusa.mission_canvas.recomposition.evidence.get</code> endpoint. Evidence refs from the projection: {liveBinding.projection.evidence_refs.length ?? 0} available.</p>
          {/if}
        {:else}
          <p>Open Mission Canvas to bind a Workstream first.</p>
        {/if}
      </section>
    {:else}
      <StatePanel
        state="blocked"
        title={`${activeWorkspace.label} is not implemented at 5%`}
        description="Navigation is active, but this workspace remains unavailable until its typed Workstream contracts and parity Evidence are complete."
      />
    {/if}
    {/if}
    </div>
    {/key}
  </main>

  <footer>
    <span>Focusa Desktop 0.9.143</span>
    {#if liveBinding}
      <span>{liveBinding.authority.workstream.scope.scope_key.canonical_name ?? 'Focusa'} · {liveBinding.authority.workstream.workstream_id}</span>
      <span>r{liveBinding.projection.projection_revision} · {liveBinding.projection.workspace_profile_id}/{liveBinding.projection.activity_mode_id}</span>
    {:else}
      <span>{daemon.kind === 'read-only' ? 'Daemon connected' : daemon.label}</span>
      <span>No canonical state duplicated</span>
    {/if}
  </footer>
</div>
<CommandPalette bind:open={commandOpen} onSelect={executePresentationCommand}/>
