<script lang="ts">
  import { onMount } from 'svelte';
  import { FOCUSA_DESKTOP_WORKSPACES, workspaceById } from '$lib/shell/workspace-manifest';
  import { readDaemonHealth, type DaemonReadStatus } from '$lib/shell/daemon-health';

  let activeWorkspaceId = $state('mission-deck');
  let uiMode = $state<'tui' | 'canvas'>('canvas');
  let shellMode = $state<'browser preview' | 'native desktop'>('browser preview');
  let daemon = $state<DaemonReadStatus>({
    kind: 'checking',
    label: 'Checking daemon',
    detail: 'Reading infrastructure health only.'
  });
  let activeWorkspace = $derived(workspaceById(activeWorkspaceId));

  async function refreshDaemon(): Promise<void> {
    daemon = { kind: 'checking', label: 'Checking daemon', detail: 'Reading infrastructure health only.' };
    daemon = await readDaemonHealth();
  }

  onMount(() => {
    shellMode = '__TAURI_INTERNALS__' in window ? 'native desktop' : 'browser preview';
    void refreshDaemon();
  });
</script>

<svelte:head>
  <title>Focusa Desktop</title>
</svelte:head>

<div class="desktop-shell" class:tui-mode={uiMode === 'tui'}>
  <header class="titlebar">
    <div class="brand-lockup" aria-label="Focusa Desktop">
      <span class="brand-mark" aria-hidden="true">F</span>
      <div>
        <strong>Focusa Desktop</strong>
        <span>Workstream workspace</span>
      </div>
    </div>
    <div class="titlebar-status">
      <div class="ui-mode-switch" aria-label="Application interface mode">
        <span class:active={uiMode === 'tui'}>Agent TUI (Pi)</span>
        <button
          type="button"
          role="switch"
          aria-checked={uiMode === 'canvas'}
          aria-label="Switch between TUI and Mission Canvas"
          onclick={() => (uiMode = uiMode === 'canvas' ? 'tui' : 'canvas')}
        ><i class:canvas={uiMode === 'canvas'}></i></button>
        <span class:active={uiMode === 'canvas'}>Mission Canvas</span>
      </div>
      <span class="mode-chip">{shellMode}</span>
      <span class:unavailable={daemon.kind === 'unavailable'} class:connected={daemon.kind === 'read-only'} class="daemon-chip">
        <span class="status-dot" aria-hidden="true"></span>{daemon.label}
      </span>
    </div>
  </header>

  {#if uiMode === 'canvas'}
  <aside class="sidebar" aria-label="Focusa workspaces">
    <div class="scope-card">
      <span class="eyebrow">Context Control</span>
      <strong>Unbound</strong>
      <p>No Scope, Workstream, or Attachment selected.</p>
    </div>
    <nav>
      {#each FOCUSA_DESKTOP_WORKSPACES as workspace}
        <button
          type="button"
          class:active={workspace.id === activeWorkspaceId}
          aria-current={workspace.id === activeWorkspaceId ? 'page' : undefined}
          onclick={() => (activeWorkspaceId = workspace.id)}
        >
          <span>{workspace.shortLabel}</span>
          {#if workspace.availability === 'planned'}
            <small>M{workspace.milestone}</small>
          {:else}
            <small>live</small>
          {/if}
        </button>
      {/each}
    </nav>
  </aside>
  {/if}

  <main>
    {#if uiMode === 'tui'}
      <section class="tui-surface" aria-label="Focusa terminal compatibility projection">
        <div class="tui-bar">AGENT TUI (PI) · NATIVE WORK SURFACE</div>
        <pre><span class="tui-accent">ACTIVE WORKSTREAM</span>  unbound
<span class="tui-dim">SCOPE</span>              no ScopeRef selected
<span class="tui-dim">ATTACHMENT</span>         no Pi runtime attached

┌─ CURRENT MISSION ─────────────────────────────────────────────┐
│ Await exact Workstream and Attachment authority.             │
└───────────────────────────────────────────────────────────────┘

┌─ WORK RAIL ───────────────────┬─ EVIDENCE ────────────────────┐
│ No active Workpoint           │ No scoped Evidence            │
│ Writer: unavailable           │ Proof posture: unavailable    │
└───────────────────────────────┴───────────────────────────────┘

<span class="tui-good">Focusa Desktop is connected read-only.</span>
<span class="tui-dim">Use the app-wide switch to replace this entire inner surface with Mission Canvas.</span></pre>
      </section>
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
            <span>ScopeRef</span><i>→</i><span>WorkstreamId</span><i>→</i><span>ContinuityId</span><i>→</i><span>AttachmentKey</span>
          </div>
        </article>
        <article class="status-card">
          <div class="card-heading">
            <div>
              <span class="eyebrow">Infrastructure</span>
              <h2>{daemon.label}</h2>
            </div>
            <span class="status-orb {daemon.kind}" aria-hidden="true"></span>
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
            <li>Infrastructure health only</li>
            <li>No canonical cognitive cache</li>
            <li>No implicit active Workstream</li>
            <li>No domain mutation</li>
          </ul>
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
    {:else}
      <section class="empty-state planned-state">
        <span class="eyebrow">Truthful milestone boundary</span>
        <h2>{activeWorkspace.label} is not implemented at 5%</h2>
        <p>
          Navigation is active, but this workspace will remain visibly unavailable until its typed
          Workstream contracts and parity evidence are complete.
        </p>
      </section>
    {/if}
    {/if}
  </main>

  <footer>
    <span>Focusa Desktop 0.9.143</span>
    <span>UIAI Engine browser proof</span>
    <span>No canonical state duplicated</span>
  </footer>
</div>
