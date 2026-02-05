<!--
  GatePanel — full-tab view of gate candidates and recent signals.
-->
<script lang="ts">
  import { gateStore } from '$lib/stores/gate.svelte';

  let candidates = $derived(gateStore.candidates);
  let signals = $derived(gateStore.signals);
</script>

<div class="gate-view">
  {#if candidates.length === 0 && signals.length === 0}
    <div class="empty-state">
      <div class="empty-icon">◇</div>
      <div class="empty-title">Gate is Quiet</div>
      <div class="empty-desc">
        No candidates are pressing for attention. The intuition engine has no recent signals.
      </div>
      <div class="empty-hint">
        Candidates surface when the daemon detects context switches, stalls, or priority changes.
      </div>
    </div>
  {:else}
    {#if candidates.length > 0}
      <section class="section">
        <div class="section-label">CANDIDATES ({candidates.length})</div>
        {#each candidates as c}
          <div class="candidate" style="--pressure: {Math.max(0.3, c.pressure)}">
            <div class="candidate-top">
              <span class="candidate-kind">{c.kind}</span>
              <div class="candidate-right">
                {#if c.pinned}<span class="pin" title="Pinned">📌</span>{/if}
                <span class="pressure-label">{Math.round(c.pressure * 100)}%</span>
              </div>
            </div>
            <div class="candidate-label">{c.label}</div>
            <div class="pressure-track">
              <div class="pressure-fill" style="width: {c.pressure * 100}%"></div>
            </div>
          </div>
        {/each}
      </section>
    {/if}

    {#if signals.length > 0}
      <section class="section">
        <div class="section-label">RECENT SIGNALS ({signals.length})</div>
        <div class="signal-list">
          {#each signals.slice(-10).reverse() as s}
            <div class="signal">
              <span class="signal-kind">{s.kind}</span>
              <span class="signal-summary">{s.summary}</span>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .gate-view {
    padding: var(--sp-3);
  }

  /* Empty */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: var(--sp-6) var(--sp-4);
    min-height: 300px;
    justify-content: center;
  }

  .empty-icon {
    font-size: 36px;
    color: var(--fg-tertiary);
    margin-bottom: var(--sp-3);
  }

  .empty-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--fg);
    margin-bottom: var(--sp-1);
  }

  .empty-desc {
    font-size: var(--text-sm);
    color: var(--fg-secondary);
    max-width: 260px;
    line-height: 1.5;
    margin-bottom: var(--sp-3);
  }

  .empty-hint {
    font-size: var(--text-xs);
    color: var(--fg-tertiary);
    max-width: 240px;
    line-height: 1.5;
  }

  /* Sections */
  .section { margin-bottom: var(--sp-4); }

  .section-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--fg-tertiary);
    letter-spacing: 0.8px;
    margin-bottom: var(--sp-2);
  }

  /* Candidates */
  .candidate {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: var(--sp-2) var(--sp-3);
    margin-bottom: var(--sp-2);
    opacity: var(--pressure);
    transition: opacity var(--dur-normal) var(--ease);
  }

  .candidate:hover { opacity: 1; }

  .candidate-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .candidate-kind {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--accent);
    font-weight: 500;
  }

  .candidate-right {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .pin { font-size: 11px; }

  .pressure-label {
    font-size: 10px;
    color: var(--fg-tertiary);
    font-family: var(--font-mono);
  }

  .candidate-label {
    font-size: var(--text-sm);
    color: var(--fg);
    margin-top: 2px;
  }

  .pressure-track {
    height: 2px;
    background: var(--border);
    border-radius: 1px;
    margin-top: var(--sp-2);
    overflow: hidden;
  }

  .pressure-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 1px;
    transition: width var(--dur-normal) var(--ease);
  }

  /* Signals */
  .signal-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .signal {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--r-sm);
    font-size: var(--text-xs);
  }

  .signal:hover { background: var(--bg-hover); }

  .signal-kind {
    font-family: var(--font-mono);
    color: var(--fg-secondary);
  }

  .signal-summary {
    color: var(--fg-tertiary);
    font-size: 10px;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 180px;
  }
</style>
