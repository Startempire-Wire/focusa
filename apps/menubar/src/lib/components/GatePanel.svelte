<!--
  GatePanel — small vertical panel showing surfaced candidates.
  Pressure as opacity. Pin/suppress actions only.
  No "switch focus" button.
-->
<script lang="ts">
  import { gateStore } from '$lib/stores/gate';

  let { onclose }: { onclose: () => void } = $props();
  let candidates = $derived(gateStore.candidates);
</script>

<div class="gate-panel">
  <div class="gate-header">
    <span class="gate-title">Gate Candidates</span>
    <button class="close-btn" onclick={onclose}>×</button>
  </div>

  {#if candidates.length === 0}
    <div class="empty">No candidates</div>
  {:else}
    <div class="candidate-list">
      {#each candidates as candidate}
        <div
          class="candidate"
          class:surfaced={candidate.status === 'Surfaced'}
          class:pinned={candidate.pinned}
          style="--pressure-opacity: {Math.max(0.3, candidate.pressure)}"
        >
          <div class="candidate-header">
            <span class="candidate-kind">{candidate.kind}</span>
            {#if candidate.pinned}
              <span class="pin">📌</span>
            {/if}
          </div>
          <div class="candidate-label">{candidate.label}</div>
          <div class="candidate-pressure">
            <div class="pressure-bar" style="width: {candidate.pressure * 100}%"></div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .gate-panel {
    width: 260px;
    max-height: 320px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-panel);
    overflow-y: auto;
    animation: slideUp var(--transition-fade) forwards;
  }

  .gate-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .gate-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--fg-dim);
    cursor: pointer;
    font-size: 1.2rem;
  }

  .empty {
    padding: var(--space-md);
    color: var(--fg-muted);
    font-size: var(--font-size-sm);
    text-align: center;
  }

  .candidate-list {
    padding: var(--space-xs);
  }

  .candidate {
    padding: var(--space-sm);
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-xs);
    opacity: var(--pressure-opacity);
    transition: opacity var(--transition-gentle);
  }

  .candidate.surfaced {
    background: var(--active-glow);
  }

  .candidate-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .candidate-kind {
    font-size: var(--font-size-sm);
    color: var(--accent);
    font-family: var(--font-mono);
  }

  .pin { font-size: 0.7rem; }

  .candidate-label {
    font-size: var(--font-size-sm);
    color: var(--fg);
    margin-top: 2px;
  }

  .candidate-pressure {
    margin-top: 4px;
    height: 2px;
    background: var(--border);
    border-radius: 1px;
    overflow: hidden;
  }

  .pressure-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 1px;
    transition: width var(--transition-gentle);
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
