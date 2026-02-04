<script lang="ts">
  import FocusBubble from '$lib/components/FocusBubble.svelte';
  import FocusStackView from '$lib/components/FocusStackView.svelte';
  import IntuitionPulse from '$lib/components/IntuitionPulse.svelte';
  import GatePanel from '$lib/components/GatePanel.svelte';
  import ReferencePeek from '$lib/components/ReferencePeek.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { focusStore } from '$lib/stores/focus';
  import { gateStore } from '$lib/stores/gate';
  import { intuitionStore } from '$lib/stores/intuition';
  import { onMount } from 'svelte';

  let showGate = $state(false);
  let showSettings = $state(false);

  onMount(() => {
    // Poll Focusa API every 2 seconds using saved URL.
    const getApiUrl = () =>
      localStorage.getItem('focusa_api_url') || 'http://127.0.0.1:8787';

    const interval = setInterval(async () => {
      try {
        const base = getApiUrl();
        const resp = await fetch(`${base}/v1/state/dump`, {
          signal: AbortSignal.timeout(3000),
        });
        if (resp.ok) {
          const data = await resp.json();
          focusStore.update(data);
          gateStore.update(data.focus_gate);
          intuitionStore.update(data);
        } else {
          focusStore.disconnect();
        }
      } catch {
        focusStore.disconnect();
      }
    }, 2000);
    return () => clearInterval(interval);
  });
</script>

<div class="menubar-view">
  <!-- Central focus bubble -->
  <div class="focus-area">
    <FocusBubble />
    <IntuitionPulse />
  </div>

  <!-- Background thought clouds (inactive frames) -->
  <div class="background-clouds">
    <FocusStackView />
  </div>

  <!-- Gate panel (click to open) -->
  <button class="gate-trigger" onclick={() => showGate = !showGate}>
    {#if gateStore.surfacedCount > 0}
      <span class="gate-dot"></span>
    {/if}
  </button>

  {#if showGate}
    <div class="gate-overlay">
      <GatePanel onclose={() => showGate = false} />
    </div>
  {/if}

  <!-- Reference peek (on hover) -->
  <ReferencePeek />

  <!-- Settings -->
  <button class="settings-trigger" onclick={() => showSettings = !showSettings}>⚙</button>

  {#if showSettings}
    <div class="settings-overlay">
      <Settings onclose={() => showSettings = false} />
    </div>
  {/if}
</div>

<style>
  .menubar-view {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .focus-area {
    position: relative;
    z-index: 2;
  }

  .background-clouds {
    position: absolute;
    inset: 0;
    z-index: 1;
    pointer-events: none;
  }

  .gate-trigger {
    position: absolute;
    bottom: var(--space-md);
    right: var(--space-md);
    width: 24px;
    height: 24px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border);
    background: var(--bg-panel);
    cursor: pointer;
    z-index: 3;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .gate-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--accent);
    animation: gentle-pulse var(--transition-drift) infinite;
  }

  @keyframes gentle-pulse {
    0%, 100% { opacity: 0.6; transform: scale(1); }
    50% { opacity: 1; transform: scale(1.2); }
  }

  .gate-overlay {
    position: absolute;
    bottom: 48px;
    right: var(--space-md);
    z-index: 10;
  }

  .settings-trigger {
    position: absolute;
    top: var(--space-sm);
    right: var(--space-sm);
    background: none;
    border: none;
    color: var(--fg-muted);
    cursor: pointer;
    font-size: 0.9rem;
    z-index: 3;
    opacity: 0.4;
    transition: opacity var(--transition-gentle);
  }

  .settings-trigger:hover { opacity: 1; }

  .settings-overlay {
    position: absolute;
    top: 32px;
    right: var(--space-sm);
    z-index: 20;
  }
</style>
