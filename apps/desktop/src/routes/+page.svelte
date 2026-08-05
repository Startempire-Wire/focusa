<script lang="ts">
  import { onMount } from 'svelte';
  import { FOCUSA_DESKTOP_WORKSPACES, workspaceById } from '$lib/shell/workspace-manifest';
  import { readDaemonHealth, type DaemonReadStatus } from '$lib/shell/daemon-health';
  import MissionCanvasShell from '$lib/shell/MissionCanvasShell.svelte';
  import AgentTuiSurface from '$lib/shell/AgentTuiSurface.svelte';
  import { readSidebarPreferences, saveSidebarPreferences, type DesktopSidebarMode } from '$lib/shell/sidebar-preferences';
  import Icon, { type IconName } from '$lib/ui/Icon.svelte';
  import IconButton from '$lib/ui/IconButton.svelte';
  import StatePanel from '$lib/ui/StatePanel.svelte';
  import ThinkingOrb from '$lib/ui/ThinkingOrb.svelte';
  import MotionControl from '$lib/ui/MotionControl.svelte';
  import { installMotionPreference, scene } from '$lib/ui/motion';

  const sidebarGroups: ReadonlyArray<{ id: string; label: string; workspaceIds: readonly string[] }> = [
    { id: 'orient', label: 'Orient', workspaceIds: ['mission-deck', 'mission-canvas'] },
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
  let daemonOrbState = $derived<'idle' | 'loading' | 'error'>(daemon.kind === 'checking' ? 'loading' : daemon.kind === 'unavailable' ? 'error' : 'idle');
  let sidebarMode = $state<DesktopSidebarMode>('expanded');
  let sidebarWidth = $state(248);
  let collapsedSidebarGroups = $state<string[]>([]);
  let sidebarResizeStart: { x: number; width: number } | null = null;

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
  function resizeSidebar(event: PointerEvent): void {
    if (!sidebarResizeStart) return;
    sidebarWidth = Math.min(320, Math.max(208, sidebarResizeStart.width + event.clientX - sidebarResizeStart.x));
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

  onMount(() => {
    shellMode = '__TAURI_INTERNALS__' in window ? 'native desktop' : 'browser preview';
    const stopMotionPreference = installMotionPreference();
    const preferences = readSidebarPreferences();
    sidebarMode = preferences.mode;
    sidebarWidth = preferences.widthPx;
    collapsedSidebarGroups = preferences.collapsedGroups;
    const onKeyDown = (event: KeyboardEvent) => {
      const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement || event.target instanceof HTMLSelectElement;
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

<div class="desktop-shell" class:tui-mode={uiMode === 'tui'} class:sidebar-compact={sidebarMode === 'compact'} style={`--sidebar-width:${sidebarWidth}px`}>
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
    <button class="scope-card" type="button" aria-label="Context Control: Unbound" title="Context Control · Unbound">
      <span class="scope-icon" aria-hidden="true"><Icon name="scope" size={18} /></span>
      <span class="scope-copy"><span class="eyebrow">Context Control</span><strong>Unbound</strong><small>No exact Attachment selected.</small></span>
    </button>
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
    {#if sidebarMode === 'expanded'}<button class="sidebar-resize-handle" type="button" aria-label="Resize sidebar" onpointerdown={beginSidebarResize}></button>{/if}
  </aside>
  {/if}

  <main>
    {#key `${uiMode}:${activeWorkspaceId}`}
    <div class="view-scene" in:scene={{ duration: 220, y: 5 }} out:scene={{ duration: 110, y: 2 }}>
    {#if uiMode === 'tui'}
      <AgentTuiSurface />
    {:else}
    <section class="workspace-heading">
      <div>
        <span class="eyebrow">Primary application · 5% native shell</span>
        <h1>{activeWorkspace.label}</h1>
        <p>{activeWorkspace.description}</p>
      </div>
      <span class:planned={activeWorkspace.availability === 'planned'} class="availability">
        {activeWorkspace.availability === 'shell' ? 'Shell available' : `Planned milestone ${activeWorkspace.milestone}%`}
      </span>
    </section>

    {#if activeWorkspace.id === 'mission-deck'}
      <section class="mission-grid" aria-label="Mission Deck shell">
        <article class="hero-card">
          <span class="eyebrow">Canonical authority</span>
          <h2>No Workstream attached</h2>
          <p>
            Desktop will not derive authority from the current tab, local directory, last project,
            latest record, or daemon-global selection.
          </p>
          <div class="identity-chain" aria-label="Required canonical identity chain">
            <span>ScopeRef</span><i><Icon name="chevron-right" size={14}/></i><span>WorkstreamId</span><i><Icon name="chevron-right" size={14}/></i><span>ContinuityId</span><i><Icon name="chevron-right" size={14}/></i><span>AttachmentKey</span>
          </div>
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
      </section>
    {:else if activeWorkspace.id === 'mission-canvas'}
      <MissionCanvasShell />
    {:else if activeWorkspace.id === 'agent-runtime'}
      <section class="empty-state">
        <span class="eyebrow">Infrastructure plane</span>
        <h2>{daemon.label}</h2>
        <p>{daemon.detail}</p>
        <button class="secondary-button" type="button" onclick={refreshDaemon} disabled={daemon.kind === 'checking'}>
          {daemon.kind === 'checking' ? 'Checking…' : 'Refresh health'}
        </button>
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
    <span>UIAI Engine browser proof</span>
    <span>No canonical state duplicated</span>
  </footer>
</div>
