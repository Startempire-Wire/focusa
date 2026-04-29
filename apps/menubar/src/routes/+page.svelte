<script lang="ts">
  import { fetchJson } from '$lib/api';
  import { focusStore } from '$lib/stores/focus.svelte';
  import { gateStore } from '$lib/stores/gate.svelte';
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import FocusView from '$lib/components/FocusView.svelte';
  import GatePanel from '$lib/components/GatePanel.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { onMount } from 'svelte';

  import SyncPanel from '$lib/components/SyncPanel.svelte';

  type Tab = 'focus' | 'gate' | 'sync' | 'settings';
  let activeTab = $state<Tab>('focus');

  let pollTimer: ReturnType<typeof setInterval> | undefined;

  async function poll() {
    try {
      const [state, health, contracts, workpoint, workLoop, events] = await Promise.all([
        fetchJson('/v1/state/dump', 5000),
        fetchJson('/v1/health'),
        fetchJson('/v1/ontology/tool-contracts'),
        fetchJson('/v1/workpoint/current'),
        fetchJson('/v1/work-loop/status'),
        fetchJson('/v1/events/recent?limit=5'),
      ]);
      focusStore.update(state);
      gateStore.update(state.focus_gate);
      runtimeStore.update({
        health,
        workpoint,
        workLoop,
        ontologyContractsVersion: contracts.version ?? null,
        ontologyContractsCount: Array.isArray(contracts.contracts) ? contracts.contracts.length : 0,
        recentEventCount: Array.isArray(events.events) ? events.events.length : 0,
      });
    } catch (e: any) {
      const msg = e?.message || 'Failed to connect';
      focusStore.setError(msg);
      runtimeStore.setError(msg);
    }
  }

  onMount(() => {
    focusStore.setConnecting();
    poll(); // immediate first poll
    pollTimer = setInterval(poll, 2000);
    return () => {
      if (pollTimer) clearInterval(pollTimer);
    };
  });
</script>

<!-- Header bar -->
<header class="header">
  <div class="header-left">
    <div class="status-dot" class:connected={focusStore.connected === 'connected'} class:error={focusStore.connected === 'error'}></div>
    <span class="header-title">Focusa</span>
  </div>
  <nav class="tabs">
    <button class="tab" class:active={activeTab === 'focus'} onclick={() => activeTab = 'focus'}>
      Focus
    </button>
    <button class="tab" class:active={activeTab === 'gate'} onclick={() => activeTab = 'gate'}>
      Gate
      {#if gateStore.surfacedCount > 0}
        <span class="badge">{gateStore.surfacedCount}</span>
      {/if}
    </button>
    <button class="tab" class:active={activeTab === 'sync'} onclick={() => activeTab = 'sync'}>
      Sync
    </button>
    <button class="tab" class:active={activeTab === 'settings'} onclick={() => activeTab = 'settings'}>
      ⚙
    </button>
  </nav>
</header>

<!-- Content -->
<main class="content">
  {#if activeTab === 'focus'}
    <FocusView />
  {:else if activeTab === 'gate'}
    <GatePanel />
  {:else if activeTab === 'sync'}
    <SyncPanel />
  {:else if activeTab === 'settings'}
    <Settings />
  {/if}
</main>

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
    gap: 2px;
  }

  .tab {
    font-family: var(--font);
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--fg-tertiary);
    background: none;
    border: none;
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--r-sm);
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tab:hover {
    color: var(--fg-secondary);
    background: var(--bg-hover);
  }

  .tab.active {
    color: var(--fg);
    background: var(--bg-elevated);
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
